# MPI Integration for Rust Exodus Library

**Last Updated:** 2025-11-14
**Status:** Research Complete - Implementation Planning Phase

## Executive Summary

This document analyzes MPI integration options for the Rust exodus-rs library and provides recommendations for enabling parallel I/O capabilities on modern HPC platforms. Based on comprehensive research of the Rust HPC ecosystem as of 2024-2025, we present a phased approach that balances production readiness, safety guarantees, and HPC platform compatibility.

**Key Findings:**
- ✅ **rsmpi** is the production-ready MPI library for Rust (v0.8.0, 2024)
- ⚠️ **Parallel I/O** to single shared files is not yet well-supported in Rust ecosystem
- ✅ **File-per-processor** model (Nemesis-style) is achievable with current tools
- 🔬 **Active research** ongoing to improve Rust MPI type safety and HPC compatibility

---

## Table of Contents

1. [Background](#background)
2. [Exodus Parallel I/O Models](#exodus-parallel-io-models)
3. [Rust MPI Ecosystem Analysis](#rust-mpi-ecosystem-analysis)
4. [Implementation Options](#implementation-options)
5. [Recommended Approach](#recommended-approach)
6. [Technical Roadmap](#technical-roadmap)
7. [Challenges and Mitigations](#challenges-and-mitigations)
8. [References](#references)

---

## Background

### What is MPI?

The Message Passing Interface (MPI) is the de facto standard for distributed memory parallel computing on HPC systems. It enables:
- **Process coordination** across compute nodes
- **Point-to-point communication** between processes
- **Collective operations** (broadcast, reduce, gather, scatter)
- **Parallel I/O** to shared files (MPI-IO)

### Exodus and Parallel Computing

The Exodus II library is widely used in computational mechanics and finite element analysis on HPC systems. Parallel applications typically use Exodus in one of two ways:

1. **File-per-processor** (Nemesis model) - Each MPI rank writes its own file
2. **Collective parallel I/O** - All ranks write to a single shared file using MPI-IO

The original **Nemesis library** provided parallel-specific API extensions for Exodus, supporting file-per-processor workflows with:
- Load balancing metadata
- Communication maps between processors
- Partial I/O operations for domain-decomposed meshes
- Global metadata aggregation

These Nemesis functions have been integrated into modern Exodus API (with `ex_*` prefix replacing `ne_*`).

---

## Exodus Parallel I/O Models

### 1. File-Per-Processor (Nemesis Model)

**Description:** Each MPI rank writes a separate `.exo` file containing its local mesh partition.

**Workflow:**
```
mesh.exo.4.0  ← Rank 0's portion
mesh.exo.4.1  ← Rank 1's portion
mesh.exo.4.2  ← Rank 2's portion
mesh.exo.4.3  ← Rank 3's portion
```

**Advantages:**
- ✅ No file locking or contention
- ✅ Scales to thousands of processors
- ✅ Simple error recovery (isolated failures)
- ✅ Standard Exodus format (no special parallel build needed)
- ✅ Well-supported on all HPC file systems

**Disadvantages:**
- ❌ Many small files to manage
- ❌ Post-processing requires file assembly
- ❌ Higher filesystem metadata overhead

**Rust Implementation Feasibility:** **HIGH** - Requires only MPI rank information, no parallel I/O

**API Requirements:**
- Load balancing parameters (`ex_put_loadbal_param`)
- Communication maps (`ex_put_cmap_params`, `ex_put_node_cmap`, `ex_put_elem_cmap`)
- Processor node/element maps (`ex_put_processor_node_maps`, `ex_put_processor_elem_maps`)
- Global metadata (`ex_put_init_global`, `ex_put_ns_param_global`, `ex_put_ss_param_global`)
- Partial I/O operations (`ex_put_partial_*`)

### 2. Collective Parallel I/O (Single Shared File)

**Description:** All MPI ranks write to a single shared `.exo` file using parallel HDF5/NetCDF-4.

**Workflow:**
```
All ranks → mesh.exo (shared file)
```

**Advantages:**
- ✅ Single output file (easier management)
- ✅ Lower filesystem metadata overhead
- ✅ Built-in consistency guarantees

**Disadvantages:**
- ❌ Requires MPI-enabled HDF5 and NetCDF-4
- ❌ File locking and coordination overhead
- ❌ Complex error recovery
- ❌ Scalability challenges beyond ~1000 ranks

**Rust Implementation Feasibility:** **LOW-MEDIUM** - Requires full stack parallel I/O support

**Requirements:**
- HDF5 built with `--enable-parallel`
- NetCDF-4 built with `--enable-pnetcdf` or parallel HDF5
- MPI-IO support in all layers
- Rust bindings that expose parallel I/O APIs

---

## Rust MPI Ecosystem Analysis

### Primary MPI Library: rsmpi

**Repository:** https://github.com/rsmpi/rsmpi
**Crate:** `mpi` (version 0.8.0, released 2024)
**License:** MIT/Apache-2.0
**Downloads:** ~3,154/month (as of 2024)

#### Supported MPI Implementations

**Tested in CI:**
- OpenMPI 4.0.3 (Ubuntu 20.04)
- OpenMPI 4.1.2 (macOS)
- MPICH 3.3.2 (Ubuntu 20.04)
- MS-MPI 10.1.2 (Windows 2022)

**User-reported success:**
- IBM Spectrum MPI 10.3.0.1
- Cray MPI 8.1.16 with PrgEnv-amd/8.3.3
- Intel MPI (with caveats)

#### Feature Coverage

**✅ Fully Supported (MPI-3.1):**
- Communicator management
- Point-to-point messaging (blocking and non-blocking)
- Collective operations (broadcast, reduce, gather, scatter, allreduce, alltoall)
- Derived datatypes
- Process topologies (Cartesian, graph)
- User-defined reduction operations

**❌ Not Supported:**
- Parallel I/O (MPI-IO) - **Critical gap for collective parallel I/O**
- One-sided communication (RMA)
- Inter-communicators
- Dynamic process management

#### Production Readiness

**Strengths:**
- ✅ Memory-safe abstractions over unsafe C bindings
- ✅ Type-safe communication (compile-time type checking)
- ✅ Active maintenance (567 GitHub stars, ongoing commits)
- ✅ Recent research (2024) improving type safety
- ✅ Used in production HPC applications (e.g., petsc-rs)

**Challenges:**
- ⚠️ **Build dependency on bindgen/libclang** - Not always available on HPC login nodes
- ⚠️ **SLURM compatibility issues** - Programs may freeze in `mpi::initialize()` within SLURM jobs
- ⚠️ **Cray-specific bugs** - Locking issues reported on Cray systems
- ⚠️ **Early adoption phase** - Not yet widespread in production HPC

#### Recent Research (2024-2025)

A 2024 paper published at Euro-Par introduced **TypedCommunicator**, enhancing rsmpi with:
- Static type verification for point-to-point communication
- Compile-time detection of type mismatches
- Zero runtime overhead
- Improved developer productivity

Reference: "Enhancing Type Safety in MPI with Rust: A Statically Verified Approach for RSMPI" (2024)

### HDF5 and NetCDF Parallel Support

#### hdf5-rust

**Repository:** https://github.com/aldanor/hdf5-rust
**Crate:** `hdf5` (version 0.8.x)
**Parallel Support:** ✅ **YES** (OpenMPI and MPICH tested on Linux/macOS)

**Features:**
- Thread-safe API (even with non-threadsafe libhdf5)
- Native HDF5 type representation
- ndarray integration
- Derive macros for automatic type mapping

**MPI Configuration:**
- Requires HDF5 built with `--enable-parallel`
- `mpio` feature flag in `hdf5-sys` crate
- Not supported in static linking mode

**Limitation:** The Rust bindings support linking against parallel HDF5, but **high-level parallel I/O APIs are not exposed**. Applications would need to manually coordinate writes using MPI collectives.

#### netcdf-rust

**Repository:** https://github.com/georust/netcdf
**Crate:** `netcdf` (version 0.11.x)
**Parallel Support:** ❌ **NO**

**Current Status:**
- Uses **global mutex** to serialize all library access
- Thread-safe but not parallel
- No exposed parallel I/O APIs
- Underlying `netcdf-c` library may support parallel I/O, but bindings don't expose it

**Impact:** Cannot use collective parallel I/O through Rust NetCDF bindings, even if underlying libraries support it.

### Alternative Approaches

#### 1. Lamellar

**Repository:** https://github.com/latesnow/lamellar-rt
**Description:** PGAS (Partitioned Global Address Space) runtime for Rust

**Paradigm:** Asynchronous tasking and global memory model (different from MPI message-passing)

**Use Case:** Modern alternative to MPI for certain workloads, but not compatible with existing MPI ecosystems or file formats expecting MPI-IO.

**Verdict for Exodus:** ❌ Not suitable - Exodus parallel I/O expects MPI semantics

#### 2. Rayon

**Crate:** `rayon`
**Description:** Data parallelism within a single node

**Use Case:** Multi-threaded array operations, parallel iteration

**Verdict for Exodus:** ✅ Already used in exodus-rs for in-process parallelism, but not for distributed computing

#### 3. Direct C FFI

**Approach:** Call MPI C library directly via FFI

**Advantages:**
- ✅ Access to full MPI functionality
- ✅ No dependency on rsmpi

**Disadvantages:**
- ❌ Unsafe code throughout
- ❌ Manual memory management
- ❌ No type safety
- ❌ Error-prone

**Verdict:** ❌ Not recommended - Defeats purpose of Rust safety guarantees

---

## Implementation Options

### Option 1: File-Per-Processor with rsmpi (Recommended)

**Approach:** Implement Nemesis-style parallel support using rsmpi for MPI coordination.

**Architecture:**
```rust
use mpi::topology::Rank;
use exodus_rs::ExodusFile;

fn parallel_write(comm: &impl Communicator) -> Result<(), ExodusError> {
    let rank = comm.rank();
    let size = comm.size();

    // Each rank writes its own file
    let filename = format!("mesh.exo.{}.{}", size, rank);
    let mut file = ExodusFile::create(&filename, Default::default())?;

    // Write local mesh partition
    file.put_init_global(...)?;           // Global mesh metadata
    file.put_loadbal_param(...)?;         // Load balancing info
    file.put_cmap_params(...)?;           // Communication maps
    file.put_processor_node_maps(...)?;   // Inter-processor node mapping

    // Standard exodus operations for local data
    file.put_coords(...)?;
    file.put_block(...)?;
    // ... etc

    Ok(())
}
```

**Implementation Phases:**

**Phase 1: Core MPI Integration (2-3 weeks)**
- Add `rsmpi` as optional dependency
- Add `mpi` feature flag
- Implement MPI-aware file naming utilities
- Create communicator wrapper types

**Phase 2: Nemesis API (3-4 weeks)**
- Implement load balancing parameter storage
- Implement communication maps (node/element)
- Implement processor maps
- Implement global metadata aggregation
- Add partial I/O convenience methods

**Phase 3: Examples and Testing (2 weeks)**
- Create parallel mesh decomposition examples
- Test with OpenMPI and MPICH
- Validate on HPC systems (if available)
- Documentation and tutorials

**Advantages:**
- ✅ Achievable with current Rust ecosystem
- ✅ No parallel I/O dependency
- ✅ Standard Exodus files (compatible with C library)
- ✅ Proven scalability model
- ✅ Type-safe MPI operations

**Disadvantages:**
- ❌ Generates multiple files
- ❌ Requires post-processing for visualization
- ❌ rsmpi build challenges on some HPC systems

**Risk Level:** **LOW-MEDIUM**

### Option 2: Collective Parallel I/O (Future Work)

**Approach:** Implement true collective parallel I/O using parallel HDF5/NetCDF-4.

**Requirements:**
- Extend `hdf5-rust` or `netcdf-rust` with parallel I/O APIs
- Build entire dependency chain with parallel support
- Coordinate MPI collective writes through Rust

**Architecture (Conceptual):**
```rust
use mpi::topology::Communicator;
use exodus_rs::ExodusFile;

fn collective_write(comm: &impl Communicator) -> Result<(), ExodusError> {
    // Open file with MPI communicator
    let mut file = ExodusFile::create_parallel("mesh.exo", comm, options)?;

    // All ranks participate in collective writes
    file.put_coords_collective(...)?;
    file.put_block_collective(...)?;

    Ok(())
}
```

**Challenges:**
- ⚠️ Requires upstream changes to `hdf5-rust` or `netcdf-rust`
- ⚠️ Complex HDF5 parallel I/O semantics
- ⚠️ MPI-IO support in Rust ecosystem is immature
- ⚠️ Testing requires parallel HPC environment

**Advantages:**
- ✅ Single output file
- ✅ Modern I/O pattern
- ✅ Future-proof approach

**Risk Level:** **HIGH** - Significant ecosystem work required

**Timeline:** 6-12 months (includes upstream contributions)

### Option 3: Hybrid Approach

**Approach:** Support both file-per-processor (Phase 1) and collective I/O (Phase 2+).

**Architecture:**
```rust
pub enum ParallelMode {
    FilePerProcessor,
    CollectiveIO,
}

impl ExodusFile {
    pub fn create_parallel(
        path: &str,
        comm: &impl Communicator,
        mode: ParallelMode,
        options: CreateOptions,
    ) -> Result<Self, ExodusError> {
        match mode {
            ParallelMode::FilePerProcessor => {
                // Nemesis-style implementation
                let rank = comm.rank();
                let size = comm.size();
                let filename = format!("{}.{}.{}", path, size, rank);
                Self::create(&filename, options)
            }
            ParallelMode::CollectiveIO => {
                // Future: true parallel I/O
                unimplemented!("Collective I/O not yet implemented")
            }
        }
    }
}
```

**Advantages:**
- ✅ Immediate functionality (file-per-processor)
- ✅ Migration path to collective I/O
- ✅ User choice based on use case

**Risk Level:** **MEDIUM**

---

## Recommended Approach

### Three-Phase Implementation Strategy

#### Phase 1: File-Per-Processor Foundation (Short-term: 3-4 months)

**Goal:** Production-ready file-per-processor parallel support

**Deliverables:**
1. Add `rsmpi` as optional dependency (feature = `"mpi"`)
2. Implement Nemesis-compatible API functions:
   - `ex_put_init_global` / `ex_get_init_global`
   - `ex_put_loadbal_param` / `ex_get_loadbal_param`
   - `ex_put_cmap_params` / `ex_get_cmap_params`
   - `ex_put_node_cmap` / `ex_get_node_cmap`
   - `ex_put_elem_cmap` / `ex_get_elem_cmap`
   - `ex_put_processor_node_maps` / `ex_get_processor_node_maps`
   - `ex_put_processor_elem_maps` / `ex_get_processor_elem_maps`
3. Create parallel-aware builder API
4. Comprehensive test suite (serial + MPI environments)
5. Example applications demonstrating parallel decomposition
6. Documentation and migration guide

**Success Criteria:**
- ✅ Rust exodus library can write Nemesis-compatible files
- ✅ Files readable by C Exodus library
- ✅ Scalability testing on ≥256 cores
- ✅ Integration with domain decomposition tools (e.g., `decomp`, Zoltan)

#### Phase 2: HPC Platform Hardening (Medium-term: 2-3 months)

**Goal:** Ensure robust operation on production HPC systems

**Deliverables:**
1. Testing on major HPC platforms:
   - Cray systems
   - IBM CORAL systems
   - Lustre file systems
   - GPFS file systems
2. Build system improvements for HPC environments
3. Pre-built binaries or Spack integration
4. Performance benchmarking and optimization
5. Collaboration with rsmpi maintainers on HPC issues

**Success Criteria:**
- ✅ Builds and runs on ≥3 different HPC platforms
- ✅ Performance within 10% of C Exodus library
- ✅ Documented installation procedures for common HPC systems

#### Phase 3: Advanced Features (Long-term: 6-12 months)

**Goal:** Explore collective parallel I/O and advanced patterns

**Deliverables:**
1. Investigate parallel HDF5 bindings enhancement
2. Prototype collective I/O implementation
3. Evaluate PnetCDF as alternative backend
4. Asynchronous I/O support
5. In-situ analysis integration (e.g., ADIOS2, Catalyst)

**Success Criteria:**
- ✅ Proof-of-concept collective I/O working
- ✅ Performance comparison: file-per-processor vs collective
- ✅ Community feedback and adoption metrics

---

## Technical Roadmap

### Milestone 1: Project Setup (Week 1-2)

**Tasks:**
- [ ] Add `mpi` dependency to `Cargo.toml` with feature flag
- [ ] Configure CI to test with OpenMPI and MPICH
- [ ] Create `src/parallel.rs` module
- [ ] Define public API types and traits

**Deliverables:**
```toml
# Cargo.toml
[dependencies]
mpi = { version = "0.8", optional = true }

[features]
mpi = ["dep:mpi"]
```

```rust
// src/parallel.rs
#[cfg(feature = "mpi")]
pub mod mpi_support {
    use mpi::topology::Communicator;
    // ...
}
```

### Milestone 2: Basic MPI Integration (Week 3-4)

**Tasks:**
- [ ] Implement communicator-aware file creation
- [ ] Add rank-based filename generation
- [ ] Create utility functions for MPI coordination

**Deliverables:**
```rust
pub struct ParallelContext {
    rank: i32,
    size: i32,
}

impl ParallelContext {
    pub fn from_communicator(comm: &impl Communicator) -> Self {
        Self {
            rank: comm.rank(),
            size: comm.size(),
        }
    }

    pub fn filename(&self, base: &str) -> String {
        format!("{}.{}.{}", base, self.size, self.rank)
    }
}
```

### Milestone 3: Nemesis Metadata (Week 5-8)

**Tasks:**
- [ ] Implement load balancing parameter storage
- [ ] Implement communication map structures
- [ ] Add processor map support
- [ ] Create global metadata aggregation helpers

**API Design:**
```rust
impl<M: Mode> ExodusFile<M> {
    /// Write load balancing parameters
    pub fn put_loadbal_param(
        &mut self,
        num_internal_nodes: i64,
        num_border_nodes: i64,
        num_external_nodes: i64,
        num_internal_elems: i64,
        num_border_elems: i64,
        num_node_cmaps: i64,
        num_elem_cmaps: i64,
        processor: i32,
    ) -> Result<(), ExodusError>;

    /// Write communication map parameters
    pub fn put_cmap_params(
        &mut self,
        node_cmap_ids: &[i64],
        node_cmap_node_cnts: &[i64],
        elem_cmap_ids: &[i64],
        elem_cmap_elem_cnts: &[i64],
        processor: i32,
    ) -> Result<(), ExodusError>;

    /// Write node communication map
    pub fn put_node_cmap(
        &mut self,
        map_id: i64,
        node_ids: &[i64],
        proc_ids: &[i32],
        processor: i32,
    ) -> Result<(), ExodusError>;
}
```

### Milestone 4: Testing and Examples (Week 9-10)

**Tasks:**
- [ ] Create parallel mesh decomposition example
- [ ] Write integration tests with MPI
- [ ] Verify interoperability with C Exodus
- [ ] Performance benchmarking

**Example Application:**
```rust
// examples/parallel_write.rs
use mpi::traits::*;
use exodus_rs::{ExodusFile, ParallelContext};

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let ctx = ParallelContext::from_communicator(&world);

    let filename = ctx.filename("parallel_mesh.exo");
    let mut file = ExodusFile::create(&filename, Default::default()).unwrap();

    // Each rank writes its local mesh
    file.put_init_global(...).unwrap();
    file.put_loadbal_param(...).unwrap();
    // ...

    println!("Rank {} wrote {}", ctx.rank, filename);
}
```

### Milestone 5: Documentation (Week 11-12)

**Tasks:**
- [ ] Write MPI integration guide
- [ ] Document Nemesis API compatibility
- [ ] Create HPC deployment guide
- [ ] Add troubleshooting section

---

## Challenges and Mitigations

### Challenge 1: rsmpi Build Dependencies on HPC Systems

**Problem:** Many HPC login nodes lack modern libclang/bindgen, preventing rsmpi compilation.

**Mitigations:**
1. **Pre-built binaries:** Provide pre-compiled libraries for common HPC platforms
2. **Spack integration:** Distribute via Spack package manager (widely used in HPC)
3. **Static linking:** Bundle dependencies to reduce system requirements
4. **Docker/Singularity:** Container images for consistent build environments
5. **Vendor modules:** Work with HPC centers to provide rsmpi as a module

**Community Efforts:** rsmpi issue #28 discusses removing bindgen dependency

### Challenge 2: SLURM Compatibility

**Problem:** Some users report freezing during `mpi::initialize()` within SLURM jobs.

**Mitigations:**
1. **Environment detection:** Auto-detect SLURM and apply workarounds
2. **Explicit initialization:** Provide alternative initialization paths
3. **Documentation:** Clear guidance for SLURM users
4. **Upstream fixes:** Collaborate with rsmpi maintainers

**Workaround:**
```rust
// Potential workaround for SLURM environments
if env::var("SLURM_JOB_ID").is_ok() {
    // Use PMI initialization instead of default
    // (requires rsmpi support for PMI)
}
```

### Challenge 3: Testing Without HPC Access

**Problem:** CI/CD systems don't have multi-node MPI environments.

**Mitigations:**
1. **Local multi-process testing:** Use `mpirun -n 4` on single-node CI runners
2. **Mock MPI context:** Create test doubles for MPI communicators
3. **Community testing:** Partner with HPC centers for acceptance testing
4. **Simulation mode:** Test parallel logic with simulated ranks

**CI Configuration:**
```yaml
# .github/workflows/mpi-tests.yml
- name: Install MPI
  run: sudo apt-get install -y libopenmpi-dev openmpi-bin

- name: Run MPI tests
  run: |
    mpirun -n 4 cargo test --features mpi --test parallel_tests
```

### Challenge 4: Parallel I/O API Maturity

**Problem:** Rust ecosystem lacks mature parallel I/O APIs for HDF5/NetCDF.

**Mitigations:**
1. **Start with file-per-processor:** Proven model that doesn't require parallel I/O
2. **Upstream contributions:** Work with hdf5-rust/netcdf-rust maintainers
3. **FFI layer:** Implement parallel I/O via direct C calls if needed
4. **Alternative backends:** Investigate ADIOS2 or PnetCDF bindings

### Challenge 5: Debugging MPI Applications in Rust

**Problem:** MPI bugs can manifest as deadlocks or race conditions, harder to debug without standard tools.

**Mitigations:**
1. **Extensive logging:** Use `env_logger` or `tracing` with rank-aware output
2. **Sanitizers:** Leverage Rust sanitizers (TSAN, ASAN) with MPI
3. **MPI debuggers:** Test with TotalView, DDT, or gdb4hpc
4. **Deterministic testing:** Use fixed random seeds and controlled inputs

---

## Performance Considerations

### Expected Performance

**File-per-processor model:**
- Should achieve **near-linear scaling** to 1000+ cores
- Overhead primarily from filesystem metadata operations
- Write bandwidth limited by storage system, not MPI

**Benchmark targets:**
- Single-rank performance: Within 5% of C Exodus library
- Weak scaling efficiency: >90% up to 512 cores
- Strong scaling efficiency: >80% up to 256 cores

### Optimization Strategies

1. **Chunk cache tuning:** Use existing `PerformanceConfig` for HDF5 caching
2. **Buffered writes:** Minimize NetCDF-4 API calls via write buffering
3. **Collective metadata operations:** Coordinate metadata writes across ranks
4. **Async I/O:** Overlap computation and I/O using non-blocking operations (future)

---

## Ecosystem Collaboration Opportunities

### 1. rsmpi Enhancements

**Contribute:**
- HPC platform testing and bug reports
- Documentation improvements for HPC users
- Investigate bindgen-free build options

### 2. hdf5-rust Parallel I/O

**Contribute:**
- Expose parallel I/O APIs (H5Pset_fapl_mpio, H5Dwrite collective)
- Add MPI communicator management
- Create parallel I/O examples

### 3. netcdf-rust Parallel Support

**Contribute:**
- Remove global mutex for parallel builds
- Add nc_open_par / nc_create_par support
- Coordinate with PnetCDF bindings

### 4. Scientific Computing in Rust Community

**Engage:**
- Present at Scientific Computing in Rust workshops
- Share HPC deployment experiences
- Collaborate on shared infrastructure (e.g., MPI testing harness)

---

## Alternative: FFI-Based Approach

### When to Consider Direct FFI

If rsmpi proves unsuitable due to HPC compatibility issues, consider:

**Approach:** Minimal FFI wrapper around MPI C library

**Scope:**
- Only wrap essential MPI functions (init, finalize, comm_rank, comm_size, barrier)
- Keep unsafe code isolated in dedicated module
- Provide safe Rust API layer on top

**Example:**
```rust
// src/ffi/mpi.rs (unsafe FFI layer)
#[link(name = "mpi")]
extern "C" {
    fn MPI_Init(argc: *mut i32, argv: *mut *mut *mut i8) -> i32;
    fn MPI_Comm_rank(comm: MPI_Comm, rank: *mut i32) -> i32;
    fn MPI_Comm_size(comm: MPI_Comm, size: *mut i32) -> i32;
    fn MPI_Finalize() -> i32;
}

// src/parallel/mpi_simple.rs (safe wrapper)
pub struct SimpleMPI {
    rank: i32,
    size: i32,
}

impl SimpleMPI {
    pub fn initialize() -> Result<Self, MPIError> {
        unsafe {
            let ret = MPI_Init(std::ptr::null_mut(), std::ptr::null_mut());
            if ret != 0 { return Err(MPIError::InitFailed); }

            let mut rank = 0;
            let mut size = 0;
            MPI_Comm_rank(MPI_COMM_WORLD, &mut rank);
            MPI_Comm_size(MPI_COMM_WORLD, &mut size);

            Ok(SimpleMPI { rank, size })
        }
    }
}
```

**Pros:**
- ✅ Minimal dependencies
- ✅ Direct control over MPI calls
- ✅ Easier to debug on HPC systems

**Cons:**
- ❌ Unsafe code (reduces Rust safety benefits)
- ❌ Manual memory management
- ❌ Limited to basic MPI functionality
- ❌ Maintenance burden

**Recommendation:** Use as fallback only if rsmpi blockers are insurmountable.

---

## Compatibility Matrix

### Supported Configurations (Phase 1 Target)

| MPI Implementation | Platform | Status | Priority |
|--------------------|----------|--------|----------|
| OpenMPI 4.x | Linux (Ubuntu/RHEL) | ✅ Tested | High |
| MPICH 3.3+ | Linux | ✅ Tested | High |
| Cray MPI 8.x | Cray XC/EX | 🧪 Community-tested | High |
| Intel MPI | Linux | 🧪 Experimental | Medium |
| IBM Spectrum MPI | IBM Power | 🧪 Community-tested | Medium |
| MS-MPI | Windows | ✅ Tested | Low |

### HPC Schedulers

| Scheduler | Status | Notes |
|-----------|--------|-------|
| SLURM | ⚠️ Known issues | Workarounds available |
| PBS/Torque | 🧪 Untested | Should work (standard MPI) |
| LSF | 🧪 Untested | Should work (standard MPI) |
| Flux | ✅ Should work | Uses standard MPI init |

### File Systems

| File System | File-per-processor | Collective I/O (future) |
|-------------|-------------------|------------------------|
| Lustre | ✅ Excellent | ✅ Good |
| GPFS | ✅ Excellent | ✅ Good |
| NFS | ⚠️ Poor scaling | ❌ Not recommended |
| Local SSD | ✅ Excellent | N/A |

---

## Success Metrics

### Phase 1 Completion Criteria

- [ ] All Nemesis API functions implemented
- [ ] 100% compatibility with C Exodus parallel files
- [ ] Test suite passing with 2, 4, 8, 16 MPI ranks
- [ ] Performance within 10% of C library (file-per-processor)
- [ ] Documentation complete and reviewed
- [ ] At least 2 example applications
- [ ] Tested on ≥2 HPC platforms (e.g., OpenMPI + MPICH)

### Community Adoption Metrics (6-12 months post-release)

- [ ] ≥5 external users/projects
- [ ] ≥3 HPC sites reporting successful deployment
- [ ] ≥10 citations in research publications
- [ ] Integration with parallel mesh generation tools

---

## References

### Rust MPI Ecosystem

1. **rsmpi GitHub Repository**
   https://github.com/rsmpi/rsmpi

2. **"Enhancing Type Safety in MPI with Rust: A Statically Verified Approach for RSMPI"**
   Euro-Par 2024, September 2024
   https://link.springer.com/chapter/10.1007/978-3-031-97196-9_11

3. **Scientific Computing in Rust 2025**
   https://scientificcomputing.rs/2025/

4. **"Using Rust for High-Performance Computing: A 2025 Guide"**
   https://www.nxsyed.com/blog/rust-for-hpc

### HDF5 and NetCDF Parallel I/O

5. **hdf5-rust GitHub Repository**
   https://github.com/aldanor/hdf5-rust

6. **georust/netcdf GitHub Repository**
   https://github.com/georust/netcdf

7. **PnetCDF (Parallel netCDF)**
   https://parallel-netcdf.github.io/

### Exodus and Nemesis

8. **SEACAS Documentation**
   https://sandialabs.github.io/seacas-docs/

9. **Nemesis to Exodus API Mapping**
   `packages/seacas/libraries/exodus/include/nemesis-to-exodus-api-mapping.md`

10. **Exodus II File Format Specification**
    https://sandialabs.github.io/seacas-docs/exodusII/

### HPC Best Practices

11. **"A large-scale study of MPI usage in open-source HPC applications"**
    SC '19 Proceedings
    https://dl.acm.org/doi/10.1145/3295500.3356176

12. **OpenHPC Community**
    https://openhpc.community/

---

## Next Steps

### Immediate Actions (This Quarter)

1. **Create RFC:** Propose MPI integration to exodus-rs community
2. **Prototype:** Build minimal file-per-processor example with rsmpi
3. **Engage rsmpi community:** Report HPC-specific issues, offer testing
4. **Design API:** Finalize parallel API surface and naming conventions

### Short-term (Next 3-6 Months)

1. **Implement Phase 1:** Complete file-per-processor support
2. **HPC testing:** Partner with HPC centers for platform validation
3. **Documentation:** Write comprehensive parallel I/O guide
4. **Benchmarking:** Compare performance with C Exodus library

### Long-term (6-12 Months)

1. **Phase 2 planning:** Design collective I/O architecture
2. **Ecosystem contributions:** Work on hdf5-rust parallel APIs
3. **Community building:** Present at HPC/Rust conferences
4. **Production adoption:** Support early adopters and gather feedback

---

## Conclusion

Implementing MPI support in the Rust exodus library is **achievable and valuable** for the HPC community. The recommended three-phase approach prioritizes:

1. ✅ **Near-term value:** File-per-processor model using rsmpi (3-4 months)
2. 🔧 **HPC hardening:** Platform-specific testing and optimization (2-3 months)
3. 🔬 **Future research:** Collective parallel I/O as ecosystem matures (6-12 months)

**Key Insight:** The file-per-processor model (Nemesis-style) is the pragmatic choice given current Rust HPC ecosystem maturity. This approach:
- Leverages proven, scalable I/O patterns
- Avoids dependency on immature parallel I/O bindings
- Provides immediate value to HPC users
- Maintains 100% safety guarantees of Rust

**Risk Assessment:** **LOW-MEDIUM** for Phase 1, **MEDIUM-HIGH** for collective I/O

**Recommendation:** **Proceed with Phase 1 implementation** using rsmpi for file-per-processor parallel support.

---

**Document Version:** 1.0
**Author:** AI Research Assistant
**Review Status:** Draft - Awaiting Community Review
**License:** BSD-3-Clause (same as SEACAS)
