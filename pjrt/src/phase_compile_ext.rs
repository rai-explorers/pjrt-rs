//! PJRT Phase Compile Extension
//!
//! This module provides safe Rust bindings for the PJRT Phase Compile extension.
//! This extension provides the ability to interact with individual compilation phases,
//! which is essential for caching intermediate artifacts and facilitating debugging.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::phase_compile::{PhaseCompileExtension, PhaseCompiler};
//!
//! // Get the phase compile extension
//! let phase_ext = api.extension::<PhaseCompileExtension>()?;
//!
//! // Get a phase compiler
//! let compiler = phase_ext.get_compiler()?;
//!
//! // Get phase names
//! let phases = compiler.get_phase_names()?;
//!
//! // Run specific phases
//! let output = compiler.run_phases(&input_programs, &["phase1", "phase2"], &options, &topology)?;
//! ```

use std::rc::Rc;

use pjrt_sys::{
    PJRT_PhaseCompile_C_Buffers_Destroy_Args, PJRT_PhaseCompile_Destroy_Compiler_Args,
    PJRT_PhaseCompile_Extension, PJRT_PhaseCompile_Get_Compiler_Args,
    PJRT_PhaseCompile_Get_PhaseNames_Args, PJRT_PhaseCompile_Run_Phase_Args, PJRT_PhaseCompiler,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, CompileOptions, Result, TopologyDescription};

/// Safe wrapper for PJRT Phase Compile extension
///
/// This extension provides access to individual compilation phases for
/// debugging and caching purposes.
pub struct PhaseCompileExtension {
    raw: Rc<PJRT_PhaseCompile_Extension>,
    api: Api,
}

impl std::fmt::Debug for PhaseCompileExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhaseCompileExtension")
            .field("api_version", &1i32)
            .finish()
    }
}

unsafe impl Extension for PhaseCompileExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::PhaseCompile
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let ext = ptr as *mut PJRT_PhaseCompile_Extension;
        if (*ext).base.type_ != ExtensionType::PhaseCompile.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new(*ext),
            api: api.clone(),
        })
    }
}

/// A phase compiler for running specific compilation phases
pub struct PhaseCompiler {
    raw: *const PJRT_PhaseCompiler,
    ext: PhaseCompileExtension,
}

impl std::fmt::Debug for PhaseCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhaseCompiler")
            .field("ptr", &self.raw)
            .finish()
    }
}

impl Drop for PhaseCompiler {
    fn drop(&mut self) {
        let mut args = PJRT_PhaseCompile_Destroy_Compiler_Args {
            struct_size: std::mem::size_of::<PJRT_PhaseCompile_Destroy_Compiler_Args>(),
            extension_start: std::ptr::null_mut(),
            phase_compiler: self.raw,
        };

        if let Some(destroy_fn) = self.ext.raw.phase_compile_destroy_compiler {
            unsafe {
                destroy_fn(&mut args);
            }
        }
    }
}

/// Output from running compilation phases
pub struct PhaseCompileOutput {
    /// Output programs as serialized byte arrays
    pub output_programs: Vec<Vec<u8>>,
}

impl PhaseCompileExtension {
    /// Get a phase compiler
    ///
    /// Returns a phase compiler that can be used to run specific compilation phases.
    /// The caller is responsible for freeing the compiler using `drop()`.
    pub fn get_compiler(&self) -> Result<PhaseCompiler> {
        let mut args = PJRT_PhaseCompile_Get_Compiler_Args {
            struct_size: std::mem::size_of::<PJRT_PhaseCompile_Get_Compiler_Args>(),
            extension_start: std::ptr::null_mut(),
            phase_compiler: std::ptr::null(),
        };

        let ext_fn = self
            .raw
            .phase_compile_get_compiler
            .expect("PJRT_PhaseCompile_Get_Compiler not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        if args.phase_compiler.is_null() {
            return Err(crate::Error::NullPointer);
        }

        Ok(PhaseCompiler {
            raw: args.phase_compiler,
            ext: PhaseCompileExtension {
                raw: self.raw.clone(),
                api: self.api.clone(),
            },
        })
    }
}

