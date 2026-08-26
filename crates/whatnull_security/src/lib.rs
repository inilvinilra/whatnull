use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use url::{Host, Url};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDecision {
    Allow,
    OpenExternally,
    Reject,
}

const INTERNAL_SCHEME_PREFIXES: [&str; 2] = ["blob:", "about:"];

const REJECTED_SCHEMES: [&str; 7] = [
    "file",
    "javascript",
    "data",
    "ftp",
    "ssh",
    "chrome",
    "devtools",
];

const ALLOWED_HOST_SUFFIXES: [&str; 4] =
    ["whatsapp.com", "whatsapp.net", "facebook.com", "fbcdn.net"];

fn is_internal_scheme(url_str: &str) -> bool {
    INTERNAL_SCHEME_PREFIXES
        .iter()
        .any(|prefix| url_str.starts_with(prefix))
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() {
        return true;
    }

    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => {
            let octets = v6.octets();
            let unique_local = (octets[0] & 0xfe) == 0xfc;
            let link_local = octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80;
            unique_local
                || link_local
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| is_blocked_ip(IpAddr::V4(mapped)))
        }
    }
}

fn is_allowed_host(host: &str) -> bool {
    ALLOWED_HOST_SUFFIXES.iter().any(|suffix| {
        host == *suffix
            || (host.len() > suffix.len()
                && host.ends_with(suffix)
                && host.as_bytes()[host.len() - suffix.len() - 1] == b'.')
    })
}

pub struct NavigationPolicy;

impl NavigationPolicy {
    pub fn evaluate(url_str: &str) -> NavigationDecision {
        if is_internal_scheme(url_str) {
            return NavigationDecision::Allow;
        }

        let parsed = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return NavigationDecision::Reject,
        };

        let scheme = parsed.scheme().to_lowercase();
        if REJECTED_SCHEMES.contains(&scheme.as_str()) {
            return NavigationDecision::Reject;
        }

        if scheme != "http" && scheme != "https" {
            return NavigationDecision::Reject;
        }

        let domain = match parsed.host() {
            Some(Host::Domain(domain)) => domain.to_lowercase(),
            Some(Host::Ipv4(addr)) => {
                return if is_blocked_ip(IpAddr::V4(addr)) {
                    NavigationDecision::Reject
                } else {
                    NavigationDecision::OpenExternally
                };
            }
            Some(Host::Ipv6(addr)) => {
                return if is_blocked_ip(IpAddr::V6(addr)) {
                    NavigationDecision::Reject
                } else {
                    NavigationDecision::OpenExternally
                };
            }
            None => return NavigationDecision::Reject,
        };

        if domain == "localhost" || domain.ends_with(".localhost") {
            return NavigationDecision::Reject;
        }

        if is_allowed_host(&domain) {
            return NavigationDecision::Allow;
        }

        NavigationDecision::OpenExternally
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whatsapp_domains_allowed() {
        assert_eq!(
            NavigationPolicy::evaluate("https://web.whatsapp.com"),
            NavigationDecision::Allow
        );
        assert_eq!(
            NavigationPolicy::evaluate("https://flows.whatsapp.net/flows/cache_management/"),
            NavigationDecision::Allow
        );
        assert_eq!(
            NavigationPolicy::evaluate("https://webtp.whatsapp.net/pdf-viewer/?locale=en_GB"),
            NavigationDecision::Allow
        );
        assert_eq!(
            NavigationPolicy::evaluate("blob:https://web.whatsapp.com/uuid"),
            NavigationDecision::Allow
        );
    }

    #[test]
    fn test_external_opened_externally() {
        assert_eq!(
            NavigationPolicy::evaluate("https://example.com"),
            NavigationDecision::OpenExternally
        );
        assert_eq!(
            NavigationPolicy::evaluate("https://google.com/search?q=test"),
            NavigationDecision::OpenExternally
        );
    }

    #[test]
    fn test_blocked_schemes_rejected() {
        assert_eq!(
            NavigationPolicy::evaluate("javascript:alert(1)"),
            NavigationDecision::Reject
        );
        assert_eq!(
            NavigationPolicy::evaluate("file:///etc/passwd"),
            NavigationDecision::Reject
        );
    }

    #[test]
    fn lookalike_hosts_are_not_treated_as_whatsapp() {
        assert_eq!(
            NavigationPolicy::evaluate("https://evilwhatsapp.com"),
            NavigationDecision::OpenExternally
        );
        assert_eq!(
            NavigationPolicy::evaluate("https://whatsapp.com.attacker.io"),
            NavigationDecision::OpenExternally
        );
        assert_eq!(
            NavigationPolicy::evaluate("https://web.whatsapp.com.attacker.io"),
            NavigationDecision::OpenExternally
        );
    }

    #[test]
    fn private_and_loopback_addresses_are_rejected() {
        for url in [
            "http://127.0.0.1:8080",
            "http://localhost/admin",
            "http://192.168.1.1",
            "http://10.0.0.5",
            "http://169.254.169.254/latest/meta-data",
            "http://[::1]:3000",
            "http://[fd00::1]/",
            "http://[fe80::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://[::]/",
            "http://0.0.0.0/",
            "http://app.localhost/",
        ] {
            assert_eq!(
                NavigationPolicy::evaluate(url),
                NavigationDecision::Reject,
                "expected {url} to be rejected"
            );
        }
    }
}
