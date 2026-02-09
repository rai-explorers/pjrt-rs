//! PJRT Executable
//!
//! This module provides the `Executable` struct and related types for managing
//! compiled PJRT programs. An executable represents a fully compiled program
//! that can be loaded onto devices for execution.
//!
//! The module provides:
//!
//! - `Executable`: A compiled program ready to be loaded
//! - `SerializedExecutable`: A serialized form of an executable
//! - `SerializedCompileOptions`: Serialized compilation options
//! - `CompiledMemoryStats`: Memory usage statistics for compiled executables
//!
//! Executables are created by compiling programs through the `Api::compile` or
//! `Client::compile` methods.

use std::borrow::{Borrow, Cow};

use bon::bon;
use pjrt_sys::{
    PJRT_Executable, PJRT_Executable_Destroy_Args, PJRT_Executable_Fingerprint_Args,
    PJRT_Executable_GetCompileOptions_Args, PJRT_Executable_GetCompiledMemoryStats_Args,
    PJRT_Executable_GetCostAnalysis_Args, PJRT_Executable_Name_Args,
    PJRT_Executable_NumOutputs_Args, PJRT_Executable_NumPartitions_Args,
    PJRT_Executable_NumReplicas_Args, PJRT_Executable_OptimizedProgram_Args,
    PJRT_Executable_OutputDimensions_Args, PJRT_Executable_OutputElementTypes_Args,
    PJRT_Executable_OutputMemoryKinds_Args, PJRT_Executable_Serialize_Args,
    PJRT_Executable_SizeOfGeneratedCodeInBytes_Args, PJRT_SerializedCompileOptions,
    PJRT_SerializedExecutable,
};

use crate::program::ProgramFormat;
use crate::{
    utils, Api, Client, CompileOptions, CompileToExecutable, NamedValueMap, PrimitiveType, Program,
    Result, TopologyDescription,
};

/// A compiled PJRT program ready to be loaded onto devices.
///
/// An `Executable` represents the result of compiling a `Program`. It contains
/// the compiled code and metadata about the compilation, but is not yet loaded
/// onto specific devices for execution.
///
/// To execute a program, an `Executable` must first be loaded to create a
/// `LoadedExecutable`.
///
/// # Example
///
/// ```rust,ignore
/// // Compile a program to an executable
/// let executable = api.compile(&program, &topology, options, None)?;
///
/// // Query compilation metadata
/// println!("Name: {}", executable.name());
/// println!("Code size: {} bytes", executable.code_size());
///
/// // Serialize for later use
/// let serialized = executable.serialize();
/// ```
/// # Thread Safety
///
/// `Executable` is `!Send + !Sync` due to the raw `*mut PJRT_Executable`
/// pointer. It must be used on the same thread where it was created.
pub struct Executable {
    api: Api,
    pub(crate) ptr: *mut PJRT_Executable,
}

impl Drop for Executable {
    fn drop(&mut self) {
        let mut args = PJRT_Executable_Destroy_Args::new();
        args.executable = self.ptr;
        let _ = self.api.PJRT_Executable_Destroy(args);
    }
}

impl std::fmt::Debug for Executable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executable")
            .field("name", &self.name().unwrap_or_default())
            .field("num_replicas", &self.num_replicas().unwrap_or(0))
            .field("num_partitions", &self.num_partitions().unwrap_or(0))
            .field("num_outputs", &self.num_outputs().unwrap_or(0))
            .field("code_size", &self.code_size().unwrap_or(0))
            .finish()
    }
}

