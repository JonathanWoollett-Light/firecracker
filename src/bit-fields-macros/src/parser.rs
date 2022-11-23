// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use proc_macro2::{Delimiter, Literal, Span, TokenTree};

use crate::{Field, Flag, Member};

/// Parses a slice of tokens into an iterator of bit field members.
pub(crate) struct BitFieldMembersParser<'a> {
    /// Token slice iterator.
    iter: std::slice::Iter<'a, TokenTree>,
    /// Used identifiers.
    existing: HashSet<String>,
    /// Rustdoc.
    rustdoc: String,
    /// Data type size e.g. u8 = 8.
    size: u8,
    /// Error flag we set when encountering an error.
    error: bool,
}
impl<'a> From<(u8, std::slice::Iter<'a, TokenTree>)> for BitFieldMembersParser<'a> {
    fn from((size, iter): (u8, std::slice::Iter<'a, TokenTree>)) -> Self {
        Self {
            iter,
            existing: HashSet::new(),
            rustdoc: String::new(),
            size,
            error: false,
        }
    }
}

/// Error type for [`<BitFieldMembersParser<'_> as std::iter::Iterator>::next`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum BitFieldMembersParserIterError {
    /// Identifier already used.
    #[error("Identifier already used.")]
    DuplicateIdentifier,
    /// Failed to get field index.
    #[error("Failed to get field indices: {0}")]
    IndexField(IndexFieldError),
    /// Failed to get flag index.
    #[error("Failed to get flag index: {0}")]
    IndexFlag(IndexFlagError),
    /// Malformed rustdoc comment.
    #[error("Malformed rustdoc comment.")]
    RustdocMalformed,
    /// Rustdoc comment missing enclosing " characters.
    #[error("Rustdoc comment missing enclosing \" characters.")]
    RustdocUnenclosed,
    /// Badly defined members.
    #[error("Badly defined members: {0:?}.")]
    BadlyDefinedMembers(Vec<String>),
}

