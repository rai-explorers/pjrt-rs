//! PJRT Compilation
//!
//! This module provides types and traits for compiling PJRT programs.
//! It includes:
//!
//! - `CompileOptions`: Configuration for the compilation process
//! - `ExecutableBuildOptions`: Device-specific build options
//! - `CompileToExecutable`: Trait for compiling to executables
//! - `CompileToLoadedExecutable`: Trait for compiling to loaded executables
//!
//! The module provides both compile-time and runtime configuration options
//! for controlling how programs are compiled to device executables.

use pjrt_sys::protos::xla::{
    CompilationEnvironmentsProto, CompileOptionsProto, ExecutableBuildOptionsProto,
};
use prost::Message;

use crate::{Client, Executable, LoadedExecutable, Result, TopologyDescription};

/// Trait for types that can compile programs to executables.
///
/// This trait is implemented by `Api` and allows compiling programs
/// without a client, using only a topology description.
pub trait CompileToExecutable<T> {
    fn compile(
        &self,
        program: &T,
        topology: &TopologyDescription,
        options: &CompileOptions,
        client: Option<&Client>,
    ) -> Result<Executable>;
}

/// Trait for types that can compile programs to loaded executables.
///
/// This trait is implemented by `Client` and allows compiling programs
/// directly to loaded executables that are ready for execution.
pub trait CompileToLoadedExecutable<T> {
    fn compile(&self, program: &T, options: &CompileOptions) -> Result<LoadedExecutable>;
}

/// Configuration options for PJRT compilation.
///
/// `CompileOptions` provides a high-level interface for configuring the
/// compilation process. It wraps XLA's `CompileOptionsProto` and provides
/// convenient builder methods.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    proto: CompileOptionsProto,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CompileOptions {
    pub fn new() -> Self {
        let v = Self {
            proto: CompileOptionsProto::default(),
        };
        v.executable_build_options(ExecutableBuildOptions::new())
    }

    pub fn proto(&self) -> &CompileOptionsProto {
        &self.proto
    }

    pub fn proto_mut(&mut self) -> &mut CompileOptionsProto {
        &mut self.proto
    }

    pub fn executable_build_options(
        mut self,
        options: impl Into<Option<ExecutableBuildOptions>>,
    ) -> Self {
        self.proto.executable_build_options = options.into().map(|v| v.proto);
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        self.proto.encode_to_vec()
    }
}

/// Device-specific options for building executables.
///
/// `ExecutableBuildOptions` configures how a compiled program is built
/// for specific devices, including device selection, partitioning,
/// and various optimization options.
#[derive(Debug, Clone)]
pub struct ExecutableBuildOptions {
    proto: ExecutableBuildOptionsProto,
}

impl ExecutableBuildOptions {
    pub fn new() -> Self {
        let proto = ExecutableBuildOptionsProto {
            device_ordinal: -1,
            num_partitions: 1,
            num_replicas: 1,
            ..Default::default()
        };
        Self { proto }
    }

    pub fn proto(&self) -> &ExecutableBuildOptionsProto {
        &self.proto
    }

    pub fn proto_mut(&mut self) -> &mut ExecutableBuildOptionsProto {
        &mut self.proto
    }

    /// If set, this is the device to build the computation for. Valid
    /// device_ordinal values are: 0 to # of devices - 1. These values are
    /// identical to the device ordinal values used by StreamExecutor. The built
    /// executable will be executable on any device equivalent to the specified
    /// device as determined by Backend::devices_equivalent(). A value of -1
    /// indicates this option has not been set.
    pub fn device_ordinal(mut self, device_ordinal: i64) -> Self {
        self.proto.device_ordinal = device_ordinal;
        self
    }

    /// The number of replicas of this computation that are to be executed.
    pub fn num_partitions(mut self, num_partitions: i64) -> Self {
        self.proto.num_partitions = num_partitions;
        self
    }

    /// The number of partitions in this computation.
    pub fn num_replicas(mut self, num_replicas: i64) -> Self {
        self.proto.num_replicas = num_replicas;
        self
    }

    /// Indicates whether to use SPMD (true) or MPMD (false) partitioning when
    /// num_partitions > 1 and XLA is requested to partition the input program.
    pub fn use_spmd_partitioning(mut self, use_spmd_partitioning: bool) -> Self {
        self.proto.use_spmd_partitioning = use_spmd_partitioning;
        self
    }