#[bon]
impl Executable {
    pub(crate) fn wrap(api: &Api, ptr: *mut PJRT_Executable) -> Self {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
        }
    }

    #[builder(finish_fn = build)]
    pub fn builder<T>(
        #[builder(start_fn)] api: &Api,
        #[builder(start_fn)] program: &T,
        #[builder(start_fn)] topology: &TopologyDescription,
        #[builder(default)] options: CompileOptions,
        client: Option<&Client>,
    ) -> Result<Self>
    where
        Api: CompileToExecutable<T>,
    {
        api.compile(program, topology, options, client)
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn name(&self) -> Result<Cow<'_, str>> {
        let mut args = PJRT_Executable_Name_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_Name(args)?;
        Ok(utils::str_from_raw(
            args.executable_name,
            args.executable_name_size,
        ))
    }

    pub fn num_replicas(&self) -> Result<usize> {
        let mut args = PJRT_Executable_NumReplicas_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_NumReplicas(args)?;
        Ok(args.num_replicas)
    }

    pub fn num_partitions(&self) -> Result<usize> {
        let mut args = PJRT_Executable_NumPartitions_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_NumPartitions(args)?;
        Ok(args.num_partitions)
    }

    pub fn num_outputs(&self) -> Result<usize> {
        let mut args = PJRT_Executable_NumOutputs_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_NumOutputs(args)?;
        Ok(args.num_outputs)
    }

    pub fn code_size(&self) -> Result<i64> {
        let mut args = PJRT_Executable_SizeOfGeneratedCodeInBytes_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_SizeOfGeneratedCodeInBytes(args)?;
        Ok(args.size_in_bytes)
    }

    pub fn output_primitive_types(&self) -> Result<Vec<PrimitiveType>> {
        let mut args = PJRT_Executable_OutputElementTypes_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_OutputElementTypes(args)?;
        let s = unsafe { std::slice::from_raw_parts(args.output_types, args.num_output_types) };
        s.iter().map(|s| PrimitiveType::try_from(*s)).collect()
    }

    #[allow(clippy::needless_range_loop)]
    pub fn output_dims(&self) -> Result<Vec<Vec<i64>>> {
        let mut args = PJRT_Executable_OutputDimensions_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_OutputDimensions(args)?;
        let output_dim_size =
            unsafe { std::slice::from_raw_parts(args.dim_sizes, args.num_outputs) };
        let mut out = Vec::with_capacity(args.num_outputs);
        let mut offset = 0usize;
        for i in 0..args.num_outputs {
            let s =
                unsafe { std::slice::from_raw_parts(args.dims.add(offset), output_dim_size[i]) };
            let dims = s.to_owned();
            out.push(dims);
            offset += output_dim_size[i];
        }
        Ok(out)
    }

    pub fn fingerprint(&self) -> Result<Cow<'_, str>> {
        let mut args = PJRT_Executable_Fingerprint_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_Fingerprint(args)?;
        Ok(utils::str_from_raw(
            args.executable_fingerprint,
            args.executable_fingerprint_size,
        ))
    }

    pub fn cost_analysis(&self) -> Result<NamedValueMap> {
        let mut args = PJRT_Executable_GetCostAnalysis_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_GetCostAnalysis(args)?;
        utils::to_named_value_map(args.properties, args.num_properties)
    }

    pub fn optimize(&self) -> Result<Program> {
        let mut args = PJRT_Executable_OptimizedProgram_Args::new();
        args.executable = self.ptr;
        // first call to get the size
        args = self.api.PJRT_Executable_OptimizedProgram(args)?;
        // prepare the code buffer — write directly to the pointed-to struct,
        // not a local copy, so the second call sees the buffer pointer.
        let code_size = unsafe { (*args.program).code_size };
        let mut code: Vec<u8> = vec![0; code_size];
        unsafe {
            (*args.program).code = code.as_mut_ptr() as *mut _;
        }
        // second call to get the code
        args = self.api.PJRT_Executable_OptimizedProgram(args)?;
        let prog = unsafe { *args.program };
        let format = utils::str_from_raw(prog.format, prog.format_size);
        let format = ProgramFormat::try_from(format.borrow())?;
        Ok(Program::new(format, code))
    }

    #[allow(clippy::needless_range_loop)]
    pub fn output_memory_kinds(&self) -> Result<Vec<Cow<'_, str>>> {
        let mut args = PJRT_Executable_OutputMemoryKinds_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_OutputMemoryKinds(args)?;
        let memory_kind_sizes =
            unsafe { std::slice::from_raw_parts(args.memory_kind_sizes, args.num_outputs) };
        let mut out = Vec::with_capacity(args.num_outputs);
        for i in 0..args.num_outputs {
            let ptr = unsafe { *args.memory_kinds.add(i) };
            let kind = utils::str_from_raw(ptr, memory_kind_sizes[i]);
            out.push(kind);
        }
        Ok(out)
    }

    pub fn serialize(&self) -> Result<SerializedExecutable> {
        let mut args = PJRT_Executable_Serialize_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_Serialize(args)?;
        let deleter = args
            .serialized_executable_deleter
            .ok_or_else(|| crate::Error::InvalidArgument("null executable deleter".into()))?;
        Ok(SerializedExecutable {
            ptr: args.serialized_executable,
            deleter,
            data_ptr: args.serialized_bytes as *const u8,
            data_len: args.serialized_bytes_size,
        })
    }

    pub fn compiled_memory_stats(&self) -> Result<CompiledMemoryStats> {
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_GetCompiledMemoryStats(args)?;
        Ok(CompiledMemoryStats::from(args))
    }

    /// Returns the serialized compile options that were used to create this executable.
    ///
    /// The returned bytes represent a serialized `CompileOptionsProto` that can be
    /// deserialized using the XLA protobuf definitions. This is useful for debugging
    /// and for understanding the compilation configuration.
    pub fn compile_options(&self) -> Result<SerializedCompileOptions> {
        let mut args = PJRT_Executable_GetCompileOptions_Args::new();
        args.executable = self.ptr;
        args = self.api.PJRT_Executable_GetCompileOptions(args)?;
        let deleter = args
            .serialized_compile_options_deleter
            .ok_or_else(|| crate::Error::InvalidArgument("null compile_options deleter".into()))?;
        Ok(SerializedCompileOptions {
            ptr: args.serialized_compile_options,
            deleter,
            data_ptr: args.serialized_bytes as *const u8,
            data_len: args.serialized_bytes_size,
        })
    }
}

