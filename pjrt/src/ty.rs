use std::any::Any;
use std::fmt::Debug;

use pjrt_sys::{
    PJRT_Buffer_Type_PJRT_Buffer_Type_BF16, PJRT_Buffer_Type_PJRT_Buffer_Type_C128,
    PJRT_Buffer_Type_PJRT_Buffer_Type_C64, PJRT_Buffer_Type_PJRT_Buffer_Type_F16,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F32, PJRT_Buffer_Type_PJRT_Buffer_Type_F64,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3B11FNUZ, PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FN,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FNUZ, PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2,
    PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2FNUZ, PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID,
    PJRT_Buffer_Type_PJRT_Buffer_Type_PRED, PJRT_Buffer_Type_PJRT_Buffer_Type_S16,
    PJRT_Buffer_Type_PJRT_Buffer_Type_S2, PJRT_Buffer_Type_PJRT_Buffer_Type_S32,
    PJRT_Buffer_Type_PJRT_Buffer_Type_S4, PJRT_Buffer_Type_PJRT_Buffer_Type_S64,
    PJRT_Buffer_Type_PJRT_Buffer_Type_S8, PJRT_Buffer_Type_PJRT_Buffer_Type_TOKEN,
    PJRT_Buffer_Type_PJRT_Buffer_Type_U16, PJRT_Buffer_Type_PJRT_Buffer_Type_U2,
    PJRT_Buffer_Type_PJRT_Buffer_Type_U32, PJRT_Buffer_Type_PJRT_Buffer_Type_U4,
    PJRT_Buffer_Type_PJRT_Buffer_Type_U64, PJRT_Buffer_Type_PJRT_Buffer_Type_U8,
};

use crate::{Error, HostBuffer, Result, Shape};

pub trait Type: Sized + Copy + Debug + 'static {
    const NAME: &'static str;
    const PRIMITIVE_TYPE: PrimitiveType;
    const TYPE: Self;
    const SIZE: usize = std::mem::size_of::<Self::ElemType>();
    const ALIGNMENT: usize = std::mem::align_of::<Self::ElemType>();
    type ElemType: ElemType<Type = Self>;

    fn new_host_buffer(
        data: impl Into<Vec<Self::ElemType>>,
        shape: impl Shape,
    ) -> Result<HostBuffer> {
        Err(Error::NotImplemented)
    }
}

pub trait ElemType: Sized + Copy + Debug + 'static {
    type Type: Type<ElemType = Self>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct S8;

impl Type for S8 {
    const NAME: &'static str = "s8";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S8;
    const TYPE: Self = S8;
    type ElemType = i8;
}

