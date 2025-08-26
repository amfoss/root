use async_graphql::MergedObject;
use mutations::{AttendanceMutations, MemberMutations, StreakMutations};
use queries::{AttendanceQueries, MemberQueries};

pub mod mutations;
pub mod queries;

#[derive(MergedObject, Default)]
pub struct Query(MemberQueries, AttendanceQueries);

#[derive(MergedObject, Default)]
pub struct Mutation(MemberMutations, AttendanceMutations, StreakMutations);