/// Serialized compilation options from an executable.
///
/// This struct holds the serialized form of the `CompileOptions` that were
/// used to create an `Executable`. The data can be deserialized to inspect
/// the compilation configuration.
pub struct SerializedCompileOptions {
    ptr: *mut PJRT_SerializedCompileOptions,
    deleter: unsafe extern "C" fn(options: *mut PJRT_SerializedCompileOptions),
    data_ptr: *const u8,
    data_len: usize,
}

impl Drop for SerializedCompileOptions {
    fn drop(&mut self) {
        unsafe { (self.deleter)(self.ptr) };
    }
}

impl SerializedCompileOptions {
    /// Returns the serialized compile options as a byte slice.
    ///
    /// This represents a serialized `CompileOptionsProto` that can be deserialized
    /// using the appropriate protobuf library.
    pub fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.data_len) }
    }
}

impl std::fmt::Debug for SerializedCompileOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializedCompileOptions")
            .field("len", &self.data_len)
            .finish()
    }
}

/// A serialized PJRT executable.
///
/// This struct holds the serialized form of an `Executable`, which can be
/// saved to disk or transferred to another process. The executable can later
/// be deserialized using `Client::load_executable`.
///
/// # Example
///
/// ```rust,ignore
/// // Serialize an executable
/// let serialized = executable.serialize();
/// std::fs::write("model.pjrt", serialized.bytes())?;
///
/// // Later, load it back
/// let bytes = std::fs::read("model.pjrt")?;
/// let loaded_executable = client.load_executable(&bytes)?;
/// ```
pub struct SerializedExecutable {
    ptr: *mut PJRT_SerializedExecutable,
    deleter: unsafe extern "C" fn(exec: *mut PJRT_SerializedExecutable),
    data_ptr: *const u8,
    data_len: usize,
}

impl Drop for SerializedExecutable {
    fn drop(&mut self) {
        unsafe { (self.deleter)(self.ptr) };
    }
}

impl SerializedExecutable {
    pub fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.data_len) }
    }
}

impl std::fmt::Debug for SerializedExecutable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializedExecutable")
            .field("len", &self.data_len)
            .finish()
    }
}

/// Memory usage statistics for a compiled executable.
///
/// This struct provides detailed information about memory requirements for
/// both device and host memory when executing a compiled program.
pub struct CompiledMemoryStats {
    /// Size of generated device code in bytes.
    pub generated_code_size_in_bytes: i64,
    /// Size of argument buffers on device in bytes.
    pub argument_size_in_bytes: i64,
    /// Size of output buffers on device in bytes.
    pub output_size_in_bytes: i64,
    /// Size of aliased buffers on device in bytes.
    pub alias_size_in_bytes: i64,
    /// Size of temporary buffers on device in bytes.
    pub temp_size_in_bytes: i64,
    /// Size of generated host code in bytes.
    pub host_generated_code_size_in_bytes: i64,
    /// Size of argument buffers on host in bytes.
    pub host_argument_size_in_bytes: i64,
    /// Size of output buffers on host in bytes.
    pub host_output_size_in_bytes: i64,
    /// Size of aliased buffers on host in bytes.
    pub host_alias_size_in_bytes: i64,
    /// Size of temporary buffers on host in bytes.
    pub host_temp_size_in_bytes: i64,
}

impl From<PJRT_Executable_GetCompiledMemoryStats_Args> for CompiledMemoryStats {
    fn from(value: PJRT_Executable_GetCompiledMemoryStats_Args) -> Self {
        Self {
            generated_code_size_in_bytes: value.generated_code_size_in_bytes,
            argument_size_in_bytes: value.argument_size_in_bytes,
            output_size_in_bytes: value.output_size_in_bytes,
            alias_size_in_bytes: value.alias_size_in_bytes,
            temp_size_in_bytes: value.temp_size_in_bytes,
            host_generated_code_size_in_bytes: value.host_generated_code_size_in_bytes,
            host_argument_size_in_bytes: value.host_argument_size_in_bytes,
            host_output_size_in_bytes: value.host_output_size_in_bytes,
            host_alias_size_in_bytes: value.host_alias_size_in_bytes,
            host_temp_size_in_bytes: value.host_temp_size_in_bytes,
        }
    }
}
