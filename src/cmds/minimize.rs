use std::collections::HashSet;

use crate::usage;
use crate::addr::IpAddress;
use crate::cmds::{NetworkSpecs, parse_same_family_netspecs};
use crate::net::IpNetwork;


pub fn minimize(args: &[String]) -> i32 {
    // ripcalc --minimize IPADDRESS/SUBNET...
    if args.len() < 3 {
        usage();
        return 1;
    }

    match parse_same_family_netspecs(&args[2..]) {
        Ok(NetworkSpecs::Nothing) => {
            0
        },
        Ok(NetworkSpecs::MixedSpecs) => {
            eprintln!("mixing IPv4 and IPv6 is not supported");
            1
        },
        Ok(NetworkSpecs::Ipv4(addrs_subnets)) => {
            let subnets = addrs_subnets.iter()
                .map(|(_a, s)| *s)
                .collect();
            let minimized = minimize_subnets(subnets);
            for min_net in minimized {
                println!("{}", min_net);
            }
            0
        },
        Ok(NetworkSpecs::Ipv6(addrs_subnets)) => {
            let subnets = addrs_subnets.iter()
                .map(|(_a, s)| *s)
                .collect();
            let minimized = minimize_subnets(subnets);
            for min_net in minimized {
                println!("{}", min_net);
            }
            0
        },
        Err(e) => {
            eprintln!("parsing error: {}", e);
            1
        },
    }
}

/// Minimizes the list of networks such that duplicate entries and networks that are subnets of
/// other networks in the list are removed from the list, and adjacent networks are merged if
/// possible.
pub fn minimize_subnets<A: IpAddress>(
    mut subnets: Vec<IpNetwork<A>>,
) -> Vec<IpNetwork<A>> {
    subnets.sort_unstable_by_key(|net| (net.base_addr(), net.subnet_mask()));

    let mut filtered_subnets: HashSet<IpNetwork<A>> = HashSet::new();
    filtered_subnets.extend(subnets.iter());

    // eliminate subnets
    for i in 0..subnets.len() {
        for j in (i+1)..subnets.len() {
            if subnets[i].is_superset_of(&subnets[j]) && subnets[i] != subnets[j] {
                // i is a subset of j
                filtered_subnets.remove(&subnets[j]);
            }
        }
    }

    // try joining adjacent same-size subnets
    let mut subnets_merged = true;
    while subnets_merged {
        subnets_merged = false;

        subnets = filtered_subnets.iter()
            .map(|net| *net)
            .collect();
        subnets.sort_unstable_by_key(|net| (net.base_addr(), net.subnet_mask()));

        for i in 0..subnets.len() {
            for j in (i+1)..subnets.len() {
                if subnets[i].subnet_mask() != subnets[j].subnet_mask() {
                    // not the same size
                    continue;
                }

                if let Some(last_ip_plus_one) = subnets[i].next_subnet_base_addr() {
                    if last_ip_plus_one != subnets[j].base_addr() {
                        // not adjacent
                        continue;
                    }
                }

                // adjacent!

                // which bit do they differ in?
                let differ_bit_address: A = subnets[i].base_addr() ^ subnets[j].base_addr();

                // ensure it's only one bit
                let difference_pop_count = differ_bit_address.count_ones();
                if difference_pop_count > 1 {
                    // not just a single-bit difference
                    continue;
                }

                // remove that bit from the subnet mask
                let new_subnet_mask: A = subnets[i].subnet_mask() & differ_bit_address.bitwise_negate();
                let new_subnet = IpNetwork::new_with_mask(subnets[i].base_addr(), new_subnet_mask);

                // quick sanity check
                assert!(new_subnet.is_superset_of(&subnets[i]));
                assert!(new_subnet.is_superset_of(&subnets[j]));

                // replace the lower subnets with the upper subnet
                filtered_subnets.remove(&subnets[i]);
                filtered_subnets.remove(&subnets[j]);
                filtered_subnets.insert(new_subnet);

                subnets_merged = true;
                break;
            }

            if subnets_merged {
                break;
            }
        }
    }

    subnets = filtered_subnets.iter()
        .map(|net| *net)
        .collect();
    subnets.sort_unstable_by_key(|net| (net.base_addr(), net.subnet_mask()));
    subnets
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::addr::Ipv4Address;

    fn parse_ipv4net(addr_str: &str, cidr: usize) -> IpNetwork<Ipv4Address> {
        IpNetwork::new_with_prefix(addr_str.parse().unwrap(), cidr)
    }
    fn parse_ipv4netm(addr_str: &str, mask_str: &str) -> IpNetwork<Ipv4Address> {
        IpNetwork::new_with_mask(addr_str.parse().unwrap(), mask_str.parse().unwrap())
    }

    #[test]
    fn test_minimize() {
        let minimize_us = vec![
            parse_ipv4net("128.130.0.0", 16),
            parse_ipv4net("128.131.0.0", 16),
            parse_ipv4net("128.132.0.0", 16),
            parse_ipv4net("128.133.0.0", 16),
            parse_ipv4net("192.35.240.0", 22),
            parse_ipv4net("192.35.244.0", 24),
            parse_ipv4net("193.170.72.0", 21),
            parse_ipv4net("193.170.72.0", 22),
            parse_ipv4net("193.170.76.0", 23),
            parse_ipv4net("193.170.78.0", 24),
        ];
        let minimized = minimize_subnets(minimize_us);
        assert_eq!(5, minimized.len());
        assert_eq!(parse_ipv4net("128.130.0.0", 15), minimized[0]);
        assert_eq!(parse_ipv4net("128.132.0.0", 15), minimized[1]);
        assert_eq!(parse_ipv4net("192.35.240.0", 22), minimized[2]);
        assert_eq!(parse_ipv4net("192.35.244.0", 24), minimized[3]);
        assert_eq!(parse_ipv4net("193.170.72.0", 21), minimized[4]);
    }

    #[test]
    fn test_minimize_mixed() {
        let minimize_us = vec![
            parse_ipv4netm("128.0.0.130", "255.0.0.255"),
            parse_ipv4netm("128.0.0.131", "255.0.0.255"),
            parse_ipv4netm("128.0.0.132", "255.0.0.255"),
            parse_ipv4netm("128.0.0.133", "255.0.0.255"),
        ];
        let minimized = minimize_subnets(minimize_us);
        assert_eq!(2, minimized.len());
        assert_eq!(parse_ipv4netm("128.0.0.130", "255.0.0.254"), minimized[0]);
        assert_eq!(parse_ipv4netm("128.0.0.132", "255.0.0.254"), minimized[1]);
    }
}