    /// Whether to automatically generate XLA shardings for SPMD partitioner.
    pub fn use_auto_spmd_partitioning(mut self, use_auto_spmd_partitioning: bool) -> Self {
        self.proto.use_auto_spmd_partitioning = use_auto_spmd_partitioning;
        self
    }

    /// Whether HLOs should be deduplicated.
    pub fn deduplicate_hlo(mut self, deduplicate_hlo: bool) -> Self {
        self.proto.deduplicate_hlo = deduplicate_hlo;
        self
    }

    /// Whether input and output buffers are aliased if the associated parameter is
    /// passed-through XLA modules without being changed.
    pub fn alias_passthrough_params(mut self, alias_passthrough_params: bool) -> Self {
        self.proto.alias_passthrough_params = alias_passthrough_params;
        self
    }

    /// By default, XLA builds an executable by invoking standard compilation, i.e.
    /// running Compiler::Compile, or both Compiler::RunHloPasses and
    /// Compiler::RunBackend. When run_backend_only is set to true, XLA builds an
    /// executable by invoking only RunBackend and skip invoking RunHloPasses,
    /// which can be used to compile post-optimizations HLO modules.
    pub fn run_backend_only(mut self, run_backend_only: bool) -> Self {
        self.proto.run_backend_only = run_backend_only;
        self
    }

    /// Allows sharding propagation to propagate to the parameters. This changes
    /// the input shape of the computation (which is undesirable), but it can be
    /// used to allow to run partial compilation to determine what would be the
    /// input sharding of a computation if XLA would be allowed to propagate the
    /// sharding which can be used by higher level framework as a way to query
    /// intermediate sharding of operations when multiple computation would be
    /// chained and merged together.
    /// This is a vector of bool, because the user can control which parameters can
    /// have the sharding substituted. If only one boolean value is passed in the
    /// vector that is interpreted as the value to be applied for every parameter.
    pub fn allow_spmd_sharding_propagation_to_parameters(
        mut self,
        allow_spmd_sharding_propagation_to_parameters: Vec<bool>,
    ) -> Self {
        self.proto.allow_spmd_sharding_propagation_to_parameters =
            allow_spmd_sharding_propagation_to_parameters;
        self
    }

    /// Allows sharding propagation to propagate to the outputs. This changes the
    /// output shape of the computation (which is undesirable), but it can be used
    /// to allow to run partial compilation to determine what would be the output
    /// sharding of a computation if XLA would be allowed to propagate the sharding
    /// which can be used by higher level framework as a way to query intermediate
    /// sharding of operations when multiple computation would be chained and
    /// merged together.
    /// This is a vector of bool, because the user can control (if the output of
    /// the computation is a tuple) which elements of the tuple can have the
    /// sharding substituted and which don't. If only one boolean value is passed
    /// in the vector that's interpreted as the value to be applied for every
    /// single element of the output tuple. One value per element of the tuple
    /// means that each value is attached to one of the output elements.
    pub fn allow_spmd_sharding_propagation_to_output(
        mut self,
        allow_spmd_sharding_propagation_to_output: Vec<bool>,
    ) -> Self {
        self.proto.allow_spmd_sharding_propagation_to_output =
            allow_spmd_sharding_propagation_to_output;
        self
    }

    pub fn device_memory_size(mut self, device_memory_size: i64) -> Self {
        self.proto.device_memory_size = device_memory_size;
        self
    }

    /// Mesh shape in auto sharding options.
    pub fn auto_spmd_partitioning_mesh_shape(
        mut self,
        auto_spmd_partitioning_mesh_shape: Vec<i64>,
    ) -> Self {
        self.proto.auto_spmd_partitioning_mesh_shape = auto_spmd_partitioning_mesh_shape;
        self
    }

    /// Mesh ids in auto sharding options.
    pub fn auto_spmd_partitioning_mesh_ids(
        mut self,
        auto_spmd_partitioning_mesh_ids: Vec<i64>,
    ) -> Self {
        self.proto.auto_spmd_partitioning_mesh_ids = auto_spmd_partitioning_mesh_ids;
        self
    }

    /// Use Shardy, a new partitioner, to replace the existing
    /// ShardingPropagation and SpmdPartitioner.
    pub fn use_shardy_partitioner(mut self, use_shardy_partitioner: bool) -> Self {
        self.proto.use_shardy_partitioner = use_shardy_partitioner;
        self
    }

    /// Expose access to the XLA debug options which will be passed to the
    /// compilation process.
    pub fn debug_options(mut self, debug_options: impl Into<Option<DebugOptions>>) -> Self {
        self.proto.debug_options = debug_options.into().map(|v| v.proto);
        self
    }

