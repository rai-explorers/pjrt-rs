use pjrt::ProgramFormat::MLIR;
use pjrt::{self, HostBuffer, Result};

const CODE: &'static [u8] = include_bytes!("program.mlir");

fn main() -> Result<()> {
    let api = pjrt::load_plugin("pjrt_c_api_cpu_plugin.so")?;
    println!("api_version = {:?}", api.version());

    let client = api.client().create()?;
    println!("platform_name = {}", client.platform_name());

    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_executable = client.program(&program).compile()?;
    println!("compiled");

    let a = HostBuffer::scalar(1.0f32);
    println!("input = {:?}", a);

    let inputs = a.copy_to_sync(&client)?;

    let result = loaded_executable.execute_sync(inputs)?;

    let ouput = &result[0][0];
    let output = ouput.copy_to_host_sync()?;
    println!("output= {:?}", output);

    Ok(())
}
