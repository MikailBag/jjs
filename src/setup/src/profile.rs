use serde::Deserialize;
use std::path::PathBuf;
/// Profile contains all settings and other data, representing desired state
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    pub(crate) data_dir: Option<PathBuf>,
    pub(crate) install_dir: PathBuf,
    pub(crate) pg: Option<PgProfile>,
    pub(crate) toolchains: Option<TcsProfile>,
    pub(crate) problems: Option<ProblemsProfile>,
    #[serde(default = "default_configs")]
    pub(crate) configs: bool,
    pub(crate) pki: Option<CertsProfile>,
}

fn default_configs() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct PgProfile {
    pub(crate) conn_string: String,
    pub(crate) db_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct TcsProfile {
    /// All toolchains from this list will be skipped
    #[serde(default)]
    pub(crate) blacklist: Vec<String>,
    /// If non-empty, all toolchains not from this list will be skipped
    #[serde(default)]
    pub(crate) whitelist: Vec<String>,
    /// Strategies used by `jjs-configure-toolchains`. If empty, default list will be used.
    #[serde(default)]
    pub(crate) strategies: Vec<String>,
    /// Will be appended to `jjs-configure-toolchains` argv.
    #[serde(default)]
    pub(crate) additional_args: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct ProblemsProfile {
    #[serde(default)]
    pub(crate) compile: ProblemsCompileProfile,
    #[serde(default)]
    pub(crate) archive: ProblemsArchiveProfile,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ProblemsArchiveProfile {
    pub(crate) sources: Vec<Source>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ProblemsCompileProfile {
    pub(crate) sources: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum Source {
    Path { path: std::path::PathBuf },
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CertsProfile {
    #[serde(default)]
    pub(crate) create_ca: bool,
}
