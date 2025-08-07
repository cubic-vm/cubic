use regex::Regex;
use std::net::IpAddr;
use std::str::FromStr;

const FORMAT_ERROR: &str = "Must comply with format: [host_ip:]host_port:guest_port[/(udp|tcp)] (e.g. -p 8000:80 or -p 127.0.0.1:9000:90/tcp)";

#[derive(Clone, Debug, PartialEq)]
pub enum Protocol {
    Udp,
    Tcp,
    Both,
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "udp" => Ok(Protocol::Udp),
            "tcp" => Ok(Protocol::Tcp),
            _ => Err(FORMAT_ERROR.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortForward {
    host_ip: Option<IpAddr>,
    host_port: u16,
    guest_port: u16,
    protocol: Protocol,
}

impl PortForward {
    pub fn new(
        host_ip: Option<IpAddr>,
        host_port: u16,
        guest_port: u16,
        protocol: Protocol,
    ) -> Self {
        Self {
            host_ip,
            host_port,
            guest_port,
            protocol,
        }
    }

    pub fn to_qemu(&self) -> String {
        format!(
            "{}:{}:{}-:{}",
            match self.protocol {
                Protocol::Udp => "udp",
                Protocol::Tcp => "tcp",
                Protocol::Both => "",
            },
            self.host_ip.map(|ip| ip.to_string()).unwrap_or_default(),
            self.host_port,
            self.guest_port
        )
    }
}

impl FromStr for PortForward {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^(([\d.:]+):)?(\d+):(\d+)(/(\w+))?$").unwrap();
        let caps: Vec<_> = re
            .captures(value)
            .ok_or(FORMAT_ERROR.to_string())?
            .iter()
            .collect();

        if let &[_, _, host_ip, Some(host_port), Some(guest_port), _, protocol] = caps.as_slice() {
            Ok(Self::new(
                if let Some(ip) = host_ip {
                    Some(
                        ip.as_str()
                            .parse::<IpAddr>()
                            .map_err(|_| FORMAT_ERROR.to_string())?,
                    )
                } else {
                    None
                },
                host_port
                    .as_str()
                    .parse::<u16>()
                    .map_err(|_| FORMAT_ERROR.to_string())?,
                guest_port
                    .as_str()
                    .parse::<u16>()
                    .map_err(|_| FORMAT_ERROR.to_string())?,
                if let Some(protocol) = protocol {
                    protocol
                        .as_str()
                        .parse()
                        .map_err(|_| FORMAT_ERROR.to_string())?
                } else {
                    Protocol::Both
                },
            ))
        } else {
            Err(FORMAT_ERROR.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_basic_parsing() {
        assert_eq!(
            "1000:10".parse(),
            Ok(PortForward::new(None, 1000, 10, Protocol::Both))
        )
    }

    #[test]
    fn test_localhost_parsing() {
        assert_eq!(
            "127.0.0.1:2000:20".parse(),
            Ok(PortForward::new(
                Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                2000,
                20,
                Protocol::Both
            ))
        )
    }

    #[test]
    fn test_udp_parsing() {
        assert_eq!(
            "3000:30/udp".parse(),
            Ok(PortForward::new(None, 3000, 30, Protocol::Udp))
        )
    }

    #[test]
    fn test_tcp_parsing() {
        assert_eq!(
            "4000:40/tcp".parse(),
            Ok(PortForward::new(None, 4000, 40, Protocol::Tcp))
        )
    }

    #[test]
    fn test_ip_udp_parsing() {
        assert_eq!(
            "0.0.0.0:5000:50/udp".parse(),
            Ok(PortForward::new(
                Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
                5000,
                50,
                Protocol::Udp
            ))
        )
    }

    #[test]
    fn test_ip_tcp_parsing() {
        assert_eq!(
            "192.168.0.1:6000:60/tcp".parse(),
            Ok(PortForward::new(
                Some(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1))),
                6000,
                60,
                Protocol::Tcp
            ))
        )
    }
}
