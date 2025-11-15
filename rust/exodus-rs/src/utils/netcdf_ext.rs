//! Extensions and helpers for working with the netcdf crate
//!
//! This module provides convenience functions for common NetCDF operations
//! used throughout the exodus-rs crate, reducing code duplication and
//! improving maintainability.

use crate::error::{ExodusError, Result};

#[cfg(feature = "netcdf4")]
use netcdf::{AttributeValue, Variable};

/// Extract an i64 value from a NetCDF AttributeValue
///
/// Handles conversion from various numeric AttributeValue types to i64.
/// Supports: Short, Int, Longlong, Ulonglong, Float, Double.
///
/// # Arguments
///
/// * `value` - The AttributeValue to convert
///
/// # Returns
///
/// Some(i64) if conversion is successful, None if the value type is not numeric.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::attr_value_to_i64;
///
/// let attr = variable.attribute("id").unwrap();
/// let attr_value = attr.value().unwrap();
/// if let Some(id) = attr_value_to_i64(&attr_value) {
///     println!("ID: {}", id);
/// }
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn attr_value_to_i64(value: &AttributeValue) -> Option<i64> {
    match value {
        AttributeValue::Short(v) => Some(*v as i64),
        AttributeValue::Int(v) => Some(*v as i64),
        AttributeValue::Longlong(v) => Some(*v),
        AttributeValue::Ulonglong(v) => Some(*v as i64),
        AttributeValue::Float(v) => Some(*v as i64),
        AttributeValue::Double(v) => Some(*v as i64),
        AttributeValue::Uchar(v) => Some(*v as i64),
        AttributeValue::Ushort(v) => Some(*v as i64),
        AttributeValue::Uint(v) => Some(*v as i64),
        _ => None,
    }
}

/// Extract a String from a NetCDF AttributeValue
///
/// Handles conversion from Str AttributeValue type to String.
///
/// # Arguments
///
/// * `value` - The AttributeValue to convert
///
/// # Returns
///
/// Some(String) if the value is a Str variant, None otherwise.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::attr_value_to_string;
///
/// let attr = variable.attribute("name").unwrap();
/// let attr_value = attr.value().unwrap();
/// if let Some(name) = attr_value_to_string(&attr_value) {
///     println!("Name: {}", name);
/// }
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn attr_value_to_string(value: &AttributeValue) -> Option<String> {
    match value {
        AttributeValue::Str(s) => Some(s.clone()),
        _ => None,
    }
}

/// Extract an f64 value from a NetCDF AttributeValue
///
/// Handles conversion from various numeric AttributeValue types to f64.
/// Supports: Short, Int, Longlong, Ulonglong, Float, Double.
///
/// # Arguments
///
/// * `value` - The AttributeValue to convert
///
/// # Returns
///
/// Some(f64) if conversion is successful, None if the value type is not numeric.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::attr_value_to_f64;
///
/// let attr = variable.attribute("scale").unwrap();
/// let attr_value = attr.value().unwrap();
/// if let Some(scale) = attr_value_to_f64(&attr_value) {
///     println!("Scale: {}", scale);
/// }
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn attr_value_to_f64(value: &AttributeValue) -> Option<f64> {
    match value {
        AttributeValue::Short(v) => Some(*v as f64),
        AttributeValue::Int(v) => Some(*v as f64),
        AttributeValue::Longlong(v) => Some(*v as f64),
        AttributeValue::Ulonglong(v) => Some(*v as f64),
        AttributeValue::Float(v) => Some(*v as f64),
        AttributeValue::Double(v) => Some(*v),
        AttributeValue::Uchar(v) => Some(*v as f64),
        AttributeValue::Ushort(v) => Some(*v as f64),
        AttributeValue::Uint(v) => Some(*v as f64),
        _ => None,
    }
}

