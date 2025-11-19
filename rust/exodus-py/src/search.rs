//! Spatial search utilities for Python

use pyo3::prelude::*;
use exodus_rs::search::SpatialSearchResult as RustSpatialSearchResult;

use crate::error::IntoPyResult;
use crate::file::ExodusReader;

/// Result of a spatial search for a nodal or element variable.
///
/// Contains the matched entity ID, distance from search point,
/// and the complete time-history data for the variable.
#[pyclass]
#[derive(Debug, Clone)]
pub struct SpatialSearchResult {
    /// ID of the matched node or element (1-based as per Exodus convention)
    #[pyo3(get)]
    pub id: i64,

    /// Distance from the search point to the matched location
    #[pyo3(get)]
    pub distance: f64,

    /// Time-history values for the variable at all time steps
    #[pyo3(get)]
    pub time_history: Vec<f64>,
}

impl From<RustSpatialSearchResult> for SpatialSearchResult {
    fn from(rust_result: RustSpatialSearchResult) -> Self {
        Self {
            id: rust_result.id,
            distance: rust_result.distance,
            time_history: rust_result.time_history,
        }
    }
}

#[pymethods]
impl SpatialSearchResult {
    /// Slice the time history by time step indices.
    ///
    /// Args:
    ///     start: Starting time step index (0-based, inclusive)
    ///     end: Ending time step index (0-based, exclusive), None means to the end
    ///     step: Step size for slicing (1 means every step, 2 means every other step, etc.)
    ///
    /// Returns:
    ///     SpatialSearchResult: A new result with the sliced time history
    ///
    /// Example:
    ///     >>> # Get time steps 5 through 10
    ///     >>> sliced = result.slice(5, 10, 1)
    ///     >>>
    ///     >>> # Get every other time step from 0 to end
    ///     >>> sliced = result.slice(0, None, 2)
    fn slice(&self, start: usize, end: Option<usize>, step: usize) -> Self {
        let end_idx = end.unwrap_or(self.time_history.len());
        let sliced: Vec<f64> = self.time_history
            .iter()
            .skip(start)
            .take(end_idx.saturating_sub(start))
            .step_by(step.max(1))
            .copied()
            .collect();

        Self {
            id: self.id,
            distance: self.distance,
            time_history: sliced,
        }
    }

