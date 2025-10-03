//! Atomic type for lock-free access to primitives

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize};
use std::sync::atomic::{AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize};
use std::sync::atomic::AtomicBool;
// Import Ordering privately for internal use
use std::sync::atomic::Ordering as OrderingInternal;

/// Atomic<T> - Lock-free access for primitive types
/// Usage: Atomic<u64>, Atomic<f32>, etc.
pub enum Atomic<T> {
    _Phantom(std::marker::PhantomData<T>),
    U8(Arc<AtomicU8>),
    U16(Arc<AtomicU16>),
    U32(Arc<AtomicU32>),
    U64(Arc<AtomicU64>),
    Usize(Arc<AtomicUsize>),
    I8(Arc<AtomicI8>),
    I16(Arc<AtomicI16>),
    I32(Arc<AtomicI32>),
    I64(Arc<AtomicI64>),
    Isize(Arc<AtomicIsize>),
    Bool(Arc<AtomicBool>),
    F32(Arc<AtomicU32>), // f32 stored as u32
    F64(Arc<AtomicU64>), // f64 stored as u64
}

// Macro to implement Atomic for integer types
macro_rules! impl_atomic_int {
    ($type:ty, $variant:ident, $atomic:ty) => {
        impl Atomic<$type> {
            pub fn new(value: $type) -> Self {
                Atomic::$variant(Arc::new(<$atomic>::new(value)))
            }

            pub fn load(&self) -> $type {
                match self {
                    Atomic::$variant(a) => a.load(OrderingInternal::SeqCst),
                    _ => unreachable!(),
                }
            }

            pub fn store(&self, value: $type) {
                match self {
                    Atomic::$variant(a) => a.store(value, OrderingInternal::SeqCst),
                    _ => unreachable!(),
                }
            }

            pub fn swap(&self, value: $type) -> $type {
                match self {
                    Atomic::$variant(a) => a.swap(value, OrderingInternal::SeqCst),
                    _ => unreachable!(),
                }
            }

            pub fn compare_exchange(&self, current: $type, new: $type) -> Result<$type, $type> {
                match self {
                    Atomic::$variant(a) => a.compare_exchange(current, new, OrderingInternal::SeqCst, OrderingInternal::SeqCst),
                    _ => unreachable!(),
                }
            }

            pub fn fetch_add(&self, val: $type) -> $type {
                match self {
                    Atomic::$variant(a) => a.fetch_add(val, OrderingInternal::SeqCst),
                    _ => unreachable!(),
                }
            }

            pub fn fetch_sub(&self, val: $type) -> $type {
                match self {
                    Atomic::$variant(a) => a.fetch_sub(val, OrderingInternal::SeqCst),
                    _ => unreachable!(),
                }
            }
        }

        impl Clone for Atomic<$type> {
            fn clone(&self) -> Self {
                match self {
                    Atomic::$variant(a) => Atomic::$variant(Arc::clone(a)),
                    _ => unreachable!(),
                }
            }
        }
    };
}

// Implement for all integer types
impl_atomic_int!(u8, U8, AtomicU8);
impl_atomic_int!(u16, U16, AtomicU16);
impl_atomic_int!(u32, U32, AtomicU32);
impl_atomic_int!(u64, U64, AtomicU64);
impl_atomic_int!(usize, Usize, AtomicUsize);
impl_atomic_int!(i8, I8, AtomicI8);
impl_atomic_int!(i16, I16, AtomicI16);
impl_atomic_int!(i32, I32, AtomicI32);
impl_atomic_int!(i64, I64, AtomicI64);
impl_atomic_int!(isize, Isize, AtomicIsize);

// Special implementation for bool
impl Atomic<bool> {
    pub fn new(value: bool) -> Self {
        Atomic::Bool(Arc::new(AtomicBool::new(value)))
    }

    pub fn load(&self) -> bool {
        match self {
            Atomic::Bool(a) => a.load(OrderingInternal::SeqCst),
            _ => unreachable!(),
        }
    }

    pub fn store(&self, value: bool) {
        match self {
            Atomic::Bool(a) => a.store(value, OrderingInternal::SeqCst),
            _ => unreachable!(),
        }
    }

    pub fn swap(&self, value: bool) -> bool {
        match self {
            Atomic::Bool(a) => a.swap(value, OrderingInternal::SeqCst),
            _ => unreachable!(),
        }
    }

    pub fn compare_exchange(&self, current: bool, new: bool) -> Result<bool, bool> {
        match self {
            Atomic::Bool(a) => a.compare_exchange(current, new, OrderingInternal::SeqCst, OrderingInternal::SeqCst),
            _ => unreachable!(),
        }
    }
}

