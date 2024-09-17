use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use libloading::Library;
use pjrt_sys::PJRT_Api;

use crate::{Api, Error, Result};

type GetPjrtApi = unsafe extern "C" fn() -> *const PJRT_Api;

struct PluginManager {
    plugins: Mutex<HashMap<String, (Library, Api)>>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: Mutex::new(HashMap::new()),
        }
    }

    fn load_plugin(&self, library: &str) -> Result<Api> {
        let mut libraries = self
            .plugins
            .lock()
            .map_err(|err| Error::PoisonError(err.to_string()))?;
        if let Some((_, api)) = libraries.get(library) {
            return Ok(api.clone());
        }
        let lib = unsafe { Library::new(library)? };
        let get_api_func: libloading::Symbol<GetPjrtApi> = unsafe { lib.get(b"GetPjrtApi")? };
        let ptr = unsafe { get_api_func() };
        let api = Api::wrap(ptr);
        libraries.insert(library.to_string(), (lib, api.clone()));
        Ok(api)
    }
}

static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();

pub fn load_plugin(library: &str) -> Result<Api> {
    let manager = PLUGIN_MANAGER.get_or_init(PluginManager::new);
    manager.load_plugin(library)
}
