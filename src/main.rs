mod addr;
mod bit_manip;
mod cidr;
mod cmds;
mod console;
mod net;

use crate::console::Color;


fn color_test() {
    crate::console::write_in_color("Black", Some(Color::Black), 20);
    crate::console::write_in_color("DarkBlue", Some(Color::DarkBlue), 20);
    crate::console::write_in_color("DarkGreen", Some(Color::DarkGreen), 20);
    crate::console::write_in_color("DarkCyan", Some(Color::DarkCyan), 20);
    crate::console::write_in_color("DarkRed", Some(Color::DarkRed), 20);
    crate::console::write_in_color("DarkMagenta", Some(Color::DarkMagenta), 20);
    crate::console::write_in_color("DarkYellow", Some(Color::DarkYellow), 20);
    crate::console::write_in_color("Gray", Some(Color::Gray), 20);
    crate::console::write_in_color("DarkGray", Some(Color::DarkGray), 20);
    crate::console::write_in_color("Blue", Some(Color::Blue), 20);
    crate::console::write_in_color("Green", Some(Color::Green), 20);
    crate::console::write_in_color("Cyan", Some(Color::Cyan), 20);
    crate::console::write_in_color("Red", Some(Color::Red), 20);
    crate::console::write_in_color("Magenta", Some(Color::Magenta), 20);
    crate::console::write_in_color("Yellow", Some(Color::Yellow), 20);
    crate::console::write_in_color("White", Some(Color::White), 20);
}

fn usage() {
    eprintln!("Usage: ripcalc IPADDRESS/SUBNET...");
    eprintln!("       ripcalc -m|--minimize IPADDRESS/SUBNET...");
    eprintln!("       ripcalc -d|--derange IPADDRESS IPADDRESS...");
    if cfg!(feature = "num-bigint") {
        eprintln!("       ripcalc -s|--split IPADDRESS/CIDRPREFIX HOSTCOUNT...");
    }
    eprintln!("       ripcalc -r|--resize IPADDRESS/SUBNET SUBNET");
    eprintln!("       ripcalc -e|--enumerate IPADDRESS/SUBNET");
    eprintln!();
    eprintln!("SUBNET is one of: SUBNETMASK");
    eprintln!("                  CIDRPREFIX");
    eprintln!("                  -WILDCARD");
    eprintln!();
    eprintln!("IPv4 and IPv6 are supported, but cannot be mixed within an invocation.");
}

fn do_main() -> i32 {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        usage();
        return 1;
    }

    if args[1] == "-m" || args[1] == "--minimize" {
        crate::cmds::minimize::minimize(&args)
    } else if args[1] == "-d" || args[1] == "--derange" {
        crate::cmds::derange::derange(&args)
    } else if cfg!(feature = "num-bigint") && (args[1] == "-s" || args[1] == "--split") {
        crate::cmds::split::split(&args)
    } else if args[1] == "-r" || args[1] == "--resize" {
        crate::cmds::resize::resize(&args)
    } else if args[1] == "-e" || args[1] == "--enumerate" {
        crate::cmds::enumerate::enumerate(&args)
    } else if args[1] == "--color-test" {
        color_test();
        0
    } else if args[1] == "--help" {
        usage();
        0
    } else {
        crate::cmds::show_net::show_net(&args)
    }
}

fn main() {
    std::process::exit(do_main());
}