impl ElemType for i8 {
    type Type = S8;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct S16;

impl Type for S16 {
    const NAME: &'static str = "s16";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S16;
    const TYPE: Self = S16;
    type ElemType = i16;
}

impl ElemType for i16 {
    type Type = S16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct S32;

impl Type for S32 {
    const NAME: &'static str = "s32";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S32;
    const TYPE: Self = S32;
    type ElemType = i32;
}

impl ElemType for i32 {
    type Type = S32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct S64;

impl Type for S64 {
    const NAME: &'static str = "s64";
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::S64;
    const TYPE: Self = S64;
    type ElemType = i64;
}

impl ElemType for i64 {
    type Type = S64;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrimitiveType {
    /// Invalid primitive type to serve as default.
    Invalid = PJRT_Buffer_Type_PJRT_Buffer_Type_INVALID,

    /// Predicates are two-state booleans.
    Pred = PJRT_Buffer_Type_PJRT_Buffer_Type_PRED,

    /// Signed integral values of fixed width.
    S8 = PJRT_Buffer_Type_PJRT_Buffer_Type_S8,
    S16 = PJRT_Buffer_Type_PJRT_Buffer_Type_S16,
    S32 = PJRT_Buffer_Type_PJRT_Buffer_Type_S32,
    S64 = PJRT_Buffer_Type_PJRT_Buffer_Type_S64,

    /// Unsigned integral values of fixed width.
    U8 = PJRT_Buffer_Type_PJRT_Buffer_Type_U8,
    U16 = PJRT_Buffer_Type_PJRT_Buffer_Type_U16,
    U32 = PJRT_Buffer_Type_PJRT_Buffer_Type_U32,
    U64 = PJRT_Buffer_Type_PJRT_Buffer_Type_U64,

    /// Floating-point values of fixed width.
    F16 = PJRT_Buffer_Type_PJRT_Buffer_Type_F16,
    F32 = PJRT_Buffer_Type_PJRT_Buffer_Type_F32,
    F64 = PJRT_Buffer_Type_PJRT_Buffer_Type_F64,

    /// Truncated 16 bit floating-point format. This is similar to IEEE's 16 bit
    /// floating-point format, but uses 1 bit for the sign, 8 bits for the exponent
    /// and 7 bits for the mantissa.
    BF16 = PJRT_Buffer_Type_PJRT_Buffer_Type_BF16,

    /// Complex values of fixed width.
    ///
    /// Paired F32 (real, imag), as in std::complex<float>.
    C64 = PJRT_Buffer_Type_PJRT_Buffer_Type_C64,
    /// Paired F64 (real, imag), as in std::complex<double>.
    C128 = PJRT_Buffer_Type_PJRT_Buffer_Type_C128,

    /// Truncated 8 bit floating-point formats.
    F8E5M2 = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2,
    F8E4M3FN = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FN,
    F8E4M3B11FNUZ = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3B11FNUZ,
    F8E5M2FNUZ = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2FNUZ,
    F8E4M3FNUZ = PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FNUZ,

    /// 4-bit integer types
    S4 = PJRT_Buffer_Type_PJRT_Buffer_Type_S4,
    U4 = PJRT_Buffer_Type_PJRT_Buffer_Type_U4,

    Token = PJRT_Buffer_Type_PJRT_Buffer_Type_TOKEN,

    /// 2-bit integer types
    S2 = PJRT_Buffer_Type_PJRT_Buffer_Type_S2,
    U2 = PJRT_Buffer_Type_PJRT_Buffer_Type_U2,
}

impl PrimitiveType {
    pub fn boxed_dtype(&self) -> Box<dyn DType> {
        match self {
            PrimitiveType::Invalid => todo!(),
            PrimitiveType::Pred => todo!(),
            PrimitiveType::S8 => S8.boxed_dtype(),
            PrimitiveType::S16 => S16.boxed_dtype(),
            PrimitiveType::S32 => S32.boxed_dtype(),
            PrimitiveType::S64 => S64.boxed_dtype(),
            PrimitiveType::U8 => U8.boxed_dtype(),
            PrimitiveType::U16 => U16.boxed_dtype(),
            PrimitiveType::U32 => U32.boxed_dtype(),
            PrimitiveType::U64 => U64.boxed_dtype(),
            PrimitiveType::F16 => todo!(),
            PrimitiveType::F32 => F32.boxed_dtype(),
            PrimitiveType::F64 => F64.boxed_dtype(),
            PrimitiveType::BF16 => todo!(),
            PrimitiveType::C64 => todo!(),
            PrimitiveType::C128 => todo!(),
            PrimitiveType::F8E5M2 => todo!(),
            PrimitiveType::F8E4M3FN => todo!(),
            PrimitiveType::F8E4M3B11FNUZ => todo!(),
            PrimitiveType::F8E5M2FNUZ => todo!(),
            PrimitiveType::F8E4M3FNUZ => todo!(),
            PrimitiveType::S4 => todo!(),
            PrimitiveType::U4 => todo!(),
            PrimitiveType::Token => todo!(),
            PrimitiveType::S2 => todo!(),
            PrimitiveType::U2 => todo!(),
        }
    }
}

impl TryFrom<u32> for PrimitiveType {
    type Error = Error;

    #[allow(non_upper_case_globals)]
    fn try_from(value: u32) -> Result<Self> {
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
            PJRT_Buffer_Type_PJRT_Buffer_Type_F8E5M2FNUZ => Ok(Self::F8E4M3FNUZ),
            PJRT_Buffer_Type_PJRT_Buffer_Type_F8E4M3FNUZ => Ok(Self::F8E4M3FNUZ),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S4 => Ok(Self::S4),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U4 => Ok(Self::U4),
            PJRT_Buffer_Type_PJRT_Buffer_Type_TOKEN => Ok(Self::Token),
            PJRT_Buffer_Type_PJRT_Buffer_Type_S2 => Ok(Self::S2),
            PJRT_Buffer_Type_PJRT_Buffer_Type_U2 => Ok(Self::U2),
            _ => Err(Error::InvalidPrimitiveType(value)),
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
