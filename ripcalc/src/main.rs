use std::io::Write;

use libripcalc::{output::{Color, Output, StderrOutput, StdoutOutput}, cmds::CommandResult};


fn color_test<O: Output>(stdout: &mut O) {
    fn write_in_color<O: Output>(text: &str, color: Color, align: usize, stdout: &mut O) {
        let mut color_output = stdout.in_color(color);
        write!(color_output, "{0:1$}", text, align).unwrap();
    }

    write_in_color("Black", Color::Black, 20, stdout);
    write_in_color("DarkBlue", Color::DarkBlue, 20, stdout);
    write_in_color("DarkGreen", Color::DarkGreen, 20, stdout);
    write_in_color("DarkCyan", Color::DarkCyan, 20, stdout);
    write_in_color("DarkRed", Color::DarkRed, 20, stdout);
    write_in_color("DarkMagenta", Color::DarkMagenta, 20, stdout);
    write_in_color("DarkYellow", Color::DarkYellow, 20, stdout);
    write_in_color("Gray", Color::Gray, 20, stdout);
    write_in_color("DarkGray", Color::DarkGray, 20, stdout);
    write_in_color("Blue", Color::Blue, 20, stdout);
    write_in_color("Green", Color::Green, 20, stdout);
    write_in_color("Cyan", Color::Cyan, 20, stdout);
    write_in_color("Red", Color::Red, 20, stdout);
    write_in_color("Magenta", Color::Magenta, 20, stdout);
    write_in_color("Yellow", Color::Yellow, 20, stdout);
    write_in_color("White", Color::White, 20, stdout);
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

    let mut stdout = StdoutOutput;
    let mut stderr = StderrOutput;

    let command_result = if args[1] == "-m" || args[1] == "--minimize" {
        libripcalc::cmds::minimize::minimize(&args, &mut stdout, &mut stderr)
    } else if args[1] == "-d" || args[1] == "--derange" {
        libripcalc::cmds::derange::derange(&args, &mut stdout, &mut stderr)
    } else if cfg!(feature = "num-bigint") && (args[1] == "-s" || args[1] == "--split") {
        libripcalc::cmds::split::split(&args, &mut stdout, &mut stderr)
    } else if args[1] == "-r" || args[1] == "--resize" {
        libripcalc::cmds::resize::resize(&args, &mut stdout, &mut stderr)
    } else if args[1] == "-e" || args[1] == "--enumerate" {
        libripcalc::cmds::enumerate::enumerate(&args, &mut stdout, &mut stderr)
    } else if args[1] == "--color-test" {
        color_test(&mut stdout);
        CommandResult::Ok
    } else if args[1] == "--help" {
        usage();
        CommandResult::Ok
    } else {
        libripcalc::cmds::show_net::show_net(&args, &mut stdout, &mut stderr)
    };

    match command_result {
        CommandResult::Ok => 0,
        CommandResult::WrongUsage => {
            usage();
            return 1;
        },
        CommandResult::Error(ec) => ec,
    }
}

fn main() {
    std::process::exit(do_main());
}
