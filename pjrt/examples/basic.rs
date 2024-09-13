use pjrt::protos::xla::ExecutableBuildOptionsProto;
use pjrt::{self, HostBuffer, Result};

const MLIR_STR: &str = r#"
module {
func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
  %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
  %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
  %2 = mhlo.add %0, %1 : tensor<f32>
  return %2 : tensor<f32>
}}"#;

fn main() -> Result<()> {
    let api = pjrt::load_plugin("pjrt_c_api_cpu_plugin.so")?;
    let client = api.create_client([])?;
    println!("platform_name {}", client.platform_name());

    let mut options = pjrt::CompileOptions::new();

    let program = pjrt::Program::with_mlir(MLIR_STR.to_owned());
    let loaded_executable = client.compile(&program, &options)?;

    let a = HostBuffer::scalar(1.0f32);
    println!("input = {:?}", a);

    let inputs = a.copy_to_sync(&client)?;

    let result = loaded_executable.execute_sync(inputs)?;

    let ouput = &result[0][0];
    let output = ouput.copy_to_host_sync()?;
    println!("output= {:?}", output);

    Ok(())
}
