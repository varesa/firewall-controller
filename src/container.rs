use crate::netns::NetworkNamespace;
use anyhow::{Context, Error};
use serde_derive::Deserialize;
use std::fs::File;
use std::process::Command;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerState {
    pid: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerInspection {
    id: String,
    state: ContainerState,
}

pub struct Container {
    pub id: String,
}

impl Container {
    pub fn exists(id: &str) -> Result<bool, Error> {
        let exists = Command::new("podman")
            .arg("container")
            .arg("exists")
            .arg(id)
            .status()?
            .success();
        Ok(exists)
    }

    pub fn get(id: &str) -> Result<Self, Error> {
        if Self::exists(id).context("Failed to check if container exists")? {
            let inspection = Self::_inspect(id).context("Failed to inspect container")?;
            Ok(Self { id: inspection.id })
        } else {
            Err(Error::msg(format!("Container {} doesn't exist", id)))
        }
    }

    pub fn inspect(&self) -> Result<ContainerInspection, Error> {
        Self::_inspect(&self.id)
    }

    fn _inspect(name: &str) -> Result<ContainerInspection, Error> {
        let output = Command::new("podman")
            .arg("container")
            .arg("inspect")
            .arg(name)
            .output()?;
        if !output.status.success() {
            Err(Error::msg(String::from_utf8(output.stderr)?)
                .context(format!("podman container inspect {} failed", &name)))
        } else {
            let mut containers: Vec<ContainerInspection> =
                serde_json::from_slice(&output.stdout)
                    .context("Failed to decode JSON to ContainerInspection")?;
            assert_eq!(containers.len(), 1);
            Ok(containers.remove(0))
        }
    }

    pub fn get_pid(&self) -> Result<u64, Error> {
        let pid = self
            .inspect()
            .context("Failed to inspect container")?
            .state
            .pid;
        Ok(pid)
    }

    pub fn get_netns(&self) -> Result<NetworkNamespace, Error> {
        let pid = self.get_pid().context("Failed to get container PID")?;
        let path = format!("/proc/{pid}/ns/net");
        let file = File::open(path).context("Failed to open netns file")?;
        Ok(NetworkNamespace::from_file(file))
    }
}
