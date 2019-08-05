#![allow(unused_imports)]
#[macro_use]
extern crate quick_error;
#[cfg(unix)]
extern crate unix_socket;

use quick_error::ResultExt;
use std::fs::{metadata, File};
use std::io::{self, stderr, stdin, stdout, BufWriter, Read, Write};

#[cfg(unix)]
use std::net::Shutdown;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use unix_socket::UnixStream;

#[derive(PartialEq)]
pub enum NumMode {
    NumNull,
    NumNonEmpty,
    NumAll,
}

quick_error! {
    #[derive(Debug)]
    pub enum Errors {
        Input(err: io::Error, path: String) {
            display("cat: {0}: {1}", path, err)
            context(path: &'a str, err: io::Error) -> (err, path.to_owned())
            cause(err)
        }

        Output(err: io::Error) {
            display("cat: {0}", err) from()
            cause(err)
        }

        Filetype(p: String) {
            display("cat: {0}: unknown filetype", p)
        }

        Problem(c: usize) {
            display("cat: {0} problems", c)
        }

        Directory(p: String) {
            display("cat: {0}: Is a directory", p)
        }
    }
}

/// Options for output.
pub struct Options {
    pub number: NumMode,     // Line numbering mode
    pub squeeze_blank: bool, // Compress repeated empty lines
    pub show_tabs: bool,     // show TAB characters
    pub tab: String,         // string to show when show_tabs is on
    pub end_of_line: String, // show characters other than \n at line ends
    pub show_nonprint: bool, // use ^ and M- notation
}

pub struct Handle {
    reader: Box<Read>
}

/// Recognized file types.
pub enum Type {
    Directory,
    File,
    Stdin,
    SymLink,
    #[cfg(unix)]
    BlockDevice,
    #[cfg(unix)]
    CharDevice,
    #[cfg(unix)]
    Fifo,
    #[cfg(unix)]
    Socket,
}

type CatResult<T> = Result<T, Errors>;

/// Determines file type of input file *path*.
pub fn get_input_type(path: &str) -> CatResult<Type> {
    if path == "-" {
        return Ok(Type::Stdin);
    }

    match metadata(path).context(path)?.file_type() {
        #[cfg(unix)]
        ft if ft.is_block_device() => {
            Ok(Type::BlockDevice)
        }
        #[cfg(unix)]
        ft if ft.is_char_device() => {
            Ok(Type::CharDevice)
        }
        #[cfg(unix)]
        ft if ft.is_fifo() => {
            Ok(Type::Fifo)
        }
        #[cfg(unix)]
        ft if ft.is_socket() => {
            Ok(Type::Socket)
        }
        ft if ft.is_dir() => {
            Ok(Type::Directory)
        },
        ft if ft.is_file() => {
            Ok(Type::File)
        },
        ft if ft.is_symlink() => {
            Ok(Type::SymLink)
        },
        _ => Err(Errors::Filetype(path.to_owned())),
    }
}

/// Opens a file.
/// Returns a Handle from which a Reader can be accessed, or an error
pub fn open(path: &str) -> CatResult<Handle> {
    if path == "-" {
        let stdin = stdin();
        return Ok(Handle {
            reader: Box::new(stdin) as Box<Read>,
        });
    }

    match get_input_type(path)? {
        Type::Directory => {
            Err(Errors::Directory(path.to_owned()))
        },
        #[cfg(unix)]
        Type::Socket => {
            let socket = UnixStream::connect(path).context(path)?;
            socket.shutdown(Shutdown::Write).context(path)?;
            Ok(Handle {
                reader: Box::new(socket) as Box<Read>,
            })
        },
        _ => {
            let file = File::open(path).context(path)?;
            Ok(Handle {
                reader: Box::new(file) as Box<Read>,
            })
        }
    }
}

/// Writes files to stdout with no configuration. This allows
/// a simple memory copy. Returns Ok(()) if no errors
/// were encountered, or an error with the number of errors encountered
///
/// Takes a vector of file paths as an argument.
pub fn write_fast(files: Vec<&str>) -> CatResult<()> {
    let mut writer = stdout();
    let mut in_buf = [0; 1024 * 64];
    let mut error_count = 0;

    for file in files {
        match open(&file) {
            Ok(mut handle) => while let Ok(n) = handle.reader.read(&mut in_buf) {
                if n == 0 {
                    break;
                }
                writer.write_all(&in_buf[..n]).context(&file[..])?;
            },
            Err(e) => {
                writeln!(&mut stderr(), "{}", e)?;
                error_count += 1;
            }
        }
    }

    match error_count {
        0 => Ok(()),
        _ => Err(Errors::Problem(error_count))
    }
}

pub struct OutputState {
    line_number: usize,   // the current line number
    at_line_start: bool,  // whether the output cursor is at the beginning of a new line
}

/// Writes files to stdout with 'options' as configuration. Returns Ok
/// if no errors were encountered, or an error with the number of
/// errors encountered.

pub fn write_lines(files: Vec<&str>, options: &Options) -> CatResult<()> {
    let mut error_count = 0;
    let mut state = OutputState {
        line_number: 1,
        at_line_start: true
    };

    for file in files {
        if let Err(e) = write_file_lines(&file, options, &mut state) {
            writeln!(&mut stderr(), "{}", e).context(&file[..])?;
            error_count += 1;
        }
    }

    match error_count {
        0 => Ok(()),
        _ => Err(Errors::Problem(error_count))
    }
}

/// Outputs file contents to stdout, propagating errors.
///
/// # Arguments
///
/// **file** is a path to the file, **options** is a reference to an Options struct, and **state** is an
/// OutputState.
pub fn write_file_lines(file: &str, options: &Options, state: &mut OutputState) -> CatResult<()> {
    let mut handle = open(file)?;
    let mut in_buf = [0; 1024 * 31];
    let mut writer = BufWriter::with_capacity(1024 * 64, stdout());
    let mut one_blank: bool = false;

    while let Ok(n) = handle.reader.read(&mut in_buf) {
        if n == 0 {
            break;
        }
        let in_buf = &in_buf[..n];
        let mut pos = 0;
        while pos < n {
            // skip empty line_number enumerating them if needed

            if in_buf[pos] == b'\n' {
                if !state.at_line_start || !options.squeeze_blank || !one_blank {
                    one_blank = true;
                    if state.at_line_start && options.number == NumMode::NumAll {
                        write!(&mut writer, "{0:6}\t", state.line_number)?;
                        state.line_number += 1;
                    }

                    writer.write_all(options.end_of_line.as_bytes())?;
                    // writer.flash().context(&file[..])?;
                }
                state.at_line_start = true;
                pos += 1;
                continue;
            }

            one_blank = false;
            if state.at_line_start && options.number != NumMode::NumNull {
                write!(&mut writer, "{0:6}\t", state.line_number)?;
                state.line_number += 1;
            }

            // print to end of line, or buffer
            let offset = if options.show_nonprint {
                write_nonprint_to_end(&in_buf[pos..], &mut writer, options.tab.as_bytes())
            } else if options.show_tabs {
                write_tab_to_end(&in_buf[pos..], &mut writer)
            } else {
                write_to_end(&in_buf[pos..], &mut writer)
            };

            if offset == 0 {
                state.at_line_start = false;
                break;
            }

            // print appropriate line ender
            writer.write_all(options.end_of_line.as_bytes())?;
            // writer.flush()?;
            //
            // note: you may need the interactive bit for this

            state.at_line_start = true;
            pos += offset;
        }
    }

    Ok(())
}

/// Write all symbols until the end of line, or until the end of buffer is reached
/// Returns the number of written symbols +1, or 0 if the end is reached
pub fn write_to_end<W: Write>(in_buf: &[u8], writer: &mut W) -> usize {
    match in_buf.iter().position(|c| *c == b'\n') {
        Some(p) => {
            writer.write_all(&in_buf[..p]).unwrap();
            p + 1
        },
        None => {
            writer.write_all(in_buf).unwrap();
            0
        }
    }
}

pub fn write_tab_to_end<W: Write>(mut in_buf: &[u8], writer: &mut W) -> usize {
    let mut count = 0;
    loop {
        match in_buf
            .iter()
            .position(|c| *c == b'\n' || *c == b'\t')
        {
            Some(p) => {
                writer.write_all(&in_buf[..p]).unwrap();
                if in_buf[p] == b'\n' {
                    return count + p + 1;
                } else {
                    writer.write_all(b"^I").unwrap();
                    in_buf = &in_buf[p + 1..];
                    count += p + 1;
                }
            },
            None => {
                writer.write_all(in_buf).unwrap();
                return 0;
            }
        };
    }
}

pub fn write_nonprint_to_end<W: Write>(in_buf: &[u8], writer: &mut W, tab: &[u8]) -> usize {
    let mut count = 0;

    for byte in in_buf.iter().map(|c| *c) {
        if byte == b'\n' {
            break;
        }
        match byte {
            9 => writer.write_all(tab),
            0...8 | 10...31 => writer.write_all(&[b'^', byte + 64]),
            32...126 => writer.write_all(&[byte]),
            127 => writer.write_all(&[b'^', byte - 64]),
            128...159 => writer.write_all(&[b'M', b'-', b'^', byte - 64]),
            160...254 => writer.write_all(&[b'M', b'-', byte - 128]),
            _ => writer.write_all(&[b'M', b'-', b'^', 63]),
        }.unwrap();
        count += 1;
    }
    if count != in_buf.len() {
        count + 1
    } else {
        0
    }
}

