// Copyright 2018 The Chromium OS Authors. All rights reserved.
// Copyright © 2019 Intel Corporation
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-BSD-3-Clause file.
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use crate::resource::*;
use crate::id::Id;

use std::collections::HashMap;
use std::result;

/// Errors associated with system resources allocation.
#[derive(Debug)]
pub enum Error {
    /// The allocator already exists.
    Exist,
}

pub type Result<T> = result::Result<T, Error>;

/// SystemAllocator contains different kinds of resources on demands of vmm.
///
/// vmm needs create a callback function and store it inside
/// vm-device::DeviceManager so it can be used to allocate each resource.
/// # Example
///
/// let allocate_cb = Arc::new(Box::new(sys: SystemAllocator, res: Box<IdResourceAllocator>) -> Result<Box<IdResourceAllocator>> {
///     sys.find_allocator(res.name()).allocate_id()
/// }
/// ));
/// vm-device::DeviceManager::assign_allocate_cb(allocate_cb);
///
#[derive(Default)]
pub struct SystemAllocator {
    // Different types of address as vmm request
    addr_alloc: HashMap<String, Box<AddrResourceAllocator>>,
    // Instance id and irq as different vmm request
    id_alloc: HashMap<String, Box<IdResourceAllocator>>,
}

impl SystemAllocator {
    pub fn new() -> Self {
        SystemAllocator {
            addr_alloc: HashMap::new(),
            id_alloc: HashMap::new(),
        }
    }

    pub fn add_addr_allocator(&mut self, allocator_name: String, allocator: Box<AddrResourceAllocator>) -> Result<()> {
        if self.addr_alloc.contains_key(&allocator_name) {
            return Err(Error::Exist);
        }
        self.addr_alloc.insert(allocator_name, allocator);
        Ok(())
    }

    pub fn add_id_allocator(&mut self, allocator_name: String, allocator: Box<IdResourceAllocator>) -> Result<()> {
        if self.id_alloc.contains_key(&allocator_name) {
            return Err(Error::Exist);
        }
        self.id_alloc.insert(allocator_name, allocator);
        Ok(())
    }

    pub fn allocate_id(&mut self, _allocator_id: String) -> Result<Box<Resource<V = u32>>> {

        Ok(Box::new(Id(10)))
    }
}

#[cfg(test)]
mod tests {
    use crate::system::{Error, SystemAllocator};
    use crate::id::{Id, IdAllocator};
    use crate::resource::IdResourceAllocator;

    #[test]
    fn test_allocate() -> Result<(), Error> {
        let mut sys = SystemAllocator::new();
        let id_allocator = IdAllocator::new(Id(1), Id(100)).ok_or(Error::Exist)?;
        let id_name = id_allocator.name();
        sys.add_id_allocator(id_name.clone(), Box::new(id_allocator))?;

        // Use sys to allocate an id
        let id = sys.allocate_id(id_name.clone())?;
        assert_eq!(id.raw_value(), 10);
        Ok(())
    }
}
