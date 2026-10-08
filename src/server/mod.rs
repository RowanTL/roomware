//! Tiny HTTPS server so something outside the game master window can start the intro video.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix_files::Files;
use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::http::header;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use relm4::Sender;

use crate::player::PlayerMsg;
use crate::rooms;

mod cert;

const PORT: u16 = 8080;
const PASSCODE: &str = "1717";
const COOKIE_NAME: &str = "roomware_session";
const SESSION_LIFETIME: Duration = Duration::from_secs(2 * 60 * 60); // 2 Hours
const MAX_FAILED_LOGINS: u32 = 5;
const LOGIN_WINDOW: Duration = Duration::from_secs(15 * 60); // 15 Minutes

struct Sessions {
    lifetime: Duration,
    expiries: Mutex<HashMap<String, Instant>>,
}

impl Sessions {
    fn new(lifetime: Duration) -> Self {
        Self {
            lifetime,
            expiries: Mutex::default(),
        }
    }

    fn start(&self) -> String {
        let token = format!("{:032x}", rand::random::<u128>());
        let now = Instant::now();

        let mut expiries = self.expiries.lock().unwrap();
        expiries.retain(|_, expiry| *expiry > now);
        expiries.insert(token.clone(), now + self.lifetime);

        token
    }

    fn is_valid(&self, token: &str) -> bool {
        let expiries = self.expiries.lock().unwrap();
        expiries
            .get(token)
            .is_some_and(|expiry| *expiry > Instant::now())
    }
}

struct Failures {
    count: u32,
    since: Instant,
}

/// Locks an address out of logging in once it has failed too often.
struct LoginLimiter {
    max_failures: u32,
    window: Duration,
    failures: Mutex<HashMap<IpAddr, Failures>>,
}

impl LoginLimiter {
    fn new(max_failures: u32, window: Duration) -> Self {
        Self {
            max_failures,
            window,
            failures: Mutex::default(),
        }
    }

    /// How long `ip` must wait before it may try again, if it is locked out.
    fn locked_out_for(&self, ip: IpAddr) -> Option<Duration> {
        let failures = self.failures.lock().unwrap();
        let failed = failures.get(&ip).filter(|f| f.count >= self.max_failures)?;

        (failed.since + self.window).checked_duration_since(Instant::now())
    }

    fn record_failure(&self, ip: IpAddr) {
        let now = Instant::now();

        let mut failures = self.failures.lock().unwrap();
        failures.retain(|_, f| f.since + self.window > now);
        failures
            .entry(ip)
            .or_insert(Failures {
                count: 0,
                since: now,
            })
            .count += 1;
    }

    fn clear(&self, ip: IpAddr) {
        self.failures.lock().unwrap().remove(&ip);
    }
}

/// Runs on its own thread because GTK owns the main one.
pub fn spawn(player: Sender<PlayerMsg>) {
    std::thread::spawn(move || {
        let player = web::Data::new(player);
        let sessions = web::Data::new(Sessions::new(SESSION_LIFETIME));
        let limiter = web::Data::new(LoginLimiter::new(MAX_FAILED_LOGINS, LOGIN_WINDOW));

        let tls = match cert::server_config(&cert::dir()) {
            Ok(tls) => tls,
            Err(error) => {
                eprintln!("roomware: web server not started, no certificate: {error}");
                return;
            }
        };

        let web_dir = web_dir();
        if web_dir.is_none() {
            eprintln!("roomware: no web directory found, the control page is not served");
        }

        let result = actix_web::rt::System::new().block_on(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(player.clone())
                    .app_data(sessions.clone())
                    .app_data(limiter.clone())
                    .configure(routes)
                    // Last, since it matches every path the routes above didn't.
                    .configure(|config| {
                        if let Some(dir) = &web_dir {
                            config.service(static_files(dir));
                        }
                    })
            })
            .disable_signals()
            .bind_rustls_0_23(("0.0.0.0", PORT), tls)?
            .run()
            .await
        });

        if let Err(error) = result {
            eprintln!("roomware: web server stopped: {error}");
        }
    });
}

/// The installed one beside `bin`, or `web` in the working directory when run from the repo.
fn web_dir() -> Option<PathBuf> {
    [
        rooms::installed_dir().with_file_name("web"),
        PathBuf::from("web"),
    ]
    .into_iter()
    .find(|dir| dir.is_dir())
}

fn static_files(dir: &Path) -> Files {
    Files::new("/", dir).index_file("index.html")
}

fn routes(config: &mut web::ServiceConfig) {
    config
        .route("/login", web::post().to(login))
        .route("/play-video", web::post().to(play_video));
}

