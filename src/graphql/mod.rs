use async_graphql::MergedObject;
use mutations::{
    AttendanceMutations, FetchCodeForces, FetchLeetCode, MemberMutations, ProjectMutations,
    StreakMutations,
};
use queries::{
    AttendanceQueries, LeaderboardQueries, MemberQueries, ProjectQueries, StreakQueries,
};

pub mod api;
pub mod mutations;
pub mod queries;

#[derive(MergedObject, Default)]
pub struct Query(
    MemberQueries,
    AttendanceQueries,
    StreakQueries,
    ProjectQueries,
    LeaderboardQueries,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    MemberMutations,
    AttendanceMutations,
    StreakMutations,
    ProjectMutations,
    FetchLeetCode,
    FetchCodeForces,
);
