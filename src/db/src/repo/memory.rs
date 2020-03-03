use super::{InvocationsRepo, Repo, RunsRepo, UsersRepo};
use crate::schema::*;
use anyhow::{bail, Context, Result};
use std::{convert::TryFrom, sync::Mutex};

#[derive(Debug, Default)]
struct Data {
    // None if run was deleted
    runs: Vec<Option<Run>>,
    invs: Vec<Invocation>,
    users: Vec<User>,
}

#[derive(Debug, Default)]
pub struct MemoryRepo {
    conn: Mutex<Data>,
}

impl MemoryRepo {
    pub fn new() -> Self {
        // TODO duplicates db/migrations/<initial>/up.sql
        let this: Self = Self::default();
        this.user_new(NewUser {
            username: "Global/Root".to_string(),
            password_hash: None,
            groups: vec![],
        })
        .unwrap();
        this.user_new(NewUser {
            username: "Global/Guest".to_string(),
            password_hash: None,
            groups: vec![],
        })
        .unwrap();
        this
    }
}

impl RunsRepo for MemoryRepo {
    fn run_new(&self, run_data: NewRun) -> Result<Run> {
        let mut data = self.conn.lock().unwrap();
        let run_id = data.runs.len() as RunId;
        let run = Run {
            id: run_id,
            toolchain_id: run_data.toolchain_id,
            problem_id: run_data.problem_id,
            rejudge_id: run_data.rejudge_id,
            user_id: run_data.user_id,
            contest_name: run_data.contest_name,
        };
        data.runs.push(Some(run.clone()));
        Ok(run)
    }

    fn run_try_load(&self, run_id: i32) -> Result<Option<Run>> {
        let data = self.conn.lock().unwrap();
        let idx = run_id as usize;
        Ok(data.runs.get(idx).cloned().unwrap_or(None))
    }

    fn run_update(&self, run_id: i32, patch: RunPatch) -> Result<()> {
        let mut data = self.conn.lock().unwrap();
        let idx = run_id as usize;
        let cur = match data.runs.get_mut(idx) {
            Some(Some(x)) => x,
            None | Some(None) => bail!("run_update@memory: unknown run id"),
        };
        if let Some(new_rejudge_id) = patch.rejudge_id {
            cur.rejudge_id = new_rejudge_id;
        }

        Ok(())
    }

    fn run_delete(&self, run_id: i32) -> Result<()> {
        let mut data = self.conn.lock().unwrap();
        let cur = match data.runs.get_mut(run_id as usize) {
            Some(x) => x,
            None => bail!("run_delete@memory: unknown run id"),
        };
        if cur.take().is_some() {
            Ok(())
        } else {
            bail!("run_delete@memory: run already deleted")
        }
    }

    fn run_select(&self, with_run_id: Option<RunId>, limit: Option<u32>) -> Result<Vec<Run>> {
        let lim = limit
            .map(|x| usize::try_from(x).unwrap())
            .unwrap_or(usize::max_value());
        if lim == 0 {
            return Ok(Vec::new());
        }
        match with_run_id {
            Some(r) => Ok(self
                .run_try_load(r)
                .into_iter()
                .filter_map(std::convert::identity)
                .collect()),
            None => {
                let data = self.conn.lock().unwrap();
                let cnt = std::cmp::min(lim, data.runs.len());
                Ok(data.runs[..cnt].iter().filter_map(|x| x.clone()).collect())
            }
        }
    }
}

impl InvocationsRepo for MemoryRepo {
    fn inv_new(&self, inv_data: NewInvocation) -> Result<Invocation> {
        let mut data = self.conn.lock().unwrap();
        let inv_id = data.invs.len() as InvocationId;
        let inv = Invocation {
            id: inv_id,
            run_id: inv_data.run_id,
            invoke_task: inv_data.invoke_task,
            state: inv_data.state,
            outcome: inv_data.outcome,
        };
        data.invs.push(inv.clone());
        Ok(inv)
    }

