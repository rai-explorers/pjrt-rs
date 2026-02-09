//! PJRT Type System
//!
//! This module defines the type system for PJRT buffers, providing type-safe
//! representations of all supported PJRT data types. The type system includes:
//!
//! - Floating-point types: F16, BF16, F32, F64, and experimental F8 types
//! - Integer types: Signed (S2, S4, S8, S16, S32, S64) and unsigned (U2-U64)
//! - Complex types: C64 (complex float32) and C128 (complex float64)
//! - Boolean and token types
//!
//! The type system uses Rust's type system to provide compile-time type safety
//! while mapping to PJRT's runtime type system.

use std::any::Any;
use std::fmt::Debug;

use pjrt_sys::{
    PJRT_Buffer_Type, PJRT_Buffer_Type_PJRT_Buffer_Type_BF16,
    PJRT_Buffer_Type_PJRT_Buffer_Type_C128, PJRT_Buffer_Type_PJRT_Buffer_Type_C64,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F16, PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F64, PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3B11FNUZ,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FN, PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FNUZ,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2, PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2FNUZ,
    PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID, PJRT_Buffer_Type_PJRT_Buffer_Type_PRED,
    PJRT_Buffer_Type_PJRT_Buffer_Type_S16, PJRT_Buffer_Type_PJRT_Buffer_Type_S2,
    PJRT_Buffer_Type_PJRT_Buffer_Type_S32, PJRT_Buffer_Type_PJRT_Buffer_Type_S4,
    PJRT_Buffer_Type_PJRT_Buffer_Type_S64, PJRT_Buffer_Type_PJRT_Buffer_Type_S8,
    PJRT_Buffer_Type_PJRT_Buffer_Type_TOKEN, PJRT_Buffer_Type_PJRT_Buffer_Type_U16,
    PJRT_Buffer_Type_PJRT_Buffer_Type_U2, PJRT_Buffer_Type_PJRT_Buffer_Type_U32,
    PJRT_Buffer_Type_PJRT_Buffer_Type_U4, PJRT_Buffer_Type_PJRT_Buffer_Type_U64,
    PJRT_Buffer_Type_PJRT_Buffer_Type_U8,
};

use crate::{Error, Result};

pub trait Type: Sized + Copy + Debug + 'static {
    const NAME: &'static str;
    const PRIMITIVE_TYPE: PrimitiveType;
    const TYPE: Self;
    const SIZE: usize = std::mem::size_of::<Self::ElemType>();
    const ALIGNMENT: usize = std::mem::align_of::<Self::ElemType>();
    type ElemType: ElemType<Type = Self>;
}

pub trait ElemType: Sized + Copy + Debug + 'static {
    type Type: Type<ElemType = Self>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bool;

impl Type for Bool {
    const NAME: &'static str = "bool";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::Pred;
    const TYPE: Self = Bool;
    type ElemType = bool;
}

