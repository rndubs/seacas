//! Blob operations for Exodus II files.
//!
//! Blobs provide storage for arbitrary binary data associated with the mesh,
//! such as images, embedded documents, or custom application data.

use crate::error::{ExodusError, Result};
use crate::types::Blob;
use crate::{mode, ExodusFile};

#[cfg(feature = "netcdf4")]
// ============================================================================
// Write Operations
// ============================================================================
impl ExodusFile<mode::Write> {
    /// Store a blob with binary data
    ///
    /// Blobs store arbitrary binary data associated with the mesh.
    ///
    /// # Arguments
    ///
    /// * `blob` - Blob definition with ID and name
    /// * `data` - Binary data to store
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The blob name is too long (max 32 characters)
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// let blob = Blob {
    ///     id: 1,
    ///     name: "config_data".into(),
    /// };
    /// let data = vec![0x12, 0x34, 0x56, 0x78];
    /// file.put_blob(&blob, &data)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_blob(&mut self, blob: &Blob, data: &[u8]) -> Result<()> {
        const MAX_NAME_LENGTH: usize = 32;

        if blob.name.len() > MAX_NAME_LENGTH {
            return Err(ExodusError::StringTooLong {
                max: MAX_NAME_LENGTH,
                actual: blob.name.len(),
            });
        }

        // Count existing blobs by checking for blob variables
        let mut num_blobs = 0;
        while self
            .nc_file
            .variable(&format!("blob{}_data", num_blobs + 1))
            .is_some()
        {
            num_blobs += 1;
        }

        // Create blob-specific dimensions and variables
        let blob_index = num_blobs;

        // Create dimension for blob data size
        let data_dim_name = format!("num_bytes_blob{}", blob_index + 1);
        self.nc_file
            .add_dimension(&data_dim_name, data.len())
            .map_err(ExodusError::NetCdf)?;

        // Create variable for blob data
        let data_var_name = format!("blob{}_data", blob_index + 1);
        let mut data_var = self
            .nc_file
            .add_variable::<u8>(&data_var_name, &[&data_dim_name])
            .map_err(ExodusError::NetCdf)?;

        // Write blob data
        data_var.put_values(data, ..).map_err(ExodusError::NetCdf)?;

        // Store blob metadata as attributes
        data_var
            .put_attribute("id", blob.id)
            .map_err(ExodusError::NetCdf)?;

        data_var
            .put_attribute("name", blob.name.as_str())
            .map_err(ExodusError::NetCdf)?;

        Ok(())
    }
}

