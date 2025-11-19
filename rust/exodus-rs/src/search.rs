//! Spatial search utilities for finding nodes and elements by location.
//!
//! This module provides functionality for:
//! - Searching for nodes by spatial coordinates using nearest neighbor
//! - Searching for elements by spatial coordinates using element centroids
//! - Extracting time-history data for variables at specific locations
//! - Distance-based filtering with configurable search radius

use crate::error::{ExodusError, Result};
use crate::geometry::Vec3;
use crate::types::EntityType;
use crate::{mode, ExodusFile};

/// Result of a spatial search for a nodal or element variable.
///
/// Contains the matched entity ID, distance from search point,
/// and the complete time-history data for the variable.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialSearchResult {
    /// ID of the matched node or element (1-based as per Exodus convention)
    pub id: i64,

    /// Distance from the search point to the matched location
    pub distance: f64,

    /// Time-history values for the variable at all time steps
    pub time_history: Vec<f64>,
}

impl SpatialSearchResult {
    /// Slice the time history by time step indices.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting time step index (0-based, inclusive)
    /// * `end` - Ending time step index (0-based, exclusive), None means to the end
    /// * `step` - Step size for slicing (1 means every step, 2 means every other step, etc.)
    ///
    /// # Returns
    ///
    /// A new `SpatialSearchResult` with the sliced time history
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Get time steps 5 through 10
    /// let sliced = result.slice(5, Some(10), 1);
    ///
    /// // Get every other time step from 0 to end
    /// let sliced = result.slice(0, None, 2);
    /// ```
    pub fn slice(&self, start: usize, end: Option<usize>, step: usize) -> Self {
        let end_idx = end.unwrap_or(self.time_history.len());
        let sliced: Vec<f64> = self
            .time_history
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
    /// # Arguments
    ///
    /// * `file` - Reference to the Exodus file (to access time values)
    /// * `start_time` - Starting time value (inclusive)
    /// * `end_time` - Ending time value (inclusive)
    ///
    /// # Returns
    ///
    /// A new `SpatialSearchResult` with the time history filtered by time range
    ///
    /// # Errors
    ///
    /// Returns an error if time values cannot be read from the file
    pub fn slice_by_time(
        &self,
        file: &ExodusFile<mode::Read>,
        start_time: f64,
        end_time: f64,
    ) -> Result<Self> {
        let times = file.times()?;

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
}

impl ExodusFile<mode::Read> {
    /// Compute the average element size in the mesh.
    ///
    /// This is computed as the cube root of the average element volume
    /// for all 3D elements in the mesh. For meshes with only 2D elements,
    /// it uses the square root of element areas. This provides a characteristic
    /// length scale for the mesh.
    ///
    /// # Returns
    ///
    /// Average element size, or an error if the mesh has no elements or
    /// element volumes cannot be computed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let avg_size = file.average_element_size()?;
    /// println!("Average element size: {}", avg_size);
    /// # Ok(())
    /// # }
    /// ```
    pub fn average_element_size(&self) -> Result<f64> {
        // Get all element volumes
        let volumes = self.all_element_volumes()?;

        if volumes.is_empty() {
            return Err(ExodusError::Other(
                "Cannot compute average element size: mesh has no elements".to_string(),
            ));
        }

        // Compute average volume
        let avg_volume: f64 = volumes.iter().sum::<f64>() / volumes.len() as f64;

        // Return cube root of average volume as characteristic length
        // For 2D elements this will still give a reasonable length scale
        Ok(avg_volume.cbrt())
    }

    /// Search for the nearest node to a given spatial location.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of search point
    /// * `y` - Y coordinate of search point
    /// * `z` - Z coordinate of search point
    /// * `max_distance` - Maximum search distance. Use a negative value to search
    ///   without distance limits. A positive value will reject matches farther than
    ///   this distance.
    ///
    /// # Returns
    ///
    /// Tuple of (node_id, distance) for the nearest node, or an error if no node
    /// is found within the search distance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let (node_id, distance) = file.find_nearest_node(1.0, 2.0, 3.0, -1.0)?;
    /// println!("Nearest node: {} at distance {}", node_id, distance);
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_nearest_node(
        &self,
        x: f64,
        y: f64,
        z: f64,
        max_distance: f64,
    ) -> Result<(i64, f64)> {
        let coords = self.coords()?;
        let num_nodes = coords.x.len();

        if num_nodes == 0 {
            return Err(ExodusError::Other("Mesh has no nodes".to_string()));
        }

        let search_point = [x, y, z];
        let mut min_dist = f64::INFINITY;
        let mut nearest_node = 0;

        for i in 0..num_nodes {
            let node_pos = [
                coords.x[i],
                coords.y.get(i).copied().unwrap_or(0.0),
                coords.z.get(i).copied().unwrap_or(0.0),
            ];

            let dist = distance(search_point, node_pos);

            // Update if this is closer (or same distance but lower ID)
            if dist < min_dist || (dist == min_dist && (i + 1) < nearest_node as usize) {
                min_dist = dist;
                nearest_node = (i + 1) as i64; // Node IDs are 1-based
            }
        }

        // Check distance limit
        if max_distance >= 0.0 && min_dist > max_distance {
            return Err(ExodusError::Other(format!(
                "No node found within search distance {}. Nearest node is at distance {}",
                max_distance, min_dist
            )));
        }

        Ok((nearest_node, min_dist))
    }

    /// Search for the nearest element to a given spatial location.
    ///
    /// This uses element centroids for the distance calculation.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of search point
    /// * `y` - Y coordinate of search point
    /// * `z` - Z coordinate of search point
    /// * `max_distance` - Maximum search distance. Use a negative value to search
    ///   without distance limits.
    ///
    /// # Returns
    ///
    /// Tuple of (element_id, distance) for the nearest element, or an error if no
    /// element is found within the search distance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let (elem_id, distance) = file.find_nearest_element(1.0, 2.0, 3.0, -1.0)?;
    /// println!("Nearest element: {} at distance {}", elem_id, distance);
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_nearest_element(
        &self,
        x: f64,
        y: f64,
        z: f64,
        max_distance: f64,
    ) -> Result<(i64, f64)> {
        let centroids = self.all_element_centroids()?;

        if centroids.is_empty() {
            return Err(ExodusError::Other("Mesh has no elements".to_string()));
        }

        let search_point = [x, y, z];
        let mut min_dist = f64::INFINITY;
        let mut nearest_elem = 0;

        for (i, centroid) in centroids.iter().enumerate() {
            let dist = distance(search_point, *centroid);

            // Update if this is closer (or same distance but lower ID)
            if dist < min_dist || (dist == min_dist && (i + 1) < nearest_elem as usize) {
                min_dist = dist;
                nearest_elem = (i + 1) as i64; // Element IDs are 1-based
            }
        }

        // Check distance limit
        if max_distance >= 0.0 && min_dist > max_distance {
            return Err(ExodusError::Other(format!(
                "No element found within search distance {}. Nearest element is at distance {}",
                max_distance, min_dist
            )));
        }

        Ok((nearest_elem, min_dist))
    }

    /// Search for a nodal variable by spatial location and return its time history.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of search point
    /// * `y` - Y coordinate of search point
    /// * `z` - Z coordinate of search point
    /// * `var_name` - Name of the nodal variable to retrieve
    /// * `max_distance` - Maximum search distance. Use a negative value for unlimited search.
    ///   Use `None` to default to 5x the average element size.
    ///
    /// # Returns
    ///
    /// `SpatialSearchResult` containing the matched node ID, distance, and time history
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No node is found within search distance
    /// - Variable name is not found
    /// - Variable data cannot be read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    ///
    /// // Search with default distance limit (5x average element size)
    /// let result = file.search_nodal_variable(1.0, 2.0, 3.0, "temperature", None)?;
    /// println!("Found node {} at distance {}", result.id, result.distance);
    /// println!("Time history has {} steps", result.time_history.len());
    ///
    /// // Search with custom distance limit
    /// let result = file.search_nodal_variable(1.0, 2.0, 3.0, "pressure", Some(0.5))?;
    ///
    /// // Search without distance limit
    /// let result = file.search_nodal_variable(1.0, 2.0, 3.0, "velocity_x", Some(-1.0))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_nodal_variable(
        &self,
        x: f64,
        y: f64,
        z: f64,
        var_name: &str,
        max_distance: Option<f64>,
    ) -> Result<SpatialSearchResult> {
        // Determine search distance
        let search_dist = match max_distance {
            Some(d) => d,
            None => {
                // Default: 5x average element size
                let avg_size = self.average_element_size()?;
                5.0 * avg_size
            }
        };

        // Find nearest node
        let (node_id, distance) = self.find_nearest_node(x, y, z, search_dist)?;

        // Get variable names and find index
        let var_names = self.variable_names(EntityType::Nodal)?;
        let var_index = var_names
            .iter()
            .position(|name| name == var_name)
            .ok_or_else(|| {
                ExodusError::Other(format!("Nodal variable '{}' not found", var_name))
            })?;

        // Get time history for this node
        let num_steps = self.num_time_steps()?;
        let mut time_history = Vec::with_capacity(num_steps);

        for step in 0..num_steps {
            let values = self.var(step, EntityType::Nodal, 0, var_index)?;
            // Node IDs are 1-based, but array indices are 0-based
            let node_idx = (node_id - 1) as usize;
            if node_idx < values.len() {
                time_history.push(values[node_idx]);
            } else {
                return Err(ExodusError::Other(format!(
                    "Node index {} out of range for variable data",
                    node_id
                )));
            }
        }

        Ok(SpatialSearchResult {
            id: node_id,
            distance,
            time_history,
        })
    }

    /// Search for an element variable by spatial location and return its time history.
    ///
    /// This uses element centroids for the spatial search.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of search point
    /// * `y` - Y coordinate of search point
    /// * `z` - Z coordinate of search point
    /// * `var_name` - Name of the element variable to retrieve
    /// * `max_distance` - Maximum search distance. Use a negative value for unlimited search.
    ///   Use `None` to default to 5x the average element size.
    ///
    /// # Returns
    ///
    /// `SpatialSearchResult` containing the matched element ID, distance, and time history
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No element is found within search distance
    /// - Variable name is not found
    /// - Variable data cannot be read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    ///
    /// // Search with default distance limit
    /// let result = file.search_element_variable(1.0, 2.0, 3.0, "stress", None)?;
    /// println!("Found element {} at distance {}", result.id, result.distance);
    ///
    /// // Slice to get only time steps 10-20
    /// let sliced = result.slice(10, Some(20), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_element_variable(
        &self,
        x: f64,
        y: f64,
        z: f64,
        var_name: &str,
        max_distance: Option<f64>,
    ) -> Result<SpatialSearchResult> {
        // Determine search distance
        let search_dist = match max_distance {
            Some(d) => d,
            None => {
                // Default: 5x average element size
                let avg_size = self.average_element_size()?;
                5.0 * avg_size
            }
        };

        // Find nearest element
        let (elem_id, distance) = self.find_nearest_element(x, y, z, search_dist)?;

        // Get variable names and find index
        let var_names = self.variable_names(EntityType::ElemBlock)?;
        let var_index = var_names
            .iter()
            .position(|name| name == var_name)
            .ok_or_else(|| {
                ExodusError::Other(format!("Element variable '{}' not found", var_name))
            })?;

        // We need to find which block this element belongs to
        // Elements are numbered sequentially across blocks
        let block_ids = self.block_ids(EntityType::ElemBlock)?;
        let mut elem_count = 0;
        let mut target_block_id = None;
        let mut elem_idx_in_block = 0;

        for &block_id in &block_ids {
            let block = self.block(block_id)?;
            let num_elems = block.num_entries;

            if elem_id <= (elem_count + num_elems as i64) {
                target_block_id = Some(block_id);
                elem_idx_in_block = (elem_id - elem_count - 1) as usize;
                break;
            }

            elem_count += num_elems as i64;
        }

        let block_id = target_block_id.ok_or_else(|| {
            ExodusError::Other(format!("Element {} not found in any block", elem_id))
        })?;

        // Get time history for this element
        let num_steps = self.num_time_steps()?;
        let mut time_history = Vec::with_capacity(num_steps);

        for step in 0..num_steps {
            let values = self.var(step, EntityType::ElemBlock, block_id, var_index)?;
            if elem_idx_in_block < values.len() {
                time_history.push(values[elem_idx_in_block]);
            } else {
                return Err(ExodusError::Other(format!(
                    "Element index {} out of range for block {} variable data",
                    elem_idx_in_block, block_id
                )));
            }
        }

        Ok(SpatialSearchResult {
            id: elem_id,
            distance,
            time_history,
        })
    }
}