impl ElemType for bool {
    type Type = Bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct F32;

impl Type for F32 {
    const NAME: &'static str = "f32";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::F32;
    const TYPE: Self = F32;
    type ElemType = f32;
}

impl ElemType for f32 {
    type Type = F32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct F64;

impl Type for F64 {
    const NAME: &'static str = "f64";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::F64;
    const TYPE: Self = F64;
    type ElemType = f64;
}

impl ElemType for f64 {
    type Type = F64;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I8;

impl Type for I8 {
    const NAME: &'static str = "i8";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S8;
    const TYPE: Self = I8;
    type ElemType = i8;
}

impl ElemType for i8 {
    type Type = I8;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I16;

impl Type for I16 {
    const NAME: &'static str = "i16";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S16;
    const TYPE: Self = I16;
    type ElemType = i16;
}

impl ElemType for i16 {
    type Type = I16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I32;

impl Type for I32 {
    const NAME: &'static str = "i32";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S32;
    const TYPE: Self = I32;
    type ElemType = i32;
}

impl ElemType for i32 {
    type Type = I32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I64;

impl Type for I64 {
    const NAME: &'static str = "i64";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S64;
    const TYPE: Self = I64;
    type ElemType = i64;
}

impl ElemType for i64 {
    type Type = I64;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U8;

impl Type for U8 {
    const NAME: &'static str = "u8";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::U8;
    const TYPE: Self = U8;
    type ElemType = u8;
}

impl ElemType for u8 {
    type Type = U8;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U16;

impl Type for U16 {
    const NAME: &'static str = "u16";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::U16;
    const TYPE: Self = U16;
    type ElemType = u16;
}

impl ElemType for u16 {
    type Type = U16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct U32;

impl Type for U32 {
    const NAME: &'static str = "u32";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::U32;
    const TYPE: Self = U32;
    type ElemType = u32;
}

impl ElemType for u32 {
    type Type = U32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct U64;

impl Type for U64 {
    const NAME: &'static str = "u64";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::U64;
    const TYPE: Self = U64;
    type ElemType = u64;
}

impl ElemType for u64 {
    type Type = U64;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct F16;

impl Type for F16 {
    const NAME: &'static str = "f16";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::F16;
    const TYPE: Self = F16;
    type ElemType = half::f16;
}

impl ElemType for half::f16 {
    type Type = F16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BF16;

impl Type for BF16 {
    const NAME: &'static str = "bf16";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::BF16;
    const TYPE: Self = BF16;
    type ElemType = half::bf16;
}

impl ElemType for half::bf16 {
    type Type = BF16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct C64;

impl Type for C64 {
    const NAME: &'static str = "c64";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::C64;
    const TYPE: Self = C64;
    type ElemType = num_complex::Complex<f32>;
}

impl ElemType for num_complex::Complex<f32> {
    type Type = C64;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct C128;

impl Type for C128 {
    const NAME: &'static str = "c128";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::C128;
    const TYPE: Self = C128;
    type ElemType = num_complex::Complex<f64>;
}

impl ElemType for num_complex::Complex<f64> {
    type Type = C128;
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrimitiveType {
    /// Invalid primitive type to serve as default.
    Invalid = PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID as i32,

    /// Predicates are two-state booleans.
    Pred = PJRT_Buffer_Type_PJRT_Buffer_Type_PRED as i32,

    /// Signed integral values of fixed width.
    S8 = PJRT_Buffer_Type_PJRT_Buffer_Type_S8 as i32,
    S16 = PJRT_Buffer_Type_PJRT_Buffer_Type_S16 as i32,
    S32 = PJRT_Buffer_Type_PJRT_Buffer_Type_S32 as i32,
    S64 = PJRT_Buffer_Type_PJRT_Buffer_Type_S64 as i32,

    /// Unsigned integral values of fixed width.
    U8 = PJRT_Buffer_Type_PJRT_Buffer_Type_U8 as i32,
    U16 = PJRT_Buffer_Type_PJRT_Buffer_Type_U16 as i32,
    U32 = PJRT_Buffer_Type_PJRT_Buffer_Type_U32 as i32,
    U64 = PJRT_Buffer_Type_PJRT_Buffer_Type_U64 as i32,

    /// Floating-point values of fixed width.
    F16 = PJRT_Buffer_Type_PJRT_Buffer_Type_F16 as i32,
    F32 = PJRT_Buffer_Type_PJRT_Buffer_Type_F32 as i32,
    F64 = PJRT_Buffer_Type_PJRT_Buffer_Type_F64 as i32,

    /// Truncated 16 bit floating-point format. This is similar to IEEE's 16 bit
    /// floating-point format, but uses 1 bit for the sign, 8 bits for the exponent
    /// and 7 bits for the mantissa.
    BF16 = PJRT_Buffer_Type_PJRT_Buffer_Type_BF16 as i32,

    /// Complex values of fixed width.
    ///
    /// Paired F32 (real, imag), as in `std::complex<float>`.
    C64 = PJRT_Buffer_Type_PJRT_Buffer_Type_C64 as i32,
    /// Paired F64 (real, imag), as in `std::complex<double>`.
    C128 = PJRT_Buffer_Type_PJRT_Buffer_Type_C128 as i32,

    /// Truncated 8 bit floating-point formats.
    F8E5M2 = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2 as i32,
    F8E4M3FN = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FN as i32,
    F8E4M3B11FNUZ = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3B11FNUZ as i32,
    F8E5M2FNUZ = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2FNUZ as i32,
    F8E4M3FNUZ = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FNUZ as i32,

    /// 4-bit integer types
    S4 = PJRT_Buffer_Type_PJRT_Buffer_Type_S4 as i32,
    U4 = PJRT_Buffer_Type_PJRT_Buffer_Type_U4 as i32,

    Token = PJRT_Buffer_Type_PJRT_Buffer_Type_TOKEN as i32,

    /// 2-bit integer types
    S2 = PJRT_Buffer_Type_PJRT_Buffer_Type_S2 as i32,
    U2 = PJRT_Buffer_Type_PJRT_Buffer_Type_U2 as i32,
}

impl TryFrom<PrimitiveType> for Box<dyn DType> {
    type Error = Error;

