// Copyright © 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: (Apache-2.0 AND BSD-3-Clause)

//! System level device management.
//!
//! [DeviceManager](struct.DeviceManager.html) responds to manage all devices
//! of virtual machine, store basic device information like name and
//! parent bus, register IO resources callback, unregister devices and help
//! VM IO exit handling.

// NOTE: use enum VmResource.
extern crate vm_allocator;

use crate::device::*;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;
use std::result;
use std::sync::Arc;
use vm_allocator::VmResource;
use vm_memory::{GuestAddress, GuestUsize};

/// Guest physical address and size pair to describe a range.
#[derive(Eq, Debug, Copy, Clone)]
pub struct Range(pub GuestAddress, pub GuestUsize);

impl PartialEq for Range {
    fn eq(&self, other: &Range) -> bool {
        self.0 == other.0
    }
}

impl Ord for Range {
    fn cmp(&self, other: &Range) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Range) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Error type for `DeviceManager` usage.
#[derive(Debug)]
pub enum Error {
    /// The insertion failed because the new device overlapped with an old device.
    DeviceOverlap,
    /// The insertion failed because device already exists.
    DeviceExist,
    /// The removing fails because the device doesn't exist.
    DeviceNonExist,
}

/// Simplify the `Result` type.
pub type Result<T> = result::Result<T, Error>;

/// System device manager serving for all devices management and VM exit handling.
pub struct DeviceManager {
    /// Interrupt manager.
    irq_manager: Arc<Box<dyn InterruptManager>>,
    /// Devices information mapped by name.
    devices: HashMap<String, DeviceDescriptor>,
    /// Range mapping for VM exit mmio operations.
    mmio_bus: BTreeMap<Range, Arc<dyn Device>>,
    /// Range mapping for VM exit pio operations.
    pio_bus: BTreeMap<Range, Arc<dyn Device>>,
}

impl DeviceManager {
    /// Create a new `DeviceManager`.
    ///
    /// Passing on a `InterruptManager` which is
    /// used to manage interrupt resource group for devices.
    pub fn new(irq_manager: Arc<Box<dyn InterruptManager>>) -> Self {
        DeviceManager {
            irq_manager,
            devices: HashMap::new(),
            mmio_bus: BTreeMap::new(),
            pio_bus: BTreeMap::new(),
        }
    }

    fn insert(&mut self, dev: DeviceDescriptor) -> Result<()> {
        // Insert if the key is non-present, else report error.
        if self.devices.contains_key(&(dev.name)) {
            return Err(Error::DeviceExist);
        }
        self.devices.insert(dev.name.clone(), dev);
        Ok(())
    }

    fn remove(&mut self, name: String) -> Option<DeviceDescriptor> {
        self.devices.remove(&name)
    }

    fn device_descriptor(
        &self,
        id: u32,
        dev: Arc<dyn Device>,
        parent_bus: Option<Arc<dyn Device>>,
        resources: Vec<VmResource>,
    ) -> DeviceDescriptor {
        DeviceDescriptor::new(id, dev.name(), dev.clone(), parent_bus, resources)
    }

    // Create the corresponding interrupt group by the interrupt manager.
    // Return the failure case when fails, or else return instance id and interrupt source group.
    fn register_resources(&mut self, dev: Arc<dyn Device>, resources: &Vec<VmResource>) -> Result<(u32, Arc<Box<dyn InterruptSourceGroup>>)> {
        let mut instance_id = 0;
        let mut interrupt_group;

        // Register and mark device resources
        // The resources addresses being registered are sucessfully allocated before.
        for (idx, res) in resources.iter().enumerate() {
            match res {
                VmResource::Address(addr, size, ty) => {
                    match ty => {
                        IoType::Pio => {
                            if self
                                .pio_bus
                                .insert(Range(addr, size), dev.clone())
                                .is_some()
                            {
                                // Unregister and let VMM free resources.
                                if idx > 0 {
                                    self.unregister_resources(&resources[0..idx]);
                                }
                                return Err(Error::DeviceOverlap);
                            }
                        }
                        IoType::Mmio => {
                            if self
                                .mmio_bus
                                .insert(Range(addr, size), dev.clone())
                                .is_some()
                            {
                                // Unregister and let VMM free resources.
                                if idx > 0 {
                                    self.unregister_resources(&resources[0..idx]);
                                }
                                return Err(Error::DeviceOverlap);
                            }
                        IoType::PhysicalMmio => continue,
                    }
                }
                VmResource::Interrupt(ty, base, count) => {
                    // Create an interrupt group for corresponding type.
                    match self
                        .irq_manager
                        .create_group(ty, base, count) {
                        Ok((group)) => { let interrupt_group = group; },
                        Err(_) => {
                            // Unregister and let VMM free resources.
                            if idx > 0 {
                                self.unregister_resources(&resources[0..idx]);
                            }
                            return Error::IrqSrcGrpCreateError;
                        }
                    }
                }
                VmResource::Id(id) => {
                    instance_id = id;
                }
            }
        }
        Ok((id, interrupt_group))
    }

    /// Register a new device with its parent bus and resources.
    ///
    /// # Arguements
    ///
    /// * `dev`: device instance object to be registered
    /// * `parent_bus`: parent bus of the device
    /// * `resources`: resources that this device owns, might include instance id,
    ///                port I/O and memory-mapped I/O ranges, interrupt source.
    pub fn register_device(
        &mut self,
        dev: Arc<dyn Device>,
        parent_bus: Option<Arc<dyn Device>>,
        resources: &Vec<VmResource>,
    ) -> Result<()> {
        // Register the IO resource, get the instance id and interrupt source group.
        let (id, interrupt_group) = self.register_resources(dev.clone(), resources)?;

        // VMM: set the allocated resources back
        // dev.set_resources(resources);

        // Insert bus/device to DeviceManager with parent bus
        let descriptor = self.device_descriptor(id, dev, parent_bus, resources.to_vec(), interrupt_group);
        self.insert(descriptor)
    }

    // Unregister resources with all entries addresses valid.
    fn unregister_resources(&mut self, resources: &[VmResource]) {
        for res in resources.iter() {
            match res {
                VmResource::Address(addr, size, ty) => {
                    IoType::Pio => self.pio_bus.remove(&Range(addr, size)),
                    IoType::Mmio => self.mmio_bus.remove(&Range(addr, size)),
                    IoType::PhysicalMmio => continue,
                }
                VmResource::Id(_) | VmResource::Interrupt(_, _, _) => continue,
            };
        }
    }

    /// Unregister a device from `DeviceManager`.
    pub fn unregister_device(&mut self, dev: Arc<dyn Device>) -> Result<()> {
        if let Some(descriptor) = self.remove(dev.name()) {
            // Unregister resources
            self.unregister_resources(&descriptor.resources);
            // VMM: Free the resources
            // self.free_io_resources(&descriptor.resources);
            Ok(())
        } else {
            Err(Error::DeviceNonExist)
        }
    }
}
