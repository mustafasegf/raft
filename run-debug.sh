#!/usr/bin/env bash
RED=$(printf '\[31m')
GREEN=$(printf '\[32m')
BLUE=$(printf '\[34m')
ENDCOLOR=$(printf '\033')
ESC=$(printf '\033')

trap 'kill %1; kill %2' SIGINT
cargo build &&
./target/debug/raft --id 1 --server 127.0.0.1:4001 --peers 2@127.0.0.1:4002 3@127.0.0.1:4003 | tee 1.log | sed -e "s/.*/${ESC}${RED}[Node1] &${ESC}${ENDCOLOR}/" &
./target/debug/raft --id 2 --server 127.0.0.1:4002 --peers 1@127.0.0.1:4001 3@127.0.0.1:4003 | tee 2.log | sed -e "s/.*/${ESC}${GREEN}[Node2] &${ESC}${ENDCOLOR}/" &
./target/debug/raft --id 3 --server 127.0.0.1:4003 --peers 1@127.0.0.1:4001 2@127.0.0.1:4002 | tee 3.log | sed -e "s/.*/${ESC}${BLUE}[Node2] &${ESC}${ENDCOLOR}/"

# trap 'kill %1; kill %2' SIGINT
# cargo build &&
# ./target/debug/raft --id 1 --server 127.0.0.1:4001 --peers 2@127.0.0.1:4002 | tee 1.log | sed -e "s/.*/${ESC}${RED}[Node1] &${ESC}${ENDCOLOR}/" &
# ./target/debug/raft --id 2 --server 127.0.0.1:4002 --peers 1@127.0.0.1:4001 | tee 2.log | sed -e "s/.*/${ESC}${GREEN}[Node2] &${ESC}${ENDCOLOR}/" 
#
