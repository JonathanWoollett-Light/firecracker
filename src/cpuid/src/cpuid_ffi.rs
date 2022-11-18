// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use std::alloc::Layout;
use std::cmp::{Eq, PartialEq};
use std::convert::TryFrom;
use std::fmt;
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use construct::Inline;
use serde::{Deserialize, Serialize};

/// Mimic of the currently unstable
/// [`Vec::into_raw_parts`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.into_raw_parts)
/// .
fn vec_into_raw_parts<T>(v: Vec<T>) -> (*mut T, usize, usize) {
    let mut me = std::mem::ManuallyDrop::new(v);
    (me.as_mut_ptr(), me.len(), me.capacity())
}

/// Error type for [`Cpuid::new`].
#[cfg(cpuid)]
#[derive(Debug, thiserror::Error)]
#[error("Could not access KVM: {0}")]
pub struct KvmGetSupportedRawCpuidError(#[from] utils::errno::Error);

/// A rusty mimic of
/// [`kvm_cpuid`](https://elixir.bootlin.com/linux/v5.10.129/source/arch/x86/include/uapi/asm/kvm.h#L226)
/// .
///
/// [`RawCpuid`] has an identical memory layout to
/// [`kvm_cpuid`](https://elixir.bootlin.com/linux/v5.10.129/source/arch/x86/include/uapi/asm/kvm.h#L226)
/// .
///
/// This allows [`RawCpuid`] to function as a simpler replacement for [`kvm_bindings::CpuId`]. In
/// the future it may replace [`kvm_bindings::CpuId`] fully.
///
/// For implementation details see <https://doc.rust-lang.org/nomicon/vec/vec.html>.
#[derive(Debug)]
#[repr(C)]
pub struct RawCpuid {
    /// Number of entries.
    nent: u32,
    /// Padding.
    padding: Padding<{ size_of::<u32>() }>,
    // TODO Use `std::ptr::Unqiue` when stabilized
    /// Pointer to entries.
    entries: NonNull<RawKvmCpuidEntry>,
    _marker: PhantomData<RawKvmCpuidEntry>,
}
impl RawCpuid {
    /// Alias for [`RawCpuid::default()`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns number of elements.
    #[must_use]
    pub fn nent(&self) -> u32 {
        self.nent
    }
    /// Returns an entry for a given lead (function) and sub-leaf (index).
    ///
    /// Returning `None` if it is not present.
    #[must_use]
    pub fn get(&self, leaf: u32, sub_leaf: u32) -> Option<&RawKvmCpuidEntry> {
        // TODO Would using binary search here for leaf offer much speedup?
        self.iter()
            .find(|entry| entry.function == leaf && entry.index == sub_leaf)
    }
    /// Resizes allocated memory
    #[allow(clippy::cast_ptr_alignment)]
    fn resize(&mut self, n: usize) {
        // alloc
        if self.nent == 0 && n > 0 {
            let new_layout = Layout::array::<RawKvmCpuidEntry>(n).unwrap();

            // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
            assert!(
                isize::try_from(new_layout.size()).is_ok(),
                "Allocation too large"
            );

            let new_ptr = unsafe { std::alloc::alloc(new_layout) };
            self.entries = match NonNull::new(new_ptr.cast::<RawKvmCpuidEntry>()) {
                Some(p) => p,
                None => std::alloc::handle_alloc_error(new_layout),
            };
        }
        // realloc
        else if self.nent > 0 && n > 0 {
            let new_layout = Layout::array::<RawKvmCpuidEntry>(n).unwrap();

            // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
            assert!(
                isize::try_from(new_layout.size()).is_ok(),
                "Allocation too large"
            );

            let old_layout =
                Layout::array::<RawKvmCpuidEntry>(usize::try_from(self.nent).unwrap()).unwrap();
            let old_ptr = self.entries.as_ptr().cast::<u8>();
            let new_ptr = unsafe { std::alloc::realloc(old_ptr, old_layout, new_layout.size()) };

            self.entries = match NonNull::new(new_ptr.cast::<RawKvmCpuidEntry>()) {
                Some(p) => p,
                None => std::alloc::handle_alloc_error(new_layout),
            };
        }
        // dealloc
        else if self.nent > 0 && n == 0 {
            let old_layout =
                Layout::array::<RawKvmCpuidEntry>(usize::try_from(self.nent).unwrap()).unwrap();
            let old_ptr = self.entries.as_ptr().cast::<u8>();
            unsafe { std::alloc::dealloc(old_ptr, old_layout) };
            self.entries = NonNull::dangling();
        }
        self.nent = u32::try_from(n).unwrap();
    }

    /// Pushes entry onto end.
    ///
    /// # Panics
    ///
    /// On allocation failure.
    pub fn push(&mut self, entry: RawKvmCpuidEntry) {
        self.resize(usize::try_from(self.nent).unwrap() + 1);
        unsafe {
            std::ptr::write(
                self.entries
                    .as_ptr()
                    .add(usize::try_from(self.nent).unwrap()),
                entry,
            );
        }
    }
    /// Pops entry from end.
    ///
    /// # Panics
    ///
    /// On allocation failure.
    pub fn pop(&mut self) -> Option<RawKvmCpuidEntry> {
        if self.nent > 0 {
            let u_nent = usize::try_from(self.nent).unwrap();
            let rtn = unsafe { Some(std::ptr::read(self.entries.as_ptr().add(u_nent))) };
            self.resize(u_nent - 1);
            rtn
        } else {
            None
        }
    }

    /// Gets supported CPUID by KVM.
    ///
    /// # Errors
    ///
    /// When failed to access KVM.
    #[cfg(cpuid)]
    pub fn kvm_get_supported_cpuid() -> std::result::Result<Self, KvmGetSupportedRawCpuidError> {
        let supported_kvm_cpuid =
            kvm_ioctls::Kvm::new()?.get_supported_cpuid(kvm_bindings::KVM_MAX_CPUID_ENTRIES)?;
        Ok(RawCpuid::from(supported_kvm_cpuid))
    }
}
// TODO Make this more efficient.
impl Clone for RawCpuid {
    fn clone(&self) -> Self {
        let mut new_raw_cpuid = Self::new();
        new_raw_cpuid.resize(self.nent as usize);
        for i in 0..self.nent as usize {
            new_raw_cpuid[i] = self[i].clone();
        }
        new_raw_cpuid
    }
}
impl serde::Serialize for RawCpuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.nent as usize))?;
        for i in 0..self.nent as usize {
            seq.serialize_element(&self[i])?;
        }
        seq.end()
    }
}
struct RawCpuidVisitor;
impl<'de> serde::de::Visitor<'de> for RawCpuidVisitor {
    type Value = RawCpuid;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Expected sequence of RawKvmCpuidEntry")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut entries = Vec::new();
        while let Some(next) = seq.next_element::<RawKvmCpuidEntry>()? {
            entries.push(next);
        }
        Ok(Self::Value::from(entries))
    }
}
impl<'de> serde::Deserialize<'de> for RawCpuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(RawCpuidVisitor)
    }
}
impl PartialEq for RawCpuid {
    fn eq(&self, other: &Self) -> bool {
        if self.nent == other.nent {
            for i in 0..self.nent as usize {
                if self[i] != other[i] {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}
impl Eq for RawCpuid {}
unsafe impl Send for RawCpuid {}
unsafe impl Sync for RawCpuid {}
impl Inline for RawCpuid {
    fn inline(&self) -> construct::TokenStream {
        let nent = self.nent;
        let item = (0..self.nent).map(|i| self[i as usize].inline());
        let i = 0..self.nent as usize;
        construct::quote! {
            let mut cpuid = Self::new();
            cpuid.resize(#nent);
            #(
                cpuid[#i] = #item;
            )*
            cpuid
        }
    }
}
impl Default for RawCpuid {
    fn default() -> Self {
        Self {
            nent: 0,
            padding: Padding::default(),
            entries: NonNull::dangling(),
            _marker: PhantomData,
        }
    }
}
// We implement custom drop which drops all entries using `self.nent`
impl Drop for RawCpuid {
    fn drop(&mut self) {
        // TODO Is this unsafe when using `RawCpuid::default()`?
        if self.nent != 0 {
            unsafe {
                std::alloc::dealloc(
                    self.entries.as_ptr().cast::<u8>(),
                    Layout::array::<RawKvmCpuidEntry>(usize::try_from(self.nent).unwrap()).unwrap(),
                );
            }
        }
    }
}
impl Deref for RawCpuid {
    type Target = [RawKvmCpuidEntry];
    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.entries.as_ptr(), usize::try_from(self.nent).unwrap())
        }
    }
}
impl DerefMut for RawCpuid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.entries.as_ptr(),
                usize::try_from(self.nent).unwrap(),
            )
        }
    }
}
#[cfg(cpuid)]
impl From<kvm_bindings::CpuId> for RawCpuid {
    fn from(value: kvm_bindings::CpuId) -> Self {
        // As cannot acquire ownership of the underlying slice, we clone it.
        let cloned = value.as_slice().to_vec();
        let (ptr, len, _cap) = vec_into_raw_parts(cloned);
        Self {
            nent: u32::try_from(len).unwrap(),
            padding: Padding::default(),
            entries: NonNull::new(ptr.cast::<RawKvmCpuidEntry>()).unwrap(),
            _marker: PhantomData,
        }
    }
}
impl From<Vec<RawKvmCpuidEntry>> for RawCpuid {
    fn from(vec: Vec<RawKvmCpuidEntry>) -> Self {
        let (ptr, len, _cap) = vec_into_raw_parts(vec);
        Self {
            nent: u32::try_from(len).unwrap(),
            padding: Padding::default(),
            entries: NonNull::new(ptr.cast::<RawKvmCpuidEntry>()).unwrap(),
            _marker: PhantomData,
        }
    }
}
#[cfg(cpuid)]
impl From<RawCpuid> for kvm_bindings::CpuId {
    fn from(this: RawCpuid) -> Self {
        let cpuid_slice = unsafe {
            std::slice::from_raw_parts(this.entries.as_ptr(), usize::try_from(this.nent).unwrap())
        };
        // println!("cpuid_slice: {:?}",cpuid_slice);
        #[allow(clippy::transmute_ptr_to_ptr)]
        let kvm_bindings_slice = unsafe { std::mem::transmute(cpuid_slice) };
        kvm_bindings::CpuId::from_entries(kvm_bindings_slice).unwrap()
    }
}

