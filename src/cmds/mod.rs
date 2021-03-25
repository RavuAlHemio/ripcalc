pub mod derange;
pub mod enumerate;
pub mod minimize;
pub mod resize;
pub mod show_net;
#[cfg(feature = "num-bigint")]
pub mod split;


use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::addr::{IpAddress, IpAddressParseError, Ipv4Address, Ipv6Address};
use crate::net::IpNetwork;


static IPV4_WITH_SUBNET_REGEX: Lazy<Regex> = Lazy::new(||
    Regex::new("^(?P<addr>[0-9]+(?:[.][0-9]+){3})/(?P<wildcard>-)?(?P<mask>[0-9]+(?:[.][0-9]+){3})$").unwrap()
);
static IPV4_WITH_CIDR_REGEX: Lazy<Regex> = Lazy::new(||
    Regex::new("^(?P<addr>[0-9]+(?:[.][0-9]+){3})/(?P<wildcard>-)?(?P<cidr>[0-9]+)$").unwrap()
);
static IPV6_WITH_SUBNET_REGEX: Lazy<Regex> = Lazy::new(||
    Regex::new("^(?P<addr>[0-9a-f:]+)/(?P<wildcard>-)?(?P<mask>[0-9a-f:]*:[0-9a-f:]*)$").unwrap()
);
static IPV6_WITH_CIDR_REGEX: Lazy<Regex> = Lazy::new(||
    Regex::new("^(?P<addr>[0-9a-f:]+)/(?P<wildcard>-)?(?P<cidr>[0-9]+)$").unwrap()
);


/// An IP address that has been parsed from a string.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParsedIpAddress {
    Ipv4(Ipv4Address),
    Ipv6(Ipv6Address),
}
impl ParsedIpAddress {
    pub fn version(&self) -> u32 {
        match self {
            ParsedIpAddress::Ipv4(_) => 4,
            ParsedIpAddress::Ipv6(_) => 6,
        }
    }
}

/// An IP network specification parsed from a string, consisting of an IP address and a network
/// within which this IP address is contained.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSpec {
    Ipv4(Ipv4Address, IpNetwork<Ipv4Address>),
    Ipv6(Ipv6Address, IpNetwork<Ipv6Address>),
}

/// A list of IP network specifications parsed from strings.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSpecs {
    Nothing,
    MixedSpecs,
    Ipv4(Vec<(Ipv4Address, IpNetwork<Ipv4Address>)>),
    Ipv6(Vec<(Ipv6Address, IpNetwork<Ipv6Address>)>),
}

/// A subnet specification parsed from a string, in the form of either a CIDR prefix or a subnet
/// mask.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParsedSubnet {
    Cidr(usize),
    Ipv4Mask(Ipv4Address),
    Ipv6Mask(Ipv6Address),
}

/// An error that occurs when attempting to parse an IP network specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseNetspecError {
    /// The format of the IP network specification was not recognized. The contained string is the
    /// original specification string.
    Unrecognized(String),

    /// The IP address could not be parsed. The contained error describes why parsing the IP address
    /// failed.
    Address(IpAddressParseError),

    /// The subnet mask could not be parsed. The contained error describes why parsing the mask
    /// failed.
    Mask(IpAddressParseError),

    /// The CIDR prefix could not be parsed. The contained error describes why parsing the prefix
    /// failed.
    CidrParse(ParseIntError),

    /// The parsed CIDR prefix is out of range. The first value is the CIDR prefix that was parsed
    /// and the second value is the maximum CIDR prefix for the given IP address type.
    CidrRange(usize, usize),
}
impl fmt::Display for ParseNetspecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseNetspecError::Unrecognized(spec)
                => write!(f, "unrecognized network specification: {:?}", spec),
            ParseNetspecError::Address(e)
                => write!(f, "failed to parse address: {:?}", e),
            ParseNetspecError::Mask(e)
                => write!(f, "failed to parse mask: {:?}", e),
            ParseNetspecError::CidrParse(e)
                => write!(f, "failed to parse CIDR prefix: {:?}", e),
            ParseNetspecError::CidrRange(got, max)
                => write!(f, "CIDR prefix {} is greater than the maximum ({})", got, max),
        }
    }
}
impl Error for ParseNetspecError {
}


/// Attempts to parse a single IP address.
pub fn parse_addr(spec: &str) -> Result<ParsedIpAddress, IpAddressParseError> {
    if spec.contains('.') {
        if spec.contains(':') {
            // wtf
            return Err(IpAddressParseError::UnknownAddressType);
        }

        spec.parse()
            .map(|a| ParsedIpAddress::Ipv4(a))
    } else if spec.contains(':') {
        spec.parse()
            .map(|a| ParsedIpAddress::Ipv6(a))
    } else {
        Err(IpAddressParseError::UnknownAddressType)
    }
}

