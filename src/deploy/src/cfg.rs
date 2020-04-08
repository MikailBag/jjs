use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum BuildProfile {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "release-dbg")]
    RelWithDebInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub cargo: String,
    pub cmake: String,
    pub docker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub profile: BuildProfile,
    pub target: Option<String>,
    pub tool_info: ToolInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentsConfig {
    pub man: bool,
    pub api_doc: bool,
    pub testlib: bool,
    pub archive: bool,
    pub tools: bool,
    pub core: bool,
    pub extras: bool,
    pub json_schema: bool,
    pub example_problems: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdConfig {
    pub install_to_lib_systemd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagingConfig {
    /// Contains additional options for deb/build.sh
    pub deb: Option<Vec<String>>,
    pub systemd: bool,
    pub docker: Option<DockerConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub build_options: Vec<String>,
    pub tag: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub artifacts_dir: Option<PathBuf>,
    pub install_prefix: Option<PathBuf>,
    pub verbose: bool,
    pub packaging: PackagingConfig,
    pub build: BuildConfig,
    pub components: ComponentsConfig,
}
