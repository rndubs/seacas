use crate::cli::{Axis, Operation, Result, TransformError};
use exodus_rs::{transformations::rotation_matrix_from_euler, ExodusFile};

/// Apply a simple operation (not CopyMirrorMerge) to the mesh
pub fn apply_simple_operation(
    file: &mut ExodusFile<exodus_rs::mode::Append>,
    op: &Operation,
    verbose: bool,
) -> Result<()> {
    match op {
        Operation::ScaleLen(factor) => {
            if verbose {
                println!("  Scaling coordinates by factor {}", factor);
            }
            file.scale_uniform(*factor)?;
        }
        Operation::Mirror(axis) => {
            let scale = match axis {
                Axis::X => [-1.0, 1.0, 1.0],
                Axis::Y => [1.0, -1.0, 1.0],
                Axis::Z => [1.0, 1.0, -1.0],
            };
            if verbose {
                println!("  Mirroring about {:?} axis", axis);
            }
            file.scale(&scale)?;
        }
        Operation::Translate(offset) => {
            if verbose {
                println!(
                    "  Translating by [{}, {}, {}]",
                    offset[0], offset[1], offset[2]
                );
            }
            file.translate(offset)?;
        }
        Operation::Rotate(sequence, angles) => {
            if verbose {
                let rotation_type = if sequence.chars().next().unwrap().is_uppercase() {
                    "extrinsic"
                } else {
                    "intrinsic"
                };
                println!(
                    "  Rotating {} '{}' by {:?} degrees",
                    rotation_type, sequence, angles
                );
            }
            // Get the rotation matrix and apply it
            let matrix = rotation_matrix_from_euler(sequence, angles, true)?;
            file.apply_rotation(&matrix)?;
        }
        Operation::ScaleField(field_name, scale_factor) => {
            if verbose {
                println!(
                    "  Scaling field variable '{}' by factor {}",
                    field_name, scale_factor
                );
            }
            file.scale_field_variable(field_name, *scale_factor, verbose)?;
        }
        Operation::CopyMirrorMerge(_, _) => {
            // This should be handled separately, not through apply_simple_operation
            return Err(TransformError::InvalidFormat(
                "CopyMirrorMerge must be handled specially".to_string(),
            ));
        }
    }
    Ok(())
}

/// Normalize time values by subtracting the first time step from all time steps
pub fn normalize_time(file: &mut ExodusFile<exodus_rs::mode::Append>, verbose: bool) -> Result<()> {
    let times = file.times()?;

    if times.is_empty() {
        if verbose {
            println!("  No time steps to normalize");
        }
        return Ok(());
    }

    let first_time = times[0];
    if verbose {
        println!(
            "  Normalizing time: subtracting {} from all {} time steps",
            first_time,
            times.len()
        );
    }

    for (step, time) in times.iter().enumerate() {
        let normalized = time - first_time;
        file.put_time(step, normalized)?;
    }

    Ok(())
}