    fn try_from(value: PrimitiveType) -> Result<Self> {
        value.try_into_dtype()
    }
}

impl PrimitiveType {
    pub fn try_into_dtype(&self) -> Result<Box<dyn DType>> {
        match self {
            PrimitiveType::Invalid => Err(Error::Unimplemented),
            PrimitiveType::Pred => Ok(Bool.boxed_dtype()),
            PrimitiveType::S8 => Ok(I8.boxed_dtype()),
            PrimitiveType::S16 => Ok(I16.boxed_dtype()),
            PrimitiveType::S32 => Ok(I32.boxed_dtype()),
            PrimitiveType::S64 => Ok(I64.boxed_dtype()),
            PrimitiveType::U8 => Ok(U8.boxed_dtype()),
            PrimitiveType::U16 => Ok(U16.boxed_dtype()),
            PrimitiveType::U32 => Ok(U32.boxed_dtype()),
            PrimitiveType::U64 => Ok(U64.boxed_dtype()),
            PrimitiveType::F32 => Ok(F32.boxed_dtype()),
            PrimitiveType::F64 => Ok(F64.boxed_dtype()),
            PrimitiveType::F16 => Ok(F16.boxed_dtype()),
            PrimitiveType::BF16 => Ok(BF16.boxed_dtype()),
            PrimitiveType::C64 => Ok(C64.boxed_dtype()),
            PrimitiveType::C128 => Ok(C128.boxed_dtype()),
            PrimitiveType::F8E5M2 => Err(Error::Unimplemented),
            PrimitiveType::F8E4M3FN => Err(Error::Unimplemented),
            PrimitiveType::F8E4M3B11FNUZ => Err(Error::Unimplemented),
            PrimitiveType::F8E5M2FNUZ => Err(Error::Unimplemented),
            PrimitiveType::F8E4M3FNUZ => Err(Error::Unimplemented),
            PrimitiveType::S4 => Err(Error::Unimplemented),
            PrimitiveType::U4 => Err(Error::Unimplemented),
            PrimitiveType::Token => Err(Error::Unimplemented),
            PrimitiveType::S2 => Err(Error::Unimplemented),
            PrimitiveType::U2 => Err(Error::Unimplemented),
        }
    }
}

impl TryFrom<PJRT_Buffer_Type> for PrimitiveType {
    type Error = Error;

    #[allow(non_upper_case_globals)]
    #[allow(non_snake_case)]
    fn try_from(value: PJRT_Buffer_Type) -> Result<Self> {
        match value {
            PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID => Ok(Self::Invalid),
            PJRT_Buffer_Type_PJRT_Buffer_Type_PRED => Ok(Self::Pred),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S8 => Ok(Self::S8),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S16 => Ok(Self::S16),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S32 => Ok(Self::S32),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S64 => Ok(Self::S64),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U8 => Ok(Self::U8),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U16 => Ok(Self::U16),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U32 => Ok(Self::U32),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U64 => Ok(Self::U64),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F16 => Ok(Self::F16),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F32 => Ok(Self::F32),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F64 => Ok(Self::F64),
            PJRT_Buffer_Type_PJRT_Buffer_Type_BF16 => Ok(Self::BF16),
            PJRT_Buffer_Type_PJRT_Buffer_Type_C64 => Ok(Self::C64),
            PJRT_Buffer_Type_PJRT_Buffer_Type_C128 => Ok(Self::C128),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2 => Ok(Self::F8E5M2),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FN => Ok(Self::F8E4M3FN),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3B11FNUZ => Ok(Self::F8E4M3B11FNUZ),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2FNUZ => Ok(Self::F8E5M2FNUZ),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FNUZ => Ok(Self::F8E4M3FNUZ),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S4 => Ok(Self::S4),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U4 => Ok(Self::U4),
            PJRT_Buffer_Type_PJRT_Buffer_Type_TOKEN => Ok(Self::Token),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S2 => Ok(Self::S2),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U2 => Ok(Self::U2),
            _ => Err(Error::InvalidPrimitiveType(value as i32)),
        }
    }
}

pub trait DType {
    fn name(&self) -> &'static str;
    fn primitive_type(&self) -> PrimitiveType;
    fn size(&self) -> usize;
    fn alignment(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn boxed_dtype(&self) -> Box<dyn DType>;
}

impl Clone for Box<dyn DType> {
    fn clone(&self) -> Self {
        self.boxed_dtype()
    }
}

impl<T: Type> DType for T {
    fn name(&self) -> &'static str {
        T::NAME
    }

