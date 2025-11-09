//! Metadata operations
//!
//! This module provides QA records, info records, and name operations for Exodus files.

use crate::error::{ExodusError, Result};
use crate::types::QaRecord;
use crate::{mode, ExodusFile};

#[cfg(feature = "netcdf4")]
use netcdf;

// Constants for maximum string lengths
const MAX_QA_STRING_LENGTH: usize = 32;
const MAX_INFO_STRING_LENGTH: usize = 80;
const MAX_NAME_STRING_LENGTH: usize = 32;

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Write QA (Quality Assurance) records
    ///
    /// QA records track software provenance - what codes have processed the file,
    /// what versions, and when.
    ///
    /// # Arguments
    ///
    /// * `qa_records` - Array of QA records to write
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any string field exceeds 32 characters
    /// - NetCDF operations fail
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::{ExodusFile, QaRecord};
    ///
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// let qa = vec![
    ///     QaRecord {
    ///         code_name: "exodus-rs".into(),
    ///         code_version: "0.1.0".into(),
    ///         date: "2025-01-09".into(),
    ///         time: "12:00:00".into(),
    ///     },
    /// ];
    /// file.put_qa(&qa)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn put_qa(&mut self, qa_records: &[QaRecord]) -> Result<()> {
        // Validate all QA records first
        for (i, qa) in qa_records.iter().enumerate() {
            if qa.code_name.len() > MAX_QA_STRING_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_QA_STRING_LENGTH,
                    actual: qa.code_name.len(),
                });
            }
            if qa.code_version.len() > MAX_QA_STRING_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_QA_STRING_LENGTH,
                    actual: qa.code_version.len(),
                });
            }
            if qa.date.len() > MAX_QA_STRING_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_QA_STRING_LENGTH,
                    actual: qa.date.len(),
                });
            }
            if qa.time.len() > MAX_QA_STRING_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_QA_STRING_LENGTH,
                    actual: qa.time.len(),
                });
            }
        }

        // Create dimensions
        let num_qa = qa_records.len();
        if num_qa > 0 {
            self.nc_file.add_dimension("num_qa_rec", num_qa)?;
            self.nc_file
                .add_dimension("four", 4)?; // QA records have 4 strings each
            self.nc_file
                .add_dimension("len_string", MAX_QA_STRING_LENGTH)?;

            // Create variable: qa_records(num_qa_rec, four, len_string)
            let mut var = self
                .nc_file
                .add_variable::<u8>("qa_records", &["num_qa_rec", "four", "len_string"])?;

            // Write QA records
            for (i, qa) in qa_records.iter().enumerate() {
                let strings = [
                    &qa.code_name,
                    &qa.code_version,
                    &qa.date,
                    &qa.time,
                ];

                for (j, s) in strings.iter().enumerate() {
                    let mut padded = [b' '; MAX_QA_STRING_LENGTH];
                    let bytes = s.as_bytes();
                    let copy_len = bytes.len().min(MAX_QA_STRING_LENGTH);
                    padded[..copy_len].copy_from_slice(&bytes[..copy_len]);

                    // Write string as byte array
                    var.put_values(&padded, Some(&[i, j, 0]), Some(&[1, 1, MAX_QA_STRING_LENGTH]))?;
                }
            }
        }

        Ok(())
    }

    /// Write information records
    ///
    /// Info records are arbitrary text strings (max 80 characters each) that
    /// can contain any information about the file.
    ///
    /// # Arguments
    ///
    /// * `info` - Array of information strings
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any string exceeds 80 characters
    /// - NetCDF operations fail
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// let info = vec![
    ///     "This is a test mesh",
    ///     "Generated for demonstration purposes",
    /// ];
    /// file.put_info(&info)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn put_info(&mut self, info: &[impl AsRef<str>]) -> Result<()> {
        // Validate all info records first
        for (i, s) in info.iter().enumerate() {
            let s = s.as_ref();
            if s.len() > MAX_INFO_STRING_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_INFO_STRING_LENGTH,
                    actual: s.len(),
                });
            }
        }

        // Create dimensions
        let num_info = info.len();
        if num_info > 0 {
            self.nc_file.add_dimension("num_info", num_info)?;
            self.nc_file
                .add_dimension("len_line", MAX_INFO_STRING_LENGTH)?;

            // Create variable: info_records(num_info, len_line)
            let mut var = self
                .nc_file
                .add_variable::<u8>("info_records", &["num_info", "len_line"])?;

            // Write info records
            for (i, s) in info.iter().enumerate() {
                let s = s.as_ref();
                let mut padded = [b' '; MAX_INFO_STRING_LENGTH];
                let bytes = s.as_bytes();
                let copy_len = bytes.len().min(MAX_INFO_STRING_LENGTH);
                padded[..copy_len].copy_from_slice(&bytes[..copy_len]);

                // Write string as byte array
                var.put_values(&padded, Some(&[i, 0]), Some(&[1, MAX_INFO_STRING_LENGTH]))?;
            }
        }

        Ok(())
    }

    /// Set coordinate axis names
    ///
    /// # Arguments
    ///
    /// * `names` - Array of coordinate names (length must match num_dim)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Number of names doesn't match num_dim
    /// - Any name exceeds 32 characters
    /// - File not initialized
    /// - NetCDF operations fail
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::{ExodusFile, InitParams};
    ///
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// file.init(&InitParams {
    ///     num_dim: 3,
    ///     ..Default::default()
    /// })?;
    /// file.put_coord_names(&["X", "Y", "Z"])?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn put_coord_names(&mut self, names: &[impl AsRef<str>]) -> Result<()> {
        // Check initialization
        let num_dim = self
            .metadata
            .num_dim
            .ok_or_else(|| ExodusError::NotInitialized)?;

        // Validate number of names
        if names.len() != num_dim {
            return Err(ExodusError::InvalidArrayLength {
                expected: num_dim,
                actual: names.len(),
            });
        }

        // Validate string lengths
        for name in names {
            let name = name.as_ref();
            if name.len() > MAX_NAME_STRING_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_NAME_STRING_LENGTH,
                    actual: name.len(),
                });
            }
        }

        // Ensure len_name dimension exists
        if self.nc_file.dimension("len_name").is_none() {
            self.nc_file
                .add_dimension("len_name", MAX_NAME_STRING_LENGTH)?;
        }

        // Create variable: coor_names(num_dim, len_name)
        let mut var = self
            .nc_file
            .add_variable::<u8>("coor_names", &["num_dim", "len_name"])?;

        // Write coordinate names
        for (i, name) in names.iter().enumerate() {
            let name = name.as_ref();
            let mut padded = [b' '; MAX_NAME_STRING_LENGTH];
            let bytes = name.as_bytes();
            let copy_len = bytes.len().min(MAX_NAME_STRING_LENGTH);
            padded[..copy_len].copy_from_slice(&bytes[..copy_len]);

            // Write string as byte array
            var.put_values(&padded, Some(&[i, 0]), Some(&[1, MAX_NAME_STRING_LENGTH]))?;
        }

        Ok(())
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Get QA records from the file
    ///
    /// # Returns
    ///
    /// Vector of QA records stored in the file (may be empty)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// let qa = file.qa_records()?;
    /// for record in qa {
    ///     println!("{} v{}", record.code_name, record.code_version);
    /// }
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn qa_records(&self) -> Result<Vec<QaRecord>> {
        // Check if QA records exist
        let num_qa = self
            .nc_file
            .dimension("num_qa_rec")
            .map(|d| d.len())
            .unwrap_or(0);

        if num_qa == 0 {
            return Ok(Vec::new());
        }

        // Read QA records variable
        let var = self
            .nc_file
            .variable("qa_records")
            .ok_or_else(|| ExodusError::VariableNotDefined("qa_records".to_string()))?;

        let mut records = Vec::with_capacity(num_qa);

        for i in 0..num_qa {
            let mut strings: [String; 4] = Default::default();

            for j in 0..4 {
                let mut buffer = [0u8; MAX_QA_STRING_LENGTH];
                var.get_values(&mut buffer, Some(&[i, j, 0]), Some(&[1, 1, MAX_QA_STRING_LENGTH]))?;

                // Convert to string, trimming trailing spaces
                let s = String::from_utf8_lossy(&buffer);
                strings[j] = s.trim_end().to_string();
            }

            records.push(QaRecord {
                code_name: strings[0].clone(),
                code_version: strings[1].clone(),
                date: strings[2].clone(),
                time: strings[3].clone(),
            });
        }

        Ok(records)
    }

    /// Get information records from the file
    ///
    /// # Returns
    ///
    /// Vector of information strings (may be empty)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// let info = file.info_records()?;
    /// for line in info {
    ///     println!("{}", line);
    /// }
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn info_records(&self) -> Result<Vec<String>> {
        // Check if info records exist
        let num_info = self
            .nc_file
            .dimension("num_info")
            .map(|d| d.len())
            .unwrap_or(0);

        if num_info == 0 {
            return Ok(Vec::new());
        }

        // Read info records variable
        let var = self
            .nc_file
            .variable("info_records")
            .ok_or_else(|| ExodusError::VariableNotDefined("info_records".to_string()))?;

        let mut records = Vec::with_capacity(num_info);

        for i in 0..num_info {
            let mut buffer = [0u8; MAX_INFO_STRING_LENGTH];
            var.get_values(&mut buffer, Some(&[i, 0]), Some(&[1, MAX_INFO_STRING_LENGTH]))?;

            // Convert to string, trimming trailing spaces
            let s = String::from_utf8_lossy(&buffer);
            records.push(s.trim_end().to_string());
        }

        Ok(records)
    }

    /// Get coordinate axis names
    ///
    /// # Returns
    ///
    /// Vector of coordinate names (defaults to ["X", "Y", "Z"] if not set)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// let names = file.coord_names()?;
    /// println!("Coordinate axes: {:?}", names);
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn coord_names(&self) -> Result<Vec<String>> {
        // Get num_dim
        let num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::VariableNotDefined("num_dim".to_string()))?
            .len();

        // Check if coordinate names variable exists
        if let Some(var) = self.nc_file.variable("coor_names") {
            let mut names = Vec::with_capacity(num_dim);

            for i in 0..num_dim {
                let mut buffer = [0u8; MAX_NAME_STRING_LENGTH];
                var.get_values(&mut buffer, Some(&[i, 0]), Some(&[1, MAX_NAME_STRING_LENGTH]))?;

                // Convert to string, trimming trailing spaces
                let s = String::from_utf8_lossy(&buffer);
                names.push(s.trim_end().to_string());
            }

            Ok(names)
        } else {
            // Return default names
            Ok(match num_dim {
                1 => vec!["X".to_string()],
                2 => vec!["X".to_string(), "Y".to_string()],
                3 => vec!["X".to_string(), "Y".to_string(), "Z".to_string()],
                _ => {
                    return Err(ExodusError::InvalidDimension {
                        expected: "1, 2, or 3".to_string(),
                        actual: num_dim,
                    })
                }
            })
        }
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Append> {
    /// Get QA records (same as Read mode)
    pub fn qa_records(&self) -> Result<Vec<QaRecord>> {
        let num_qa = self
            .nc_file
            .dimension("num_qa_rec")
            .map(|d| d.len())
            .unwrap_or(0);

        if num_qa == 0 {
            return Ok(Vec::new());
        }

        let var = self
            .nc_file
            .variable("qa_records")
            .ok_or_else(|| ExodusError::VariableNotDefined("qa_records".to_string()))?;

        let mut records = Vec::with_capacity(num_qa);

        for i in 0..num_qa {
            let mut strings: [String; 4] = Default::default();

            for j in 0..4 {
                let mut buffer = [0u8; MAX_QA_STRING_LENGTH];
                var.get_values(&mut buffer, Some(&[i, j, 0]), Some(&[1, 1, MAX_QA_STRING_LENGTH]))?;

                let s = String::from_utf8_lossy(&buffer);
                strings[j] = s.trim_end().to_string();
            }

            records.push(QaRecord {
                code_name: strings[0].clone(),
                code_version: strings[1].clone(),
                date: strings[2].clone(),
                time: strings[3].clone(),
            });
        }

        Ok(records)
    }

    /// Get information records (same as Read mode)
    pub fn info_records(&self) -> Result<Vec<String>> {
        let num_info = self
            .nc_file
            .dimension("num_info")
            .map(|d| d.len())
            .unwrap_or(0);

        if num_info == 0 {
            return Ok(Vec::new());
        }

        let var = self
            .nc_file
            .variable("info_records")
            .ok_or_else(|| ExodusError::VariableNotDefined("info_records".to_string()))?;

        let mut records = Vec::with_capacity(num_info);

        for i in 0..num_info {
            let mut buffer = [0u8; MAX_INFO_STRING_LENGTH];
            var.get_values(&mut buffer, Some(&[i, 0]), Some(&[1, MAX_INFO_STRING_LENGTH]))?;

            let s = String::from_utf8_lossy(&buffer);
            records.push(s.trim_end().to_string());
        }

        Ok(records)
    }

    /// Get coordinate axis names (same as Read mode)
    pub fn coord_names(&self) -> Result<Vec<String>> {
        let num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::VariableNotDefined("num_dim".to_string()))?
            .len();

        if let Some(var) = self.nc_file.variable("coor_names") {
            let mut names = Vec::with_capacity(num_dim);

            for i in 0..num_dim {
                let mut buffer = [0u8; MAX_NAME_STRING_LENGTH];
                var.get_values(&mut buffer, Some(&[i, 0]), Some(&[1, MAX_NAME_STRING_LENGTH]))?;

                let s = String::from_utf8_lossy(&buffer);
                names.push(s.trim_end().to_string());
            }

            Ok(names)
        } else {
            Ok(match num_dim {
                1 => vec!["X".to_string()],
                2 => vec!["X".to_string(), "Y".to_string()],
                3 => vec!["X".to_string(), "Y".to_string(), "Z".to_string()],
                _ => {
                    return Err(ExodusError::InvalidDimension {
                        expected: "1, 2, or 3".to_string(),
                        actual: num_dim,
                    })
                }
            })
        }
    }
}

