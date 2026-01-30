use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, HostBuffer, LoadedExecutable, Result};

const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    println!("api_version = {:?}", api.version());

    let client = Client::builder(&api).build()?;

    println!("platform_name = {}", client.platform_name());

    let program = pjrt::Program::new(MLIR, CODE);

    let loaded_executable = LoadedExecutable::builder(&client, &program).build()?;

    let a = HostBuffer::from_scalar(1.25f32);
    println!("input = {:?}", a);

    let inputs = a.to_sync(&client).copy()?;

    let result = loaded_executable.execution(inputs).run_sync()?;

    let ouput = &result[0][0];
    let output = ouput.to_host_sync().copy()?;
    println!("output= {:?}", output);

    Ok(())
}
