use color_eyre::Result;
use compact_str::CompactString;
use eyre::eyre;
use gcmodule::{ThreadedCc, Trace};
use std::{fmt::Debug, hash::Hash, ops::Deref};

use crate::Vm;

// TODO: lots of locking here

#[derive(Clone, Debug, PartialEq)]
pub enum OdraValue {
    Number(f64),
    String(CompactString), // compact str has atomically reference counted strings and a few nice small-size optimizations
    List(im::Vector<OdraRef>),
    Map(im::HashMap<OdraRef, OdraRef>),
}

impl Eq for OdraValue {}
impl Hash for OdraValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            OdraValue::Number(n) => n.to_bits().hash(state), // we do a little trolling
            OdraValue::String(s) => s.hash(state),
            OdraValue::List(l) => l.hash(state),
            OdraValue::Map(m) => m.hash(state),
        }
    }
}

/// Hash and Eq impls have weird semantics for Refs - it's structural iff the ref points to number or string, identity otherwise.
/// In other words, Lists and Maps are compared by ref and other runtime types are compared by value.
#[derive(Clone)]
pub struct OdraRef(ThreadedCc<OdraValue>);

impl OdraRef {
    #[inline]
    fn address_usize(&self) -> usize {
        self.0.borrow().deref() as *const _ as usize
    }
}

impl Hash for OdraRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.0.borrow().deref() {
            OdraValue::Number(n) => {
                n.to_bits().hash(state);
                return;
            }
            OdraValue::String(s) => {
                s.hash(state);
                return;
            }
            _ => (),
        }

        self.address_usize().hash(state)
    }
}

impl PartialEq for OdraRef {
    fn eq(&self, other: &Self) -> bool {
        self.address_usize() == other.address_usize()
            || match (self.0.borrow().deref(), other.0.borrow().deref()) {
                (OdraValue::Number(n), OdraValue::Number(other_n)) => n == other_n,
                (OdraValue::String(s), OdraValue::String(other_s)) => s == other_s,
                _ => false,
            }
    }
}

impl Eq for OdraRef {}

impl Debug for OdraRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OdraRef")
            .field(self.0.borrow().deref())
            .finish()
    }
}

impl Trace for OdraValue {
    fn trace(&self, tracer: &mut gcmodule::Tracer) {
        match self {
            OdraValue::List(l) => l.iter().for_each(|cc| cc.0.trace(tracer)),
            OdraValue::Map(m) => m.iter().for_each(|(k, v)| {
                k.0.trace(tracer);
                v.0.trace(tracer);
            }),
            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OdraType {
    Number,
    String,
    OtherNamed(CompactString),
}

pub trait AsOdraValue: AsOdraType {
    fn as_odra_value(self) -> OdraValue;
}

pub trait AsOdraType {
    fn odra_type() -> Option<OdraType>;
}

impl AsOdraType for &mut Vm {
    fn odra_type() -> Option<OdraType> {
        None
    }
}

pub trait FromOdraValue
where
    Self: Sized,
{
    fn from_odra_value(odra_value: OdraValue) -> Result<Self>;
}

macro_rules! impl_numbers {
    ($($t:ty),*) => {
        $(
            impl AsOdraType for $t {
                fn odra_type() -> Option<OdraType> { Some(OdraType::Number) }
            }

            impl AsOdraValue for $t {
                fn as_odra_value(self) -> OdraValue {
                    // NOTE: `as` conversion, can lose precision I guess
                    OdraValue::Number(self as f64)
                }
            }

            impl FromOdraValue for $t {
                fn from_odra_value(odra_value: OdraValue) -> Result<Self> {
                    if let OdraValue::Number(f) = odra_value {
                        let i = f as $t;
                        if f == i as f64 {
                            Ok(i)
                        } else {
                            Err(eyre!("{} cannot be accurately represented as an integer"))
                        }
                    } else {
                        Err(eyre!("Could not convert {:?} to number", odra_value))
                    }
                }
            }
        )*
    };
}

impl_numbers!(u8, u16, u32, u64, i8, i16, i32, i64, f64);
