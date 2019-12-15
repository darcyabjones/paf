use std::fmt;
use std::str::FromStr;

use std::convert::TryFrom;

use nom::combinator::{cut, all_consuming};
use crate::errors::Error;

/// Represent the strand of the alignment between two loci.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Strand {
    Plus,
    Minus,
}


impl fmt::Display for Strand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let character: char = self.into();
        write!(f, "{}", character)
    }
}


impl Into<char> for &Strand {
    fn into(self) -> char {
        match self {
            Strand::Plus => '+',
            Strand::Minus => '-'
        }
    }
}


impl Into<u8> for &Strand {
    fn into(self) -> u8 {
        match self {
            Strand::Plus => b'+',
            Strand::Minus => b'-'
        }
    }
}


impl TryFrom<char> for Strand {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Strand::Plus),
            '-' => Ok(Strand::Minus),
            c   => Err(Error::ParseChar { got: c, expected: "+-".to_string() })
        }
    }
}


impl TryFrom<u8> for Strand {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'+' => Ok(Strand::Plus),
            b'-' => Ok(Strand::Minus),
            c    => Err(Error::ParseChar { got: c as char, expected: "+-".to_string() })
        }
    }
}


/// Represent the aligned region from one of the sequences.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct Locus {
    name: String,
    length: u64,
    start: u64,
    end: u64
}


impl Locus {
    /// Construct a new `Locus` object.
    pub fn new(name: String, length: u64, start: u64, end: u64) -> Self {
        Locus { name, length, start, end }
    }
}


impl fmt::Display for Locus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t{}\t{}\t{}", self.name, self.length, self.start, self.end)
    }
}


impl FromStr for Locus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (rem, locus) = all_consuming(cut(crate::complete::locus_str))(s)
            .map_err(|e| {
                match e {
                    nom::Err::Error(ei) | nom::Err::Failure(ei) => {
                        crate::errors::convert_error_str(s, ei, None)
                    },
                    _ => unreachable!(),
                }
            })?;

        Ok(locus)
    }
}


//#[derive(Debug, Eq, PartialEq, Clone)]
//pub enum Fields {
//    Type,
//    NumMinimizers,
//    ChainScore,
//    ChainScoreSecondary,
//    NumMismatches,
//    GenRefSeq,
//    AlignScore,
//    AlignScoreBestSegment,
//    NumAmbiguous,
//    TranscriptStrand,
//    Cigar,
//    Difference,
//    SeqDivergence,
//    CompressedSeqDivergence,
//    LenRepetitiveSeeds,
//}


#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PAF {
    query: Locus,
    strand: Strand,
    target: Locus,
    nmatch: u64,
    alnlen: u64,
    mq: u8,
    fields: Vec<String>,
}


impl PAF {
    pub fn new(
        query: Locus,
        strand: Strand,
        target: Locus,
        nmatch: u64,
        alnlen: u64,
        mq: u8,
        fields: Vec<String>
    ) -> Self {
        PAF {
            query,
            strand,
            target,
            nmatch,
            alnlen,
            mq,
            fields,
        }
    }
}


impl fmt::Display for PAF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.fields.len() > 1 {
            write!(
                f,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                self.query,
                self.strand,
                self.target,
                self.nmatch,
                self.alnlen,
                self.mq,
                self.fields.join("\t")
            )
        } else {
            write!(
                f,
                "{}\t{}\t{}\t{}\t{}\t{}",
                self.query,
                self.strand,
                self.target,
                self.nmatch,
                self.alnlen,
                self.mq
            )
        }
    }
}


impl FromStr for PAF {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (rem, paf) = all_consuming(cut(crate::complete::paf_str))(s)
            .map_err(|e| {
                match e {
                    nom::Err::Error(ei) | nom::Err::Failure(ei) => {
                        crate::errors::convert_error_str(s, ei, None)
                    },
                    _ => unreachable!(),
                }
            })?;

        Ok(paf)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //fn test_strand_parse_str() {
    //    assert_eq!("+".parse(), Ok(Strand::Plus));
    //    assert_eq!("-".parse(), Ok(Strand::Minus));
    //    assert_eq!(
    //        "?".parse::<Strand>(),
    //        Err(Error::ParseStrand { got: "?".to_string() })
    //    );
    //}

    //#[test]
    //fn test_strand_try_from_char() {
    //    assert_eq!('+'.try_into(), Ok(Strand::Plus));
    //    assert_eq!('-'.try_into(), Ok(Strand::Minus));
    //    assert_eq!(
    //        Strand::try_from('?'),
    //        Err(Error::ParseStrand { got: "?".to_string() })
    //    );
    //}
}

