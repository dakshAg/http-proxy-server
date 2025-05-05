# Makefile for the Rust project

# Variables
CARGO := cargo
TARGET := htproxy

# Default target
all: build

# Build the project
build:
	$(CARGO) build --release
	cp target/release/$(TARGET) .

# Run the project
run: build
	./$(TARGET)

# Clean the project
clean:
	$(CARGO) clean
	rm -f $(TARGET)

# Format the code
format:
	$(CARGO) fmt

# Check for linting issues
lint:
	$(CARGO) clippy -- -D warnings

.PHONY: all build run clean format lint