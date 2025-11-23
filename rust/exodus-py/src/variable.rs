//! Variable and time step operations for Exodus files

use pyo3::prelude::*;

use crate::error::IntoPyResult;
use crate::file::{ExodusReader, ExodusWriter};
use crate::types::{EntityType, TruthTable};

#[cfg(feature = "numpy")]
use numpy::{PyArray1, PyArray2, PyArrayMethods};

/// Variable operations for ExodusReader
#[pymethods]
impl ExodusReader {
    /// Get variable names for an entity type
    ///
    /// Args:
    ///     var_type: Entity type (e.g., EntityType.GLOBAL, EntityType.NODAL)
    ///
    /// Returns:
    ///     List of variable names
    ///
    /// Example:
    ///     >>> names = reader.variable_names(EntityType.NODAL)
    ///     >>> print(names)  # ['Temperature', 'Pressure']
    fn variable_names(&self, var_type: EntityType) -> PyResult<Vec<String>> {
        self.file.variable_names(var_type.to_rust()).into_py()
    }

    /// Get number of time steps
    ///
    /// Returns:
    ///     Number of time steps in the file
    fn num_time_steps(&self) -> PyResult<usize> {
        self.file.num_time_steps().into_py()
    }

    /// Get all time values
    ///
    /// Returns:
    ///     List of time values for all steps
    fn times(&self) -> PyResult<Vec<f64>> {
        self.file.times().into_py()
    }

    /// Get time value for a specific step
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///
    /// Returns:
    ///     Time value
    fn time(&self, step: usize) -> PyResult<f64> {
        self.file.time(step).into_py()
    }

    /// Read variable values at a time step as NumPy array
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (block ID for block variables, 0 for global/nodal)
    ///     var_index: Variable index (0-based)
    ///
    /// Returns:
    ///     1D NumPy array of variable values
    ///
    /// Example:
    ///     >>> temp = reader.var(0, EntityType.NODAL, 0, 0)
    ///     >>> print(f"Temperature at t=0: {temp}")
    #[cfg(feature = "numpy")]
    fn var<'py>(
        &self,
        py: Python<'py>,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let data = self
            .file
            .var(step, var_type.to_rust(), entity_id, var_index)
            .into_py()?;
        Ok(PyArray1::from_vec(py, data))
    }

    /// Read variable values at a time step as list (deprecated)
    ///
    /// .. deprecated::
    ///     Use :meth:`var` instead for better performance with NumPy arrays
    fn var_list(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
    ) -> PyResult<Vec<f64>> {
        self.file
            .var(step, var_type.to_rust(), entity_id, var_index)
            .into_py()
    }

    /// Read variable values at a time step (no NumPy fallback)
    #[cfg(not(feature = "numpy"))]
    fn var(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
    ) -> PyResult<Vec<f64>> {
        self.file
            .var(step, var_type.to_rust(), entity_id, var_index)
            .into_py()
    }

    /// Read all variables for an entity at a time step
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (block ID for block variables, 0 for global/nodal)
    ///
    /// Returns:
    ///     Flat list of all variable values
    fn var_multi(&self, step: usize, var_type: EntityType, entity_id: i64) -> PyResult<Vec<f64>> {
        self.file
            .var_multi(step, var_type.to_rust(), entity_id)
            .into_py()
    }

    /// Read variable time series as 2D NumPy array
    ///
    /// Args:
    ///     start_step: Starting time step index (0-based)
    ///     end_step: Ending time step index (exclusive)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (block ID for block variables, 0 for global/nodal)
    ///     var_index: Variable index (0-based)
    ///
    /// Returns:
    ///     2D NumPy array with shape (num_time_steps, num_entities)
    ///
    /// Example:
    ///     >>> data = reader.var_time_series(0, 100, EntityType.NODAL, 0, 0)
    ///     >>> print(data.shape)  # (100, num_nodes)
    #[cfg(feature = "numpy")]
    fn var_time_series<'py>(
        &self,
        py: Python<'py>,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        // Use optimized var_time_series_array() method from Rust
        let arr = self
            .file
            .var_time_series_array(
                start_step,
                end_step,
                var_type.to_rust(),
                entity_id,
                var_index,
            )
            .into_py()?;

        // Convert to NumPy array (zero-copy transfer)
        Ok(PyArray2::from_owned_array(py, arr))
    }

    /// Read variable time series as flat list (deprecated)
    ///
    /// .. deprecated::
    ///     Use :meth:`var_time_series` instead for better performance with NumPy arrays
    fn var_time_series_list(
        &self,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
    ) -> PyResult<Vec<f64>> {
        self.file
            .var_time_series(
                start_step,
                end_step,
                var_type.to_rust(),
                entity_id,
                var_index,
            )
            .into_py()
    }

    /// Read variable time series (no NumPy fallback)
    #[cfg(not(feature = "numpy"))]
    fn var_time_series(
        &self,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
    ) -> PyResult<Vec<f64>> {
        self.file
            .var_time_series(
                start_step,
                end_step,
                var_type.to_rust(),
                entity_id,
                var_index,
            )
            .into_py()
    }

    /// Get truth table for sparse variable storage
    ///
    /// Args:
    ///     var_type: Entity type (must be a block type)
    ///
    /// Returns:
    ///     TruthTable object
    fn truth_table(&self, var_type: EntityType) -> PyResult<TruthTable> {
        let table = self.file.truth_table(var_type.to_rust()).into_py()?;
        Ok(TruthTable::from_rust(&table))
    }

    /// Get reduction variable names for an entity type
    ///
    /// Reduction variables store aggregated/summary values for entire objects
    /// (e.g., assemblies, blocks, sets) rather than for individual entities within those objects.
    ///
    /// Args:
    ///     var_type: Entity type (e.g., EntityType.ASSEMBLY, EntityType.ELEM_BLOCK)
    ///
    /// Returns:
    ///     List of reduction variable names
    ///
    /// Example:
    ///     >>> names = reader.reduction_variable_names(EntityType.ASSEMBLY)
    ///     >>> print(names)  # ['Momentum_X', 'Momentum_Y', 'Kinetic_Energy']
    fn reduction_variable_names(&self, var_type: EntityType) -> PyResult<Vec<String>> {
        self.file
            .reduction_variable_names(var_type.to_rust())
            .into_py()
    }

    /// Read reduction variable values for a time step
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (e.g., assembly ID, block ID, set ID)
    ///
    /// Returns:
    ///     List of reduction variable values
    ///
    /// Example:
    ///     >>> values = reader.get_reduction_vars(0, EntityType.ASSEMBLY, 100)
    ///     >>> print(f"Momentum: {values[0]}, Energy: {values[3]}")
    fn get_reduction_vars(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
    ) -> PyResult<Vec<f64>> {
        self.file
            .get_reduction_vars(step, var_type.to_rust(), entity_id)
            .into_py()
    }
}

