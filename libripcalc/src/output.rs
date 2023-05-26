use std::io;


/// A color for text output.
///
/// Matches the ANSI color codes for color terminals.
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


/// A sink that can receive textual data.
pub trait Output : io::Write {
    fn in_color(&mut self, color: Color) -> Box<dyn io::Write>;
}


/// Outputs text to standard output.
pub struct StdoutOutput;
impl io::Write for StdoutOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.flush()
    }
}
impl Output for StdoutOutput {
    #[cfg(not(target_os = "windows"))]
    fn in_color(&mut self, color: Color) -> Box<dyn io::Write> {
        if std::env::var_os("NO_COLOR").map(|c| c.len() > 0).unwrap_or(false) {
            // no color; just return ourselves
            Box::new(StdoutOutput)
        } else {
            Box::new(StdoutAnsiColorOutput::new(color))
        }
    }

    #[cfg(target_os = "windows")]
    fn in_color(&mut self, color: Color) -> Box<dyn io::Write> {
        if std::env::var_os("NO_COLOR").map(|c| c.len() > 0).unwrap_or(false) {
            // no color; just return ourselves
            Box::new(StdoutOutput)
        } else {
            Box::new(StdoutWindowsColorOutput::new(color))
        }
    }
}

/// Outputs text to standard error.
pub struct StderrOutput;
impl io::Write for StderrOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let stderr = std::io::stderr();
        let mut stderr_lock = stderr.lock();
        stderr_lock.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let stderr = std::io::stderr();
        let mut stderr_lock = stderr.lock();
        stderr_lock.flush()
    }
}
impl Output for StderrOutput {
    fn in_color(&mut self, _color: Color) -> Box<dyn io::Write> {
        // no color on stderr
        Box::new(StderrOutput)
    }
}

/// Outputs text to standard output in a color using ANSI escape codes.
pub struct StdoutAnsiColorOutput {
    color: Color,
}
impl StdoutAnsiColorOutput {
    pub fn new(color: Color) -> Self {
        Self {
            color,
        }
    }
}
impl io::Write for StdoutAnsiColorOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let color: [u8; 2] = match self.color {
            Color::Black => *b"30",
            Color::DarkRed => *b"31",
            Color::DarkGreen => *b"32",
            Color::DarkYellow => *b"33",
            Color::DarkBlue => *b"34",
            Color::DarkMagenta => *b"35",
            Color::DarkCyan => *b"36",
            Color::Gray => *b"37",
            Color::DarkGray => *b"90",
            Color::Red => *b"91",
            Color::Green => *b"92",
            Color::Yellow => *b"93",
            Color::Blue => *b"94",
            Color::Magenta => *b"95",
            Color::Cyan => *b"96",
            Color::White => *b"97",
        };
        let mut color_escape = *b"\x1B[00m";
        color_escape[2] = color[0];
        color_escape[3] = color[1];
        const RESET_ESCAPE: &[u8] = b"\x1B[0m";

        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.write_all(&color_escape)?;
        stdout_lock.write_all(buf)?;
        stdout_lock.write_all(RESET_ESCAPE)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.flush()
    }
}


