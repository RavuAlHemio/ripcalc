use std::io;

use libripcalc::output::{Color, Output};

use crate::{BUFFER_SIZE, U16_BUFFER};


#[link(wasm_import_module = "wasm_interop")]
extern {
    fn append_output();
    fn append_error();
}

pub(crate) fn write_to<F: FnMut()>(buf: &str, mut append_func: F) {
    let mut offset = 0;
    for word in buf.encode_utf16() {
        unsafe { U16_BUFFER[offset] = word };
        offset += 1;
        if offset == unsafe { U16_BUFFER.len() } {
            // we've hit the limit; push out what we have
            unsafe { BUFFER_SIZE = offset };
            append_func();
            offset = 0;
        }
    }
    if offset > 0 {
        // push out last batch
        unsafe { BUFFER_SIZE = offset };
        append_func();
    }
}

pub(crate) fn write_to_output(buf: &str) {
    write_to(buf, || unsafe { append_output() });
}
pub(crate) fn write_to_error(buf: &str) {
    write_to(buf, || unsafe { append_error() });
}

pub(crate) struct HtmlWasmStdout;
impl io::Write for HtmlWasmStdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf_str = std::str::from_utf8(buf)
            .expect("failed to decode UTF-8");
        let mut remaining_buf = buf_str;
        while remaining_buf.len() > 0 {
            // find next character in need of escaping
            let next_escape_pos_opt = remaining_buf.bytes().position(|b| b == b'<' || b == b'>' || b == b'&');
            match next_escape_pos_opt {
                Some(pos) => {
                    if pos > 0 {
                        // write out the chunk that needs no escaping
                        write_to_output(&remaining_buf[..pos]);
                    }

                    // write out the byte in its escaped form
                    let escaped_str = match remaining_buf.bytes().nth(pos).unwrap() {
                        b'<' => "&lt;",
                        b'>' => "&gt;",
                        b'&' => "&amp;",
                        _ => unreachable!(),
                    };
                    write_to_output(escaped_str);

                    // next time around, consider the rest
                    remaining_buf = &remaining_buf[pos+1..];
                },
                None => {
                    // the rest of the string needs no escaping
                    write_to_output(remaining_buf);
                    remaining_buf = "";
                },
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Output for HtmlWasmStdout {
    fn in_color(&mut self, color: Color) -> Box<dyn io::Write> {
        Box::new(HtmlColorWasmStdout {
            color,
        })
    }
}

pub(crate) struct HtmlColorWasmStdout {
    color: Color,
}
impl io::Write for HtmlColorWasmStdout {
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

        write_to_output(&start_string);
        {
            let mut escaping_stdout = HtmlWasmStdout;
            escaping_stdout.write(buf)?;
        }
        write_to_output(END_STRING);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) struct WasmStderr;
impl io::Write for WasmStderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // no escaping here
        let buf_str = std::str::from_utf8(buf)
            .expect("failed to decode UTF-8");
        write_to_error(buf_str);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Output for WasmStderr {
    fn in_color(&mut self, _color: Color) -> Box<dyn io::Write> {
        // no colors, just return ourself
        Box::new(WasmStderr)
    }
}
