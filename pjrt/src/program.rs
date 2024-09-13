use std::fs;
use std::path::Path;

use pjrt_sys::PJRT_Program;

use crate::Result;

pub static FORMAT_MLIR: &str = "mlir";
pub static FORMAT_HLO: &str = "hlo";
pub struct Program {
    code: Vec<u8>,
    pub(crate) prog: PJRT_Program,
}

impl Program {
    pub fn with_bytes(code: Vec<u8>, format: &'static str) -> Self {
        let mut program = Program {
            code,
            prog: PJRT_Program::new(),
        };
        program.prog.code = program.code.as_ptr() as *mut i8;
        program.prog.code_size = program.code.len();
        program.prog.format = format.as_ptr() as *const i8;
        program.prog.format_size = format.len();
        program
    }

    pub fn with_mlir(code: String) -> Self {
        let mut program = Program {
            code: code.into_bytes(),
            prog: PJRT_Program::new(),
        };
        program.prog.code = program.code.as_ptr() as *mut i8;
        program.prog.code_size = program.code.len();
        program.prog.format = FORMAT_MLIR.as_ptr() as *const i8;
        program.prog.format_size = FORMAT_MLIR.len();
        program
    }

    pub fn from_mlir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let code = fs::read_to_string(path)?;
        Ok(Program::with_mlir(code))
    }

    pub fn with_hlo(serilized: Vec<u8>) -> Self {
        let mut program = Program {
            code: serilized,
            prog: PJRT_Program::new(),
        };
        program.prog.code = program.code.as_ptr() as *mut i8;
        program.prog.code_size = program.code.len();
        program.prog.format = FORMAT_HLO.as_ptr() as *const i8;
        program.prog.format_size = FORMAT_HLO.len();
        program
    }

    pub fn from_hlo<P: AsRef<Path>>(path: P) -> Result<Self> {
        let code = fs::read(path)?;
        Ok(Program::with_hlo(code))
    }
}
