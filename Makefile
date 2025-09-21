LIB_NAME = anLocales
DIST = dist
RUST_TARGET = release

UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Linux)
    LIB_EXT = so
    MKDIR = mkdir -p
    CP = cp
endif
ifeq ($(UNAME_S),Darwin)
    LIB_EXT = dylib
    MKDIR = mkdir -p
    CP = cp
endif
ifeq ($(OS),Windows_NT)
    LIB_EXT = dll
    MKDIR = mkdir -p
    CP = copy
endif

TARGET_PATH = target/$(RUST_TARGET)

ifeq ($(OS),Windows_NT)
    LIB_FILE = $(LIB_NAME).dll
else
    LIB_FILE = lib$(LIB_NAME).$(LIB_EXT)
endif

all: build copy

build:
	cargo build --release

copy: build
	$(MKDIR) $(DIST)
	$(CP) $(TARGET_PATH)/$(LIB_FILE) $(DIST)/
	$(CP) header/anlocales.h $(DIST)/

clean:
	cargo clean
	rm -rf $(DIST)

.PHONY: all build copy clean
