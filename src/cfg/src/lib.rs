extern crate serde;
extern crate toml;
#[macro_use]
extern crate serde_derive;

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::exit,
};

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Command {
    #[serde(default = "Command::default_env")]
    pub env: HashMap<String, String>,
    pub argv: Vec<String>,
    #[serde(default = "Command::default_cwd")]
    pub cwd: String,
}

impl Command {
    fn default_env() -> HashMap<String, String> {
        HashMap::new()
    }

    fn default_cwd() -> String {
        String::from("/jjs")
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Toolchain {
    /// Human-readable
    pub title: String,

    /// Machine-readable
    #[serde(skip, default)]
    pub name: String,

    pub filename: String,

    #[serde(rename = "build")]
    pub build_commands: Vec<Command>,

    #[serde(rename = "run")]
    pub run_command: Command,

    #[serde(rename = "build-limits", default)]
    pub limits: pom::Limits,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProblemBinding {
    /// Problem unique ID
    pub name: String,

    /// Problem ID in contest
    pub code: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Contest {
    pub title: String,

    /// Information about problems, not related to judging
    /// process (which is controlled by problem itself)
    pub problems: Vec<ProblemBinding>,

    /// List of groups of judges
    /// Judges will have full capabilities in this contest
    pub judges: Vec<String>,

    /// Which group members are considered registered for contest
    pub group: Vec<String>,

    /// Whether contest is visible for users that are not included in contestants
    #[serde(rename = "vis-unreg")]
    pub unregistered_visible: bool,

    /// Whether contest is visible for anonymous users
    #[serde(rename = "vis-anon")]
    pub anon_visible: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(skip)]
    pub toolchains: Vec<Toolchain>,

    #[serde(skip)]
    pub sysroot: PathBuf,

    #[serde(skip)]
    pub install_dir: PathBuf,

    #[serde(rename = "toolchain-root")]
    pub toolchain_root: String,

    #[serde(rename = "global-env", default)]
    pub global_env: HashMap<String, String>,

    #[serde(rename = "env-passing")]
    pub env_passing: bool,

    #[serde(rename = "env-blacklist", default)]
    pub env_blacklist: Vec<String>,

    #[serde(skip)]
    pub contests: Vec<Contest>,

    #[serde(skip)]
    pub problems: HashMap<String, pom::Problem>,
}

impl Config {
    pub fn postprocess(&mut self) {
        fn command_inherit_env(cmd: &mut Command, dfl: &HashMap<String, String>) {
            for (key, val) in dfl.iter() {
                cmd.env.entry(key.clone()).or_insert_with(|| val.clone());
            }
        }

        let mut inherit_env = self.global_env.clone();
        if self.env_passing {
            for (key, value) in std::env::vars() {
                if self.env_blacklist.contains(&key) {
                    continue;
                }
                inherit_env.entry(key).or_insert(value);
            }
        }

        for toolchain in &mut self.toolchains {
            for mut cmd in &mut toolchain.build_commands {
                command_inherit_env(&mut cmd, &inherit_env);
            }
            command_inherit_env(&mut toolchain.run_command, &inherit_env);
        }
    }

    pub fn find_toolchain(&self, name: &str) -> Option<&Toolchain> {
        for t in &self.toolchains {
            if name == t.name {
                return Some(t);
            }
        }
        None
    }

    pub fn find_problem(&self, name: &str) -> Option<&pom::Problem> {
        self.problems.get(name)
    }

    pub fn find_contest(&self, name: &str) -> Option<&Contest> {
        match name {
            "TODO" => Some(&self.contests[0]),
            _ => None,
        }
    }
}

pub fn parse_file(path: PathBuf) -> Config {
    let file_content = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Couldn't read main config file {:?} due to error {:?}",
            &path, e
        )
    });
    let raw_data: toml::Value = file_content.parse().unwrap();
    let cfg: Config = match toml::from_str(&file_content) {
        Ok(x) => x,
        Err(e) => panic!(
            "Error ocured when parsing config: {:?}.\nRaw config:\n{:#?}",
            e, raw_data
        ),
    };
    cfg
}

pub fn get_config() -> Config {
    let sysroot = env::var_os("JJS_SYSROOT").expect("Sysroot must be provided in JJS_SYSROOT");
    let sysroot = PathBuf::from(sysroot);
    let jjs_install_dir =
        env::var_os("JJS_PATH").expect("JJS installation dir must be provided in JJS_PATH");
    let jjs_install_dir = PathBuf::from(jjs_install_dir);

    let mut c = parse_file(sysroot.join("etc/jjs.toml"));
    // load toolchains
    for item in fs::read_dir(sysroot.join("etc/toolchains"))
        .expect("couldn't find toolchains config dir (JJS_SYSROOT/etc/jjs/toolchains")
    {
        let item = item.unwrap().path();
        let tc_cfg = fs::read_to_string(&item).expect("Couldn't read toolchain config file");
        let raw_toolchain_spec_data: toml::Value = tc_cfg.parse().unwrap();
        let mut toolchain_spec: Toolchain = match toml::from_str(&tc_cfg) {
            Ok(x) => x,
            Err(e) => panic!(
                "Following error when parsing toolchain config: {:?}.\nRaw config:\n{:#?}",
                e, raw_toolchain_spec_data
            ),
        };
        let toolchain_name = item.file_name().unwrap();

        let toolchain_name = Path::new(toolchain_name);
        let toolchain_name = toolchain_name
            .file_stem()
            .expect("toolchain config file name must start with toolchain name")
            .to_str()
            .expect("Toolchain name is not string")
            .to_string();
        toolchain_spec.name = toolchain_name;
        c.toolchains.push(toolchain_spec);
    }
    // load contests
    // TODO: support multiple contests
    {
        let contest_cfg_path = sysroot.join("etc/contest.toml");
        let contest_cfg = fs::read_to_string(contest_cfg_path).expect("failed read contest config");
        let mut contest: Contest = toml::from_str(&contest_cfg).expect("failed parse contest");
        for problem in contest.problems.iter_mut() {
            let problem_manifest_path = sysroot
                .join("var/problems")
                .join(&problem.name)
                .join("manifest.json");

            let problem_manifest = match fs::read(&problem_manifest_path) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!(
                        "Error: couldn't read manifest {} for problem {}: {}",
                        problem_manifest_path.display(),
                        &problem.name,
                        err
                    );
                    exit(1);
                }
            };

            let problem_manifest: pom::Problem = serde_json::from_slice(&problem_manifest).unwrap();
            c.problems.insert(problem.name.clone(), problem_manifest);
        }
        c.contests.push(contest);
    }
    c.sysroot = sysroot;
    c.install_dir = jjs_install_dir;
    c.postprocess();
    c
}
