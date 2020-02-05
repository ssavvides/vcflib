use linked_hash_map::LinkedHashMap;
use std::{
    collections::HashSet,
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

pub const OTHER_KEY: &str = "Value";

/// The header of the VCF file
#[derive(Debug)]
pub struct Header {
    /// The VCF version. Captures the string that comes right after "##fileformat=".
    pub version: Version,

    /// Captures all other lines of the header.
    pub header_lines: Vec<HeaderLine>,

    /// The additional column names not containing the expected fixed columns.
    pub column_names: Vec<String>,
}

impl Header {
    pub fn new(version: String, header_lines: Vec<HeaderLine>, column_names: Vec<String>) -> Self {
        Self {
            version: Version { value: version },
            header_lines,
            column_names,
        }
    }
}

#[derive(Debug)]
pub struct Version {
    pub value: String,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "##fileformat={}", self.value)
    }
}

#[derive(Debug, PartialEq)]
pub enum HeaderLine {
    /// Example:
    /// ##ALT=<ID=type,Description=description>
    Alt { id: Vec<AltId>, description: String },

    /// Example:
    /// ##assembly=ftp://ftp-trace.ncbi.nih.gov/1000genomes
    Assembly(String),

    /// Example:
    /// ##contig=<ID=ctg1,length=81195210,species="Homo sapiens",URL=ftp://somewhere.org/assembly.fa,>
    Contig {
        id: String,
        species: Option<String>,
        other: LinkedHashMap<String, String>,
    },

    /// Example:
    /// ##fileDate=20100501
    FileDate(String),

    /// Example:
    /// ##FILTER=<ID=ID,Description="description">
    Filter { id: String, description: String },

    /// Example:
    /// ##FORMAT=<ID=ID,Number=number,Type=type,Description="description">
    Format {
        id: String,
        number: Number,
        typ: FormatType,
        description: String,
    },

    /// Example:
    /// ##INFO=<ID=ID,Number=number,Type=type,Description="description",Source="source",Version="version">
    Info {
        id: String,
        number: Number,
        typ: InfoType,
        description: String,
        source: Option<String>,
        version: Option<String>,
    },

    /// Example:
    /// ##META=<ID=Assay,Type=String,Number=.,Values=[WholeGenome, Exome]>
    Meta {
        id: String,
        // possible values for type are not defined in specification v4.3, (CF Sec 1.4.8)
        typ: String,
        // possible values for number are not defined in specification v4.3, (CF Sec 1.4.8)
        number: Number,
        values: Vec<String>,
    },

    /// Example:
    /// ##PEDIGREE=<ID=TumourSample,Original=GermlineID>
    /// ##PEDIGREE=<ID=ChildID,Father=FatherID,Mother=MotherID>
    /// ##PEDIGREE=<ID=SampleID,Name_1=Ancestor_1,...,Name_N=Ancestor_N>
    Pedigree { id: String, relation: PedigreeType },

    /// Example:
    /// ##pedigreeDB=URL
    PedigreeDB(String),

    /// Example:
    /// ##reference=1000GenomesPilot-NCBI36
    Other { key: String, value: String },

    /// Example:
    /// ##SAMPLE=<ID=Sample1,Assay=WholeGenome,Ethnicity=AFR,Disease=None,Description="Patient germline genome",DOI=url>
    /// ##SAMPLE=<ID=TissueSample,Genomes=Germline;Tumor,Mixture=.3;.7,Description="Patient germline genome;Patient tumor genome">
    Sample {
        id: String,
        meta: LinkedHashMap<String, Vec<String>>,
        description: String,
        doi: Option<String>,
    },
}

impl FromStr for HeaderLine {
    type Err = anyhow::Error;

