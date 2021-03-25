use std::cmp::{max, min};

use crate::usage;
use crate::addr::IpAddress;
use crate::cmds::{parse_addr, ParsedIpAddress};
use crate::net::IpNetwork;


pub fn derange(args: &[String]) -> i32 {
    // ripcalc --derange ONE OTHER
    if args.len() != 4 {
        usage();
        return 1;
    }

    let one = match parse_addr(&args[2]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("failed to parse first address: {}", e);
            return 1;
        },
    };
    let other = match parse_addr(&args[3]) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("failed to parse second address: {}", e);
            return 1;
        },
    };

    if one.version() != other.version() {
        eprintln!("both addresses must be the same version");
        return 1;
    } else if let ParsedIpAddress::Ipv4(one_addr) = one {
        if let ParsedIpAddress::Ipv4(other_addr) = other {
            let subnets = range_to_subnets(one_addr, other_addr);
            for subnet in subnets {
                println!("{}", subnet);
            }
        }
    } else if let ParsedIpAddress::Ipv6(one_addr) = one {
        if let ParsedIpAddress::Ipv6(other_addr) = other {
            let subnets = range_to_subnets(one_addr, other_addr);
            for subnet in subnets {
                println!("{}", subnet);
            }
        }
    }

    0
}


/// Converts a range of IP addresses (whose inclusive ends are passed as `end_one` and `end_two`)
/// into the equivalent set of IP networks.
pub fn range_to_subnets<A: IpAddress>(
    end_one: A,
    end_two: A,
) -> Vec<IpNetwork<A>> {
    let mut ret = Vec::new();

    let mut first_addr = min(end_one, end_two);
    let last_addr = max(end_one, end_two);

    // start with the full mask
    let mut current_subnet = IpNetwork::new_with_prefix(first_addr, last_addr.byte_count() * 8);
    while first_addr <= last_addr {
        // try enlarging the subnet
        let larger_subnet = IpNetwork::new_with_prefix(first_addr, current_subnet.cidr_prefix().unwrap() - 1);
        if larger_subnet.base_addr() != first_addr || larger_subnet.last_addr_of_subnet() > last_addr {
            // we've gone beyond; store what we have and continue with the next chunk
            ret.push(current_subnet);
            first_addr = current_subnet.next_subnet_base_addr().unwrap();
            current_subnet = IpNetwork::new_with_prefix(first_addr, last_addr.byte_count() * 8);
        } else {
            // anchor the growth and continue
            current_subnet = larger_subnet;
        }
    }

    ret
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::net::test::{parse_ipv4, parse_ipv4net, parse_ipv6, parse_ipv6net};


    #[test]
    fn test_derange_single_subnet() {
        let end_one = parse_ipv4("192.0.2.0");
        let end_two = parse_ipv4("192.0.2.255");
        let subnet = range_to_subnets(end_one, end_two);
        assert_eq!(1, subnet.len());
        assert_eq!(parse_ipv4net("192.0.2.0", 24), subnet[0]);

        let end_one = parse_ipv6("2001:db8::");
        let end_two = parse_ipv6("2001:db8::ffff:ffff:ffff:ffff");
        let subnet = range_to_subnets(end_one, end_two);
        assert_eq!(1, subnet.len());
        assert_eq!(parse_ipv6net("2001:db8::", 64), subnet[0]);
    }

    #[test]
    fn test_derange_multiple_subnets() {
        let end_one = parse_ipv4("192.0.2.1");
        let end_two = parse_ipv4("192.0.2.254");
        let subnet = range_to_subnets(end_one, end_two);
        assert_eq!(14, subnet.len());
        assert_eq!(parse_ipv4net("192.0.2.1", 32), subnet[0]);
        assert_eq!(parse_ipv4net("192.0.2.2", 31), subnet[1]);
        assert_eq!(parse_ipv4net("192.0.2.4", 30), subnet[2]);
        assert_eq!(parse_ipv4net("192.0.2.8", 29), subnet[3]);
        assert_eq!(parse_ipv4net("192.0.2.16", 28), subnet[4]);
        assert_eq!(parse_ipv4net("192.0.2.32", 27), subnet[5]);
        assert_eq!(parse_ipv4net("192.0.2.64", 26), subnet[6]);
        assert_eq!(parse_ipv4net("192.0.2.128", 26), subnet[7]);
        assert_eq!(parse_ipv4net("192.0.2.192", 27), subnet[8]);
        assert_eq!(parse_ipv4net("192.0.2.224", 28), subnet[9]);
        assert_eq!(parse_ipv4net("192.0.2.240", 29), subnet[10]);
        assert_eq!(parse_ipv4net("192.0.2.248", 30), subnet[11]);
        assert_eq!(parse_ipv4net("192.0.2.252", 31), subnet[12]);
        assert_eq!(parse_ipv4net("192.0.2.254", 32), subnet[13]);

        let end_one = parse_ipv6("2001:db8::1");
        let end_two = parse_ipv6("2001:db8::fffe");
        let subnet = range_to_subnets(end_one, end_two);
        assert_eq!(30, subnet.len());
        assert_eq!(parse_ipv6net("2001:db8::1", 128), subnet[0]);
        assert_eq!(parse_ipv6net("2001:db8::2", 127), subnet[1]);
        assert_eq!(parse_ipv6net("2001:db8::4", 126), subnet[2]);
        assert_eq!(parse_ipv6net("2001:db8::8", 125), subnet[3]);
        assert_eq!(parse_ipv6net("2001:db8::10", 124), subnet[4]);
        assert_eq!(parse_ipv6net("2001:db8::20", 123), subnet[5]);
        assert_eq!(parse_ipv6net("2001:db8::40", 122), subnet[6]);
        assert_eq!(parse_ipv6net("2001:db8::80", 121), subnet[7]);
        assert_eq!(parse_ipv6net("2001:db8::100", 120), subnet[8]);
        assert_eq!(parse_ipv6net("2001:db8::200", 119), subnet[9]);
        assert_eq!(parse_ipv6net("2001:db8::400", 118), subnet[10]);
        assert_eq!(parse_ipv6net("2001:db8::800", 117), subnet[11]);
        assert_eq!(parse_ipv6net("2001:db8::1000", 116), subnet[12]);
        assert_eq!(parse_ipv6net("2001:db8::2000", 115), subnet[13]);
        assert_eq!(parse_ipv6net("2001:db8::4000", 114), subnet[14]);
        assert_eq!(parse_ipv6net("2001:db8::8000", 114), subnet[15]);
        assert_eq!(parse_ipv6net("2001:db8::c000", 115), subnet[16]);
        assert_eq!(parse_ipv6net("2001:db8::e000", 116), subnet[17]);
        assert_eq!(parse_ipv6net("2001:db8::f000", 117), subnet[18]);
        assert_eq!(parse_ipv6net("2001:db8::f800", 118), subnet[19]);
        assert_eq!(parse_ipv6net("2001:db8::fc00", 119), subnet[20]);
        assert_eq!(parse_ipv6net("2001:db8::fe00", 120), subnet[21]);
        assert_eq!(parse_ipv6net("2001:db8::ff00", 121), subnet[22]);
        assert_eq!(parse_ipv6net("2001:db8::ff80", 122), subnet[23]);
        assert_eq!(parse_ipv6net("2001:db8::ffc0", 123), subnet[24]);
        assert_eq!(parse_ipv6net("2001:db8::ffe0", 124), subnet[25]);
        assert_eq!(parse_ipv6net("2001:db8::fff0", 125), subnet[26]);
        assert_eq!(parse_ipv6net("2001:db8::fff8", 126), subnet[27]);
        assert_eq!(parse_ipv6net("2001:db8::fffc", 127), subnet[28]);
        assert_eq!(parse_ipv6net("2001:db8::fffe", 128), subnet[29]);
    }
}