    /// Slice the time history by actual time values.
    ///
    /// Args:
    ///     reader: ExodusReader instance to access time values
    ///     start_time: Starting time value (inclusive)
    ///     end_time: Ending time value (inclusive)
    ///
    /// Returns:
    ///     SpatialSearchResult: A new result with the time history filtered by time range
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> result = reader.search_nodal_variable(1.0, 2.0, 3.0, "temperature")
    ///     >>> # Get data from time 0.5 to 1.5
    ///     >>> sliced = result.slice_by_time(reader, 0.5, 1.5)
    fn slice_by_time(
        &self,
        reader: &ExodusReader,
        start_time: f64,
        end_time: f64,
    ) -> PyResult<Self> {
        let times = reader.file.times().into_py()?;

        let mut sliced = Vec::new();
        for (i, &time) in times.iter().enumerate() {
            if time >= start_time && time <= end_time && i < self.time_history.len() {
                sliced.push(self.time_history[i]);
            }
        }

        Ok(Self {
            id: self.id,
            distance: self.distance,
            time_history: sliced,
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "SpatialSearchResult(id={}, distance={:.6}, time_steps={})",
            self.id,
            self.distance,
            self.time_history.len()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

// Add spatial search methods to ExodusReader
#[pymethods]
impl ExodusReader {
    /// Compute the average element size in the mesh.
    ///
    /// This is computed as the cube root of the average element volume
    /// for all 3D elements in the mesh.
    ///
    /// Returns:
    ///     float: Average element size
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> avg_size = reader.average_element_size()
    ///     >>> print(f"Average element size: {avg_size}")
    fn average_element_size(&self) -> PyResult<f64> {
        self.file.average_element_size().into_py()
    }

    /// Search for the nearest node to a given spatial location.
    ///
    /// Args:
    ///     x: X coordinate of search point
    ///     y: Y coordinate of search point
    ///     z: Z coordinate of search point
    ///     max_distance: Maximum search distance. Use a negative value to search
    ///         without distance limits.
    ///
    /// Returns:
    ///     tuple: (node_id, distance) for the nearest node
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> node_id, distance = reader.find_nearest_node(1.0, 2.0, 3.0, -1.0)
    ///     >>> print(f"Nearest node: {node_id} at distance {distance}")
    fn find_nearest_node(
        &self,
        x: f64,
        y: f64,
        z: f64,
        max_distance: f64,
    ) -> PyResult<(i64, f64)> {
        self.file.find_nearest_node(x, y, z, max_distance).into_py()
    }

    /// Search for the nearest element to a given spatial location.
    ///
    /// This uses element centroids for the distance calculation.
    ///
    /// Args:
    ///     x: X coordinate of search point
    ///     y: Y coordinate of search point
    ///     z: Z coordinate of search point
    ///     max_distance: Maximum search distance. Use a negative value to search
    ///         without distance limits.
    ///
    /// Returns:
    ///     tuple: (element_id, distance) for the nearest element
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> elem_id, distance = reader.find_nearest_element(1.0, 2.0, 3.0, -1.0)
    ///     >>> print(f"Nearest element: {elem_id} at distance {distance}")
    fn find_nearest_element(
        &self,
        x: f64,
        y: f64,
        z: f64,
        max_distance: f64,
    ) -> PyResult<(i64, f64)> {
        self.file.find_nearest_element(x, y, z, max_distance).into_py()
    }

    /// Search for a nodal variable by spatial location and return its time history.
    ///
    /// Args:
    ///     x: X coordinate of search point
    ///     y: Y coordinate of search point
    ///     z: Z coordinate of search point
    ///     var_name: Name of the nodal variable to retrieve
    ///     max_distance: Maximum search distance. Use None for default (5x average element size),
    ///         or a negative value for unlimited search.
    ///
    /// Returns:
    ///     SpatialSearchResult: Result containing matched node ID, distance, and time history
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>>
    ///     >>> # Search with default distance limit (5x average element size)
    ///     >>> result = reader.search_nodal_variable(1.0, 2.0, 3.0, "temperature")
    ///     >>> print(f"Found node {result.id} at distance {result.distance}")
    ///     >>> print(f"Time history has {len(result.time_history)} steps")
    ///     >>>
    ///     >>> # Search with custom distance limit
    ///     >>> result = reader.search_nodal_variable(1.0, 2.0, 3.0, "pressure", 0.5)
    ///     >>>
    ///     >>> # Search without distance limit
    ///     >>> result = reader.search_nodal_variable(1.0, 2.0, 3.0, "velocity_x", -1.0)
    ///     >>>
    ///     >>> # Slice to get only time steps 10-20
    ///     >>> sliced = result.slice(10, 20, 1)
    fn search_nodal_variable(
        &self,
        x: f64,
        y: f64,
        z: f64,
        var_name: &str,
        max_distance: Option<f64>,
    ) -> PyResult<SpatialSearchResult> {
        let result = self.file.search_nodal_variable(x, y, z, var_name, max_distance).into_py()?;
        Ok(result.into())
    }

    /// Search for an element variable by spatial location and return its time history.
    ///
    /// This uses element centroids for the spatial search.
    ///
    /// Args:
    ///     x: X coordinate of search point
    ///     y: Y coordinate of search point
    ///     z: Z coordinate of search point
    ///     var_name: Name of the element variable to retrieve
    ///     max_distance: Maximum search distance. Use None for default (5x average element size),
    ///         or a negative value for unlimited search.
    ///
    /// Returns:
    ///     SpatialSearchResult: Result containing matched element ID, distance, and time history
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>>
    ///     >>> # Search with default distance limit
    ///     >>> result = reader.search_element_variable(1.0, 2.0, 3.0, "stress")
    ///     >>> print(f"Found element {result.id} at distance {result.distance}")
    ///     >>>
    ///     >>> # Slice to get only time steps 10-20
    ///     >>> sliced = result.slice(10, 20, 1)
    ///     >>>
    ///     >>> # Slice by time values
    ///     >>> time_sliced = result.slice_by_time(reader, 0.5, 1.5)
    fn search_element_variable(
        &self,
        x: f64,
        y: f64,
        z: f64,
        var_name: &str,
        max_distance: Option<f64>,
    ) -> PyResult<SpatialSearchResult> {
        let result = self.file.search_element_variable(x, y, z, var_name, max_distance).into_py()?;
        Ok(result.into())
    }
}
