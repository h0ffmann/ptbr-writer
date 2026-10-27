//! A stand-in for Ollama's /api/chat on 127.0.0.1:PORT (first argument), for the screenshot build.

fn main() {
    let port = std::env::args().nth(1).unwrap_or_else(|| "11999".into());
    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{port}")).expect("bind");
    eprintln!("fake-provider: http://127.0.0.1:{port}");
    ptbr_app::fake::serve(listener);
}
