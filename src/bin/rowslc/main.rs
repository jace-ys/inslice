use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process;

use clap::Clap;

use slice::filter::{Filter, FilterSet};

#[derive(Clap)]
#[clap(
    name = "rowslc",
    version = "1.0.0",
    author = "Jace Tan <jaceys.tan@gmail.com>"
)]
struct Opts {
    /// Path to input file
    filepath: Option<String>,

    #[clap(short, long)]
    /// Filters to be applied
    filters: Vec<Filter>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let reader: Box<dyn BufRead> = match opts.filepath.as_deref() {
        Some("-") => Box::new(BufReader::new(io::stdin())),
        Some(input) => {
            let file = File::open(input)
                .map_err(|err| format!("failed to open file {}: {}", input, err))?;
            Box::new(BufReader::new(file))
        }
        None => Box::new(BufReader::new(io::stdin())),
    };
    let mut writer = BufWriter::new(io::stdout());

    let mut slicer = RowSlicer {
        reader,
        filters: FilterSet::new(opts.filters),
    };

    slicer
        .slice(&mut writer)
        .map_err(|err| format!("slice operation failed: {}", err))?;

    Ok(())
}

struct RowSlicer<R: BufRead> {
    reader: R,
    filters: FilterSet,
}

impl<R: BufRead> RowSlicer<R> {
    fn slice<W: Write>(&mut self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        let mut buf = String::new();
        let mut index = 0;

        loop {
            match self.reader.read_line(&mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    if self.filters.is_empty() || self.filters.apply(1 + index) {
                        write!(writer, "{}", buf)?;
                    }

                    buf.clear();
                    index += 1;
                }
                Err(err) => return Err(err.into()),
            }
        }

        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {}
