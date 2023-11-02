extern crate chumsky;
use std::{str::FromStr, fmt::Display, convert::identity, io};
use chumsky::{prelude::*, combinator::ThenIgnore, recursive::Recursive, Stream};
use lazy_static::lazy_static;

macro_rules! count {
    () => { 0 };
    ($cur:tt $($rest:tt)*) => { count!($($rest)*) + 1 }
}

macro_rules! literals {
    ($vis:vis $name:ident: $($ident:ident = $text:literal),+) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        $vis enum $name {
            $($ident),+
        }
        impl $name {
            pub const VALUES: [$name; count!($($ident)+)] = [$($name::$ident),+];
            pub const fn into_str(self) -> &'static str {
                match self {
                    $(
                        Self::$ident => $text
                    ),+
                }
            }
        }
        impl Into<&'static str> for $name {
            fn into(self) -> &'static str {
                self.into_str()
            }
        }
    }
}
macro_rules! delims {
    ($vis:vis $name:ident: $($open:literal $ident:ident $close:literal),+) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        $vis enum $name {
            $($ident),+
        }
        impl $name {
            pub const VALUES: [$name; count!($($ident)+)] = [$($name::$ident),+];
            pub const fn opening(self) -> char {
                match self {
                    $(
                        Self::$ident => $open
                    ),+
                }
            }
            pub const fn closing(self) -> char {
                match self {
                    $(
                        Self::$ident => $close
                    ),+
                }
            }
            pub const fn by_opening(c: char) -> Option<Self> {
                match c {
                    $(
                        $open => Some(Self::$ident),
                    )+
                    _ => None
                }
            }
            pub const fn by_closing(c: char) -> Option<Self> {
                match c {
                    $(
                        $close => Some(Self::$ident),
                    )+
                    _ => None
                }
            }
            pub const fn by_chars(cc: (char, char)) -> Option<Self> {
                match cc {
                    $(
                        ($open, $close) => Some(Self::$ident),
                    )+
                    _ => None
                }
            }
            #[inline]
            pub const fn chars(self) -> (char, char) {
                (self.opening(), self.closing())
            }
        }
        impl Into<(char, char)> for $name {
            fn into(self) -> (char, char) {
                self.chars()
            }
        }
    }
}

literals!(pub Kw: Fn = "fn", Class = "class", Struct = "struct");
literals!(pub Sym: Bang = "!", At = "@", Hash = "#", Dollar = "$", Percent = "%", Caret = "^", And = "&", Mul = "*", Minus = "-", Plus = "+", Equal = "=", Or = "|", Semi = ";", Colon = ":", Left = "<", Right = ">", Comma = ",", Dot = ".", Slash = "/", Ques = "?");
delims!(pub Delim: '(' Paren ')', '[' Bracket ']', '{' Brace '}');

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Num {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64)
}
impl Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (n, s) = match *self {
            Num::I8(i) => (i.to_string(), ":8"),
            Num::I16(i) => (i.to_string(), ":16"),
            Num::I32(i) => (i.to_string(), ":32"),
            Num::I64(i) => (i.to_string(), ":64"),
        };
        let s = Color::White.dimmed().paint(s);
        write!(f, "{n}{s}")
    }
}

