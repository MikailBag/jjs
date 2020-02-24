use super::{super::prelude::*, ContestId, ProblemId};

#[derive(GraphQLObject)]
pub(crate) struct Problem {
    /// Problem title as contestants see, e.g. "Find max flow".
    pub title: String,
    /// Problem external id (aka problem code) as contestants see. This is usually one letter or
    /// something similar, e.g. 'A' or '3F'.
    pub id: ProblemId,
}

pub(crate) struct Contest {
    pub title: String,
    pub id: ContestId,
}

#[juniper::object(Context = Context)]
impl Contest {
    /// E.g. "Berlandian Olympiad in Informatics. Finals. Day 3."
    fn title(&self) -> &str {
        &self.title
    }

    /// Configured by human, something readable like 'olymp-2019', or 'test-contest'
    fn id(&self) -> &str {
        &self.id
    }

    fn problems(&self, ctx: &Context) -> Vec<Problem> {
        let contest_cfg = ctx.cfg.contests.get(0).unwrap();
        contest_cfg
            .problems
            .iter()
            .map(|p| Problem {
                title: ctx
                    .cfg
                    .find_problem(&p.name)
                    .expect("problem not found")
                    .title
                    .clone(),
                id: p.code.clone(),
            })
            .collect()
    }
}
