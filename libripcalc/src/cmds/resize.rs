use std::cmp::Ordering;
use std::convert::TryInto;

use crate::addr::{IpAddress, Ipv4Address, Ipv6Address};
use crate::bit_manip::{unravel_address, weave_address};
use crate::cidr::subnet_mask_bytes_from_prefix;
use crate::cmds::{CommandResult, NetworkSpec, ParsedSubnet, parse_netspec, parse_subnet};
use crate::cmds::show_net::{output_ipv4_network, output_ipv6_network};
use crate::net::IpNetwork;
use crate::output::Output;


pub fn resize<S: AsRef<str>, O: Output, E: Output>(args: &[S], stdout: &mut O, stderr: &mut E) -> CommandResult {
    if args.len() != 4 {
        // ripcalc --resize IPADDRESS/SUBNET SUBNET
        return CommandResult::WrongUsage;
    }

    match parse_netspec(args[2].as_ref()) {
        Err(e) => {
            writeln!(stderr, "failed to parse network spec {:?}: {}", args[2].as_ref(), e).unwrap();
            CommandResult::Error(1)
        },
        Ok(NetworkSpec::Ipv4(_addr, net)) => {
            let mask = match parse_subnet(args[3].as_ref()) {
                Err(e) => {
                    writeln!(stderr, "failed to parse subnet {:?}: {}", args[3].as_ref(), e).unwrap();
                    return CommandResult::Error(1);
                },
                Ok(ParsedSubnet::Cidr(cidr)) => {
                    if cidr > 32 {
                        writeln!(stderr, "CIDR value {} is greater than maximum for IPv4 (32)", cidr).unwrap();
                        return CommandResult::Error(1);
                    }
                    let mask_bytes = subnet_mask_bytes_from_prefix(cidr, 4);
                    Ipv4Address::from_bytes(&mask_bytes).unwrap()
                },
                Ok(ParsedSubnet::Ipv4Mask(m)) => {
                    m
                },
                Ok(ParsedSubnet::Ipv6Mask(_)) => {
                    writeln!(stderr, "cannot resize an IPv4 subnet to an IPv6 mask").unwrap();
                    return CommandResult::Error(1);
                },
            };
            resize_and_output(net, mask, output_ipv4_network, stdout);
            CommandResult::Ok
        },
        Ok(NetworkSpec::Ipv6(_addr, net)) => {
            let mask = match parse_subnet(args[3].as_ref()) {
                Err(e) => {
                    writeln!(stderr, "failed to parse subnet {:?}: {}", args[3].as_ref(), e).unwrap();
                    return CommandResult::Error(1);
                },
                Ok(ParsedSubnet::Cidr(cidr)) => {
                    if cidr > 128 {
                        writeln!(stderr, "CIDR value {} is greater than maximum for IPv6 (128)", cidr).unwrap();
                        return CommandResult::Error(1);
                    }
                    let mask_bytes = subnet_mask_bytes_from_prefix(cidr, 16);
                    Ipv6Address::from_bytes(&mask_bytes).unwrap()
                },
                Ok(ParsedSubnet::Ipv6Mask(m)) => {
                    m
                },
                Ok(ParsedSubnet::Ipv4Mask(_)) => {
                    writeln!(stderr, "cannot resize an IPv6 subnet to an IPv4 mask").unwrap();
                    return CommandResult::Error(1);
                },
            };
            resize_and_output(net, mask, output_ipv6_network, stdout);
            CommandResult::Ok
        },
    }
}

fn resize_and_output<
    A: IpAddress,
    ON: Fn(IpNetwork<A>, Option<A>, &mut O),
    O: Output,