    fn from_str(header_line_str: &str) -> anyhow::Result<Self> {
        let eq_index = header_line_str.find('=');
        if eq_index.is_none() {
            return Err(anyhow::anyhow!(
                "invalid header line `{}`, (header lines must contain an `=` sign)",
                header_line_str
            ));
        }

        // example:
        // ##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">
        //   ^^^^^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //   ^type  ^payload
        let (mut header_type, mut header_payload) = header_line_str.split_at(eq_index.unwrap());

        // remove "##" from meta type
        if !header_type.starts_with("##") {
            return Err(anyhow::anyhow!(
                "invalid header type `{}`, (header lines must start with `##`)",
                header_type
            ));
        }
        header_type = &header_type[2..];

        // remove `=` sign and parse to parts
        header_payload = &header_payload[1..];
        let payload_parts = parse_header_payload(header_payload)?;

        let header_line = match header_type {
            "ALT" => HeaderLine::Alt {
                id: AltId::new_alt_ids(
                    payload_parts
                        .get("ID")
                        .ok_or_else(|| anyhow::anyhow!("value not found"))?,
                )?,
                description: get_map_value(&payload_parts, "Description")?,
            },
            "assembly" => HeaderLine::Assembly(get_map_value(&payload_parts, OTHER_KEY)?),
            "contig" => {
                let id = get_map_value(&payload_parts, "ID")?;
                let species = get_map_value(&payload_parts, "species").ok();
                let mut other: LinkedHashMap<String, String> = LinkedHashMap::new();
                for (key, value) in payload_parts {
                    if key != "ID" && key != "species" {
                        other.insert(key.to_string(), value.to_string());
                    }
                }
                HeaderLine::Contig { id, species, other }
            }
            "fileDate" => HeaderLine::FileDate(get_map_value(&payload_parts, OTHER_KEY)?),
            "FILTER" => HeaderLine::Filter {
                id: get_map_value(&payload_parts, "ID")?,
                description: get_map_value(&payload_parts, "Description")?,
            },
            "FORMAT" => HeaderLine::Format {
                id: get_map_value(&payload_parts, "ID")?,
                number: Number::new(payload_parts.get("Number").copied())?,
                typ: FormatType::new(payload_parts.get("Type").copied())?,
                description: get_map_value(&payload_parts, "Description")?,
            },
            "INFO" => HeaderLine::Info {
                id: get_map_value(&payload_parts, "ID")?,
                number: Number::new(payload_parts.get("Number").copied())?,
                typ: InfoType::new(payload_parts.get("Type").copied())?,
                description: get_map_value(&payload_parts, "Description")?,
                source: payload_parts.get("Source").map(|s| (*s).to_string()),
                version: payload_parts.get("Version").map(|s| (*s).to_string()),
            },
            "META" => HeaderLine::Meta {
                id: get_map_value(&payload_parts, "ID")?,
                typ: get_map_value(&payload_parts, "Type")?,
                number: Number::new(payload_parts.get("Number").copied())?,
                values: {
                    let value_string = payload_parts.get("Values").unwrap();
                    value_string
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                },
            },
            "PEDIGREE" => HeaderLine::Pedigree {
                id: get_map_value(&payload_parts, "ID")?,
                relation: PedigreeType::new(payload_parts)?,
            },
            "pedigreeDB" => HeaderLine::PedigreeDB(get_map_value(&payload_parts, OTHER_KEY)?),
            "SAMPLE" => {
                let id = get_map_value(&payload_parts, "ID")?;
                let description = get_map_value(&payload_parts, "Description")?;
                let doi = payload_parts.get("DOI").map(|s| (*s).to_string());
                let mut meta: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
                for (key, value) in payload_parts {
                    if key != "ID" && key != "Description" && key != "DOI" {
                        meta.insert(
                            key.to_string(),
                            value.split(';').map(|s| s.to_string()).collect(),
                        );
                    }
                }
                HeaderLine::Sample {
                    id,
                    meta,
                    description,
                    doi,
                }
            }
            _ => HeaderLine::Other {
                key: header_type.to_string(),
                value: get_map_value(&payload_parts, OTHER_KEY)?,
            },
        };

        Ok(header_line)
    }
}