impl std::iter::Iterator for BitFieldMembersParser<'_> {
    type Item = Result<Member, crate::ProcError<BitFieldMembersParserIterError>>;
    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.error {
            return None;
        }
        loop {
            #[allow(clippy::pattern_type_mismatch)]
            match self.iter.as_slice() {
                // Bit field: [ident, punct, literal, punct, punct, literal, punct, ..]
                [TokenTree::Ident(ident), TokenTree::Punct(colon), TokenTree::Literal(start_token), TokenTree::Punct(d1), TokenTree::Punct(d2), TokenTree::Literal(stop_token), TokenTree::Punct(comma), ..]
                    if colon.as_char() == ':'
                        && d1.as_char() == '.'
                        && d2.as_char() == '.'
                        && comma.as_char() == ',' =>
                {
                    // Check indentifier not already used.
                    if !self.existing.insert(ident.to_string()) {
                        self.error = true;
                        return Some(Err((
                            ident.span(),
                            BitFieldMembersParserIterError::DuplicateIdentifier,
                        )));
                    }
                    // Check indices
                    return match index_field(start_token, stop_token, self.size) {
                        Ok((start, stop)) => {
                            // Get return type
                            let member = Member::Field(Field {
                                start,
                                rustdoc: self.rustdoc.clone(),
                                identifier: ident.clone(),
                                stop,
                            });
                            // Reset rustdoc
                            self.rustdoc.clear();
                            // Move past this identified slice.
                            self.iter.nth(6);
                            // Return member
                            Some(Ok(member))
                        }
                        Err((span, err)) => {
                            self.error = true;
                            Some(Err((span, BitFieldMembersParserIterError::IndexField(err))))
                        }
                    };
                }
                // Bit field: [ident, punct, literal, punct, punct, literal]
                [TokenTree::Ident(ident), TokenTree::Punct(colon), TokenTree::Literal(start_token), TokenTree::Punct(d1), TokenTree::Punct(d2), TokenTree::Literal(stop_token)]
                    if colon.as_char() == ':' && d1.as_char() == '.' && d2.as_char() == '.' =>
                {
                    // Check indentifier not already used.
                    if !self.existing.insert(ident.to_string()) {
                        self.error = true;
                        return Some(Err((
                            ident.span(),
                            BitFieldMembersParserIterError::DuplicateIdentifier,
                        )));
                    }
                    // Check indices
                    return match index_field(start_token, stop_token, self.size) {
                        Ok((start, stop)) => {
                            // Get return type
                            let member = Member::Field(Field {
                                start,
                                rustdoc: self.rustdoc.clone(),
                                identifier: ident.clone(),
                                stop,
                            });
                            // Reset rustdoc
                            self.rustdoc.clear();
                            // Move past this identified slice.
                            self.iter.nth(5);
                            // Return member
                            Some(Ok(member))
                        }
                        Err((span, err)) => {
                            self.error = true;
                            Some(Err((span, BitFieldMembersParserIterError::IndexField(err))))
                        }
                    };
                }
                // Bit flag: [ ident, punct, literal, punct, .. ]
                [TokenTree::Ident(ident), TokenTree::Punct(colon), TokenTree::Literal(index_token), TokenTree::Punct(comma), ..]
                    if colon.as_char() == ':' && comma.as_char() == ',' =>
                {
                    // Check indentifier not already used.
                    if !self.existing.insert(ident.to_string()) {
                        self.error = true;
                        return Some(Err((
                            ident.span(),
                            BitFieldMembersParserIterError::DuplicateIdentifier,
                        )));
                    }
                    return match index_flag(index_token, self.size) {
                        Ok(index) => {
                            // Get return type
                            let member = Member::Flag(Flag {
                                index,
                                rustdoc: self.rustdoc.clone(),
                                identifier: ident.clone(),
                            });
                            // Reset rustdoc
                            self.rustdoc.clear();
                            // Move past this identified slice.
                            self.iter.nth(3);
                            // Return member
                            Some(Ok(member))
                        }
                        Err((span, err)) => {
                            self.error = true;
                            Some(Err((span, BitFieldMembersParserIterError::IndexFlag(err))))
                        }
                    };
                }
                // Bit flag: [ ident, punct, literal]
                [TokenTree::Ident(ident), TokenTree::Punct(colon), TokenTree::Literal(index_token)]
                    if colon.as_char() == ':' =>
                {
                    // Check indentifier not already used.
                    if !self.existing.insert(ident.to_string()) {
                        self.error = true;
                        return Some(Err((
                            ident.span(),
                            BitFieldMembersParserIterError::DuplicateIdentifier,
                        )));
                    }
                    return match index_flag(index_token, self.size) {
                        Ok(index) => {
                            // Get return type
                            let member = Member::Flag(Flag {
                                index,
                                rustdoc: self.rustdoc.clone(),
                                identifier: ident.clone(),
                            });
                            // Reset rustdoc
                            self.rustdoc.clear();
                            // Move past this identified slice.
                            self.iter.nth(2);
                            // Return member
                            Some(Ok(member))
                        }
                        Err((span, err)) => {
                            self.error = true;
                            Some(Err((span, BitFieldMembersParserIterError::IndexFlag(err))))
                        }
                    };
                }
                // Rustdoc comment: [ punct, group, .. ]
                [TokenTree::Punct(punct), TokenTree::Group(doc_group), ..]
                    if punct.as_char() == '#' && doc_group.delimiter() == Delimiter::Bracket =>
                {
                    let rustdoc_vec = doc_group.stream().into_iter().collect::<Vec<_>>();
                    // From `#[doc="some comment"]` we are getting `"some comment"`
                    let doc_comment = match &*rustdoc_vec {
                        [TokenTree::Ident(group_ident), TokenTree::Punct(group_punct), TokenTree::Literal(group_lit)]
                            if *group_ident == "doc" && group_punct.as_char() == '=' =>
                        {
                            group_lit
                        }
                        _ => {
                            self.error = true;
                            return Some(Err((
                                doc_group.span(),
                                BitFieldMembersParserIterError::RustdocMalformed,
                            )));
                        }
                    };
                    // Check for then remove " from start and end of string.
                    let comment_unenclosed = {
                        let comment_str = doc_comment.to_string();
                        let mut chars = comment_str.chars();
                        if let (Some('"'), Some('"')) = (chars.next(), chars.next_back()) {
                            String::from(chars.as_str())
                        } else {
                            self.error = true;
                            return Some(Err((
                                doc_comment.span(),
                                BitFieldMembersParserIterError::RustdocUnenclosed,
                            )));
                        }
                    };
                    // Trim space leading spaces.
                    // E.g. A coment like `/// abcde` will become `" abcde"` and we want `abcde`.
                    let comment_trimmed = comment_unenclosed.trim_start();
                    // We append to the rustdoc string. When we hit a bit flag or field, we use the
                    // rustdoc string for this flag or field, then empty the rustdoc string.
                    self.rustdoc.push_str(comment_trimmed);
                    self.rustdoc.push(' ');
                    // Move past this identified slice.
                    self.iter.nth(1);
                }
                // On an exhausted iterator return none.
                [] => return None,
                // https://doc.rust-lang.org/proc_macro/struct.Span.html#method.join is curretly
                // unstable, but when it is stablized we should collect and join spans of remaining
                // element for this error message.
                _ => {
                    self.error = true;
                    return Some(Err((
                        Span::call_site(),
                        BitFieldMembersParserIterError::BadlyDefinedMembers(
                            self.iter
                                .clone()
                                .map(std::string::ToString::to_string)
                                .collect::<Vec<_>>(),
                        ),
                    )));
                }
            }
        }
    }
}

