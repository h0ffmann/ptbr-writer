//! What the user typed as a site, as a hostname.

/// Turns what the user typed into a lowercase hostname, or `None` when it is not acceptable: a
/// wildcard, a port, a path alone, an IP address, a host without a dot, a scheme other than http(s).
pub fn normalize_host(input: &str) -> Option<String> {
    let text = input.trim();
    if text.is_empty() {
        return None;
    }
    let rest = match text.split_once("://") {
        Some((scheme, rest)) if is_scheme(scheme) => {
            if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
                return None;
            }
            rest
        }
        _ => text,
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains(['*', ':', '@']) || authority.chars().any(char::is_whitespace) {
        return None;
    }
    let host = authority.to_ascii_lowercase();
    // shortcut: the URL parser's host validation reduced to its ASCII alphabet (no IDNA, no
    // percent-decoding); add the `idna` crate when a user needs a non-ASCII domain.
    if !host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_')) {
        return None;
    }
    if !host.contains('.') || host.ends_with('.') || host.starts_with('[') {
        return None;
    }
    // An IPv4 address, or what the URL parser reads as one: a last label of digits.
    if host.rsplit('.').next().is_some_and(|last| last.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    Some(host)
}

fn is_scheme(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_hosts_urls_and_mixed_case() {
        assert_eq!(normalize_host("example.com").as_deref(), Some("example.com"));
        assert_eq!(normalize_host("https://example.com/path?q=1").as_deref(), Some("example.com"));
        assert_eq!(normalize_host("  EXAMPLE.com ").as_deref(), Some("example.com"));
        assert_eq!(normalize_host("HTTP://sub.example.com#x").as_deref(), Some("sub.example.com"));
    }

    #[test]
    fn rejects_wildcard_port_path_only_ip_localhost_empty_other_schemes() {
        for bad in [
            "*.example.com",
            "example.com:8080",
            "https://example.com:443",
            "/path",
            "10.0.0.1",
            "[::1]",
            "localhost",
            "",
            "   ",
            "ftp://example.com",
            "a b.com",
            "user@example.com",
            "1.2.3",
            "exa<mple.com",
        ] {
            assert_eq!(normalize_host(bad), None, "{bad}");
        }
    }
}
