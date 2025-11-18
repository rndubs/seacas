//! Element topology utilities for face and side definitions.
//!
//! This module provides face/side connectivity information for standard Exodus II
//! element topologies. The face definitions follow the canonical Exodus II conventions
//! as defined in the SEACAS library and documented in the Exodus II specification.
//!
//! Face numbering uses 1-based indexing (matching Exodus convention), while node
//! indices within the connectivity arrays are 0-based (matching Rust arrays).

use crate::types::Topology;

/// Face/side definition for an element.
///
/// Defines which nodes form a particular face/side of an element,
/// using 0-based indices into the element's connectivity array.
#[derive(Debug, Clone, PartialEq)]
pub struct FaceDef {
    /// Exodus side number (1-based)
    pub side_number: usize,
    /// Node indices (0-based) that form this face
    pub node_indices: Vec<usize>,
}

impl Topology {
    /// Get all face definitions for this topology.
    ///
    /// Returns `None` for topologies that don't support face/side extraction
    /// (e.g., point elements, unsupported types).
    ///
    /// The face definitions use:
    /// - 1-based side numbering (matching Exodus convention)
    /// - 0-based node indices (for indexing into connectivity arrays)
    /// - Right-hand rule winding (normals point outward from element)
    ///
    /// # Examples
    ///
    /// ```
    /// use exodus_rs::Topology;
    ///
    /// let hex = Topology::Hex8;
    /// let faces = hex.faces().unwrap();
    /// assert_eq!(faces.len(), 6); // Hex has 6 faces
    ///
    /// // Face 1 (side 1) uses nodes 0,1,5,4 of the hex
    /// assert_eq!(faces[0].side_number, 1);
    /// assert_eq!(faces[0].node_indices, vec![0, 1, 5, 4]);
    /// ```
    pub fn faces(&self) -> Option<Vec<FaceDef>> {
        match self {
            // Hexahedral elements
            Topology::Hex8 | Topology::Hex20 | Topology::Hex27 => Some(hex_faces()),

            // Tetrahedral elements
            Topology::Tet4
            | Topology::Tet8
            | Topology::Tet10
            | Topology::Tet14
            | Topology::Tet15 => Some(tet_faces()),

            // Wedge/Prism elements
            Topology::Wedge6 | Topology::Wedge15 | Topology::Wedge18 => Some(wedge_faces()),

            // Pyramidal elements
            Topology::Pyramid5 | Topology::Pyramid13 | Topology::Pyramid14 => Some(pyramid_faces()),

            // Quadrilateral elements (2D or shell)
            Topology::Quad4 | Topology::Quad8 | Topology::Quad9 => Some(quad_faces()),

            // Triangular elements (2D or shell)
            Topology::Tri3 | Topology::Tri6 | Topology::Tri7 => Some(tri_faces()),

            // Unsupported or point elements
            _ => None,
        }
    }

    /// Get the number of faces/sides for this topology.
    ///
    /// Returns `None` for topologies that don't support face extraction.
    pub fn num_faces(&self) -> Option<usize> {
        self.faces().map(|faces| faces.len())
    }
}

/// Hexahedral element face definitions.
///
/// 6 quadrilateral faces, following Exodus II convention.
/// Based on IOSS Hex8 implementation and Exodus II specification.
///
/// Face numbering (1-based):
/// - Face 1: Front  (nodes 1,2,6,5 in 1-based, 0,1,5,4 in 0-based)
/// - Face 2: Right  (nodes 2,3,7,6)
/// - Face 3: Back   (nodes 3,4,8,7)
/// - Face 4: Left   (nodes 1,5,8,4)
/// - Face 5: Bottom (nodes 1,4,3,2)
/// - Face 6: Top    (nodes 5,6,7,8)
fn hex_faces() -> Vec<FaceDef> {
    vec![
        FaceDef {
            side_number: 1,
            node_indices: vec![0, 1, 5, 4],
        },
        FaceDef {
            side_number: 2,
            node_indices: vec![1, 2, 6, 5],
        },
        FaceDef {
            side_number: 3,
            node_indices: vec![2, 3, 7, 6],
        },
        FaceDef {
            side_number: 4,
            node_indices: vec![0, 4, 7, 3],
        },
        FaceDef {
            side_number: 5,
            node_indices: vec![0, 3, 2, 1],
        },
        FaceDef {
            side_number: 6,
            node_indices: vec![4, 5, 6, 7],
        },
    ]
}