/// Error type for [`index_field`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum IndexFieldError {
    /// Failed to parse token for start index.
    #[error("Failed to parse token for start index: {0}")]
    ParseStart(std::num::ParseIntError),
    /// Start index outside of valid range.
    #[error("Start index ({start}) outside of valid range ({valid_range:?}).")]
    InvalidStart {
        /// Parsed start index.
        start: u8,
        /// Valid range that `start` lies outside of.
        valid_range: std::ops::Range<u8>,
    },
    /// Failed to parse token for stop index.
    #[error("Failed to parse token for stop index: {0}")]
    ParseStop(std::num::ParseIntError),
    /// Stop index outside of valid range.
    #[error("Stop index ({stop}) outside of valid range ({valid_range:?}).")]
    InvalidStop {
        /// Parsed stop index.
        stop: u8,
        /// Valid range that `stop` lies outside of.
        valid_range: std::ops::RangeInclusive<u8>,
    },
}

/// For bit field indices, checks if they are both within the range of the data type and the stop is
/// greater than or equal to the start then returns them as `u8`s.
fn index_field(
    start_token: &Literal,
    stop_token: &Literal,
    size: u8,
) -> Result<(u8, u8), crate::ProcError<IndexFieldError>> {
    // Get start, checking if in range of data type.
    let start = match start_token.to_string().parse::<u8>() {
        Ok(s) if (0..size).contains(&s) => Ok(s),
        Ok(s) => Err((
            start_token.span(),
            IndexFieldError::InvalidStart {
                start: s,
                valid_range: 0..size,
            },
        )),
        Err(err) => Err((start_token.span(), IndexFieldError::ParseStart(err))),
    }?;
    // Get stop, checking if in range of data type.
    let stop = match stop_token.to_string().parse::<u8>() {
        Ok(s) if (start..=size).contains(&s) => Ok(s),
        Ok(s) => Err((
            stop_token.span(),
            IndexFieldError::InvalidStop {
                stop: s,
                valid_range: start..=size,
            },
        )),
        Err(err) => Err((stop_token.span(), IndexFieldError::ParseStop(err))),
    }?;

    Ok((start, stop))
}

/// Error type for [`index_field`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum IndexFlagError {
    /// Failed to parse token for index.
    #[error("Failed to parse token for index: {0}")]
    Parse(std::num::ParseIntError),
    /// Index outside of valid range.
    #[error("Index ({index}) outside valid range ({valid_range:?}).")]
    Invalid {
        /// Parsed index.
        index: u8,
        /// Valid range that `index` lies outside of.
        valid_range: std::ops::Range<u8>,
    },
}

