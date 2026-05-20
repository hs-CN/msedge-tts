/// VER (1), CMD (1), DSTPORT (2), DSTIP (4), ID (? + 1), socks4a? DOMAIN (? + 1)
fn build_socks4_connection_request(
    target_host: &str,
    dst_ip: Option<std::net::Ipv4Addr>,
    username: Option<&str>,
) -> Vec<u8> {
    // VER, CMD, DSTPORT
    let mut bytes = vec![0x04, 0x01, 0x01, 0xbb];

    // DSTIP (4)
    if let Some(ip) = dst_ip {
        bytes.extend(ip.octets()); // socks4
    } else {
        bytes.extend([0x00, 0x00, 0x00, 0x01]); // socks4a
    }

    // ID (? + 1)
    if let Some(username) = username {
        bytes.extend(username.as_bytes());
    }
    bytes.push(0x00);

    // socks4a DOMAIN (? + 1)
    if dst_ip.is_none() {
        bytes.extend(target_host.as_bytes());
        bytes.push(0x00);
    }

    bytes
}

/// VER (1), IDLEN (1), ID (IDLEN), PWLEN (1), PW (PWLEN)
fn build_socks5_authentication_request(username: &str, password: &str) -> Vec<u8> {
    let mut bytes = vec![0x01]; // VER
    if username.len() == 0 {
        bytes.extend([0x01, 0x00]);
    } else {
        bytes.push(username.len() as u8); // IDLEN
        bytes.extend(username.as_bytes()); // ID
    }
    if password.len() == 0 {
        bytes.extend([0x01, 0x00]);
    } else {
        bytes.push(password.len() as u8); // PWLEN
        bytes.extend(password.as_bytes()); // PW
    }
    bytes
}

/// VER (1), CMD (1), RSV (1), DSTADDR [TYPE (1), ADDR (?)], DSTPORT (2)
fn build_socks5_connection_request(target_host: &str, dst_ip: Option<std::net::IpAddr>) -> Vec<u8> {
    // VER, CMD, RSV
    let mut bytes = vec![0x05, 0x01, 0x00];

    // DSTADDR
    if let Some(ip) = dst_ip {
        match ip {
            std::net::IpAddr::V4(ip) => {
                bytes.push(0x01); // TYPE
                bytes.extend(ip.octets()); // ADDR
            }
            std::net::IpAddr::V6(ip) => {
                bytes.push(0x04); // TYPE
                bytes.extend(ip.octets()); // ADDR
            }
        }
    } else {
        bytes.push(0x03); // TYPE
        bytes.push(target_host.len() as u8); // ADDRLEN
        bytes.extend(target_host.as_bytes()); // ADDR
    }

    // DSTPORT
    bytes.extend([0x01, 0xbb]);

    bytes
}

fn build_http_proxy_request(
    target_host: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> String {
    use base64::*;

    if username.is_some() && password.is_some() {
        let credential = base64::prelude::BASE64_STANDARD.encode(format!(
            "{}:{}",
            username.unwrap(),
            password.unwrap()
        ));
        format!(
            "CONNECT {}:443 HTTP/1.1\r\nHost: {}:443\r\nProxy-Authorization: Basic {}\r\nProxy-Connection: Keep-Alive\r\n\r\n",
            target_host, target_host, credential
        )
    } else {
        format!(
            "CONNECT {}:443 HTTP/1.1\r\nHost: {}:443\r\nProxy-Connection: Keep-Alive\r\n\r\n",
            target_host, target_host
        )
    }
}

#[cfg(feature = "blocking")]
pub(crate) mod blocking;

#[cfg(feature = "smol-runtime")]
pub(crate) mod smol_runtime;

#[cfg(feature = "tokio-runtime")]
pub(crate) mod tokio_runtime;
