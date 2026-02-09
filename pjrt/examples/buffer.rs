//! Buffer Operations Example
//!
//! This example demonstrates various buffer operations:
//! 1. Creating host buffers from data
//! 2. Transferring data between host and device (async/sync)
//! 3. Copying buffers between devices
//! 4. Managing buffer layouts and dimensions
//!
//! This example showcases both synchronous and asynchronous operations,
//! which is essential for understanding performance characteristics
//! of different transfer methods.

use pjrt::{self, Client, HostBuffer, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    println!("{:?}", api.plugin_attributes()?);

    let client = Client::builder(&api).build()?;

    let host_buf = HostBuffer::from_data(vec![1.0f32, 2.0, 3.0, 4.0], Some(vec![2, 2]), None);
    println!("{:?}", host_buf);

    let dev1 = client.lookup_addressable_device(0)?;
    let dev2 = client.lookup_addressable_device(1)?;

    println!("-- ASYNC --");
    let dev_buf = host_buf.to(&dev1).await?;
    println!("to {:?}, {:?}", dev_buf.dims()?, dev_buf.layout()?);

    let b = dev_buf.to_host(None).await?;
    println!("to_host {:?}", b);

    let b = dev_buf.to_device(&dev2).await?;
    println!("to_device {:?}, {:?}", b.dims()?, b.layout()?);

    println!("-- SYNC --");
    let dev_buf = host_buf.to_sync(&dev1).copy()?;
    println!("to_sync {:?}, {:?}", dev_buf.dims()?, dev_buf.layout()?);

    let b = dev_buf.to_host_sync(None)?;
    println!("to_host_sync {:?}", b);

    let b = dev_buf.to_device_sync(&dev2).copy()?;
    println!("to_device_sync {:?}, {:?}", b.dims()?, b.layout()?);

    Ok(())
}
