//! A stand-in for Ollama's /api/chat, for the screenshot build and the tests: every check gets the
//! same two suggestions on `DEMO_TEXT`. No request leaves the machine.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

pub const DEMO_TEXT: &str = "Foi decidido que o deploy vai ser na sexta. A gente vai fazer a migração do banco de dados de forma a garantir que nenhum dado se perca no processo.";

/// The model's answer: an erro in the first sentence, the second rewritten (aviso).
pub fn reply() -> String {
    let suggestions = serde_json::json!([
        { "original": "Foi decidido", "replacement": "A equipe decidiu", "rule": "PTW-1", "severity": "erro" },
        {
            "original": "A gente vai fazer a migração do banco de dados de forma a garantir que nenhum dado se perca no processo.",
            "replacement": "A equipe migra o banco de dados sem perder nenhum dado.",
            "rule": "PTW-17",
            "severity": "aviso"
        }
    ]);
    serde_json::json!({ "message": { "role": "assistant", "content": suggestions.to_string() }, "done_reason": "stop" })
        .to_string()
}

/// Answers each connection, one at a time, until the process ends.
pub fn serve(listener: TcpListener) {
    for stream in listener.incoming().flatten() {
        let mut r = BufReader::new(&stream);
        let mut len = 0usize;
        let mut line = String::new();
        let mut first = String::new();
        while r.read_line(&mut line).is_ok_and(|n| n > 0) {
            if first.is_empty() {
                first = line.clone();
            }
            if line == "\r\n" {
                break;
            }
            if let Some((k, v)) = line.split_once(':')
                && k.eq_ignore_ascii_case("content-length")
            {
                len = v.trim().parse().unwrap_or(0);
            }
            line.clear();
        }
        let mut body = vec![0; len];
        let _ = r.read_exact(&mut body);
        let (status, out) =
            if first.starts_with("POST /api/chat ") { ("200 OK", reply()) } else { ("404 Not Found", "{}".into()) };
        let _ = write!(
            &stream,
            "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{out}",
            out.len()
        );
    }
}
