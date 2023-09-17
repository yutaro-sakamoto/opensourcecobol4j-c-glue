SHELL=/bin/bash

# local settings
# export OC4J_C_GLUE_JNI_INCLUDE="-I/usr/lib/jvm/java-11-openjdk-amd64/include/ -I/usr/lib/jvm/java-11-openjdk-amd64/include/linux/"

COBJ_C_GLUE = target/release/cobj-c-glue
COBJ_C_GLUE_SRC = $(wildcard src/*.rs)
CC = gcc
C_FLAGS_JNI_MODULE = -shared -fPIC
JAVAC = javac
JAVA = java
COBJ = cobj
C_INFO_OUTUT_SOURCE = info.c
C_INFO_OUTUT_BIN = ./info
TEST_C_SOURCE = tests/basic/basic.c
TEST_C_BIN = basic.o
FUNCTIONS_SCHEMA = function_schema.yml

all: $(COBJ_C_GLUE) $(C_INFO_OUTUT_BIN) $(FUNCTIONS_SCHEMA)
	$(CC) -c $(TEST_C_SOURCE) -o $(TEST_C_BIN)
	$(COBJ_C_GLUE) generate_java $(FUNCTIONS_SCHEMA)
	$(JAVAC) -h . *.java
	$(COBJ_C_GLUE) generate_c function_schema.yml
	$(CC) $${OC4J_C_GLUE_JNI_INCLUDE} -shared -fPIC -o libinit.so init.c basic.o
	$(CC) $${OC4J_C_GLUE_JNI_INCLUDE} -shared -fPIC -o libdestroy.so destroy.c basic.o
	$(JAVAC) *.java
	$(COBJ) prog.cbl
	$(JAVA) -Djava.library.path=. prog

$(COBJ_C_GLUE): $(COBJ_C_GLUE_SRC)
	cargo build --release

$(C_INFO_OUTUT_BIN): $(C_INFO_OUTUT_SOURCE)
	$(CC) info.c -o $(C_INFO_OUTUT_BIN)

$(C_INFO_OUTUT_SOURCE): $(TEST_C_SOURCE)
	$(COBJ_C_GLUE) parse_c <(cproto -f 3 $(TEST_C_SOURCE)) > ${C_INFO_OUTUT_SOURCE}

$(FUNCTIONS_SCHEMA): $(C_INFO_OUTUT_BIN)
	$(C_INFO_OUTUT_BIN) > $(FUNCTIONS_SCHEMA)