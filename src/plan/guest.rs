use std::net::Ipv4Addr;

use anyhow::{Context, Result};

use crate::model::{ConfigBundle, GuestAddressMode, GuestKind, GuestPciDevice, GuestVirtioFs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGuest {
    pub name: String,
    pub kind: GuestKind,
    pub host: String,
    pub vmid: u32,
    pub mac: String,
    pub interface: String,
    pub bridge: String,
    pub ipv4: Ipv4Addr,
    pub address_mode: GuestAddressMode,
    pub onboot: bool,
    pub startup: Option<String>,
    pub pci_devices: Vec<GuestPciDevice>,
    pub virtiofs: Vec<GuestVirtioFs>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedGuestPlan {
    pub location: String,
    pub guests: Vec<ResolvedGuest>,
}

impl ResolvedGuestPlan {
    pub fn print(&self) {
        println!("Guest plan for '{}':", self.location);
        for guest in &self.guests {
            println!(
                "{}\t{:?}\thost={}\tvmid={}\tbridge={}\tmac={}\tipv4={}\tmode={:?}",
                guest.name,
                guest.kind,
                guest.host,
                guest.vmid,
                guest.bridge,
                guest.mac,
                guest.ipv4,
                guest.address_mode,
            );
        }
    }
}

pub fn resolve_guest_plan(bundle: &ConfigBundle) -> Result<ResolvedGuestPlan> {
    let mut guests = Vec::new();

    for (name, guest) in &bundle.guests.guests {
        let parent = bundle.inventory.hosts.get(&guest.host).with_context(|| {
            format!(
                "guest '{name}' references unknown parent host '{}'",
                guest.host
            )
        })?;
        if !parent.enabled {
            continue;
        }

        let location_host = bundle.location.hosts.get(&guest.host).with_context(|| {
            format!(
                "guest '{name}' parent '{}' is not attached at location '{}'",
                guest.host, bundle.location.name
            )
        })?;
        let interface = location_host
            .interfaces
            .get(&guest.interface)
            .with_context(|| {
                format!(
                    "guest '{name}' references unknown location interface '{}:{}'",
                    guest.host, guest.interface
                )
            })?;
        let segment = &bundle.location.segments[&interface.segment];
        if !segment.ipv4_cidr.contains(&guest.ipv4) {
            anyhow::bail!(
                "guest '{name}' address {} is outside segment '{}' ({})",
                guest.ipv4,
                interface.segment,
                segment.ipv4_cidr
            );
        }

        guests.push(ResolvedGuest {
            name: name.clone(),
            kind: guest.kind,
            host: guest.host.clone(),
            vmid: guest.vmid,
            mac: guest.mac.clone(),
            interface: guest.interface.clone(),
            bridge: guest.bridge.clone(),
            ipv4: guest.ipv4,
            address_mode: guest.address_mode,
            onboot: guest.onboot,
            startup: guest.startup.clone(),
            pci_devices: guest.pci_devices.clone(),
            virtiofs: guest.virtiofs.clone(),
        });
    }

    guests.sort_by_key(|guest| (guest.host.clone(), guest.vmid, guest.name.clone()));
    Ok(ResolvedGuestPlan {
        location: bundle.location.name.clone(),
        guests,
    })
}