/// Attempts to parse a single IP network specification (address + network).
pub fn parse_netspec(spec: &str) -> Result<NetworkSpec, ParseNetspecError> {
    if let Some(caps) = IPV4_WITH_SUBNET_REGEX.captures(spec) {
        let addr_str = caps.name("addr").expect("'addr' captured").as_str();
        let is_wildcard = caps.name("wildcard").is_some();
        let mask_str = caps.name("mask").expect("'mask' captured").as_str();

        let addr: Ipv4Address = addr_str.parse()
            .map_err(|e| ParseNetspecError::Address(e))?;
        let mut mask: Ipv4Address = mask_str.parse()
            .map_err(|e| ParseNetspecError::Mask(e))?;
        if is_wildcard {
            mask = mask.bitwise_negate();
        }

        let net = IpNetwork::new_with_mask(addr, mask);
        Ok(NetworkSpec::Ipv4(addr, net))
    } else if let Some(caps) = IPV4_WITH_CIDR_REGEX.captures(spec) {
        let addr_str = caps.name("addr").unwrap().as_str();
        let is_wildcard = caps.name("wildcard").is_some();
        let mask_str = caps.name("cidr").unwrap().as_str();

        let addr: Ipv4Address = addr_str.parse()
            .map_err(|e| ParseNetspecError::Address(e))?;
        let mut cidr: usize = mask_str.parse()
            .map_err(|e| ParseNetspecError::CidrParse(e))?;
        if cidr > 32 {
            return Err(ParseNetspecError::CidrRange(cidr, 32));
        }
        if is_wildcard {
            cidr = 32 - cidr;
        }

        let net = IpNetwork::new_with_prefix(addr, cidr);
        Ok(NetworkSpec::Ipv4(addr, net))
    } else if let Some(caps) = IPV6_WITH_SUBNET_REGEX.captures(spec) {
        let addr_str = caps.name("addr").unwrap().as_str();
        let is_wildcard = caps.name("wildcard").is_some();
        let mask_str = caps.name("mask").unwrap().as_str();

        let addr: Ipv6Address = addr_str.parse()
            .map_err(|e| ParseNetspecError::Address(e))?;
        let mut mask: Ipv6Address = mask_str.parse()
            .map_err(|e| ParseNetspecError::Mask(e))?;
        if is_wildcard {
            mask = mask.bitwise_negate();
        }

        let net = IpNetwork::new_with_mask(addr, mask);
        Ok(NetworkSpec::Ipv6(addr, net))
    } else if let Some(caps) = IPV6_WITH_CIDR_REGEX.captures(spec) {
        let addr_str = caps.name("addr").unwrap().as_str();
        let is_wildcard = caps.name("wildcard").is_some();
        let mask_str = caps.name("cidr").unwrap().as_str();

        let addr: Ipv6Address = addr_str.parse()
            .map_err(|e| ParseNetspecError::Address(e))?;
        let mut cidr: usize = mask_str.parse()
            .map_err(|e| ParseNetspecError::CidrParse(e))?;
        if cidr > 128 {
            return Err(ParseNetspecError::CidrRange(cidr, 128));
        }
        if is_wildcard {
            cidr = 128 - cidr;
        }

        let net = IpNetwork::new_with_prefix(addr, cidr);
        Ok(NetworkSpec::Ipv6(addr, net))
    } else {
        Err(ParseNetspecError::Unrecognized(String::from(spec)))
    }
}

/// Attempts to parse multiple IP network specifications (address + network), ensuring that all are
/// of the same IP version.
pub fn parse_same_family_netspecs<S: AsRef<str>>(spec_strs: &[S]) -> Result<NetworkSpecs, ParseNetspecError> {
    if spec_strs.len() == 0 {
        return Ok(NetworkSpecs::Nothing);
    }

    match parse_netspec(spec_strs[0].as_ref())? {
        NetworkSpec::Ipv4(addr, net) => {
            let mut specs = Vec::with_capacity(spec_strs.len());
            specs.push((addr, net));

            for spec_str in &spec_strs[1..] {
                match parse_netspec(spec_str.as_ref())? {
                    NetworkSpec::Ipv4(addr, net) => {
                        specs.push((addr, net));
                    },
                    NetworkSpec::Ipv6(_, _) => {
                        return Ok(NetworkSpecs::MixedSpecs);
                    },
                };
            }

            Ok(NetworkSpecs::Ipv4(specs))
        },
        NetworkSpec::Ipv6(addr, net) => {
            let mut specs = Vec::with_capacity(spec_strs.len());
            specs.push((addr, net));

            for spec_str in &spec_strs[1..] {
                match parse_netspec(spec_str.as_ref())? {
                    NetworkSpec::Ipv6(addr, net) => {
                        specs.push((addr, net));
                    },
                    NetworkSpec::Ipv4(_, _) => {
                        return Ok(NetworkSpecs::MixedSpecs);
                    },
                };
            }

            Ok(NetworkSpecs::Ipv6(specs))
        },
    }
}

/// Attempts to parse a subnet specification (mask or CIDR prefix).
pub fn parse_subnet(spec: &str) -> Result<ParsedSubnet, ParseNetspecError> {
    if spec.contains(':') {
        let ipv6_addr: Ipv6Address = match spec.parse() {
            Ok(ia) => ia,
            Err(e) => {
                return Err(ParseNetspecError::Mask(e));
            },
        };
        Ok(ParsedSubnet::Ipv6Mask(ipv6_addr))
    } else if spec.contains('.') {
        let ipv4_addr: Ipv4Address = match spec.parse() {
            Ok(ia) => ia,
            Err(e) => {
                return Err(ParseNetspecError::Mask(e));
            },
        };
        Ok(ParsedSubnet::Ipv4Mask(ipv4_addr))
    } else {
        let cidr_prefix: usize = match spec.parse() {
            Ok(cp) => cp,
            Err(e) => {
                return Err(ParseNetspecError::CidrParse(e));
            },
        };
        Ok(ParsedSubnet::Cidr(cidr_prefix))
    }
}