/// Get an attribute value from a variable and convert it to i64
///
/// This is a convenience function that combines attribute lookup,
/// value extraction, and conversion to i64 in one call.
///
/// # Arguments
///
/// * `var` - The NetCDF variable to read from
/// * `attr_name` - Name of the attribute to read
///
/// # Returns
///
/// The i64 value if successful, or an error if the attribute doesn't exist
/// or cannot be converted to i64.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::get_attr_i64;
///
/// let id = get_attr_i64(&variable, "id")?;
/// println!("ID: {}", id);
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn get_attr_i64(var: &Variable<'_>, attr_name: &str) -> Result<i64> {
    let attr = var.attribute(attr_name).ok_or_else(|| ExodusError::Other(
        format!("Attribute '{}' not found", attr_name)
    ))?;

    let value = attr.value().map_err(ExodusError::NetCdf)?;

    attr_value_to_i64(&value).ok_or_else(|| ExodusError::Other(
        format!("Cannot convert attribute '{}' to i64", attr_name)
    ))
}

/// Get an attribute value from a variable and convert it to String
///
/// This is a convenience function that combines attribute lookup,
/// value extraction, and conversion to String in one call.
///
/// # Arguments
///
/// * `var` - The NetCDF variable to read from
/// * `attr_name` - Name of the attribute to read
///
/// # Returns
///
/// The String value if successful, or an error if the attribute doesn't exist
/// or is not a string type.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::get_attr_string;
///
/// let name = get_attr_string(&variable, "name")?;
/// println!("Name: {}", name);
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn get_attr_string(var: &Variable<'_>, attr_name: &str) -> Result<String> {
    let attr = var.attribute(attr_name).ok_or_else(|| ExodusError::Other(
        format!("Attribute '{}' not found", attr_name)
    ))?;

    let value = attr.value().map_err(ExodusError::NetCdf)?;

    attr_value_to_string(&value).ok_or_else(|| ExodusError::Other(
        format!("Cannot convert attribute '{}' to String", attr_name)
    ))
}

/// Get an attribute value from a variable and convert it to f64
///
/// This is a convenience function that combines attribute lookup,
/// value extraction, and conversion to f64 in one call.
///
/// # Arguments
///
/// * `var` - The NetCDF variable to read from
/// * `attr_name` - Name of the attribute to read
///
/// # Returns
///
/// The f64 value if successful, or an error if the attribute doesn't exist
/// or cannot be converted to f64.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::get_attr_f64;
///
/// let scale = get_attr_f64(&variable, "scale")?;
/// println!("Scale: {}", scale);
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn get_attr_f64(var: &Variable<'_>, attr_name: &str) -> Result<f64> {
    let attr = var.attribute(attr_name).ok_or_else(|| ExodusError::Other(
        format!("Attribute '{}' not found", attr_name)
    ))?;

    let value = attr.value().map_err(ExodusError::NetCdf)?;

    attr_value_to_f64(&value).ok_or_else(|| ExodusError::Other(
        format!("Cannot convert attribute '{}' to f64", attr_name)
    ))
}

/// Try to get an attribute value from a variable and convert it to i64
///
/// Unlike `get_attr_i64`, this function returns None instead of an error
/// if the attribute doesn't exist or cannot be converted.
///
/// # Arguments
///
/// * `var` - The NetCDF variable to read from
/// * `attr_name` - Name of the attribute to read
///
/// # Returns
///
/// Some(i64) if successful, None if the attribute doesn't exist or cannot be converted.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::try_get_attr_i64;
///
/// if let Some(id) = try_get_attr_i64(&variable, "id") {
///     println!("ID: {}", id);
/// } else {
///     println!("No ID attribute");
/// }
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn try_get_attr_i64(var: &Variable<'_>, attr_name: &str) -> Option<i64> {
    let attr = var.attribute(attr_name)?;
    let value = attr.value().ok()?;
    attr_value_to_i64(&value)
}

/// Try to get an attribute value from a variable and convert it to String
///
/// Unlike `get_attr_string`, this function returns None instead of an error
/// if the attribute doesn't exist or cannot be converted.
///
/// # Arguments
///
/// * `var` - The NetCDF variable to read from
/// * `attr_name` - Name of the attribute to read
///
/// # Returns
///
/// Some(String) if successful, None if the attribute doesn't exist or is not a string.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::try_get_attr_string;
///
/// let name = try_get_attr_string(&variable, "name").unwrap_or_else(|| "unnamed".to_string());
/// println!("Name: {}", name);
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn try_get_attr_string(var: &Variable<'_>, attr_name: &str) -> Option<String> {
    let attr = var.attribute(attr_name)?;
    let value = attr.value().ok()?;
    attr_value_to_string(&value)
}

