//! Coordinate operations
//!
//! This module provides functionality for reading and writing nodal coordinates
//! in Exodus files. Coordinates can be stored as either f32 or f64 values.

use crate::error::{ExodusError, Result};
use crate::{mode, ExodusFile};

/// Trait for coordinate value types
///
/// This trait abstracts over f32 and f64 to allow flexible coordinate storage
/// and conversion between different precision levels.
pub trait CoordValue: Copy + Default + 'static {
    /// Convert from f32
    fn from_f32(v: f32) -> Self;
    /// Convert from f64
    fn from_f64(v: f64) -> Self;
    /// Convert to f64
    fn to_f64(self) -> f64;
}

impl CoordValue for f32 {
    fn from_f32(v: f32) -> Self {
        v
    }
    fn from_f64(v: f64) -> Self {
        v as f32
    }
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl CoordValue for f64 {
    fn from_f32(v: f32) -> Self {
        v as f64
    }
    fn from_f64(v: f64) -> Self {
        v
    }
    fn to_f64(self) -> f64 {
        self
    }
}

/// Container for coordinate data
///
/// Stores x, y, and z coordinates for all nodes in the mesh.
/// The number of dimensions (1, 2, or 3) determines which coordinates are used.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::coord::Coordinates;
///
/// let coords = Coordinates {
///     x: vec![0.0, 1.0, 1.0, 0.0],
///     y: vec![0.0, 0.0, 1.0, 1.0],
///     z: vec![0.0, 0.0, 0.0, 0.0],
///     num_dim: 2,
/// };
///
/// // Get coordinate for node 0
/// let coord = coords.get(0).unwrap();
/// assert_eq!(coord, [0.0, 0.0, 0.0]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Coordinates<T: CoordValue> {
    /// X coordinates for all nodes
    pub x: Vec<T>,
    /// Y coordinates for all nodes
    pub y: Vec<T>,
    /// Z coordinates for all nodes
    pub z: Vec<T>,
    /// Number of spatial dimensions (1, 2, or 3)
    pub num_dim: usize,
}

impl<T: CoordValue> Coordinates<T> {
    /// Get coordinate for node i (0-indexed)
    ///
    /// # Arguments
    ///
    /// * `i` - Node index (0-based)
    ///
    /// # Returns
    ///
    /// Some([x, y, z]) if index is valid, None otherwise
    pub fn get(&self, i: usize) -> Option<[T; 3]> {
        if i >= self.x.len() {
            return None;
        }
        let y_val = self.y.get(i).copied().unwrap_or_default();
        let z_val = self.z.get(i).copied().unwrap_or_default();
        Some([self.x[i], y_val, z_val])
    }

    /// Iterator over coordinates
    ///
    /// # Returns
    ///
    /// An iterator that yields [x, y, z] arrays for each node
    pub fn iter(&self) -> CoordinateIterator<'_, T> {
        CoordinateIterator {
            coords: self,
            index: 0,
        }
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Check if there are no nodes
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }
}

/// Iterator over coordinates
#[derive(Debug)]
pub struct CoordinateIterator<'a, T: CoordValue> {
    coords: &'a Coordinates<T>,
    index: usize,
}