/// The body is the passcode; a correct one is answered with a session cookie.
async fn login(
    request: HttpRequest,
    body: String,
    sessions: web::Data<Sessions>,
    limiter: web::Data<LoginLimiter>,
) -> HttpResponse {
    let ip = request
        .peer_addr()
        .map_or(Ipv4Addr::UNSPECIFIED.into(), |addr| addr.ip());

    if let Some(wait) = limiter.locked_out_for(ip) {
        return HttpResponse::TooManyRequests()
            .insert_header((header::RETRY_AFTER, (wait.as_secs() + 1).to_string()))
            .finish();
    }

    if body.trim() != PASSCODE {
        limiter.record_failure(ip);
        return HttpResponse::Unauthorized().finish();
    }
    limiter.clear(ip);

    let max_age = time::Duration::try_from(sessions.lifetime).unwrap_or(time::Duration::MAX);

    HttpResponse::NoContent()
        .cookie(
            Cookie::build(COOKIE_NAME, sessions.start())
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .max_age(max_age)
                .finish(),
        )
        .finish()
}

async fn play_video(
    request: HttpRequest,
    sessions: web::Data<Sessions>,
    player: web::Data<Sender<PlayerMsg>>,
) -> HttpResponse {
    let logged_in = request
        .cookie(COOKIE_NAME)
        .is_some_and(|cookie| sessions.is_valid(cookie.value()));
    if !logged_in {
        return HttpResponse::Unauthorized().finish();
    }

    player.emit(PlayerMsg::PlayVideo);
    HttpResponse::NoContent().finish()
}

#[cfg(test)]
mod tests {
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::http::StatusCode;
    use actix_web::test::{self, TestRequest};

    use super::*;

    async fn service() -> (
        impl Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
        relm4::Receiver<PlayerMsg>,
    ) {
        service_with_lifetime(SESSION_LIFETIME).await
    }

    async fn service_with_lifetime(
        lifetime: Duration,
    ) -> (
        impl Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
        relm4::Receiver<PlayerMsg>,
    ) {
        let limiter = LoginLimiter::new(MAX_FAILED_LOGINS, LOGIN_WINDOW);
        service_with(lifetime, limiter).await
    }

