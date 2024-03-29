use api::cli::Cli;
use vmm::vmm::Vmm;

fn main() {
    // Create a new CLI object.
    let cli = Cli::new();

    // Launch the CI to parse the configuration file.
    let vmm_config = cli.launch().unwrap();

    // Create a new VMM.
    let vmm = Vmm::try_from(vmm_config).unwrap();

    // Run the VMM.
    vmm.run().unwrap();
}
