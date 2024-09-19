use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use bon::builder;
use libloading::Library;
use pjrt_sys::PJRT_Api;

use crate::{Api, Error, Result};

type GetPjrtApi = unsafe extern "C" fn() -> *const PJRT_Api;

struct PluginManager {
    plugins: Mutex<HashMap<String, (Library, Api)>>,
    aliases: Mutex<HashMap<String, Api>>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: Mutex::new(HashMap::new()),
            aliases: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_plugin(&self, library: String, alias: Option<String>) -> Result<Api> {
        let mut libraries = self
            .plugins
            .lock()
            .map_err(|err| Error::PoisonError(err.to_string()))?;
        if let Some((_, api)) = libraries.get(library.as_str()) {
            return Ok(api.clone());
        }
        let lib = unsafe { Library::new(library.as_str())? };
        let get_api_func: libloading::Symbol<GetPjrtApi> = unsafe { lib.get(b"GetPjrtApi")? };
        let ptr = unsafe { get_api_func() };
        let api = Api::wrap(ptr);
        libraries.insert(library, (lib, api.clone()));
        if let Some(alias) = alias {
            let mut aliases = self
                .aliases
                .lock()
                .map_err(|err| Error::PoisonError(err.to_string()))?;
            aliases.insert(alias, api.clone());
        }
        Ok(api)
    }

    pub fn get_plugin(&self, alias: &str) -> Option<Api> {
        let aliases = self
            .aliases
            .lock()
            .map_err(|err| Error::PoisonError(err.to_string()))
            .ok()?;
        aliases.get(alias).cloned()
    }
}

static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();

#[builder(finish_fn = "load")]
pub fn plugin(
    #[builder(start_fn, into)] library: String,
    #[builder(into)] alias: Option<String>,
) -> Result<Api> {
    let manager = PLUGIN_MANAGER.get_or_init(PluginManager::new);
    manager.load_plugin(library, alias)
}

#[allow(dead_code)]
pub fn get_plugin(alias: &str) -> Result<Api> {
    let manager = PLUGIN_MANAGER.get_or_init(PluginManager::new);
    manager
        .get_plugin(alias)
        .ok_or_else(|| Error::PluginNotFound(alias.to_string()))
}
