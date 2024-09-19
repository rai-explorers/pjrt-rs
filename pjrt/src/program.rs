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
