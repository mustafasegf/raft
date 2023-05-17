#!/usr/bin/env bash
cargo build
./target/debug/raft --id 1 --server localhost:4001 --peers 2@localhost:4002 3@localhost:4003
./target/debug/raft --id 2 --server localhost:4002 --peers 1@localhost:4001 3@localhost:4003
./target/debug/raft --id 3 --server localhost:4003 --peers 1@localhost:4001 2@localhost:4002


