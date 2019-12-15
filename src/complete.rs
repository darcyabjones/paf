use std::convert::TryInto;

use nom::IResult;
use nom::Offset;
use nom::bytes::complete::is_not;
use nom::character::complete::{
    tab,
    newline,
    digit1,
    one_of
};
use nom::sequence::{tuple, terminated, preceded};
use nom::multi::separated_list;
use nom::combinator::{
    map,
    map_res,
    cut,
    opt,
    all_consuming
};
use nom::error::{
    context,
    ParseError,
    VerboseError,
    ErrorKind,
    VerboseErrorKind
};

use crate::paf::{Strand, Locus, PAF};


/// Parse a + or - character as a strand object.
fn strand<I, E>(i: I) -> IResult<I, Strand, E>
where
    I: Clone,
    I: nom::InputIter,
    I: nom::Slice<std::ops::RangeFrom<usize>>,
    I: nom::FindToken<<I as nom::InputIter>::Item>,
    <I as nom::InputIter>::Item: nom::AsChar + Copy,
    &'static str: nom::FindToken<<I as nom::InputIter>::Item>,
    E: ParseError<I>,
{
    map(
        context("expected either '+' or '-'", one_of("+-")),
        |s| s.try_into().unwrap()
    )(i)
}


/// Parse any str of non-tab characters as a string.
fn string_str<'a, E>(i: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str>
{
    context(
        "expected an utf-8 string",
        map(is_not("\t\r\n"), |bs| String::from(bs))
    )(i)
}


/// Parse a byte string of non-tab characters as a string.
fn string_u8<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>
{
    context(
        "expected an utf-8 string",
        map_res(is_not(&b"\t\r\n"[..]), |bs| {
            std::str::from_utf8(bs).map(|s| s.to_string())
        })
    )(i)
}


/// Parse a string as a u64.
fn uint64_str<'a, E>(i: &'a str) -> IResult<&'a str, u64, E>
where
    E: ParseError<&'a str>
{
    context(
        "expected an unsigned 64-bit integer",
        map_res(digit1, |s: &str| s.parse::<u64>())
    )(i)
}


/// Parse a byte string as a u64.
fn uint64_u8<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], u64, E>
where
    E: ParseError<&'a [u8]>
{
    context(
        "expected an unsigned 64-bit integer",
        map_res(digit1, |s| {
            // This should be safe because we check for digits with nom.
            let string = unsafe { std::str::from_utf8_unchecked(s) };
            string.parse()
        }),
    )(i)
}


/// Parse a string as a u8.
fn uint8_str<'a, E>(i: &'a str) -> IResult<&'a str, u8, E>
where
    E: ParseError<&'a str>
{
    context(
        "expected an unsigned 8-bit integer",
        map_res(digit1, |s: &str| s.parse::<u8>())
    )(i)
}


/// Parse a byte string as a u8.
fn uint8_u8<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], u8, E>
where
    E: ParseError<&'a [u8]>
{
    context(
        "expected an unsigned 8-bit integer",
        map_res(digit1, |s| {
            // This should be safe because we check for digits with nom.
            let string = unsafe { std::str::from_utf8_unchecked(s) };
            string.parse()
        }),
    )(i)
}


/// Parse a string as a locus object.
pub(crate) fn locus_str<'a, E>(i: &'a str) -> IResult<&'a str, Locus, E>
where
    E: ParseError<&'a str>
{
    map(
        tuple((
            context("in column: seqid", terminated(string_str, tab)),
            context("in column: length", terminated(uint64_str, tab)),
            context("in column: start", terminated(uint64_str, tab)),
            context("in column: end", uint64_str)
        )),
        |tup| Locus::new(tup.0, tup.1, tup.2, tup.3)
    )(i)
}


/// Parse a string as a locus object.
pub(crate) fn locus_u8<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], Locus, E>
where
    E: ParseError<&'a [u8]>
{
    map(
        tuple((
            context("in column: seqid", terminated(string_u8, tab)),
            context("in column: length", terminated(uint64_u8, tab)),
            context("in column: start", terminated(uint64_u8, tab)),
            context("in column: end", uint64_u8)
        )),
        |tup| Locus::new(tup.0, tup.1, tup.2, tup.3)
    )(i)
}


/// Parse the optional sam fields.
fn sam_fields_str<'a, E>(i: &'a str) -> IResult<&'a str, Vec<String>, E>
where
    E: ParseError<&'a str>
{
    separated_list(tab, string_str)(i)
}


/// Parse the optional sam fields.
fn sam_fields_u8<'a, E>(i: &'a [u8]) -> IResult<&'a [u8], Vec<String>, E>
where
    E: ParseError<&'a [u8]>
{
    separated_list(tab, string_u8)(i)
}