#[cfg(test)]
#[cfg(feature = "netcdf4")]
mod tests {
    use super::*;
    use crate::types::InitParams;
    use tempfile::NamedTempFile;

    #[test]
    fn test_qa_records() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create_default(tmp.path()).unwrap();
            file.init(&InitParams::default()).unwrap();

            let qa = vec![QaRecord {
                code_name: "exodus-rs".into(),
                code_version: "0.1.0".into(),
                date: "2025-01-09".into(),
                time: "12:00:00".into(),
            }];

            file.put_qa(&qa).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let qa = file.qa_records().unwrap();
            assert_eq!(qa.len(), 1);
            assert_eq!(qa[0].code_name, "exodus-rs");
            assert_eq!(qa[0].code_version, "0.1.0");
        }
    }

    #[test]
    fn test_info_records() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create_default(tmp.path()).unwrap();
            file.init(&InitParams::default()).unwrap();

            let info = vec!["Line 1", "Line 2", "Line 3"];
            file.put_info(&info).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let info = file.info_records().unwrap();
            assert_eq!(info.len(), 3);
            assert_eq!(info[0], "Line 1");
            assert_eq!(info[1], "Line 2");
            assert_eq!(info[2], "Line 3");
        }
    }

    #[test]
    fn test_coord_names() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create_default(tmp.path()).unwrap();
            file.init(&InitParams {
                num_dim: 3,
                ..Default::default()
            })
            .unwrap();

            file.put_coord_names(&["X", "Y", "Z"]).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let names = file.coord_names().unwrap();
            assert_eq!(names, vec!["X", "Y", "Z"]);
        }
    }

    #[test]
    fn test_coord_names_default() {
        let tmp = NamedTempFile::new().unwrap();

        {
            let mut file = ExodusFile::create_default(tmp.path()).unwrap();
            file.init(&InitParams {
                num_dim: 2,
                ..Default::default()
            })
            .unwrap();
        }

        // Read - should get default names
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let names = file.coord_names().unwrap();
            assert_eq!(names, vec!["X", "Y"]);
        }
    }

    #[test]
    fn test_qa_string_too_long() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = ExodusFile::create_default(tmp.path()).unwrap();
        file.init(&InitParams::default()).unwrap();

        let qa = vec![QaRecord {
            code_name: "a".repeat(33), // Too long
            code_version: "1.0".into(),
            date: "2025-01-09".into(),
            time: "12:00:00".into(),
        }];

        let result = file.put_qa(&qa);
        assert!(result.is_err());
    }

    #[test]
    fn test_info_string_too_long() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = ExodusFile::create_default(tmp.path()).unwrap();
        file.init(&InitParams::default()).unwrap();

        let info = vec!["a".repeat(81)]; // Too long
        let result = file.put_info(&info);
        assert!(result.is_err());
    }
}