impl<'a, T: CoordValue> Iterator for CoordinateIterator<'a, T> {
    type Item = [T; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.coords.x.len() {
            let result = [
                self.coords.x[self.index],
                self.coords.y[self.index],
                self.coords.z[self.index],
            ];
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

// Writer methods
#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Write all coordinates at once
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinates for all nodes
    /// * `y` - Y coordinates for all nodes (required if num_dim >= 2)
    /// * `z` - Z coordinates for all nodes (required if num_dim >= 3)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The file is not initialized
    /// - Array lengths don't match num_nodes
    /// - Coordinates have already been written
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// file.builder().dimensions(2).nodes(4).finish()?;
    ///
    /// let x = vec![0.0, 1.0, 1.0, 0.0];
    /// let y = vec![0.0, 0.0, 1.0, 1.0];
    /// file.put_coords(&x, Some(&y), None)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn put_coords<T: CoordValue>(
        &mut self,
        x: &[T],
        y: Option<&[T]>,
        z: Option<&[T]>,
    ) -> Result<()> {
        // Ensure we're in data mode for writing coordinate values
        self.ensure_data_mode()?;

        // Validate that file is initialized
        if !self.metadata.initialized {
            return Err(ExodusError::NotInitialized);
        }

        // Get number of nodes (may be 0, in which case dimension doesn't exist)
        let num_nodes = self
            .metadata
            .dim_cache
            .get("num_nodes")
            .copied()
            .unwrap_or(0);
        let num_dim = self.metadata.num_dim.ok_or(ExodusError::Other(
            "num_dim not set in metadata".to_string(),
        ))?;

        // If num_nodes is 0, just return Ok if coordinate arrays are empty
        if num_nodes == 0 {
            if x.is_empty() && y.map_or(true, |v| v.is_empty()) && z.map_or(true, |v| v.is_empty())
            {
                return Ok(());
            } else {
                return Err(ExodusError::InvalidArrayLength {
                    expected: 0,
                    actual: x.len(),
                });
            }
        }

        // Validate array lengths
        if x.len() != num_nodes {
            return Err(ExodusError::InvalidArrayLength {
                expected: num_nodes,
                actual: x.len(),
            });
        }

        // Write X coordinates
        self.put_coord_x(x)?;

        // Write Y coordinates if provided and needed
        if num_dim >= 2 {
            if let Some(y_coords) = y {
                if y_coords.len() != num_nodes {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_nodes,
                        actual: y_coords.len(),
                    });
                }
                self.put_coord_y(y_coords)?;
            } else {
                return Err(ExodusError::Other(
                    "Y coordinates required for num_dim >= 2".to_string(),
                ));
            }
        }

