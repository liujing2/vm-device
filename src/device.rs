// Copyright © 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: (Apache-2.0 AND BSD-3-Clause)

//! Handles routing to devices in an address space.
use std::string::String;
use std::sync::Arc;
use vm_memory::{GuestAddress, GuestUsize};

/// Trait for devices with basic functions.
#[allow(unused_variables)]
pub trait Device: Send {
    /// Get the device name.
    fn name(&self) -> String;
    /// Read from the guest physical address `addr` to `data`.
    fn read(&self, addr: GuestAddress, data: &mut [u8], io_type: IoType);
    /// Write `data` to the guest physical address `addr`.
    fn write(&self, addr: GuestAddress, data: &[u8], io_type: IoType);
    /// Set the allocated resource to device.
    ///
    /// This will be called by DeviceManager::register_device() to set
    /// the allocated resource from the vm_allocator back to device.
    fn set_resources(&self, res: &[Resource]);
}

/// Resource type.
#[derive(Debug, Copy, Clone)]
pub enum IoType {
    /// Port I/O resource.
    Pio,
    /// Memory I/O resource.
    Mmio,
    /// Non-exit physically backed mmap IO
    PhysicalMmio,
}

/// Storing Device information and for topology managing by name.
pub struct DeviceDescriptor {
    /// Device name.
    pub name: String,
    /// The device to descript.
    pub device: Arc<dyn Device>,
    /// The parent bus of this device.
    pub parent_bus: Option<Arc<dyn Device>>,
    /// Device resource set.
    pub resources: Vec<VmResource>,
    /// Interrupt source group.
    pub irq_group: Arc<Box<InterruptGroup>>,
}

impl DeviceDescriptor {
    /// Create a descriptor for one device.
    pub fn new(
        name: String,
        dev: Arc<dyn Device>,
        parent_bus: Option<Arc<dyn Device>>,
        resources: Vec<VmResource>,
        irq_group: Arc<Box<InterruptGroup>>,
    ) -> Self {
        DeviceDescriptor {
            name,
            device: dev,
            parent_bus,
            resources,
            irq_group,
        }
    }
}