impl Delim {
    pub const fn encloser(self) -> impl Fn(Vec<Tk>) -> Tk {
        return move |tree| self.enclose(tree)
    }
    pub const fn enclose(self, tree: Vec<Tk>) -> Tk {
        Tk::Delim(self, tree)
    }
    pub fn enclose_just(self, tk: Tk) -> Tk {
        Tk::Delim(self, vec![tk])
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Tk {
    Ident(Box<str>),
    Kw(Kw),
    Sym(Sym),
    Num(Num),
    Delim(Delim, Vec<Tk>),
    Err
}

use ansi_term::{Color, Style};


impl Display for Tk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Tk::Ident(name) => Color::Blue.paint(name.as_ref()),
            Tk::Kw(kw) => Color::Purple.paint(<&'static str>::from((*kw).into())),
            Tk::Sym(sym) => Color::Yellow.paint(<&'static str>::from((*sym).into())),
            Tk::Num(n) => Color::Green.paint(n.to_string()),
            Tk::Delim(d, ref inner) => {
                let delim_style = Style::new().bold();
                let inner = inner.into_iter().map(ToString::to_string).collect::<Vec<_>>().join(" ");
                let (start, end) = d.chars();
                let start = delim_style.paint(start.to_string());
                let end = delim_style.paint(end.to_string());
                delim_style.paint(format!("{start}{inner}{end}"))
            },
            Tk::Err => Color::Red.bold().paint("?")
        };
        write!(f, "{}", out)
    }
}

fn readint<A: 'static>(radix: u32, parser: impl Parser<char, A, Error = Simple<char>> + 'static) -> impl Parser<char, Tk, Error = Simple<char>> {
    fn readfit<T: FromStr + 'static, A>(parser: impl Parser<char, String, Error = Simple<char>> + 'static, suffix: impl Parser<char, A, Error = Simple<char>>, wrap: fn(T) -> Num) -> impl Parser<char, Tk, Error = Simple<char>> where <T as FromStr>::Err: Display {
        parser.boxed().then_ignore(suffix).try_map(move |s, span| {
            T::from_str(&s)
            .map_or_else(
                move |e| Err(Simple::custom(span, format!("Invalid number: {}", e))),
                move |n| Ok(Tk::Num(wrap(n)))
            )
        })
    }

    let p = parser.ignore_then(text::int(radix)).boxed();
    choice((
        readfit(p.clone(), just(":8"), Num::I8),
        readfit(p.clone(), just(":16"), Num::I16),
        readfit(p.clone(), just(":32").or_not(), Num::I32),
        readfit(p, just(":64"), Num::I64),
    ))
}

pub fn never<I: Clone, O, E: chumsky::Error<I>>() -> impl Parser<I, O, Error = E> {
    filter_map(|span, c| Err(E::expected_input_found(span, [], Some(c))))
}

pub fn lexer() -> impl Parser<char, Vec<Tk>, Error = Simple<char>> {
    recursive(|lexer|
        choice([
            text::ident().map(|ident: String| {
                for kw in Kw::VALUES {
                    if ident == kw.into_str() {
                        return Tk::Kw(kw);
                    }
                }
                Tk::Ident(ident.into_boxed_str())
            }).labelled("ident").boxed(),
            Sym::VALUES.into_iter().fold(never().boxed(), |p, c| p.or(just::<char, _, _>(c.into_str()).to(Tk::Sym(c))).boxed()).boxed(),
            readint(16, just("\\x")).boxed(),
            readint(10, just("\\d").or_not()).boxed(),
            readint(8, just("\\o")).boxed(),
            readint(2, just("\\b")).boxed(),
            lexer.clone().delimited_by(just('('), just(')')).map(Delim::Paren.encloser()).recover_with(nested_delimiters('(', ')', [('[', ']'), ('{', '}')], |_| Tk::Err)).boxed(),
            lexer.clone().delimited_by(just('['), just(']')).map(Delim::Bracket.encloser()).recover_with(nested_delimiters('[', ']', [('(', ')'), ('{', '}')], |_| Tk::Err)).boxed(),
            lexer.delimited_by(just('{'), just('}')).map(Delim::Brace.encloser()).recover_with(nested_delimiters('{', '}', [('(', ')'), ('[', ']')], |_| Tk::Err)).boxed(),
        ]).labelled("token").padded().repeated().labelled("token sequence")
    )
}

pub fn lex(input: &str) -> Result<Vec<Tk>, (Vec<Simple<char>>, Option<Vec<Tk>>)> {
    match lexer().parse_recovery(input) {
        (Some(result), err) if err.len() == 0 => Ok(result),
        (Some(result), err) => Err((err, Some(result))),
        (None, err) if err.len() == 0 => unreachable!(),
        (None, err) => Err((err, None))
    }
}
