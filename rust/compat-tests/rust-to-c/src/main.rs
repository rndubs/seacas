//! Rust-to-C compatibility test writer
//!
//! This program generates Exodus II files using the Rust exodus-rs library.
//! The generated files are then read and validated by C programs.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod basic_mesh;
mod element_blocks;
mod sets;
mod variables;

#[derive(Parser)]
#[command(name = "exodus-rust-writer")]
#[command(about = "Generate Exodus files for C compatibility testing")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output directory for generated files
    #[arg(short, long, default_value = "output")]
    output_dir: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a basic 2D mesh with a single quad element
    BasicMesh2d,

    /// Generate a basic 3D mesh with a single hex element
    BasicMesh3d,

    /// Generate mesh with multiple element blocks
    MultipleBlocks,

    /// Generate mesh with node sets
    NodeSets,

    /// Generate mesh with side sets
    SideSets,

    /// Generate mesh with element sets
    ElementSets,

    /// Generate mesh with all set types
    AllSets,

    /// Generate mesh with global variables
    GlobalVariables,

    /// Generate mesh with nodal variables
    NodalVariables,

    /// Generate mesh with element variables
    ElementVariables,

    /// Generate mesh with all variable types
    AllVariables,

    /// Generate all test files
    All,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&cli.output_dir)
        .context("Failed to create output directory")?;

    if cli.verbose {
        println!("Output directory: {}", cli.output_dir.display());
    }

    match cli.command {
        Commands::BasicMesh2d => {
            let path = cli.output_dir.join("basic_mesh_2d.exo");
            basic_mesh::generate_2d(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::BasicMesh3d => {
            let path = cli.output_dir.join("basic_mesh_3d.exo");
            basic_mesh::generate_3d(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::MultipleBlocks => {
            let path = cli.output_dir.join("multiple_blocks.exo");
            element_blocks::generate_multiple_blocks(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::NodeSets => {
            let path = cli.output_dir.join("node_sets.exo");
            sets::generate_node_sets(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::SideSets => {
            let path = cli.output_dir.join("side_sets.exo");
            sets::generate_side_sets(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::ElementSets => {
            let path = cli.output_dir.join("element_sets.exo");
            sets::generate_element_sets(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::AllSets => {
            let path = cli.output_dir.join("all_sets.exo");
            sets::generate_all_sets(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::GlobalVariables => {
            let path = cli.output_dir.join("global_variables.exo");
            variables::generate_global_variables(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::NodalVariables => {
            let path = cli.output_dir.join("nodal_variables.exo");
            variables::generate_nodal_variables(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::ElementVariables => {
            let path = cli.output_dir.join("element_variables.exo");
            variables::generate_element_variables(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::AllVariables => {
            let path = cli.output_dir.join("all_variables.exo");
            variables::generate_all_variables(&path)?;
            println!("✓ Generated: {}", path.display());
        }

        Commands::All => {
            let tests = [
                ("basic_mesh_2d.exo", basic_mesh::generate_2d as fn(&PathBuf) -> Result<()>),
                ("basic_mesh_3d.exo", basic_mesh::generate_3d),
                ("multiple_blocks.exo", element_blocks::generate_multiple_blocks),
                ("node_sets.exo", sets::generate_node_sets),
                ("side_sets.exo", sets::generate_side_sets),
                ("element_sets.exo", sets::generate_element_sets),
                ("all_sets.exo", sets::generate_all_sets),
                ("global_variables.exo", variables::generate_global_variables),
                ("nodal_variables.exo", variables::generate_nodal_variables),
                ("element_variables.exo", variables::generate_element_variables),
                ("all_variables.exo", variables::generate_all_variables),
            ];

            for (filename, generator) in tests {
                let path = cli.output_dir.join(filename);
                generator(&path)?;
                println!("✓ Generated: {}", path.display());
            }

            println!("\n✓ All {} test files generated successfully!", tests.len());
        }
    }

    Ok(())
}
