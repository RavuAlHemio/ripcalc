use std::convert::TryInto;

#[cfg(feature = "console")]
use console;

/// An ANSI color code for color terminals.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    DarkMagenta,
    DarkYellow,
    Gray,
    DarkGray,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Yellow,
    White,
}

/// Outputs text, optionally in a given color, padded to a specific length. Positive padding values
/// pad at the end, negative at the beginning.
pub fn write_in_color<S: AsRef<str>>(text: S, color: Option<Color>, pad_to: isize) {
    // pad the string
    let mut padded = String::from(text.as_ref());
    let padded_len_isize: isize = padded.len().try_into().unwrap();
    if pad_to > 0 {
        if pad_to > padded_len_isize {
            // pad at end
            let delta = pad_to - padded_len_isize;
            for _ in 0..delta {
                padded.push(' ');
            }
        }
    } else if pad_to < 0 {
        if pad_to < -padded_len_isize {
            // pad at beginning
            let delta = (-pad_to) - padded_len_isize;
            let mut padding = String::with_capacity(delta.try_into().unwrap());
            for _ in 0..delta {
                padding.push(' ');
            }
            padded.insert_str(0, &padding);
        }
    }

    if cfg!(feature = "console") {
        if console::colors_enabled() {
            if let Some(clr) = color {
                let styled = console::style(padded);
                let colored = match clr {
                    Color::Black => styled.black(),
                    Color::DarkBlue => styled.blue(),
                    Color::DarkGreen => styled.green(),
                    Color::DarkCyan => styled.cyan(),
                    Color::DarkRed => styled.red(),
                    Color::DarkMagenta => styled.magenta(),
                    Color::DarkYellow => styled.yellow(),
                    Color::Gray => styled.white(),
                    Color::DarkGray => styled.bright().black(),
                    Color::Blue => styled.bright().blue(),
                    Color::Green => styled.bright().green(),
                    Color::Cyan => styled.bright().cyan(),
                    Color::Red => styled.bright().red(),
                    Color::Magenta => styled.bright().magenta(),
                    Color::Yellow => styled.bright().yellow(),
                    Color::White => styled.bright().white(),
                };
                print!("{}", colored);
                return;
            }
        }
    }

    println!("{}", padded);
}
