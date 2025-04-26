use crate::backend::Backend;
use crate::backend::backend_type::BackendType;
use crate::cli::args::BackendArg;
use crate::cmd::CmdLineRunner;
use crate::config::Config;
use crate::http::HTTP_FETCH;
use crate::install_context::InstallContext;
use crate::toolset::ToolVersion;
use itertools::Itertools;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;
use versions::Versioning;

#[derive(Debug)]
pub struct CabalBackend {
    ba: BackendArg,
}

impl CabalBackend {
    pub fn from_arg(ba: BackendArg) -> Self {
        Self { ba }
    }
}

impl Backend for CabalBackend {
    fn get_type(&self) -> BackendType {
        BackendType::Cabal
    }

    fn ba(&self) -> &BackendArg {
        &self.ba
    }

    fn get_dependencies(&self) -> eyre::Result<Vec<&str>> {
        Ok(vec!["cabal"])
    }

    fn _list_remote_versions(&self) -> eyre::Result<Vec<String>> {
        let url = format!("https://hackage.haskell.org/package/{}", self.tool_name());
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse()?);
        let data: HashMap<String, Value> = HTTP_FETCH.json_with_headers(url, &headers)?;
        let versions = data
            .keys()
            .cloned()
            // hackage versioning: https://pvp.haskell.org/
            .sorted_by_cached_key(|s| Versioning::new(s));
        Ok(versions.collect())
    }

    fn install_version_(&self, ctx: &InstallContext, tv: ToolVersion) -> eyre::Result<ToolVersion> {
        let config = Config::try_get()?;
        CmdLineRunner::new("cabal")
            .arg("install")
            .arg(format!("{}-{}", self.tool_name(), tv.version))
            .arg("--installdir")
            .arg(tv.install_path().join("bin"))
            .with_pr(&ctx.pr)
            .envs(ctx.ts.env_with_path(&config)?)
            .prepend_path(ctx.ts.list_paths())?
            .prepend_path(self.dependency_toolset()?.list_paths())?
            .execute()?;
        Ok(tv)
    }
}
