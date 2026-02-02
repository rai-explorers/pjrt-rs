//! Memory Layout Example
//!
//! This example demonstrates working with memory layouts:
//! 1. Getting default memory layouts for different data types
//! 2. Serializing and storing memory layouts
//! 3. Using layouts for buffer creation
//!
//! Memory layouts are important for performance optimization and
//! interoperability with external frameworks.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example memory_layout
//! ```

use pjrt::{self, Client, LayoutsExt, PrimitiveType, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;

    let client = Client::builder(&api).build()?;

    println!("Memory Layout Example");
    println!("====================");

    // Try to access the layouts extension
    match client.layouts_extension() {
        Some(layouts_ext) => {
            println!("Layouts extension is available!");
            demonstrate_memory_layouts(&client, &layouts_ext)?;
        }
        None => {
            println!("Layouts extension is not available in this plugin.");
            println!(
                "This example will demonstrate the API structure without using actual layouts."
            );
            demonstrate_layout_api_concepts(&client)?;
        }
    }

    println!("\nNote: Memory layouts are important for:");
    println!("- Optimizing data placement in memory");
    println!("- Interoperability with external frameworks");
    println!("- Performance tuning for specific hardware");

    Ok(())
}

/// Demonstrates memory layout functionality when extension is available
fn demonstrate_memory_layouts(client: &Client, layouts_ext: &pjrt::LayoutsExtension) -> Result<()> {
    println!("\nMemory Layout Functions:");

    // Get example dimensions to test layouts
    let dims = vec![1024, 1024];

    // Test different primitive types
    let types = vec![
        PrimitiveType::F32,
        PrimitiveType::F16,
        PrimitiveType::BF16,
        PrimitiveType::S32,
        PrimitiveType::S16,
    ];

    for ty in types {
        println!("  Getting layout for: {:?}", ty);

        // In a real implementation, you would get the layout:
        let layout = layouts_ext.client_default_layout(client, ty, &dims)?;
        println!("    Layout size: {} bytes", layout.size());

        // Serialize the layout for storage or transmission
        let serialized = layout.serialize()?;
        println!("    Serialized to {} bytes", serialized.bytes().len());
        println!("    First 8 bytes (hex): {:?}", &serialized.bytes()[..8]);

        // The serialized layout could be stored or transmitted to another process
    }

    // Example: Layout for a specific device
    if let Some(_device) = client.devices().first() {
        let layout = layouts_ext.client_default_layout(client, PrimitiveType::F32, &dims)?;
        println!("  Device-specific layout: {:?}", layout);
    }

    Ok(())
}

