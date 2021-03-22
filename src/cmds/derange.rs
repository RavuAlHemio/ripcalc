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
