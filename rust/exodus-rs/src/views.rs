//! View types for zero-copy data access with ndarray integration.
//!
//! This module provides view types that enable zero-copy access to Exodus data
//! for efficient NumPy integration via PyO3. These views are compatible with
//! NumPy's memory layout (C-contiguous by default) and minimize memory overhead
//! when working with large files.
//!
//! # Features
//!
//! This module is only available when the `ndarray` feature is enabled.
//!
//! # Examples
//!
//! ```rust,ignore
//! use exodus_rs::ExodusFile;
//! use exodus_rs::mode::Read;
//!
//! let file = ExodusFile::<Read>::open("mesh.exo")?;
//!
//! // Get coordinates as ndarray view (zero-copy)
//! let coords_view = file.coords_view()?;
//! println!("X coordinates: {:?}", coords_view.x);
//!
//! // Get coordinates as owned 2D array (N, 3)
//! let coords_array = file.coords_array()?;
//! println!("Coordinates shape: {:?}", coords_array.shape());
//! # Ok::<(), exodus_rs::ExodusError>(())
//! ```

#[cfg(feature = "ndarray")]
use ndarray::{ArrayView1, ArrayView2};

#[cfg(feature = "ndarray")]
use crate::coord::CoordValue;

/// Read-only view into coordinate data.
///
/// Provides zero-copy access to coordinate arrays with lifetime tied to
/// the underlying buffer. Compatible with NumPy's memory layout.
///
/// # Type Parameters
///
/// - `T`: Coordinate value type (typically `f32` or `f64`)
///
/// # Examples
///
/// ```rust,ignore
/// let view = file.coords_view()?;
/// assert_eq!(view.num_dim, 3);
/// println!("First X coordinate: {}", view.x[0]);
/// ```
#[cfg(feature = "ndarray")]
#[derive(Debug)]
pub struct CoordinatesView<'a, T: CoordValue> {
    /// X coordinates (borrowed)
    pub x: ArrayView1<'a, T>,
    /// Y coordinates (borrowed, may be empty for 1D)
    pub y: ArrayView1<'a, T>,
    /// Z coordinates (borrowed, may be empty for 1D/2D)
    pub z: ArrayView1<'a, T>,
    /// Number of dimensions (1, 2, or 3)
    pub num_dim: usize,
}

#[cfg(feature = "ndarray")]
impl<'a, T: CoordValue> CoordinatesView<'a, T> {
    /// Create a new coordinates view.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate array view
    /// * `y` - Y coordinate array view
    /// * `z` - Z coordinate array view
    /// * `num_dim` - Number of spatial dimensions
    pub fn new(
        x: ArrayView1<'a, T>,
        y: ArrayView1<'a, T>,
        z: ArrayView1<'a, T>,
        num_dim: usize,
    ) -> Self {
        Self { x, y, z, num_dim }
    }

    /// Get the number of nodes.
    pub fn num_nodes(&self) -> usize {
        self.x.len()
    }
}

/// Read-only view into connectivity data.
///
/// Provides zero-copy access to element connectivity as a 2D array view.
///
/// # Examples
///
/// ```rust,ignore
/// let view = file.connectivity_view(block_id)?;
/// println!("Shape: {:?}", view.data.shape());  // (num_elements, nodes_per_element)
/// ```
#[cfg(feature = "ndarray")]
#[derive(Debug)]
pub struct ConnectivityView<'a> {
    /// Connectivity matrix: (num_entries, nodes_per_entry)
    pub data: ArrayView2<'a, i64>,
}

#[cfg(feature = "ndarray")]
impl<'a> ConnectivityView<'a> {
    /// Create a new connectivity view.
    ///
    /// # Arguments
    ///
    /// * `data` - 2D array view of connectivity data
    pub fn new(data: ArrayView2<'a, i64>) -> Self {
        Self { data }
    }

    /// Get the number of entries (elements/edges/faces).
    pub fn num_entries(&self) -> usize {
        self.data.nrows()
    }

    /// Get the number of nodes per entry.
    pub fn nodes_per_entry(&self) -> usize {
        self.data.ncols()
    }
}

