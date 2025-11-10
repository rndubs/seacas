# Makefile for SEACAS Rust Docker environment
# Provides convenient shortcuts for common Docker operations

.PHONY: help build run shell test test-rust test-python clean prune

# Default target - show help
help:
	@echo "SEACAS Rust Docker Environment"
	@echo ""
	@echo "Available targets:"
	@echo "  make build         - Build the Docker image"
	@echo "  make run           - Run container with interactive shell"
	@echo "  make shell         - Alias for 'make run'"
	@echo "  make test          - Run all tests (Rust + Python)"
	@echo "  make test-rust     - Run Rust tests only"
	@echo "  make test-python   - Run Python tests only"
	@echo "  make ci            - Run CI/CD pipeline"
	@echo "  make clean         - Remove container and volumes"
	@echo "  make prune         - Deep clean (removes images too)"
	@echo ""
	@echo "Docker Compose targets:"
	@echo "  make up            - Start services with docker-compose"
	@echo "  make down          - Stop services"
	@echo "  make logs          - View container logs"
	@echo ""
	@echo "Build targets:"
	@echo "  make build-rust    - Build exodus-rs in container"
	@echo "  make build-python  - Build Python bindings in container"
	@echo "  make build-all     - Build everything in container"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs          - Generate Rust documentation"
	@echo "  make readme        - Show README.md"

# Build the Docker image
build:
	@echo "Building Docker image..."
	docker build -t seacas-rust:latest .

# Build without cache
build-clean:
	@echo "Building Docker image (no cache)..."
	docker build --no-cache -t seacas-rust:latest .

# Run interactive shell in container
run: build
	@echo "Starting interactive container..."
	docker run -it --rm \
		-v $$(cd ../.. && pwd):/workspace \
		seacas-rust:latest bash

# Alias for run
shell: run

# Run all tests
test: build
	@echo "Running all tests..."
	docker run --rm \
		-v $$(cd ../.. && pwd):/workspace \
		seacas-rust:latest \
		bash -c "cd /workspace/rust/exodus-rs && cargo test --features netcdf4 -- --test-threads=1"

# Run Rust tests only
test-rust: build
	@echo "Running Rust tests..."
	docker run --rm \
		-v $$(cd ../.. && pwd):/workspace \
		seacas-rust:latest \
		bash -c "cd /workspace/rust/exodus-rs && cargo test --features netcdf4 -- --test-threads=1"

# Run Python tests
test-python: build
	@echo "Building and testing Python bindings..."
	docker run --rm \
		-v $$(cd ../.. && pwd):/workspace \
		seacas-rust:latest \
		bash -c "cd /workspace/rust/exodus-py && maturin develop --features numpy && pytest tests/"

# Build exodus-rs
build-rust: build
	@echo "Building exodus-rs..."
	docker run --rm \
		-v $$(cd ../.. && pwd):/workspace \
		seacas-rust:latest \
		bash -c "cd /workspace/rust/exodus-rs && cargo build --release --features netcdf4"

# Build Python bindings
build-python: build
	@echo "Building Python bindings..."
	docker run --rm \
		-v $$(cd ../.. && pwd):/workspace \
		seacas-rust:latest \
		bash -c "cd /workspace/rust/exodus-py && maturin build --release --features numpy"

# Build everything
build-all: build-rust build-python
	@echo "All builds complete!"

# Run CI/CD pipeline
ci: build
	@echo "Running CI/CD pipeline..."
	docker-compose run --rm seacas-rust-ci

# Generate documentation
docs: build
	@echo "Generating documentation..."
	docker run --rm \
		-v $$(cd ../.. && pwd):/workspace \
		seacas-rust:latest \
		bash -c "cd /workspace/rust/exodus-rs && cargo doc --features netcdf4 --no-deps"
	@echo "Documentation available at ../../rust/exodus-rs/target/doc/exodus_rs/index.html"

# Show README
readme:
	@cat README.md

# Docker Compose shortcuts
up:
	@echo "Starting services..."
	docker-compose up -d

down:
	@echo "Stopping services..."
	docker-compose down

logs:
	@echo "Showing logs..."
	docker-compose logs -f

# Clean up containers and volumes
clean:
	@echo "Cleaning up containers and volumes..."
	docker-compose down -v
	docker container prune -f

# Deep clean (includes images)
prune: clean
	@echo "Removing Docker images..."
	docker image rm seacas-rust:latest 2>/dev/null || true
	docker image prune -f

# Show container information
info:
	@echo "Docker images:"
	@docker images | grep seacas-rust || echo "  (no images found)"
	@echo ""
	@echo "Running containers:"
	@docker ps | grep seacas-rust || echo "  (no containers running)"
	@echo ""
	@echo "Docker volumes:"
	@docker volume ls | grep rust- || echo "  (no volumes found)"

# Quick check - verify environment
check: build
	@echo "Checking environment..."
	docker run --rm seacas-rust:latest bash -c " \
		echo 'Rust version:' && rustc --version && \
		echo 'Cargo version:' && cargo --version && \
		echo 'Python version:' && python3 --version && \
		echo 'Maturin version:' && maturin --version && \
		echo 'HDF5 version:' && pkg-config --modversion hdf5 && \
		echo 'NetCDF version:' && pkg-config --modversion netcdf && \
		echo '' && \
		echo 'Environment check passed!'"
