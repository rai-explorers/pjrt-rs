use pjrt::{self, Client, HostBuffer, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let api = pjrt::plugin("pjrt_c_api_cpu_plugin.so").load()?;
    println!("{:?}", api.plugin_attributes());

    let client = Client::builder(&api).build()?;

    let host_buf = HostBuffer::from_data([1.0f32, 2.0, 3.0, 4.0])
        .dims([2, 2])
        .build();
    println!("{:?}", host_buf);

    let dev1 = client.lookup_addressable_device(0)?;
    let dev2 = client.lookup_addressable_device(1)?;

    println!("-- ASYNC --");
    let dev_buf = host_buf.to(&dev1).copy().await?;
    println!("to {:?}, {:?}", dev_buf.dims(), dev_buf.layout());

    let b = dev_buf.to_host().copy().await?;
    println!("to_host {:?}", b);

    let b = dev_buf.to_device(&dev2).copy().await?;
    println!("to_device {:?}, {:?}", b.dims(), b.layout());

    println!("-- SYNC --");
    let dev_buf = host_buf.to_sync(&dev1).copy()?;
    println!("to_sync {:?}, {:?}", dev_buf.dims(), dev_buf.layout());

    let b = dev_buf.to_host_sync().copy()?;
    println!("to_host_sync {:?}", b);

    let b = dev_buf.to_device_sync(&dev2).copy()?;
    println!("to_device_sync {:?}, {:?}", b.dims(), b.layout());

    Ok(())
}
