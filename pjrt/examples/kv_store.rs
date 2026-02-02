//! Key-Value Store Example
//!
//! This example demonstrates how to use the PJRT KeyValueStore trait:
//! 1. Implementing a custom KeyValueStore
//! 2. Using KeyValueStore with Client builder for distributed coordination
//! 3. Understanding get, put, and try_get operations
//!
//! The KeyValueStore is essential for distributed/multi-node PJRT setups
//! where processes need to coordinate through a shared key-value store.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example kv_store
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use pjrt::{self, KeyValueStore, Result};

/// A simple in-memory KeyValueStore implementation for demonstration
#[derive(Debug, Clone)]
struct InMemoryKeyValueStore {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemoryKeyValueStore {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl KeyValueStore for InMemoryKeyValueStore {
    fn get(&self, key: &str, timeout_in_ms: i32) -> Result<String> {
        println!("   KV Store: GET '{}' (timeout={}ms)", key, timeout_in_ms);

        let data = self.data.lock().unwrap();
        match data.get(key) {
            Some(value) => {
                println!("   KV Store: Found value for '{}'", key);
                Ok(value.clone())
            }
            None => {
                println!(
                    "   KV Store: Key '{}' not found (would wait in real impl)",
                    key
                );
                Err(pjrt::Error::InvalidArgument(format!(
                    "Key not found: {}",
                    key
                )))
            }
        }
    }

    fn put(&self, key: &str, value: &str) -> Result<()> {
        println!("   KV Store: PUT '{}' = '{}'", key, value);

        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn try_get(&self, key: &str) -> Result<Option<String>> {
        println!("   KV Store: TRY_GET '{}'", key);

        let data = self.data.lock().unwrap();
        match data.get(key) {
            Some(value) => {
                println!("   KV Store: Found value for '{}'", key);
                Ok(Some(value.clone()))
            }
            None => {
                println!("   KV Store: Key '{}' not found (non-blocking)", key);
                Ok(None)
            }
        }
    }
}

fn main() -> Result<()> {
    println!("Key-Value Store Example");
    println!("======================\n");

    // Demonstrate the KeyValueStore trait usage
    demonstrate_kv_store_trait()?;

    // Show how it would be used with Client (conceptual)
    demonstrate_client_integration()?;

    Ok(())
}

/// Demonstrates the KeyValueStore trait operations
fn demonstrate_kv_store_trait() -> Result<()> {
    println!("1. KeyValueStore Operations");
    println!("   ------------------------");

    let store = InMemoryKeyValueStore::new();

    // Demonstrate PUT operations
    println!("   Storing configuration data:");
    store.put("cluster/size", "4")?;
    store.put("cluster/node0/address", "192.168.1.10:8080")?;
    store.put("cluster/node1/address", "192.168.1.11:8080")?;
    store.put("cluster/node2/address", "192.168.1.12:8080")?;
    store.put("cluster/node3/address", "192.168.1.13:8080")?;

    println!();

    // Demonstrate GET operations
    println!("   Retrieving data with GET:");
    let size = store.get("cluster/size", 5000)?;
    println!("   Cluster size: {}", size);

    let node0 = store.get("cluster/node0/address", 5000)?;
    println!("   Node 0 address: {}", node0);

    println!();

    // Demonstrate TRY_GET operations
    println!("   Retrieving data with TRY_GET (non-blocking):");
    match store.try_get("cluster/node1/address")? {
        Some(value) => println!("   Node 1 address: {}", value),
        None => println!("   Node 1 address not found"),
    }

    match store.try_get("cluster/nonexistent")? {
        Some(value) => println!("   Nonexistent: {}", value),
        None => println!("   Nonexistent key returns None (as expected)"),
    }

    println!();

    Ok(())
}

/// Demonstrates how KeyValueStore integrates with Client builder
fn demonstrate_client_integration() -> Result<()> {
    println!("2. Client Integration");
    println!("   ------------------");

    println!("   In a distributed setup, the KeyValueStore is used during");
    println!("   client creation to coordinate across multiple processes.\n");

    println!("   Example usage pattern:");
    println!();
    println!("   let kv_store = MyKeyValueStore::new(redis_connection);");
    println!();
    println!("   let client = Client::builder(&api)");
    println!("       .key_value_store(&kv_store)  // Provide the KV store");
    println!("       .process_index(0)              // This process index");
    println!("       .process_count(4)              // Total processes");
    println!("       .build()?;");
    println!();

    println!("   Common KeyValueStore implementations:");
    println!("   - Redis: For distributed coordination");
    println!("   - etcd: For Kubernetes-native deployments");
    println!("   - Consul: For service discovery");
    println!("   - Custom: In-memory (testing), filesystem, etc.\n");

    println!("   Typical keys used by PJRT:");
    println!("   - Process metadata and heartbeats");
    println!("   - Device topology information");
    println!("   - Collective operation coordination");
    println!("   - Checkpoint and recovery state\n");

    Ok(())
}

/// Example of a more sophisticated KeyValueStore implementation
#[allow(dead_code)]
mod advanced_kv_store {
    use std::time::{Duration, Instant};

    use super::*;

    /// A KeyValueStore with timeouts and retry logic
    pub struct RetryableKeyValueStore {
        inner: InMemoryKeyValueStore,
        retry_count: u32,
        retry_delay_ms: u64,
    }

    impl RetryableKeyValueStore {
        pub fn new(inner: InMemoryKeyValueStore, retry_count: u32, retry_delay_ms: u64) -> Self {
            Self {
                inner,
                retry_count,
                retry_delay_ms,
            }
        }

        fn with_retry<F, T>(&self, operation: F) -> Result<T>
        where
            F: Fn() -> Result<T>,
        {
            let _start = Instant::now();
            let mut last_error = None;

            for attempt in 0..self.retry_count {
                match operation() {
                    Ok(result) => return Ok(result),
                    Err(err) => {
                        last_error = Some(err);
                        if attempt < self.retry_count - 1 {
                            std::thread::sleep(Duration::from_millis(self.retry_delay_ms));
                        }
                    }
                }
            }

            Err(last_error
                .unwrap_or_else(|| pjrt::Error::InvalidArgument("Retry failed".to_string())))
        }
    }

    impl KeyValueStore for RetryableKeyValueStore {
        fn get(&self, key: &str, timeout_in_ms: i32) -> Result<String> {
            // For GET, we respect the timeout but also add retries
            let effective_timeout = timeout_in_ms.max(1000); // Minimum 1 second
            self.with_retry(|| self.inner.get(key, effective_timeout))
        }

        fn put(&self, key: &str, value: &str) -> Result<()> {
            self.with_retry(|| self.inner.put(key, value))
        }

        fn try_get(&self, key: &str) -> Result<Option<String>> {
            self.with_retry(|| self.inner.try_get(key))
        }
    }
}
