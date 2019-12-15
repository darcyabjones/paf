pub(crate) mod paf;
pub(crate) mod errors;
pub(crate) mod complete;

pub use crate::paf::PAF;
pub use crate::paf::Locus;

//use std::io::BufRead;
//use std::io::Read;
//
//
//#[derive(Default)]
//pub struct ParseError;
//
//impl std::fmt::Display for ParseError {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "A parsing error occurred.")
//    }
//}
//
//
//impl std::fmt::Debug for ParseError {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        <ParseError as std::fmt::Display>::fmt(self, f)
//    }
//}
//
//impl std::error::Error for ParseError { }
//
//
//#[derive(Clone, Default, Debug)]
//pub struct Mount {
//    pub device: String,
//    pub mount_point: String,
//    pub file_system_type: String,
//    pub options: Vec<String>,
//}
//
//
//pub fn mounts() -> Result<(), Box<dyn std::error::Error>> {
//    let file = std::fs::File::open("/proc/mounts")?;
//    let buf_reader = std::io::BufReader::new(file);
//
//    for line in buf_reader.lines() {
//        match parsers::parse_line(&line?[..]) {
//            Ok( (_, m) ) => println!("{:?}", m),
//            Err(_) => return Err(ParseError::default().into())
//        }
//    }
//
//    Ok(())
//}