>(
    initial_net: IpNetwork<A>,
    new_subnet_mask: A,
    output_network: ON,
    stdout: &mut O,
) {
    let (resized, net_ordering) = resize_network(initial_net, new_subnet_mask);

    writeln!(stdout, "Original network:").unwrap();
    output_network(initial_net, None, stdout);
    writeln!(stdout).unwrap();

    match net_ordering {
        Ordering::Less => {
            writeln!(stdout, "Supernet:").unwrap();
            output_network(resized[0], None, stdout);
            writeln!(stdout).unwrap();
        },
        Ordering::Equal => {
            writeln!(stdout, "Same-sized net:").unwrap();
            output_network(resized[0], None, stdout);
            writeln!(stdout).unwrap();
        },
        Ordering::Greater => {
            for i in 0..resized.len() {
                writeln!(stdout, "Subnet {}:", i+1).unwrap();
                output_network(resized[i], None, stdout);
                writeln!(stdout).unwrap();
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
                let unraveled_nsba = unravel_address(nsba, new_net.subnet_mask());
                current_unraveled_base_addr = unraveled_nsba;
            } else {
                break;
            }
        }
        (nets, Ordering::Greater)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::net::test::{
        parse_ipv4, parse_ipv4net, parse_ipv4netm, parse_ipv6, parse_ipv6net, parse_ipv6netm,
    };

    #[test]
    fn test_resize_ipv4() {
        // 1:1
        let (resized, ordure) = resize_network(
            parse_ipv4net("192.0.2.0", 24),
            parse_ipv4("255.255.255.0"),
        );
        assert_eq!(Ordering::Equal, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv4net("192.0.2.0", 24), resized[0]);

        // shifting bits
        let (resized, ordure) = resize_network(
            parse_ipv4net("192.0.2.0", 24),
            parse_ipv4("255.0.255.255"),
        );
        assert_eq!(Ordering::Equal, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv4netm("192.0.0.2", "255.0.255.255"), resized[0]);

        // subnets
        let (resized, ordure) = resize_network(
            parse_ipv4net("192.0.2.0", 24),
            parse_ipv4("255.255.255.192"),
        );
        assert_eq!(Ordering::Greater, ordure);
        assert_eq!(4, resized.len());
        assert_eq!(parse_ipv4net("192.0.2.0", 26), resized[0]);
        assert_eq!(parse_ipv4net("192.0.2.64", 26), resized[1]);
        assert_eq!(parse_ipv4net("192.0.2.128", 26), resized[2]);
        assert_eq!(parse_ipv4net("192.0.2.192", 26), resized[3]);

        // subnets shifting bits
        let (resized, ordure) = resize_network(
            parse_ipv4net("192.0.2.0", 24),
            parse_ipv4("255.255.192.255"),
        );
        assert_eq!(Ordering::Greater, ordure);
        assert_eq!(4, resized.len());
        assert_eq!(parse_ipv4netm("192.0.0.8", "255.255.192.255"), resized[0]);
        assert_eq!(parse_ipv4netm("192.0.0.9", "255.255.192.255"), resized[1]);
        assert_eq!(parse_ipv4netm("192.0.0.10", "255.255.192.255"), resized[2]);
        assert_eq!(parse_ipv4netm("192.0.0.11", "255.255.192.255"), resized[3]);

        // supernet
        let (resized, ordure) = resize_network(
            parse_ipv4net("192.0.2.0", 24),
            parse_ipv4("255.255.0.0"),
        );
        assert_eq!(Ordering::Less, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv4net("192.0.0.0", 16), resized[0]);

        // supernet shifting bits
        let (resized, ordure) = resize_network(
            parse_ipv4net("192.0.2.0", 24),
            parse_ipv4("255.0.255.0"),
        );
        assert_eq!(Ordering::Less, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv4netm("192.0.0.0", "255.0.255.0"), resized[0]);
    }

    #[test]
    fn test_resize_ipv6() {
        // 1:1
        let (resized, ordure) = resize_network(
            parse_ipv6net("2001:db8::", 64),
            parse_ipv6("ffff:ffff:ffff:ffff::"),
        );
        assert_eq!(Ordering::Equal, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv6net("2001:db8::", 64), resized[0]);

        // shifting bits
        let (resized, ordure) = resize_network(
            parse_ipv6net("2001:db8::", 64),
            parse_ipv6("ffff:0:ffff:ffff:ffff::"),
        );
        assert_eq!(Ordering::Equal, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv6netm("2001:0:db8::", "ffff:0:ffff:ffff:ffff::"), resized[0]);

        // subnets
        let (resized, ordure) = resize_network(
            parse_ipv6net("2001:db8::", 64),
            parse_ipv6("ffff:ffff:ffff:ffff:c000::"),
        );
        assert_eq!(Ordering::Greater, ordure);
        assert_eq!(4, resized.len());
        assert_eq!(parse_ipv6net("2001:db8::", 66), resized[0]);
        assert_eq!(parse_ipv6net("2001:db8:0:0:4000::", 66), resized[1]);
        assert_eq!(parse_ipv6net("2001:db8:0:0:8000::", 66), resized[2]);
        assert_eq!(parse_ipv6net("2001:db8:0:0:c000::", 66), resized[3]);

        // subnets shifting bits
        let (resized, ordure) = resize_network(
            parse_ipv6net("2001:db8::", 64),
            parse_ipv6("ffff:ffff:ffff:ffff:0:c000::"),
        );
        assert_eq!(Ordering::Greater, ordure);
        assert_eq!(4, resized.len());
        assert_eq!(parse_ipv6netm("2001:db8::", "ffff:ffff:ffff:ffff:0:c000::"), resized[0]);
        assert_eq!(parse_ipv6netm("2001:db8:0:0:0:4000::", "ffff:ffff:ffff:ffff:0:c000::"), resized[1]);
        assert_eq!(parse_ipv6netm("2001:db8:0:0:0:8000::", "ffff:ffff:ffff:ffff:0:c000::"), resized[2]);
        assert_eq!(parse_ipv6netm("2001:db8:0:0:0:c000::", "ffff:ffff:ffff:ffff:0:c000::"), resized[3]);

        // supernet
        let (resized, ordure) = resize_network(
            parse_ipv6net("2001:db8:1234:1234::", 64),
            parse_ipv6("ffff:ffff:ffff::"),
        );
        assert_eq!(Ordering::Less, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv6net("2001:db8:1234::", 48), resized[0]);

        // supernet shifting bits
        let (resized, ordure) = resize_network(
            parse_ipv6net("2001:db8:1234:1234::", 64),
            parse_ipv6("ffff:ffff:0:ffff::"),
        );
        assert_eq!(Ordering::Less, ordure);
        assert_eq!(1, resized.len());
        assert_eq!(parse_ipv6netm("2001:db8:0:1234::", "ffff:ffff:0:ffff::"), resized[0]);
    }
}
