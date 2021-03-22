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
    if let Some(next_unused_address) = max_used_address.add_offset(1) {
        println!("Unused networks:");
        let last_address = subnet.last_addr_of_subnet();
        let unused_subnets = range_to_subnets(next_unused_address, last_address);

        for unused_subnet in unused_subnets {
            println!("{}", unused_subnet);
        }
    }

    0
}

fn split_subnet<A: IpAddress>(subnet: IpNetwork<A>, host_counts: Vec<BigInt>) -> Option<Vec<IpNetwork<A>>> {
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
