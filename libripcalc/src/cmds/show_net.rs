use std::fmt::Debug;

#[cfg(feature = "num-bigint")]
use num_bigint::BigInt;

use crate::bit_manip::bytes_to_binary;
use crate::cmds::{CommandResult, NetworkSpec, parse_netspec};
use crate::output::{Color, Output};
use crate::addr::{IpAddress, Ipv4Address, Ipv6Address};
use crate::net::IpNetwork;


const LABEL_COLOR: Color = Color::White;
const IP_ADDRESS_COLOR: Color = Color::Blue;
const HOST_BITS_COLOR: Color = Color::Yellow;
const NET_BITS_COLOR: Color = Color::Green;
const MASK_BITS_COLOR: Color = Color::Red;
const CLASS_BITS_COLOR: Color = Color::Magenta;
const ADDR_SEP_COLOR: Color = Color::White;


pub fn show_net<S: AsRef<str> + Debug, O: Output, E: Output>(args: &Vec<S>, stdout: &mut O, stderr: &mut E) -> CommandResult {
    let mut specs = Vec::new();
    for arg in &args[1..] {
        match parse_netspec(arg.as_ref()) {
            Ok(spec) => specs.push(spec),
            Err(e) => {
                writeln!(stderr, "{}", e).unwrap();
                return CommandResult::Error(1);
            },
        };
    }

    let mut is_first = true;
    for spec in &specs {
        if !is_first {
            writeln!(stdout).unwrap();
        }
        is_first = false;

        match spec {
            NetworkSpec::Ipv4(a, n) => output_ipv4_network(*n, Some(*a), stdout),
            NetworkSpec::Ipv6(a, n) => output_ipv6_network(*n, Some(*a), stdout),
        };
    }

    CommandResult::Ok
}

fn output_network<
    A: IpAddress,
    OBA: Fn(A, Option<A>, bool, Option<Color>, &mut O),
    OC: Fn(&str, &str, &mut O),
    O: Output,
>(
    label_width: usize,
    address_width: usize,
    output_binary_address: OBA,
    output_class: OC,
    net: IpNetwork<A>,
    addr: Option<A>,
    stdout: &mut O,
) {
    let output_initial_columns = |label: &str, address: &str, stdout: &mut O| {
        {
            let mut label_color_writer = stdout.in_color(LABEL_COLOR);
            write!(label_color_writer, "{0:1$}", label, label_width).unwrap();
        }
        {
            let mut ip_color_writer = stdout.in_color(IP_ADDRESS_COLOR);
            write!(ip_color_writer, "{0:1$}", address, address_width).unwrap();
        }
    };

    if let Some(a) = addr {
        output_initial_columns("Address:", &a.to_string(), stdout);
        output_binary_address(a, Some(net.subnet_mask()), false, None, stdout);
        writeln!(stdout).unwrap();

        let netmask_addr_str = if let Some(pfx) = net.cidr_prefix() {
            format!("{} = {}", net.subnet_mask(), pfx)
        } else {
            net.subnet_mask().to_string()
        };
        output_initial_columns("Netmask:", &netmask_addr_str, stdout);
        output_binary_address(net.subnet_mask(), None, false, Some(MASK_BITS_COLOR), stdout);
        writeln!(stdout).unwrap();

        output_initial_columns("Wildcard:", &net.cisco_wildcard().to_string(), stdout);
        output_binary_address(net.cisco_wildcard(), None, false, None, stdout);
        writeln!(stdout).unwrap();

        {
            let mut label_color_writer = stdout.in_color(LABEL_COLOR);
            write!(label_color_writer, "=>").unwrap();
        }
        writeln!(stdout).unwrap();
    }

    let net_str = if let Some(pfx) = net.cidr_prefix() {
        format!("{}/{}", net.base_addr(), pfx)
    } else {
        net.base_addr().to_string()
    };
    output_initial_columns("Network:", &net_str, stdout);
    output_binary_address(net.base_addr(), Some(net.subnet_mask()), true, None, stdout);
    writeln!(stdout).unwrap();

    if let Some(fha) = net.first_host_addr() {
        output_initial_columns("HostMin:", &fha.to_string(), stdout);
        output_binary_address(fha, None, false, None, stdout);
        writeln!(stdout).unwrap();
        let lha = net.last_host_addr().unwrap();
        output_initial_columns("HostMax:", &lha.to_string(), stdout);
        output_binary_address(lha, None, false, None, stdout);
    } else {
        let mut label_color_writer = stdout.in_color(LABEL_COLOR);
        write!(label_color_writer, "no hosts").unwrap();
    }
    writeln!(stdout).unwrap();

    if let Some(bc) = net.broadcast_addr() {
        output_initial_columns("Broadcast:", &bc.to_string(), stdout);
        output_binary_address(bc, None, false, None, stdout);
    } else {
        let mut label_color_writer = stdout.in_color(LABEL_COLOR);
        write!(label_color_writer, "no broadcast").unwrap();
    }
    writeln!(stdout).unwrap();

    if cfg!(feature = "num-bigint") {
        if net.host_count() > BigInt::from(0) {
            output_initial_columns("Hosts/Net:", &net.host_count().to_string(), stdout);
            let top_bits = bytes_to_binary(&net.base_addr().to_bytes()[0..1]);
            let top_mask_bits = bytes_to_binary(&net.subnet_mask().to_bytes()[0..1]);
            output_class(&top_bits, &top_mask_bits, stdout);
            writeln!(stdout).unwrap();
        } else {
            let mut label_color_writer = stdout.in_color(LABEL_COLOR);
            write!(label_color_writer, "no hosts/net").unwrap();
        }
    }
}

