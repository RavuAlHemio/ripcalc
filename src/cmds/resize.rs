use std::cmp::Ordering;
use std::convert::TryInto;

use crate::usage;
use crate::addr::{IpAddress, Ipv4Address, Ipv6Address};
use crate::bit_manip::{unravel_address, weave_address};
use crate::cidr::subnet_mask_bytes_from_prefix;
use crate::cmds::{NetworkSpec, ParsedSubnet, parse_netspec, parse_subnet};
use crate::cmds::show_net::{output_ipv4_network, output_ipv6_network};
use crate::net::IpNetwork;


pub fn resize(args: &[String]) -> i32 {
    if args.len() != 4 {
        // ripcalc --resize IPADDRESS/SUBNET SUBNET
        usage();
        return 1;
    }

    match parse_netspec(&args[2]) {
        Err(e) => {
            eprintln!("failed to parse network spec {:?}: {}", args[2], e);
            1
        },
        Ok(NetworkSpec::Ipv4(_addr, net)) => {
            let mask = match parse_subnet(&args[3]) {
                Err(e) => {
                    eprintln!("failed to parse subnet {:?}: {}", args[3], e);
                    return 1;
                },
                Ok(ParsedSubnet::Cidr(cidr)) => {
                    if cidr > 32 {
                        eprintln!("CIDR value {} is greater than maximum for IPv4 (32)", cidr);
                        return 1;
                    }
                    let mask_bytes = subnet_mask_bytes_from_prefix(cidr, 4);
                    Ipv4Address::from_bytes(&mask_bytes).unwrap()
                },
                Ok(ParsedSubnet::Ipv4Mask(m)) => {
                    m
                },
                Ok(ParsedSubnet::Ipv6Mask(_)) => {
                    eprintln!("cannot resize an IPv4 subnet to an IPv6 mask");
                    return 1;
                },
            };
            resize_and_output(net, mask, output_ipv4_network);
            0
        },
        Ok(NetworkSpec::Ipv6(_addr, net)) => {
            let mask = match parse_subnet(&args[3]) {
                Err(e) => {
                    eprintln!("failed to parse subnet {:?}: {}", args[3], e);
                    return 1;
                },
                Ok(ParsedSubnet::Cidr(cidr)) => {
                    if cidr > 128 {
                        eprintln!("CIDR value {} is greater than maximum for IPv6 (128)", cidr);
                        return 1;
                    }
                    let mask_bytes = subnet_mask_bytes_from_prefix(cidr, 16);
                    Ipv6Address::from_bytes(&mask_bytes).unwrap()
                },
                Ok(ParsedSubnet::Ipv6Mask(m)) => {
                    m
                },
                Ok(ParsedSubnet::Ipv4Mask(_)) => {
                    eprintln!("cannot resize an IPv6 subnet to an IPv4 mask");
                    return 1;
                },
            };
            resize_and_output(net, mask, output_ipv6_network);
            0
        },
    }
}

fn resize_and_output<A: IpAddress, ON: Fn(IpNetwork<A>, Option<A>)>(initial_net: IpNetwork<A>, new_subnet_mask: A, output_network: ON) {
    let (resized, net_ordering) = resize_network(initial_net, new_subnet_mask);

    println!("Original network:");
    output_network(initial_net, None);
    println!();

    match net_ordering {
        Ordering::Less => {
            println!("Supernet:");
            output_network(resized[0], None);
            println!();
        },
        Ordering::Equal => {
            println!("Same-sized net:");
            output_network(resized[0], None);
            println!();
        },
        Ordering::Greater => {
            for i in 0..resized.len() {
                println!("Subnet {}:", i+1);
                output_network(resized[i], None);
                println!();
            }
        },
    }
}

/// Resizes the given network to the given subnet mask, returning the network or networks created by
/// this operation as well as whether a supernet, a same-sized net or multiple subnets were created.
pub fn resize_network<A: IpAddress>(initial_net: IpNetwork<A>, new_subnet_mask: A) -> (Vec<IpNetwork<A>>, Ordering) {
    let initial_host_bits = initial_net.subnet_mask().count_zeros();
    let new_net_bits: usize = new_subnet_mask.count_ones().try_into().unwrap();
    let new_host_bits = new_subnet_mask.count_zeros();

    if new_host_bits > initial_host_bits {
        // supernet
        let unraveled_initial_base_addr = unravel_address(initial_net.base_addr(), initial_net.subnet_mask());
        let unraveled_shortened_net = IpNetwork::new_with_prefix(unraveled_initial_base_addr, new_net_bits);
        let woven_new_base_addr = weave_address(unraveled_shortened_net.base_addr(), new_subnet_mask);
        let new_net = IpNetwork::new_with_mask(woven_new_base_addr, new_subnet_mask);

        let mut nets = Vec::new();
        nets.push(new_net);
        (nets, Ordering::Less)
    } else if new_host_bits == initial_host_bits {
        // samenet
        let unraveled_base_addr = unravel_address(initial_net.base_addr(), initial_net.subnet_mask());
        let woven_new_base_addr = weave_address(unraveled_base_addr, new_subnet_mask);
        let new_net = IpNetwork::new_with_mask(woven_new_base_addr, new_subnet_mask);

        let mut nets = Vec::new();
        nets.push(new_net);
        (nets, Ordering::Equal)
    } else {
        // subnet(s)

        let unraveled_base_addr = unravel_address(initial_net.base_addr(), initial_net.subnet_mask());
        let unraveled_last_addr = unravel_address(initial_net.last_addr_of_subnet(), initial_net.subnet_mask());

        let mut nets = Vec::new();

        let mut current_unraveled_base_addr = unraveled_base_addr;
        while current_unraveled_base_addr <= unraveled_last_addr {
            let woven_new_base_addr = weave_address(current_unraveled_base_addr, new_subnet_mask);
            let new_net = IpNetwork::new_with_mask(woven_new_base_addr, new_subnet_mask);

            nets.push(new_net);

            if let Some(nsba) = new_net.next_subnet_base_addr() {
                current_unraveled_base_addr = nsba;
            } else {
                break;
            }
        }
        (nets, Ordering::Greater)
    }
}
