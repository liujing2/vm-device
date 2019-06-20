// Copyright 2018 The Chromium OS Authors. All rights reserved.
// Copyright © 2019 Intel Corporation
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-BSD-3-Clause file.
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::result;

#[derive(Debug)]
pub enum Error {
    Invalid,
    Overflow,
    Duplicated,
}

pub type Result<T> = result::Result<T, Error>;

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
    start: u32,
    end: u32,
    used: Vec<u32>,
}

impl IdAllocator {
    /// Creates a new `IdAllocator` for managing a range of unsigned integer.
    pub fn new(start: u32, end: u32) -> Option<Self> {
        Some(IdAllocator {
            start,
            end,
            used: Vec::new(),
        })
    }

    fn first_usable_number(&self) -> Option<u32> {
        if self.used.is_empty() {
            return Some(self.start);
        }

        let mut previous = self.start;

        for iter in self.used.iter() {
            if *iter > previous {
                return Some(previous);
            } else {
                match iter.checked_add(1) {
                    Some(p) => previous = p,
                    None => return None,
                }
            }
        }
        if previous <= self.end {
            Some(previous)
        } else {
            None
        }
    }

    /// Allocates a number from the managed region. Returns `Ok(allocated_id)`
    /// when successful, or Error indicates the failure reason.
    pub fn allocate(&mut self, number: Option<u32>) -> Result<u32> {
        let new = match number {
            // Specified number to be allocated.
            Some(num) => {
                if num < self.start || num > self.end {
                    return Err(Error::Invalid);
                }
                match self.used.iter().find(|&&x| x == num) {
                    Some(_) => {
                        return Err(Error::Duplicated);
                    }
                    None => num,
                }
            }
            None => self.first_usable_number().ok_or(Error::Overflow)?,
        };
        self.used.push(new);
        self.used.sort();
        Ok(new)
    }

    /// Free an already allocated id and will keep the order.
    pub fn free(&mut self, number: u32) {
        if let Ok(idx) = self.used.binary_search(&number) {
            self.used.remove(idx);
        }
    }
}
