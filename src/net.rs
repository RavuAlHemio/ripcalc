use std::convert::TryFrom;
use std::fmt;

use crate::addr::IpAddress;
use crate::{bit_manip, cidr};


/// An IP network, consisting of a base address and subnet mask.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IpNetwork<A: IpAddress> {
    base_addr: A,
    subnet_mask: A,
    cidr_prefix: Option<usize>,
}

impl<A: IpAddress> IpNetwork<A> {
    /// Creates a new IpNetwork from the given IP address and subnet mask.
    pub fn new_with_mask(
        addr: A,
        subnet_mask: A,
    ) -> IpNetwork<A> {
        // calculate base address by ANDing address with subnet mask
        let base_addr = addr & subnet_mask;
        let cidr_prefix = cidr::prefix_from_subnet_mask_bytes(&subnet_mask.to_bytes());
        IpNetwork {
            base_addr,
            subnet_mask,
            cidr_prefix,
        }
    }

    /// Creates a new IpNetwork from the given IP address and CIDR prefix.
    pub fn new_with_prefix(
        addr: A,
        cidr_prefix: usize,
    ) -> IpNetwork<A> {
        let mask_bytes = cidr::subnet_mask_bytes_from_prefix(cidr_prefix, addr.byte_count());
        let subnet_mask: A = A::from_bytes(&mask_bytes).unwrap();
        // calculate base address by ANDing address with subnet mask
        let base_addr = addr & subnet_mask;

        IpNetwork {
            base_addr,
            subnet_mask,
            cidr_prefix: Some(cidr_prefix),
        }
    }

    /// Creates a new IpNetwork from the given IP address and subnet mask. Returns `None` if `addr`
    /// is not the base address of the specified subnet.
    pub fn new_with_mask_strict(
        addr: A,
        subnet_mask: A,
    ) -> Option<IpNetwork<A>> {
        let net = Self::new_with_mask(addr, subnet_mask);
        if net.base_addr() == addr {
            Some(net)
        } else {
            None
        }
    }

    /// Creates a new IpNetwork from the given IP address and CIDR prefix. Returns `None` if `addr`
    /// is not the base address of the specified subnet.
    pub fn new_with_prefix_strict(
        addr: A,
        cidr_prefix: usize,
    ) -> Option<IpNetwork<A>> {
        let net = Self::new_with_prefix(addr, cidr_prefix);
        if net.base_addr() == addr {
            Some(net)
        } else {
            None
        }
    }

    /// The base address of this IP network.
    pub fn base_addr(&self) -> A { self.base_addr }

    /// The subnet mask of this IP network.
    pub fn subnet_mask(&self) -> A { self.subnet_mask }

    /// The CIDR prefix of this IP network, or None if this network has a mixed subnet mask (i.e. a
    /// subnet mask with network and host bits interspersed).
    pub fn cidr_prefix(&self) -> Option<usize> { self.cidr_prefix }

    /// The Cisco wildcard of this IP network, i.e. the bitwise complement of the subnet mask.
    pub fn cisco_wildcard(&self) -> A {
        self.subnet_mask.bitwise_negate()
    }

    /// The number of addresses in this network.
    #[cfg(feature = "num-bigint")]
    pub fn address_count(&self) -> num_bigint::BigUint {
        let mut ret = num_bigint::BigUint::from(1u32);
        let two = num_bigint::BigUint::from(2u32);
        for b in self.cisco_wildcard().to_bytes() {
            for _ in 0..b.count_ones() {
                ret *= &two;
            }
        }
        ret
    }

    /// The number of host addresses, i.e. non-network and non-broadcast addresses, in this network.
    #[cfg(feature = "num-bigint")]
    pub fn host_count(&self) -> num_bigint::BigInt {
        let addr_count: num_bigint::BigInt = self.address_count().into();
        addr_count - 2
    }

    /// The address of the first host in this network, or `None` if the network has too few
    /// addresses to have even a single host address.
    pub fn first_host_addr(&self) -> Option<A> {
        let host_bits_available: usize = self.cisco_wildcard().to_bytes()
            .iter()
            .map(|b| usize::try_from(b.count_ones()).unwrap())
            .sum();
        if host_bits_available < 2 {
            // all ones: the base address is the network
            // all ones except one zero: 0 is the network, 1 is broadcast
            // => at least two zeroes necessary for a non-degenerate subnet
            return None;
        }

        // unravel and weave
        let unraveled_base = bit_manip::unravel_address(self.base_addr, self.subnet_mask);
        let unraveled_first_host = unraveled_base.add_offset(1)?;
        Some(bit_manip::weave_address(unraveled_first_host, self.subnet_mask))
    }

