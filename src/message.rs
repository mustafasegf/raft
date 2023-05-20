// Generated from prost-build

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    /// uint64 term = 1;
    #[prost(oneof = "data::MsgType", tags = "2, 3, 4")]
    pub msg_type: ::core::option::Option<data::MsgType>,
}
/// Nested message and enum types in `Data`.
pub mod data {
    /// uint64 term = 1;
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MsgType {
        #[prost(message, tag = "2")]
        Vote(super::VoteRequest),
        #[prost(message, tag = "3")]
        Append(super::AppendRequest),
        #[prost(message, tag = "4")]
        Response(super::Response),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteRequest {
    #[prost(uint64, tag = "1")]
    pub term: u64,
    #[prost(uint64, tag = "2")]
    pub candidate_id: u64,
    #[prost(uint64, tag = "3")]
    pub last_log_idx: u64,
    #[prost(uint64, tag = "4")]
    pub last_log_term: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendRequest {
    #[prost(uint64, tag = "1")]
    pub term: u64,
    #[prost(uint64, tag = "2")]
    pub leader_id: u64,
    #[prost(uint64, tag = "3")]
    pub prev_log_idx: u64,
    #[prost(uint64, tag = "4")]
    pub prev_log_term: u64,
    #[prost(uint64, tag = "5")]
    pub leader_commit: u64,
    #[prost(message, repeated, tag = "6")]
    pub entries: ::prost::alloc::vec::Vec<Entry>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entry {
    #[prost(uint64, tag = "1")]
    pub idx: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(uint64, tag = "1")]
    pub term: u64,
    #[prost(bool, tag = "2")]
    pub status: bool,
}

