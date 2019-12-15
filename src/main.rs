use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let input = "seqid\t10\t0\t10\t+\tseqid2\t10\t0\t10\t1\t1\t1000\n";
    match paf::PAF::from_str(input) {
        Ok(p) => println!("{}", p),
        Err(e) => println!("{}", e),
    };

    Ok(())
}
