use std::fmt::Debug;

#[cfg(feature = "num-bigint")]
use num_bigint::BigInt;

use crate::bit_manip::bytes_to_binary;
use crate::cmds::{NetworkSpec, parse_netspec};
use crate::console::{Color, write_in_color};
use crate::addr::{IpAddress, Ipv4Address, Ipv6Address};
use crate::net::IpNetwork;


const LABEL_COLOR: Color = Color::White;
const IP_ADDRESS_COLOR: Color = Color::Blue;
const HOST_BITS_COLOR: Color = Color::Yellow;
const NET_BITS_COLOR: Color = Color::Green;
const MASK_BITS_COLOR: Color = Color::Red;
const CLASS_BITS_COLOR: Color = Color::Magenta;
const ADDR_SEP_COLOR: Color = Color::White;


pub fn show_net<S: AsRef<str> + Debug>(args: &Vec<S>) -> i32 {
    let mut specs = Vec::new();
    for arg in &args[1..] {
        match parse_netspec(arg.as_ref()) {
            Ok(spec) => specs.push(spec),
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            },
        };
    }

    let mut is_first = true;
    for spec in &specs {
        if !is_first {
            println!();
        }
        is_first = false;

        match spec {
            NetworkSpec::Ipv4(a, n) => output_ipv4_network(*n, Some(*a)),
            NetworkSpec::Ipv6(a, n) => output_ipv6_network(*n, Some(*a)),
        };
    }

    0
}

fn output_network<A: IpAddress, OBA: Fn(A, Option<A>, bool, Option<Color>), OC: Fn(&str, &str)>(
    label_width: isize,
    address_width: isize,
    output_binary_address: OBA,
    output_class: OC,
    net: IpNetwork<A>,
    addr: Option<A>,
) {
    let output_initial_columns = |label: &str, address: &str| {
        write_in_color(label, Some(LABEL_COLOR), label_width);
        write_in_color(address, Some(IP_ADDRESS_COLOR), address_width);
    };

    if let Some(a) = addr {
        output_initial_columns("Address:", &a.to_string());
        output_binary_address(a, Some(net.subnet_mask()), false, None);
        println!();

        let netmask_addr_str = if let Some(pfx) = net.cidr_prefix() {
            format!("{} = {}", net.subnet_mask(), pfx)
        } else {
            net.subnet_mask().to_string()
        };
        output_initial_columns("Netmask:", &netmask_addr_str);
        output_binary_address(net.subnet_mask(), None, false, Some(MASK_BITS_COLOR));
        println!();

        output_initial_columns("Wildcard:", &net.cisco_wildcard().to_string());
        output_binary_address(net.cisco_wildcard(), None, false, None);
        println!();

        write_in_color("=>", Some(LABEL_COLOR), 0);
        println!();
    }

    let net_str = if let Some(pfx) = net.cidr_prefix() {
        format!("{}/{}", net.base_addr(), pfx)
    } else {
        net.base_addr().to_string()
    };
    output_initial_columns("Network:", &net_str);
    output_binary_address(net.base_addr(), Some(net.subnet_mask()), true, None);
    println!();

    if let Some(fha) = net.first_host_addr() {
        output_initial_columns("HostMin:", &fha.to_string());
        output_binary_address(fha, None, false, None);
        println!();
        let lha = net.last_host_addr().unwrap();
        output_initial_columns("HostMax:", &lha.to_string());
        output_binary_address(lha, None, false, None);
    } else {
        write_in_color("no hosts", Some(LABEL_COLOR), 0);
    }
    println!();

    if let Some(bc) = net.broadcast_addr() {
        output_initial_columns("Broadcast:", &bc.to_string());
        output_binary_address(bc, None, false, None);
    } else {
        write_in_color("no broadcast", Some(LABEL_COLOR), 0);
    }
    println!();

    if cfg!(feature = "num-bigint") {
        if net.host_count() > BigInt::from(0) {
            output_initial_columns("Hosts/Net:", &net.host_count().to_string());
            let top_bits = bytes_to_binary(&net.base_addr().to_bytes()[0..1]);
            let top_mask_bits = bytes_to_binary(&net.subnet_mask().to_bytes()[0..1]);
            output_class(&top_bits, &top_mask_bits);
            println!();
        } else {
            write_in_color("no hosts/net", Some(LABEL_COLOR), 0);
        }
    }
}

fn output_ipv4_class(top_bits: &str, top_mask_bits: &str) {
    if top_bits.starts_with("0") && top_mask_bits.starts_with("1") {
        write_in_color("Class A", Some(CLASS_BITS_COLOR), 0);
    } else if top_bits.starts_with("10") && top_mask_bits.starts_with("11") {
        write_in_color("Class B", Some(CLASS_BITS_COLOR), 0);
    } else if top_bits.starts_with("110") && top_mask_bits.starts_with("111") {
        write_in_color("Class C", Some(CLASS_BITS_COLOR), 0);
    } else if top_mask_bits.starts_with("1111") {
        if top_bits.starts_with("1110") {
            write_in_color("Class D (multicast)", Some(CLASS_BITS_COLOR), 0);
        } else if top_bits.starts_with("1111") {
            write_in_color("Class E (reserved)", Some(CLASS_BITS_COLOR), 0);
        }
    }
}

