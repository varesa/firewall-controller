use std::env;
use anyhow::{Context, Error};
use firewall_controller::pod::Pod;

fn main() -> Result<(), Error> {
    let mut args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
    let dp_name = args.remove(1);

    let pod_name = format!("dp-{dp_name}");
    let pod = Pod::ensure_exists(&pod_name).context("Failed to ensure pod exists")?;
    pod.ensure_is_running().context("Failed to ensure pod is running")?;
    let netstate = pod.get_network_state().context("Failed to get pod network state")?;
    println!("{netstate:?}");

    Ok(())
}
