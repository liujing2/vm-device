// Copyright 2018 The Chromium OS Authors. All rights reserved.
// Copyright © 2019 Intel Corporation
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-BSD-3-Clause file.
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::result;

use crate::resource::*;

pub type Result<T> = result::Result<T, Error>;

/// Id
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Id(pub u32);


impl Resource for Id {
    type V = u32;

    fn name(&self) -> String {
        "id".to_string().clone()
    }

    fn raw_value(&self) -> u32 {
        self.0
    }
}

/// Manages allocating unsigned integer resources.
/// Use `IdAllocator` whenever a unique unsigned 32-bit number needs to be allocated.
///
/// #Arguments
///
/// * `start` - The starting number to manage.
/// * `end` - The ending number to manage.
/// * `used` - The used numbers ordered from lowest to highest.
///
/// # Examples
///
/// ```
/// # use vm_allocator::IdAllocator;
///   IdAllocator::new(1, std::u32::MAX).map(|mut p| {
///       assert_eq!(p.allocate(Some(1)).unwrap(), 1);
///       assert_eq!(p.allocate(Some(3)).unwrap(), 3);
/// });
///
/// ```
#[derive(Debug)]
pub struct IdAllocator {
    start: Id,
    end: Id,
    used: Vec<Id>,
}

impl IdAllocator {
    /// Creates a new `IdAllocator` for managing a range of unsigned integer.
    pub fn new(start: Id, end: Id) -> Option<Self> {
        Some(IdAllocator {
            start,
            end,
            used: Vec::new(),
        })
    }

    fn first_usable_number(&self) -> Result<Id> {
        if self.used.is_empty() {
            return Ok(self.start);
        }

        let mut previous = self.start;

        for iter in self.used.iter() {
            if *iter > previous {
                return Ok(previous);
            } else {
                match iter.0.checked_add(1) {
                    Some(p) => previous = Id(p),
                    None => return Err(Error::Overflow),
                }
            }
        }
        if previous <= self.end {
            Ok(previous)
        } else {
            Err(Error::Overflow)
        }
    }
}

impl IdResourceAllocator for IdAllocator {
    fn name(&self) -> String {
        "id".to_string().clone()
    }

    fn allocate(&mut self, resource: Option<Box<Resource<V=u32>>>) -> Result<Box<Resource<V=u32>>> {
        let ret = match resource {
            // Specified resource to be allocated.
            Some(res) => {
                // Check resource request type.
                if res.name() != self.name() {
                    return Err(Error::Invalid);
                }
                let value = res.raw_value() as u32;

                if value < self.start.raw_value() || value > self.end.raw_value() {
                    return Err(Error::OutofScope);
                }
                match self.used.iter().find(|&&x| x.raw_value() == value) {
                    Some(_) => {
                        return Err(Error::Duplicated);
                    }
                    None => Id(value),
                }
            }
            None => self.first_usable_number()?,
        };
        self.used.push(ret);
        self.used.sort();
        Ok(Box::new(ret))
    }

    /// Free an already allocated id and will keep the order.
    fn free(&mut self, res: Box<Resource<V=u32>>) {
        if let Ok(idx) = self.used.binary_search(&Id(res.raw_value())) {
            self.used.remove(idx);
        }
    }
}
