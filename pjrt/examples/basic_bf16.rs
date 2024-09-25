use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, HostBuffer, LoadedExecutable, Result};

const CODE: &[u8] = include_bytes!("prog_bf16.mlir");

fn main() -> Result<()> {
    let api = pjrt::plugin("pjrt_c_api_cpu_plugin.so").load()?;
    println!("api_version = {:?}", api.version());

    let client = Client::builder(&api).build()?;
    println!("platform_name = {}", client.platform_name());

    let program = pjrt::Program::new(MLIR, CODE);

    let loaded_executable = LoadedExecutable::builder(&client, &program).build()?;

    let a = HostBuffer::from_scalar(half::bf16::from_f32(1.25));
    println!("input = {:?}", a);

    let inputs = a.to_sync(&client).copy()?;

    let result = loaded_executable.execution(inputs).run_sync()?;

    let ouput = &result[0][0];
    let output = ouput.to_host_sync().copy()?;
    println!("output= {:?}", output);

    Ok(())
}
