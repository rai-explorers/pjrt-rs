//! PJRT Triton Extension
//!
//! This module provides safe Rust bindings for the PJRT Triton extension.
//! The Triton extension provides capabilities for compiling Triton kernels
//! for GPU execution.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::triton::TritonExtension;
//!
//! // Get the Triton extension
//! let triton_ext = api.get_extension::<TritonExtension>()?;
//!
//! // Compile a Triton kernel
//! let result = triton_ext.compile(
//!     triton_module_code,
//!     "sm_80",  // Architecture name
//!     4,        // num_warps
//!     1,        // num_ctas
//!     3,        // num_stages
//! )?;
//!
//! println!("Compiled ASM: {}", result.asm_code);
//! println!("Shared memory: {} bytes", result.smem_bytes);
//! ```

use std::rc::Rc;

use pjrt_sys::{PJRT_Triton, PJRT_Triton_Compile_Args};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Result};

/// Safe wrapper for PJRT Triton extension
///
/// The Triton extension provides capabilities for compiling Triton kernels
/// for GPU execution.
pub struct TritonExtension {
    raw: Rc<PJRT_Triton>,
    api: Api,
}

impl std::fmt::Debug for TritonExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TritonExtension")
            .field("api_version", &1i32) // Version 1
            .finish()
    }
}

unsafe impl Extension for TritonExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Triton
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let triton_ext = ptr as *mut PJRT_Triton;
        if (*triton_ext).base.type_ != ExtensionType::Triton.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new(*triton_ext),
            api: api.clone(),
        })
    }
}

/// Result of compiling a Triton kernel
pub struct TritonCompileResult {
    /// The compiled assembly code
    pub asm_code: String,
    /// The size of the compiled ASM in bytes
    pub asm_size: usize,
    /// The amount of shared memory required in bytes
    pub smem_bytes: i64,
}

impl TritonExtension {
    /// Compile a Triton kernel
    ///
    /// Compiles a Triton kernel module for the specified GPU architecture.
    ///
    /// # Arguments
    ///
    /// * `module` - The Triton kernel module code (Python/Triton source)
    /// * `arch_name` - The GPU architecture name (e.g., "sm_80" for Ampere)
    /// * `num_warps` - Number of warps per block
    /// * `num_ctas` - Number of CTAs per cluster
    /// * `num_stages` - Number of pipeline stages
    ///
    /// # Returns
    ///
    /// A `TritonCompileResult` containing the compiled assembly and metadata
    pub fn compile(
        &self,
        module: &str,
        arch_name: &str,
        num_warps: i32,
        num_ctas: i32,
        num_stages: i32,
    ) -> Result<TritonCompileResult> {
        let mut args = unsafe { std::mem::zeroed::<PJRT_Triton_Compile_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_Triton_Compile_Args>();
        args.module = module.as_ptr() as *const i8;
        args.module_size = module.len();
        args.arch_name = arch_name.as_ptr() as *const i8;
        args.arch_name_size = arch_name.len();
        args.num_warps = num_warps;
        args.num_ctas = num_ctas;
        args.num_stages = num_stages;

        let ext_fn = self
            .raw
            .compile
            .expect("PJRT_Triton_Compile not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        // Convert the output ASM to a String
        let asm_code = if args.out_asm.is_null() {
            String::new()
        } else {
            unsafe {
                std::slice::from_raw_parts(args.out_asm as *const u8, args.out_asm_size)
                    .to_vec()
                    .into_iter()
                    .map(|b| b as char)
                    .collect()
            }
        };

        Ok(TritonCompileResult {
            asm_code,
            asm_size: args.out_asm_size,
            smem_bytes: args.out_smem_bytes,
        })
    }
}