/// Read-only view into 1D variable data.
///
/// Provides zero-copy access to variable values for a single time step.
///
/// # Examples
///
/// ```rust,ignore
/// let view = file.var_view(step, var_type, id, var_idx)?;
/// println!("Variable values: {:?}", view.data);
/// ```
#[cfg(feature = "ndarray")]
#[derive(Debug)]
pub struct VarView<'a> {
    /// Variable data (borrowed)
    pub data: ArrayView1<'a, f64>,
}

#[cfg(feature = "ndarray")]
impl<'a> VarView<'a> {
    /// Create a new variable view.
    ///
    /// # Arguments
    ///
    /// * `data` - 1D array view of variable data
    pub fn new(data: ArrayView1<'a, f64>) -> Self {
        Self { data }
    }

    /// Get the number of values.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the view is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Read-only view into time series data.
///
/// Provides zero-copy access to variable values across all time steps as a 2D array.
///
/// # Shape
///
/// The array has shape `(num_time_steps, num_entities)` for efficient time-based access.
///
/// # Examples
///
/// ```rust,ignore
/// let view = file.var_time_series_view(var_type, id, var_idx)?;
/// println!("Time series shape: {:?}", view.data.shape());
///
/// // Access specific time step
/// let step_0 = view.data.row(0);
///
/// // Access specific entity history
/// let entity_5 = view.data.column(5);
/// ```
#[cfg(feature = "ndarray")]
#[derive(Debug)]
pub struct VarTimeSeriesView<'a> {
    /// Time series data: (num_time_steps, num_entities)
    pub data: ArrayView2<'a, f64>,
}

#[cfg(feature = "ndarray")]
impl<'a> VarTimeSeriesView<'a> {
    /// Create a new time series view.
    ///
    /// # Arguments
    ///
    /// * `data` - 2D array view of time series data
    pub fn new(data: ArrayView2<'a, f64>) -> Self {
        Self { data }
    }

    /// Get the number of time steps.
    pub fn num_time_steps(&self) -> usize {
        self.data.nrows()
    }

    /// Get the number of entities.
    pub fn num_entities(&self) -> usize {
        self.data.ncols()
    }
}

// Note: Mutable view variants can be added in the future if needed for write operations:
// - CoordinatesViewMut
// - ConnectivityViewMut
// - VarViewMut
// These would use ArrayViewMut1/ArrayViewMut2 for mutable borrows.

#[cfg(all(test, feature = "ndarray"))]
mod tests {
    use super::*;
    use ndarray::{Array1, Array2};

    #[test]
    fn test_coordinates_view() {
        let x_data = Array1::from(vec![0.0, 1.0, 2.0]);
        let y_data = Array1::from(vec![0.0, 0.0, 0.0]);
        let z_data = Array1::from(vec![0.0, 0.0, 0.0]);

        let view = CoordinatesView::new(x_data.view(), y_data.view(), z_data.view(), 3);

        assert_eq!(view.num_nodes(), 3);
        assert_eq!(view.num_dim, 3);
        assert_eq!(view.x[0], 0.0);
        assert_eq!(view.x[1], 1.0);
    }

    #[test]
    fn test_var_view() {
        let data = Array1::from(vec![1.0, 2.0, 3.0, 4.0]);
        let view = VarView::new(data.view());

        assert_eq!(view.len(), 4);
        assert!(!view.is_empty());
        assert_eq!(view.data[0], 1.0);
    }

    #[test]
    fn test_connectivity_view() {
        let data = Array2::from_shape_vec((2, 4), vec![1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        let view = ConnectivityView::new(data.view());

        assert_eq!(view.num_entries(), 2);
        assert_eq!(view.nodes_per_entry(), 4);
    }

    #[test]
    fn test_var_time_series_view() {
        // 3 time steps, 5 entities
        let data = Array2::from_shape_vec(
            (3, 5),
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            ],
        )
        .unwrap();
        let view = VarTimeSeriesView::new(data.view());

        assert_eq!(view.num_time_steps(), 3);
        assert_eq!(view.num_entities(), 5);
        assert_eq!(view.data[[0, 0]], 1.0);
        assert_eq!(view.data[[2, 4]], 15.0);
    }
}
