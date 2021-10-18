[![ci](https://github.com/jace-ys/inslice/workflows/ci/badge.svg)](https://github.com/jace-ys/inslice/actions?query=workflow%3Aci)
[![release](https://github.com/jace-ys/inslice/workflows/release/badge.svg)](https://github.com/jace-ys/inslice/actions?query=workflow%3Arelease)

# `inslice`

Extracting specific columns and rows from a chunk of text is a common task one needs to do in the command-line, whether it's from files or standard input. However, current ways of doing so, such as `awk` or `tail`, are not the most straightforward or intuitive, especially since they typically require special syntax or invocations to achieve the task at hand.

`inslice` is a command-line utility written in Rust that aims to address this problem, by allowing one to easily filter input text by columns and rows in a literal and explicit manner using column and row numbers. It is made up of two separate binaries, `colslc` and `rowslc`, that operate on columns and rows respectively, and can be used in conjunction to achieve the desired output. This follows the Unix philosophy of writing programs that do one thing and do it well, and that work together.

To draw similarities to existing equivalent commands, see [Comparisons](#comparisons).

## Installation

#### Pre-Built Binaries

Pre-built `colslc` and `rowslc` binaries compiled for various target platforms can be found under the [Releases](https://github.com/jace-ys/inslice/releases) section of this repository.

#### Cargo

To install `colslc` and `rowslc` using the `cargo` toolchain:

```shell
cargo install inslice
```

## Usage

### `colslc`

```
colslc 1.0.0

Jace Tan <jaceys.tan@gmail.com>

A command-line utility for filtering input text by columns and writing them to standard output

USAGE:
    colslc [OPTIONS] [--] [PATH]

ARGS:
    <PATH>
            Path to input file. To read from standard input, specify - as the path. If no path is
            provided, the default behaviour will be to read from standard input

FLAGS:
    -h, --help
            Print help information

    -V, --version
            Print version information

OPTIONS:
    -d, --delimiter <DELIMITER>
            Optional delimiter to use for splitting input text into columns. If no delimiter is
            provided, the default behaviour will be to split by any amount of whitespace

    -f, --filters <FILTERS>...
            Filters to be applied, using column numbers to denote which columns from the input text
            should be retained. Multiple filters can be applied, the result of which is their union.
            The following are accepted formats for filters, with column indexing starting from one,
            beginning from the left-most column:
            
            * [n] - an exact filter for selecting the n'th column
            
            * [n:m] - a range-based filter for selecting the n'th to m'th (inclusive) columns
            
            * [n:] - a range-based filter for selecting the n'th to last (inclusive) columns
            
            * [:n] - a range-based filter for selecting the first to n'th (inclusive) columns
            
            * [:n] - a range-based filter for selecting the first to last (inclusive) columns
            
            Example:
            
            `colslc - -f 1 4:6` will result in the 1st, 4th, 5th, and 6th columns of the input text
            provided from standard input being written to standard output, separated by whitespace.
```

### `rowslc`

```
rowslc 1.0.0

Jace Tan <jaceys.tan@gmail.com>

A command-line utility for filtering input text by rows and writing them to standard output

USAGE:
    rowslc [OPTIONS] [--] [PATH]

ARGS:
    <PATH>
            Path to input file. To read from standard input, specify - as the path. If no path is
            provided, the default behaviour will be to read from standard input

FLAGS:
    -h, --help
            Print help information

    -V, --version
            Print version information

OPTIONS:
    -f, --filters <FILTERS>...
            Filters to be applied, using row numbers to denote which rows from the input text should
            be retained. Multiple filters can be applied, the result of which is their union. The
            following are accepted formats for filters, with row indexing starting from one,
            beginning from the top-most row:
            
            * [n] - an exact filter for selecting the n'th row
            
            * [n:m] - a range-based filter for selecting the n'th to m'th (inclusive) rows
            
            * [n:] - a range-based filter for selecting the n'th to last (inclusive) rows
            
            * [:n] - a range-based filter for selecting the first to n'th (inclusive) rows
            
            * [:n] - a range-based filter for selecting the first to last (inclusive) rows
            
            Example:
            
            `rowslc - -f 1 4:6` will result in the 1st, 4th, 5th, and 6th rows of the input text
            provided from standard input being written to standard output, separated by a newline.
```

## Comparisons

For the given input file:

```shell
$ cat src/testdata/input.txt
REPOSITORY   TAG           IMAGE ID       CREATED         SIZE
vault        1.8.4         dc15db720d79   2 days ago      186MB
redis        6.2-alpine    6960a2858b36   3 days ago      31.3MB
postgres     14.0-alpine   ae192c4d3ada   17 months ago   152MB
traefik      2.5           72bfc37343a4   18 months ago   68.9MB
```

### `colslc`

#### `awk`

```shell
cat src/testdata/input.txt | awk '{ print $1, $4, $5, $6}'
```

is equivalent to

```shell
cat src/testdata/input.txt | colslc -f 1 4:6
```

### `rowslc`

#### `head`

```shell
cat src/testdata/input.txt | head -3
```

is equivalent to 

```
cat src/testdata/input.txt | rowslc -f :3
```

#### `tail`

```shell
cat src/testdata/input.txt | tail +3
```

is equivalent to 

```
cat src/testdata/input.txt | rowslc -f 3:
```

#### `head` + `tail`

```shell
cat src/testdata/input.txt | head -3 | tail +3
```

is equivalent to 

```
cat src/testdata/input.txt | rowslc -f 3
```

## License

See [LICENSE](LICENSE).