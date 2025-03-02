use clap::{Parser, Subcommand, command};
use serde::{Deserialize, Serialize};

// CLI Arguments
#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct PupCli {
    #[command(subcommand)]
    pub command: Option<PupCliCommands>,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum PupCliCommands {
    PrintModules,
    PrintTraits,
    Check,
}

impl PupCli {
    // This shows up as dead for one of our bins. It's not, so
    // we silence it.
    #![allow(dead_code)]

    pub fn to_env_str(&self) -> String {
        serde_json::to_string(self).expect("Failed serializing CLI args")
    }

    pub fn from_env_str(env_str: &str) -> PupCli {
        serde_json::from_str(env_str).expect("Failed deserializing CLI args")
    }
}