        // Write Z coordinates if provided and needed
        if num_dim >= 3 {
            if let Some(z_coords) = z {
                if z_coords.len() != num_nodes {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_nodes,
                        actual: z_coords.len(),
                    });
                }
                self.put_coord_z(z_coords)?;
            } else {
                return Err(ExodusError::Other(
                    "Z coordinates required for num_dim >= 3".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Write X coordinates
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinates for all nodes
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not initialized or NetCDF write fails
    pub fn put_coord_x<T: CoordValue>(&mut self, x: &[T]) -> Result<()> {
        self.put_coord_dim(0, x)
    }

    /// Write Y coordinates
    ///
    /// # Arguments
    ///
    /// * `y` - Y coordinates for all nodes
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not initialized or NetCDF write fails
    pub fn put_coord_y<T: CoordValue>(&mut self, y: &[T]) -> Result<()> {
        self.put_coord_dim(1, y)
    }

    /// Write Z coordinates
    ///
    /// # Arguments
    ///
    /// * `z` - Z coordinates for all nodes
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not initialized or NetCDF write fails
    pub fn put_coord_z<T: CoordValue>(&mut self, z: &[T]) -> Result<()> {
        self.put_coord_dim(2, z)
    }

    /// Write coordinates for a specific dimension
    fn put_coord_dim<T: CoordValue>(&mut self, dim: usize, coords: &[T]) -> Result<()> {
        let var_name = match dim {
            0 => "coordx",
            1 => "coordy",
            2 => "coordz",
            _ => {
                return Err(ExodusError::InvalidDimension {
                    expected: "0, 1, or 2".to_string(),
                    actual: dim,
                })
            }
        };

        // Get or create the variable
        let mut var = if let Some(var) = self.nc_file.variable_mut(var_name) {
            var
        } else {
            // Variable doesn't exist, we need to create it
            // This should have been done during initialization
            return Err(ExodusError::VariableNotDefined(var_name.to_string()));
        };

        // Convert to f64 for writing
        let data: Vec<f64> = coords.iter().map(|&v| v.to_f64()).collect();

        // Write the data (use .. for full range)
        var.put_values(&data, ..)?;

        Ok(())
    }

    /// Write partial coordinates (for large datasets)
    ///
    /// # Arguments
    ///
    /// * `start` - Starting node index (0-based)
    /// * `count` - Number of nodes to write
    /// * `x` - X coordinates
    /// * `y` - Y coordinates (required if num_dim >= 2)
    /// * `z` - Z coordinates (required if num_dim >= 3)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File is not initialized
    /// - Array lengths don't match count
    /// - start + count exceeds num_nodes
    pub fn put_partial_coords<T: CoordValue>(
        &mut self,
        start: usize,
        count: usize,
        x: &[T],
        y: Option<&[T]>,
        z: Option<&[T]>,
    ) -> Result<()> {
        // Validate that file is initialized
        if !self.metadata.initialized {
            return Err(ExodusError::NotInitialized);
        }

        // Get number of nodes and dimensions
        let num_nodes =
            self.metadata
                .dim_cache
                .get("num_nodes")
                .copied()
                .ok_or(ExodusError::Other(
                    "num_nodes dimension not found".to_string(),
                ))?;
        let num_dim = self.metadata.num_dim.ok_or(ExodusError::Other(
            "num_dim not set in metadata".to_string(),
        ))?;

        // Validate range
        if start + count > num_nodes {
            return Err(ExodusError::InvalidArrayLength {
                expected: num_nodes - start,
                actual: count,
            });
        }

        // Validate array lengths
        if x.len() != count {
            return Err(ExodusError::InvalidArrayLength {
                expected: count,
                actual: x.len(),
            });
        }

        // Write X coordinates
        self.put_partial_coord_dim(0, start, x)?;

        // Write Y coordinates if needed
        if num_dim >= 2 {
            if let Some(y_coords) = y {
                if y_coords.len() != count {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: count,
                        actual: y_coords.len(),
                    });
                }
                self.put_partial_coord_dim(1, start, y_coords)?;
            } else {
                return Err(ExodusError::Other(
                    "Y coordinates required for num_dim >= 2".to_string(),
                ));
            }
        }

        // Write Z coordinates if needed
        if num_dim >= 3 {
            if let Some(z_coords) = z {
                if z_coords.len() != count {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: count,
                        actual: z_coords.len(),
                    });
                }
                self.put_partial_coord_dim(2, start, z_coords)?;
            } else {
                return Err(ExodusError::Other(
                    "Z coordinates required for num_dim >= 3".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Write partial coordinates for a specific dimension
    fn put_partial_coord_dim<T: CoordValue>(
        &mut self,
        dim: usize,
        start: usize,
        coords: &[T],
    ) -> Result<()> {
        let var_name = match dim {
            0 => "coordx",
            1 => "coordy",
            2 => "coordz",
            _ => {
                return Err(ExodusError::InvalidDimension {
                    expected: "0, 1, or 2".to_string(),
                    actual: dim,
                })
            }
        };

        let mut var = self
            .nc_file
            .variable_mut(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        // Convert to f64 for writing
        let data: Vec<f64> = coords.iter().map(|&v| v.to_f64()).collect();

        // Write the data at the specified offset (use range)
        var.put_values(&data, start..(start + coords.len()))?;

        Ok(())
    }

    /// Write coordinate names
    ///
    /// # Arguments
    ///
    /// * `names` - Array of coordinate names (e.g., ["X", "Y", "Z"])
    ///   Length should match num_dim (1, 2, or 3)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The file is not initialized
    /// - Array length doesn't match num_dim
    /// - Any name exceeds 32 characters
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// file.builder().dimensions(3).nodes(8).finish()?;
    ///
    /// file.put_coord_names(&["X", "Y", "Z"])?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn put_coord_names(&mut self, names: &[&str]) -> Result<()> {
        const MAX_NAME_LENGTH: usize = 32;

        // Validate that file is initialized
        if !self.metadata.initialized {
            return Err(ExodusError::NotInitialized);
        }

        let num_dim = self.metadata.num_dim.ok_or(ExodusError::Other(
            "num_dim not set in metadata".to_string(),
        ))?;

        // Validate array length
        if names.len() != num_dim {
            return Err(ExodusError::InvalidArrayLength {
                expected: num_dim,
                actual: names.len(),
            });
        }

        // Validate name lengths
        for name in names.iter() {
            if name.len() > MAX_NAME_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_NAME_LENGTH,
                    actual: name.len(),
                });
            }
        }

        // Create or get len_name dimension
        if self.nc_file.dimension("len_name").is_none() {
            self.nc_file.add_dimension("len_name", MAX_NAME_LENGTH)?;
        }

        // Create coordinate names variable: coor_names(num_dim, len_name)
        if self.nc_file.variable("coor_names").is_none() {
            self.nc_file
                .add_variable::<u8>("coor_names", &["num_dim", "len_name"])?;
        }

        // Write each coordinate name
        if let Some(mut var) = self.nc_file.variable_mut("coor_names") {
            for (i, name) in names.iter().enumerate() {
                let mut buf = vec![0u8; MAX_NAME_LENGTH];
                let bytes = name.as_bytes();
                let copy_len = bytes.len().min(MAX_NAME_LENGTH);
                buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
                var.put_values(&buf, (i..i + 1, ..))?;
            }
        }

        Ok(())
    }
}

