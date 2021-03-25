use std::iter::Iterator;

use crate::usage;
use crate::addr::IpAddress;
use crate::bit_manip::{unravel_address, weave_address};
use crate::cmds::{NetworkSpec, parse_netspec};
use crate::net::IpNetwork;


struct NetworkIter<A: IpAddress> {
    is_empty: bool,
    unraveled_addr: A,
    last_unraveled_addr: A,
    subnet_mask: A,
}
impl<A: IpAddress> NetworkIter<A> {
    pub fn new(network: IpNetwork<A>) -> Self {
        let unraveled_addr = unravel_address(network.base_addr(), network.subnet_mask());
        let last_unraveled_addr = unravel_address(network.last_addr_of_subnet(), network.subnet_mask());
        Self {
            is_empty: false,
            unraveled_addr,
            last_unraveled_addr,
            subnet_mask: network.subnet_mask(),
        }
    }
}
impl<A: IpAddress> Iterator for NetworkIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty {
            return None;
        }

        if self.unraveled_addr > self.last_unraveled_addr {
            return None;
        }

        let woven_addr = weave_address(self.unraveled_addr, self.subnet_mask);
        if let Some(next_addr) = self.unraveled_addr.add_offset(1) {
            self.unraveled_addr = next_addr;
        } else {
            self.is_empty = true;
        }

        Some(woven_addr)
    }
}

pub fn enumerate(args: &[String]) -> i32 {
    // ripcalc --enumerate IPNETWORK...
    if args.len() < 3 {
        usage();
        return 1;
    }

    let mut ret: i32 = 0;
    for net_str in &args[2..] {
        match parse_netspec(net_str) {
            Err(e) => {
                eprintln!("failed to parse network {:?}: {}", net_str, e);
                ret = 1;
            },
            Ok(NetworkSpec::Ipv4(_addr, net)) => {
                let iterator = NetworkIter::new(net);
                for addr in iterator {
                    println!("{}", addr);
                }
            },
            Ok(NetworkSpec::Ipv6(_addr, net)) => {
                let iterator = NetworkIter::new(net);
                for addr in iterator {
                    println!("{}", addr);
                }
            },
        };
    }

    ret
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::net::test::{
        parse_ipv4, parse_ipv4net, parse_ipv4netm, parse_ipv6, parse_ipv6net, parse_ipv6netm,
    };

    #[test]
    fn test_enumerate_subnet() {
        let mut iter = NetworkIter::new(parse_ipv4net("192.0.2.64", 28));
        assert_eq!(Some(parse_ipv4("192.0.2.64")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.65")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.66")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.67")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.68")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.69")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.70")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.71")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.72")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.73")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.74")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.75")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.76")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.77")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.78")), iter.next());
        assert_eq!(Some(parse_ipv4("192.0.2.79")), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());

        let mut iter = NetworkIter::new(parse_ipv6net("2001:db8::10", 124));
        assert_eq!(Some(parse_ipv6("2001:db8::10")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::11")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::12")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::13")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::14")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::15")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::16")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::17")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::18")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::19")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1a")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1b")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1c")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1d")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1e")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1f")), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_enumerate_mixed_subnet() {
        let mut iter = NetworkIter::new(parse_ipv4netm("192.64.2.0", "255.240.255.255"));
        assert_eq!(Some(parse_ipv4("192.64.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.65.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.66.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.67.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.68.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.69.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.70.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.71.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.72.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.73.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.74.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.75.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.76.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.77.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.78.2.0")), iter.next());
        assert_eq!(Some(parse_ipv4("192.79.2.0")), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());

        let mut iter = NetworkIter::new(parse_ipv6netm("2001:db8::1042", "ffff:ffff:ffff:ffff:ffff:ffff:ffff:f0ff"));
        assert_eq!(Some(parse_ipv6("2001:db8::1042")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1142")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1242")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1342")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1442")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1542")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1642")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1742")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1842")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1942")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1a42")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1b42")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1c42")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1d42")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1e42")), iter.next());
        assert_eq!(Some(parse_ipv6("2001:db8::1f42")), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }
}
