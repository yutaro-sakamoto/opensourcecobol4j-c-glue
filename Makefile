SHELL=/bin/bash

# local settings
# export OC4J_C_GLUE_JNI_INCLUDE="-I/usr/lib/jvm/java-11-openjdk-amd64/include/ -I/usr/lib/jvm/java-11-openjdk-amd64/include/linux/"

COBJ_C_GLUE = target/release/cobj-c-glue
COBJ_C_GLUE_SRC = $(wildcard src/*.rs)

all: $(COBJ_C_GLUE)
	$(COBJ_C_GLUE) parse_c <(cproto -f 3 tests/basic/basic.c) > info.c
	gcc -c tests/basic/basic.c -o basic.o
	gcc info.c -o info
	./info | tee function_schema.yml
	$(COBJ_C_GLUE) generate_java function_schema.yml
	javac -h . *.java
	$(COBJ_C_GLUE) generate_c function_schema.yml
	gcc $${OC4J_C_GLUE_JNI_INCLUDE} -shared -fPIC -o libinit.so init.c basic.o
	gcc $${OC4J_C_GLUE_JNI_INCLUDE} -shared -fPIC -o libdestroy.so destroy.c basic.o
	javac *.java
	cobj prog.cbl
	java -Djava.library.path=. prog

$(COBJ_C_GLUE): $(COBJ_C_GLUE_SRC)
	cargo build --release