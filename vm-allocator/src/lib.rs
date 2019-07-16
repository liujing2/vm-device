// Copyright 2018 The Chromium OS Authors. All rights reserved.
// Copyright © 2019 Intel Corporation
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-BSD-3-Clause file.
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//#![deny(missing_docs)]

//! Manages system resources that can be allocated to VMs and their devices.
#![deny(missing_docs)]

extern crate libc;

mod resource;

pub use crate::resource::{
    Error as ResourceAllocatorError, Resource, ResourceAllocator, ResourceSize,
};