/// Demonstrates layout API concepts when extension is not available
fn demonstrate_layout_api_concepts(_client: &Client) -> Result<()> {
    println!("\nMemory Layout API Concepts:");

    // Show what would be available with the extension
    let dimensions = vec![512, 512, 3];
    println!("  Example dimensions: {:?}", dimensions);

    // For each type, show the memory layout concept
    println!("\n  Layout Concepts for Different Types:");

    let types_and_info = vec![
        (
            PrimitiveType::F32,
            "4 bytes per element, standard float format",
        ),
        (
            PrimitiveType::F16,
            "2 bytes per element, half-precision float",
        ),
        (PrimitiveType::BF16, "2 bytes per element, brain float"),
        (PrimitiveType::C64, "8 bytes per element, complex float"),
        (PrimitiveType::S8, "1 byte per element, signed integer"),
        (PrimitiveType::U8, "1 byte per element, unsigned integer"),
        (PrimitiveType::S16, "2 bytes per element, signed integer"),
        (PrimitiveType::U16, "2 bytes per element, unsigned integer"),
    ];

    let total_elements: i64 = dimensions.iter().product();

    for (ty, description) in types_and_info {
        println!("    {:?}: {}", ty, description);

        // Calculate theoretical memory requirements
        let element_size = match ty {
            PrimitiveType::F32 | PrimitiveType::C64 => 4,
            PrimitiveType::F16 | PrimitiveType::BF16 | PrimitiveType::S16 | PrimitiveType::U16 => 2,
            PrimitiveType::S8 | PrimitiveType::U8 => 1,
            _ => 4, // Default
        };

        let total_bytes = total_elements * element_size;
        println!("      Total for {:?}: {} bytes", ty, total_bytes);
    }

    // Demonstrate serialized layout concept
    println!("\n  Serialized Layout Concept:");
    println!("    Serialized layouts allow:");
    println!("    - Storing memory layout information");
    println!("    - Transferring layout between processes");
    println!("    - Caching layout for reuse");

    // Show placeholder for what a serialized layout would contain
    println!("\n  Example Serialized Layout Structure:");
    println!("    - Layout type identifier");
    println!("    - Dimension sizes");
    println!("    - Element type information");
    println!("    - Alignment/padding requirements");
    println!("    - Platform-specific memory attributes");

    // Show how layouts might affect buffer creation
    println!("\n  Using Layouts for Buffer Creation:");
    println!("    With a known layout, you can:");
    println!("    - Create buffers with specific memory placement");
    println!("    - Optimize for specific hardware characteristics");
    println!("    - Ensure compatibility with external frameworks");

    // Example of creating a buffer with hypothetical layout
    println!("\n  Buffer Creation with Layout:");
    let dims = vec![100, 100];
    let element_size = 4; // F32
    let total_size = dims.iter().product::<i64>() * element_size;
    println!("    Buffer dimensions: {:?}", dims);
    println!("    Element size: {} bytes", element_size);
    println!("    Total size: {} bytes", total_size);

    Ok(())
}

/// Example of a memory layout cache for frequent reuse
#[allow(dead_code)]
mod layout_cache {
    use std::collections::HashMap;

    use pjrt::{PrimitiveType, Result};

    /// A simple cache for memory layouts
    /// In a real application, this would store actual layout objects
    struct LayoutCache {
        cache: HashMap<(Vec<i64>, PrimitiveType), CachedLayout>,
    }

    struct CachedLayout {
        // In a real implementation, this would store:
        // - The actual LayoutsMemoryLayout
        // - Serialization of the layout
        // - Creation timestamp
        // - Usage count
        serialized: Vec<u8>,
        size_bytes: usize,
    }

    impl LayoutCache {
        /// Create a new empty cache
        pub fn new() -> Self {
            Self {
                cache: HashMap::new(),
            }
        }

        /// Get a layout from cache or compute it
        pub fn get_or_compute<F>(
            &mut self,
            dims: &[i64],
            ty: PrimitiveType,
            compute_fn: F,
        ) -> Result<&CachedLayout>
        where
            F: FnOnce(&[i64], PrimitiveType) -> Result<CachedLayout>,
        {
            let key = (dims.to_vec(), ty);

            if !self.cache.contains_key(&key) {
                let layout = compute_fn(dims, ty)?;
                self.cache.insert(key.clone(), layout);
            }

            Ok(self.cache.get(&key).unwrap())
        }
    }

    /// Example of using the layout cache
    pub fn demonstrate_layout_cache() -> Result<()> {
        let mut cache = LayoutCache::new();

        // Simulate computing a layout for F32
        let compute_layout = |dims: &[i64], ty: PrimitiveType| -> Result<CachedLayout> {
            // In a real implementation, you would call layouts_ext.client_default_layout()
            println!("Computing layout for {:?} with shape {:?}", ty, dims);

            // Simulate serialization
            let serialized = format!("layout_{:?}_{:?}", ty, dims).into_bytes();
            let size_bytes = serialized.len();

            Ok(CachedLayout {
                serialized,
                size_bytes,
            })
        };

        // First call computes the layout
        let dims = vec![256, 256];
        let layout = cache.get_or_compute(&dims, PrimitiveType::F32, compute_layout)?;
        println!("Cached layout size: {} bytes", layout.size_bytes);

        // Second call uses the cached layout
        let layout = cache.get_or_compute(&dims, PrimitiveType::F32, compute_layout)?;
        println!("Using cached layout size: {} bytes", layout.size_bytes);

        Ok(())
    }
}