    /// The broadcast address of this network, or `None` if the network has too few addresses to
    /// have a broadcast address.
    pub fn broadcast_addr(&self) -> Option<A> {
        let host_bits_available: usize = self.cisco_wildcard().to_bytes()
            .iter()
            .map(|b| usize::try_from(b.count_ones()).unwrap())
            .sum();
        if host_bits_available < 1 {
            // all ones: the base address is the network
            // => at least one zero necessary for a subnet with a broadcast address
            return None;
        }

        let unraveled_base = bit_manip::unravel_address(self.base_addr, self.subnet_mask);
        let hca_bytes = cidr::subnet_mask_bytes_from_prefix(
            self.base_addr.to_bytes().len()*8 - host_bits_available,
            self.base_addr.byte_count(),
        );
        let host_count_address = A::from_bytes(&hca_bytes)
            .expect("subnet mask from prefix")
            .bitwise_negate();
        let unraveled_broadcast = unraveled_base.add_addr(&host_count_address)?;
        Some(bit_manip::weave_address(unraveled_broadcast, self.subnet_mask))
    }

    /// The address of the last host in this network, or `None` if the network has too few addresses
    /// to have even a single host address.
    pub fn last_host_addr(&self) -> Option<A> {
        let host_bits_available: usize = self.cisco_wildcard().to_bytes()
            .iter()
            .map(|b| usize::try_from(b.count_ones()).unwrap())
            .sum();
        if host_bits_available < 2 {
            // all ones: the base address is the network
            // all ones except one zero: 0 is the network, 1 is broadcast
            // => at least two zeroes necessary for a non-degenerate subnet
            return None;
        }

        let unraveled_base = bit_manip::unravel_address(self.base_addr, self.subnet_mask);
        let hca_bytes = cidr::subnet_mask_bytes_from_prefix(
            self.base_addr.to_bytes().len()*8 - host_bits_available,
            self.base_addr.byte_count(),
        );
        let host_count_address = A::from_bytes(&hca_bytes)
            .expect("subnet mask from prefix")
            .bitwise_negate();
        let unraveled_broadcast = unraveled_base.add_addr(&host_count_address)?;
        let unraveled_last_host = unraveled_broadcast.subtract_offset(1)?;
        Some(bit_manip::weave_address(unraveled_last_host, self.subnet_mask))
    }

    /// The base address of the network immediately following this one, or `None` if this network
    /// borders the end of the address space.
    pub fn next_subnet_base_addr(&self) -> Option<A> {
        let host_bits_available: usize = self.cisco_wildcard().to_bytes()
            .iter()
            .map(|b| usize::try_from(b.count_ones()).unwrap())
            .sum();
        let unraveled_base = bit_manip::unravel_address(self.base_addr, self.subnet_mask);
        let hca_bytes = cidr::subnet_mask_bytes_from_prefix(
            self.base_addr.to_bytes().len()*8 - host_bits_available,
            self.base_addr.byte_count(),
        );
        let host_count_address = A::from_bytes(&hca_bytes)
            .expect("subnet mask from prefix")
            .bitwise_negate();
        let unraveled_broadcast = unraveled_base.add_addr(&host_count_address)?;
        let unraveled_next_base = unraveled_broadcast.add_offset(1)?;
        Some(bit_manip::weave_address(unraveled_next_base, self.subnet_mask))
    }

    /// The last address of the network, which is the broadcast address or, if there is no broadcast
    /// address, the base address of the network.
    pub fn last_addr_of_subnet(&self) -> A {
        self.broadcast_addr().unwrap_or(self.base_addr)
    }

    /// Returns whether this network contains the given address.
    pub fn contains(&self, addr: &A) -> bool {
        (*addr & self.subnet_mask) == self.base_addr
    }

    /// Returns whether this network is a superset of another network, i.e. all addresses that are
    /// contained in the other network are also contained in this network.
    pub fn is_superset_of(&self, other: &IpNetwork<A>) -> bool {
        // a network A is a superset of a network B if:
        // 1. the base address of B bitwise AND with the subnet mask of A returns the base address of A
        //    (B is contained in A)
        // 2. the subnet mask of A bitwise AND with the subnet mask of B returns the subnet mask of A
        //    (all host bits in B are host bits in A)
        (other.base_addr & self.subnet_mask == self.base_addr)
            && (other.subnet_mask & self.subnet_mask == self.subnet_mask)
    }