/// Variable operations for ExodusWriter
#[pymethods]
impl ExodusWriter {
    /// Define variables for an entity type
    ///
    /// Args:
    ///     var_type: Entity type (e.g., EntityType.GLOBAL, EntityType.NODAL)
    ///     names: List of variable names
    ///
    /// Example:
    ///     >>> writer.define_variables(EntityType.NODAL, ["Temperature", "Pressure"])
    fn define_variables(&mut self, var_type: EntityType, names: Vec<String>) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            file.define_variables(var_type.to_rust(), &name_refs)
                .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Write time value for a time step
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///     time: Time value
    ///
    /// Example:
    ///     >>> writer.put_time(0, 0.0)
    ///     >>> writer.put_time(1, 1.0)
    fn put_time(&mut self, step: usize, time: f64) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.put_time(step, time).into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Write variable values for a time step (accepts NumPy arrays or lists)
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (block ID for block variables, 0 for global/nodal)
    ///     var_index: Variable index (0-based)
    ///     values: Variable values as NumPy array or list
    ///
    /// Example:
    ///     >>> import numpy as np
    ///     >>> writer.put_var(0, EntityType.NODAL, 0, 0, np.array([100.0, 200.0, 300.0]))
    #[cfg(feature = "numpy")]
    fn put_var(
        &mut self,
        _py: Python<'_>,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
        values: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Convert NumPy array or list to Vec
        let values_vec = if let Ok(arr) = values.clone().cast_into::<PyArray1<f64>>() {
            arr.readonly().as_slice()?.to_vec()
        } else {
            values.extract::<Vec<f64>>()?
        };

        if let Some(ref mut file) = self.file {
            file.put_var(step, var_type.to_rust(), entity_id, var_index, &values_vec)
                .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Write variable values for a time step (no NumPy)
    #[cfg(not(feature = "numpy"))]
    fn put_var(
        &mut self,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
        values: Vec<f64>,
    ) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.put_var(step, var_type.to_rust(), entity_id, var_index, &values)
                .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Write all variables for an entity at a time step
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (block ID for block variables, 0 for global/nodal)
    ///     values: Flat list of all variable values
    ///
    /// Example:
    ///     >>> # Write 2 nodal variables for 3 nodes
    ///     >>> writer.put_var_multi(0, EntityType.NODAL, 0,
    ///     ...     [100.0, 200.0, 300.0, 1.0, 2.0, 3.0])
    fn put_var_multi(
        &mut self,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
        values: Vec<f64>,
    ) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.put_var_multi(step, var_type.to_rust(), entity_id, &values)
                .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Write variable across multiple time steps (accepts NumPy arrays or lists)
    ///
    /// Args:
    ///     start_step: Starting time step index (0-based)
    ///     end_step: Ending time step index (exclusive)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (block ID for block variables, 0 for global/nodal)
    ///     var_index: Variable index (0-based)
    ///     values: Variable values for all time steps as NumPy array or list
    ///
    /// Example:
    ///     >>> import numpy as np
    ///     >>> # Write 5 time steps of a global variable
    ///     >>> writer.put_var_time_series(0, 5, EntityType.GLOBAL, 0, 0,
    ///     ...     np.array([10.0, 9.0, 8.0, 7.0, 6.0]))
    #[cfg(feature = "numpy")]
    fn put_var_time_series(
        &mut self,
        _py: Python<'_>,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
        values: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Convert NumPy array or list to Vec
        let values_vec = if let Ok(arr) = values.clone().cast_into::<PyArray1<f64>>() {
            arr.readonly().as_slice()?.to_vec()
        } else if let Ok(arr) = values.clone().cast_into::<PyArray2<f64>>() {
            // Flatten 2D array to 1D
            arr.readonly().as_slice()?.to_vec()
        } else {
            values.extract::<Vec<f64>>()?
        };

        if let Some(ref mut file) = self.file {
            file.put_var_time_series(
                start_step,
                end_step,
                var_type.to_rust(),
                entity_id,
                var_index,
                &values_vec,
            )
            .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Write variable across multiple time steps (no NumPy)
    #[cfg(not(feature = "numpy"))]
    fn put_var_time_series(
        &mut self,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: i64,
        var_index: usize,
        values: Vec<f64>,
    ) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.put_var_time_series(
                start_step,
                end_step,
                var_type.to_rust(),
                entity_id,
                var_index,
                &values,
            )
            .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Set truth table (which blocks have which variables)
    ///
    /// Args:
    ///     var_type: Entity type (must be a block type)
    ///     table: TruthTable object
    ///
    /// Example:
    ///     >>> table = TruthTable.new(EntityType.ELEM_BLOCK, 2, 2)
    ///     >>> table.set(1, 1, False)  # Block 2 doesn't have variable 2
    ///     >>> writer.put_truth_table(EntityType.ELEM_BLOCK, table)
    fn put_truth_table(&mut self, var_type: EntityType, table: &TruthTable) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.put_truth_table(var_type.to_rust(), &table.to_rust())
                .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Define reduction variables for an entity type
    ///
    /// Reduction variables store aggregated/summary values for entire objects
    /// (e.g., assemblies, blocks, sets) rather than for individual entities within those objects.
    ///
    /// Args:
    ///     var_type: Entity type (e.g., EntityType.ASSEMBLY, EntityType.ELEM_BLOCK)
    ///     names: List of variable names
    ///
    /// Example:
    ///     >>> writer.define_reduction_variables(EntityType.ASSEMBLY,
    ///     ...     ["Momentum_X", "Momentum_Y", "Kinetic_Energy"])
    fn define_reduction_variables(
        &mut self,
        var_type: EntityType,
        names: Vec<String>,
    ) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            file.define_reduction_variables(var_type.to_rust(), &name_refs)
                .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Write reduction variable values for a time step
    ///
    /// Reduction variables store aggregate values for entire objects (assemblies, blocks, sets)
    /// rather than for individual entities within those objects.
    ///
    /// Args:
    ///     step: Time step index (0-based)
    ///     var_type: Entity type
    ///     entity_id: Entity ID (e.g., assembly ID, block ID, set ID)
    ///     values: Variable values (one per reduction variable)
    ///
    /// Example:
    ///     >>> writer.put_reduction_vars(0, EntityType.ASSEMBLY, 100,
    ///     ...     [1.5, 2.3, 45.6])  # Momentum_X, Momentum_Y, Kinetic_Energy
    fn put_reduction_vars(
        &mut self,
        step: usize,
        var_type: EntityType,
        entity_id: i64,
        values: Vec<f64>,
    ) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.put_reduction_vars(step, var_type.to_rust(), entity_id, &values)
                .into_py()
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }
}
