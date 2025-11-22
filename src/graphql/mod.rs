use async_graphql::MergedObject;
use mutations::{AttendanceMutations, AuthMutations, MemberMutations, StatusMutations};
use queries::MemberQueries;

pub mod mutations;
pub mod queries;

#[derive(MergedObject, Default)]
pub struct Query(MemberQueries);

#[derive(MergedObject, Default)]
pub struct Mutation(
    MemberMutations,
    AttendanceMutations,
    StatusMutations,
    AuthMutations,
);