impl PhaseCompiler {
    /// Get the names of all registered phases in order
    ///
    /// Returns a list of phase names that can be used with `run_phases()`.
    pub fn get_phase_names(&self) -> Result<Vec<String>> {
        let mut args = PJRT_PhaseCompile_Get_PhaseNames_Args {
            struct_size: std::mem::size_of::<PJRT_PhaseCompile_Get_PhaseNames_Args>(),
            extension_start: std::ptr::null_mut(),
            phase_compiler: self.raw,
            phase_names: std::ptr::null_mut(),
            phase_names_sizes: std::ptr::null(),
            num_phase_names: 0,
        };

        let ext_fn = self
            .ext
            .raw
            .phase_compile_get_phase_names
            .expect("PJRT_PhaseCompile_Get_PhaseNames not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.ext.api.err_or(err, ())?;

        if args.phase_names.is_null() || args.num_phase_names == 0 {
            return Ok(Vec::new());
        }

        let names = unsafe {
            let name_ptrs = std::slice::from_raw_parts(args.phase_names, args.num_phase_names);
            let name_sizes =
                std::slice::from_raw_parts(args.phase_names_sizes, args.num_phase_names);

            let result: Vec<String> = name_ptrs
                .iter()
                .zip(name_sizes.iter())
                .map(|(&ptr, &size)| {
                    if ptr.is_null() {
                        String::new()
                    } else {
                        let slice = std::slice::from_raw_parts(ptr as *const u8, size);
                        String::from_utf8_lossy(slice).to_string()
                    }
                })
                .collect();

            // Clean up the buffers
            if let Some(destroy_fn) = self.ext.raw.phase_compile_c_buffers_destroy {
                let mut destroy_args = PJRT_PhaseCompile_C_Buffers_Destroy_Args {
                    struct_size: std::mem::size_of::<PJRT_PhaseCompile_C_Buffers_Destroy_Args>(),
                    extension_start: std::ptr::null_mut(),
                    char_buffers: args.phase_names,
                    char_buffer_sizes: args.phase_names_sizes as *mut usize,
                    num_char_buffers: args.num_phase_names,
                };
                destroy_fn(&mut destroy_args);
            }

            result
        };

        Ok(names)
    }

    /// Run specific compilation phases
    ///
    /// # Arguments
    ///
    /// * `input_programs` - Serialized xla::PjRtPartialProgramProto programs
    /// * `phases_to_run` - Names of phases to run
    /// * `compile_options` - Compile options for the compilation
    /// * `topology` - Device topology description
    ///
    /// # Returns
    ///
    /// Output programs after running the specified phases
    pub fn run_phases(
        &self,
        input_programs: &[Vec<u8>],
        phases_to_run: &[String],
        compile_options: &CompileOptions,
        topology: &TopologyDescription,
    ) -> Result<PhaseCompileOutput> {
        // Convert input programs to C-compatible format
        let input_programs_ptrs: Vec<*const i8> = input_programs
            .iter()
            .map(|p| p.as_ptr() as *const i8)
            .collect();
        let input_programs_sizes: Vec<usize> = input_programs.iter().map(|p| p.len()).collect();

        // Convert phase names to C-compatible format
        let phase_names_cstrings: Vec<std::ffi::CString> = phases_to_run
            .iter()
            .map(|s| std::ffi::CString::new(s.as_str()).expect("phase name contains null"))
            .collect();
        let phase_names_ptrs: Vec<*const i8> =
            phase_names_cstrings.iter().map(|s| s.as_ptr()).collect();
        let phase_names_sizes: Vec<usize> = phases_to_run.iter().map(|s| s.len()).collect();

        // Serialize compile options
        let serialized_options = compile_options.encode();

        let mut args = PJRT_PhaseCompile_Run_Phase_Args {
            struct_size: std::mem::size_of::<PJRT_PhaseCompile_Run_Phase_Args>(),
            extension_start: std::ptr::null_mut(),
            phase_compiler: self.raw,
            input_programs: if input_programs_ptrs.is_empty() {
                std::ptr::null_mut()
            } else {
                input_programs_ptrs.as_ptr() as *mut *const i8
            },
            input_programs_sizes: if input_programs_sizes.is_empty() {
                std::ptr::null()
            } else {
                input_programs_sizes.as_ptr()
            },
            num_input_programs: input_programs.len(),
            phases_to_run: if phase_names_ptrs.is_empty() {
                std::ptr::null_mut()
            } else {
                phase_names_ptrs.as_ptr() as *mut *const i8
            },
            phases_to_run_sizes: if phase_names_sizes.is_empty() {
                std::ptr::null()
            } else {
                phase_names_sizes.as_ptr()
            },
            num_phases_to_run: phases_to_run.len(),
            compile_options: serialized_options.as_ptr() as *const i8,
            compile_options_size: serialized_options.len(),
            topology: topology.ptr,
            output_programs: std::ptr::null_mut(),
            output_programs_sizes: std::ptr::null(),
            num_output_programs: 0,
        };

        let ext_fn = self
            .ext
            .raw
            .phase_compile_run_phases
            .expect("PJRT_PhaseCompile_Run_Phase not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.ext.api.err_or(err, ())?;

        // Extract output programs
        let output_programs = if args.output_programs.is_null() || args.num_output_programs == 0 {
            Vec::new()
        } else {
            unsafe {
                let output_ptrs =
                    std::slice::from_raw_parts(args.output_programs, args.num_output_programs);
                let output_sizes = std::slice::from_raw_parts(
                    args.output_programs_sizes,
                    args.num_output_programs,
                );

                let result: Vec<Vec<u8>> = output_ptrs
                    .iter()
                    .zip(output_sizes.iter())
                    .map(|(&ptr, &size)| {
                        if ptr.is_null() {
                            Vec::new()
                        } else {
                            let slice = std::slice::from_raw_parts(ptr as *const u8, size);
                            slice.to_vec()
                        }
                    })
                    .collect();

                // Clean up the output buffers
                if let Some(destroy_fn) = self.ext.raw.phase_compile_c_buffers_destroy {
                    let mut destroy_args = PJRT_PhaseCompile_C_Buffers_Destroy_Args {
                        struct_size: std::mem::size_of::<PJRT_PhaseCompile_C_Buffers_Destroy_Args>(
                        ),
                        extension_start: std::ptr::null_mut(),
                        char_buffers: args.output_programs,
                        char_buffer_sizes: args.output_programs_sizes as *mut usize,
                        num_char_buffers: args.num_output_programs,
                    };
                    destroy_fn(&mut destroy_args);
                }

                result
            }
        };

        Ok(PhaseCompileOutput { output_programs })
    }
}
