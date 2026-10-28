//! The extension's `Fetch` on the desktop: reqwest over rustls. A request carries no Origin, so
//! Ollama and Ollaya answer without OLLAMA_ORIGINS or OLLAYA_ORIGINS.

use ptbr_ext::providers::{Abort, Failed, Fetch, HttpRequest, HttpResponse};
use std::time::Duration;

pub struct Http(reqwest::Client);

impl Default for Http {
    fn default() -> Self {
        // reqwest is built without a crypto provider (Cargo.toml): ring is the one, installed once.
        let _ = rustls::crypto::ring::default_provider().install_default();
        // A local model on a CPU can take minutes for a long text.
        let b = reqwest::Client::builder().timeout(Duration::from_secs(300));
        #[cfg(target_os = "android")]
        let b = b.tls_backend_preconfigured(android_tls());
        Http(b.build().expect("an HTTP client"))
    }
}

/// rustls over Mozilla's roots (webpki-roots): the platform verifier needs a JNI setup and a Kotlin
/// library that the overlay service, started without the window, would not have.
#[cfg(target_os = "android")]
fn android_tls() -> rustls::ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut c = rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
    c.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    c
}

impl Fetch for Http {
    /// The window runs one check at a time, so nothing aborts one: `abort` goes unused.
    async fn fetch(&self, req: HttpRequest, _abort: &Abort) -> Result<HttpResponse, Failed> {
        let mut b = self.0.post(&req.url).body(req.body);
        for (k, v) in req.headers {
            b = b.header(k, v);
        }
        let res = b.send().await.map_err(|_| Failed)?;
        let status = res.status().as_u16();
        let body = res.text().await.map_err(|_| Failed)?;
        Ok(HttpResponse { status, body })
    }
}
