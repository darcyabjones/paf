use std::iter::repeat;

use snafu::Snafu;
use nom::error::{VerboseError, VerboseErrorKind};
use nom::combinator::{cut, all_consuming};
use nom::Offset;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(
        "Error while parsing character. Expected any of '{}' but got {}.",
        expected,
        got
    ))]
    ParseChar { got: char, expected: String },
    #[snafu(display(
        "Error while parsing line:\n{}\n{}",
        line.replace("\t", "    "),  // replaces tabs with spaces.
        join_parse_details(line, column, details)
    ))]
    ParseLine { line: String, column: usize, details: Vec<String> },
    #[snafu(display(
        "Error while parsing line {}:\n{}\n{}",
        line_num,
        line.replace("\t", "    "),  // replaces tabs with spaces.
        join_parse_details(line, column, details)
    ))]
    Parse { line_num: usize, line: String, column: usize, details: Vec<String> },
    #[snafu(display(
        "Error while parsing line {}: expected paf line but got empty input.",
        line_num
    ))]
    EmptyLine { line_num: usize },
    #[snafu(display("Error while parsing line: expected paf line but got empty input."))]
    Empty,
}


/// Join the strings and format the indicator for display.
fn join_parse_details(line: &str, column: &usize, details: &[String]) -> String {
    // Because tabs can be displayed at variable sizes, we need to convert
    // them to spaces and update the offset.
    let count = line.chars().take(*column).filter(|c| *c == '\t').count();
    let new_column = (column - count) + (count * 4);

    let mut output = String::new();

    // This adds spaces before the indicator so that it aligns with the
    // troublesome line.
    output += &repeat(' ').take(new_column).collect::<String>();
    output += "^ ";

    // This just joins the strings.
    for (i, detail) in details.iter().enumerate() {
        if i > 0 { output += " " };
        output += &detail.to_string();
    }

    output
}


/// Return an error if the input is empty.
fn check_empty_input(
    line_num: &Option<usize>,
    lines: &[String]
) -> Result<(), crate::errors::Error> {
    if lines.is_empty() {
        let empty_err = match line_num {
            Some(l) => crate::errors::Error::EmptyLine { line_num: *l },
            None => crate::errors::Error::Empty,
        };
        Err(empty_err)
    } else {
        Ok(())
    }
}


/// Find the line number and the column offset.
/// This handles case that input is multiline.
fn find_offset(initial: &usize, lines: &Vec<String>) -> (usize, usize) {
    let mut line_no = 0;
    let mut offset = *initial;

    for (j, l) in lines.iter().enumerate() {
        if offset <= l.len() {
            line_no = j;
            break;
        } else {
            offset = offset - l.len() - 1;
        }
    }

    (line_no, offset)
}


/// Map error-kinds to strings and push them onto a vector.
/// Mutates input vector.
fn push_errorkind(details: &mut Vec<String>, kind: &VerboseErrorKind) {
    match kind {
        VerboseErrorKind::Char(c) => {
            details.push(format!("expected character '{}'", c.to_string()));
        }
        VerboseErrorKind::Context(s) => {
            details.push(s.to_string());
        },
        VerboseErrorKind::Nom(_) => {} // details.push(format!("{:?}", e));
    }
}


/// transforms a nom `VerboseError` into a trace with input position information
pub fn convert_error_str(
    input: &str,
    error: VerboseError<&str>,
    line_num_offset: Option<usize>,
) -> crate::errors::Error {

    let mut details: Vec<String> = Vec::new();

    let lines: Vec<String> = input.lines().map(String::from).collect();

    if let Err(e) = check_empty_input(&line_num_offset, &lines) {
        return e;
    }

    // Get the first error. This is the main one that we're interested in.
    let mut iterator = error.errors.iter();
    let (line_num, column, kind) = iterator
        .next()
        .map_or_else(
            || (0, 0, &VerboseErrorKind::Context("Somehow we didn't get an error.")),
            |(substring, kind)| {
                let (l, c) = find_offset(&input.offset(substring), &lines);
                (l, c, kind)
            },
        );

    push_errorkind(&mut details, kind);

    // Get each error kind as a string and add to details.
    for (_, kind) in iterator {
        push_errorkind(&mut details, kind);
    }

    // Raise a parse error using line num if input is Some.
    match line_num_offset {
        Some(l) => {
            super::errors::Error::Parse {
                line_num: line_num + l,
                line: lines[line_num].clone(),
                column: column,
                details: details
            }
        },
        None => {
            super::errors::Error::ParseLine {
                line: lines[line_num].clone(),
                column: column,
                details: details
            }
        },
    }
}
