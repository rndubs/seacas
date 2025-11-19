//! NumPy conversion utilities for exodus-py
//!
//! This module provides helper functions for converting between Rust types
//! and NumPy arrays, with backward compatibility for Python lists.

use pyo3::prelude::*;
use pyo3::types::{PyList, PySequence};

#[cfg(feature = "numpy")]
use numpy::{PyArray1, PyArrayMethods, ToPyArray};

/// Extract a Vec<f64> from either a Python list or NumPy array
///
/// This function accepts both:
/// - Python lists: [1.0, 2.0, 3.0]
/// - NumPy arrays: np.array([1.0, 2.0, 3.0])
///
/// This provides backward compatibility while enabling NumPy support.
#[cfg(feature = "numpy")]
pub fn extract_f64_vec(_py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Vec<f64>> {
    // Try NumPy array first
    if let Ok(arr) = obj.downcast::<PyArray1<f64>>() {
        return Ok(arr.readonly().as_slice()?.to_vec());
    }

    // Try NumPy array with different dtypes and convert
    if let Ok(arr) = obj.downcast::<PyArray1<f32>>() {
        return Ok(arr.readonly().as_slice()?.iter().map(|&x| x as f64).collect());
    }
    if let Ok(arr) = obj.downcast::<PyArray1<i64>>() {
        return Ok(arr.readonly().as_slice()?.iter().map(|&x| x as f64).collect());
    }
    if let Ok(arr) = obj.downcast::<PyArray1<i32>>() {
        return Ok(arr.readonly().as_slice()?.iter().map(|&x| x as f64).collect());
    }

    // Fall back to extracting as a sequence (handles lists, tuples, etc.)
    let seq: Bound<'_, PySequence> = obj.downcast::<PySequence>()?.clone();
    let len = seq.len()?;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        result.push(seq.get_item(i)?.extract::<f64>()?);
    }
    Ok(result)
}

/// Extract a Vec<i64> from either a Python list or NumPy array
///
/// This function accepts both:
/// - Python lists: [1, 2, 3]
/// - NumPy arrays: np.array([1, 2, 3])
///
/// This provides backward compatibility while enabling NumPy support.
#[cfg(feature = "numpy")]
pub fn extract_i64_vec(_py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Vec<i64>> {
    // Try NumPy array first
    if let Ok(arr) = obj.downcast::<PyArray1<i64>>() {
        return Ok(arr.readonly().as_slice()?.to_vec());
    }

    // Try NumPy array with different dtypes and convert
    if let Ok(arr) = obj.downcast::<PyArray1<i32>>() {
        return Ok(arr.readonly().as_slice()?.iter().map(|&x| x as i64).collect());
    }
    if let Ok(arr) = obj.downcast::<PyArray1<u64>>() {
        return Ok(arr.readonly().as_slice()?.iter().map(|&x| x as i64).collect());
    }
    if let Ok(arr) = obj.downcast::<PyArray1<u32>>() {
        return Ok(arr.readonly().as_slice()?.iter().map(|&x| x as i64).collect());
    }

    // Fall back to extracting as a sequence (handles lists, tuples, etc.)
    let seq: Bound<'_, PySequence> = obj.downcast::<PySequence>()?.clone();
    let len = seq.len()?;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        result.push(seq.get_item(i)?.extract::<i64>()?);
    }
    Ok(result)
}

/// Convert a Vec<f64> to a NumPy array
#[cfg(feature = "numpy")]
pub fn vec_to_numpy_f64<'py>(py: Python<'py>, vec: Vec<f64>) -> Bound<'py, PyArray1<f64>> {
    vec.to_pyarray(py)
}

/// Convert a Vec<i64> to a NumPy array
#[cfg(feature = "numpy")]
pub fn vec_to_numpy_i64<'py>(py: Python<'py>, vec: Vec<i64>) -> Bound<'py, PyArray1<i64>> {
    vec.to_pyarray(py)
}

/// Convert a tuple of coordinate vectors to NumPy arrays
#[cfg(feature = "numpy")]
pub fn coords_to_numpy<'py>(
    py: Python<'py>,
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
) -> (Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>) {
    (
        vec_to_numpy_f64(py, x),
        vec_to_numpy_f64(py, y),
        vec_to_numpy_f64(py, z),
    )
}

// Fallback implementations when numpy feature is disabled
#[cfg(not(feature = "numpy"))]
pub fn extract_f64_vec(_py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Vec<f64>> {
    obj.extract::<Vec<f64>>()
}

#[cfg(not(feature = "numpy"))]
pub fn extract_i64_vec(_py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Vec<i64>> {
    obj.extract::<Vec<i64>>()
}