impl Clone for Atomic<bool> {
    fn clone(&self) -> Self {
        match self {
            Atomic::Bool(a) => Atomic::Bool(Arc::clone(a)),
            _ => unreachable!(),
        }
    }
}

// Special implementation for f32 (using AtomicU32)
impl Atomic<f32> {
    pub fn new(value: f32) -> Self {
        Atomic::F32(Arc::new(AtomicU32::new(value.to_bits())))
    }

    pub fn load(&self) -> f32 {
        match self {
            Atomic::F32(a) => f32::from_bits(a.load(OrderingInternal::SeqCst)),
            _ => unreachable!(),
        }
    }

    pub fn store(&self, value: f32) {
        match self {
            Atomic::F32(a) => a.store(value.to_bits(), OrderingInternal::SeqCst),
            _ => unreachable!(),
        }
    }

    pub fn swap(&self, value: f32) -> f32 {
        match self {
            Atomic::F32(a) => f32::from_bits(a.swap(value.to_bits(), OrderingInternal::SeqCst)),
            _ => unreachable!(),
        }
    }

    pub fn compare_exchange(&self, current: f32, new: f32) -> Result<f32, f32> {
        match self {
            Atomic::F32(a) => a.compare_exchange(
                current.to_bits(),
                new.to_bits(),
                OrderingInternal::SeqCst,
                OrderingInternal::SeqCst,
            ).map(f32::from_bits).map_err(f32::from_bits),
            _ => unreachable!(),
        }
    }
}

impl Clone for Atomic<f32> {
    fn clone(&self) -> Self {
        match self {
            Atomic::F32(a) => Atomic::F32(Arc::clone(a)),
            _ => unreachable!(),
        }
    }
}

// Special implementation for f64 (using AtomicU64)
impl Atomic<f64> {
    pub fn new(value: f64) -> Self {
        Atomic::F64(Arc::new(AtomicU64::new(value.to_bits())))
    }

    pub fn load(&self) -> f64 {
        match self {
            Atomic::F64(a) => f64::from_bits(a.load(OrderingInternal::SeqCst)),
            _ => unreachable!(),
        }
    }

    pub fn store(&self, value: f64) {
        match self {
            Atomic::F64(a) => a.store(value.to_bits(), OrderingInternal::SeqCst),
            _ => unreachable!(),
        }
    }

    pub fn swap(&self, value: f64) -> f64 {
        match self {
            Atomic::F64(a) => f64::from_bits(a.swap(value.to_bits(), OrderingInternal::SeqCst)),
            _ => unreachable!(),
        }
    }

    pub fn compare_exchange(&self, current: f64, new: f64) -> Result<f64, f64> {
        match self {
            Atomic::F64(a) => a.compare_exchange(
                current.to_bits(),
                new.to_bits(),
                OrderingInternal::SeqCst,
                OrderingInternal::SeqCst,
            ).map(f64::from_bits).map_err(f64::from_bits),
            _ => unreachable!(),
        }
    }
}

impl Clone for Atomic<f64> {
    fn clone(&self) -> Self {
        match self {
            Atomic::F64(a) => Atomic::F64(Arc::clone(a)),
            _ => unreachable!(),
        }
    }
}

// Helper function to create atomic values
pub fn atomic<T>(value: T) -> Atomic<T>
where
    Atomic<T>: From<T>,
{
    Atomic::from(value)
}

// Implement From for convenience
impl From<u8> for Atomic<u8> { fn from(v: u8) -> Self { Self::new(v) } }
impl From<u16> for Atomic<u16> { fn from(v: u16) -> Self { Self::new(v) } }
impl From<u32> for Atomic<u32> { fn from(v: u32) -> Self { Self::new(v) } }
impl From<u64> for Atomic<u64> { fn from(v: u64) -> Self { Self::new(v) } }
impl From<usize> for Atomic<usize> { fn from(v: usize) -> Self { Self::new(v) } }
impl From<i8> for Atomic<i8> { fn from(v: i8) -> Self { Self::new(v) } }
impl From<i16> for Atomic<i16> { fn from(v: i16) -> Self { Self::new(v) } }
impl From<i32> for Atomic<i32> { fn from(v: i32) -> Self { Self::new(v) } }
impl From<i64> for Atomic<i64> { fn from(v: i64) -> Self { Self::new(v) } }
impl From<isize> for Atomic<isize> { fn from(v: isize) -> Self { Self::new(v) } }
impl From<bool> for Atomic<bool> { fn from(v: bool) -> Self { Self::new(v) } }
impl From<f32> for Atomic<f32> { fn from(v: f32) -> Self { Self::new(v) } }
impl From<f64> for Atomic<f64> { fn from(v: f64) -> Self { Self::new(v) } }

// Re-export Ordering directly from std
pub use std::sync::atomic::Ordering;