// ============================================================================
// Read Operations
// ============================================================================

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Get all blob IDs
    ///
    /// # Returns
    ///
    /// Vector of blob IDs, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let blob_ids = file.blob_ids()?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn blob_ids(&self) -> Result<Vec<i64>> {
        // Count blobs by checking for blob variables
        let mut num_blobs = 0;
        while self
            .nc_file
            .variable(&format!("blob{}_data", num_blobs + 1))
            .is_some()
        {
            num_blobs += 1;
        }

        if num_blobs == 0 {
            return Ok(Vec::new());
        }

        let mut ids = Vec::with_capacity(num_blobs);

        for i in 0..num_blobs {
            let data_var_name = format!("blob{}_data", i + 1);
            if let Some(var) = self.nc_file.variable(&data_var_name) {
                if let Some(id_attr) = var.attribute("id") {
                    if let Ok(id_value) = id_attr.value() {
                        match id_value {
                            netcdf::AttributeValue::Short(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Int(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Longlong(id) => ids.push(id),
                            netcdf::AttributeValue::Ulonglong(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Float(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Double(id) => ids.push(id as i64),
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(ids)
    }

    /// Get blob by ID (returns Blob struct and binary data)
    ///
    /// # Arguments
    ///
    /// * `blob_id` - Blob ID to retrieve
    ///
    /// # Returns
    ///
    /// Tuple of (Blob, Vec<u8>) containing the blob metadata and binary data,
    /// or an error if not found or reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let (blob, data) = file.blob(1)?;
    /// println!("Blob {} contains {} bytes", blob.name, data.len());
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn blob(&self, blob_id: i64) -> Result<(Blob, Vec<u8>)> {
        // Count blobs by checking for blob variables
        let mut num_blobs = 0;
        while self
            .nc_file
            .variable(&format!("blob{}_data", num_blobs + 1))
            .is_some()
        {
            num_blobs += 1;
        }

        if num_blobs == 0 {
            return Err(ExodusError::EntityNotFound {
                entity_type: "blob".to_string(),
                id: blob_id,
            });
        }

        for i in 0..num_blobs {
            let data_var_name = format!("blob{}_data", i + 1);
            if let Some(var) = self.nc_file.variable(&data_var_name) {
                // Check if this is the blob we're looking for
                if let Some(id_attr) = var.attribute("id") {
                    if let Ok(id_value) = id_attr.value() {
                        let id = match id_value {
                            netcdf::AttributeValue::Short(v) => v as i64,
                            netcdf::AttributeValue::Int(v) => v as i64,
                            netcdf::AttributeValue::Longlong(v) => v,
                            netcdf::AttributeValue::Ulonglong(v) => v as i64,
                            netcdf::AttributeValue::Float(v) => v as i64,
                            netcdf::AttributeValue::Double(v) => v as i64,
                            _ => continue,
                        };

                        if id == blob_id {
                            // Found it! Read all the data
                            let name = if let Some(name_attr) = var.attribute("name") {
                                if let Ok(name_value) = name_attr.value() {
                                    match name_value {
                                        netcdf::AttributeValue::Str(s) => s,
                                        _ => String::new(),
                                    }
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            let data: Vec<u8> = var.get_values(..).map_err(ExodusError::NetCdf)?;

                            return Ok((Blob { id: blob_id, name }, data));
                        }
                    }
                }
            }
        }

        Err(ExodusError::EntityNotFound {
            entity_type: "blob".to_string(),
            id: blob_id,
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "netcdf4")]
mod tests {
    use super::*;
    use crate::types::{CreateMode, CreateOptions, InitParams};
    use tempfile::NamedTempFile;

    #[test]
    fn test_blob() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Blob Test".into(),
                num_dim: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let blob = Blob {
                id: 1,
                name: "config_data".into(),
            };

            let data = vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF];
            file.put_blob(&blob, &data).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let ids = file.blob_ids().unwrap();
            assert_eq!(ids, vec![1]);

            let (blob, data) = file.blob(1).unwrap();
            assert_eq!(blob.name, "config_data");
            assert_eq!(data, vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF]);
        }
    }

    #[test]
    fn test_multiple_blobs() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Multiple Blobs Test".into(),
                num_dim: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Add multiple blobs
            file.put_blob(
                &Blob {
                    id: 1,
                    name: "data1".into(),
                },
                &[1, 2, 3],
            )
            .unwrap();

            file.put_blob(
                &Blob {
                    id: 2,
                    name: "data2".into(),
                },
                &[4, 5, 6, 7, 8],
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let ids = file.blob_ids().unwrap();
            assert_eq!(ids.len(), 2);
            assert!(ids.contains(&1));
            assert!(ids.contains(&2));

            let (blob1, data1) = file.blob(1).unwrap();
            assert_eq!(blob1.name, "data1");
            assert_eq!(data1.len(), 3);

            let (blob2, data2) = file.blob(2).unwrap();
            assert_eq!(blob2.name, "data2");
            assert_eq!(data2.len(), 5);
        }
    }

    #[test]
    fn test_empty_blob() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Empty Blob Test".into(),
                num_dim: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let blob = Blob {
                id: 42,
                name: "empty".into(),
            };

            file.put_blob(&blob, &[]).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let (blob, data) = file.blob(42).unwrap();
            assert_eq!(blob.name, "empty");
            assert_eq!(data.len(), 0);
        }
    }
}
