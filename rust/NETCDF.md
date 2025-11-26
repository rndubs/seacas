# NetCDF/Exodus Variable Storage Format Support

## Issue Summary

The exodus-rs library fails to read nodal variable data from files that use the **combined 3D storage format**. The error manifests as `Error: Exodus(NetCdf(Str("Conversion not supported")))`.

### Root Cause

Exodus II supports two storage formats for nodal variables:

1. **Separate variable format** (currently supported):
   ```
   double vals_nod_var1(time_step, num_nodes) ;
   double vals_nod_var2(time_step, num_nodes) ;
   ```

2. **Combined 3D format** (not supported):
   ```
   double vals_nod_var(time_step, num_nod_var, num_nodes) ;
   ```

The `var()` function in `variable.rs` only looks for the separate format (`vals_nod_var1`, `vals_nod_var2`, etc.). When files use the combined format, the numbered variables don't exist, causing read failures.

## Affected Files

- `rust/exodus-rs/src/variable.rs` - `var()` and `get_var_name_read()` functions

## Implementation Plan

### Phase 1: Add 3D Format Detection and Reading

1. **Modify `var()` function** (around line 1821 in variable.rs):
   - First attempt to read from separate variable format (current behavior)
   - If variable not found, fall back to combined 3D format
   - For nodal variables, read from `vals_nod_var(step, var_index, :)`

2. **Add helper function** `read_nodal_var_combined()`:
   ```rust
   fn read_nodal_var_combined(&self, step: usize, var_index: usize) -> Result<Vec<f64>> {
       let var = self.nc_file.variable("vals_nod_var")
           .ok_or_else(|| ExodusError::VariableNotDefined("vals_nod_var".into()))?;
       // Read slice: (step, var_index, 0..num_nodes)
       Ok(var.get_values((step..step+1, var_index..var_index+1, ..))?)
   }
   ```

3. **Update `var()` for Nodal case**:
   ```rust
   EntityType::Nodal => {
       // Try separate variable format first
       if let Some(var) = self.nc_file.variable(&var_name) {
           Ok(var.get_values((step..step + 1, ..))?)
       } else {
           // Fall back to combined 3D format
           self.read_nodal_var_combined(step, var_index)
       }
   }
   ```

### Phase 2: Extend to Other Entity Types (if needed)

Similar combined formats may exist for:
- Element block variables: `vals_elem_var(time_step, num_elem_var, num_elem)`
- Global variables: Already uses combined format `vals_glo_var(time_step, num_glo_var)`

### Testing

1. Create test files with both storage formats
2. Verify reading works for both formats
3. Test copy-mirror-merge workflow end-to-end

## Temporary Workaround

The copy-mirror-merge workflow now handles read failures gracefully (as of commit 2d009c0):
- Variable name reading uses `unwrap_or_default()`
- Variable data reading catches errors and continues
- Mesh geometry is preserved even when variables can't be read