/// A structure for owning unused memory for padding.
///
/// A wrapper around an uninitialized `N` element array of `u8`s (`MaybeUninit<[u8;N]>` constructed
/// with `Self(MaybeUninit::uninit())`).
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Padding<const N: usize>(MaybeUninit<[u8; N]>);
impl<const N: usize> Default for Padding<N> {
    fn default() -> Self {
        Self(MaybeUninit::uninit())
    }
}
impl<const N: usize> serde::Serialize for Padding<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_struct("Padding")
    }
}
impl<'de, const N: usize> serde::Deserialize<'de> for Padding<N> {
    fn deserialize<D>(_deserializer: D) -> Result<Padding<N>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Padding(MaybeUninit::uninit()))
    }
}
impl<const N: usize> PartialEq for Padding<N> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<const N: usize> Eq for Padding<N> {}

impl<const N: usize> Inline for Padding<N> {
    fn inline(&self) -> construct::TokenStream {
        construct::quote! { Self::default() }
    }
}

// Definitions from `kvm/arch/x86/include/uapi/asm/kvm.h
#[rustfmt::skip]
bit_fields::bitfield!(KvmCpuidFlags, u32, {
    /// Indicates if the `index` field is used for indexing sub-leaves (if false, this CPUID leaf
    /// has no subleaves).
    significant_index: 0,
    /// Depcrecated.
    stateful_func: 1,
    /// Depcrecated.
    state_read_next: 2,
});