    /// Returns whether this network is a subset of another network, i.e. all addresses that are
    /// contained in this network are also contained in the other network.
    pub fn is_subset_of(&self, other: &IpNetwork<A>) -> bool {
        other.is_superset_of(self)
    }

    /// Returns whether this network and another network intersect, i.e. there is at least one
    /// address that is contained in both networks.
    pub fn intersects(&self, other: &IpNetwork<A>) -> bool {
        let self_first = self.base_addr;
        let self_last = self.last_addr_of_subnet();
        let other_first = other.base_addr;
        let other_last = other.last_addr_of_subnet();

        // thisFirst <= otherLast && otherFirst <= thisLast
        self_first <= other_last && other_first <= self_last
    }
}
impl<A: IpAddress> fmt::Display for IpNetwork<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix) = self.cidr_prefix {
            write!(f, "{}/{}", self.base_addr, prefix)
        } else {
            write!(f, "{}/{}", self.base_addr, self.subnet_mask)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[cfg(feature = "num-bigint")]
    use num_bigint::{BigInt, BigUint};
    use crate::addr::{IpAddressParseError, Ipv4Address, Ipv6Address};

    fn parse_addr<A: FromStr<Err = IpAddressParseError> + IpAddress>(s: &str) -> A { s.parse().unwrap() }
    fn parse_ipv4(s: &str) -> Ipv4Address { parse_addr(s) }
    fn parse_ipv6(s: &str) -> Ipv6Address { parse_addr(s) }
    fn parse_bigint(s: &str) -> BigInt { s.parse().unwrap() }
    fn parse_biguint(s: &str) -> BigUint { s.parse().unwrap() }

    #[test]
    fn test_ipv4_new_with_mask() {
        // CIDR mask
        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_mask(
            parse_addr("127.0.0.1"),
            parse_addr("255.0.0.0"),
        );
        assert_eq!(parse_ipv4("127.0.0.0"), net.base_addr());
        assert_eq!(parse_ipv4("255.0.0.0"), net.subnet_mask());
        assert_eq!(Some(8), net.cidr_prefix());
        assert_eq!(parse_ipv4("0.255.255.255"), net.cisco_wildcard());
        assert_eq!(Some(parse_ipv4("127.0.0.1")), net.first_host_addr());
        assert_eq!(Some(parse_ipv4("127.255.255.255")), net.broadcast_addr());
        assert_eq!(Some(parse_ipv4("127.255.255.254")), net.last_host_addr());
        assert_eq!(Some(parse_ipv4("128.0.0.0")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.first_host_addr().unwrap()));
        assert!(net.contains(&parse_ipv4("127.31.41.59")));
        assert!(net.contains(&net.last_host_addr().unwrap()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(BigUint::from(16777216u32), net.address_count());
            assert_eq!(BigInt::from(16777214), net.host_count());
        }

        // mixed mask
        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_mask(
            parse_addr("127.0.0.1"),
            parse_addr("255.0.255.0"),
        );
        assert_eq!(parse_ipv4("127.0.0.0"), net.base_addr());
        assert_eq!(parse_ipv4("255.0.255.0"), net.subnet_mask());
        assert_eq!(None, net.cidr_prefix());
        assert_eq!(parse_ipv4("0.255.0.255"), net.cisco_wildcard());
        assert_eq!(Some(parse_ipv4("127.0.0.1")), net.first_host_addr());
        assert_eq!(Some(parse_ipv4("127.255.0.255")), net.broadcast_addr());
        assert_eq!(Some(parse_ipv4("127.255.0.254")), net.last_host_addr());
        assert_eq!(Some(parse_ipv4("127.0.1.0")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.first_host_addr().unwrap()));
        assert!(net.contains(&parse_ipv4("127.31.0.59")));
        assert!(net.contains(&net.last_host_addr().unwrap()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(BigUint::from(65536u32), net.address_count());
            assert_eq!(BigInt::from(65534), net.host_count());
        }

        // full mask
        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_mask(
            parse_addr("127.0.0.1"),
            parse_addr("255.255.255.255"),
        );
        assert_eq!(parse_ipv4("127.0.0.1"), net.base_addr());
        assert_eq!(parse_ipv4("255.255.255.255"), net.subnet_mask());
        assert_eq!(Some(32), net.cidr_prefix());
        assert_eq!(parse_ipv4("0.0.0.0"), net.cisco_wildcard());
        assert_eq!(None, net.first_host_addr());
        assert_eq!(None, net.broadcast_addr());
        assert_eq!(None, net.last_host_addr());
        assert_eq!(Some(parse_ipv4("127.0.0.2")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(!net.contains(&parse_ipv4("127.0.0.0")));
        assert!(!net.contains(&parse_ipv4("127.0.0.2")));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(BigUint::from(1u32), net.address_count());
            assert_eq!(BigInt::from(-1), net.host_count());
        }

        // point-to-point mask
        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_mask(
            parse_addr("127.0.0.1"),
            parse_addr("255.255.255.254"),
        );
        assert_eq!(parse_ipv4("127.0.0.0"), net.base_addr());
        assert_eq!(parse_ipv4("255.255.255.254"), net.subnet_mask());
        assert_eq!(Some(31), net.cidr_prefix());
        assert_eq!(parse_ipv4("0.0.0.1"), net.cisco_wildcard());
        assert_eq!(None, net.first_host_addr());
        assert_eq!(Some(parse_ipv4("127.0.0.1")), net.broadcast_addr());
        assert_eq!(None, net.last_host_addr());
        assert_eq!(Some(parse_ipv4("127.0.0.2")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(BigUint::from(2u32), net.address_count());
            assert_eq!(BigInt::from(0), net.host_count());
        }

        // full-space subnet
        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_mask(
            parse_addr("0.0.0.0"),
            parse_addr("0.0.0.0"),
        );
        assert_eq!(parse_ipv4("0.0.0.0"), net.base_addr());
        assert_eq!(parse_ipv4("0.0.0.0"), net.subnet_mask());
        assert_eq!(Some(0), net.cidr_prefix());
        assert_eq!(parse_ipv4("255.255.255.255"), net.cisco_wildcard());
        assert_eq!(Some(parse_ipv4("0.0.0.1")), net.first_host_addr());
        assert_eq!(Some(parse_ipv4("255.255.255.255")), net.broadcast_addr());
        assert_eq!(Some(parse_ipv4("255.255.255.254")), net.last_host_addr());
        assert_eq!(None, net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.first_host_addr().unwrap()));
        assert!(net.contains(&parse_ipv4("31.41.59.26")));
        assert!(net.contains(&net.last_host_addr().unwrap()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(BigUint::from(4294967296u64), net.address_count());
            assert_eq!(BigInt::from(4294967294u32), net.host_count());
        }
    }

    #[test]
    fn test_ipv6_new_with_mask() {
        // CIDR mask
        let net: IpNetwork<Ipv6Address> = IpNetwork::new_with_mask(
            parse_addr("fe80::1"),
            parse_addr("ffc0::"),
        );
        assert_eq!(parse_ipv6("fe80::"), net.base_addr());
        assert_eq!(parse_ipv6("ffc0::"), net.subnet_mask());
        assert_eq!(Some(10), net.cidr_prefix());
        assert_eq!(parse_ipv6("3f:ffff:ffff:ffff:ffff:ffff:ffff:ffff"), net.cisco_wildcard());
        assert_eq!(Some(parse_ipv6("fe80::1")), net.first_host_addr());
        assert_eq!(Some(parse_ipv6("febf:ffff:ffff:ffff:ffff:ffff:ffff:ffff")), net.broadcast_addr());
        assert_eq!(Some(parse_ipv6("febf:ffff:ffff:ffff:ffff:ffff:ffff:fffe")), net.last_host_addr());
        assert_eq!(Some(parse_ipv6("fec0::")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.first_host_addr().unwrap()));
        assert!(net.contains(&parse_ipv6("feab:1234:5678:1234:5678:1234:5678:1234")));
        assert!(net.contains(&net.last_host_addr().unwrap()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(parse_biguint("332306998946228968225951765070086144"), net.address_count());
            assert_eq!(parse_bigint("332306998946228968225951765070086142"), net.host_count());
        }

        // mixed mask
        let net: IpNetwork<Ipv6Address> = IpNetwork::new_with_mask(
            parse_addr("1234:1234:1234:1234:1234:1234:1234:1234"),
            parse_addr("ffff:0000:ffff:0000:0000:ffff:0000:ffff"),
        );
        assert_eq!(parse_ipv6("1234:0000:1234:0000:0000:1234:0000:1234"), net.base_addr());
        assert_eq!(parse_ipv6("ffff:0000:ffff:0000:0000:ffff:0000:ffff"), net.subnet_mask());
        assert_eq!(None, net.cidr_prefix());
        assert_eq!(parse_ipv6("0000:ffff:0000:ffff:ffff:0000:ffff:0000"), net.cisco_wildcard());
        assert_eq!(Some(parse_ipv6("1234:0000:1234:0000:0000:1234:0001:1234")), net.first_host_addr());
        assert_eq!(Some(parse_ipv6("1234:ffff:1234:ffff:ffff:1234:ffff:1234")), net.broadcast_addr());
        assert_eq!(Some(parse_ipv6("1234:ffff:1234:ffff:ffff:1234:fffe:1234")), net.last_host_addr());
        assert_eq!(Some(parse_ipv6("1234:0000:1234:0000:0000:1234:0000:1235")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.first_host_addr().unwrap()));
        assert!(net.contains(&parse_ipv6("1234:3141:1234:5926:5358:1234:9793:1234")));
        assert!(net.contains(&net.last_host_addr().unwrap()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(parse_biguint("18446744073709551616"), net.address_count());
            assert_eq!(parse_bigint("18446744073709551614"), net.host_count());
        }

        // full mask
        let net: IpNetwork<Ipv6Address> = IpNetwork::new_with_mask(
            parse_addr("::1"),
            parse_addr("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"),
        );
        assert_eq!(parse_ipv6("::1"), net.base_addr());
        assert_eq!(parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"), net.subnet_mask());
        assert_eq!(Some(128), net.cidr_prefix());
        assert_eq!(parse_ipv6("::"), net.cisco_wildcard());
        assert_eq!(None, net.first_host_addr());
        assert_eq!(None, net.broadcast_addr());
        assert_eq!(None, net.last_host_addr());
        assert_eq!(Some(parse_ipv6("::2")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(BigUint::from(1u32), net.address_count());
            assert_eq!(BigInt::from(-1), net.host_count());
        }

        // point-to-point mask
        let net: IpNetwork<Ipv6Address> = IpNetwork::new_with_mask(
            parse_addr("fe80::3"),
            parse_addr("ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe"),
        );
        assert_eq!(parse_ipv6("fe80::2"), net.base_addr());
        assert_eq!(parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe"), net.subnet_mask());
        assert_eq!(Some(127), net.cidr_prefix());
        assert_eq!(parse_ipv6("::1"), net.cisco_wildcard());
        assert_eq!(None, net.first_host_addr());
        assert_eq!(Some(parse_ipv6("fe80::3")), net.broadcast_addr());
        assert_eq!(None, net.last_host_addr());
        assert_eq!(Some(parse_ipv6("fe80::4")), net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        assert!(!net.contains(&net.next_subnet_base_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(BigUint::from(2u32), net.address_count());
            assert_eq!(BigInt::from(0), net.host_count());
        }

        // full-space subnet
        let net: IpNetwork<Ipv6Address> = IpNetwork::new_with_mask(
            parse_addr("::"),
            parse_addr("::"),
        );
        assert_eq!(parse_ipv6("::"), net.base_addr());
        assert_eq!(parse_ipv6("::"), net.subnet_mask());
        assert_eq!(Some(0), net.cidr_prefix());
        assert_eq!(parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"), net.cisco_wildcard());
        assert_eq!(Some(parse_ipv6("::1")), net.first_host_addr());
        assert_eq!(Some(parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff")), net.broadcast_addr());
        assert_eq!(Some(parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe")), net.last_host_addr());
        assert_eq!(None, net.next_subnet_base_addr());
        assert!(net.contains(&net.base_addr()));
        assert!(net.contains(&net.first_host_addr().unwrap()));
        assert!(net.contains(&parse_ipv6("1234:3141:1234:5926:5358:1234:9793:1234")));
        assert!(net.contains(&net.last_host_addr().unwrap()));
        assert!(net.contains(&net.broadcast_addr().unwrap()));
        if cfg!(feature = "num-bigint") {
            assert_eq!(parse_biguint("340282366920938463463374607431768211456"), net.address_count());
            assert_eq!(parse_bigint("340282366920938463463374607431768211454"), net.host_count());
        }
    }

    #[test]
    fn test_ipv4_new_with_prefix() {
        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_prefix(
            parse_addr("127.0.0.1"),
            8,
        );
        assert_eq!(parse_ipv4("127.0.0.0"), net.base_addr());
        assert_eq!(parse_ipv4("255.0.0.0"), net.subnet_mask());
        assert_eq!(Some(8), net.cidr_prefix);

        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_prefix(
            parse_addr("1.2.3.4"),
            24,
        );
        assert_eq!(parse_ipv4("1.2.3.0"), net.base_addr());
        assert_eq!(parse_ipv4("255.255.255.0"), net.subnet_mask());
        assert_eq!(Some(24), net.cidr_prefix);
    }

    #[test]
    fn test_ipv6_new_with_prefix() {
        let net: IpNetwork<Ipv6Address> = IpNetwork::new_with_prefix(
            parse_addr("feba::"),
            10,
        );
        assert_eq!(parse_addr::<Ipv6Address>("fe80::"), net.base_addr());
        assert_eq!(parse_addr::<Ipv6Address>("ffc0::"), net.subnet_mask());
        assert_eq!(Some(10), net.cidr_prefix);
    }

    #[test]
    fn test_ipv4_new_with_mask_strict() {
        // CIDR mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("127.0.0.0"),
                parse_ipv4("255.0.0.0"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("127.0.0.1"),
                parse_ipv4("255.0.0.0"),
            )
                .is_none()
        );

        // mixed mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("127.0.0.0"),
                parse_ipv4("255.0.255.0"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("127.0.0.1"),
                parse_ipv4("255.0.255.0"),
            )
                .is_none()
        );

        // full mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("127.0.0.1"),
                parse_ipv4("255.255.255.255"),
            )
                .is_some()
        );

        // point-to-point mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("127.0.0.0"),
                parse_ipv4("255.255.255.254"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("127.0.0.1"),
                parse_ipv4("255.255.255.254"),
            )
                .is_none()
        );

        // full-space subnet
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("0.0.0.0"),
                parse_ipv4("0.0.0.0"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv4("5.0.0.0"),
                parse_ipv4("0.0.0.0"),
            )
                .is_none()
        );
    }

    #[test]
    fn test_ipv6_new_with_mask_strict() {
        // CIDR mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("fe80::"),
                parse_ipv6("ffc0::"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("fe80::1"),
                parse_ipv6("ffc0::"),
            )
                .is_none()
        );

        // mixed mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("1234:0:1234::1234:0:1234"),
                parse_ipv6("ffff:0000:ffff:0000:0000:ffff:0000:ffff"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("1234:1234:1234:1234:1234:1234:1234:1234"),
                parse_ipv6("ffff:0000:ffff:0000:0000:ffff:0000:ffff"),
            )
                .is_none()
        );

        // full mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("::1"),
                parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"),
            )
                .is_some()
        );

        // point-to-point mask
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("fe80::2"),
                parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("fe80::3"),
                parse_ipv6("ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe"),
            )
                .is_none()
        );

        // full-space subnet
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("::"),
                parse_ipv6("::"),
            )
                .is_some()
        );
        assert!(
            IpNetwork::new_with_mask_strict(
                parse_ipv6("1::"),
                parse_ipv6("::"),
            )
                .is_none()
        );
    }

    #[test]
    fn test_ipv4_new_with_prefix_strict() {
        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_prefix(
            parse_addr("127.0.0.1"),
            8,
        );
        assert_eq!(parse_ipv4("127.0.0.0"), net.base_addr());
        assert_eq!(parse_ipv4("255.0.0.0"), net.subnet_mask());
        assert_eq!(Some(8), net.cidr_prefix);

        let net: IpNetwork<Ipv4Address> = IpNetwork::new_with_prefix(
            parse_addr("1.2.3.4"),
            24,
        );
        assert_eq!(parse_ipv4("1.2.3.0"), net.base_addr());
        assert_eq!(parse_ipv4("255.255.255.0"), net.subnet_mask());
        assert_eq!(Some(24), net.cidr_prefix);
    }

    #[test]
    fn test_ipv6_new_with_prefix_strict() {
        let net: IpNetwork<Ipv6Address> = IpNetwork::new_with_prefix(
            parse_addr("feba::"),
            10,
        );
        assert_eq!(parse_addr::<Ipv6Address>("fe80::"), net.base_addr());
        assert_eq!(parse_addr::<Ipv6Address>("ffc0::"), net.subnet_mask());
        assert_eq!(Some(10), net.cidr_prefix);
    }
}