/// For  abit flag index, checks if it is within the range of the data type, then returns it as a
/// `u8`.
fn index_flag(index: &Literal, size: u8) -> Result<u8, crate::ProcError<IndexFlagError>> {
    // Get index, checking if in range of data type.
    match index.to_string().parse::<u8>() {
        Ok(s) if (0..size).contains(&s) => Ok(s),
        Ok(s) => Err((
            index.span(),
            IndexFlagError::Invalid {
                index: s,
                valid_range: 0..size,
            },
        )),
        Err(err) => Err((index.span(), IndexFlagError::Parse(err))),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        non_snake_case,
        clippy::dbg_macro,
        clippy::unwrap_used,
        clippy::as_conversions,
        clippy::shadow_unrelated
    )]

    use proc_macro2::{Group, Ident, Literal, Punct, Spacing, TokenTree};

    use super::*;

    // Construct an ident with a given string.
    fn ident(s: &str) -> Ident {
        Ident::new(s, Span::call_site())
    }
    fn punct(c: char) -> Punct {
        Punct::new(c, Spacing::Alone)
    }
    fn rustdoc(s: &str) -> [TokenTree; 2] {
        [
            TokenTree::Punct(punct('#')),
            TokenTree::Group(Group::new(Delimiter::Bracket, quote::quote! { doc=#s })),
        ]
    }
    fn field(name: &str, start: u8, stop: u8) -> [TokenTree; 6] {
        [
            TokenTree::Ident(ident(name)),
            TokenTree::Punct(punct(':')),
            TokenTree::Literal(Literal::u8_unsuffixed(start)),
            TokenTree::Punct(punct('.')),
            TokenTree::Punct(punct('.')),
            TokenTree::Literal(Literal::u8_unsuffixed(stop)),
        ]
    }
    fn field_comma(name: &str, start: u8, stop: u8) -> [TokenTree; 7] {
        [
            TokenTree::Ident(ident(name)),
            TokenTree::Punct(punct(':')),
            TokenTree::Literal(Literal::u8_unsuffixed(start)),
            TokenTree::Punct(punct('.')),
            TokenTree::Punct(punct('.')),
            TokenTree::Literal(Literal::u8_unsuffixed(stop)),
            TokenTree::Punct(punct(',')),
        ]
    }
    fn flag(name: &str, index: u8) -> [TokenTree; 3] {
        [
            TokenTree::Ident(ident(name)),
            TokenTree::Punct(punct(':')),
            TokenTree::Literal(Literal::u8_unsuffixed(index)),
        ]
    }
    fn flag_comma(name: &str, index: u8) -> [TokenTree; 4] {
        [
            TokenTree::Ident(ident(name)),
            TokenTree::Punct(punct(':')),
            TokenTree::Literal(Literal::u8_unsuffixed(index)),
            TokenTree::Punct(punct(',')),
        ]
    }

    #[test]
    fn bit_field_members_parser_empty() {
        let tokens = &[];
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_field() {
        let tokens = &field("field", 0, 1);
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 0, rustdoc, identifier: ident ,stop: 1 }))) if rustdoc.is_empty() && ident == "field")
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_field_comma() {
        let tokens = &field_comma("field", 0, 1);
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 0, rustdoc, identifier: ident ,stop: 1 }))) if rustdoc.is_empty() && ident == "field")
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_field_no_comma() {
        let tokens = &[field("field1", 0, 1), field("field2", 1, 2)].concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Err((_,BitFieldMembersParserIterError::BadlyDefinedMembers(vec)))) if vec == vec!["field1",":","0",".",".","1","field2",":","1",".",".","2",])
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_flag_duplicate() {
        let tokens = &[
            field_comma("field", 0, 1).as_slice(),
            field("field", 1, 2).as_slice(),
        ]
        .concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 0, rustdoc, identifier: ident ,stop: 1 }))) if rustdoc.is_empty() && ident == "field")
        );
        assert!(matches!(
            parser.next(),
            Some(Err((
                _,
                BitFieldMembersParserIterError::DuplicateIdentifier
            )))
        ));
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_flag_comma_duplicate() {
        let tokens = &[field_comma("field", 0, 1), field_comma("field", 1, 2)].concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 0, rustdoc, identifier: ident ,stop: 1 }))) if rustdoc.is_empty() && ident == "field")
        );
        assert!(matches!(
            parser.next(),
            Some(Err((
                _,
                BitFieldMembersParserIterError::DuplicateIdentifier
            )))
        ));
        dbg!(parser.next());
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_field_seperating_comma() {
        let tokens = &[
            field_comma("field1", 0, 1).as_slice(),
            field("field2", 1, 2).as_slice(),
        ]
        .concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 0, rustdoc, identifier: ident ,stop: 1 }))) if rustdoc.is_empty() && ident == "field1")
        );
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 1, rustdoc, identifier: ident ,stop: 2 }))) if rustdoc.is_empty() && ident == "field2")
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_field_rustdoc() {
        let tokens = &[
            rustdoc("some docs").as_slice(),
            field("field", 0, 1).as_slice(),
        ]
        .concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 0, rustdoc, identifier: ident ,stop: 1 }))) if rustdoc == "some docs " && ident == "field")
        );
        assert!(matches!(parser.next(), None));
    }

    #[test]
    fn bit_field_members_parser_flag() {
        let tokens = &flag("flag", 0);
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Flag(Flag { index: 0, rustdoc, identifier: ident }))) if rustdoc.is_empty() && ident == "flag")
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_flag_comma() {
        let tokens = &flag_comma("flag", 0);
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Flag(Flag { index: 0, rustdoc, identifier: ident }))) if rustdoc.is_empty() && ident == "flag")
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_flag_no_comma() {
        let tokens = &[flag("flag1", 0), flag("flag2", 1)].concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Err((_,BitFieldMembersParserIterError::BadlyDefinedMembers(vec)))) if vec == vec!["flag1",":","0","flag2",":","1"])
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_flag_seperating_comma() {
        let tokens = &[
            flag_comma("flag1", 0).as_slice(),
            flag("flag2", 1).as_slice(),
        ]
        .concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Flag(Flag { index: 0, rustdoc, identifier: ident }))) if rustdoc.is_empty() && ident == "flag1")
        );
        assert!(
            matches!(parser.next(),Some(Ok(Member::Flag(Flag { index: 1, rustdoc, identifier: ident }))) if rustdoc.is_empty() && ident == "flag2")
        );
        assert!(matches!(parser.next(), None));
    }
    #[test]
    fn bit_field_members_parser_flag_rustdoc() {
        let tokens = &[rustdoc("some docs").as_slice(), flag("flag", 0).as_slice()].concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Flag(Flag { index: 0, rustdoc, identifier: ident }))) if rustdoc == "some docs " && ident == "flag")
        );
        assert!(matches!(parser.next(), None));
    }

    #[test]
    fn bit_field_members_parser_mixed() {
        let tokens = &[
            rustdoc("some docs").as_slice(),
            flag_comma("flag1", 0).as_slice(),
            rustdoc("some more docs").as_slice(),
            field_comma("field1", 1, 2).as_slice(),
            flag_comma("flag2", 3).as_slice(),
            rustdoc("some extra docs").as_slice(),
            field_comma("field2", 4, 5).as_slice(),
        ]
        .concat();
        let mut parser = BitFieldMembersParser::from((8, tokens.iter()));
        assert!(
            matches!(parser.next(),Some(Ok(Member::Flag(Flag { index: 0, rustdoc, identifier: ident }))) if rustdoc == "some docs " && ident == "flag1")
        );
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 1, rustdoc, identifier: ident ,stop: 2 }))) if rustdoc == "some more docs " && ident == "field1")
        );
        assert!(
            matches!(parser.next(),Some(Ok(Member::Flag(Flag { index: 3, rustdoc, identifier: ident }))) if rustdoc.is_empty() && ident == "flag2")
        );
        assert!(
            matches!(parser.next(),Some(Ok(Member::Field(Field { start: 4, rustdoc, identifier: ident ,stop: 5 }))) if rustdoc == "some extra docs " && ident == "field2")
        );
        assert!(matches!(parser.next(), None));
    }

    #[test]
    fn index_field_suffixed() {
        let res = index_field(&Literal::u8_suffixed(0), &Literal::u8_suffixed(4), 8);
        dbg!(&res);
        assert!(matches!(res, Err((_, IndexFieldError::ParseStart(_)))));

        let res = index_field(&Literal::u8_unsuffixed(0), &Literal::u8_suffixed(4), 8);
        dbg!(&res);
        assert!(matches!(res, Err((_, IndexFieldError::ParseStop(_)))));
    }
    #[test]
    fn index_field_unsuffixed() {
        let res = index_field(&Literal::u8_unsuffixed(0), &Literal::u8_unsuffixed(4), 8);
        dbg!(&res);
        assert!(matches!(res, Ok((0, 4))));
    }

    #[test]
    fn index_field_start_outside() {
        let res = index_field(&Literal::u8_unsuffixed(8), &Literal::u8_unsuffixed(4), 8);
        dbg!(&res);
        assert!(
            matches!(res,Err((_,IndexFieldError::InvalidStart { start: 8, valid_range })) if (valid_range == (0..8)))
        );
    }
    #[test]
    fn index_field_stop_outside() {
        let res = index_field(&Literal::u8_unsuffixed(0), &Literal::u8_unsuffixed(9), 8);
        dbg!(&res);
        assert!(
            matches!(res,Err((_,IndexFieldError::InvalidStop { stop: 9, valid_range })) if (valid_range == (0..=8)) )
        );
    }
    #[test]
    fn index_field_stop_before() {
        let res = index_field(&Literal::u8_unsuffixed(4), &Literal::u8_unsuffixed(3), 8);
        dbg!(&res);
        assert!(
            matches!(res,Err((_,IndexFieldError::InvalidStop{ stop: 3, valid_range } )) if (valid_range == (4..=8)))
        );
    }
    #[test]
    fn index_flag_suffixed() {
        let res = index_flag(&Literal::u8_suffixed(4), 8);
        dbg!(&res);
        assert!(matches!(res, Err((_, IndexFlagError::Parse(_)))));
    }
    #[test]
    fn index_flag_unsuffixed() {
        let res = index_flag(&Literal::u8_unsuffixed(4), 8);
        dbg!(&res);
        assert!(matches!(res, Ok(4)));
    }
    #[test]
    fn index_flag_outside() {
        let res = index_flag(&Literal::u8_unsuffixed(8), 8);
        dbg!(&res);
        assert!(
            matches!(res,Err((_,IndexFlagError::Invalid { index: 8, valid_range })) if (valid_range == (0..8)))
        );
    }
}
