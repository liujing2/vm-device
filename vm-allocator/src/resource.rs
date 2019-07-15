// Copyright 2018 The Chromium OS Authors. All rights reserved.
// Copyright © 2019 Intel Corporation
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-BSD-3-Clause file.
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::fmt::{self, Display};
use vm_memory::GuestAddress;

#[derive(Debug)]
pub enum Error {
    OutofScope,
    Overflow,
    Duplicated,
    Invalid,
}

impl Display for Error {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            OutofScope => write!(f, "Resource being allocated is out of scope"),
            Overflow => write!(f, "Resource being allocated is overflow"),
            Duplicated => write!(f, "Resource being allocated is duplicated"),
            Invalid => write!(f, "Resource request is invalid"),
        }
    }
}

pub trait Resource
{
    /// Type of the resource raw value.
    type V;
    /// Get the name describing the resource type.
    fn name(&self) -> String;

    fn raw_value(&self) -> Self::V;
}

//TODO: we should specify 'V' when using Resource trait.
// So the two traits can not be as a common one.

/// Unsigned integer allocator
pub trait IdResourceAllocator {
    fn name(&self) -> String; 

    /// Unsigned integer resource allocation.
    fn allocate(
        &mut self,
        res: Option<Box<Resource<V = u32>>>,
   ) -> Result<Box<Resource<V = u32>>, Error>;

    /// Unsigned integer resource free.
    fn free(&mut self, res: Box<Resource<V = u32>>);
}

pub trait AddrResourceAllocator {
    fn name(&self) -> String; 

    /// Address resource allocation.
    fn allocate(
        &mut self,
        res: Option<Box<Resource<V = GuestAddress>>>,
    ) -> Result<Box<Resource<V = GuestAddress>>, Error>;

    /// Address resource free.
    fn free(&mut self, res: Box<Resource<V = GuestAddress>>);
}
