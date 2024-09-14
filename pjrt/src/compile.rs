use pjrt_sys::protos::xla::{CompileOptionsProto, ExecutableBuildOptionsProto};
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
        v.build_options(ExecutableBuildOptions::new());
        v
    }

    pub fn proto(&self) -> &CompileOptionsProto {
        &self.proto
    }

    pub fn proto_mut(&mut self) -> &mut CompileOptionsProto {
        &mut self.proto
    }

    pub fn build_options(&mut self, build_options: ExecutableBuildOptions) {
        self.proto
            .executable_build_options
            .replace(build_options.proto);
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

    pub fn device_ordinal(mut self, device_ordinal: i64) -> Self {
        self.proto.device_ordinal = device_ordinal;
        self
    }

    pub fn num_partitions(mut self, num_partitions: i64) -> Self {
        self.proto.num_partitions = num_partitions;
        self
    }

    pub fn num_replicas(mut self, num_replicas: i64) -> Self {
        self.proto.num_replicas = num_replicas;
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        self.proto.encode_to_vec()
    }
}