/// Parse a string as a complete paf line.
pub fn paf_str<'a, E>(line: &'a str) -> IResult<&'a str, PAF, E>
where
    E: ParseError<&'a str>
{
    map(
        tuple((
            context("in column: query seqid", terminated(string_str, tab)),
            context("in column: query length", terminated(uint64_str, tab)),
            context("in column: query start", terminated(uint64_str, tab)),
            context("in column: query end", terminated(uint64_str, tab)),
            context("in column: strand", terminated(strand, tab)),
            context("in column: target seqid", terminated(string_str, tab)),
            context("in column: target length", terminated(uint64_str, tab)),
            context("in column: target start", terminated(uint64_str, tab)),
            context("in column: target end", terminated(uint64_str, tab)),
            context("in column: number matches", terminated(uint64_str, tab)),
            context("in column: alignment length", terminated(uint64_str, tab)),
            context("in column: mapping quality", uint8_str),
            context("in column: optional sam fields", preceded(opt(tab), sam_fields_str)),
            opt(newline)
        )),
        |tup| {
            let query = Locus::new(tup.0, tup.1, tup.2, tup.3);
            let target = Locus::new(tup.5, tup.6, tup.7, tup.8);
            PAF::new(query, tup.4, target, tup.9, tup.10, tup.11, tup.12)
        }
    )(line)
}


/// Parse a byte string as a complete paf line.
pub fn paf_u8<'a, E>(line: &'a [u8]) -> IResult<&'a [u8], PAF, E>
where
    E: ParseError<&'a [u8]>
{
    map(
        tuple((
            context("in column: query seqid", terminated(string_u8, tab)),
            context("in column: query length", terminated(uint64_u8, tab)),
            context("in column: query start", terminated(uint64_u8, tab)),
            context("in column: query end", terminated(uint64_u8, tab)),
            context("in column: strand", terminated(strand, tab)),
            context("in column: target seqid", terminated(string_u8, tab)),
            context("in column: target length", terminated(uint64_u8, tab)),
            context("in column: target start", terminated(uint64_u8, tab)),
            context("in column: target end", terminated(uint64_u8, tab)),
            context("in column: number matches", terminated(uint64_u8, tab)),
            context("in column: alignment length", terminated(uint64_u8, tab)),
            context("in column: mapping quality", uint8_u8),
            context("in column: optional sam fields", preceded(opt(tab), sam_fields_u8)),
            opt(newline)
        )),
        |tup| {
            let query = Locus::new(tup.0, tup.1, tup.2, tup.3);
            let target = Locus::new(tup.5, tup.6, tup.7, tup.8);
            PAF::new(query, tup.4, target, tup.9, tup.10, tup.11, tup.12)
        }
    )(line)
}
//all_consuming(




#[cfg(test)]
mod tests {
    use super::*;

