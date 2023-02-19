use anyhow::{Context, Error};
use serde_derive::Deserialize;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, Deserialize)]
pub struct DataplaneList {
    pub dataplanes: Vec<Dataplane>,
}

#[derive(Debug, Deserialize)]
pub struct Dataplane {
    pub name: String,
    pub id: u32,
}

async fn reload_systemd() -> Result<(), Error> {
    let conn = zbus::Connection::system().await.context("Failed D-Bus connection")?;
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn).await.context("Failed to get systemd API")?;
    manager.reload().await.context("Failed to call reload method on systemd.Manager")?;
    Ok(())
}

async fn enable_systemd_unit_now(name: &str) -> Result<(), Error> {
    let conn = zbus::Connection::system().await.context("Failed D-Bus connection")?;
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn).await.context("Failed to get systemd API")?;
    manager.enable_unit_files(vec![format!("dataplane@{name}.service")], false, false).await.context("Failed to enable unit")?;
    manager.start_unit(format!("dataplane@{name}.service"), String::from("replace")).await.context("Failed to start unit")?;
    Ok(())
}

impl Dataplane {
    pub fn create_template_service() -> Result<(), Error> {
        let template = include_str!("templates/dataplane@.service");
        let filepath = Path::new("/etc/systemd/system/dataplane@.service");

        if filepath.is_file() {
            let mut buf = Vec::new();
            File::open(filepath)?.read_to_end(&mut buf)?;
            if String::from_utf8_lossy(&buf) == template {
                return Ok(());
            }
        }

        let mut service_template_file = File::create(filepath)
            .context(format!("Failed to create {}", filepath.display()))?;
        service_template_file.write_all(template.as_bytes())
            .context(format!("Failed to write to {}", filepath.display()))?;
        tokio::runtime::Runtime::new()?
            .block_on(reload_systemd())
            .context("Failed to reload systemd")?;
        Ok(())
    }

    pub fn enable_now(&self) -> Result<(), Error> {
        tokio::runtime::Runtime::new()?
            .block_on(enable_systemd_unit_now(&self.name))
            .context("Failed to enable and start dataplane unit")?;
        Ok(())
    }
}