fn output_binary_ipv4_address(
    addr: Ipv4Address,
    subnet_mask: Option<Ipv4Address>,
    mut color_class: bool,
    override_color: Option<Color>
) {
    let addr_bytes = addr.to_bytes();
    let mask_bytes = subnet_mask.as_ref().map(|m| m.to_bytes());

    for i in 0..addr_bytes.len() {
        let b = addr_bytes[i];
        let m = mask_bytes.as_ref().map(|m| m[i]);

        let bits = bytes_to_binary(&[b]);
        let mask_bits = m.map(|mb| bytes_to_binary(&[mb]));

        if override_color.is_some() {
            // simply output the address
            write_in_color(bits, override_color, 0);
        } else if mask_bits.is_none() {
            // simple output here too
            write_in_color(bits, Some(HOST_BITS_COLOR), 0);
        } else {
            // we must differentiate

            let bitvec: Vec<char> = bits.chars().collect();
            if i == 0 && color_class {
                let mask_bitvec: Vec<char> = mask_bits.as_ref().unwrap().chars().collect();

                // check if this is a classful network
                if mask_bitvec[0] == '0' {
                    // first bit isn't part of the network
                    color_class = false;
                } else if bitvec[0] == '1' && mask_bitvec[1] == '0' {
                    // first bit, 1, is part of the network, but second isn't
                    color_class = false;
                } else if bitvec[1] == '1' && mask_bitvec[2] == '0' {
                    // first two bits, both 1, are part of the network, but third isn't
                    color_class = false;
                } else if bitvec[2] == '1' && mask_bitvec[3] == '0' {
                    // first two bits, both 1, are part of the network, but third isn't
                    color_class = false;
                }
            }

            for bit in 0..8 {
                // assign color
                let color = if let Some(mb) = &mask_bits {
                    if mb.chars().nth(bit).unwrap() == '1' {
                        NET_BITS_COLOR
                    } else {
                        HOST_BITS_COLOR
                    }
                } else {
                    HOST_BITS_COLOR
                };

                let class_color = if i == 0 && color_class {
                    // the old-style class might be relevant

                    if bit == 0 {
                        Some(CLASS_BITS_COLOR)
                    } else if bit == 1 && bitvec[0] == '1' {
                        Some(CLASS_BITS_COLOR)
                    } else if bit == 2 && bits.starts_with("11") {
                        Some(CLASS_BITS_COLOR)
                    } else if bit == 3 && bits.starts_with("111") {
                        Some(CLASS_BITS_COLOR)
                    } else {
                        None
                    }
                } else {
                    None
                };

                write_in_color(&String::from(bitvec[bit]), class_color.or(Some(color)), 0);
            }
        }

        if i < addr_bytes.len() - 1 {
            // add separator (dot)
            write_in_color(".", Some(ADDR_SEP_COLOR), 0);
        }
    }
}

fn output_binary_ipv6_address(
    addr: Ipv6Address,
    subnet_mask: Option<Ipv6Address>,
    _color_class: bool,
    override_color: Option<Color>
) {
    let addr_bytes = addr.to_bytes();
    let mask_bytes = subnet_mask.as_ref().map(|m| m.to_bytes());

    for i in 0..addr_bytes.len() {
        let b = addr_bytes[i];
        let m = mask_bytes.as_ref().map(|m| m[i]);

        let bits = bytes_to_binary(&[b]);
        let mask_bits = m.map(|mb| bytes_to_binary(&[mb]));

        if override_color.is_some() {
            // simply output the address
            write_in_color(bits, override_color, 0);
        } else if mask_bits.is_none() {
            // simple output here too
            write_in_color(bits, Some(HOST_BITS_COLOR), 0);
        } else {
            // we must differentiate
            let bitvec: Vec<char> = bits.chars().collect();
            let mask_bitvec: Option<Vec<char>> = mask_bits.map(|mb| mb.chars().collect());
            for bit in 0..8 {
                // assign color
                let color = if let Some(mbv) = &mask_bitvec {
                    if mbv[bit] == '1' {
                        NET_BITS_COLOR
                    } else {
                        HOST_BITS_COLOR
                    }
                } else {
                    HOST_BITS_COLOR
                };

                write_in_color(&String::from(bitvec[bit]), Some(color), 0);
            }
        }

        if i < addr_bytes.len() - 1 && i % 2 == 1 {
            // add separator (colon)
            write_in_color(":", Some(ADDR_SEP_COLOR), 0);
        }
    }
}

/// Outputs and dissects information about an IPv4 network.
pub fn output_ipv4_network(net: IpNetwork<Ipv4Address>, addr: Option<Ipv4Address>) {
    output_network(
        11,
        21,
        output_binary_ipv4_address,
        output_ipv4_class,
        net,
        addr,
    )
}

/// Outputs and dissects information about an IPv6 network.
pub fn output_ipv6_network(net: IpNetwork<Ipv6Address>, addr: Option<Ipv6Address>) {
    output_network(
        11,
        46,
        output_binary_ipv6_address,
        |_top_bits, _top_mask_bits| {},
        net,
        addr,
    )
}
