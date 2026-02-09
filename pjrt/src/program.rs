//! PJRT Program Representation
//!
//! This module provides the `Program` struct for representing PJRT programs
//! that can be compiled and executed. Programs can be in either MLIR or HLO
//! format and are the input to the compilation process.
//!
//! The module includes functionality to:
//! - Load programs from files or strings
//! - Convert between program formats
//! - Serialize/deserialize programs
//!
//! # Examples
//!
//! ## Creating a Program from Code
//!
//! ```rust
//! use pjrt::{Program, ProgramFormat};
//!
//! // Create an MLIR program from inline code
//! let mlir_code = br#"
//!     module @example {
//!         func.func @main(%arg0: tensor<4xf32>) -> tensor<4xf32> {
//!             %0 = stablehlo.add %arg0, %arg0 : tensor<4xf32>
//!             return %0 : tensor<4xf32>
//!         }
//!     }
//! "#;
//!
//! let program = Program::new(ProgramFormat::MLIR, mlir_code.as_slice());
//! assert_eq!(program.format(), ProgramFormat::MLIR);
//! ```
//!
//! ## Loading from a File
//!
//! ```rust,ignore
//! use pjrt::{Program, ProgramFormat};
//!
//! // Load MLIR from file
//! let program = Program::from_file("model.mlir", ProgramFormat::MLIR)?;
//!
//! // Or use auto-detection based on extension
//! let program = Program::from_file_auto("model.mlir")?;
//! ```
//!
//! ## Program Formats
//!
//! ```rust
//! use pjrt::ProgramFormat;
//!
//! // MLIR StableHLO format (recommended)
//! let mlir = ProgramFormat::MLIR;
//! assert_eq!(mlir.as_str(), "mlir");
//!
//! // HLO format
//! let hlo = ProgramFormat::HLO;
//! assert_eq!(hlo.as_str(), "hlo");
//!
//! // Parse from string
//! let format: ProgramFormat = "mlir".try_into().unwrap();
//! assert_eq!(format, ProgramFormat::MLIR);
//! ```

use std::fs;
use std::path::Path;

use pjrt_sys::PJRT_Program;

use crate::{Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgramFormat {
    MLIR,
    HLO,
}

impl ProgramFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProgramFormat::MLIR => "mlir",
            ProgramFormat::HLO => "hlo",
        }
    }

    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            ProgramFormat::MLIR => b"mlir",
            ProgramFormat::HLO => b"hlo",
        }
    }
}

impl TryFrom<&str> for ProgramFormat {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "mlir" => Ok(ProgramFormat::MLIR),
            "hlo" => Ok(ProgramFormat::HLO),
            _ => Err(Error::InvalidProgramFormat(value.to_string())),
        }
    }
}

/// A compiled program (MLIR or HLO) ready to be loaded on a client.
///
/// # Thread Safety
///
/// `Program` is `!Send + !Sync` because the internal `PJRT_Program` struct
/// contains raw pointers into the owned `code` and `format` data. The
/// program itself is logically immutable after construction and could be
/// made `Send + Sync` with an `unsafe impl` if cross-thread compilation
/// is needed.
pub struct Program {
    format: ProgramFormat,
    code: Vec<u8>,
    pub(crate) prog: PJRT_Program,
}

impl Program {
    pub fn new(format: ProgramFormat, code: impl Into<Vec<u8>>) -> Self {
        let mut program = Program {
            format,
            code: code.into(),
            prog: PJRT_Program::new(),
        };
        program.prog.code = program.code.as_ptr() as *mut i8;
        program.prog.code_size = program.code.len();
        let format = program.format.as_bytes();
        program.prog.format = format.as_ptr() as *const i8;
        program.prog.format_size = format.len();
        program
    }

    pub fn format(&self) -> ProgramFormat {
        self.format
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn from_mlir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let code = fs::read_to_string(path)?;
        Ok(Program::new(ProgramFormat::MLIR, code))
    }