/// Outputs text to standard output in a color using ANSI escape codes.
#[cfg(target_os = "windows")]
pub struct StdoutWindowsColorOutput {
    color: Color,
}
#[cfg(target_os = "windows")]
impl StdoutWindowsColorOutput {
    pub fn new(color: Color) -> Self {
        Self {
            color,
        }
    }
}
#[cfg(target_os = "windows")]
impl io::Write for StdoutWindowsColorOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use windows::Win32::System::Console::{
            CONSOLE_CHARACTER_ATTRIBUTES, CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO,
            FOREGROUND_BLUE as BLU, FOREGROUND_GREEN as GRN, FOREGROUND_INTENSITY as INT,
            FOREGROUND_RED as RED, GetConsoleMode, GetConsoleScreenBufferInfo, GetStdHandle,
            SetConsoleTextAttribute, STD_OUTPUT_HANDLE,
        };

        // get a handle on stdout
        let mut stdout_console = None;
        let stdout_console_res = unsafe {
            GetStdHandle(STD_OUTPUT_HANDLE)
        };
        if let Ok(o) = stdout_console_res {
            // is this a console?
            let mut mode = CONSOLE_MODE::default();
            let result = unsafe {
                GetConsoleMode(o, &mut mode)
            };
            if result.as_bool() {
                // yes, it is a console
                stdout_console = Some(o);
            }
        }

        let mut console_screen_buffer_info = CONSOLE_SCREEN_BUFFER_INFO::default();
        if let Some(console) = stdout_console {
            // get current attributes
            unsafe {
                GetConsoleScreenBufferInfo(
                    console,
                    &mut console_screen_buffer_info,
                )
            };

            // set new attributes
            const NAH: CONSOLE_CHARACTER_ATTRIBUTES = CONSOLE_CHARACTER_ATTRIBUTES(0);
            const COLOR_MASK: CONSOLE_CHARACTER_ATTRIBUTES = CONSOLE_CHARACTER_ATTRIBUTES(INT.0 | BLU.0 | GRN.0 | RED.0);
            let new_color = match self.color {
                Color::Black => NAH | NAH | NAH,
                Color::DarkRed => NAH | NAH | RED,
                Color::DarkGreen => NAH | GRN | NAH,
                Color::DarkYellow => NAH | GRN | RED,
                Color::DarkBlue => BLU | NAH | NAH,
                Color::DarkMagenta => BLU | NAH | RED,
                Color::DarkCyan => BLU | GRN | NAH,
                Color::Gray => BLU | GRN | RED,
                Color::DarkGray => INT | NAH | NAH | NAH,
                Color::Red => INT | NAH | NAH | RED,
                Color::Green => INT | NAH | GRN | NAH,
                Color::Yellow => INT | NAH | GRN | RED,
                Color::Blue => INT | BLU | NAH | NAH,
                Color::Magenta => INT | BLU | NAH | RED,
                Color::Cyan => INT | BLU | GRN | NAH,
                Color::White => INT | BLU | GRN | RED,
            };
            let new_attributes = new_color | (console_screen_buffer_info.wAttributes & (!COLOR_MASK));
            unsafe {
                SetConsoleTextAttribute(console, new_attributes)
            };
        }

        // perform regular write to stdout
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        let bytes_written = stdout_lock.write(buf)?;

        // flush before we switch back
        stdout_lock.flush()?;

        if let Some(console) = stdout_console {
            // reset state
            unsafe {
                SetConsoleTextAttribute(console, console_screen_buffer_info.wAttributes)
            };
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.flush()
    }
}


/// Outputs text to standard output as HTML.
pub struct HtmlStdoutOutput;
impl io::Write for HtmlStdoutOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();

        let mut remaining_buf = buf;
        while remaining_buf.len() > 0 {
            // find next character in need of escaping
            let next_escape_pos_opt = remaining_buf.iter().position(|b| *b == b'<' || *b == b'>' || *b == b'&');
            match next_escape_pos_opt {
                Some(pos) => {
                    if pos > 0 {
                        // write out the chunk that needs no escaping
                        stdout_lock.write_all(&remaining_buf[..pos])?;
                    }

                    // write out the byte in its escaped form
                    let escaped_bytes: &[u8] = match remaining_buf[pos] {
                        b'<' => b"&lt;",
                        b'>' => b"&gt;",
                        b'&' => b"&amp;",
                        _ => unreachable!(),
                    };
                    stdout_lock.write_all(escaped_bytes)?;

                    // next time around, consider the rest
                    remaining_buf = &remaining_buf[pos+1..];
                },
                None => {
                    // the rest of the string needs no escaping
                    stdout_lock.write_all(remaining_buf)?;
                    remaining_buf = &[];
                },
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.flush()
    }
}
impl Output for HtmlStdoutOutput {
    fn in_color(&mut self, color: Color) -> Box<dyn io::Write> {
        Box::new(ColorHtmlStdoutOutput::new(color))
    }
}

/// Outputs text as HTML to standard output.
pub struct ColorHtmlStdoutOutput {
    color: Color,
}
impl ColorHtmlStdoutOutput {
    pub fn new(color: Color) -> Self {
        Self {
            color,
        }
    }
}
impl io::Write for ColorHtmlStdoutOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let color_class = match self.color {
            Color::Black => "black",
            Color::DarkRed => "dark-red",
            Color::DarkGreen => "dark-green",
            Color::DarkYellow => "dark-yellow",
            Color::DarkBlue => "dark-blue",
            Color::DarkMagenta => "dark-magenta",
            Color::DarkCyan => "dark-cyan",
            Color::Gray => "gray",
            Color::DarkGray => "dark-gray",
            Color::Red => "red",
            Color::Green => "green",
            Color::Yellow => "yellow",
            Color::Blue => "blue",
            Color::Magenta => "magenta",
            Color::Cyan => "cyan",
            Color::White => "white",
        };
        let start_string = format!("<span class=\"color color-{}\">", color_class);
        const END_STRING: &str = "</span>";

        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.write_all(start_string.as_bytes())?;
        stdout_lock.write_all(buf)?;
        stdout_lock.write_all(END_STRING.as_bytes())?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        stdout_lock.flush()
    }
}
