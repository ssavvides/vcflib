use std::fs::File;
use vcflib::parser::{VCFParser, VCFWriter};

fn main() {
    let file = File::open("test/resources/valid/small-4.2.vcf").unwrap();

    // reader
    let parser = VCFParser::new(file).unwrap();

    let header = parser.header;
    let reader = parser.reader;

    // writer
    let mut buf = vec![];
    let mut writer = VCFWriter::new(&mut buf, &header).unwrap();
    for dl in reader {
        let data_line = &mut dl.unwrap();
        writer.write_data_line(data_line).unwrap();
    }
    println!("{}", std::str::from_utf8(&buf).unwrap());
}
