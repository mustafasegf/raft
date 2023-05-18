#!/usr/bin/env bash
RED=$(printf '\[31m')
GREEN=$(printf '\[32m')
BLUE=$(printf '\[34m')
ENDCOLOR=$(printf '\033')
ESC=$(printf '\033')

trap 'kill %1; kill %2' SIGINT
cargo build --release &&
./target/release/raft --id 1 --server localhost:4001 --peers 2@localhost:4002 3@localhost:4003 | tee 1.log | sed -e "s/.*/${ESC}${RED}[Node1] &${ESC}${ENDCOLOR}/" &
./target/release/raft --id 2 --server localhost:4002 --peers 1@localhost:4001 3@localhost:4003 | tee 2.log | sed -e "s/.*/${ESC}${GREEN}[Node2] &${ESC}${ENDCOLOR}/" &
./target/release/raft --id 3 --server localhost:4003 --peers 1@localhost:4001 2@localhost:4002 | tee 3.log | sed -e "s/.*/${ESC}${BLUE}[Node2] &${ESC}${ENDCOLOR}/"

