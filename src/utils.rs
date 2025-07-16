//! Utility functions for working with Assimp scenes and bone hierarchies

use std::collections::{HashMap, HashSet};

use log::debug;
use rbx_types::{Matrix3, Vector3};
use russimp::{node::Node, scene::Scene};

use crate::types::NodeInfo;

/// Extract node information from an Assimp scene
pub fn get_bone_infos(scene: &Scene) -> HashMap<String, NodeInfo> {
    let mut bone_infos = HashMap::new();

    // First, collect all node names that have animation channels
    let mut animated_channels = HashSet::new();
    for anim in &scene.animations {
        for channel in &anim.channels {
            animated_channels.insert(channel.name.clone());
        }
    }

    debug!(
        "Found {} animated channels: {:?}",
        animated_channels.len(),
        animated_channels
    );

    if let Some(root) = &scene.root {
        collect_node_bone_infos(root, None, &mut bone_infos, &animated_channels);
    }

    bone_infos
}

fn collect_node_bone_infos(
    node: &Node,
    parent: Option<String>,
    transforms: &mut HashMap<String, NodeInfo>,
    animated_channels: &HashSet<String>,
) {
    // A node is a bone if it has animation channels
    let is_bone = animated_channels.contains(&node.name);

    if is_bone {
        // Store this node's rest transform
        transforms.insert(
            node.name.clone(),
            NodeInfo {
                rest_transform: node.transformation,
                parent: parent.clone(),
            },
        );
        debug!("Added BONE: {} (parent: {:?})", node.name, parent);
    }

    // Recursively collect children, passing current node as parent only if it's a bone
    let next_parent = if is_bone {
        Some(node.name.clone())
    } else {
        parent
    };
    for child in node.children.borrow().iter() {
        collect_node_bone_infos(child, next_parent.clone(), transforms, animated_channels);
    }
}

/// Check if two Vector3 values are approximately equal
pub fn approx_equal_vec3(a: &Vector3, b: &Vector3, epsilon: f32) -> bool {
    (a.x - b.x).abs() <= epsilon && (a.y - b.y).abs() <= epsilon && (a.z - b.z).abs() <= epsilon
}

/// Check if two Matrix3 values are approximately equal
pub fn approx_equal_matrix3(a: &Matrix3, b: &Matrix3, epsilon: f32) -> bool {
    approx_equal_vec3(&a.x, &b.x, epsilon)
        && approx_equal_vec3(&a.y, &b.y, epsilon)
        && approx_equal_vec3(&a.z, &b.z, epsilon)
}

/// Check if two CFrame values are approximately equal
pub fn approx_equal_cframe(a: &rbx_types::CFrame, b: &rbx_types::CFrame, epsilon: f32) -> bool {
    approx_equal_vec3(&a.position, &b.position, epsilon)
        && approx_equal_matrix3(&a.orientation, &b.orientation, epsilon)
}
