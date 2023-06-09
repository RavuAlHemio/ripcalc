use std::collections::HashMap;

use num_bigint::BigInt;

use crate::usage;
use crate::addr::IpAddress;
use crate::cmds::{NetworkSpec, parse_netspec};
use crate::cmds::derange::range_to_subnets;
use crate::cmds::show_net::{output_ipv4_network, output_ipv6_network};
use crate::net::IpNetwork;


pub fn split(args: &[String]) -> i32 {
    // ripcalc --split IPADDRESS/CIDRPREFIX HOSTCOUNT...
    if args.len() < 4 {
        usage();
        return 1;
    }

    let zero = BigInt::from(0);

    let mut host_counts: Vec<BigInt> = Vec::with_capacity(args.len() - 3);
    for count_str in &args[3..] {
        let host_count: BigInt = match count_str.parse() {
            Ok(bu) => bu,
            Err(e) => {
                eprintln!("failed to parse host count {:?}: {}", count_str, e);
                return 1;
            },
        };
        if host_count < zero {
            eprintln!("host counts must be zero or greater");
            return 1;
        }
        host_counts.push(host_count);
    }

    match parse_netspec(&args[2]) {
        Err(e) => {
            eprintln!("failed to parse network specification {:?}: {}", args[2], e);
            1
        },
        Ok(NetworkSpec::Ipv4(_addr, net)) => {
            output_split(net, host_counts, output_ipv4_network)
        },
        Ok(NetworkSpec::Ipv6(_addr, net)) => {
            output_split(net, host_counts, output_ipv6_network)
        },
    }
}

fn output_split<A: IpAddress, ON: Fn(IpNetwork<A>, Option<A>)>(subnet: IpNetwork<A>, host_counts: Vec<BigInt>, output_network: ON) -> i32 {
    println!("Subnet to split:");
    output_network(subnet, None);
    println!();

    let split_subnets = match split_subnet(subnet, host_counts.clone()) {
        Some(s) => s,
        None => {
            println!("Not enough addresses available for this split.");
            return 1;
        },
    };
    for (host_count, splitnet) in host_counts.iter().zip(&split_subnets) {
        println!("Subnet for {} hosts:", host_count);
        output_network(*splitnet, None);
        println!();
    }

    let max_used_address = split_subnets.iter()
        .map(|sn| sn.last_addr_of_subnet())
        .max()
        .expect("no subnets returned");
    if !subnet.contains(&max_used_address) {
        println!("Network is too small");
    } else if let Some(next_unused_address) = max_used_address.add_offset(1) {
        println!("Unused networks:");
        let last_address = subnet.last_addr_of_subnet();
        let unused_subnets = range_to_subnets(next_unused_address, last_address);

        for unused_subnet in unused_subnets {
            println!("{}", unused_subnet);
        }
    }

    0
}

