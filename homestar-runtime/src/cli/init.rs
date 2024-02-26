use clap::ValueEnum;
use inquire::{ui::RenderConfig, Confirm, CustomType, Select};
use miette::{bail, miette, Result};
use rand::Rng;
use serde::de::IntoDeserializer;
use serde_with::{base64::Standard, formats::Padded, DeserializeAs, SerializeAs};
use std::{
    fmt::Display,
    fs::File,
    io::{empty, stdout, IsTerminal, Write},
    path::PathBuf,
    str::FromStr,
};

use crate::{
    settings::{KeyType, PubkeyConfig},
    ExistingKeyPath, NetworkBuilder, NodeBuilder, RNGSeed, Settings, SettingsBuilder,
};

use super::InitArgs;

/// Where to write the resulting configuration.
#[derive(Debug)]
pub enum OutputMode {
    /// Write to standard output.
    StdOut,
    /// Write to a file.
    File {
        /// The path to write to.
        path: PathBuf,
        /// Automatically overwrite the file if it exists.
        force: bool,
    },
}

#[derive(Debug)]
enum PubkeyConfigOption {
    GenerateFromSeed,
    FromFile,
}

/// The arguments for configuring the key
#[derive(Debug)]
pub enum KeyArg {
    /// Load the key from an existing file
    File {
        /// The path of the file
        path: PathBuf,
    },
    /// Generate the key from a seed
    Seed {
        /// The base64 encoded 32 byte seed to use for key generation
        seed: Option<String>,
    },
}

/// The type of key to generate
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum KeyTypeArg {
    Ed25519,
    Secp256k1,
}

#[derive(Debug, Clone)]
struct PubkeySeed([u8; 32]);

impl Display for PubkeySeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_with::base64::Base64::<Standard, Padded>::serialize_as(&self.0, f)
    }
}

impl FromStr for PubkeySeed {
    type Err = serde::de::value::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        Ok(Self(
            serde_with::base64::Base64::<Standard, Padded>::deserialize_as(s.into_deserializer())?,
        ))
    }
}

impl Display for PubkeyConfigOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PubkeyConfigOption::GenerateFromSeed => write!(f, "Generate from seed"),
            PubkeyConfigOption::FromFile => write!(f, "From file"),
        }
    }
}

