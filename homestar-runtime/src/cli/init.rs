use std::{
    fs::File,
    io::{empty, stdout, Write},
    path::PathBuf,
};

use inquire::Confirm;
use miette::{bail, Result};

use crate::Settings;

/// Where to write the resulting configuration.
#[derive(Debug)]
pub enum OutputMode {
    /// Write to standard output.
    StdOut,
    /// Write to a file.
    File(PathBuf),
}

/// Handle the `init` command.
pub fn handle_init_command(output_mode: OutputMode, quiet: bool) -> Result<()> {
    let settings = Settings::default();

    let mut writer = handle_quiet(quiet)?;
    let mut settings_writer = handle_output_mode(output_mode, &mut writer)?;

    let settings_toml = toml::to_string_pretty(&settings).expect("to serialize settings");

    settings_writer
        .write_all(settings_toml.as_bytes())
        .expect("to write settings file");

    Ok(())
}

fn handle_quiet(quiet: bool) -> Result<Box<dyn Write>> {
    if quiet {
        Ok(Box::new(empty()))
    } else {
        Ok(Box::new(stdout()))
    }
}

fn handle_output_mode(
    output_mode: OutputMode,
    writer: &mut Box<dyn Write>,
) -> Result<Box<dyn Write>> {
    match output_mode {
        OutputMode::StdOut => Ok(Box::new(stdout())),
        OutputMode::File(path) => {
            let settings_file = File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path);

            // This seemingly convoluted match is to avoid the risk of a
            // TOCTOU race condition, where another process creates the file
            // in between this one checking for its existence and opening it.
            //
            // TODO: there should probably be a flag for non-interactive use
            // that automatically overwrites the file.
            let settings_file = match settings_file {
                Ok(file) => file,
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    let should_overwrite = Confirm::new(&format!(
                        "Settings file already exists at {:?}, overwrite?",
                        path
                    ))
                    .with_default(false)
                    .prompt()
                    .expect("to prompt for overwrite");

                    if !should_overwrite {
                        bail!("Aborting...");
                    }

                    File::options()
                        .read(true)
                        .write(true)
                        .create_new(false)
                        .open(&path)
                        .expect("to open settings file")
                }
                err => err.expect("to open settings file"),
            };

            writeln!(writer, "Writing settings to {:?}", path).expect("to write");

            Ok(Box::new(settings_file))
        }
    }
}
