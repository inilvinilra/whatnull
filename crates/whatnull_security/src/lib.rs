use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDecision {
    Allow,
    OpenExternally,
    Reject,
}

pub struct NavigationPolicy;

impl NavigationPolicy {
    pub fn evaluate(url_str: &str) -> NavigationDecision {
        let parsed = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return NavigationDecision::Reject,
        };

        let scheme = parsed.scheme().to_lowercase();
        if ["file", "javascript", "data", "blob", "ftp", "ssh", "about", "chrome", "devtools"].contains(&scheme.as_str()) {
            return NavigationDecision::Reject;
        }

        if scheme != "http" && scheme != "https" {
            return NavigationDecision::Reject;
        }

        let host = match parsed.host_str() {
            Some(h) => h.to_lowercase(),
            None => return NavigationDecision::Reject,
        };

        if host == "localhost" {
            return NavigationDecision::Reject;
        }

        if let Ok(ip) = host.parse::<IpAddr>() {
            if ip.is_loopback() || ip.is_unspecified() {
                return NavigationDecision::Reject;
            }
            match ip {
                IpAddr::V4(ipv4) => {
                    if ipv4.is_private() || ipv4.is_link_local() {
                        return NavigationDecision::Reject;
                    }
                }
                IpAddr::V6(ipv6) => {
                    let octets = ipv6.octets();
                    let is_unique_local = (octets[0] & 0xfe) == 0xfc;
                    let is_link_local = (octets[0] == 0xfe) && ((octets[1] & 0xc0) == 0x80);
                    if is_unique_local || is_link_local {
                        return NavigationDecision::Reject;
                    }
                }
            }
        }

        if host == "web.whatsapp.com" || host.ends_with(".web.whatsapp.com") {
            return NavigationDecision::Allow;
        }

        NavigationDecision::OpenExternally
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whatsapp_allowed() {
        assert_eq!(NavigationPolicy::evaluate("https://web.whatsapp.com"), NavigationDecision::Allow);
        assert_eq!(NavigationPolicy::evaluate("https://sub.web.whatsapp.com"), NavigationDecision::Allow);
        assert_eq!(NavigationPolicy::evaluate("http://web.whatsapp.com"), NavigationDecision::Allow);
    }

    #[test]
    fn test_external_opened_externally() {
        assert_eq!(NavigationPolicy::evaluate("https://example.com"), NavigationDecision::OpenExternally);
        assert_eq!(NavigationPolicy::evaluate("https://google.com/search?q=test"), NavigationDecision::OpenExternally);
    }

    #[test]
    fn test_blocked_schemes_rejected() {
        assert_eq!(NavigationPolicy::evaluate("javascript:alert(1)"), NavigationDecision::Reject);
        assert_eq!(NavigationPolicy::evaluate("file:///etc/passwd"), NavigationDecision::Reject);
        assert_eq!(NavigationPolicy::evaluate("data:text/html,test"), NavigationDecision::Reject);
        assert_eq!(NavigationPolicy::evaluate("blob:https://web.whatsapp.com/uuid"), NavigationDecision::Reject);
    }

    #[test]
    fn test_localhost_rejected() {
        assert_eq!(NavigationPolicy::evaluate("http://127.0.0.1"), NavigationDecision::Reject);
        assert_eq!(NavigationPolicy::evaluate("https://127.0.0.1"), NavigationDecision::Reject);
        assert_eq!(NavigationPolicy::evaluate("http://localhost"), NavigationDecision::Reject);
        assert_eq!(NavigationPolicy::evaluate("http://localhost:1420"), NavigationDecision::Reject);
    }
}