impl Display for HeaderLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            HeaderLine::Alt { id, description } => {
                let mut id_str = String::new();
                for (i, v) in id.iter().enumerate() {
                    if i > 0 {
                        id_str.push_str(format!(":{}", v).as_str());
                    } else {
                        id_str.push_str(format!("{}", v).as_str());
                    }
                }
                write!(f, "##ALT=<ID={},Description=\"{}\">", id_str, description)
            }
            HeaderLine::Assembly(s) => write!(f, "##assembly={}", s),
            HeaderLine::Contig { id, species, other } => {
                let mut species_str = String::new();
                if let Some(s) = species {
                    species_str.push_str(format!(",species=\"{}\"", s).as_str())
                }
                let mut other_str = String::new();
                for (k, v) in other {
                    other_str.push_str(format!(",{}={}", k, v).as_str())
                }
                write!(f, "##contig=<ID={}{}{}>", id, species_str, other_str)?;
                Ok(())
            }
            HeaderLine::FileDate(s) => write!(f, "##fileDate={}", s),
            HeaderLine::Filter { id, description } => {
                write!(f, "##FILTER=<ID={},Description=\"{}\">", id, description)
            }
            HeaderLine::Format {
                id,
                number,
                typ,
                description,
            } => write!(
                f,
                "##FORMAT=<ID={},Number={},Type={},Description=\"{}\">",
                id, number, typ, description
            ),
            HeaderLine::Info {
                id,
                number,
                typ,
                description,
                source,
                version,
            } => {
                let mut optional_str = String::new();
                if let Some(s) = source {
                    optional_str.push_str(format!(",Source=\"{}\"", s).as_str());
                }
                if let Some(s) = version {
                    optional_str.push_str(format!(",Version=\"{}\"", s).as_str());
                }
                write!(
                    f,
                    "##INFO=<ID={},Number={},Type={},Description=\"{}\"{}>",
                    id, number, typ, description, optional_str
                )?;
                Ok(())
            }
            HeaderLine::Meta {
                id,
                typ,
                number,
                values,
            } => {
                let mut values_str = String::new();
                if values.is_empty() {
                    values_str.push_str(format!(",Values=[{}]", values.join(",")).as_str());
                }
                write!(
                    f,
                    "##META=<ID={},Type={},Number={}{}>",
                    id, typ, number, values_str
                )
            }
            HeaderLine::Pedigree { id, relation } => {
                write!(f, "##PEDIGREE=<ID={},{}>", id, relation)
            }
            HeaderLine::PedigreeDB(s) => write!(f, "##pedigreeDB={}", s),
            HeaderLine::Other { key, value } => write!(f, "##{}={}", key, value),
            HeaderLine::Sample {
                id,
                meta,
                description,
                doi,
            } => {
                let mut meta_str = String::new();
                if !meta.is_empty() {
                    for (k, v) in meta {
                        meta_str.push_str(format!(",{}={}", k, v.join(";")).as_str());
                    }
                }
                let mut doi_str = String::new();
                if let Some(s) = doi {
                    doi_str.push_str(format!(",DOI={}", s).as_str());
                }
                write!(
                    f,
                    "##SAMPLE=<ID={}{},Description=\"{}\"{}>",
                    id, meta_str, description, doi_str
                )
            }
        }
    }
}

/// A number of values that can be included within the FORMAT or INFO field.
#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    /// A non negative number.
    Integer(u32),

    /// The field has one value per alternate allele. ("A")
    Allele,

    /// The field has one value for each possible allele, including the reference. ("R")
    Reference,

    /// The field has one value for each possible genotype. ("G")
    Genotype,

    /// The number of possible values varies, is unknown or unbounded. (".")
    Unknown,
}

impl Number {
    fn new(number_str: Option<&str>) -> anyhow::Result<Self> {
        if number_str.is_none() {
            return Ok(Number::Unknown);
        }
        let number = match number_str.unwrap() {
            "A" => Number::Allele,
            "G" => Number::Genotype,
            "R" => Number::Reference,
            "." => Number::Unknown,
            s => {
                let n = s.parse::<u32>();
                if n.is_err() {
                    return Err(anyhow::anyhow!("invalid Number value `{}`", s));
                }
                let n = n.unwrap();
                Number::Integer(n)
            }
        };
        Ok(number)
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Number::Allele => write!(f, "A"),
            Number::Genotype => write!(f, "G"),
            Number::Reference => write!(f, "R"),
            Number::Unknown => write!(f, "."),
            Number::Integer(n) => write!(f, "{}", n),
        }
    }
}

