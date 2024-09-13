use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::{Mutex, OnceLock};

use libloading::Library;
use pjrt_sys::PJRT_Api;

use crate::{utils, Api, Error, Result};

struct PluginManager {
    plugins: Mutex<HashMap<String, Library>>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: Mutex::new(HashMap::new()),
        }
    }

    fn get_pjrt_api(&self, lib: &Library) -> Result<Api> {
        let get_api_func: libloading::Symbol<unsafe extern "C" fn() -> *const PJRT_Api> =
            unsafe { lib.get(b"GetPjrtApi")? };
        let ptr = unsafe { get_api_func() };
        Ok(Api::new(ptr))
    }

    fn load_plugin(&self, library: &str) -> Result<Api> {
        let mut libraries = self.plugins.lock().unwrap();
        if let Some(lib) = libraries.get(library) {
            self.get_pjrt_api(lib)
        } else {
            let lib = unsafe { Library::new(library)? };
            libraries.insert(library.to_string(), lib);
            self.get_pjrt_api(&libraries[library])
        }
    }
}

static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();

pub fn load_plugin(library: &str) -> Result<Api> {
    let manager = PLUGIN_MANAGER.get_or_init(PluginManager::new);
    manager.load_plugin(library)
}
