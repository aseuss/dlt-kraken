use std::path;
use clap::Parser;

#[derive(Parser,Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<path::PathBuf>,

    /// input files
    #[arg(short, long, value_name = "INPUT", required = true)]
    input: Vec<path::PathBuf>,

    /// ECU id for filtering
    #[arg(long = "ecu")]
    ecu_id: Option<String>,

    /// APP id for filtering
    #[arg(long = "app")]
    app_id: Option<String>,

    /// CONTEXT id for filtering
    #[arg(long = "ctx")]
    context_id: Option<String>,

    /// patterns used for filtering
    #[arg(short, long)]
    patterns: Vec<String>,
}

impl Cli {
    pub fn config(&self) -> &Option<path::PathBuf> {
        &self.config
    }

    pub fn input(&self) -> &Vec<path::PathBuf> {
        &self.input
    }

}