    fn primitive_type(&self) -> PrimitiveType {
        T::PRIMITIVE_TYPE
    }

    fn size(&self) -> usize {
        T::SIZE
    }

    fn alignment(&self) -> usize {
        T::ALIGNMENT
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_dtype(&self) -> Box<dyn DType> {
        Box::new(T::TYPE)
    }
}

pub trait AsDType {
    fn as_dtype(&self) -> &dyn DType;
}

impl<T: DType> AsDType for T {
    fn as_dtype(&self) -> &dyn DType {
        self
    }
}

#[cfg(test)]
mod tests {
    use num_complex::Complex;

    use super::*;

    #[test]
    fn test_bool_type_properties() {
        assert_eq!(Bool::NAME, "bool");
        assert_eq!(Bool::PRIMITIVE_TYPE, PrimitiveType::Pred);
        assert_eq!(Bool::SIZE, 1);
        assert_eq!(Bool::TYPE, Bool);
        assert_eq!(std::mem::size_of::<bool>(), 1);
    }

    #[test]
    fn test_f32_type_properties() {
        assert_eq!(F32::NAME, "f32");
        assert_eq!(F32::PRIMITIVE_TYPE, PrimitiveType::F32);
        assert_eq!(F32::SIZE, 4);
        assert_eq!(F32::TYPE, F32);
        assert_eq!(std::mem::size_of::<f32>(), 4);
    }

    #[test]
    fn test_f64_type_properties() {
        assert_eq!(F64::NAME, "f64");
        assert_eq!(F64::PRIMITIVE_TYPE, PrimitiveType::F64);
        assert_eq!(F64::SIZE, 8);
        assert_eq!(F64::TYPE, F64);
        assert_eq!(std::mem::size_of::<f64>(), 8);
    }

    #[test]
    fn test_i8_type_properties() {
        assert_eq!(I8::NAME, "i8");
        assert_eq!(I8::PRIMITIVE_TYPE, PrimitiveType::S8);
        assert_eq!(I8::SIZE, 1);
        assert_eq!(I8::TYPE, I8);
    }

    #[test]
    fn test_i16_type_properties() {
        assert_eq!(I16::NAME, "i16");
        assert_eq!(I16::PRIMITIVE_TYPE, PrimitiveType::S16);
        assert_eq!(I16::SIZE, 2);
        assert_eq!(I16::TYPE, I16);
    }

    #[test]
    fn test_i32_type_properties() {
        assert_eq!(I32::NAME, "i32");
        assert_eq!(I32::PRIMITIVE_TYPE, PrimitiveType::S32);
        assert_eq!(I32::SIZE, 4);
        assert_eq!(I32::TYPE, I32);
    }

    #[test]
    fn test_i64_type_properties() {
        assert_eq!(I64::NAME, "i64");
        assert_eq!(I64::PRIMITIVE_TYPE, PrimitiveType::S64);
        assert_eq!(I64::SIZE, 8);
        assert_eq!(I64::TYPE, I64);
    }

    #[test]
    fn test_u8_type_properties() {
        assert_eq!(U8::NAME, "u8");
        assert_eq!(U8::PRIMITIVE_TYPE, PrimitiveType::U8);
        assert_eq!(U8::SIZE, 1);
        assert_eq!(U8::TYPE, U8);
    }

    #[test]
    fn test_u16_type_properties() {
        assert_eq!(U16::NAME, "u16");
        assert_eq!(U16::PRIMITIVE_TYPE, PrimitiveType::U16);
        assert_eq!(U16::SIZE, 2);
        assert_eq!(U16::TYPE, U16);
    }

    #[test]
    fn test_u32_type_properties() {
        assert_eq!(U32::NAME, "u32");
        assert_eq!(U32::PRIMITIVE_TYPE, PrimitiveType::U32);
        assert_eq!(U32::SIZE, 4);
        assert_eq!(U32::TYPE, U32);
    }

    #[test]
    fn test_u64_type_properties() {
        assert_eq!(U64::NAME, "u64");
        assert_eq!(U64::PRIMITIVE_TYPE, PrimitiveType::U64);
        assert_eq!(U64::SIZE, 8);
        assert_eq!(U64::TYPE, U64);
    }

