use std::{ io::{ self, Read, Write }, fmt::Write as FmtWrite, process::{Termination, ExitCode}, collections::HashSet, mem::{self, ManuallyDrop}, convert::identity };
use ansi_term::Color::*;
mod signalling_filter_map;
use signalling_filter_map::SignallingFilterMapExt as _;

mod lexer;
// mod count_remove;
use lexer::lex;

use crate::lexer::Delim;

fn to_eof(c: Option<char>) -> String {
    c.map_or("end of input".to_string(), |c| format!("'{c}'"))
}

// /// Copies a T out of &T, even if T: !Copy.
// unsafe fn clone_in_place<T>(value: &T) -> T {
//     std::ptr::read(value as *const T)
// }

// fn map_in_place<T, R>(vec: Vec<T>, mut func: impl FnMut(T) -> R) -> Vec<R> {
//     assert!(mem::size_of::<R>() <= mem::size_of::<T>(), "Result size must be <= target size");
//     assert!(mem::size_of::<Vec<T>>() == mem::size_of::<Vec<R>>(), "Size of two Vecs should be equal");
//     let vec = ManuallyDrop::new(vec.into_iter().map(|i| ManuallyDrop::new(i)).enumerate());
//     unsafe {
//         // SAFETY: idk
//         let rvec: Vec<R> = mem::transmute_copy(&vec);
//         for (n, i) in *vec {
//             rvec[n] = func(*i)
//         }
//         mem::forget(vec);
//         rvec;
//     }
// }

fn select_eof(set: impl Iterator<Item = Option<char>>) -> String {
    let mut eof = false;
    let chars: Vec<_> = set.signalling_filter_map(identity, &mut eof).collect();
    match (&*chars, eof) {
        ([], false) => "nothing (error may be earlier)".to_string(),
        ([i], false) => format!("'{i}'"),
        ([l, r], false) => format!("'{l}' or '{r}'"),
        ([c@.., e], false) => format!("{}, or '{e}'", c.into_iter().map(|i| format!("'{i}'")).fold(String::with_capacity(c.len() * 4), |mut str, i| { write!(str, ", {i}").expect("Failed to accumulate characters"); str })),
        ([], true) => "end of input".to_string(),
        ([i], true) => format!("'{i}' or end of input"),
        (c, true) => format!("{}, or end of input", c.into_iter().map(|i| format!("'{i}'")).fold(String::with_capacity(c.len() * 4), |mut str, i| { write!(str, ", {i}").expect("Failed to accumulate characters"); str })),
    }
}

fn main() -> io::Result<ExitCode> {
    let mut prgm = String::new();
    {
        let len = io::stdin().read_to_string(&mut prgm)?;
        prgm.truncate(len);
        prgm.shrink_to_fit();
    }
    match lex(&prgm) {
        Ok(tks) => {
            println!("{}", Green.bold().paint("Input parsed successfully"));
            println!("- token tree: {}", tks.into_iter().map(|tk| tk.to_string()).collect::<Vec<_>>().join(" "));
            io::Result::Ok(ExitCode::SUCCESS)
        },
        Err((errs, tks)) => {
            println!("{}", Red.bold().paint("Input failed to parse"));
            for err in errs {
                let mut stdout = io::stdout().lock();
                let span = err.span();
                write!(stdout, "{}: ", Blue.paint(format!("{}..{}", span.start, span.end)))?;
                let (result, hint) = match err.reason() {
                    chumsky::error::SimpleReason::Unexpected => (writeln!(stdout, "unexpected {}", to_eof(err.found().copied())), Some(select_eof(err.expected().copied()))),
                    chumsky::error::SimpleReason::Unclosed { span, delimiter } => (writeln!(stdout, "mismatched '{delimiter}'"), {
                        let delim = Delim::by_opening(*delimiter).expect("Delimiter character should have an entry in Delim");
                        let closing = delim.closing();
                        let mut expected_pair = false;
                        let expected = err.expected().signalling_filter_map(|i| {
                            match i {
                                None => Some(None),
                                Some(v) if v == closing => None,
                                
                            }
                        }, &mut expected_pair).peekable();
                        let msg = match expected.peek() {
                            Some(_) => format!("expected '{closing}', or also {}", select_eof(expected)),
                            None => format!("expected '{closing}' to close opening '{delimiter}'")
                        };
                        Some(msg)
                    }),
                    chumsky::error::SimpleReason::Custom(msg)_ => todo!(),
                };
            }
            if let Some(tks) = tks {
                println!("- recovered token tree: {}", tks.into_iter().map(|tk| tk.to_string()).collect::<Vec<_>>().join(" "));
            } else {
                println!("- token tree could not be recovered")
            }
            io::Result::Ok(ExitCode::FAILURE)
        }
    }
}

