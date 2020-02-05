use crate::parser::FIXED_COLUMNS;
use std::{
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

/// A data line of the VCF file.
#[derive(Debug, PartialEq)]
pub struct DataLine {
    /// An identifier from the reference genome or an angle-bracketed ID String (“<ID>”)
    /// pointing to a contig in the assembly file (cf. the ##assembly line in the header).
    pub chromosome: String,

    /// The reference position.
    pub position: u64,

    /// Semi-colon separated list of unique identifiers where available.
    pub id: IdType,

    /// Each base must be one of A,C,G,T,N (case insensitive). Multiple bases are permitted.
    pub reference: String,

    /// Comma separated list of alternate non-reference alleles.
    pub alternative: AltType,

    ///
    pub quality: QualType,

    ///
    pub filter: FilterType,

    ///
    pub info: InfoType,

    ///
    pub format: Option<FormatType>,

    ///
    pub samples: Vec<SampleType>,
}

#[derive(Debug, PartialEq)]
pub enum IdType {
    Missing,
    Entries(Vec<String>),
}

impl FromStr for IdType {
    type Err = anyhow::Error;

    fn from_str(id_str: &str) -> anyhow::Result<Self> {
        if id_str.is_empty() {
            return Err(anyhow::anyhow!("id cannot be empty"));
        }
        let id = if id_str == "." {
            IdType::Missing
        } else {
            IdType::Entries(id_str.split(';').map(|s| s.to_string()).collect())
        };
        Ok(id)
    }
}

impl Display for IdType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            IdType::Missing => write!(f, "."),
            IdType::Entries(s) => write!(f, "{}", s.join(";")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AltType {
    Missing,
    Entries(Vec<String>),
}

impl FromStr for AltType {
    type Err = anyhow::Error;

    fn from_str(alt_str: &str) -> anyhow::Result<Self> {
        if alt_str.is_empty() {
            return Err(anyhow::anyhow!("alt cannot be empty"));
        }
        let alt = if alt_str == "." {
            AltType::Missing
        } else {
            AltType::Entries(alt_str.split(',').map(|s| s.to_string()).collect())
        };
        Ok(alt)
    }
}

impl Display for AltType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            AltType::Missing => write!(f, "."),
            AltType::Entries(s) => write!(f, "{}", s.join(",")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum QualType {
    Missing,
    Integer(u32),
}

impl FromStr for QualType {
    type Err = anyhow::Error;

    fn from_str(qual_str: &str) -> anyhow::Result<Self> {
        if qual_str.is_empty() {
            return Err(anyhow::anyhow!("qual cannot be empty"));
        }
        let qual = if qual_str == "." {
            QualType::Missing
        } else {
            QualType::Integer(qual_str.parse::<u32>()?)
        };
        Ok(qual)
    }
}

impl Display for QualType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            QualType::Missing => write!(f, "."),
            QualType::Integer(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FilterType {
    Missing,
    Pass,
    Entries(Vec<String>),
}

impl FromStr for FilterType {
    type Err = anyhow::Error;

    fn from_str(filter_str: &str) -> anyhow::Result<Self> {
        if filter_str.is_empty() {
            return Err(anyhow::anyhow!("filter cannot be empty"));
        }
        let filter = if filter_str == "." {
            FilterType::Missing
        } else if filter_str == "PASS" {
            FilterType::Pass
        } else {
            FilterType::Entries(filter_str.split(';').map(|s| s.to_string()).collect())
        };
        Ok(filter)
    }
}

impl Display for FilterType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            FilterType::Missing => write!(f, "."),
            FilterType::Pass => write!(f, "PASS"),
            FilterType::Entries(s) => write!(f, "{}", s.join(";")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InfoType {
    Missing,
    Entries(Vec<String>),
}

impl FromStr for InfoType {
    type Err = anyhow::Error;

    fn from_str(info_str: &str) -> anyhow::Result<Self> {
        if info_str.is_empty() {
            return Err(anyhow::anyhow!("info cannot be empty"));
        }
        let info = if info_str == "." {
            InfoType::Missing
        } else {
            InfoType::Entries(info_str.split(';').map(|s| s.to_string()).collect())
        };
        Ok(info)
    }
}

impl Display for InfoType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            InfoType::Missing => write!(f, "."),
            InfoType::Entries(s) => write!(f, "{}", s.join(";")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FormatType {
    Missing,
    Entries(Vec<String>),
}

impl FromStr for FormatType {
    type Err = anyhow::Error;

    fn from_str(format_str: &str) -> anyhow::Result<Self> {
        if format_str.is_empty() {
            return Err(anyhow::anyhow!("format cannot be empty"));
        }
        let format = if format_str == "." {
            FormatType::Missing
        } else {
            FormatType::Entries(format_str.split(':').map(|s| s.to_string()).collect())
        };
        Ok(format)
    }
}

impl Display for FormatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            FormatType::Missing => write!(f, "."),
            FormatType::Entries(s) => write!(f, "{}", s.join(":")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SampleType {
    Missing,
    Entries(Vec<String>),
}

impl SampleType {
    fn new(sample_vec: Vec<&str>) -> anyhow::Result<Vec<Self>> {
        let mut result = vec![];
        for sample_str in sample_vec {
            if sample_str.is_empty() {
                return Err(anyhow::anyhow!("sample cannot be empty"));
            }
            let sample = if sample_str == "." {
                SampleType::Missing
            } else {
                SampleType::Entries(sample_str.split(':').map(|s| s.to_string()).collect())
            };
            result.push(sample);
        }
        Ok(result)
    }
}

impl Display for SampleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            SampleType::Missing => write!(f, "."),
            SampleType::Entries(s) => write!(f, "{}", s.join(":")),
        }
    }
}

impl DataLine {
    pub fn new(line_str: &str, column_names: &[String]) -> anyhow::Result<DataLine> {
        let parts: Vec<&str> = line_str.split('\t').collect();

        let expected_len = if !column_names.is_empty() {
            // + 1 is for "FORMAT" column
            FIXED_COLUMNS.len() + column_names.len() + 1
        } else {
            FIXED_COLUMNS.len()
        };
        if parts.len() != expected_len {
            return Err(anyhow::anyhow!(
                "invalid number of columns found, expected {}, found {}",
                expected_len,
                parts.len()
            ));
        }

        let format: Option<FormatType> = if expected_len > 8 {
            Some(parts[8].parse()?)
        } else {
            None
        };

        let samples: Vec<SampleType> = if expected_len > 9 {
            SampleType::new((&parts[9..]).to_vec())?
        } else {
            vec![]
        };

        Ok(DataLine {
            chromosome: parts[0].parse()?,
            position: parts[1].parse()?,
            id: parts[2].parse()?,
            reference: parts[3].parse()?,
            alternative: parts[4].parse()?,
            quality: parts[5].parse()?,
            filter: parts[6].parse()?,
            info: parts[7].parse()?,
            format,
            samples,
        })
    }

    pub fn format_index(&self, entry: &str) -> Option<usize> {
        if let Some(format) = &self.format {
            match format {
                FormatType::Entries(entries) => entries.iter().position(|e| e == entry),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Display for DataLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.chromosome)?;
        write!(f, "\t{}", self.position)?;
        write!(f, "\t{}", self.id)?;
        write!(f, "\t{}", self.reference)?;
        write!(f, "\t{}", self.alternative)?;
        write!(f, "\t{}", self.quality)?;
        write!(f, "\t{}", self.filter)?;
        write!(f, "\t{}", self.info)?;

        if let Some(form) = &self.format {
            write!(f, "\t{}", form)?;
            for sample in self.samples.iter() {
                write!(f, "\t{}", sample)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::body::DataLine;

    #[test]
    fn test_valid() {
        // single sample
        let column_names = vec!["Sample01".to_string()];
        let line_str =
            "1	10177	.	A	AC	.	LOWCONF	.	GT:RC:AC:GP:DS	0/1:0:0:0.199096,0.522516,0.278389:1.07929";
        let actual_line = DataLine::new(line_str, &column_names);
        assert!(actual_line.is_ok());

        // two samples
        let column_names = vec!["Sample01".to_string(), "Sample02".to_string()];
        let line_str =
            "1	10177	.	A	AC	.	LOWCONF	.	GT:RC:AC:GP:DS	0/1:0:0:0.199096,0.522516,0.278389:1.07929	0/0:0:0:1,1e-10,1e-10:3e-10";
        let actual_line = DataLine::new(line_str, &column_names);
        println!("{:?}", actual_line);
        assert!(actual_line.is_ok());
    }

    #[test]
    fn test_invalid() {
        // missing column
        let column_names = vec!["Sample01".to_string()];
        let line_str = "1	10177	.	A	AC	.	LOWCONF	.	GT:RC:AC:GP:DS";
        let actual_line = DataLine::new(line_str, &column_names);
        println!("{:?}", actual_line);
        assert!(actual_line.is_err());
    }
}