/// Tetrahedral element face definitions.
///
/// 4 triangular faces, following Exodus II convention.
/// Based on IOSS Tet4 implementation and Exodus II specification.
///
/// Face numbering (1-based):
/// - Face 1: nodes 1,2,4 (in 1-based), 0,1,3 (in 0-based)
/// - Face 2: nodes 2,3,4
/// - Face 3: nodes 1,4,3
/// - Face 4: nodes 1,3,2
fn tet_faces() -> Vec<FaceDef> {
    vec![
        FaceDef {
            side_number: 1,
            node_indices: vec![0, 1, 3],
        },
        FaceDef {
            side_number: 2,
            node_indices: vec![1, 2, 3],
        },
        FaceDef {
            side_number: 3,
            node_indices: vec![0, 3, 2],
        },
        FaceDef {
            side_number: 4,
            node_indices: vec![0, 2, 1],
        },
    ]
}

/// Wedge/Prism element face definitions.
///
/// 5 faces: 3 quadrilaterals and 2 triangles, following Exodus II convention.
/// Based on IOSS Wedge6 implementation and Exodus II specification.
///
/// Note: Exodus face numbering differs from MSC/Patran convention.
///
/// Face numbering (1-based):
/// - Face 1: Quad (nodes 1,2,5,4)
/// - Face 2: Quad (nodes 2,3,6,5)
/// - Face 3: Quad (nodes 1,4,6,3)
/// - Face 4: Triangle (nodes 1,3,2)
/// - Face 5: Triangle (nodes 4,5,6)
fn wedge_faces() -> Vec<FaceDef> {
    vec![
        FaceDef {
            side_number: 1,
            node_indices: vec![0, 1, 4, 3],
        },
        FaceDef {
            side_number: 2,
            node_indices: vec![1, 2, 5, 4],
        },
        FaceDef {
            side_number: 3,
            node_indices: vec![0, 3, 5, 2],
        },
        FaceDef {
            side_number: 4,
            node_indices: vec![0, 2, 1],
        },
        FaceDef {
            side_number: 5,
            node_indices: vec![3, 4, 5],
        },
    ]
}

/// Pyramidal element face definitions.
///
/// 5 faces: 4 triangles and 1 quadrilateral, following Exodus II convention.
/// Based on IOSS Pyramid5 implementation and Exodus II specification.
///
/// Face numbering (1-based):
/// - Face 1: Triangle (nodes 1,2,5)
/// - Face 2: Triangle (nodes 2,3,5)
/// - Face 3: Triangle (nodes 3,4,5)
/// - Face 4: Triangle (nodes 4,1,5)
/// - Face 5: Quad (nodes 1,4,3,2)
fn pyramid_faces() -> Vec<FaceDef> {
    vec![
        FaceDef {
            side_number: 1,
            node_indices: vec![0, 1, 4],
        },
        FaceDef {
            side_number: 2,
            node_indices: vec![1, 2, 4],
        },
        FaceDef {
            side_number: 3,
            node_indices: vec![2, 3, 4],
        },
        FaceDef {
            side_number: 4,
            node_indices: vec![3, 0, 4],
        },
        FaceDef {
            side_number: 5,
            node_indices: vec![0, 3, 2, 1],
        },
    ]
}

/// Quadrilateral element face definitions (edges for 2D elements).
///
/// 4 edges, following Exodus II convention.
/// For 2D quadrilateral elements, "faces" are actually edges.
///
/// Face/Edge numbering (1-based):
/// - Face 1: Edge from node 1 to 2
/// - Face 2: Edge from node 2 to 3
/// - Face 3: Edge from node 3 to 4
/// - Face 4: Edge from node 4 to 1
fn quad_faces() -> Vec<FaceDef> {
    vec![
        FaceDef {
            side_number: 1,
            node_indices: vec![0, 1],
        },
        FaceDef {
            side_number: 2,
            node_indices: vec![1, 2],
        },
        FaceDef {
            side_number: 3,
            node_indices: vec![2, 3],
        },
        FaceDef {
            side_number: 4,
            node_indices: vec![3, 0],
        },
    ]
}