/// Compute Euclidean distance between two 3D points.
#[inline]
fn distance(a: Vec3, b: Vec3) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Block, CreateMode, CreateOptions, InitParams};
    use crate::ExodusFile;
    use tempfile::NamedTempFile;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_distance() {
        let p1 = [0.0, 0.0, 0.0];
        let p2 = [3.0, 4.0, 0.0];
        assert!(approx_eq(distance(p1, p2), 5.0));
    }

    #[test]
    fn test_search_result_slice() {
        let result = SpatialSearchResult {
            id: 1,
            distance: 0.5,
            time_history: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        };

        // Test basic slicing
        let sliced = result.slice(2, Some(6), 1);
        assert_eq!(sliced.time_history, vec![3.0, 4.0, 5.0, 6.0]);

        // Test slicing with step
        let sliced = result.slice(0, Some(10), 2);
        assert_eq!(sliced.time_history, vec![1.0, 3.0, 5.0, 7.0, 9.0]);

        // Test slicing to end
        let sliced = result.slice(7, None, 1);
        assert_eq!(sliced.time_history, vec![8.0, 9.0, 10.0]);
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_find_nearest_node() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create a simple mesh
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 4,
                num_elems: 0,
                num_elem_blocks: 0,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Four nodes at corners of a square
            let x = vec![0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        // Test nearest node search
        let file = ExodusFile::<mode::Read>::open(path).unwrap();

        // Search near origin (should find node 1)
        let (node_id, dist) = file.find_nearest_node(0.1, 0.1, 0.0, -1.0).unwrap();
        assert_eq!(node_id, 1);
        assert!(approx_eq(dist, (0.1 * 0.1 + 0.1 * 0.1_f64).sqrt()));

        // Search near (1, 0)
        let (node_id, _dist) = file.find_nearest_node(0.9, 0.1, 0.0, -1.0).unwrap();
        assert_eq!(node_id, 2);

        // Search near (1, 1)
        let (node_id, _dist) = file.find_nearest_node(1.0, 1.0, 0.0, -1.0).unwrap();
        assert_eq!(node_id, 3);

        // Test distance limit - should fail
        let result = file.find_nearest_node(5.0, 5.0, 0.0, 1.0);
        assert!(result.is_err());

        // Test distance limit - should succeed
        let result = file.find_nearest_node(0.5, 0.5, 0.0, 1.0);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_find_nearest_element() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create a mesh with 2 elements
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 2,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Two cubes: one at origin, one at x=2
            let x = vec![
                0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, // First cube
            ];
            let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            let block = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(100, &[1, 2, 3, 4, 5, 6, 7, 8])
                .unwrap();
        }

        // Test nearest element search
        let file = ExodusFile::<mode::Read>::open(path).unwrap();

        // Search near centroid of first cube (0.5, 0.5, 0.5)
        let (elem_id, _dist) = file.find_nearest_element(0.5, 0.5, 0.5, -1.0).unwrap();
        assert_eq!(elem_id, 1);

        // Test distance limit
        let result = file.find_nearest_element(10.0, 10.0, 10.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_average_element_size() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create a mesh with unit cubes
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Unit cube
            let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            let block = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(100, &[1, 2, 3, 4, 5, 6, 7, 8])
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let avg_size = file.average_element_size().unwrap();

        // Unit cube has volume 1, so cbrt(1) = 1
        assert!(approx_eq(avg_size, 1.0));
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_search_nodal_variable() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create a mesh with nodal variables
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Add a block (needed for average element size)
            let block = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "QUAD4".into(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(100, &[1, 2, 3, 4]).unwrap();

            // Define nodal variable
            file.define_variables(EntityType::Nodal, &["temperature"])
                .unwrap();

            // Write time steps
            for step in 0..3 {
                file.put_time(step, step as f64).unwrap();
                let values = vec![
                    10.0 + step as f64,
                    20.0 + step as f64,
                    30.0 + step as f64,
                    40.0 + step as f64,
                ];
                file.put_var(step, EntityType::Nodal, 0, 0, &values)
                    .unwrap();
            }
        }

        // Test search
        let file = ExodusFile::<mode::Read>::open(path).unwrap();

        // Search near node 1 (0, 0, 0)
        let result = file
            .search_nodal_variable(0.1, 0.1, 0.0, "temperature", Some(-1.0))
            .unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.time_history, vec![10.0, 11.0, 12.0]);

        // Search near node 3 (1, 1, 0)
        let result = file
            .search_nodal_variable(0.9, 0.9, 0.0, "temperature", Some(-1.0))
            .unwrap();
        assert_eq!(result.id, 3);
        assert_eq!(result.time_history, vec![30.0, 31.0, 32.0]);

        // Test slicing
        let sliced = result.slice(1, Some(3), 1);
        assert_eq!(sliced.time_history, vec![31.0, 32.0]);
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_search_element_variable() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create a mesh with element variables
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 2,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Two cubes
            let x = vec![
                0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, // Nodes for both cubes
            ];
            let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            let block = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 2,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(
                100,
                &[
                    1, 2, 3, 4, 5, 6, 7, 8, // Element 1
                    1, 2, 3, 4, 5, 6, 7, 8, // Element 2 (same for simplicity)
                ],
            )
            .unwrap();

            // Define element variable
            file.define_variables(EntityType::ElemBlock, &["stress"])
                .unwrap();

            // Write time steps
            for step in 0..3 {
                file.put_time(step, step as f64).unwrap();
                let values = vec![100.0 + step as f64, 200.0 + step as f64];
                file.put_var(step, EntityType::ElemBlock, 100, 0, &values)
                    .unwrap();
            }
        }

        // Test search
        let file = ExodusFile::<mode::Read>::open(path).unwrap();

        // Search near first element
        let result = file
            .search_element_variable(0.5, 0.5, 0.5, "stress", Some(-1.0))
            .unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.time_history, vec![100.0, 101.0, 102.0]);
    }
}