/// The possible types for the key "Type" of "INFO" fields.
#[derive(Debug, PartialEq)]
pub enum InfoType {
    Character,
    Flag,
    Float,
    Integer,
    String,
}

impl InfoType {
    fn new(type_str: Option<&str>) -> anyhow::Result<Self> {
        if type_str.is_none() {
            return Ok(InfoType::String);
        }
        let info_type = match type_str.unwrap() {
            "Character" => InfoType::Character,
            "Flag" => InfoType::Flag,
            "Float" => InfoType::Float,
            "Integer" => InfoType::Integer,
            "String" => InfoType::String,
            s => {
                return Err(anyhow::anyhow!("invalid InfoType value `{}`", s));
            }
        };
        Ok(info_type)
    }
}

impl Display for InfoType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            InfoType::Character => write!(f, "Character"),
            InfoType::Flag => write!(f, "Flag"),
            InfoType::Integer => write!(f, "Integer"),
            InfoType::Float => write!(f, "Float"),
            InfoType::String => write!(f, "String"),
        }
    }
}

/// The possible types for the key "ID" of "ALT" fields.
#[derive(Debug, PartialEq)]
pub enum AltId {
    DEL,
    INS,
    DUP,
    INV,
    CNV,
    BND,

    /// For ambiguity codes.
    Other(String),
}

impl AltId {
    fn new(id_str: &str) -> anyhow::Result<Self> {
        if id_str.is_empty() {
            return Err(anyhow::anyhow!("invalid AltId, empty value"));
        }
        let alt_id = match id_str {
            "DEL" => AltId::DEL,
            "INS" => AltId::INS,
            "DUP" => AltId::DUP,
            "INV" => AltId::INV,
            "CNV" => AltId::CNV,
            "BND" => AltId::BND,
            s => AltId::Other(s.to_string()),
        };
        Ok(alt_id)
    }

    fn new_alt_ids(ids_str: &str) -> anyhow::Result<Vec<AltId>> {
        Ok(ids_str.split(':').map(|s| AltId::new(s).unwrap()).collect())
    }
}

impl Display for AltId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            AltId::DEL => write!(f, "DEL"),
            AltId::INS => write!(f, "INS"),
            AltId::DUP => write!(f, "DUP"),
            AltId::INV => write!(f, "INV"),
            AltId::CNV => write!(f, "CNV"),
            AltId::BND => write!(f, "BND"),
            AltId::Other(s) => write!(f, "{}", s),
        }
    }
}

/// The possible types for the key "Type" of "FORMAT" fields.
#[derive(Clone, Debug, PartialEq)]
pub enum FormatType {
    Character,
    Integer,
    Float,
    String,
}

impl FormatType {
    fn new(type_str: Option<&str>) -> anyhow::Result<Self> {
        if type_str.is_none() {
            return Ok(FormatType::String);
        }
        let info_type = match type_str.unwrap() {
            "Character" => FormatType::Character,
            "Float" => FormatType::Float,
            "Integer" => FormatType::Integer,
            "String" => FormatType::String,
            s => {
                return Err(anyhow::anyhow!("invalid FormatType value `{}`", s));
            }
        };
        Ok(info_type)
    }
}

impl Display for FormatType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            FormatType::Character => write!(f, "Character"),
            FormatType::Integer => write!(f, "Integer"),
            FormatType::Float => write!(f, "Float"),
            FormatType::String => write!(f, "String"),
        }
    }
}

/// The possible types for the key "Type" of "FORMAT" fields.
#[derive(Debug, PartialEq)]
pub enum PedigreeType {
    Original(String),
    Parents {
        father_id: String,
        mother_id: String,
    },
    Ancestors(Vec<String>),
}

