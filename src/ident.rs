// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use proc_macro2::Ident;
use quote::format_ident;

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "static", "struct", "trait", "true", "type", "union", "unsafe", "use",
    "where", "while",
];

const ILLEGAL_WORDS: &[&str] = &["_", "super", "self", "Self", "crate", ""];

pub(crate) fn contains_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}

pub(crate) fn is_illegal(s: &str) -> bool {
    ILLEGAL_WORDS.contains(&s)
}

pub(crate) fn make_ident(ident: &str) -> Ident {
    if is_illegal(ident) {
        // prepending with r_ for illegal raw idents
        format_ident!("r_{ident}")
    } else if contains_keyword(ident) {
        // prepending with r_ for illegal raw idents
        format_ident!("r#{ident}")
    } else {
        format_ident!("{ident}")
    }
}
