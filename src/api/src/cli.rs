// Copyright (c) Bao Project and Contributors. All rights reserved.
//          João Peixoto <joaopeixotooficial@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Bao CLI.

use super::types::VMMConfig;
use clap::{App, Arg, Error};
use std::fs::File;
use std::io::Read;

/// Command line interface.
pub struct Cli;

impl Cli {
    /// Creates a new `Cli` object.
    ///
    /// # Returns
    ///
    /// A `Cli` object.
    pub fn new() -> Self {
        Cli
    }

    /// Launches the command line interface.
    ///
    /// # Examples
    ///
    /// $ bao-vmm --config /path/to/your/config.yaml
    ///
    /// or (short version)
    ///
    /// $ bao-vmm -c /path/to/your/config.yaml
    ///
    /// # Returns
    ///
    /// * `Result<VMMConfig, Error>` - A VMMConfig struct containing the parsed configuration.
    pub fn launch(&self) -> Result<VMMConfig, Error> {
        let vmm_config = match self.parse() {
            Ok(config) => config,
            Err(e) => {
                return Err(Error::with_description(
                    e.to_string(),
                    clap::ErrorKind::InvalidValue,
                ));
            }
        };
        Ok(vmm_config)
    }

    /// Launches the command line interface with a config file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A reference to a string containing the path to the YAML file.
    ///
    /// # Returns
    ///
    /// * `Result<VMMConfig, Error>` - A VMMConfig struct containing the parsed configuration.
    pub fn launch_with_file(&self, file_path: &str) -> Result<VMMConfig, Error> {
        let vmm_config = match self.parse_yaml_config_file(file_path) {
            Ok(config) => config,
            Err(e) => {
                return Err(Error::with_description(
                    e.to_string(),
                    clap::ErrorKind::InvalidValue,
                ));
            }
        };
        Ok(vmm_config)
    }

    /// Parses the VMM arguments.
    ///
    /// # Returns
    ///
    /// * `Result<ConfigFrontends, Box<dyn std::error::Error>>` - A ConfigFrontends struct containing the parsed configuration.
    fn parse(&self) -> Result<VMMConfig, Box<dyn std::error::Error>> {
        // Get the environment command line arguments
        let matches = App::new("Bao Vhost Frontend")
            .arg(
                Arg::with_name("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file")
                    .takes_value(true)
                    .required(true),
            )
            .get_matches();

        // Extract the config file path
        let config_file = matches.value_of("config").unwrap();

        // Parse the YAML file
        let frontends = self.parse_yaml_config_file(config_file)?;

        // Return the configuration
        Ok(frontends)
    }

    /// Parses the YAML configuration file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A reference to a string containing the path to the YAML file.
    ///
    /// # Returns
    ///
    /// * `Result<VMMConfig, Box<dyn std::error::Error>>` - A VMMConfig struct containing the parsed configuration.
    fn parse_yaml_config_file(
        &self,
        file_path: &str,
    ) -> Result<VMMConfig, Box<dyn std::error::Error>> {
        // Open the YAML file
        let mut file = File::open(file_path).unwrap();
        // Read the YAML file
        let mut yaml_content = String::new();
        file.read_to_string(&mut yaml_content).unwrap();
        // Parse the YAML file
        let vmm_config: VMMConfig = serde_yaml::from_str(&yaml_content).unwrap();
        // Return the configuration
        Ok(vmm_config)
    }
}