    type TupStrErr = (&'static str, ErrorKind);
    type TupU8Err = (&'static [u8], ErrorKind);

    #[test]
    fn test_tab() {
        assert_eq!(
            tab::<&str, TupStrErr>("\t"),
            Ok(("", '\t'))
        );

        assert_eq!(
            tab::<&str, TupStrErr>("\tremaining"),
            Ok(("remaining", '\t'))
        );

        assert_eq!(
            tab::<&str, TupStrErr>("Should fail"),
            Err(nom::Err::Error(("Should fail", ErrorKind::Char)))
        );

        assert_eq!(
            tab::<&str, TupStrErr>("\\tShould fail"),
            Err(nom::Err::Error(("\\tShould fail", ErrorKind::Char)))
        );

        assert_eq!(
            tab::<&[u8], TupU8Err>(&b"\t"[..]),
            Ok((&b""[..], '\t'))
        );

        assert_eq!(
            tab::<&[u8], TupU8Err>(&b"\tremaining"[..]),
            Ok((&b"remaining"[..], '\t'))
        );

        assert_eq!(
            tab::<&[u8], TupU8Err>(&b"Should fail"[..]),
            Err(nom::Err::Error((&b"Should fail"[..], ErrorKind::Char)))
        );

        assert_eq!(
            tab::<&[u8], TupU8Err>(&b"\\tShould fail"[..]),
            Err(nom::Err::Error((&b"\\tShould fail"[..], ErrorKind::Char)))
        );
    }

    #[test]
    fn test_strand() {
        assert_eq!(
            strand::<&str, TupStrErr>("+\tone"),
            Ok(("\tone", Strand::Plus))
        );

        assert_eq!(
            strand::<&str, TupStrErr>("-\tone"),
            Ok(("\tone", Strand::Minus))
        );

        assert_eq!(
            strand::<&str, TupStrErr>("?\tone"),
            Err(nom::Err::Error(("?\tone", ErrorKind::OneOf)))
        );

        assert_eq!(
            strand::<&[u8], TupU8Err>(&b"+\tone"[..]),
            Ok((&b"\tone"[..], Strand::Plus))
        );

        assert_eq!(
            strand::<&[u8], TupU8Err>(&b"-\tone"[..]),
            Ok((&b"\tone"[..], Strand::Minus))
        );

        assert_eq!(
            strand::<&[u8], TupU8Err>(&b"?\tone"[..]),
            Err(nom::Err::Error((&b"?\tone"[..], ErrorKind::OneOf)))
        );
    }

    #[test]
    fn test_string_str() {
        assert_eq!(
            string_str::<TupStrErr>("Hello\tone"),
            Ok(("\tone", "Hello".to_string()))
        );

        assert_eq!(
            string_str::<TupStrErr>("Hello"),
            Ok(("", "Hello".to_string()))
        );

        assert_eq!(
            string_str::<TupStrErr>("\tShould fail"),
            Err(nom::Err::Error(("\tShould fail", ErrorKind::IsNot)))
        );
    }

    #[test]
    fn test_string_u8() {
        assert_eq!(
            string_u8::<TupU8Err>(&b"Hello\tone"[..]),
            Ok((&b"\tone"[..], "Hello".to_string()))
        );

        assert_eq!(
            string_u8::<TupU8Err>(&b"Hello"[..]),
            Ok((&b""[..], "Hello".to_string()))
        );

        assert_eq!(
            string_u8::<TupU8Err>(&b"\tShould fail"[..]),
            Err(nom::Err::Error((&b"\tShould fail"[..], ErrorKind::IsNot)))
        );
    }

    #[test]
    fn test_uint64_str() {
        assert_eq!(
            uint64_str::<TupStrErr>("123\tone"),
            Ok(("\tone", 123u64))
        );

        assert_eq!(
            uint64_str::<TupStrErr>("one\t123"),
            Err(nom::Err::Error(("one\t123", ErrorKind::Digit)))
        );
    }

    #[test]
    fn test_uint64_u8() {
        assert_eq!(
            uint64_u8::<TupU8Err>(&b"123\tone"[..]),
            Ok((&b"\tone"[..], 123u64))
        );

        assert_eq!(
            uint64_u8::<TupU8Err>(&b"one\t123"[..]),
            Err(nom::Err::Error((&b"one\t123"[..], ErrorKind::Digit)))
        );
    }

    #[test]
    fn test_uint8_str() {
        assert_eq!(
            uint8_str::<TupStrErr>("123\tone"),
            Ok(("\tone", 123u8))
        );

        assert_eq!(
            uint8_str::<TupStrErr>("one\t123"),
            Err(nom::Err::Error(("one\t123", ErrorKind::Digit)))
        );

        // Integer overflow. Maximum u8 size is 255.
        assert_eq!(
            uint8_str::<TupStrErr>("123456\tone"),
            Err(nom::Err::Error(("123456\tone", ErrorKind::MapRes)))
        );
    }

    #[test]
    fn test_uint8_u8() {
        assert_eq!(
            uint8_u8::<TupU8Err>(&b"123\tone"[..]),
            Ok((&b"\tone"[..], 123u8))
        );

        // Integer overfloww. Maximum u8 size is 255.
        assert_eq!(
            uint8_u8::<TupU8Err>(&b"123456\tone"[..]),
            Err(nom::Err::Error((&b"123456\tone"[..], ErrorKind::MapRes)))
        );

        assert_eq!(
            uint8_u8::<TupU8Err>(&b"one\t123"[..]),
            Err(nom::Err::Error((&b"one\t123"[..], ErrorKind::Digit)))
        );
    }

    #[test]
    fn test_sam_fields_str() {
        assert_eq!(
            sam_fields_str::<TupStrErr>("Hey\tDarcy"),
            Ok(("", vec!["Hey".to_string(), "Darcy".to_string()]))
        );

        assert_eq!(
            sam_fields_str::<TupStrErr>("Hey\tDarcy\t"),
            Ok(("\t", vec!["Hey".to_string(), "Darcy".to_string()]))
        );

        assert_eq!(
            sam_fields_str::<TupStrErr>(&""),
            Ok(("", vec![]))
        );
    }

    #[test]
    fn test_sam_fields_u8() {
        assert_eq!(
            sam_fields_u8::<TupU8Err>(&b"Hey\tDarcy"[..]),
            Ok((&b""[..], vec!["Hey".to_string(), "Darcy".to_string()]))
        );

        assert_eq!(
            sam_fields_u8::<TupU8Err>(&b"Hey\tDarcy\t"[..]),
            Ok((&b"\t"[..], vec!["Hey".to_string(), "Darcy".to_string()]))
        );

        assert_eq!(
            sam_fields_u8::<TupU8Err>(&b""[..]),
            Ok((&b""[..], vec![]))
        );
    }

    #[test]
    fn test_locus_str() {
        assert_eq!(
            locus_str::<TupStrErr>("seqid\t10\t0\t9"),
            Ok(("", Locus::new("seqid".to_string(), 10, 0, 9)))
        );

        assert_eq!(
            locus_str::<TupStrErr>("one two\t10\t0\t9\tthree"),
            Ok(("\tthree", Locus::new("one two".to_string(), 10, 0, 9)))
        );

        assert_eq!(
            locus_str::<TupStrErr>("seqid\t10\t0"),
            Err(nom::Err::Error(("", ErrorKind::Char)))
        );

        assert_eq!(
            locus_str::<TupStrErr>("seqid\t10\t0\t"),
            Err(nom::Err::Error(("", ErrorKind::Digit)))
        );

        assert_eq!(
            locus_str::<TupStrErr>("seqid\t10\t0\tthree"),
            Err(nom::Err::Error(("three", ErrorKind::Digit)))
        );
    }

    #[test]
    fn test_locus_u8() {
        assert_eq!(
            locus_u8::<TupU8Err>(&b"seqid\t10\t0\t9"[..]),
            Ok((&b""[..], Locus::new("seqid".to_string(), 10, 0, 9)))
        );

        assert_eq!(
            locus_u8::<TupU8Err>(&b"one two\t10\t0\t9\tthree"[..]),
            Ok((&b"\tthree"[..], Locus::new("one two".to_string(), 10, 0, 9)))
        );

        assert_eq!(
            locus_u8::<TupU8Err>(&b"seqid\t10\t0"[..]),
            Err(nom::Err::Error((&b""[..], ErrorKind::Char)))
        );

        assert_eq!(
            locus_u8::<TupU8Err>(&b"seqid\t10\t0\t"[..]),
            Err(nom::Err::Error((&b""[..], ErrorKind::Digit)))
        );

        assert_eq!(
            locus_u8::<TupU8Err>(&b"seqid\t10\t0\tthree"[..]),
            Err(nom::Err::Error((&b"three"[..], ErrorKind::Digit)))
        );
    }

    #[test]
    fn test_paf_str() {
        let l1 = Locus::new("seqid".to_string(), 10, 0, 10);
        let l2 = Locus::new("seqid2".to_string(), 10, 0, 10);

        assert_eq!(
            paf_str::<TupStrErr>("seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1\t1"),
            Ok(("", PAF::new(l1.clone(), Strand::Plus, l2.clone(), 1, 1, 1, Vec::new())))
        );

        assert_eq!(
            paf_str::<TupStrErr>("seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1\t1\t"),
            Ok(("", PAF::new(l1.clone(), Strand::Plus, l2.clone(), 1, 1, 1, Vec::new())))
        );

        assert_eq!(
            paf_str::<TupStrErr>("seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1\t1\tone\ttwo\tthree"),
            Ok(("", PAF::new(l1.clone(), Strand::Plus, l2.clone(), 1, 1, 1, vec!["one".to_string(), "two".to_string(), "three".to_string()])))
        );

        assert_eq!(
            paf_str::<TupStrErr>("seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1"),
            Err(nom::Err::Error(("", ErrorKind::Char)))
        );
    }

    #[test]
    fn test_paf_u8() {
        let l1 = Locus::new("seqid".to_string(), 10, 0, 10);
        let l2 = Locus::new("seqid2".to_string(), 10, 0, 10);

        assert_eq!(
            paf_u8::<TupU8Err>(&b"seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1\t1"[..]),
            Ok((&b""[..], PAF::new(l1.clone(), Strand::Plus, l2.clone(), 1, 1, 1, Vec::new())))
        );

        assert_eq!(
            paf_u8::<TupU8Err>(&b"seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1\t1\t"[..]),
            Ok((&b""[..], PAF::new(l1.clone(), Strand::Plus, l2.clone(), 1, 1, 1, Vec::new())))
        );

        assert_eq!(
            paf_u8::<TupU8Err>(&b"seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1\t1\tone\ttwo\tthree"[..]),
            Ok((&b""[..], PAF::new(l1.clone(), Strand::Plus, l2.clone(), 1, 1, 1, vec!["one".to_string(), "two".to_string(), "three".to_string()])))
        );

        assert_eq!(
            paf_u8::<TupU8Err>(&b"seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1"[..]),
            Err(nom::Err::Error((&b""[..], ErrorKind::Char)))
        );
    }
}
