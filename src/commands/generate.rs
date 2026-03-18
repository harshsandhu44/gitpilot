use anyhow::Result;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use clap::CommandFactory;
use clap_mangen::Man;
use crate::cli::{Cli, GenerateTarget};
use crate::commands::completions;

pub fn run(target: &GenerateTarget) -> Result<()> {
    match target {
        GenerateTarget::Completions { shell } => {
            completions::run(*shell)
        }
        GenerateTarget::Man { output } => {
            let cmd = Cli::command();
            let man = Man::new(cmd);
            match output {
                Some(path) => {
                    let file = File::create(path)?;
                    let mut writer = BufWriter::new(file);
                    man.render(&mut writer)?;
                    writer.flush()?;
                    eprintln!("Man page written to {}", path.display());
                }
                None => {
                    let stdout = io::stdout();
                    let mut writer = BufWriter::new(stdout.lock());
                    man.render(&mut writer)?;
                    writer.flush()?;
                }
            }
            Ok(())
        }
    }
}
