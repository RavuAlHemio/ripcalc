use std::convert::TryFrom;
use std::fmt;

use crate::addr::IpAddress;
use crate::{bit_manip, cidr};


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IpNetwork<A: IpAddress> {
    base_addr: A,
    subnet_mask: A,
    cidr_prefix: Option<usize>,
}

impl<A: IpAddress> IpNetwork<A> {
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

    pub fn base_addr(&self) -> A { self.base_addr }
    pub fn subnet_mask(&self) -> A { self.subnet_mask }
    pub fn cidr_prefix(&self) -> Option<usize> { self.cidr_prefix }

    pub fn cisco_wildcard(&self) -> A {
        self.subnet_mask.bitwise_negate()
    }

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

    #[cfg(feature = "num-bigint")]
    pub fn host_count(&self) -> num_bigint::BigInt {
        let addr_count: num_bigint::BigInt = self.address_count().into();
        addr_count - 2
    }

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

    pub fn last_addr_of_subnet(&self) -> A {
        self.broadcast_addr().unwrap_or(self.base_addr)
    }

    pub fn contains(&self, addr: &A) -> bool {
        (*addr & self.subnet_mask) == self.base_addr
    }

    pub fn is_superset_of(&self, other: &IpNetwork<A>) -> bool {
        // a network A is a superset of a network B if:
        // 1. the base address of B bitwise AND with the subnet mask of A returns the base address of A
        //    (B is contained in A)
        // 2. the subnet mask of A bitwise AND with the subnet mask of B returns the subnet mask of A
        //    (all host bits in B are host bits in A)
        (other.base_addr & self.subnet_mask == self.base_addr)
            && (other.subnet_mask & self.subnet_mask == self.subnet_mask)
    }

    pub fn is_subset_of(&self, other: &IpNetwork<A>) -> bool {
        other.is_superset_of(self)
    }

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
