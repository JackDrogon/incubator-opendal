// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use ::opendal as od;

/// The [`OperatorPtr`] owns a pointer to a [`od::BlockingOperator`].
/// It is also the key struct that OpenDAL's APIs access the real
/// operator's memory. The use of OperatorPtr is zero cost, it
/// only returns a reference of the underlying Operator.
///
/// The [`OperatorPtr`] also has a transparent layout, allowing you
/// to check its validity by native boolean operator.
/// e.g. you could check by (!ptr) on a opendal_operator_ptr type
#[repr(transparent)]
pub struct opendal_operator_ptr {
    ptr: *const od::BlockingOperator,
}

impl opendal_operator_ptr {
    /// Creates an OperatorPtr will nullptr, indicating this [`OperatorPtr`]
    /// is invalid. The `transparent` layout also guarantees that if the
    /// underlying field `ptr` is a nullptr, the [`OperatorPtr`] has the
    /// same layout as the nullptr.
    pub(crate) fn null() -> Self {
        Self {
            ptr: std::ptr::null(),
        }
    }

    /// Returns whether this points to NULL
    pub(crate) fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Returns a reference to the underlying [`BlockingOperator`]
    pub(crate) fn get_ref(&self) -> &od::BlockingOperator {
        unsafe { &*(self.ptr) }
    }

    /// Returns a mutable reference to the underlying [`od::BlockingOperator`].
    /// Note that this should be only used when the operator is being freed
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_ref_mut(&self) -> &mut od::BlockingOperator {
        let ptr_mut = self.ptr as *mut od::BlockingOperator;
        unsafe { &mut (*ptr_mut) }
    }
}

#[allow(clippy::from_over_into)]
impl From<&od::BlockingOperator> for opendal_operator_ptr {
    fn from(value: &od::BlockingOperator) -> Self {
        Self { ptr: value }
    }
}

#[allow(clippy::from_over_into)]
impl From<&mut od::BlockingOperator> for opendal_operator_ptr {
    fn from(value: &mut od::BlockingOperator) -> Self {
        Self { ptr: value }
    }
}

/// The [`Bytes`] type is a C-compatible substitute for [`Bytes`]
/// in Rust, it will not be deallocated automatically like what
/// has been done in Rust. Instead, you have to call [`opendal_free_bytes`]
/// to free the heap memory to avoid memory leak.
/// The field `data` should not be modified since it might causes
/// the reallocation of the Vector.
#[repr(C)]
pub struct opendal_bytes {
    pub data: *const u8,
    pub len: usize,
}

impl opendal_bytes {
    /// Construct a [`Vector`] from the Rust [`Vec`] of bytes
    pub(crate) fn from_vec(vec: Vec<u8>) -> Self {
        let data = vec.as_ptr() as *const u8;
        let len = vec.len();
        std::mem::forget(vec); // To avoid deallocation of the vec.
        Self { data, len }
    }
}

#[allow(clippy::from_over_into)]
impl Into<bytes::Bytes> for opendal_bytes {
    fn into(self) -> bytes::Bytes {
        let slice = unsafe { std::slice::from_raw_parts(self.data, self.len) };
        bytes::Bytes::from_static(slice)
    }
}

/// Frees the heap memory used by the [`opendal_bytes`]
#[no_mangle]
pub extern "C" fn opendal_bytes_free(vec: *const opendal_bytes) {
    unsafe {
        // this deallocates the vector by reconstructing the vector and letting
        // it be dropped when its out of scope
        Vec::from_raw_parts((*vec).data as *mut u8, (*vec).len, (*vec).len);
    }
}