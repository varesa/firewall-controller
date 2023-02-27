use anyhow::{Context, Error};
use firewall_controller::dataplane::{Dataplane, DataplaneList};
use netlink_packet_route::LinkMessage;
use rtnetlink::Handle;
use std::env;
use zbus::export::futures_util::TryStreamExt;

const HOST_NS_IFACE_NAME: &str = "host0";

async fn add_veth(handle: &Handle, name_a: &str, name_b: &str) -> Result<(), Error> {
    handle
        .link()
        .add()
        .veth(name_a.into(), name_b.into())
        .execute()
        .await
        .context("Failed to add link")?;
    Ok(())
}

async fn get_link(handle: &Handle, name: &str) -> Result<LinkMessage, Error> {
    handle
        .link()
        .get()
        .match_name(name.into())
        .execute()
        .try_next()
        .await
        .context("Failed to list links")?
        .ok_or_else(|| {
            return Error::msg(format!("Could not find link with name {}", name));
        })
}

async fn set_link_netns(handle: &Handle, index: u32, dp: &Dataplane) -> Result<(), Error> {
    handle
        .link()
        .set(index)
        .setns_by_fd(
            dp.get_pod()
                .context("Failed to get pod")?
                .get_infra_container()
                .context("Failed to get infra container")?
                .get_netns()
                .context("Failed to get netns")?
                .raw_fd(),
        )
        .execute()
        .await
        .context("Failed to switch netns")?;
    Ok(())
}

async fn add_link_altname(handle: &Handle, index: u32, altname: &str) -> Result<(), Error> {
    handle
        .link()
        .property_add(index)
        .alt_ifname(&[altname])
        .execute()
        .await
        .context("Failed to add if altname")?;
    Ok(())
}

async fn connect_dp_to_host(dp: &Dataplane) -> Result<(), Error> {
    let (connection, handle, _) =
        rtnetlink::new_connection().context("Failed to get rtnetlink connection")?;
    tokio::spawn(connection);

    let dp_if_name = format!("dp{}", dp.id);
    add_veth(&handle, &dp_if_name, HOST_NS_IFACE_NAME)
        .await
        .context("Failed to create veth pair")?;

    let veth_dp_end = get_link(&handle, HOST_NS_IFACE_NAME)
        .await
        .context("Failed to get DP link")?;
    let veth_host_end = get_link(&handle, &dp_if_name)
        .await
        .context("Failed to get host link")?;

    set_link_netns(&handle, veth_dp_end.header.index, dp)
        .await
        .context("Failed to set veth netns to DP")?;
    add_link_altname(
        &handle,
        veth_host_end.header.index,
        &format!("dp-{}", &dp.name),
    )
    .await
    .context("Failed to add DP name as altname to veth")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);

    let dp_list = DataplaneList::get().context("Unable to get dataplane list")?;
    let dp = dp_list
        .by_name(&args.remove(1))
        .context("Unable to get DP by name")?;

    let pod = dp
        .get_or_create_pod()
        .context("Failed to get or create DP pod")?;
    pod.ensure_is_running()
        .context("Failed to ensure pod is running")?;

    /*let netns = pod.get_infra_container()?.get_netns()?;
    let out = netns.run(|| Command::new("ip").arg("addr").output().unwrap().stdout);
    println!("{}", String::from_utf8(out).unwrap());*/

    //let host_netstate = NetState::retrieve().context("Failed to retrieve host netns state")?;
    let dp_netstate = pod.get_network_state().unwrap();
    if !dp_netstate.ifaces.contains_key(HOST_NS_IFACE_NAME) {
        connect_dp_to_host(dp)
            .await
            .context("Failed to connect DP to host netns")?;
    }
    //dbg!(dp_netstate);

    Ok(())
}
