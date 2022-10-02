use gur::cur::{Cur, CurBuilder};
use rand::prelude::*;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Clone)]
struct Array(Vec<u32>);
impl Array {
    fn new(size: usize) -> Self {
        Array((0..size as u32).collect())
    }

    fn get(&self) -> &[u32] {
        &self.0
    }

    fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        self.0.shuffle(rng);
    }
    fn sort(&mut self) {
        self.0.sort();
    }
}

struct App<'a> {
    data: Cur<'a, Array>,
    rng: ThreadRng,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            data: CurBuilder::new().build(Array::new(10)),
            rng: rand::thread_rng(),
        }
    }

    fn reset(&mut self, size: usize) {
        self.data = CurBuilder::new().build(Array::new(size))
    }

    fn shuffle(&mut self) {
        // NOTE
        // This method uses try_edit() to store the old array as a snapshot
        // because of non-reproducibility of the rng.
        self.data
            .try_edit(|mut data| {
                data.shuffle(&mut self.rng);
                Ok(data)
            })
            .unwrap();
    }

    fn sort(&mut self) {
        self.data.edit(|mut data| {
            data.sort();
            data
        });
    }

    fn undo(&mut self, count: usize) {
        self.data.undo_multi(count);
    }
    fn redo(&mut self, count: usize) {
        self.data.redo_multi(count);
    }

    fn get(&self) -> &[u32] {
        self.data.get()
    }
}

const COMMAND_HELP: &str = "COMMANDS
 :h          | Print command help.
 :i SIZE     | Initialize (Reset) the array by SIZE.
 :t          | Shuffle the array.
 :s          | Sort the array.
 :u [COUNT]  | Undo COUNT changes.
 :r [COUNT]  | Redo COUNT changes.
 :q          | Quit the program.";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "[Undo demonstration] Array shuffling and sorting.\n\n{}",
        COMMAND_HELP
    );

    let mut app = App::new();

    let mut stdin = BufReader::new(io::stdin().lock());

    let print = |app: &mut App| -> io::Result<()> {
        let mut stdout = BufWriter::new(io::stdout().lock());
        writeln!(stdout, "Data: {:?}", app.get())?;
        stdout.flush()?;
        Ok(())
    };

    print(&mut app)?;
    let mut command_str = String::new();
    loop {
        command_str.clear();

        write!(&mut io::stdout(), ":")?;
        io::stdout().flush()?;

        if 0 == stdin.read_line(&mut command_str)? {
            return Ok(());
        }

        match Command::parse(&command_str)? {
            Command::Help => writeln!(&mut io::stdout(), "{}", COMMAND_HELP)?,
            Command::Quit => return Ok(()),
            Command::Reset(size) => {
                app.reset(size);
                print(&mut app)?;
            }
            Command::Shuffle => {
                app.shuffle();
                print(&mut app)?;
            }
            Command::Sort => {
                app.sort();
                print(&mut app)?;
            }
            Command::Undo(count) => {
                app.undo(count);
                print(&mut app)?;
            }
            Command::Redo(count) => {
                app.redo(count);
                print(&mut app)?;
            }
        }
    }
}

enum Command {
    Help,
    Reset(usize),
    Shuffle,
    Sort,
    Undo(usize),
    Redo(usize),
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
            "q" => Ok(Command::Quit),
            "i" => Self::parse_reset(param),
            "t" => Ok(Command::Shuffle),
            "s" => Ok(Command::Sort),
            "u" => Self::parse_undo(param),
            "r" => Self::parse_redo(param),
            _ => Ok(Command::Help),
        }
    }

    fn parse_undo(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let count = Self::parse_usize(param)?;
        Ok(Command::Undo(count.unwrap_or(1)))
    }
    fn parse_redo(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let count = Self::parse_usize(param)?;
        Ok(Command::Redo(count.unwrap_or(1)))
    }
    fn parse_reset(param: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let count = Self::parse_usize(param)?;
        Ok(Command::Reset(count.unwrap_or(1)))
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
