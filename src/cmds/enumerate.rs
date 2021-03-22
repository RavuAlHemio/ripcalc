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
