use anyhow::{Context, Error};
use std::path::PathBuf;
use firewall_controller::dataplane::{Dataplane, DataplaneList};


fn main() -> Result<(), Error> {
    Dataplane::create_template_service().context("Failed to create dataplane service template")?;

    let dp_filename = "/etc/dataplanes.yaml";
    let dataplane_list = DataplaneList::from_file(&PathBuf::from(dp_filename))?;

    for dp in dataplane_list.dataplanes {
        dp.enable_now()?;
        println!("{dp:?}");
    }

    Ok(())
}