fn output_ipv4_class<O: Output>(top_bits: &str, top_mask_bits: &str, stdout: &mut O) {
    let mut class_bits_color_writer = stdout.in_color(CLASS_BITS_COLOR);
    if top_bits.starts_with("0") && top_mask_bits.starts_with("1") {
        write!(class_bits_color_writer, "Class A").unwrap();
    } else if top_bits.starts_with("10") && top_mask_bits.starts_with("11") {
        write!(class_bits_color_writer, "Class B").unwrap();
    } else if top_bits.starts_with("110") && top_mask_bits.starts_with("111") {
        write!(class_bits_color_writer, "Class C").unwrap();
    } else if top_mask_bits.starts_with("1111") {
        if top_bits.starts_with("1110") {
            write!(class_bits_color_writer, "Class D (multicast)").unwrap();
        } else if top_bits.starts_with("1111") {
            write!(class_bits_color_writer, "Class E (reserved)").unwrap();
        }
    }
}

fn output_binary_ipv4_address<O: Output>(
    addr: Ipv4Address,
    subnet_mask: Option<Ipv4Address>,
    mut color_class: bool,
    override_color: Option<Color>,
    stdout: &mut O,
) {
    let addr_bytes = addr.to_bytes();
    let mask_bytes = subnet_mask.as_ref().map(|m| m.to_bytes());

    for i in 0..addr_bytes.len() {
        let b = addr_bytes[i];
        let m = mask_bytes.as_ref().map(|m| m[i]);

        let bits = bytes_to_binary(&[b]);
        let mask_bits = m.map(|mb| bytes_to_binary(&[mb]));

        if let Some(oc) = override_color {
            // simply output the address
            let mut color_writer = stdout.in_color(oc);
            write!(color_writer, "{}", bits).unwrap();
        } else if mask_bits.is_none() {
            // simple output here too
            let mut color_writer = stdout.in_color(HOST_BITS_COLOR);
            write!(color_writer, "{}", bits).unwrap();
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

                let mut color_writer = stdout.in_color(class_color.unwrap_or(color));
                write!(color_writer, "{}", &String::from(bitvec[bit])).unwrap();
            }
        }

        if i < addr_bytes.len() - 1 {
            // add separator (dot)
            let mut color_writer = stdout.in_color(ADDR_SEP_COLOR);
            write!(color_writer, ".").unwrap();
        }
    }
}

fn output_binary_ipv6_address<O: Output>(
    addr: Ipv6Address,
    subnet_mask: Option<Ipv6Address>,
    _color_class: bool,
    override_color: Option<Color>,
    stdout: &mut O,
) {
    let addr_bytes = addr.to_bytes();
    let mask_bytes = subnet_mask.as_ref().map(|m| m.to_bytes());

    for i in 0..addr_bytes.len() {
        let b = addr_bytes[i];
        let m = mask_bytes.as_ref().map(|m| m[i]);

        let bits = bytes_to_binary(&[b]);
        let mask_bits = m.map(|mb| bytes_to_binary(&[mb]));

        if let Some(oc) = override_color {
            // simply output the address
            let mut color_writer = stdout.in_color(oc);
            write!(color_writer, "{}", bits).unwrap();
        } else if mask_bits.is_none() {
            // simple output here too
            let mut color_writer = stdout.in_color(HOST_BITS_COLOR);
            write!(color_writer, "{}", bits).unwrap();
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

                let mut color_writer = stdout.in_color(color);
                write!(color_writer, "{}", &String::from(bitvec[bit])).unwrap();
            }
        }

        if i < addr_bytes.len() - 1 && i % 2 == 1 {
            // add separator (colon)
            let mut color_writer = stdout.in_color(ADDR_SEP_COLOR);
            write!(color_writer, ":").unwrap();
        }
    }
}

/// Outputs and dissects information about an IPv4 network.
pub fn output_ipv4_network<O: Output>(net: IpNetwork<Ipv4Address>, addr: Option<Ipv4Address>, stdout: &mut O) {
    output_network(
        11,
        21,
        output_binary_ipv4_address,
        output_ipv4_class,
        net,
        addr,
        stdout,
    )
}

/// Outputs and dissects information about an IPv6 network.
pub fn output_ipv6_network<O: Output>(net: IpNetwork<Ipv6Address>, addr: Option<Ipv6Address>, stdout: &mut O) {
    output_network(
        11,
        46,
        output_binary_ipv6_address,
        |_top_bits, _top_mask_bits, _stdout| {},
        net,
        addr,
        stdout,
    )
}
