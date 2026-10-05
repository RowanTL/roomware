//! Tiny HTTPS server so something outside the game master window can start the intro video.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix_web::cookie::{Cookie, SameSite, time};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use relm4::Sender;

use crate::player::PlayerMsg;

mod cert;

const PORT: u16 = 8080;
const PASSCODE: &str = "1717";
const COOKIE_NAME: &str = "roomware_session";
const SESSION_LIFETIME: Duration = Duration::from_secs(2 * 60 * 60); // 2 Hours

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

/// Runs on its own thread because GTK owns the main one.
pub fn spawn(player: Sender<PlayerMsg>) {
    std::thread::spawn(move || {
        let player = web::Data::new(player);
        let sessions = web::Data::new(Sessions::new(SESSION_LIFETIME));

        let tls = match cert::server_config(&cert::dir()) {
            Ok(tls) => tls,
            Err(error) => {
                eprintln!("roomware: web server not started, no certificate: {error}");
                return;
            }
        };

        let result = actix_web::rt::System::new().block_on(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(player.clone())
                    .app_data(sessions.clone())
                    .configure(routes)
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

fn routes(config: &mut web::ServiceConfig) {
    config
        .route("/login", web::post().to(login))
        .route("/play-video", web::post().to(play_video));
}

/// The body is the passcode; a correct one is answered with a session cookie.
async fn login(body: String, sessions: web::Data<Sessions>) -> HttpResponse {
    if body.trim() != PASSCODE {
        return HttpResponse::Unauthorized().finish();
    }

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
        let (sender, receiver) = relm4::channel();
        let app = App::new()
            .app_data(web::Data::new(sender))
            .app_data(web::Data::new(Sessions::new(lifetime)))
            .configure(routes);

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

    #[test]
    fn starting_a_session_forgets_expired_ones() {
        let sessions = Sessions::new(Duration::ZERO);

        sessions.start();
        sessions.start();

        assert_eq!(sessions.expiries.lock().unwrap().len(), 1);
    }
}
