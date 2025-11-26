# NetCDF/Exodus Variable Storage Format Support

**Status: ✅ IMPLEMENTED (2025-11-26)**

## Issue Summary

The exodus-rs library previously failed to read nodal variable data from files that use the **combined 3D storage format**. The error manifested as `Error: Exodus(NetCdf(Str("Conversion not supported")))` or `VariableNotDefined`.

This has now been fixed by implementing automatic storage format detection.

## Root Cause

Exodus II supports two storage formats for variable data, and a single file can mix formats:

### Separate Variable Format (currently supported)
```
double vals_nod_var1(time_step, num_nodes) ;
double vals_nod_var2(time_step, num_nodes) ;
double vals_elem_var1eb1(time_step, num_el_in_blk1) ;
```

### Combined 3D Format (not supported for nodal)
```
double vals_nod_var(time_step, num_nod_var, num_nodes) ;
double vals_elem_var(time_step, num_elem_var, num_elem) ;
```

### Example: Mixed Format File
From `ncdump -h multi_physics.exo`:
```
double vals_elem_var1eb1(time_step, num_el_in_blk1) ;  // Separate
double vals_elem_var2eb1(time_step, num_el_in_blk1) ;  // Separate
double vals_nod_var(time_step, num_nod_var, num_nodes) ;  // Combined
```

The current code only looks for separate format variables (`vals_nod_var1`, etc.).

## Implementation Plan

### Design Principle

**Detect the file schema upfront during `open()`** rather than using error-catching fallbacks scattered throughout the library. NetCDF is self-describing - the variables and dimensions section IS the schema.

### Phase 1: Add Storage Format Types

Add to `types.rs`:

```rust
/// How variable data is stored in the NetCDF file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VarStorageMode {
    /// Separate variables per index: vals_nod_var1, vals_nod_var2, etc.
    /// Shape: (time_step, num_entities)
    #[default]
    Separate,
    /// Combined 3D array: vals_nod_var
    /// Shape: (time_step, num_vars, num_entities)
    Combined,
    /// No variables of this type present
    None,
}

/// Storage format for each entity type, detected on file open
#[derive(Debug, Clone, Default)]
pub struct FileStorageFormat {
    pub nodal: VarStorageMode,
    pub elem_block: VarStorageMode,
    pub edge_block: VarStorageMode,
    pub face_block: VarStorageMode,
    pub node_set: VarStorageMode,
    pub edge_set: VarStorageMode,
    pub face_set: VarStorageMode,
    pub side_set: VarStorageMode,
    pub elem_set: VarStorageMode,
    pub global: VarStorageMode,  // Always Combined (vals_glo_var)
}
```

### Phase 2: Store Format in ExodusFile

Modify `lib.rs` to include detected format:

```rust
pub struct ExodusFile<M: FileMode> {
    pub(crate) nc_file: netcdf::File,
    pub(crate) storage_format: FileStorageFormat,  // NEW
    _marker: PhantomData<M>,
}
```

### Phase 3: Detect Format on Open

Add detection logic in `open()` (and `append()`):

```rust
fn detect_storage_format(nc_file: &netcdf::File) -> FileStorageFormat {
    FileStorageFormat {
        nodal: detect_var_storage(nc_file, "vals_nod_var", "vals_nod_var1"),
        elem_block: detect_var_storage(nc_file, "vals_elem_var", "vals_elem_var1eb1"),
        global: VarStorageMode::Combined,  // Always uses vals_glo_var
        // ... other entity types
    }
}

fn detect_var_storage(
    nc_file: &netcdf::File,
    combined_name: &str,
    separate_name: &str,
) -> VarStorageMode {
    if nc_file.variable(combined_name).is_some() {
        VarStorageMode::Combined
    } else if nc_file.variable(separate_name).is_some() {
        VarStorageMode::Separate
    } else {
        VarStorageMode::None
    }
}
```

### Phase 4: Update var() to Use Detected Format

```rust
pub fn var(&self, step: usize, var_type: EntityType, entity_id: EntityId, var_index: usize) -> Result<Vec<f64>> {
    match var_type {
        EntityType::Nodal => match self.storage_format.nodal {
            VarStorageMode::Combined => self.read_var_combined("vals_nod_var", step, var_index),
            VarStorageMode::Separate => self.read_var_separate(var_type, entity_id, step, var_index),
            VarStorageMode::None => Err(ExodusError::VariableNotDefined("nodal variables".into())),
        },
        // ... other entity types
    }
}

fn read_var_combined(&self, var_name: &str, step: usize, var_index: usize) -> Result<Vec<f64>> {
    let var = self.nc_file.variable(var_name)
        .ok_or_else(|| ExodusError::VariableNotDefined(var_name.into()))?;
    // Read slice: (step, var_index, 0..num_entities)
    Ok(var.get_values((step..step+1, var_index..var_index+1, ..))?)
}
```

### Phase 5: Expose Format for Inspection

Add a public method to query the detected format:

```rust
impl<M: FileMode> ExodusFile<M> {
    /// Get the detected storage format for this file
    pub fn storage_format(&self) -> &FileStorageFormat {
        &self.storage_format
    }
}
```

## Files Modified

1. `rust/exodus-rs/src/types.rs` - Added `VarStorageMode` and `FileStorageFormat`
2. `rust/exodus-rs/src/lib.rs` - Exports new types
3. `rust/exodus-rs/src/file.rs` - Added `storage_format` field to `FileMetadata`, detect format in `open()` and `append()`, added `storage_format()` accessor
4. `rust/exodus-rs/src/variable.rs` - Updated `var()` to use detected format with `read_var_combined()` and `read_var_separate()` helpers

## Testing

Test file: `rust/exodus-rs/tests/test_storage_format.rs`

1. `test_storage_format_detection_separate` - Verifies detection of separate format files
2. `test_storage_format_detection_no_vars` - Verifies detection when no variables present
3. `test_storage_format_global_variables` - Tests global variable format detection
4. `test_storage_format_accessor` - Tests the public `storage_format()` accessor
5. `test_storage_format_append_mode` - Tests format detection in append mode
6. `test_var_storage_mode_default` - Tests enum default value
7. `test_file_storage_format_default` - Tests struct default values

## Usage

```rust
use exodus_rs::{ExodusFile, mode, VarStorageMode};

// Open a file - format is automatically detected
let file = ExodusFile::<mode::Read>::open("mesh.exo")?;

// Check the detected format
let format = file.storage_format();
match format.nodal {
    VarStorageMode::Combined => println!("Uses combined format"),
    VarStorageMode::Separate => println!("Uses separate format"),
    VarStorageMode::None => println!("No nodal variables"),
}

// Read variables - works automatically regardless of format
let values = file.var(0, EntityType::Nodal, 0, 0)?;
```

## Previous Workaround (No Longer Needed)

The temporary workaround from commit 2d009c0 is no longer needed - the library now handles both formats automatically.
