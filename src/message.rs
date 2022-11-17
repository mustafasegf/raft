// Generated from prost-build

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(uint64, tag = "1")]
    pub term: u64,
    #[prost(oneof = "request::Requests", tags = "2, 3")]
    pub requests: ::core::option::Option<request::Requests>,
}

/// Nested message and enum types in `Request`.
pub mod request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Requests {
        #[prost(message, tag = "2")]
        Vote(super::VoteRequest),
        #[prost(message, tag = "3")]
        Append(super::AppendRequest),
    }
}

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

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entry {
    #[prost(uint64, tag = "1")]
    pub idx: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(uint64, tag = "1")]
    pub term: u64,
    #[prost(bool, tag = "2")]
    pub status: bool,
}
