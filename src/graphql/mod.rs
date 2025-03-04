use async_graphql::MergedObject;
use mutations::{AttendanceMutations, MemberMutations, ProjectMutations, StreakMutations};
use queries::{AttendanceQueries, MemberQueries, ProjectQueries, StreakQueries};

pub mod mutations;
pub mod queries;

#[derive(MergedObject, Default)]
pub struct Query(
    MemberQueries,
    AttendanceQueries,
    StreakQueries,
    ProjectQueries,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    MemberMutations,
    AttendanceMutations,
    StreakMutations,
    ProjectMutations,
);
