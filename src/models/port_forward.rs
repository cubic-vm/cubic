use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Error, Formatter};
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::LazyLock;

static QEMU_PORT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\w+)?:([\d.:]+)?:(\d+)-:(\d+)$").unwrap());
static PORT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(([\d.:]+):)?(\d+):(\d+)(/(\w+))?$").unwrap());

const FORMAT_ERROR: &str = "Must comply with format: [host_ip:]host_port:guest_port[/(udp|tcp)] (e.g. -p 8000:80 or -p 127.0.0.1:9000:90/tcp)";
const QEMU_FORMAT_ERROR: &str = "Must comply with format: [tcp|udp]:[hostaddr]:hostport-[guestaddr]:guestport (e.g. ::8000-:80 or -p tcp:127.0.0.1:9000-:90)";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Udp,
    Tcp,
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

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Protocol::Udp => "udp",
                Protocol::Tcp => "tcp",
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortForward {
    host_ip: IpAddr,
    host_port: u16,
    guest_port: u16,
    protocol: Protocol,
}

impl PortForward {
    pub fn new(host_ip: IpAddr, host_port: u16, guest_port: u16, protocol: Protocol) -> Self {
        Self {
            host_ip,
            host_port,
            guest_port,
            protocol,
        }
    }

    pub fn get_host_ip(&self) -> IpAddr {
        self.host_ip
    }

    pub fn get_host_port(&self) -> u16 {
        self.host_port
    }

    pub fn get_guest_port(&self) -> u16 {
        self.guest_port
    }

    pub fn get_protocol(&self) -> Protocol {
        self.protocol
    }

    fn from_value(
        protocol: Option<&str>,
        host_ip: Option<&str>,
        host_port: &str,
        guest_port: &str,
    ) -> Result<Self, ()> {
        Ok(Self::new(
            if let Some(ip) = host_ip {
                ip.parse::<IpAddr>().map_err(|_| ())?
            } else {
                IpAddr::V4(Ipv4Addr::LOCALHOST)
            },
            host_port.parse::<u16>().map_err(|_| ())?,
            guest_port.parse::<u16>().map_err(|_| ())?,
            if let Some(protocol) = protocol {
                protocol.parse().map_err(|_| ())?
            } else {
                Protocol::Tcp
            },
        ))
    }

    pub fn from_qemu(value: &str) -> Result<Self, String> {
        let caps: Vec<_> = QEMU_PORT_REGEX
            .captures(value)
            .ok_or_else(|| QEMU_FORMAT_ERROR.to_string())?
            .iter()
            .collect();

        if let &[_, protocol, host_ip, Some(host_port), Some(guest_port)] = caps.as_slice() {
            Self::from_value(
                protocol.map(|p| p.as_str()),
                host_ip.map(|ip| ip.as_str()),
                host_port.as_str(),
                guest_port.as_str(),
            )
            .map_err(|_| QEMU_FORMAT_ERROR.to_string())
        } else {
            Err(QEMU_FORMAT_ERROR.to_string())
        }
    }

    pub fn to_qemu(&self) -> String {
        format!(
            "{}:{}:{}-:{}",
            self.protocol, self.host_ip, self.host_port, self.guest_port
        )
    }
}

impl Display for PortForward {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}:{}:{}/{}",
            self.host_ip, self.host_port, self.guest_port, self.protocol,
        )
    }
}

impl FromStr for PortForward {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let caps: Vec<_> = PORT_REGEX
            .captures(value)
            .ok_or_else(|| FORMAT_ERROR.to_string())?
            .iter()
            .collect();

        if let &[
            _,
            _,
            host_ip,
            Some(host_port),
            Some(guest_port),
            _,
            protocol,
        ] = caps.as_slice()
        {
            Self::from_value(
                protocol.map(|p| p.as_str()),
                host_ip.map(|ip| ip.as_str()),
                host_port.as_str(),
                guest_port.as_str(),
            )
            .map_err(|_| FORMAT_ERROR.to_string())
        } else {
            Err(FORMAT_ERROR.to_string())
        }
    }
}

impl Serialize for PortForward {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_qemu())
    }
}