    #[test]
    fn test_f16_type_properties() {
        assert_eq!(F16::NAME, "f16");
        assert_eq!(F16::PRIMITIVE_TYPE, PrimitiveType::F16);
        assert_eq!(F16::SIZE, 2);
        assert_eq!(F16::TYPE, F16);
    }

    #[test]
    fn test_bf16_type_properties() {
        assert_eq!(BF16::NAME, "bf16");
        assert_eq!(BF16::PRIMITIVE_TYPE, PrimitiveType::BF16);
        assert_eq!(BF16::SIZE, 2);
        assert_eq!(BF16::TYPE, BF16);
    }

    #[test]
    fn test_c64_type_properties() {
        assert_eq!(C64::NAME, "c64");
        assert_eq!(C64::PRIMITIVE_TYPE, PrimitiveType::C64);
        assert_eq!(C64::SIZE, 8);
        assert_eq!(C64::TYPE, C64);
    }

    #[test]
    fn test_c128_type_properties() {
        assert_eq!(C128::NAME, "c128");
        assert_eq!(C128::PRIMITIVE_TYPE, PrimitiveType::C128);
        assert_eq!(C128::SIZE, 16);
        assert_eq!(C128::TYPE, C128);
    }

    #[test]
    fn test_elem_type_trait() {
        fn check_elem_type<T: ElemType>() {}
        check_elem_type::<bool>();
        check_elem_type::<f32>();
        check_elem_type::<f64>();
        check_elem_type::<i8>();
        check_elem_type::<i16>();
        check_elem_type::<i32>();
        check_elem_type::<i64>();
        check_elem_type::<u8>();
        check_elem_type::<u16>();
        check_elem_type::<u32>();
        check_elem_type::<u64>();
        check_elem_type::<half::f16>();
        check_elem_type::<half::bf16>();
        check_elem_type::<Complex<f32>>();
        check_elem_type::<Complex<f64>>();
    }

    #[test]
    fn test_dtype_trait() {
        let f32_dtype: Box<dyn DType> = F32.boxed_dtype();
        assert_eq!(f32_dtype.name(), "f32");
        assert_eq!(f32_dtype.primitive_type(), PrimitiveType::F32);
        assert_eq!(f32_dtype.size(), 4);
        assert_eq!(f32_dtype.alignment(), 4);

        let f64_dtype: Box<dyn DType> = F64.boxed_dtype();
        assert_eq!(f64_dtype.name(), "f64");
        assert_eq!(f64_dtype.primitive_type(), PrimitiveType::F64);
        assert_eq!(f64_dtype.size(), 8);

        let i32_dtype: Box<dyn DType> = I32.boxed_dtype();
        assert_eq!(i32_dtype.name(), "i32");
        assert_eq!(i32_dtype.primitive_type(), PrimitiveType::S32);
    }

    #[test]
    fn test_dtype_clone() {
        let dtype1: Box<dyn DType> = F32.boxed_dtype();
        let dtype2 = dtype1.clone();
        assert_eq!(dtype1.name(), dtype2.name());
        assert_eq!(dtype1.primitive_type(), dtype2.primitive_type());
        assert_eq!(dtype1.size(), dtype2.size());
    }

    #[test]
    fn test_primitive_type_try_into_dtype() {
        let dtype: Box<dyn DType> = PrimitiveType::F32.try_into_dtype().unwrap();
        assert_eq!(dtype.name(), "f32");
        assert_eq!(dtype.primitive_type(), PrimitiveType::F32);

        let dtype: Box<dyn DType> = PrimitiveType::F64.try_into_dtype().unwrap();
        assert_eq!(dtype.name(), "f64");

        let dtype: Box<dyn DType> = PrimitiveType::S32.try_into_dtype().unwrap();
        assert_eq!(dtype.name(), "i32");

        let dtype: Box<dyn DType> = PrimitiveType::U8.try_into_dtype().unwrap();
        assert_eq!(dtype.name(), "u8");

        let dtype: Box<dyn DType> = PrimitiveType::Pred.try_into_dtype().unwrap();
        assert_eq!(dtype.name(), "bool");

        // Test unimplemented types
        assert!(PrimitiveType::F8E5M2.try_into_dtype().is_err());
        assert!(PrimitiveType::F8E4M3FN.try_into_dtype().is_err());
        assert!(PrimitiveType::S4.try_into_dtype().is_err());
        assert!(PrimitiveType::U4.try_into_dtype().is_err());
        assert!(PrimitiveType::Invalid.try_into_dtype().is_err());
    }