    /// Expose access to the XLA compilation environments, which will be passed to
    /// the compilation process.
    pub fn comp_envs(mut self, comp_envs: impl Into<Option<CompilationEnvironments>>) -> Self {
        self.proto.comp_envs = comp_envs.into().map(|v| v.proto);
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        self.proto.encode_to_vec()
    }
}

impl Default for ExecutableBuildOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DebugOptions {
    proto: pjrt_sys::protos::xla::DebugOptions,
}

impl DebugOptions {
    pub fn new() -> Self {
        Self {
            proto: pjrt_sys::protos::xla::DebugOptions::default(),
        }
    }

    pub fn proto(&self) -> &pjrt_sys::protos::xla::DebugOptions {
        &self.proto
    }

    pub fn proto_mut(&mut self) -> &mut pjrt_sys::protos::xla::DebugOptions {
        &mut self.proto
    }

    pub fn encode(&self) -> Vec<u8> {
        self.proto.encode_to_vec()
    }
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CompilationEnvironments {
    proto: CompilationEnvironmentsProto,
}

impl CompilationEnvironments {
    pub fn new() -> Self {
        Self {
            proto: CompilationEnvironmentsProto::default(),
        }
    }

    pub fn proto(&self) -> &CompilationEnvironmentsProto {
        &self.proto
    }

    pub fn proto_mut(&mut self) -> &mut CompilationEnvironmentsProto {
        &mut self.proto
    }

    pub fn encode(&self) -> Vec<u8> {
        self.proto.encode_to_vec()
    }
}

impl Default for CompilationEnvironments {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_options_new() {
        let options = CompileOptions::new();
        assert!(options.proto().executable_build_options.is_some());
    }

    #[test]
    fn test_compile_options_default() {
        let options: CompileOptions = Default::default();
        assert!(options.proto().executable_build_options.is_some());
    }

    #[test]
    fn test_compile_options_encode() {
        let options = CompileOptions::new();
        let encoded = options.encode();
        assert!(!encoded.is_empty());
        // Should be valid protobuf
    }

    #[test]
    fn test_compile_options_executable_build_options() {
        let build_options = ExecutableBuildOptions::new()
            .device_ordinal(0)
            .num_replicas(2);

        let options = CompileOptions::new().executable_build_options(build_options);
        let build_opts = options.proto().executable_build_options.as_ref().unwrap();
        assert_eq!(build_opts.device_ordinal, 0);
        assert_eq!(build_opts.num_replicas, 2);
    }

    #[test]
    fn test_executable_build_options_new() {
        let opts = ExecutableBuildOptions::new();
        assert_eq!(opts.proto().device_ordinal, -1);
        assert_eq!(opts.proto().num_partitions, 1);
        assert_eq!(opts.proto().num_replicas, 1);
    }

    #[test]
    fn test_executable_build_options_default() {
        let opts: ExecutableBuildOptions = Default::default();
        assert_eq!(opts.proto().device_ordinal, -1);
        assert_eq!(opts.proto().num_partitions, 1);
        assert_eq!(opts.proto().num_replicas, 1);
    }

    #[test]
    fn test_executable_build_options_device_ordinal() {
        let opts = ExecutableBuildOptions::new().device_ordinal(5);
        assert_eq!(opts.proto().device_ordinal, 5);
    }

    #[test]
    fn test_executable_build_options_num_partitions() {
        let opts = ExecutableBuildOptions::new().num_partitions(4);
        assert_eq!(opts.proto().num_partitions, 4);
    }

    #[test]
    fn test_executable_build_options_num_replicas() {
        let opts = ExecutableBuildOptions::new().num_replicas(8);
        assert_eq!(opts.proto().num_replicas, 8);
    }

    #[test]
    fn test_executable_build_options_use_spmd_partitioning() {
        let opts = ExecutableBuildOptions::new().use_spmd_partitioning(true);
        assert!(opts.proto().use_spmd_partitioning);
    }

    #[test]
    fn test_executable_build_options_use_auto_spmd_partitioning() {
        let opts = ExecutableBuildOptions::new().use_auto_spmd_partitioning(true);
        assert!(opts.proto().use_auto_spmd_partitioning);
    }

    #[test]
    fn test_executable_build_options_deduplicate_hlo() {
        let opts = ExecutableBuildOptions::new().deduplicate_hlo(true);
        assert!(opts.proto().deduplicate_hlo);
    }

