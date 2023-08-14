#!/bin/bash
cargo build
cargo run -- parse_c <(cproto -f 3 tests/basic/basic.c) > info.c
gcc info.c -o info
./info | tee function_schema.yml
cargo run -- generate_java function_schema.yml
