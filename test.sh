#!/bin/bash
cargo build
cargo run -- parse_c <(cproto -f 3 tests/basic/basic.c) > info.c
gcc -c tests/basic/basic.c -o basic.o
gcc info.c -o info
./info | tee function_schema.yml
cargo run -- generate_java function_schema.yml
javac -h . *.java
cargo run -- generate_c function_schema.yml
# local settings
# export OC4J_C_GLUE_JNI_INCLUDE="-I/usr/lib/jvm/java-11-openjdk-amd64/include/ -I/usr/lib/jvm/java-11-openjdk-amd64/include/linux/"
gcc ${OC4J_C_GLUE_JNI_INCLUDE} -shared -fPIC -o libinit.so init.c basic.o
gcc ${OC4J_C_GLUE_JNI_INCLUDE} -shared -fPIC -o libdestroy.so destroy.c basic.o
javac *.java
cobj prog.cbl
java -Djava.library.path=. prog
