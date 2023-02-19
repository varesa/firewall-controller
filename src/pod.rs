use std::process::Command;
use anyhow::{Context, Error};
use nispor::NetState;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PodInspection {
    state: String,
}

impl PodInspection {
    fn is_running(&self) -> bool {
        self.state == "Running"
    }
}

pub struct Pod {
    name: String,
}

impl Pod {
    pub fn exists(name: &str) -> Result<bool, Error> {
        let exists = Command::new("podman")
            .arg("pod")
            .arg("exists")
            .arg(name)
            .status()?
            .success();
        Ok(exists)
    }

    pub fn get(name: &str) -> Result<Pod, Error> {
        if Self::exists(name).context("Failed to check if pod exists")? {
            Ok(Pod {
                name: String::from(name)
            })
        } else {
            Err(Error::msg(format!("Pod {} doesn't exist", name)))
        }
    }

    pub fn ensure_exists(name: &str) -> Result<Pod, Error> {
        if !Self::exists(name).context("Failed to check if pod exists")? {
            let output = Command::new("podman")
                .arg("pod")
                .arg("create")
                .arg("--name")
                .arg(name)
                .arg("--network=none")
                .output()
                .context(format!("failed to execute pod {name} creation"))?;
            if !output.status.success() {
                return Err(Error::msg(
                    String::from_utf8(output.stderr).context("Failed to load stderr to string")?,
                ))
            }
        };
        Self::get(name)
    }

    pub fn ensure_is_running(&self) -> Result<(), Error> {
        if self.inspect()
            .context("Failed to inspect pod")?
            .is_running()
        {
            Ok(())
        } else {
            let output = Command::new("podman")
                .arg("pod")
                .arg("start")
                .arg(&self.name)
                .output()?;
            if output.status.success() {
                Ok(())
            } else {
                Err(Error::msg(String::from_utf8(output.stderr)?)
                    .context(format!("podman pod start {} failed", &self.name)))
            }
        }
    }

    pub fn inspect(&self) -> Result<PodInspection, Error> {
        let output = Command::new("podman")
            .arg("pod")
            .arg("inspect")
            .arg(&self.name)
            .output()?;
        if !output.status.success() {
            Err(Error::msg(String::from_utf8(output.stderr)?)
                .context(format!("podman pod inspect {} failed", self.name)))
        } else {
            Ok(serde_json::from_slice(&output.stdout)
                .context("Failed to decode JSON to PodInspection")?)
        }
    }

    pub fn get_network_state(&self) -> Result<NetState, Error> {
        let output = Command::new("podman")
            .arg("run")
            .arg("--rm")
            .arg("-i")
            .arg(format!("--pod={}", self.name))
            .arg("nispor")
            .output()?;
        if !output.status.success() {
            Err(Error::msg(String::from_utf8(output.stderr)?)
                .context(format!("podman run --pod={} nispor failed", self.name)))
        } else {
            let netstate: NetState = serde_yaml::from_slice(&output.stdout).context("Failed to decode YAML to NetState")?;
            Ok(netstate)
        }
    }

}