/// CPUID entry (a mimic of <https://elixir.bootlin.com/linux/v5.10.129/source/arch/x86/include/uapi/asm/kvm.h#L232>).
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Inline)]
#[repr(C)]
pub struct RawKvmCpuidEntry {
    /// CPUID function (leaf).
    pub function: u32,
    /// CPUID index (subleaf).
    pub index: u32,
    /// KVM CPUID flags.
    pub flags: KvmCpuidFlags,
    /// EAX register.
    pub eax: u32,
    /// EBX register.
    pub ebx: u32,
    /// ECX register.
    pub ecx: u32,
    /// EDX register.
    pub edx: u32,
    /// CPUID entry padding.
    pub padding: Padding<{ size_of::<[u32; 3]>() }>,
}
impl fmt::LowerHex for RawKvmCpuidEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawKvmCpuidEntry")
            .field("function", &format!("{:x}", self.function))
            .field("index", &format!("{:x}", self.index))
            .field("flags", &format!("{:x}", self.flags.0))
            .field("eax", &format!("{:x}", self.eax))
            .field("ebx", &format!("{:x}", self.ebx))
            .field("ecx", &format!("{:x}", self.ecx))
            .field("edx", &format!("{:x}", self.edx))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(cpuid)]
    use kvm_bindings::KVM_MAX_CPUID_ENTRIES;

    #[cfg(cpuid)]
    use super::*;

    #[cfg(cpuid)]
    #[test]
    fn kvm_get_supported_cpuid() {
        let kvm = kvm_ioctls::Kvm::new().unwrap();
        let kvm_cpuid = kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES).unwrap();
        let raw_cpuid = RawCpuid::from(kvm_cpuid);

        assert_eq!(raw_cpuid, RawCpuid::kvm_get_supported_cpuid().unwrap());
    }
    #[cfg(cpuid)]
    #[test]
    fn kvm_set_cpuid() {
        let kvm = kvm_ioctls::Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();
        let vcpu = vm.create_vcpu(0).unwrap();
        let kvm_cpuid = kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES).unwrap();

        println!("kvm_cpuid:");
        for x in kvm_cpuid.as_slice() {
            println!("\t{:?}", x);
        }

        let cpuid = RawCpuid::from(kvm_cpuid.clone());
        println!("cpuid:");
        for x in cpuid.iter() {
            println!("\t{:?}", x);
        }

        let kvm_cpuid_2 = kvm_bindings::CpuId::from(cpuid);
        println!("kvm_cpuid_2:");
        for x in kvm_cpuid_2.as_slice() {
            println!("\t{:?}", x);
        }
        assert_eq!(kvm_cpuid.as_slice(), kvm_cpuid_2.as_slice());

        vcpu.set_cpuid2(&kvm_cpuid_2).unwrap();

        let kvm_cpuid_3 = vcpu.get_cpuid2(KVM_MAX_CPUID_ENTRIES).unwrap();
        println!("kvm_cpuid_3:");
        for x in kvm_cpuid_3.as_slice() {
            println!("\t{:?}", x);
        }
    }
    #[cfg(cpuid)]
    #[test]
    fn between_kvm() {
        let kvm = kvm_ioctls::Kvm::new().unwrap();
        let kvm_cpuid = kvm
            .get_supported_cpuid(kvm_bindings::KVM_MAX_CPUID_ENTRIES)
            .unwrap();
        let raw_cpuid = RawCpuid::from(kvm_cpuid.clone());
        let kvm_cpuid_2 = kvm_bindings::CpuId::from(raw_cpuid);

        assert_eq!(kvm_cpuid.as_slice(), kvm_cpuid_2.as_slice());
    }
    #[cfg(cpuid)]
    #[test]
    fn clone() {
        let kvm = kvm_ioctls::Kvm::new().unwrap();
        let kvm_cpuid = kvm
            .get_supported_cpuid(kvm_bindings::KVM_MAX_CPUID_ENTRIES)
            .unwrap();
        let raw_cpuid = RawCpuid::from(kvm_cpuid);
        let cloned = raw_cpuid.clone();

        assert_eq!(raw_cpuid, cloned);
    }
}
