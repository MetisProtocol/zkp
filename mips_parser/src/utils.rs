use std::mem::transmute_copy;
use std::num::Wrapping;

// Wrapping, u32, i32 conversions

pub trait ToUnsigned {
    fn to_u32(&self) -> u32;
    fn to_u32w(&self) -> Wrapping<u32> {
        Wrapping(self.to_u32())
    }
}

pub trait ToSigned {
    fn to_i32(&self) -> i32;
    fn to_i32w(&self) -> Wrapping<i32> {
        Wrapping(self.to_i32())
    }
}

impl ToUnsigned for i32 {
    fn to_u32(&self) -> u32 {
        unsafe { transmute_copy::<i32, u32>(self) }
    }
}

impl ToSigned for i32 {
    fn to_i32(&self) -> i32 {
        *self
    }
}

impl ToUnsigned for u32 {
    fn to_u32(&self) -> u32 {
        *self
    }
}

impl ToSigned for u32 {
    fn to_i32(&self) -> i32 {
        unsafe { transmute_copy::<u32, i32>(self) }
    }
}

impl ToUnsigned for Wrapping<i32> {
    fn to_u32(&self) -> u32 {
        self.0.to_u32()
    }
}

impl ToSigned for Wrapping<i32> {
    fn to_i32(&self) -> i32 {
        self.0
    }
}

impl ToUnsigned for Wrapping<u32> {
    fn to_u32(&self) -> u32 {
        self.0
    }
}

impl ToSigned for Wrapping<u32> {
    fn to_i32(&self) -> i32 {
        self.0.to_i32()
    }
}