//! IP address parsing and formatting utilities.

use std::net::IpAddr;

/// Parse an IP address from a URI host.
pub(crate) fn parse_ip_from_uri_host(host: &str) -> Option<IpAddr> {
    // Attempt to parse directly as an IP address (IPv4 or IPv6 without brackets)
    if let Ok(ip_addr) = host.parse::<IpAddr>() {
        return Some(ip_addr);
    }

    // If direct parsing fails, check if it's an IPv6 in brackets
    host.strip_prefix('[')
        .and_then(|stripped| stripped.strip_suffix(']'))
        .and_then(|stripped| stripped.parse::<IpAddr>().ok())
}