/// Splits a larger network into smaller networks, each housing at least a specific number of hosts.
pub fn split_subnet<A: IpAddress>(subnet: IpNetwork<A>, host_counts: Vec<BigInt>) -> Option<Vec<IpNetwork<A>>> {
    // sort descending by size
    let mut indexes_and_host_counts: Vec<(usize, BigInt)> = host_counts.iter()
        .enumerate()
        .map(|(i, num)| (i, num.clone()))
        .collect();
    indexes_and_host_counts.sort_unstable_by(|(_i1, num1), (_i2, num2)|
        // descending sort => reversed
        num2.cmp(num1)
    );

    let mut index_to_subnet: HashMap<usize, IpNetwork<A>> = HashMap::new();

    let mut current_net = IpNetwork::new_with_prefix(subnet.base_addr(), subnet.subnet_mask().byte_count()*8);
    for (i, host_count) in indexes_and_host_counts {
        while current_net.host_count() < host_count {
            let cidr_prefix = current_net.cidr_prefix().unwrap();
            if cidr_prefix == 0 {
                break;
            }
            current_net = IpNetwork::new_with_prefix(current_net.base_addr(), cidr_prefix - 1);
        }

        if current_net.cidr_prefix().unwrap() == 0 {
            // this won't fit
            return None;
        }

        // we fit!
        index_to_subnet.insert(i, current_net);
        let next_subnet_base_addr = match current_net.next_subnet_base_addr() {
            Some(nsba) => nsba,
            None => return None,
        };
        current_net = IpNetwork::new_with_prefix(next_subnet_base_addr, current_net.subnet_mask().byte_count()*8);
    }

    let mut ordered_subnets: Vec<(usize, IpNetwork<A>)> = index_to_subnet.iter()
        .map(|(i, net)| (*i, *net))
        .collect();
    ordered_subnets.sort_unstable_by_key(|(i, _net)| *i);
    let ret = ordered_subnets.iter()
        .map(|(_i, net)| *net)
        .collect();

    Some(ret)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::net::test::{
        parse_ipv4net, parse_ipv6net, parse_bigint,
    };

    #[test]
    fn test_split_ipv4() {
        // single smaller net
        let nets = split_subnet(
            parse_ipv4net("192.0.2.0", 24),
            vec![10.into()],
        )
            .unwrap();
        assert_eq!(1, nets.len());
        assert_eq!(parse_ipv4net("192.0.2.0", 28), nets[0]);

        // multiple smaller nets of the same size, fitting
        let nets = split_subnet(
            parse_ipv4net("192.0.2.0", 24),
            vec![60.into(), 60.into(), 60.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(4, nets.len());
        assert_eq!(parse_ipv4net("192.0.2.0", 26), nets[0]);
        assert_eq!(parse_ipv4net("192.0.2.64", 26), nets[1]);
        assert_eq!(parse_ipv4net("192.0.2.128", 26), nets[2]);
        assert_eq!(parse_ipv4net("192.0.2.192", 26), nets[3]);

        // multiple smaller nets of the same size, not fitting
        let nets = split_subnet(
            parse_ipv4net("192.0.2.0", 24),
            vec![60.into(), 60.into(), 60.into(), 60.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(5, nets.len());
        assert_eq!(parse_ipv4net("192.0.2.0", 26), nets[0]);
        assert_eq!(parse_ipv4net("192.0.2.64", 26), nets[1]);
        assert_eq!(parse_ipv4net("192.0.2.128", 26), nets[2]);
        assert_eq!(parse_ipv4net("192.0.2.192", 26), nets[3]);
        assert_eq!(parse_ipv4net("192.0.3.0", 26), nets[4]);

        // multiple smaller nets of different sizes
        let nets = split_subnet(
            parse_ipv4net("192.0.2.0", 24),
            vec![60.into(), 100.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(3, nets.len());
        assert_eq!(parse_ipv4net("192.0.2.128", 26), nets[0]);
        assert_eq!(parse_ipv4net("192.0.2.0", 25), nets[1]);
        assert_eq!(parse_ipv4net("192.0.2.192", 26), nets[2]);

        // too many hosts
        let nets = split_subnet(
            parse_ipv4net("192.0.2.0", 24),
            vec![60.into(), 100.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(3, nets.len());
        assert_eq!(parse_ipv4net("192.0.2.128", 26), nets[0]);
        assert_eq!(parse_ipv4net("192.0.2.0", 25), nets[1]);
        assert_eq!(parse_ipv4net("192.0.2.192", 26), nets[2]);

        // too many hosts for address space
        let none_subnet = split_subnet(
            parse_ipv4net("192.0.2.0", 24),
            vec![8589934592u64.into()],
        );
        assert!(none_subnet.is_none());
    }

    #[test]
    fn test_resize_ipv6() {
        // single smaller net
        let nets = split_subnet(
            parse_ipv6net("2001:db8::", 64),
            vec![10.into()],
        )
            .unwrap();
        assert_eq!(1, nets.len());
        assert_eq!(parse_ipv6net("2001:db8::", 124), nets[0]);

        // multiple smaller nets of the same size, fitting
        let nets = split_subnet(
            parse_ipv6net("2001:db8::", 64),
            vec![60.into(), 60.into(), 60.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(4, nets.len());
        assert_eq!(parse_ipv6net("2001:db8::", 122), nets[0]);
        assert_eq!(parse_ipv6net("2001:db8::40", 122), nets[1]);
        assert_eq!(parse_ipv6net("2001:db8::80", 122), nets[2]);
        assert_eq!(parse_ipv6net("2001:db8::c0", 122), nets[3]);

        // multiple smaller nets of the same size, not fitting
        let nets = split_subnet(
            parse_ipv6net("2001:db8::", 121),
            vec![60.into(), 60.into(), 60.into(), 60.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(5, nets.len());
        assert_eq!(parse_ipv6net("2001:db8::", 122), nets[0]);
        assert_eq!(parse_ipv6net("2001:db8::40", 122), nets[1]);
        assert_eq!(parse_ipv6net("2001:db8::80", 122), nets[2]);
        assert_eq!(parse_ipv6net("2001:db8::c0", 122), nets[3]);
        assert_eq!(parse_ipv6net("2001:db8::100", 122), nets[4]);

        // multiple smaller nets of different sizes
        let nets = split_subnet(
            parse_ipv6net("2001:db8::", 64),
            vec![60.into(), 100.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(3, nets.len());
        assert_eq!(parse_ipv6net("2001:db8::80", 122), nets[0]);
        assert_eq!(parse_ipv6net("2001:db8::", 121), nets[1]);
        assert_eq!(parse_ipv6net("2001:db8::c0", 122), nets[2]);

        // too many hosts
        let nets = split_subnet(
            parse_ipv6net("2001:db8::", 121),
            vec![60.into(), 100.into(), 60.into()],
        )
            .unwrap();
        assert_eq!(3, nets.len());
        assert_eq!(parse_ipv6net("2001:db8::80", 122), nets[0]);
        assert_eq!(parse_ipv6net("2001:db8::", 121), nets[1]);
        assert_eq!(parse_ipv6net("2001:db8::c0", 122), nets[2]);

        // too many hosts for address space
        let none_subnet = split_subnet(
            parse_ipv6net("2001:db8::", 64),
            vec![parse_bigint("680564733841876926926749214863536422912")],
        );
        assert!(none_subnet.is_none());
    }
}
