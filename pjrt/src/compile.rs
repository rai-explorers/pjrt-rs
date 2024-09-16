use pjrt_sys::protos::xla::{
    CompilationEnvironmentsProto, CompileOptionsProto, ExecutableBuildOptionsProto,
};
use prost::Message;

use crate::{Client, Executable, LoadedExecutable, Result, TopologyDescription};

pub trait CompileToExecutable<T> {
    fn compile(
        &self,
        program: &T,
        topology: &TopologyDescription,
        options: &CompileOptions,
        client: Option<&Client>,
    ) -> Result<Executable>;
}

pub trait CompileToLoadedExecutable<T> {
    fn compile(&self, program: &T, options: &CompileOptions) -> Result<LoadedExecutable>;
}

pub struct CompileOptions {
    proto: CompileOptionsProto,
}

impl CompileOptions {
    pub fn new() -> Self {
        let mut v = Self {
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

pub struct ExecutableBuildOptions {
    proto: ExecutableBuildOptionsProto,
}

impl ExecutableBuildOptions {
    pub fn new() -> Self {
        let mut proto = ExecutableBuildOptionsProto::default();
        proto.device_ordinal = -1;
        proto.num_partitions = 1;
        proto.num_replicas = 1;
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
