# Makefile for the Rust project without Cargo

# Variables
RUSTC := rustc
TARGET := htproxy
SRC := src/main.rs

# Default target
all: build

# Build the project
build:
	$(RUSTC) $(SRC) -o $(TARGET)

# Run the project
run: build
	./$(TARGET)

# Clean the project
clean:
	rm -f $(TARGET)

.PHONY: all build run clean