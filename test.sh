#!/bin/bash
cargo build
cargo run <(cproto -f 2 tests/basic/basic.c) > info.c
gcc info.c -o info
./info
