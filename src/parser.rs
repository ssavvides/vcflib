use crate::{
    body::DataLine,
    header::{parse_column_names, parse_version, Header, HeaderLine},
};
use std::{
    io,
    io::{BufRead, BufReader, Read, Write},
};

#[derive(Debug)]
pub struct VCFParser<R: BufRead> {
    pub header: Header,
    pub reader: VCFReader<R>,
}

#[derive(Debug)]
pub struct VCFReader<R: BufRead> {
    pub column_names: Vec<String>,
    pub reader: R,
}

pub const FIXED_COLUMNS: &[&str] = &["CHROM", "POS", "ID", "REF", "ALT", "QUAL", "FILTER", "INFO"];

impl<R: Read> VCFParser<BufReader<R>> {
    pub fn new(read: R) -> anyhow::Result<Self> {
        let mut reader = BufReader::new(read);

        let mut line = String::new();
        let mut version = "".to_string();
        let mut header_lines = vec![];
        let mut column_names = vec![];
        loop {
            line.clear();
            let read_bytes = reader.read_line(&mut line)?;
            if read_bytes == 0 {
                break;
            }

            // remove newline
            if line.ends_with('\n') {
                line.pop();
            }

            if line.starts_with("##fileformat=") {
                version = parse_version(&line)?;
            } else if line.starts_with("##") {
                header_lines.push(line.parse::<HeaderLine>()?);
            } else if line.starts_with('#') {
                column_names = parse_column_names(&line)?;
                break;
            } else {
                return Err(anyhow::anyhow!(
                    "Invalid line while parsing header: `{}`",
                    line
                ));
            }
        }

        Ok(VCFParser {
            reader: VCFReader {
                column_names: column_names.clone(),
                reader,
            },
            header: Header::new(version, header_lines, column_names),
        })
    }
}

impl<R: BufRead> VCFReader<R> {
    pub fn next_item(&mut self) -> Option<anyhow::Result<DataLine>> {
        let mut line = String::new();
        let result = self.reader.read_line(&mut line);
        match result {
            Ok(read_bytes) => {
                if read_bytes == 0 {
                    None
                } else {
                    // remove newline
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    Some(DataLine::new(&line, &self.column_names))
                }
            }
            Err(e) => Some(Err(anyhow::anyhow!("Could not read: `{:?}`", e))),
        }
    }

    pub fn iter(&mut self) -> Iter<'_, R> {
        Iter { vcf_reader: self }
    }
}

impl<R: BufRead> IntoIterator for VCFReader<R> {
    type Item = anyhow::Result<DataLine>;
    type IntoIter = IntoIter<R>;

    fn into_iter(self) -> IntoIter<R> {
        IntoIter { vcf_reader: self }
    }
}

impl<'a, R: BufRead> IntoIterator for &'a mut VCFReader<R> {
    type Item = anyhow::Result<DataLine>;
    type IntoIter = Iter<'a, R>;

    fn into_iter(self) -> Iter<'a, R> {
        Iter { vcf_reader: self }
    }
}

#[derive(Debug)]
pub struct Iter<'a, R: BufRead> {
    vcf_reader: &'a mut VCFReader<R>,
}

impl<'a, R: BufRead> Iterator for Iter<'a, R> {
    type Item = anyhow::Result<DataLine>;

    fn next(&mut self) -> Option<Self::Item> {
        self.vcf_reader.next_item()
    }
}

#[derive(Debug)]
pub struct IntoIter<R: BufRead> {
    vcf_reader: VCFReader<R>,
}

impl<R: BufRead> Iterator for IntoIter<R> {
    type Item = anyhow::Result<DataLine>;
    fn next(&mut self) -> Option<Self::Item> {
        self.vcf_reader.next_item()
    }
}

pub struct VCFWriter<W: Write> {
    writer: W,
}

impl<W: Write> VCFWriter<W> {
    pub fn new(mut writer: W, header: &Header) -> anyhow::Result<VCFWriter<W>> {
        // write version
        writeln!(writer, "{}", header.version)?;

        // write header lines
        for hl in &header.header_lines {
            writeln!(writer, "{}", hl)?;
        }

        // write fixed columns
        for (index, column) in FIXED_COLUMNS.iter().enumerate() {
            if index == 0 {
                write!(writer, "#{}", column)?;
            } else {
                write!(writer, "\t{}", column)?;
            }
        }
        // ... and custom columns
        if !header.column_names.is_empty() {
            write!(writer, "\tFORMAT")?;
            for cn in &header.column_names {
                write!(writer, "\t{}", cn)?;
            }
        }

        Ok(VCFWriter { writer })
    }

    pub fn write_data_line(&mut self, dl: &DataLine) -> io::Result<()> {
        write!(self.writer, "\n{}", dl)
    }
}
