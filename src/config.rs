use std::error::Error;
use std::{fs, path};
use std::path::Path;
use serde_derive::Deserialize;
use toml::Value;
use std::process;

#[derive(Deserialize,Debug)]
pub struct Config {
    filters: Option<Vec<Filter>>,
}

impl Config {

    pub fn filters(&self) -> &Option<Vec<Filter>> {
        &self.filters
    }

    fn is_valid(&self) -> Result<(), &'static str> {
        let is_filter_valid = match &self.filters {
            Some(filters) => filters.iter().all(|filter| filter.is_valid()),
            None => true,
        };

        if is_filter_valid {
            Ok(())
        } else {
            Err("config file invalid")
        }
    }
}

#[derive(Deserialize,Debug)]
pub struct Filter {
    name: String,
    ecu_id: Option<String>,
    app_id: Option<String>,
    context_id: Option<String>,
    patterns: Option<Vec<String>>,
    output: Option<Output>,
}

fn validate_id(name: &str, id: &Option<String>) -> bool {
    match id {
        Some(id) if id.is_ascii() && id.len() <= 4 => true,
        Some(id) => {
            println!("{name} non-ascii or too long (4 char max): {id}");
            false
        },
        _ => true,
    }
}

impl Filter {

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn ecu_id(&self) -> &Option<String> {
        &self.ecu_id
    }

    pub fn app_id(&self) -> &Option<String> {
        &self.app_id
    }

    pub fn context_id(&self) -> &Option<String> {
        &self.context_id
    }

    pub fn patterns(&self) -> &Option<Vec<String>> {
        &self.patterns
    }

    pub fn output(&self) -> &Option<Output> {
        &self.output
    }

    fn is_valid(&self) -> bool {
        let is_ecu_id_valid = validate_id("ecu_id", &self.ecu_id);
        let is_app_id_valid = validate_id("app_id", &self.app_id);
        let is_context_id_valid = validate_id("context_id", &self.context_id);
        // TODO: validate patterns!
        let is_output_valid = match &self.output {
            Some(out) => out.is_valid(),
            None => true,
        };
        is_ecu_id_valid && is_app_id_valid && is_context_id_valid && is_output_valid
    }
}

#[derive(Deserialize,Debug)]
pub struct Output {
    csv: Option<Csv>,
    stdout: Option<Stdout>,
}

impl Output {
    pub fn csv(&self) -> &Option<Csv> {
        &self.csv
    }

    pub fn stdout(&self) -> &Option<Stdout> {
        &self.stdout
    }

    fn is_valid(&self) -> bool {
        let is_csv_valid = match &self.csv {
            Some(csv) => csv.is_valid(),
            None => true,
        };
        let is_stdout_valid = match &self.stdout {
            Some(stdout) => stdout.is_valid(),
            None => true,
        };
        is_csv_valid && is_stdout_valid
    }
}

#[derive(Deserialize,Debug)]
pub struct Csv {
    file_path: path::PathBuf,
    #[serde(default = "Csv::default_delimiter")]
    delimiter: char,
    format: Option<String>,
}

impl Csv {
    fn default_delimiter() -> char {
        ','
    }

    fn is_valid(&self) -> bool {
        // TODO: improve filename validation
        let is_file_path_valid = true;
        let is_delimiter_valid = match &self.delimiter {
            ',' => true,
            ';' => true,
            ' ' => true,
            '\t' => true,
            ':' => true,
            '|' => true,
            _ => {
                eprintln!("invalid delimiter: {}", &self.delimiter);
                false
            },
        };
        // TODO: check output format, or rather which fields should be output
        is_file_path_valid && is_delimiter_valid
    }
}

#[derive(Deserialize,Debug)]
pub struct Stdout {
    #[serde(default = "Stdout::default_enabled")]
    enabled: bool,
    delimiter: char,
    format: String,
}

impl Stdout {
    fn default_enabled() -> bool {
        false
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn delimiter(&self) -> char {
        self.delimiter
    }

    pub fn format_string(&self) -> &String {
        &self.format
    }

    fn is_valid(&self) -> bool {
        if self.enabled {
            // TODO: check output format
            true
        } else {
            true
        }
    }
}

pub fn read_config(file_path: &Path) -> Result<Config, Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)?;
    let config: Config = toml::from_str(&contents).unwrap();
    if let Err(err) = config.is_valid() {
        eprintln!("{err}");
        process::exit(1)
    }
    Ok(config)
}
