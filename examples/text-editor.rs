use gur::ur::{Ur, UrBuilder};
use std::cmp;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Clone)]
struct TextBuffer(Vec<char>);
impl TextBuffer {
    fn new<'a>(buffer: Vec<char>) -> Ur<'a, Self> {
        UrBuilder::new().build(TextBuffer(buffer))
    }

    fn get(&self) -> &[char] {
        &self.0
    }

    fn insert(&mut self, pos: usize, text: &str) {
        let pos = cmp::max(0, cmp::min(pos, self.0.len()));
        self.0.splice(pos..pos, text.chars());
    }
    fn delete(&mut self, pos: usize, count: usize) {
        let begin = cmp::max(0, cmp::min(pos, self.0.len()));
        let end = cmp::max(0, cmp::min(pos + count, self.0.len()));
        self.0.drain(begin..end);
    }
}

struct TextEditor<'a> {
    buffer: Ur<'a, TextBuffer>,
    cursor: usize,
    file: Option<PathBuf>,
}
impl<'a> TextEditor<'a> {
    fn new() -> Self {
        Self {
            buffer: TextBuffer::new(Vec::new()),
            cursor: 0,
            file: None,
        }
    }

    fn open(&mut self, path: Option<PathBuf>) -> io::Result<()> {
        if let Some(path) = path {
            let path = PathBuf::from(path);
            let mut s = String::new();
            if let Ok(f) = File::open(&path) {
                io::BufReader::new(f).read_to_string(&mut s)?;
            }

            self.buffer = TextBuffer::new(s.chars().collect());
            self.cursor = 0;
            self.file = Some(path);
        } else {
            *self = Self::new();
        }

        self.set_cursor_pos(self.cursor);
        Ok(())
    }

    fn save(&self, path: Option<PathBuf>) -> io::Result<()> {
        let path = if let Some(_) = path {
            &path
        } else {
            &self.file
        };

        if path.is_none() {
            return Ok(());
        }

        let mut file = io::BufWriter::new(File::create(path.as_ref().unwrap())?);
        self.write(&mut file)?;

        Ok(())
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let s: String = (*self.buffer).get().iter().collect();
        writer.write_all(s.as_bytes())?;
        Ok(())
    }

    fn print<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // write text befor the cursor
        let s: String = (*self.buffer).get()[0..self.cursor].iter().collect();
        writer.write_all(s.as_bytes())?;

        // write the cursor
        write!(writer, "|")?;

        // write text after the cursor
        let s: String = (*self.buffer).get()[self.cursor..].iter().collect();
        writer.write_all(s.as_bytes())?;

        Ok(())
    }

    fn seek(&mut self, count: isize) {
        let pos = if 0 < count {
            self.cursor.saturating_add(count as usize)
        } else {
            self.cursor.saturating_sub(-count as usize)
        };

        self.set_cursor_pos(pos);
    }

    fn insert(&mut self, text: String) {
        let pos = self.cursor;
        self.buffer.edit(move |mut buf| {
            buf.insert(pos, &text);
            buf
        });

        self.set_cursor_pos(self.cursor);
    }

    fn delete(&mut self, count: usize) {
        let pos = self.cursor;
        self.buffer.edit(move |mut buf| {
            buf.delete(pos, count);
            buf
        });

        self.set_cursor_pos(self.cursor);
    }

    fn undo(&mut self, count: usize) {
        for _ in 0..count {
            self.buffer.undo();
        }
        self.set_cursor_pos(self.cursor);
    }

    fn redo(&mut self, count: usize) {
        for _ in 0..count {
            self.buffer.redo();
        }
        self.set_cursor_pos(self.cursor);
    }

    fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor = cmp::min(pos, (*self.buffer).get().len());
    }
}