impl PedigreeType {
    fn new(pedigree_map: LinkedHashMap<&str, &str>) -> anyhow::Result<Self> {
        if pedigree_map.contains_key("Original") {
            Ok(PedigreeType::Original(get_map_value(
                &pedigree_map,
                "Original",
            )?))
        } else if pedigree_map.contains_key("Father") || pedigree_map.contains_key("Mother") {
            Ok(PedigreeType::Parents {
                father_id: get_map_value(&pedigree_map, "Father")?,
                mother_id: get_map_value(&pedigree_map, "Mother")?,
            })
        } else if pedigree_map.contains_key("Name_1") {
            let mut entries = Vec::new();
            let mut kv_entries: Vec<(&str, &str)> = pedigree_map.into_iter().collect();
            kv_entries.sort();
            for (key, value) in kv_entries {
                if key != "ID" {
                    if !key.starts_with("Name_") {
                        return Err(anyhow::anyhow!("invalid pedigree type name `{}`", key));
                    }
                    entries.push(value.to_string());
                }
            }
            Ok(PedigreeType::Ancestors(entries))
        } else {
            Err(anyhow::anyhow!("invalid pedigree type: {:?}", pedigree_map))
        }
    }
}

impl Display for PedigreeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            PedigreeType::Original(s) => write!(f, "Original={}", s),
            PedigreeType::Parents {
                father_id,
                mother_id,
            } => write!(f, "Father={},Mother={}", father_id, mother_id),
            PedigreeType::Ancestors(entries) => {
                for (i, e) in entries.iter().enumerate() {
                    write!(f, "Name_{}={}", i, e)?;
                }
                Ok(())
            }
        }
    }
}

/// Helper function to get a str slice value from a map, convert it to string and return it.
/// If the value does not exist an error is returned.
fn get_map_value(map: &LinkedHashMap<&str, &str>, key: &str) -> anyhow::Result<String> {
    Ok((*map.get(key).ok_or_else(|| {
        anyhow::anyhow!("value not found in map: value=`{}`, map=`{:?}`", key, map)
    })?)
    .to_string())
}

