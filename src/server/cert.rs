//! Self-signed certificate for the HTTPS server, kept on disk so browsers only warn once.

use std::fs;
use std::io::Write;
use std::net::{IpAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustls::ServerConfig;
use rustls::crypto::ring::default_provider;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::rooms;

const CERT_FILE: &str = "cert.pem";
const KEY_FILE: &str = "key.pem";

type Error = Box<dyn std::error::Error>;

pub fn dir() -> PathBuf {
    rooms::installed_dir().with_file_name("certs")
}

/// Loads saved certificate from `dir`, or makes and saves a new one if
/// there is none or can't be used.
pub fn server_config(dir: &Path) -> Result<ServerConfig, Error> {
    if let Some((cert_pem, key_pem)) = load(dir)
        && let Ok(config) = build(&cert_pem, &key_pem)
    {
        return Ok(config);
    }

    let (cert_pem, key_pem) = generate()?;
    if let Err(error) = save(dir, &cert_pem, &key_pem) {
        eprintln!(
            "roomware: could not save the certificate to {}, a new one is made next start: {error}",
            dir.display()
        );
    }

    build(&cert_pem, &key_pem)
}

fn build(cert_pem: &str, key_pem: &str) -> Result<ServerConfig, Error> {
    let certs = CertificateDer::pem_slice_iter(cert_pem.as_bytes()).collect::<Result<_, _>>()?;
    let key = PrivateKeyDer::from_pem_slice(key_pem.as_bytes())?;

    let config = ServerConfig::builder_with_provider(Arc::new(default_provider()))
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    Ok(config)
}

fn load(dir: &Path) -> Option<(String, String)> {
    Some((
        fs::read_to_string(dir.join(CERT_FILE)).ok()?,
        fs::read_to_string(dir.join(KEY_FILE)).ok()?,
    ))
}

fn generate() -> Result<(String, String), Error> {
    let mut names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    names.extend(local_ip().map(|ip| ip.to_string()));

    let rcgen::CertifiedKey { cert, signing_key } = rcgen::generate_simple_self_signed(names)?;

    Ok((cert.pem(), signing_key.serialize_pem()))
}

/// The address other machines on the network reach this one at.
fn local_ip() -> Option<IpAddr> {
    // Connecting a UDP socket only picks a route; nothing is sent.
    let socket = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    socket.connect(("8.8.8.8", 80)).ok()?;

    Some(socket.local_addr().ok()?.ip())
}

fn save(dir: &Path, cert_pem: &str, key_pem: &str) -> Result<(), Error> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join(CERT_FILE), cert_pem)?;
    write_private(&dir.join(KEY_FILE), key_pem)?;

    Ok(())
}

fn write_private(path: &Path, contents: &str) -> std::io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);

    options.open(path)?.write_all(contents.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("roomware-cert-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn first_start_saves_a_certificate() {
        let dir = temp_dir("saves");

        server_config(&dir).unwrap();

        assert!(dir.join(CERT_FILE).is_file());
        assert!(dir.join(KEY_FILE).is_file());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn later_starts_reuse_the_saved_certificate() {
        let dir = temp_dir("reuses");
        server_config(&dir).unwrap();
        let saved = load(&dir).unwrap();

        server_config(&dir).unwrap();

        assert_eq!(load(&dir).unwrap(), saved);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_broken_saved_certificate_is_replaced() {
        let dir = temp_dir("broken");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(CERT_FILE), "not a certificate").unwrap();
        fs::write(dir.join(KEY_FILE), "not a key").unwrap();

        server_config(&dir).unwrap();

        let (cert_pem, key_pem) = load(&dir).unwrap();
        assert!(build(&cert_pem, &key_pem).is_ok());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_unwritable_directory_still_serves_a_certificate() {
        let dir = temp_dir("unwritable");
        fs::create_dir_all(&dir).unwrap();
        // A file where the directory should be makes saving fail.
        let blocked = dir.join("certs");
        fs::write(&blocked, "").unwrap();

        assert!(server_config(&blocked).is_ok());
        fs::remove_dir_all(dir).unwrap();
    }
}