const COMMAND_HELP: &str = "COMMANDS
 :h          | Print command help.
 :p          | Print the current text buffer and cursor position.
 :s [COUNT]  | Seek the cursor +COUNT characters.
 :i TEXT     | Insert TEXT at the cursor position.
 :d [COUNT]  | Delete COUNT bytes from the cursor.
 :u [COUNT]  | Undo COUNT changes.
 :r [COUNT]  | Redo COUNT changes.
 :o [FILE]   | Open FILE.
 :w [FILE]   | Save the current text buffer as FILE.
 :q          | Quit the program.";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple text editor.\n\n{}", COMMAND_HELP);

    let mut app = TextEditor::new();

    let mut stdout = BufWriter::new(io::stdout().lock());
    let mut stdin = BufReader::new(io::stdin().lock());

    let mut command_str = String::new();
    loop {
        command_str.clear();

        write!(&mut stdout, ":")?;
        stdout.flush()?;

        if 0 == stdin.read_line(&mut command_str)? {
            return Ok(());
        }

        match Command::parse(&command_str)? {
            Command::Help => writeln!(&mut stdout, "{}", COMMAND_HELP)?,
            Command::Print => {
                app.print(&mut stdout)?;
                write!(&mut stdout, "\n")?;
                stdout.flush()?;
            }
            Command::Open(srcpath) => {
                app.open(srcpath)?;
            }
            Command::Write(dstpath) => {
                app.save(dstpath)?;
            }
            Command::Quit => return Ok(()),
            Command::Seek(count) => app.seek(count),
            Command::Insert(text) => app.insert(text),
            Command::Delete(count) => app.delete(count),
            Command::Undo(count) => app.undo(count),
            Command::Redo(count) => app.redo(count),
        }
    }
}

enum Command {
    Help,
    Print,
    Seek(isize),
    Insert(String),
    Delete(usize),
    Undo(usize),
    Redo(usize),
    Write(Option<PathBuf>),
    Open(Option<PathBuf>),
    Quit,
}
impl Command {
    fn parse(command_str: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let command_str = command_str
            .trim_start_matches(Self::is_whitespace)
            .trim_end_matches(&['\r', '\n']);
        let (name, param) = if let Some((name, param)) = command_str.split_once(Self::is_whitespace)
        {
            (name, param)
        } else {
            (command_str, "")
        };

        match name {
            "h" => Ok(Command::Help),
            "p" => Ok(Command::Print),
            "q" => Ok(Command::Quit),
            "s" => Self::parse_seek(param),
            "i" => Self::parse_insert(param),
            "d" => Self::parse_delete(param),
            "u" => Self::parse_undo(param),
            "r" => Self::parse_redo(param),
            "o" => Self::parse_open(param),
            "w" => Self::parse_write(param),
            _ => Ok(Command::Help),
        }
    }

    fn parse_seek(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let count = Self::parse_isize(param)?;
        Ok(Command::Seek(count.unwrap_or(0)))
    }
    fn parse_insert(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        Ok(Command::Insert(param.to_string()))
    }
    fn parse_delete(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let count = Self::parse_usize(param)?;
        Ok(Command::Delete(count.unwrap_or(1)))
    }
    fn parse_undo(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let count = Self::parse_usize(param)?;
        Ok(Command::Undo(count.unwrap_or(1)))
    }
    fn parse_redo(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let count = Self::parse_usize(param)?;
        Ok(Command::Redo(count.unwrap_or(1)))
    }
    fn parse_open(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        Ok(Command::Open(Self::parse_path(param)?))
    }
    fn parse_write(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        Ok(Command::Write(Self::parse_path(param)?))
    }

    fn parse_path(param: &str) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        let path = if let Some(_) = param.find(|c: char| !Self::is_whitespace(c)) {
            Some(PathBuf::from(param))
        } else {
            None
        };
        Ok(path)
    }
    fn parse_isize(param: &str) -> Result<Option<isize>, Box<dyn std::error::Error>> {
        if let None = param.find(|c: char| !Self::is_whitespace(c)) {
            return Ok(None);
        }
        let num = param.parse::<isize>()?;
        Ok(Some(num))
    }
    fn parse_usize(param: &str) -> Result<Option<usize>, Box<dyn std::error::Error>> {
        if let None = param.find(|c: char| !Self::is_whitespace(c)) {
            return Ok(None);
        }
        let num = param.parse::<usize>()?;
        Ok(Some(num))
    }

    fn is_whitespace(c: char) -> bool {
        c.is_ascii_whitespace()
    }
}
