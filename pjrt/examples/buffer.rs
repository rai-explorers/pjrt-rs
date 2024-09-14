use pjrt::{self, HostBuffer, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let api = pjrt::load_plugin("pjrt_c_api_cpu_plugin.so")?;
    println!("{:?}", api.plugin_attributes());

    let client = api.create_client([])?;

    let host_buf = HostBuffer::new([1.0f32, 2.0, 3.0, 4.0], [2, 2]);
    println!("{:?}", host_buf);

    let dev1 = client.lookup_addressable_device(0)?;
    let dev2 = client.lookup_addressable_device(1)?;

    println!("-- ASYNC --");
    let dev_buf = host_buf.copy_to(&dev1).await?;
    println!("to {:?}, {:?}", dev_buf.dimensions(), dev_buf.layout());

    let b = dev_buf.copy_to_host().await?;
    println!("to_host_buffer {:?}", b);

    let b = dev_buf.copy_to_device(&dev2).await?;
    println!("copy_to_device {:?}, {:?}", b.dimensions(), b.layout());

    println!("-- SYNC --");
    let dev_buf = host_buf.copy_to_sync(&dev1)?;
    println!("to {:?}, {:?}", dev_buf.dimensions(), dev_buf.layout());

    let b = dev_buf.copy_to_host_sync()?;
    println!("to_host_buffer {:?}", b);

    let b = dev_buf.copy_to_device_sync(&dev2)?;
    println!("copy_to_device {:?}, {:?}", b.dimensions(), b.layout());

    Ok(())
}