/// Parses the payload of the header.
/// Example payloads:
///     <ID=GT,Number=1,Type=String,Description="Genotype">
///     20100501
///     <ID=DEL,Description="Deletion">
///     <ID=END,Number=1,Type=Integer,Description="End position of the variant described in this record">
///     ftp://ftp-trace.ncbi.nih.gov/1000genomes/ftp/release/sv/breakpoint_assemblies.fasta
///     <ID=Assay,Type=String,Number=.,Values=[WholeGenome, Exome]>
pub fn parse_header_payload(payload: &str) -> anyhow::Result<LinkedHashMap<&str, &str>> {
    // remove triangle brackets, if any.
    let payload = if payload.starts_with('<') || payload.ends_with('>') {
        // either both exist or neither
        if !payload.starts_with('<') || !payload.ends_with('>') {
            return Err(anyhow::anyhow!(
                "invalid header payload `{}`, (unbalanced triangle brackets)",
                payload
            ));
        }
        // remove brackets
        &payload[1..payload.len() - 1]
    } else {
        payload
    };

    // a header payload cannot be empty
    if payload.is_empty() {
        return Err(anyhow::anyhow!("invalid header payload, (empty)"));
    }

    let mut result = LinkedHashMap::new();

    // handle payloads not following the key=value pattern as a single string
    if payload.find('=').is_none() {
        result.insert(OTHER_KEY, payload);
        return Ok(result);
    }

    // parse key=value pairs of payload
    // reference implementation: https://github.com/informationsea/vcf-rs
    enum PayloadParseState {
        // expecting to parse a key
        Key,

        // expecting to parse a value
        Value,

        // expecting to parse a value enclosed within specific characters, e.g., "value" or [v1, v2]
        EnclosedValue(char),

        // expecting to parse end of quoted value, e.g., a `,` or end of line
        QuoteEnded,
    }
    let mut state = PayloadParseState::Key;

    let mut key_start: usize = 0;
    let mut key_end: usize = 0;
    let mut value_start: usize = 0;
    let mut previous_ch: char = '_';

    for (ch_idx, ch) in payload.chars().enumerate() {
        match state {
            PayloadParseState::Key => {
                // '=' indicates end of a key.
                if ch == '=' {
                    key_end = ch_idx;
                    value_start = ch_idx + ch.len_utf8();
                    state = PayloadParseState::Value;
                }
            }
            PayloadParseState::Value => {
                // `,` or eol indicates end of value
                if ch == ',' || ch_idx == payload.len() - 1 {
                    let key = &payload[key_start..key_end];
                    let value = if ch == ',' {
                        &payload[value_start..ch_idx]
                    } else {
                        &payload[value_start..]
                    };
                    if key.is_empty() {
                        return Err(anyhow::anyhow!(
                            "invalid header payload `{}`, (empty key)",
                            payload
                        ));
                    }
                    if value.is_empty() {
                        return Err(anyhow::anyhow!(
                            "invalid header payload `{}`, (empty value)",
                            payload
                        ));
                    }
                    result.insert(key, value);
                    key_start = ch_idx + ch.len_utf8();
                    state = PayloadParseState::Key;
                } else if ch == '"' || ch == '[' {
                    // double quote or opening square bracket indicates an enclosed value. These
                    // characters can occur only at the start of the value
                    if ch_idx != value_start {
                        return Err(anyhow::anyhow!(
                            "invalid header payload `{}`, (invalid character `{}` found)",
                            payload,
                            ch
                        ));
                    }
                    value_start = ch_idx + ch.len_utf8();
                    previous_ch = '_';
                    state = PayloadParseState::EnclosedValue(ch);
                }
            }
            PayloadParseState::EnclosedValue(enclosing_char) => {
                // handle unescaped quote
                if (enclosing_char == '"' && ch == '"' && previous_ch != '\\')
                    || (enclosing_char == '[' && ch == ']')
                {
                    let key = &payload[key_start..key_end];
                    let value = &payload[value_start..ch_idx];
                    if key.is_empty() {
                        return Err(anyhow::anyhow!(
                            "invalid header payload `{}`, (empty key)",
                            payload
                        ));
                    }
                    if value.is_empty() {
                        return Err(anyhow::anyhow!(
                            "invalid header payload `{}`, (empty value)",
                            payload
                        ));
                    }
                    result.insert(key, value);
                    state = PayloadParseState::QuoteEnded;
                    continue;
                }
                // remember previous character.
                if ch == '\\' && previous_ch == '\\' {
                    previous_ch = '_';
                } else {
                    previous_ch = ch;
                }
            }
            PayloadParseState::QuoteEnded => {
                if ch == ',' {
                    state = PayloadParseState::Key;
                    key_start = ch_idx + ch.len_utf8();
                } else {
                    return Err(anyhow::anyhow!(
                        "invalid header payload `{}`, non `,` character found after closing quote",
                        payload
                    ));
                }
            }
        }
    }

    if let PayloadParseState::Value = state {
        return Err(anyhow::anyhow!(
            "invalid header payload `{}`, (empty value)",
            payload
        ));
    }
    if let PayloadParseState::EnclosedValue(_) = state {
        return Err(anyhow::anyhow!(
            "invalid header payload `{}`, (unbalanced quote)",
            payload
        ));
    }
    Ok(result)
}

/// Parses the version of the header.
/// Example:
///     ##fileformat=VCFv4.3 --> VCFv4.3
pub fn parse_version(version_line: &str) -> anyhow::Result<String> {
    let prefix = "##fileformat=";
    if !version_line.starts_with(prefix) {
        return Err(anyhow::anyhow!("invalid version line `{}`", version_line));
    }
    Ok((&version_line[prefix.len()..]).to_string())
}

/// Parses the column names of the header.
/// Example:
///     #CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	NA00001	NA00002	NA00003
pub fn parse_column_names(column_line: &str) -> anyhow::Result<Vec<String>> {
    let prefix = "#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO";
    if !column_line.starts_with(prefix) {
        return Err(anyhow::anyhow!(
            "invalid columns line `{}` (columns line should start with `{}`)",
            column_line,
            prefix
        ));
    }

    let remaining_line = &column_line[prefix.len()..].trim();

    let columns = if !remaining_line.is_empty() {
        let prefix = "FORMAT";
        if !remaining_line.starts_with(prefix) {
            return Err(anyhow::anyhow!(
                "unexpected column name after `INFO` in line `{}`",
                remaining_line
            ));
        }
        let remaining_line = &remaining_line[prefix.len()..].trim();
        let columns: Vec<String> = remaining_line.split('\t').map(|s| s.to_string()).collect();
        // check if column names are unique
        let mut set = HashSet::new();
        if !columns.iter().all(|x| set.insert(x)) {
            return Err(anyhow::anyhow!(
                "sample column names must be unique `{:?}` ",
                columns
            ));
        }
        columns
    } else {
        vec![]
    };
    Ok(columns)
}