    fn inv_find_waiting(
        &self,
        offset: u32,
        count: u32,
        predicate: &mut dyn FnMut(Invocation) -> Result<bool>,
    ) -> Result<Vec<Invocation>> {
        let data = self.conn.lock().unwrap();
        let items = data.invs.iter().skip(offset as usize).take(count as usize);
        let mut filtered = Vec::new();
        for x in items {
            if predicate(x.clone())? {
                filtered.push(x.clone());
            }
        }
        Ok(filtered)
    }

    fn inv_last(&self, run_id: RunId) -> Result<Invocation> {
        let data = self.conn.lock().unwrap();
        data.invs
            .iter()
            .filter(|inv| inv.run_id == run_id)
            .last()
            .ok_or_else(|| anyhow::anyhow!("no invocations for run exist"))
            .map(Clone::clone)
    }

    fn inv_update(&self, inv_id: InvocationId, patch: InvocationPatch) -> Result<()> {
        let mut data = self.conn.lock().unwrap();
        if inv_id >= data.invs.len() as i32 || inv_id < 0 {
            bail!("inv_update: no such invocation");
        }
        let mut inv = &mut data.invs[inv_id as usize];
        let InvocationPatch { state: p_state } = patch;
        if let Some(p_state) = p_state {
            inv.state = p_state;
        }
        Ok(())
    }

    fn inv_add_outcome_header(
        &self,
        inv_id: InvocationId,
        header: invoker_api::InvokeOutcomeHeader,
    ) -> Result<()> {
        let mut data = self.conn.lock().unwrap();
        let inv = match data.invs.get_mut(inv_id as usize) {
            Some(inv) => inv,
            None => bail!("inv_add_outcome_header: no such invocation"),
        };
        let headers = match inv.outcome.as_array_mut() {
            Some(hs) => hs,
            None => bail!("inb_add_outcome_header: outcome is not array"),
        };
        headers.push(
            serde_json::to_value(&header).context("failed to serialize InvokeOutcomeHeader")?,
        );
        Ok(())
    }
}

impl UsersRepo for MemoryRepo {
    fn user_new(&self, user_data: NewUser) -> Result<User> {
        let mut data = self.conn.lock().unwrap();
        let user_id = data.users.len();
        let user_id = uuid::Uuid::from_fields(user_id as u32, 0, 0, &[0; 8]).unwrap();
        let user = User {
            id: user_id,
            username: user_data.username,
            password_hash: user_data.password_hash,
            groups: user_data.groups,
        };
        data.users.push(user.clone());
        Ok(user)
    }

    fn user_try_load_by_login(&self, login: &str) -> Result<Option<User>> {
        let data = self.conn.lock().unwrap();
        let res = data
            .users
            .iter()
            .find(|user| user.username == login)
            .cloned();
        Ok(res)
    }
}

impl Repo for MemoryRepo {}

#[cfg(test)]
mod tests {
    use super::*;

    mod runs {
        use super::*;

        #[test]
        fn test_basic() {
            let repo = MemoryRepo::new();

            let john_id = uuid::Uuid::new_v4();
            assert!(repo.run_load(228).is_err());
            assert!(repo.run_load(0).is_err());
            let new_run = NewRun {
                toolchain_id: "foo".to_string(),
                problem_id: "quux".to_string(),
                rejudge_id: 33,
                user_id: john_id,
                contest_id: "olymp".to_string(),
            };
            let inserted_run = repo.run_new(new_run).unwrap();
            assert_eq!(inserted_run.id, 0);
            let run_in_db = repo.run_load(0).unwrap();
            assert_eq!(inserted_run, run_in_db);
        }

        #[test]
        fn test_patch() {
            let repo = MemoryRepo::new();
            let new_run = NewRun {
                toolchain_id: "0".to_string(),
                problem_id: "0".to_string(),
                rejudge_id: 0,
                user_id: uuid::Uuid::new_v4(),
                contest_id: "cntst".to_string(),
            };
            repo.run_new(new_run).unwrap();
            let patch = RunPatch {
                rejudge_id: Some(4),
            };
            repo.run_update(0, patch).unwrap();
            let patched_run = repo.run_load(0).unwrap();
            // now let's check that all fields that must be updated are actually updated
            assert_eq!(patched_run.rejudge_id, 4);
        }
    }
}