/// Handle the `init` command.
pub fn handle_init_command(init_args: InitArgs) -> Result<()> {
    let output_mode = if init_args.dry_run {
        OutputMode::StdOut
    } else {
        OutputMode::File {
            path: init_args.output_path.unwrap_or_else(Settings::path),
            force: init_args.force,
        }
    };

    let key_arg = init_args
        .key_file
        .map(|key_file| KeyArg::File { path: key_file })
        .or_else(|| {
            init_args
                .key_seed
                .map(|key_seed| KeyArg::Seed { seed: key_seed })
        });

    // Run non-interactively if the input device is not a TTY
    // or if the `--no-input` flag is passed.
    let no_input = init_args.no_input || !stdout().is_terminal();

    let mut settings_builder = SettingsBuilder::default();
    let mut node_builder = NodeBuilder::default();
    let mut network_builder = NetworkBuilder::default();

    let mut writer = handle_quiet(init_args.quiet)?;
    let key_type = handle_key_type(init_args.key_type, no_input, &mut writer)?;
    let keypair_config = handle_key(key_arg, key_type, no_input, &mut writer)?;

    let network = network_builder
        .keypair_config(keypair_config)
        .build()
        .expect("to build network");

    let node = node_builder
        .network(network)
        .build()
        .expect("to build node");

    let settings = settings_builder
        .node(node)
        .build()
        .expect("to builder settings");

    let settings_toml = toml::to_string_pretty(&settings).expect("to serialize settings");

    handle_output_mode(output_mode, no_input, &mut writer)?
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
    no_input: bool,
    writer: &mut Box<dyn Write>,
) -> Result<Box<dyn Write>> {
    match output_mode {
        OutputMode::StdOut => Ok(Box::new(stdout())),
        OutputMode::File { path, force: true } => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("to create parent directory");
            }

            let settings_file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .expect("to open settings file");

            writeln!(writer, "Writing settings to {:?}", path).expect("to write");

            Ok(Box::new(settings_file))
        }
        OutputMode::File { path, force: false } => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("to create parent directory");
            }

            let settings_file = File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path);

            // This seemingly convoluted match is to avoid the risk of a
            // TOCTOU race condition, where another process creates the file
            // in between this one checking for its existence and opening it.
            let settings_file = match settings_file {
                Ok(file) => file,
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    if no_input {
                        bail!("Aborting... settings file already exists at {:?}. Pass `--force` to overwrite it", path);
                    }

                    let should_overwrite = Confirm::new(&format!(
                        "Settings file already exists at {:?}, overwrite?",
                        path
                    ))
                    .with_default(false)
                    .prompt()
                    .map_err(|e| miette!(e))?;

                    if !should_overwrite {
                        bail!("Aborting... not overwriting existing settings file");
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

fn handle_key_type(
    key_type: Option<KeyTypeArg>,
    no_input: bool,
    _writer: &mut Box<dyn Write>,
) -> Result<KeyType> {
    match key_type {
        Some(KeyTypeArg::Ed25519) => Ok(KeyType::Ed25519),
        Some(KeyTypeArg::Secp256k1) => Ok(KeyType::Secp256k1),
        None => {
            if no_input {
                bail!("Aborting... cannot prompt for key type in non-interactive mode. Pass `--key-type <KEY_TYPE>` to set it.");
            }

            let options = vec![KeyType::Ed25519, KeyType::Secp256k1];

            let key_type = Select::new("Select key type", options)
                .prompt()
                .map_err(|e| miette!(e))?;

            Ok(key_type)
        }
    }
}

fn handle_key(
    key_arg: Option<KeyArg>,
    key_type: KeyType,
    no_input: bool,
    _writer: &mut Box<dyn Write>,
) -> Result<PubkeyConfig> {
    let config = match key_arg {
        None => {
            if no_input {
                bail!("Aborting... cannot prompt for key in non-interactive mode. Pass `--key-file <KEY_FILE>` or `--key-seed [<KEY_SEED>]` to configure the key.");
            }

            let options = vec![
                PubkeyConfigOption::GenerateFromSeed,
                PubkeyConfigOption::FromFile,
            ];

            let pubkey_config_choice =
                Select::new("How would you like to configure the key?", options)
                    .prompt()
                    .map_err(|e| miette!(e))?;

            match pubkey_config_choice {
                PubkeyConfigOption::GenerateFromSeed => {
                    let seed = CustomType::<PubkeySeed>::new("Enter the seed for the key")
                        .with_default(PubkeySeed(rand::thread_rng().gen::<[u8; 32]>()))
                        .with_default_value_formatter(&|_| "random".to_string())
                        .with_error_message("Please type a base64 encoding of 32 bytes")
                        .with_help_message("Base64 encoded 32 bytes")
                        .prompt()
                        .map_err(|e| miette!(e))?;

                    PubkeyConfig::GenerateFromSeed(RNGSeed::new(key_type, seed.0))
                }
                PubkeyConfigOption::FromFile => {
                    // HACK: We need to manually instantiate the struct with a custom formatter,
                    // because PathBuf doesn't implement Display, but the new constructor requires
                    // T: Display
                    let path = CustomType::<'_, PathBuf> {
                        message: "Enter the path for the key",
                        formatter: &|p: PathBuf| p.display().to_string(),
                        default_value_formatter: &|p| p.display().to_string(),
                        default: None,
                        validators: vec![],
                        placeholder: None,
                        error_message: "Please type a valid path".to_string(),
                        help_message: None,
                        parser: &|a| a.parse::<PathBuf>().map_err(|_| ()),
                        render_config: RenderConfig::default(),
                    }
                    .prompt()
                    .map_err(|e| miette!(e))?;

                    PubkeyConfig::Existing(ExistingKeyPath::new(key_type, path))
                }
            }
        }
        Some(KeyArg::File { path }) => PubkeyConfig::Existing(ExistingKeyPath::new(key_type, path)),
        Some(KeyArg::Seed { seed: None }) => {
            let seed = rand::thread_rng().gen::<[u8; 32]>();

            PubkeyConfig::GenerateFromSeed(RNGSeed::new(key_type, seed))
        }
        Some(KeyArg::Seed { seed: Some(seed) }) => {
            let Ok(seed) = PubkeySeed::from_str(&seed) else {
                bail!("Invalid seed: expected a base64 encoding of 32 bytes")
            };

            PubkeyConfig::GenerateFromSeed(RNGSeed::new(key_type, seed.0))
        }
    };

    config
        .keypair()
        .map_err(|e| miette!(format!("Failed to load key: {}", e)))?;

    Ok(config)
}
