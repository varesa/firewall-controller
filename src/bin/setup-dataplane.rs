use std::env;
use std::process::Command;
use nispor::NetState;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PodInspection {
    state: String,
}

impl PodInspection {
    fn is_running(&self) -> bool {
        self.state == "Running"
    }
}

use anyhow::{Context, Error};
use serde::Deserialize;

fn pod_exists(name: &str) -> Result<bool, Error> {
    let exists = Command::new("podman")
        .arg("pod")
        .arg("exists")
        .arg(name)
        .status()
        .context(format!("failed to check if pod {name} exists"))?
        .success();
    Ok(exists)
}

fn ensure_pod_exists(name: &str) -> Result<(), Error> {
    if pod_exists(name)? {
        Ok(())
    } else {
        let output = Command::new("podman")
            .arg("pod")
            .arg("create")
            .arg("--name")
            .arg(name)
            .arg("--network=none")
            .output()
            .context(format!("failed to execute pod {name} creation"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::msg(
                String::from_utf8(output.stderr).context("Failed to load stderr to string")?,
            ))
        }
    }
}

fn ensure_pod_is_running(name: &str) -> Result<(), Error> {
    if inspect_pod(name)
        .context("Failed to inspect pod")?
        .is_running()
    {
        Ok(())
    } else {
        let output = Command::new("podman")
            .arg("pod")
            .arg("start")
            .arg(name)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::msg(String::from_utf8(output.stderr)?)
                .context(format!("podman pod start {name} failed")))
        }
    }
}

fn inspect_pod(name: &str) -> Result<PodInspection, Error> {
    let output = Command::new("podman")
        .arg("pod")
        .arg("inspect")
        .arg(name)
        .output()?;
    if !output.status.success() {
        Err(Error::msg(String::from_utf8(output.stderr)?)
            .context(format!("podman pod inspect {name} failed")))
    } else {
        Ok(serde_json::from_slice(&output.stdout)
            .context("Failed to decode JSON to PodInspection")?)
    }
}

fn get_pod_network_state(name: &str) -> Result<NetState, Error> {
    let output = Command::new("podman")
        .arg("run")
        .arg("--rm")
        .arg("-i")
        .arg(format!("--pod={name}"))
        .arg("nispor")
        .output()?;
    if !output.status.success() {
        Err(Error::msg(String::from_utf8(output.stderr)?)
            .context(format!("podman run --pod={name} nispor failed")))
    } else {
        let netstate: NetState = serde_yaml::from_slice(&output.stdout).context("Failed to decode YAML to NetState")?;
        Ok(netstate)
    }
}

fn main() -> Result<(), Error> {
    let mut args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
    let dp_name = args.remove(1);

    let pod_name = format!("dp-{dp_name}");
    ensure_pod_exists(&pod_name).context("Failed to ensure pod exists")?;
    ensure_pod_is_running(&pod_name).context("Failed to ensure pod is running")?;
    let netstate = get_pod_network_state(&pod_name).context("Failed to get pod network state")?;
    println!("{netstate:?}");

    Ok(())
}
