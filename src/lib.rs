use std::fmt::Error;
use std::path::PathBuf;
use std::process;
use regex::{Regex};
use crate::cli::Cli;
use clap::Parser;
use crate::config::Filter;
use crate::dlt::filter::{FilterId, FilterType, Pattern};

pub mod dlt;
pub mod config;
pub mod cli;

#[derive(Debug)]
pub enum OutputField {
    Ecu,
    App,
    Ctx,
    Time,
    Timestamp,
    Payload,
    Capture(String),
}

impl OutputField {
    fn from(input: &str) -> Option<OutputField> {
        println!("transform {input}");
        match input {
            "ecu" => Some(OutputField::Ecu),
            "app" => Some(OutputField::App),
            "ctx" => Some(OutputField::Ctx),
            "time" => Some(OutputField::Time),
            "timestamp" => Some(OutputField::Timestamp),
            "payload" => Some(OutputField::Payload),
            x if x.starts_with('<') && x.ends_with('>') => {
                Some(OutputField::Capture(x[1..x.len()-1].to_string()))
            },
            _ => {
                eprintln!("invalid field name: {input}");
                None
            },
        }
    }
}

#[derive(Debug)]
pub enum OutputType {
    Csv(Csv),
    Stdout(Stdout),
}

#[derive(Debug)]
pub struct Csv {
    delimiter: char,
    file_path: PathBuf,
}

#[derive(Debug)]
pub struct Stdout {
    delimiter: char,
}

#[derive(Debug)]
pub struct Output {
    out_type: OutputType,
    fields: Vec<OutputField>,
}

impl Output {
    pub fn output_type(&self) -> &OutputType {
        &self.out_type
    }

    pub fn fields(&self) -> &Vec<OutputField> {
        &self.fields
    }

    fn validate_captures(filter : &Filter, fields: &Vec<OutputField>) -> Result<(), String> {
        let field_verifier = fields.iter().filter(|field| match field {
            OutputField::Capture(_) => true,
            _ => false,
        });
        let capture_names = filter.patterns().as_ref().map_or_else(|| None, |patterns| Pattern::capture_names(patterns));
        // validate output fields for captures
        for field in field_verifier {
            match field {
                OutputField::Capture(name) => {
                    if let Some(capture_names) = &capture_names {
                        if capture_names.iter().find(|capture_name| *capture_name == name) == None {
                            return Err::<(),String>(format!("no capture defined for stdout field '{name}' in filter '{}'", filter.name()));
                        }
                    } else {
                        return Err::<(),String>(format!("capture '{name}' from stdout not found: no captures defined in filter '{}'", filter.name()));
                    }
                },
                _ => unreachable!("other output fields should have been filtered!"),
            }
        }
        Ok(())
    }

    pub fn from_filter(filter: &Filter) -> Option<Output> {
        match filter.output() {
            Some(output) => {
                if let Some(stdout) = output.stdout() {
                    if stdout.is_enabled() {
                        let fields : Vec<_>= stdout.format_string().split(stdout.delimiter()).collect();
                        let fields : Vec<_> = fields.iter().filter_map(|field_name| OutputField::from(field_name)).collect();

                        match Output::validate_captures(filter, &fields) {
                            Ok(_) => Some(Output {
                                out_type: OutputType::Stdout(Stdout { delimiter: stdout.delimiter() }),
                                fields: fields,
                            }),
                            Err(err) => {
                                eprintln!("{err}");
                                process::exit(1);
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub fn run() {
    let args : Cli = Cli::parse();
    println!("cli {args:?}");
    let mut filters = dlt::filter::Filter::new();
    let mut output : Option<Output> = None;
    if let Some(config_path) = args.config.as_deref() {
        println!("config file: {config_path:?}");
        let config = config::read_config(config_path).unwrap_or_else(|err| {
            println!("error in reading config: {err}");
            process::exit(1);
        });
        if let Some(cfg_filters) = config.filters() {
            for cfg_filter in cfg_filters {
                match cfg_filter.ecu_id() {
                    Some(ecu_id) => {
                        filters.add(FilterId::EcuId, FilterType::EcuId(ecu_id.to_string()));
                    },
                    _ => (),
                }
                match cfg_filter.app_id() {
                    Some(app_id) => {
                        filters.add(FilterId::AppId, FilterType::AppId(app_id.to_string()));
                    },
                    _ => (),
                }
                match cfg_filter.context_id() {
                    Some(context_id) => {
                        filters.add(FilterId::ContextId, FilterType::ContextId(context_id.to_string()));
                    },
                    _ => (),
                }
                let mut capture_names : Option<Vec<String>> = None;
                match cfg_filter.patterns() {
                    Some(patterns) => {
                        let regex = Regex::new("<(?P<name>[a-z]+)>").unwrap();
                        let mut names: Vec<String> = vec![];

                        for pattern in patterns {
                            let captures : Vec<_>= regex.captures_iter(pattern).collect();
                            for capture in captures {
                                if let Some(name) = capture.name("name") {
                                    names.push(name.as_str().to_string());
                                }
                            }
                        }

                        if names.is_empty() {
                            capture_names = None;
                        } else {
                            capture_names = Some(names);
                        }

                        let patterns= Pattern::from(&*patterns);
                        filters.add(FilterId::Patterns, FilterType::Patterns(patterns));
                    },
                    _ => ()
                }

                output = Output::from_filter(&cfg_filter);
            }
        }
        println!("config: {config:?}");
    }

    println!("lib filter: {filters:?}");
    dlt::run_dlt(&args.input()[0], &filters, &output)
}