/// Try to get an attribute value from a variable and convert it to f64
///
/// Unlike `get_attr_f64`, this function returns None instead of an error
/// if the attribute doesn't exist or cannot be converted.
///
/// # Arguments
///
/// * `var` - The NetCDF variable to read from
/// * `attr_name` - Name of the attribute to read
///
/// # Returns
///
/// Some(f64) if successful, None if the attribute doesn't exist or cannot be converted.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::utils::netcdf_ext::try_get_attr_f64;
///
/// if let Some(scale) = try_get_attr_f64(&variable, "scale") {
///     println!("Scale: {}", scale);
/// } else {
///     println!("Using default scale");
/// }
/// ```
#[allow(dead_code)]
#[cfg(feature = "netcdf4")]
pub fn try_get_attr_f64(var: &Variable<'_>, attr_name: &str) -> Option<f64> {
    let attr = var.attribute(attr_name)?;
    let value = attr.value().ok()?;
    attr_value_to_f64(&value)
}

#[cfg(all(test, feature = "netcdf4"))]
mod tests {
    use super::*;
    use netcdf::AttributeValue;

    #[test]
    fn test_attr_value_to_i64() {
        assert_eq!(attr_value_to_i64(&AttributeValue::Short(42)), Some(42));
        assert_eq!(attr_value_to_i64(&AttributeValue::Int(100)), Some(100));
        assert_eq!(attr_value_to_i64(&AttributeValue::Longlong(1000)), Some(1000));
        assert_eq!(attr_value_to_i64(&AttributeValue::Ulonglong(2000)), Some(2000));
        assert_eq!(attr_value_to_i64(&AttributeValue::Float(3.14)), Some(3));
        assert_eq!(attr_value_to_i64(&AttributeValue::Double(2.71)), Some(2));
        assert_eq!(attr_value_to_i64(&AttributeValue::Uchar(255)), Some(255));
        assert_eq!(attr_value_to_i64(&AttributeValue::Ushort(500)), Some(500));
        assert_eq!(attr_value_to_i64(&AttributeValue::Uint(1000)), Some(1000));
        assert_eq!(attr_value_to_i64(&AttributeValue::Str("test".to_string())), None);
    }

    #[test]
    fn test_attr_value_to_string() {
        assert_eq!(
            attr_value_to_string(&AttributeValue::Str("hello".to_string())),
            Some("hello".to_string())
        );
        assert_eq!(attr_value_to_string(&AttributeValue::Int(42)), None);
        assert_eq!(attr_value_to_string(&AttributeValue::Double(3.14)), None);
    }

    #[test]
    fn test_attr_value_to_f64() {
        assert_eq!(attr_value_to_f64(&AttributeValue::Short(42)), Some(42.0));
        assert_eq!(attr_value_to_f64(&AttributeValue::Int(100)), Some(100.0));
        assert_eq!(attr_value_to_f64(&AttributeValue::Longlong(1000)), Some(1000.0));
        // Note: Float (f32) values lose precision when converted to f64
        let float_result = attr_value_to_f64(&AttributeValue::Float(3.14_f32));
        assert!(float_result.is_some());
        assert!((float_result.unwrap() - 3.14).abs() < 0.01); // Allow small epsilon for f32->f64 conversion
        assert_eq!(attr_value_to_f64(&AttributeValue::Double(2.718281828)), Some(2.718281828));
        assert_eq!(attr_value_to_f64(&AttributeValue::Str("test".to_string())), None);
    }

    #[test]
    fn test_attr_value_conversions_with_edge_cases() {
        // Test zero values
        assert_eq!(attr_value_to_i64(&AttributeValue::Int(0)), Some(0));
        assert_eq!(attr_value_to_f64(&AttributeValue::Double(0.0)), Some(0.0));

        // Test negative values
        assert_eq!(attr_value_to_i64(&AttributeValue::Int(-42)), Some(-42));
        assert_eq!(attr_value_to_f64(&AttributeValue::Double(-3.14)), Some(-3.14));

        // Test max values
        assert_eq!(attr_value_to_i64(&AttributeValue::Longlong(i64::MAX)), Some(i64::MAX));

        // Test empty string
        assert_eq!(
            attr_value_to_string(&AttributeValue::Str("".to_string())),
            Some("".to_string())
        );
    }
}