// Reader methods
#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Read all coordinates
    ///
    /// # Returns
    ///
    /// A `Coordinates<T>` struct containing x, y, z coordinate vectors
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File is not initialized
    /// - Coordinate variables are not found
    /// - NetCDF read fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// let coords = file.coords::<f64>()?;
    /// println!("Number of nodes: {}", coords.len());
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn coords<T: CoordValue>(&self) -> Result<Coordinates<T>> {
        // Handle the case where num_nodes is 0 (dimension may not exist)
        let num_nodes = self
            .nc_file
            .dimension("num_nodes")
            .map(|d| d.len())
            .unwrap_or(0);

        let num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::Other("num_dim dimension not found".to_string()))?
            .len();

        // Return empty vectors if num_nodes is 0
        if num_nodes == 0 {
            return Ok(Coordinates {
                x: vec![],
                y: vec![],
                z: vec![],
                num_dim,
            });
        }

        // Read X coordinates
        let x = self.get_coord_x::<T>()?;

        // Read Y coordinates if num_dim >= 2, otherwise return empty vector
        let y = if num_dim >= 2 {
            self.get_coord_y::<T>()?
        } else {
            vec![]
        };

        // Read Z coordinates if num_dim >= 3, otherwise return empty vector
        let z = if num_dim >= 3 {
            self.get_coord_z::<T>()?
        } else {
            vec![]
        };

        Ok(Coordinates { x, y, z, num_dim })
    }

    /// Read all coordinates as a 2D ndarray (NumPy-compatible)
    ///
    /// Returns coordinates as an (N, 3) ndarray where N is the number of nodes.
    /// This is more efficient for NumPy integration via PyO3 as it provides
    /// a contiguous memory layout compatible with NumPy arrays.
    ///
    /// # Returns
    ///
    /// An `Array2<f64>` with shape (num_nodes, 3) where:
    /// - Column 0: X coordinates
    /// - Column 1: Y coordinates (0.0 for 1D meshes)
    /// - Column 2: Z coordinates (0.0 for 1D/2D meshes)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File is not initialized
    /// - Coordinate variables are not found
    /// - NetCDF read fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// let coords = file.coords_array()?;
    /// println!("Shape: {:?}", coords.shape());  // (num_nodes, 3)
    /// println!("First node: {:?}", coords.row(0));
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    #[cfg(feature = "ndarray")]
    pub fn coords_array(&self) -> Result<ndarray::Array2<f64>> {
        use ndarray::Array2;

        let coords = self.coords::<f64>()?;
        let num_nodes = coords.x.len();

        // If no nodes, return empty array
        if num_nodes == 0 {
            return Ok(Array2::zeros((0, 3)));
        }

        // Create array with shape (num_nodes, 3)
        let mut arr = Array2::zeros((num_nodes, 3));

        // Fill columns
        for (i, &x) in coords.x.iter().enumerate() {
            arr[[i, 0]] = x;
        }

        if coords.num_dim >= 2 {
            for (i, &y) in coords.y.iter().enumerate() {
                arr[[i, 1]] = y;
            }
        }

        if coords.num_dim == 3 {
            for (i, &z) in coords.z.iter().enumerate() {
                arr[[i, 2]] = z;
            }
        }

        Ok(arr)
    }

    /// Read coordinates into provided buffers
    ///
    /// # Arguments
    ///
    /// * `x` - Buffer for X coordinates
    /// * `y` - Buffer for Y coordinates (required if num_dim >= 2)
    /// * `z` - Buffer for Z coordinates (required if num_dim >= 3)
    ///
    /// # Errors
    ///
    /// Returns an error if buffer sizes don't match num_nodes
    pub fn get_coords<T: CoordValue>(
        &self,
        x: &mut [T],
        y: Option<&mut [T]>,
        z: Option<&mut [T]>,
    ) -> Result<()> {
        // Get num_nodes and num_dim
        let num_nodes = self
            .nc_file
            .dimension("num_nodes")
            .ok_or_else(|| ExodusError::Other("num_nodes dimension not found".to_string()))?
            .len();

        let num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::Other("num_dim dimension not found".to_string()))?
            .len();

        // Validate buffer sizes
        if x.len() != num_nodes {
            return Err(ExodusError::InvalidArrayLength {
                expected: num_nodes,
                actual: x.len(),
            });
        }

        // Read X coordinates
        let x_data = self.get_coord_x::<T>()?;
        x.copy_from_slice(&x_data);

        // Read Y coordinates if needed
        if num_dim >= 2 {
            if let Some(y_buf) = y {
                if y_buf.len() != num_nodes {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_nodes,
                        actual: y_buf.len(),
                    });
                }
                let y_data = self.get_coord_y::<T>()?;
                y_buf.copy_from_slice(&y_data);
            } else {
                return Err(ExodusError::Other(
                    "Y buffer required for num_dim >= 2".to_string(),
                ));
            }
        }

        // Read Z coordinates if needed
        if num_dim >= 3 {
            if let Some(z_buf) = z {
                if z_buf.len() != num_nodes {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_nodes,
                        actual: z_buf.len(),
                    });
                }
                let z_data = self.get_coord_z::<T>()?;
                z_buf.copy_from_slice(&z_data);
            } else {
                return Err(ExodusError::Other(
                    "Z buffer required for num_dim >= 3".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Read X coordinates
    ///
    /// # Returns
    ///
    /// A vector of X coordinates for all nodes
    pub fn get_coord_x<T: CoordValue>(&self) -> Result<Vec<T>> {
        self.get_coord_dim(0)
    }

    /// Read Y coordinates
    ///
    /// # Returns
    ///
    /// A vector of Y coordinates for all nodes
    pub fn get_coord_y<T: CoordValue>(&self) -> Result<Vec<T>> {
        self.get_coord_dim(1)
    }

    /// Read Z coordinates
    ///
    /// # Returns
    ///
    /// A vector of Z coordinates for all nodes
    pub fn get_coord_z<T: CoordValue>(&self) -> Result<Vec<T>> {
        self.get_coord_dim(2)
    }

    /// Read coordinates for a specific dimension
    fn get_coord_dim<T: CoordValue>(&self, dim: usize) -> Result<Vec<T>> {
        let var_name = match dim {
            0 => "coordx",
            1 => "coordy",
            2 => "coordz",
            _ => {
                return Err(ExodusError::InvalidDimension {
                    expected: "0, 1, or 2".to_string(),
                    actual: dim,
                })
            }
        };

        let var = self
            .nc_file
            .variable(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        // Read as f64 from NetCDF (use .. for full range)
        let data: Vec<f64> = var.get_values(..)?;

        // Convert to target type
        Ok(data.iter().map(|&v| T::from_f64(v)).collect())
    }

    /// Read partial coordinates
    ///
    /// # Arguments
    ///
    /// * `start` - Starting node index (0-based)
    /// * `count` - Number of nodes to read
    ///
    /// # Returns
    ///
    /// A `Coordinates<T>` struct containing the requested coordinates
    pub fn get_partial_coords<T: CoordValue>(
        &self,
        start: usize,
        count: usize,
    ) -> Result<Coordinates<T>> {
        let num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::Other("num_dim dimension not found".to_string()))?
            .len();

        let num_nodes = self
            .nc_file
            .dimension("num_nodes")
            .ok_or_else(|| ExodusError::Other("num_nodes dimension not found".to_string()))?
            .len();

        // Validate range
        if start + count > num_nodes {
            return Err(ExodusError::InvalidArrayLength {
                expected: num_nodes - start,
                actual: count,
            });
        }

        // Read X coordinates
        let x = self.get_partial_coord_dim::<T>(0, start, count)?;

        // Read Y coordinates if num_dim >= 2
        let y = if num_dim >= 2 {
            self.get_partial_coord_dim::<T>(1, start, count)?
        } else {
            vec![T::default(); count]
        };

        // Read Z coordinates if num_dim >= 3
        let z = if num_dim >= 3 {
            self.get_partial_coord_dim::<T>(2, start, count)?
        } else {
            vec![T::default(); count]
        };

        Ok(Coordinates { x, y, z, num_dim })
    }

    /// Read partial coordinates for a specific dimension
    fn get_partial_coord_dim<T: CoordValue>(
        &self,
        dim: usize,
        start: usize,
        count: usize,
    ) -> Result<Vec<T>> {
        let var_name = match dim {
            0 => "coordx",
            1 => "coordy",
            2 => "coordz",
            _ => {
                return Err(ExodusError::InvalidDimension {
                    expected: "0, 1, or 2".to_string(),
                    actual: dim,
                })
            }
        };

        let var = self
            .nc_file
            .variable(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        // Read as f64 from NetCDF (use range)
        let data: Vec<f64> = var.get_values(start..(start + count))?;

        // Convert to target type
        Ok(data.iter().map(|&v| T::from_f64(v)).collect())
    }

    /// Read coordinate names
    ///
    /// # Returns
    ///
    /// Vector of coordinate names, or empty vector if not present.
    /// Length will be num_dim (1, 2, or 3).
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let names = file.coord_names()?;
    /// println!("Coordinate names: {:?}", names);
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn coord_names(&self) -> Result<Vec<String>> {
        // Check if coor_names variable exists
        match self.nc_file.variable("coor_names") {
            Some(var) => {
                let num_dim = self.metadata.num_dim.unwrap_or(3);

                // Support both classic NetCDF fixed-length char arrays (NC_CHAR) and
                // NetCDF-4 NC_STRING variables for name storage.
                let dims = var.dimensions();

                // If 1D, likely NC_STRING [num_dim]
                if dims.len() == 1 {
                    let mut names = Vec::with_capacity(num_dim);
                    for i in 0..num_dim {
                        let s = var.get_string(i..i + 1)?;
                        names.push(s.trim_end_matches('\0').trim().to_string());
                    }
                    return Ok(names);
                }

                // Otherwise, expect 2D [num_dim, len_string] of NC_CHAR
                let mut names = Vec::with_capacity(num_dim);

                // Get len_string for fixed-length names
                let len_string = dims.get(1).map(|d| d.len()).unwrap_or(33);

                // Read each coordinate name (NC_CHAR stored as i8 in older files)
                for i in 0..num_dim {
                    let name_chars_i8: Vec<i8> = var.get_values((i..i + 1, 0..len_string))?;
                    // Convert i8 bytes to u8 slice for UTF-8 decoding
                    let name_bytes: Vec<u8> = name_chars_i8.iter().map(|&b| b as u8).collect();
                    let name = String::from_utf8_lossy(&name_bytes)
                        .trim_end_matches('\0')
                        .trim()
                        .to_string();
                    names.push(name);
                }

                Ok(names)
            }
            None => Ok(Vec::new()),
        }
    }
}

// Append mode can both read and write
#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Append> {
    /// Write all coordinates (append mode)
    pub fn put_coords<T: CoordValue>(
        &mut self,
        x: &[T],
        y: Option<&[T]>,
        z: Option<&[T]>,
    ) -> Result<()> {
        // Ensure we're in data mode for writing coordinate values
        self.ensure_data_mode()?;

        // Same implementation as Write mode
        // (In a real implementation, we'd refactor to avoid duplication)
        if !self.metadata.initialized {
            return Err(ExodusError::NotInitialized);
        }

        let num_nodes =
            self.metadata
                .dim_cache
                .get("num_nodes")
                .copied()
                .ok_or(ExodusError::Other(
                    "num_nodes dimension not found".to_string(),
                ))?;
        let num_dim = self.metadata.num_dim.ok_or(ExodusError::Other(
            "num_dim not set in metadata".to_string(),
        ))?;

        if x.len() != num_nodes {
            return Err(ExodusError::InvalidArrayLength {
                expected: num_nodes,
                actual: x.len(),
            });
        }

        self.put_coord_x(x)?;

        if num_dim >= 2 {
            if let Some(y_coords) = y {
                if y_coords.len() != num_nodes {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_nodes,
                        actual: y_coords.len(),
                    });
                }
                self.put_coord_y(y_coords)?;
            } else {
                return Err(ExodusError::Other(
                    "Y coordinates required for num_dim >= 2".to_string(),
                ));
            }
        }

        if num_dim >= 3 {
            if let Some(z_coords) = z {
                if z_coords.len() != num_nodes {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_nodes,
                        actual: z_coords.len(),
                    });
                }
                self.put_coord_z(z_coords)?;
            } else {
                return Err(ExodusError::Other(
                    "Z coordinates required for num_dim >= 3".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Write X coordinates (append mode)
    pub fn put_coord_x<T: CoordValue>(&mut self, x: &[T]) -> Result<()> {
        self.put_coord_dim(0, x)
    }

    /// Write Y coordinates (append mode)
    pub fn put_coord_y<T: CoordValue>(&mut self, y: &[T]) -> Result<()> {
        self.put_coord_dim(1, y)
    }

    /// Write Z coordinates (append mode)
    pub fn put_coord_z<T: CoordValue>(&mut self, z: &[T]) -> Result<()> {
        self.put_coord_dim(2, z)
    }

    /// Write coordinates for a specific dimension
    fn put_coord_dim<T: CoordValue>(&mut self, dim: usize, coords: &[T]) -> Result<()> {
        let var_name = match dim {
            0 => "coordx",
            1 => "coordy",
            2 => "coordz",
            _ => {
                return Err(ExodusError::InvalidDimension {
                    expected: "0, 1, or 2".to_string(),
                    actual: dim,
                })
            }
        };

        let mut var = self
            .nc_file
            .variable_mut(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        let data: Vec<f64> = coords.iter().map(|&v| v.to_f64()).collect();
        var.put_values(&data, ..)?;

        Ok(())
    }

    /// Read all coordinates (append mode)
    pub fn coords<T: CoordValue>(&self) -> Result<Coordinates<T>> {
        // Handle the case where num_nodes is 0 (dimension may not exist)
        let num_nodes = self
            .nc_file
            .dimension("num_nodes")
            .map(|d| d.len())
            .unwrap_or(0);

        let num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::Other("num_dim dimension not found".to_string()))?
            .len();

        // Return empty vectors if num_nodes is 0
        if num_nodes == 0 {
            return Ok(Coordinates {
                x: vec![],
                y: vec![],
                z: vec![],
                num_dim,
            });
        }

        let x = self.get_coord_x::<T>()?;
        let y = if num_dim >= 2 {
            self.get_coord_y::<T>()?
        } else {
            vec![]
        };
        let z = if num_dim >= 3 {
            self.get_coord_z::<T>()?
        } else {
            vec![]
        };

        Ok(Coordinates { x, y, z, num_dim })
    }

    /// Read X coordinates (append mode)
    pub fn get_coord_x<T: CoordValue>(&self) -> Result<Vec<T>> {
        self.get_coord_dim(0)
    }

    /// Read Y coordinates (append mode)
    pub fn get_coord_y<T: CoordValue>(&self) -> Result<Vec<T>> {
        self.get_coord_dim(1)
    }

    /// Read Z coordinates (append mode)
    pub fn get_coord_z<T: CoordValue>(&self) -> Result<Vec<T>> {
        self.get_coord_dim(2)
    }

    /// Read coordinates for a specific dimension
    fn get_coord_dim<T: CoordValue>(&self, dim: usize) -> Result<Vec<T>> {
        let var_name = match dim {
            0 => "coordx",
            1 => "coordy",
            2 => "coordz",
            _ => {
                return Err(ExodusError::InvalidDimension {
                    expected: "0, 1, or 2".to_string(),
                    actual: dim,
                })
            }
        };

        let var = self
            .nc_file
            .variable(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        let data: Vec<f64> = var.get_values(..)?;
        Ok(data.iter().map(|&v| T::from_f64(v)).collect())
    }
}

#[cfg(test)]
#[cfg(feature = "netcdf4")]
mod tests {
    use super::*;
    use crate::ExodusFile;
    use crate::{mode, CreateMode, CreateOptions};
    use approx::assert_relative_eq;
    use tempfile::NamedTempFile;

    // Helper function to create file with clobber mode for tests
    fn create_test_file(
        path: impl AsRef<std::path::Path>,
    ) -> crate::Result<ExodusFile<mode::Write>> {
        ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_coords_2d() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = create_test_file(tmp.path()).unwrap();
            file.builder().dimensions(2).nodes(4).finish().unwrap();

            let x = vec![0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0, 0.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), None).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let coords = file.coords::<f64>().unwrap();

            assert_eq!(coords.x.len(), 4);
            assert_eq!(coords.y.len(), 4);
            assert_eq!(coords.num_dim, 2);

            let coord0 = coords.get(0).unwrap();
            assert_relative_eq!(coord0[0], 0.0);
            assert_relative_eq!(coord0[1], 0.0);

            let coord2 = coords.get(2).unwrap();
            assert_relative_eq!(coord2[0], 1.0);
            assert_relative_eq!(coord2[1], 1.0);
        }
    }

    #[test]
    fn test_coords_3d() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = create_test_file(tmp.path()).unwrap();
            file.builder().dimensions(3).nodes(8).finish().unwrap();

            let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let coords = file.coords::<f64>().unwrap();

            assert_eq!(coords.x.len(), 8);
            assert_eq!(coords.y.len(), 8);
            assert_eq!(coords.z.len(), 8);
            assert_eq!(coords.num_dim, 3);

            // Check corner coordinates
            let coord0 = coords.get(0).unwrap();
            assert_relative_eq!(coord0[0], 0.0);
            assert_relative_eq!(coord0[1], 0.0);
            assert_relative_eq!(coord0[2], 0.0);

            let coord7 = coords.get(7).unwrap();
            assert_relative_eq!(coord7[0], 0.0);
            assert_relative_eq!(coord7[1], 1.0);
            assert_relative_eq!(coord7[2], 1.0);
        }
    }

    #[test]
    fn test_partial_coords() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = create_test_file(tmp.path()).unwrap();
            file.builder().dimensions(2).nodes(10).finish().unwrap();

            // Write first 5 nodes
            let x1 = vec![0.0, 1.0, 2.0, 3.0, 4.0];
            let y1 = vec![0.0, 0.0, 0.0, 0.0, 0.0];
            file.put_partial_coords(0, 5, &x1, Some(&y1), None).unwrap();

            // Write next 5 nodes
            let x2 = vec![5.0, 6.0, 7.0, 8.0, 9.0];
            let y2 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
            file.put_partial_coords(5, 5, &x2, Some(&y2), None).unwrap();
        }

        // Read partial
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

            // Read middle section
            let coords = file.get_partial_coords::<f64>(3, 4).unwrap();
            assert_eq!(coords.len(), 4);

            assert_relative_eq!(coords.x[0], 3.0);
            assert_relative_eq!(coords.x[1], 4.0);
            assert_relative_eq!(coords.x[2], 5.0);
            assert_relative_eq!(coords.x[3], 6.0);
        }
    }

    #[test]
    fn test_coord_type_conversion() {
        let tmp = NamedTempFile::new().unwrap();

        // Write as f32
        {
            let mut file = create_test_file(tmp.path()).unwrap();
            file.builder().dimensions(2).nodes(3).finish().unwrap();

            let x: Vec<f32> = vec![0.0, 1.0, 2.0];
            let y: Vec<f32> = vec![0.0, 1.0, 2.0];
            file.put_coords(&x, Some(&y), None).unwrap();
        }

        // Read as f64
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let coords = file.coords::<f64>().unwrap();

            assert_eq!(coords.len(), 3);
            assert_relative_eq!(coords.x[1], 1.0);
            assert_relative_eq!(coords.y[2], 2.0);
        }

        // Read as f32
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let coords = file.coords::<f32>().unwrap();

            assert_eq!(coords.len(), 3);
            assert_relative_eq!(coords.x[1], 1.0);
            assert_relative_eq!(coords.y[2], 2.0);
        }
    }

    #[test]
    fn test_coordinates_iterator() {
        let coords = Coordinates {
            x: vec![0.0, 1.0, 2.0],
            y: vec![0.0, 1.0, 2.0],
            z: vec![0.0, 0.0, 0.0],
            num_dim: 2,
        };

        let points: Vec<[f64; 3]> = coords.iter().collect();
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], [0.0, 0.0, 0.0]);
        assert_eq!(points[1], [1.0, 1.0, 0.0]);
        assert_eq!(points[2], [2.0, 2.0, 0.0]);
    }

    #[test]
    fn test_get_coords_into_buffer() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = create_test_file(tmp.path()).unwrap();
            file.builder().dimensions(2).nodes(4).finish().unwrap();

            let x = vec![0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0, 0.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), None).unwrap();
        }

        // Read into buffers
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let mut x_buf = vec![0.0f64; 4];
            let mut y_buf = vec![0.0f64; 4];

            file.get_coords(&mut x_buf, Some(&mut y_buf), None).unwrap();

            assert_relative_eq!(x_buf[0], 0.0);
            assert_relative_eq!(x_buf[2], 1.0);
            assert_relative_eq!(y_buf[2], 1.0);
        }
    }

    #[test]
    fn test_individual_coord_dims() {
        let tmp = NamedTempFile::new().unwrap();

        // Write each dimension separately
        {
            let mut file = create_test_file(tmp.path()).unwrap();
            file.builder().dimensions(3).nodes(2).finish().unwrap();

            let x = vec![1.0, 2.0];
            let y = vec![3.0, 4.0];
            let z = vec![5.0, 6.0];

            file.put_coord_x(&x).unwrap();
            file.put_coord_y(&y).unwrap();
            file.put_coord_z(&z).unwrap();
        }

        // Read each dimension separately
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

            let x = file.get_coord_x::<f64>().unwrap();
            let y = file.get_coord_y::<f64>().unwrap();
            let z = file.get_coord_z::<f64>().unwrap();

            assert_eq!(x, vec![1.0, 2.0]);
            assert_eq!(y, vec![3.0, 4.0]);
            assert_eq!(z, vec![5.0, 6.0]);
        }
    }
}
