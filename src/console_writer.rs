use std::io::{self, BufWriter, Stdout, Write};

pub struct Writer {
    writer: BufWriter<Stdout>,
}

impl Writer {
    pub fn new() -> Writer {
        let stdout = io::stdout();
        Writer {
            writer: BufWriter::new(stdout),
        }
    }

    pub fn add_line(&mut self, line: String) {
        let ln = line + "\n";
        self.writer.write(ln.as_bytes()).unwrap();
    }

    pub fn print(&mut self) {
        self.writer.flush().unwrap();
    }
}