    async fn service_with(
        lifetime: Duration,
        limiter: LoginLimiter,
    ) -> (
        impl Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
        relm4::Receiver<PlayerMsg>,
    ) {
        let (sender, receiver) = relm4::channel();
        let app = App::new()
            .app_data(web::Data::new(sender))
            .app_data(web::Data::new(Sessions::new(lifetime)))
            .app_data(web::Data::new(limiter))
            .configure(routes)
            .service(static_files(Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/web"
            ))));

        (test::init_service(app).await, receiver)
    }

    /// Dropping the service drops every sender, so this ends instead of waiting for more.
    /// Responses keep the app data alive too, so callers must not hold one across this.
    fn emitted(service: impl Sized, receiver: relm4::Receiver<PlayerMsg>) -> Vec<PlayerMsg> {
        drop(service);
        std::iter::from_fn(|| receiver.recv_sync()).collect()
    }

    fn login_request(passcode: &'static str) -> TestRequest {
        TestRequest::post().uri("/login").set_payload(passcode)
    }

    fn login_request_from(ip: &str, passcode: &'static str) -> TestRequest {
        login_request(passcode).peer_addr(format!("{ip}:1234").parse().unwrap())
    }

    async fn fail_logins_from(
        service: &impl Service<
            actix_http::Request,
            Response = ServiceResponse,
            Error = actix_web::Error,
        >,
        ip: &str,
        times: u32,
    ) {
        for _ in 0..times {
            let request = login_request_from(ip, "0000").to_request();
            let response = test::call_service(service, request).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    fn play_request() -> TestRequest {
        TestRequest::post().uri("/play-video")
    }

    async fn logged_in_cookie(
        service: &impl Service<
            actix_http::Request,
            Response = ServiceResponse,
            Error = actix_web::Error,
        >,
    ) -> Cookie<'static> {
        let response = test::call_service(service, login_request(PASSCODE).to_request()).await;
        response.response().cookies().next().unwrap().into_owned()
    }

    #[actix_web::test]
    async fn the_control_page_is_served_at_the_root() {
        let (service, _receiver) = service().await;

        let request = TestRequest::get().uri("/").to_request();
        let response = test::call_service(&service, request).await;

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(content_type.to_str().unwrap().starts_with("text/html"));
    }

    #[actix_web::test]
    async fn a_missing_file_is_not_found() {
        let (service, _receiver) = service().await;

        let request = TestRequest::get().uri("/nope.html").to_request();
        let response = test::call_service(&service, request).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn play_without_a_cookie_is_unauthorized() {
        let (service, receiver) = service().await;

        let status = test::call_service(&service, play_request().to_request())
            .await
            .status();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(emitted(service, receiver).is_empty());
    }

    #[actix_web::test]
    async fn play_with_a_forged_cookie_is_unauthorized() {
        let (service, receiver) = service().await;
        let forged = Cookie::new(COOKIE_NAME, "abc");

        let status = test::call_service(&service, play_request().cookie(forged).to_request())
            .await
            .status();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(emitted(service, receiver).is_empty());
    }

    #[actix_web::test]
    async fn wrong_passcode_is_unauthorized_and_sets_no_cookie() {
        let (service, _receiver) = service().await;

        let response = test::call_service(&service, login_request("0000").to_request()).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.response().cookies().next().is_none());
    }

    #[actix_web::test]
    async fn correct_passcode_sets_a_session_cookie() {
        let (service, _receiver) = service().await;

        let response = test::call_service(&service, login_request(PASSCODE).to_request()).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let cookie = response.response().cookies().next().unwrap();
        assert_eq!(cookie.name(), COOKIE_NAME);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Strict));
        assert!(!cookie.value().is_empty());
    }

    #[actix_web::test]
    async fn passcode_is_trimmed() {
        let (service, _receiver) = service().await;

        let response = test::call_service(&service, login_request("1717\n").to_request()).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn every_login_gets_its_own_token() {
        let (service, _receiver) = service().await;

        let first = logged_in_cookie(&service).await;
        let second = logged_in_cookie(&service).await;

        assert_ne!(first.value(), second.value());
    }

    #[actix_web::test]
    async fn play_after_login_tells_the_player_to_play() {
        let (service, receiver) = service().await;
        let cookie = logged_in_cookie(&service).await;

        let status = test::call_service(&service, play_request().cookie(cookie).to_request())
            .await
            .status();

        assert_eq!(status, StatusCode::NO_CONTENT);
        let messages = emitted(service, receiver);
        assert!(matches!(messages.as_slice(), [PlayerMsg::PlayVideo]));
    }

    #[actix_web::test]
    async fn every_logged_in_cookie_keeps_working() {
        let (service, receiver) = service().await;
        let first = logged_in_cookie(&service).await;
        let second = logged_in_cookie(&service).await;

        for cookie in [first, second] {
            let response =
                test::call_service(&service, play_request().cookie(cookie).to_request()).await;
            assert_eq!(response.status(), StatusCode::NO_CONTENT);
        }

        assert_eq!(emitted(service, receiver).len(), 2);
    }

    #[actix_web::test]
    async fn cookie_max_age_is_the_session_lifetime() {
        let (service, _receiver) = service().await;

        let cookie = logged_in_cookie(&service).await;

        let lifetime = time::Duration::try_from(SESSION_LIFETIME).unwrap();
        assert_eq!(cookie.max_age(), Some(lifetime));
    }

    #[actix_web::test]
    async fn play_with_an_expired_cookie_is_unauthorized() {
        let (service, receiver) = service_with_lifetime(Duration::ZERO).await;
        let cookie = logged_in_cookie(&service).await;

        let status = test::call_service(&service, play_request().cookie(cookie).to_request())
            .await
            .status();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(emitted(service, receiver).is_empty());
    }

    #[actix_web::test]
    async fn repeated_wrong_passcodes_lock_the_address_out() {
        let (service, _receiver) = service().await;
        fail_logins_from(&service, "10.0.0.1", MAX_FAILED_LOGINS).await;

        let request = login_request_from("10.0.0.1", "0000").to_request();
        let response = test::call_service(&service, request).await;

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key(header::RETRY_AFTER));
    }

    #[actix_web::test]
    async fn the_correct_passcode_is_refused_while_locked_out() {
        let (service, _receiver) = service().await;
        fail_logins_from(&service, "10.0.0.1", MAX_FAILED_LOGINS).await;

        let request = login_request_from("10.0.0.1", PASSCODE).to_request();
        let response = test::call_service(&service, request).await;

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.response().cookies().next().is_none());
    }

    #[actix_web::test]
    async fn a_lockout_does_not_affect_other_addresses() {
        let (service, _receiver) = service().await;
        fail_logins_from(&service, "10.0.0.1", MAX_FAILED_LOGINS).await;

        let request = login_request_from("10.0.0.2", PASSCODE).to_request();
        let response = test::call_service(&service, request).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[actix_web::test]
    async fn a_correct_passcode_resets_the_failure_count() {
        let (service, _receiver) = service().await;
        fail_logins_from(&service, "10.0.0.1", MAX_FAILED_LOGINS - 1).await;
        let request = login_request_from("10.0.0.1", PASSCODE).to_request();
        test::call_service(&service, request).await;

        fail_logins_from(&service, "10.0.0.1", MAX_FAILED_LOGINS - 1).await;
    }

    #[actix_web::test]
    async fn a_lockout_ends_when_the_window_does() {
        let limiter = LoginLimiter::new(MAX_FAILED_LOGINS, Duration::ZERO);
        let (service, _receiver) = service_with(SESSION_LIFETIME, limiter).await;
        fail_logins_from(&service, "10.0.0.1", MAX_FAILED_LOGINS + 1).await;

        let request = login_request_from("10.0.0.1", PASSCODE).to_request();
        let response = test::call_service(&service, request).await;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn starting_a_session_forgets_expired_ones() {
        let sessions = Sessions::new(Duration::ZERO);

        sessions.start();
        sessions.start();

        assert_eq!(sessions.expiries.lock().unwrap().len(), 1);
    }
}