    #[test]
    fn test_executable_build_options_alias_passthrough_params() {
        let opts = ExecutableBuildOptions::new().alias_passthrough_params(true);
        assert!(opts.proto().alias_passthrough_params);
    }

    #[test]
    fn test_executable_build_options_run_backend_only() {
        let opts = ExecutableBuildOptions::new().run_backend_only(true);
        assert!(opts.proto().run_backend_only);
    }

    #[test]
    fn test_executable_build_options_device_memory_size() {
        let opts = ExecutableBuildOptions::new().device_memory_size(16_000_000_000);
        assert_eq!(opts.proto().device_memory_size, 16_000_000_000);
    }

    #[test]
    fn test_executable_build_options_use_shardy_partitioner() {
        let opts = ExecutableBuildOptions::new().use_shardy_partitioner(true);
        assert!(opts.proto().use_shardy_partitioner);
    }

    #[test]
    fn test_executable_build_options_spmd_sharding_propagation() {
        let opts = ExecutableBuildOptions::new()
            .allow_spmd_sharding_propagation_to_parameters(vec![true, false, true])
            .allow_spmd_sharding_propagation_to_output(vec![false, true]);

        assert_eq!(
            opts.proto().allow_spmd_sharding_propagation_to_parameters,
            vec![true, false, true]
        );
        assert_eq!(
            opts.proto().allow_spmd_sharding_propagation_to_output,
            vec![false, true]
        );
    }

    #[test]
    fn test_executable_build_options_auto_spmd_mesh() {
        let opts = ExecutableBuildOptions::new()
            .auto_spmd_partitioning_mesh_shape(vec![2, 4])
            .auto_spmd_partitioning_mesh_ids(vec![0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(opts.proto().auto_spmd_partitioning_mesh_shape, vec![2, 4]);
        assert_eq!(
            opts.proto().auto_spmd_partitioning_mesh_ids,
            vec![0, 1, 2, 3, 4, 5, 6, 7]
        );
    }

    #[test]
    fn test_executable_build_options_encode() {
        let opts = ExecutableBuildOptions::new()
            .device_ordinal(1)
            .num_replicas(2)
            .num_partitions(4);

        let encoded = opts.encode();
        assert!(!encoded.is_empty());
        // Should be valid protobuf
    }

    #[test]
    fn test_executable_build_options_chaining() {
        let opts = ExecutableBuildOptions::new()
            .device_ordinal(0)
            .num_replicas(2)
            .num_partitions(4)
            .use_spmd_partitioning(true)
            .deduplicate_hlo(false)
            .alias_passthrough_params(true);

        assert_eq!(opts.proto().device_ordinal, 0);
        assert_eq!(opts.proto().num_replicas, 2);
        assert_eq!(opts.proto().num_partitions, 4);
        assert!(opts.proto().use_spmd_partitioning);
        assert!(!opts.proto().deduplicate_hlo);
        assert!(opts.proto().alias_passthrough_params);
    }

    #[test]
    fn test_debug_options_new() {
        let opts = DebugOptions::new();
        // Just verify it creates successfully
        let _proto = opts.proto();
    }

    #[test]
    fn test_debug_options_default() {
        let opts: DebugOptions = Default::default();
        let _proto = opts.proto();
    }

    #[test]
    fn test_debug_options_encode() {
        let opts = DebugOptions::new();
        let encoded = opts.encode();
        // An all-default proto may encode to empty bytes, which is valid
        // Just verify the method works without panicking
        let _ = encoded;
    }

    #[test]
    fn test_compilation_environments_new() {
        let envs = CompilationEnvironments::new();
        let _proto = envs.proto();
    }

    #[test]
    fn test_compilation_environments_default() {
        let envs: CompilationEnvironments = Default::default();
        let _proto = envs.proto();
    }

    #[test]
    fn test_compilation_environments_encode() {
        let envs = CompilationEnvironments::new();
        let encoded = envs.encode();
        // An all-default proto may encode to empty bytes, which is valid
        // Just verify the method works without panicking
        let _ = encoded;
    }

    #[test]
    fn test_compile_options_proto_mut() {
        let mut options = CompileOptions::new();
        {
            let proto = options.proto_mut();
            proto.argument_layouts.push(Default::default());
        }
        assert_eq!(options.proto().argument_layouts.len(), 1);
    }

    #[test]
    fn test_executable_build_options_proto_mut() {
        let mut opts = ExecutableBuildOptions::new();
        {
            let proto = opts.proto_mut();
            proto.device_ordinal = 42;
        }
        assert_eq!(opts.proto().device_ordinal, 42);
    }
}
