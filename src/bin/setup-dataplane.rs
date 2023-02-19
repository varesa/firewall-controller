use anyhow::{Context, Error};
use firewall_controller::pod::Pod;
use std::env;
use std::process::Command;

fn main() -> Result<(), Error> {
    let mut args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
    let dp_name = args.remove(1);

    let pod_name = format!("dp-{dp_name}");
    let pod = Pod::ensure_exists(&pod_name).context("Failed to ensure pod exists")?;
    pod.ensure_is_running()
        .context("Failed to ensure pod is running")?;

    let netns = pod.get_infra_container()?.get_netns()?;
    let out = netns.run(|| Command::new("ip").arg("addr").output().unwrap().stdout);
    println!("{}", String::from_utf8(out).unwrap());

    let netstate = pod.get_network_state().unwrap();
    dbg!(netstate);

    Ok(())
}
