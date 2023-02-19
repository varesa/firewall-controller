use std::fs::File;
use anyhow::{Error, Context};

use dataplane::{DataplaneList, Dataplane};

mod dataplane;

fn main() -> Result<(), Error> {
    Dataplane::create_template_service().context("Failed to create dataplane service template")?;

    let dp_filename = "/etc/dataplanes.yaml";
    let dp_file = File::open(dp_filename).context(format!("Failed to open file {dp_filename}"))?;
    let dataplane_list: DataplaneList = serde_yaml::from_reader(dp_file).context("Failed to parse dataplanes from YAML")?;
    
    for dp in dataplane_list.dataplanes {
        dp.enable_now()?;
        println!("{dp:?}");
    }

    Ok(())
}
