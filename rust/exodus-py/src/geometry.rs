//! Geometry utilities for Python

use exodus_rs::geometry::{
    element_centroid as rust_element_centroid, element_volume as rust_element_volume, Vec3,
};
use exodus_rs::Topology;
use pyo3::prelude::*;

use crate::error::IntoPyResult;

/// Compute the volume of an element based on its topology and node coordinates.
///
/// Supports HEX8, TET4, WEDGE6, and PYRAMID5 elements.
/// Higher-order elements use the same corner nodes as their linear counterparts.
///
/// Args:
///     topology: Element topology (e.g., "HEX8", "TET4", "WEDGE6", "PYRAMID5")
///     coords: List of [x, y, z] coordinates for element nodes
///
/// Returns:
///     float: Element volume
///
/// Raises:
///     RuntimeError: If topology is unsupported or insufficient coordinates provided
///
/// Example:
///     >>> # Unit cube hex
///     >>> coords = [
///     ...     [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
///     ...     [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
///     ... ]
///     >>> volume = element_volume("HEX8", coords)
///     >>> abs(volume - 1.0) < 1e-10
///     True
#[pyfunction]
pub fn element_volume(topology: &str, coords: Vec<Vec<f64>>) -> PyResult<f64> {
    // Parse topology string
    let topo = Topology::from_string(topology);

    // Convert Vec<Vec<f64>> to Vec<Vec3>
    let coords_vec3: Vec<Vec3> = coords
        .iter()
        .map(|c| {
            if c.len() != 3 {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Each coordinate must have 3 values (x, y, z), got {}",
                    c.len()
                )))
            } else {
                Ok([c[0], c[1], c[2]])
            }
        })
        .collect::<PyResult<Vec<Vec3>>>()?;

    // Call Rust function
    rust_element_volume(topo, &coords_vec3).into_py()
}

/// Compute the centroid (geometric center) of an element.
///
/// The centroid is computed as the average of all node positions.
///
/// Args:
///     coords: List of [x, y, z] coordinates for element nodes
///
/// Returns:
///     list: Centroid position as [x, y, z]
///
/// Example:
///     >>> # Unit cube
///     >>> coords = [
///     ...     [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
///     ...     [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
///     ... ]
///     >>> centroid = element_centroid(coords)
///     >>> all(abs(c - 0.5) < 1e-10 for c in centroid)
///     True
#[pyfunction]
pub fn element_centroid(coords: Vec<Vec<f64>>) -> PyResult<Vec<f64>> {
    // Convert Vec<Vec<f64>> to Vec<Vec3>
    let coords_vec3: Vec<Vec3> = coords
        .iter()
        .map(|c| {
            if c.len() != 3 {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Each coordinate must have 3 values (x, y, z), got {}",
                    c.len()
                )))
            } else {
                Ok([c[0], c[1], c[2]])
            }
        })
        .collect::<PyResult<Vec<Vec3>>>()?;

    // Call Rust function
    let centroid = rust_element_centroid(&coords_vec3);
    Ok(vec![centroid[0], centroid[1], centroid[2]])
}

// Add methods to ExodusReader for higher-level geometry calculations
use crate::file::ExodusReader;

#[pymethods]
impl ExodusReader {
    /// Compute volumes for all elements in a block.
    ///
    /// Args:
    ///     block_id: ID of the element block
    ///
    /// Returns:
    ///     list: List of volumes, one per element in the block
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> block_ids = reader.get_block_ids()
    ///     >>> volumes = reader.block_element_volumes(block_ids[0])
    ///     >>> total_volume = sum(volumes)
    fn block_element_volumes(&self, block_id: i64) -> PyResult<Vec<f64>> {
        self.file.block_element_volumes(block_id).into_py()
    }

    /// Compute centroids for all elements in a block.
    ///
    /// Args:
    ///     block_id: ID of the element block
    ///
    /// Returns:
    ///     list: List of centroids as [x, y, z] coordinates, one per element in the block
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> block_ids = reader.get_block_ids()
    ///     >>> centroids = reader.block_element_centroids(block_ids[0])
    ///     >>> print(f"First element centroid: {centroids[0]}")
    fn block_element_centroids(&self, block_id: i64) -> PyResult<Vec<Vec<f64>>> {
        let centroids = self.file.block_element_centroids(block_id).into_py()?;
        Ok(centroids
            .into_iter()
            .map(|c| vec![c[0], c[1], c[2]])
            .collect())
    }

    /// Compute volumes for all elements in the mesh.
    ///
    /// Returns:
    ///     list: List of volumes, one per element in the entire mesh, ordered by element blocks
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> all_volumes = reader.all_element_volumes()
    ///     >>> total_volume = sum(all_volumes)
    ///     >>> print(f"Total mesh volume: {total_volume}")
    fn all_element_volumes(&self) -> PyResult<Vec<f64>> {
        self.file.all_element_volumes().into_py()
    }

    /// Compute centroids for all elements in the mesh.
    ///
    /// Returns:
    ///     list: List of centroids as [x, y, z] coordinates, one per element in the entire mesh,
    ///           ordered by element blocks
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> all_centroids = reader.all_element_centroids()
    ///     >>> print(f"Mesh has {len(all_centroids)} elements")
    fn all_element_centroids(&self) -> PyResult<Vec<Vec<f64>>> {
        let centroids = self.file.all_element_centroids().into_py()?;
        Ok(centroids
            .into_iter()
            .map(|c| vec![c[0], c[1], c[2]])
            .collect())
    }
}

/// Register geometry functions with the Python module
pub fn register_geometry_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(element_volume, m)?)?;
    m.add_function(wrap_pyfunction!(element_centroid, m)?)?;
    Ok(())
}