/// Triangular element face definitions (edges for 2D elements).
///
/// 3 edges, following Exodus II convention.
/// For 2D triangular elements, "faces" are actually edges.
///
/// Face/Edge numbering (1-based):
/// - Face 1: Edge from node 1 to 2
/// - Face 2: Edge from node 2 to 3
/// - Face 3: Edge from node 3 to 1
fn tri_faces() -> Vec<FaceDef> {
    vec![
        FaceDef {
            side_number: 1,
            node_indices: vec![0, 1],
        },
        FaceDef {
            side_number: 2,
            node_indices: vec![1, 2],
        },
        FaceDef {
            side_number: 3,
            node_indices: vec![2, 0],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex8_faces() {
        let topo = Topology::Hex8;
        let faces = topo.faces().unwrap();

        assert_eq!(faces.len(), 6);
        assert_eq!(faces[0].side_number, 1);
        assert_eq!(faces[0].node_indices, vec![0, 1, 5, 4]);
        assert_eq!(faces[5].side_number, 6);
        assert_eq!(faces[5].node_indices, vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_tet4_faces() {
        let topo = Topology::Tet4;
        let faces = topo.faces().unwrap();

        assert_eq!(faces.len(), 4);
        assert_eq!(faces[0].side_number, 1);
        assert_eq!(faces[0].node_indices, vec![0, 1, 3]);
    }

    #[test]
    fn test_wedge6_faces() {
        let topo = Topology::Wedge6;
        let faces = topo.faces().unwrap();

        assert_eq!(faces.len(), 5);
        // First 3 are quads
        assert_eq!(faces[0].node_indices.len(), 4);
        assert_eq!(faces[1].node_indices.len(), 4);
        assert_eq!(faces[2].node_indices.len(), 4);
        // Last 2 are triangles
        assert_eq!(faces[3].node_indices.len(), 3);
        assert_eq!(faces[4].node_indices.len(), 3);
    }

    #[test]
    fn test_pyramid5_faces() {
        let topo = Topology::Pyramid5;
        let faces = topo.faces().unwrap();

        assert_eq!(faces.len(), 5);
        // First 4 are triangles
        assert_eq!(faces[0].node_indices.len(), 3);
        assert_eq!(faces[1].node_indices.len(), 3);
        assert_eq!(faces[2].node_indices.len(), 3);
        assert_eq!(faces[3].node_indices.len(), 3);
        // Last is quad
        assert_eq!(faces[4].node_indices.len(), 4);
    }

    #[test]
    fn test_quad4_faces() {
        let topo = Topology::Quad4;
        let faces = topo.faces().unwrap();

        assert_eq!(faces.len(), 4);
        // All are edges (2 nodes)
        for face in faces {
            assert_eq!(face.node_indices.len(), 2);
        }
    }

    #[test]
    fn test_tri3_faces() {
        let topo = Topology::Tri3;
        let faces = topo.faces().unwrap();

        assert_eq!(faces.len(), 3);
        // All are edges (2 nodes)
        for face in faces {
            assert_eq!(face.node_indices.len(), 2);
        }
    }

    #[test]
    fn test_unsupported_topology() {
        let topo = Topology::Sphere;
        assert!(topo.faces().is_none());

        let topo = Topology::NSided;
        assert!(topo.faces().is_none());
    }

    #[test]
    fn test_higher_order_elements() {
        // Higher-order elements use the same face definitions
        // (just with additional mid-side nodes)
        assert_eq!(Topology::Hex20.faces().unwrap().len(), 6);
        assert_eq!(Topology::Hex27.faces().unwrap().len(), 6);
        assert_eq!(Topology::Tet10.faces().unwrap().len(), 4);
    }

    #[test]
    fn test_num_faces() {
        assert_eq!(Topology::Hex8.num_faces(), Some(6));
        assert_eq!(Topology::Tet4.num_faces(), Some(4));
        assert_eq!(Topology::Wedge6.num_faces(), Some(5));
        assert_eq!(Topology::Pyramid5.num_faces(), Some(5));
        assert_eq!(Topology::Quad4.num_faces(), Some(4));
        assert_eq!(Topology::Tri3.num_faces(), Some(3));
        assert_eq!(Topology::Sphere.num_faces(), None);
    }
}
