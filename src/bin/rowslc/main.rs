use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process;

use clap::Parser;

use inslice::filter::{Filter, FilterSet};

/// A command-line utility for filtering input text by rows and writing them to standard output
#[derive(Parser)]
#[clap(
    name = "rowslc",
    version = "1.0.0",
    author = "Jace Tan <jaceys.tan@gmail.com>"
)]
struct Opts {
    /// Path to input file. To read from standard input, specify - as the path. If no path is
    /// provided, the default behaviour will be to read from standard input.
    path: Option<String>,

    /// Filters to be applied, using row numbers to denote which rows from the input text should
    /// be retained. Multiple filters can be applied, the result of which is their union. The
    /// following are accepted formats for filters, with row indexing starting from one,
    /// beginning from the top-most row:
    ///
    /// * [n] - an exact filter for selecting the n'th row
    ///
    /// * [n:m] - a range-based filter for selecting the n'th to m'th (inclusive) rows
    ///     
    /// * [n:] - a range-based filter for selecting the n'th to last (inclusive) rows
    ///     
    /// * [:n] - a range-based filter for selecting the first to n'th (inclusive) rows
    ///     
    /// * [:n] - a range-based filter for selecting the first to last (inclusive) rows
    ///
    /// Example:
    ///
    /// `rowslc - -f 1 4:6` will result in the 1st, 4th, 5th, and 6th rows of the input text
    /// provided from standard input being written to standard output, separated by a newline.
    #[clap(short, long)]
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

    let reader: Box<dyn BufRead> = match opts.path.as_deref() {
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
mod test {
    use super::*;
    use std::str::FromStr;

    fn testdata() -> File {
        File::open("src/testdata/input.txt").unwrap()
    }

    fn execute(filters: Vec<Filter>, expected: &str) -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }

    #[test]
    fn rowslc_slice_exact_ok() -> Result<(), Box<dyn Error>> {
        let filters = vec![Filter::from_str("1")?];
        let expected = "\
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
";

        execute(filters, expected)
    }

    #[test]
    fn rowslc_slice_exact_multiple_ok() -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let filters = vec![Filter::from_str("1")?, Filter::from_str("3")?];
        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        let expected = "\
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
redis        6.2-alpine    6960a2858b36   3 days ago      31.3MB
";

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }

    #[test]
    fn rowslc_slice_range_ok() -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let filters = vec![Filter::from_str("1:3")?];
        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        let expected = "\
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
vault        1.8.4         dc15db720d79   2 days ago      186MB
redis        6.2-alpine    6960a2858b36   3 days ago      31.3MB
";

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }

    #[test]
    fn rowslc_slice_range_multiple_ok() -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let filters = vec![Filter::from_str("1:2")?, Filter::from_str("4:5")?];
        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        let expected = "\
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
vault        1.8.4         dc15db720d79   2 days ago      186MB
postgres     14.0-alpine   ae192c4d3ada   17 months ago   152MB
traefik      2.5           72bfc37343a4   18 months ago   68.9MB";

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }

    #[test]
    fn rowslc_slice_exact_and_range_ok() -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let filters = vec![Filter::from_str("1")?, Filter::from_str("3:4")?];
        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        let expected = "\
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
redis        6.2-alpine    6960a2858b36   3 days ago      31.3MB
postgres     14.0-alpine   ae192c4d3ada   17 months ago   152MB
";

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }

    #[test]
    fn rowslc_slice_range_start_ok() -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let filters = vec![Filter::from_str("3:")?];
        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        let expected = "\
redis        6.2-alpine    6960a2858b36   3 days ago      31.3MB
postgres     14.0-alpine   ae192c4d3ada   17 months ago   152MB
traefik      2.5           72bfc37343a4   18 months ago   68.9MB";

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }

    #[test]
    fn rowslc_slice_range_end_ok() -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let filters = vec![Filter::from_str(":3")?];
        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        let expected = "\
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
vault        1.8.4         dc15db720d79   2 days ago      186MB
redis        6.2-alpine    6960a2858b36   3 days ago      31.3MB
";

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }

    #[test]
    fn rowslc_slice_range_full_ok() -> Result<(), Box<dyn Error>> {
        let mut writer = Vec::new();

        let filters = vec![Filter::from_str(":")?];
        let mut slicer = RowSlicer {
            reader: BufReader::new(testdata()),
            filters: FilterSet::new(filters),
        };

        let expected = "\
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
vault        1.8.4         dc15db720d79   2 days ago      186MB
redis        6.2-alpine    6960a2858b36   3 days ago      31.3MB
postgres     14.0-alpine   ae192c4d3ada   17 months ago   152MB
traefik      2.5           72bfc37343a4   18 months ago   68.9MB";

        slicer.slice(&mut writer)?;
        assert_eq!(String::from_utf8(writer)?, expected);
        Ok(())
    }
}