impl<'de> Deserialize<'de> for PortForward {
    fn deserialize<D>(deserializer: D) -> Result<PortForward, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_qemu(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_forward(ip: [u8; 4], host: u16, guest: u16, protocol: Protocol) -> PortForward {
        PortForward::new(IpAddr::V4(Ipv4Addr::from(ip)), host, guest, protocol)
    }

    fn parse(input: &str) -> String {
        input.parse::<PortForward>().unwrap().to_string()
    }

    fn parse_qemu(input: &str) -> String {
        PortForward::from_qemu(input).unwrap().to_string()
    }

    // The parse tests below read their result back through `to_string`,
    // which this test pins.
    #[test]
    fn test_render_a_forward() {
        let local = build_forward([127, 0, 0, 1], 1000, 10, Protocol::Tcp);
        assert_eq!(local.to_string(), "127.0.0.1:1000:10/tcp");
        assert_eq!(local.to_qemu(), "tcp:127.0.0.1:1000-:10");

        let udp = build_forward([127, 0, 0, 1], 3000, 30, Protocol::Udp);
        assert_eq!(udp.to_string(), "127.0.0.1:3000:30/udp");
        assert_eq!(udp.to_qemu(), "udp:127.0.0.1:3000-:30");

        let any = build_forward([0, 0, 0, 0], 5000, 50, Protocol::Udp);
        assert_eq!(any.to_string(), "0.0.0.0:5000:50/udp");
        assert_eq!(any.to_qemu(), "udp:0.0.0.0:5000-:50");

        let lan = build_forward([192, 168, 0, 1], 6000, 60, Protocol::Tcp);
        assert_eq!(lan.to_string(), "192.168.0.1:6000:60/tcp");
        assert_eq!(lan.to_qemu(), "tcp:192.168.0.1:6000-:60");
    }

    #[test]
    fn test_parse_a_forward() {
        assert_eq!(parse("1000:10"), "127.0.0.1:1000:10/tcp");
        assert_eq!(parse("127.0.0.1:2000:20"), "127.0.0.1:2000:20/tcp");
        assert_eq!(parse("3000:30/udp"), "127.0.0.1:3000:30/udp");
        assert_eq!(parse("4000:40/tcp"), "127.0.0.1:4000:40/tcp");
        assert_eq!(parse("0.0.0.0:5000:50/udp"), "0.0.0.0:5000:50/udp");
        assert_eq!(parse("192.168.0.1:6000:60/tcp"), "192.168.0.1:6000:60/tcp");
    }

    #[test]
    fn test_parse_a_qemu_forward() {
        assert_eq!(parse_qemu("::1000-:10"), "127.0.0.1:1000:10/tcp");
        assert_eq!(parse_qemu(":127.0.0.1:2000-:20"), "127.0.0.1:2000:20/tcp");
        assert_eq!(parse_qemu("udp::3000-:30"), "127.0.0.1:3000:30/udp");
        assert_eq!(parse_qemu("tcp::4000-:40"), "127.0.0.1:4000:40/tcp");
        assert_eq!(parse_qemu("udp:0.0.0.0:5000-:50"), "0.0.0.0:5000:50/udp");
        assert_eq!(
            parse_qemu("tcp:192.168.0.1:6000-:60"),
            "192.168.0.1:6000:60/tcp"
        );
    }

    #[test]
    fn test_reject_an_invalid_forward() {
        assert!("abc".parse::<PortForward>().is_err());
        assert!("99999:80".parse::<PortForward>().is_err());
        assert!("999.0.0.1:8000:80".parse::<PortForward>().is_err());
        assert!(PortForward::from_qemu("garbage").is_err());
        assert!(PortForward::from_qemu("tcp:127.0.0.1:8000-:99999").is_err());
        assert!(Protocol::from_str("sctp").is_err());
    }

    #[test]
    fn test_serde_round_trip_uses_qemu_format() {
        let forward = build_forward([127, 0, 0, 1], 8000, 80, Protocol::Tcp);

        let serialized = serde_json::to_string(&forward).unwrap();
        assert_eq!(serialized, "\"tcp:127.0.0.1:8000-:80\"");

        let deserialized: PortForward = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, forward);
    }

    #[test]
    fn test_deserialize_rejects_invalid_forward() {
        assert!(serde_json::from_str::<PortForward>("\"garbage\"").is_err());
    }
}