#[cfg(test)]
mod test {
    use crate::header::{self, *};
    use linked_hash_map::LinkedHashMap;

    macro_rules! linked_map (
    ( $( $key:expr => $value:expr ),* $(,)?) => {
        {
            let mut m = linked_hash_map::LinkedHashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };);

    #[test]
    fn test_payload_valid() {
        let line = "20100501";
        let expected = linked_map!(
            header::OTHER_KEY => "20100501",
        );
        let actual = parse_header_payload(line).unwrap();
        assert_eq!(actual, expected);

        let line = "<ID=TumourSample,Original=GermlineID>";
        let expected = linked_map!(
            "ID" => "TumourSample",
            "Original" => "GermlineID",
        );
        let actual = parse_header_payload(line).unwrap();
        assert_eq!(actual, expected);

        let line = "<ID=SVTYPE,Description=\"Type of structural variant\">";
        let expected = linked_map!(
            "ID" => "SVTYPE",
            "Description" => "Type of structural variant",
        );
        let actual = parse_header_payload(line).unwrap();
        assert_eq!(actual, expected);

        let line = "<ID=SVTYPE,Description=\"Type of \\\"structural\\\" variant\">";
        let expected = linked_map!(
            "ID" => "SVTYPE",
            "Description" => "Type of \\\"structural\\\" variant",
        );
        let actual = parse_header_payload(line).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_payload_invalid() {
        let line = "";
        let expected = "invalid header payload, (empty)";
        let actual = parse_header_payload(line).err().unwrap().to_string();
        assert_eq!(actual, expected);

        let line = "<ID=Tumour\"Sample>";
        let expected = "invalid header payload `ID=Tumour\"Sample`, (invalid character `\"` found)";
        let actual = parse_header_payload(line).err().unwrap().to_string();
        assert_eq!(actual, expected);

        let line = "<=TumourSample>";
        let expected = "invalid header payload `=TumourSample`, (empty key)";
        let actual = parse_header_payload(line).err().unwrap().to_string();
        assert_eq!(actual, expected);

        let line = "<ID=,Original=GermlineID>";
        let expected = "invalid header payload `ID=,Original=GermlineID`, (empty value)";
        let actual = parse_header_payload(line).err().unwrap().to_string();
        assert_eq!(actual, expected);

        let line = "<ID=>";
        let expected = "invalid header payload `ID=`, (empty value)";
        let actual = parse_header_payload(line).err().unwrap().to_string();
        assert_eq!(actual, expected);

        let line = "<ID=SVTYPE,Description=\"Type of structural variant>";
        let expected = "invalid header payload `ID=SVTYPE,Description=\"Type of structural variant`, (unbalanced quote)";
        let actual = parse_header_payload(line).err().unwrap().to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_version() {
        let line_str = "##fileformat=VCFv4.3";
        let actual_version = parse_version(line_str).unwrap();
        let expected_version = "VCFv4.3";
        assert_eq!(actual_version, expected_version);
    }

    #[test]
    fn test_header_line_valid() {
        let line_str = "##INFO=<ID=BKPTID,Number=.,Type=String,Description=\"ID of the assembled alternate allele in the assembly file\">";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Info {
            id: "BKPTID".to_string(),
            number: Number::Unknown,
            typ: InfoType::String,
            description: "ID of the assembled alternate allele in the assembly file".to_string(),
            source: None,
            version: None,
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##FORMAT=<ID=CNQ,Number=1,Type=Float,Description=\"Copy number genotype quality for imprecise events\">";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Format {
            id: "CNQ".to_string(),
            number: Number::Integer(1),
            typ: FormatType::Float,
            description: "Copy number genotype quality for imprecise events".to_string(),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##FILTER=<ID=s50,Description=\"Less than 50% of samples have data\">";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Filter {
            id: "s50".to_string(),
            description: "Less than 50% of samples have data".to_string(),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##ALT=<ID=INS,Description=\"Insertion of novel sequence\">";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Alt {
            id: vec![AltId::INS],
            description: "Insertion of novel sequence".to_string(),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##ALT=<ID=INS:ME:ALU,Description=\"Insertion of ALU element\">";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Alt {
            id: vec![
                AltId::INS,
                AltId::Other("ME".to_string()),
                AltId::Other("ALU".to_string()),
            ],
            description: "Insertion of ALU element".to_string(),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##assembly=ftp://ftp-trace.ncbi.nih.gov/1000genomes";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line =
            HeaderLine::Assembly("ftp://ftp-trace.ncbi.nih.gov/1000genomes".to_string());
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##contig=<ID=20,length=62435964,assembly=B36,md5=f126cdf8a6e0c7f379d618ff66beb2da,species=\"Homo sapiens\",taxonomy=x>";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Contig {
            id: "20".to_string(),
            species: Some("Homo sapiens".to_string()),
            other: linked_map!(
                "length".to_string() => "62435964".to_string(),
                "assembly".to_string() => "B36".to_string(),
                "md5".to_string() => "f126cdf8a6e0c7f379d618ff66beb2da".to_string(),
                "taxonomy".to_string() => "x".to_string(),
            ),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##META=<ID=Assay,Type=String,Number=.,Values=[WholeGenome, Exome]>";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Meta {
            id: "Assay".to_string(),
            typ: "String".to_string(),
            number: Number::Unknown,
            values: vec!["WholeGenome".to_string(), "Exome".to_string()],
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##SAMPLE=<ID=Sample1,Description=\"Patient germline\">";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Sample {
            id: "Sample1".to_string(),
            meta: LinkedHashMap::default(),
            description: "Patient germline".to_string(),
            doi: None,
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##SAMPLE=<ID=TissueSample,Genomes=Germline;Tumor,Mixture=.3;.7,Description=\"Patient germline genome;Patient tumor genome\",DOI=url>";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Sample {
            id: "TissueSample".to_string(),
            meta: linked_map!(
                "Genomes".to_string() => vec!("Germline".to_string(),"Tumor".to_string()),
                "Mixture".to_string() => vec!(".3".to_string(),".7".to_string()),
            ),
            description: "Patient germline genome;Patient tumor genome".to_string(),
            doi: Some("url".to_string()),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##PEDIGREE=<ID=TumourSample,Original=GermlineID>";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Pedigree {
            id: "TumourSample".to_string(),
            relation: PedigreeType::Original("GermlineID".to_string()),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##PEDIGREE=<ID=ChildID,Father=FatherID,Mother=MotherID>";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Pedigree {
            id: "ChildID".to_string(),
            relation: PedigreeType::Parents {
                father_id: "FatherID".to_string(),
                mother_id: "MotherID".to_string(),
            },
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str =
            "##PEDIGREE=<ID=SampleID,Name_1=Ancestor_1,Name_2=Ancestor_2,Name_3=Ancestor_3>";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::Pedigree {
            id: "SampleID".to_string(),
            relation: PedigreeType::Ancestors(vec![
                "Ancestor_1".to_string(),
                "Ancestor_2".to_string(),
                "Ancestor_3".to_string(),
            ]),
        };
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##pedigreeDB=URL";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::PedigreeDB("URL".to_string());
        assert_eq!(actual_header_line, expected_header_line);

        let line_str = "##fileDate=20100501";
        let actual_header_line = HeaderLine::from_str(line_str).unwrap();
        let expected_header_line = HeaderLine::FileDate("20100501".to_string());
        assert_eq!(actual_header_line, expected_header_line);
    }

    #[test]
    fn test_column_names() {
        let line_str = "#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	NA00001	NA00002	NA00003";
        let actual_version = parse_column_names(line_str).unwrap();
        let expected_version = vec![
            "NA00001".to_string(),
            "NA00002".to_string(),
            "NA00003".to_string(),
        ];
        assert_eq!(actual_version, expected_version);

        // missing format column
        let line_str = "#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	NA00001	NA00002	NA00001";
        let actual_version = parse_column_names(line_str);
        assert!(actual_version.is_err());

        // repeated sample name
        let line_str = "#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	NA00001	NA00002	NA00001";
        let actual_version = parse_column_names(line_str);
        assert!(actual_version.is_err());
    }
}