    #[test]
    fn test_primitive_type_from_pjrt_buffer_type() {
        use pjrt_sys::{
            PJRT_Buffer_Type_PJRT_Buffer_Type_F32, PJRT_Buffer_Type_PJRT_Buffer_Type_F64,
            PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID, PJRT_Buffer_Type_PJRT_Buffer_Type_S32,
        };

        let primitive = PrimitiveType::try_from(PJRT_Buffer_Type_PJRT_Buffer_Type_F32).unwrap();
        assert_eq!(primitive, PrimitiveType::F32);

        let primitive = PrimitiveType::try_from(PJRT_Buffer_Type_PJRT_Buffer_Type_F64).unwrap();
        assert_eq!(primitive, PrimitiveType::F64);

        let primitive = PrimitiveType::try_from(PJRT_Buffer_Type_PJRT_Buffer_Type_S32).unwrap();
        assert_eq!(primitive, PrimitiveType::S32);

        // Invalid type should error
        let result = PrimitiveType::try_from(PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID);
        assert!(result.is_ok()); // Invalid is a valid enum variant
    }

    #[test]
    fn test_as_dtype_trait() {
        let f32_val = F32;
        let dtype = f32_val.as_dtype();
        assert_eq!(dtype.name(), "f32");

        // Test that concrete types implement AsDType
        fn check_as_dtype<T: AsDType>(val: T) -> &'static str {
            val.as_dtype().name()
        }
        assert_eq!(check_as_dtype(F64), "f64");
    }

    #[test]
    fn test_all_primitive_types() {
        // Test that all primitive types have unique values
        let types = vec![
            PrimitiveType::Invalid,
            PrimitiveType::Pred,
            PrimitiveType::S8,
            PrimitiveType::S16,
            PrimitiveType::S32,
            PrimitiveType::S64,
            PrimitiveType::U8,
            PrimitiveType::U16,
            PrimitiveType::U32,
            PrimitiveType::U64,
            PrimitiveType::F16,
            PrimitiveType::F32,
            PrimitiveType::F64,
            PrimitiveType::BF16,
            PrimitiveType::C64,
            PrimitiveType::C128,
            PrimitiveType::F8E5M2,
            PrimitiveType::F8E4M3FN,
            PrimitiveType::F8E4M3B11FNUZ,
            PrimitiveType::F8E5M2FNUZ,
            PrimitiveType::F8E4M3FNUZ,
            PrimitiveType::S4,
            PrimitiveType::U4,
            PrimitiveType::Token,
            PrimitiveType::S2,
            PrimitiveType::U2,
        ];

        let mut seen = std::collections::HashSet::new();
        for t in &types {
            let value = *t as i32;
            assert!(
                seen.insert(value),
                "Duplicate primitive type value: {:?} = {}",
                t,
                value
            );
        }
    }

    #[test]
    fn test_type_marker_traits() {
        // Ensure all type markers implement required traits
        fn check_traits<T: Type>() {}
        check_traits::<Bool>();
        check_traits::<F32>();
        check_traits::<F64>();
        check_traits::<I8>();
        check_traits::<I16>();
        check_traits::<I32>();
        check_traits::<I64>();
        check_traits::<U8>();
        check_traits::<U16>();
        check_traits::<U32>();
        check_traits::<U64>();
        check_traits::<F16>();
        check_traits::<BF16>();
        check_traits::<C64>();
        check_traits::<C128>();
    }

    #[test]
    fn test_complex_elem_type() {
        let _complex_f32 = Complex::<f32>::new(1.0, 2.0);
        assert_eq!(<Complex<f32> as ElemType>::Type::NAME, "c64");
        assert_eq!(<Complex<f32> as ElemType>::Type::SIZE, 8);

        let _complex_f64 = Complex::<f64>::new(1.0, 2.0);
        assert_eq!(<Complex<f64> as ElemType>::Type::NAME, "c128");
        assert_eq!(<Complex<f64> as ElemType>::Type::SIZE, 16);
    }

    #[test]
    fn test_half_elem_type() {
        assert_eq!(<half::f16 as ElemType>::Type::NAME, "f16");
        assert_eq!(<half::f16 as ElemType>::Type::SIZE, 2);

        assert_eq!(<half::bf16 as ElemType>::Type::NAME, "bf16");
        assert_eq!(<half::bf16 as ElemType>::Type::SIZE, 2);
    }
}