    pub fn from_hlo<P: AsRef<Path>>(path: P) -> Result<Self> {
        let code = fs::read(path)?;
        Ok(Program::new(ProgramFormat::HLO, code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_format_mlir() {
        assert_eq!(ProgramFormat::MLIR.as_str(), "mlir");
        assert_eq!(ProgramFormat::MLIR.as_bytes(), b"mlir");
    }

    #[test]
    fn test_program_format_hlo() {
        assert_eq!(ProgramFormat::HLO.as_str(), "hlo");
        assert_eq!(ProgramFormat::HLO.as_bytes(), b"hlo");
    }

    #[test]
    fn test_program_format_try_from_str() {
        let format: ProgramFormat = "mlir".try_into().unwrap();
        assert_eq!(format, ProgramFormat::MLIR);

        let format: ProgramFormat = "hlo".try_into().unwrap();
        assert_eq!(format, ProgramFormat::HLO);

        let result: Result<ProgramFormat> = "invalid".try_into();
        assert!(result.is_err());
        match result {
            Err(Error::InvalidProgramFormat(s)) => assert_eq!(s, "invalid"),
            _ => panic!("Expected InvalidProgramFormat error"),
        }
    }

    #[test]
    fn test_program_new_mlir() {
        let code = b"module { func.func @main() { return } }";
        let program = Program::new(ProgramFormat::MLIR, code.to_vec());

        assert_eq!(program.format(), ProgramFormat::MLIR);
        assert_eq!(program.code(), code);

        // Check that the internal PJRT_Program struct is properly initialized
        assert!(!program.prog.code.is_null());
        assert_eq!(program.prog.code_size, code.len());
        assert!(!program.prog.format.is_null());
        assert_eq!(program.prog.format_size, 4); // "mlir" is 4 bytes
    }

    #[test]
    fn test_program_new_hlo() {
        let code = b"HLO_BINARY_DATA";
        let program = Program::new(ProgramFormat::HLO, code.to_vec());

        assert_eq!(program.format(), ProgramFormat::HLO);
        assert_eq!(program.code(), code);
        assert_eq!(program.prog.format_size, 3); // "hlo" is 3 bytes
    }

    #[test]
    fn test_program_new_empty_code() {
        let code: Vec<u8> = vec![];
        let program = Program::new(ProgramFormat::MLIR, code.clone());

        assert_eq!(program.code(), &code);
        assert_eq!(program.prog.code_size, 0);
    }

    #[test]
    fn test_program_new_from_string() {
        let code =
            "func.func @main(%arg0: tensor<f32>) -> tensor<f32> { return %arg0 : tensor<f32> }";
        let program = Program::new(ProgramFormat::MLIR, code.to_string());

        assert_eq!(program.code(), code.as_bytes());
        assert_eq!(program.prog.code_size, code.len());
    }

    #[test]
    fn test_program_format_clone() {
        let format = ProgramFormat::MLIR;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_program_format_debug() {
        let format = ProgramFormat::MLIR;
        let debug = format!("{:?}", format);
        assert!(debug.contains("MLIR"));
    }

    #[test]
    fn test_program_format_equality() {
        assert_eq!(ProgramFormat::MLIR, ProgramFormat::MLIR);
        assert_eq!(ProgramFormat::HLO, ProgramFormat::HLO);
        assert_ne!(ProgramFormat::MLIR, ProgramFormat::HLO);
    }

    #[test]
    fn test_program_format_ordering() {
        // MLIR comes before HLO in the enum definition, so MLIR < HLO
        assert!(ProgramFormat::MLIR < ProgramFormat::HLO);
        assert!(ProgramFormat::HLO > ProgramFormat::MLIR);
    }

    #[test]
    fn test_program_format_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ProgramFormat::MLIR);
        set.insert(ProgramFormat::HLO);
        set.insert(ProgramFormat::MLIR); // Duplicate

        assert_eq!(set.len(), 2);
    }
}
