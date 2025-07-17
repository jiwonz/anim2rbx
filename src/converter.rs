//! Core conversion logic for transforming animation data

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use glam::{Mat3, Quat, Vec3};
use log::debug;
use ordered_float::OrderedFloat;
use rbx_dom_weak::{InstanceBuilder, WeakDom};
use rbx_types::{CFrame, EnumItem, Matrix3, Ref, Variant, Vector3};
use russimp::scene::Scene;

use crate::types::{Keyframe, NodeInfo, Pose};
use crate::utils::approx_equal_cframe;

/// Internal structure for efficiently looking up animation data
struct ChannelData {
    name: String,
    position_map: BTreeMap<OrderedFloat<f64>, russimp::Vector3D>,
    rotation_map: BTreeMap<OrderedFloat<f64>, russimp::animation::Quaternion>,
}

/// Extract keyframes from an Assimp scene
pub fn extract_keyframes_from_scene(
    scene: &Scene,
    node_infos: &HashMap<String, NodeInfo>,
) -> Vec<Keyframe> {
    let mut keyframes = Vec::new();
    let mut channels_data = Vec::new();
    let mut all_times = BTreeSet::new();

    // Build efficient lookup structures for all channels
    for anim in &scene.animations {
        let ticks_per_second = if anim.ticks_per_second > 0.0 {
            anim.ticks_per_second
        } else {
            24.0 // Default to 24 FPS if not specified
        };

        debug!("Animation: {} ticks per second", ticks_per_second);

        for channel in &anim.channels {
            // Build position map with time converted to seconds
            let position_map: BTreeMap<OrderedFloat<f64>, russimp::Vector3D> = channel
                .position_keys
                .iter()
                .map(|key| (OrderedFloat(key.time / ticks_per_second), key.value))
                .collect();

            // Build rotation map with time converted to seconds
            let rotation_map: BTreeMap<OrderedFloat<f64>, russimp::animation::Quaternion> = channel
                .rotation_keys
                .iter()
                .map(|key| (OrderedFloat(key.time / ticks_per_second), key.value))
                .collect();

            // Collect all times (now in seconds)
            for &time in position_map.keys() {
                all_times.insert(time);
            }
            for &time in rotation_map.keys() {
                all_times.insert(time);
            }

            channels_data.push(ChannelData {
                name: channel.name.clone(),
                position_map,
                rotation_map,
            });
        }
    }

    // Create keyframes for each timestamp
    for &time_ordered in &all_times {
        let time = time_ordered.into_inner();
        let mut poses = Vec::new();

        for channel_data in &channels_data {
            // Check if this channel has any data at this time
            let has_position_key = channel_data.position_map.contains_key(&time_ordered);
            let has_rotation_key = channel_data.rotation_map.contains_key(&time_ordered);

            // Skip if no animation data exists for this bone at this time
            if !has_position_key && !has_rotation_key {
                continue;
            }

            // Calculate position relative to rest pose
            let pos = channel_data
                .position_map
                .get(&time_ordered)
                .and_then(|value| {
                    if let Some(node_info) = node_infos.get(&channel_data.name) {
                        let rest_transform = node_info.rest_transform;
                        let rest_pos = Vec3 {
                            x: rest_transform.a4,
                            y: rest_transform.b4,
                            z: rest_transform.c4,
                        };
                        let relative_pos = Vec3 {
                            x: value.x - rest_pos.x,
                            y: value.y - rest_pos.y,
                            z: value.z - rest_pos.z,
                        };
                        Some(relative_pos)
                    } else {
                        None
                    }
                })
                .unwrap_or(Vec3::ZERO);

            let rot = channel_data
                .rotation_map
                .get(&time_ordered)
                .and_then(|value| {
                    if let Some(node_info) = node_infos.get(&channel_data.name) {
                        let rest_transform = node_info.rest_transform;
                        let rest_rot = Quat::from_mat3(&Mat3::from_cols(
                            Vec3 { x: rest_transform.a1, y: rest_transform.b1, z: rest_transform.c1 },
                            Vec3 { x: rest_transform.a2, y: rest_transform.b2, z: rest_transform.c2 },
                            Vec3 { x: rest_transform.a3, y: rest_transform.b3, z: rest_transform.c3 },
                        ));
                        let relative_rot = rest_rot.inverse() * Quat::from_xyzw(value.x, value.y, value.z, value.w);
                        Some(relative_rot)
                    } else {
                        None
                    }
                })
                .unwrap_or(Quat::IDENTITY);

            // Convert to CFrame
            let from_glam = Mat3::from_quat(rot);
            let cframe = CFrame::new(
                Vector3::new(pos.x, pos.y, pos.z),
                Matrix3 {
                    x: Vector3 {
                        x: from_glam.x_axis.x,
                        y: from_glam.x_axis.y,
                        z: from_glam.x_axis.z,
                    },
                    y: Vector3 {
                        x: from_glam.y_axis.x,
                        y: from_glam.y_axis.y,
                        z: from_glam.y_axis.z,
                    },
                    z: Vector3 {
                        x: from_glam.z_axis.x,
                        y: from_glam.z_axis.y,
                        z: from_glam.z_axis.z,
                    },
                },
            );

            poses.push(Pose {
                name: channel_data.name.clone(),
                cframe,
            });
        }

        // Only add keyframe if it has poses
        if !poses.is_empty() {
            keyframes.push(Keyframe { time, poses });
        }
    }

    keyframes
}

/// Filter out bones that have identical poses across all keyframes
pub fn filter_identical_bone_poses(keyframes: &mut Vec<Keyframe>, epsilon: f32) {
    debug!("Before filtering poses: {} keyframes", keyframes.len());
    let mut total_poses_before = 0;
    let mut total_poses_after = 0;

    // First, collect all unique bone names
    let mut all_bone_names = HashSet::new();
    for keyframe in keyframes.iter() {
        for pose in &keyframe.poses {
            all_bone_names.insert(pose.name.clone());
        }
    }

    // Check each bone to see if it has identical poses across all keyframes
    let mut bones_to_remove = HashSet::new();
    for bone_name in &all_bone_names {
        let mut bone_poses: Vec<&CFrame> = Vec::new();

        // Collect all poses for this bone across all keyframes
        for keyframe in keyframes.iter() {
            if let Some(pose) = keyframe.poses.iter().find(|p| &p.name == bone_name) {
                bone_poses.push(&pose.cframe);
            }
        }

        // Check if all poses for this bone are identical
        if bone_poses.len() > 1 {
            let first_pose = bone_poses[0];
            let all_identical = bone_poses[1..]
                .iter()
                .all(|pose| approx_equal_cframe(first_pose, pose, epsilon));

            if all_identical {
                bones_to_remove.insert(bone_name.clone());
                debug!(
                    "Bone {} has identical poses across all keyframes, removing",
                    bone_name
                );
            }
        }
    }

    // Remove poses for bones that have identical poses across all keyframes
    for keyframe in keyframes.iter_mut() {
        total_poses_before += keyframe.poses.len();

        keyframe.poses.retain(|pose| {
            let should_keep = !bones_to_remove.contains(&pose.name);
            if !should_keep {
                debug!(
                    "Removing identical pose: {} at time {}",
                    pose.name, keyframe.time
                );
            }
            should_keep
        });

        total_poses_after += keyframe.poses.len();
    }

    // Now remove keyframes that have no poses left after filtering
    keyframes.retain(|kf| !kf.poses.is_empty());

    debug!(
        "After filtering poses: {} keyframes, {} -> {} poses",
        keyframes.len(),
        total_poses_before,
        total_poses_after
    );
}

/// Create a Roblox WeakDom KeyframeSequence from keyframes
pub fn create_keyframe_sequence_dom(
    keyframes: &[Keyframe],
    bone_infos: &HashMap<String, NodeInfo>,
) -> WeakDom {
    // Create the WeakDom with KeyframeSequence and actual Keyframe instances
    let mut kfs = WeakDom::new(InstanceBuilder::new("KeyframeSequence").with_properties([(
        "Priority",
        EnumItem {
            ty: "AnimationPriority".to_owned(),
            value: 2,
        },
    )]));

    for keyframe in keyframes {
        debug!("Creating keyframe at time: {}", keyframe.time);

        // Create a Keyframe instance for this time
        let keyframe_instance =
            InstanceBuilder::new("Keyframe").with_properties([("Time", keyframe.time as f32)]);

        let keyframe_ref = kfs.insert(kfs.root_ref(), keyframe_instance);

        // Create Pose instances with bone hierarchy
        let mut pose_refs: HashMap<String, Ref> = HashMap::new();

        // First, create all pose instances
        for pose in &keyframe.poses {
            debug!("  Creating pose for bone: {}", pose.name);

            let pose_properties: Vec<(&str, Variant)> = vec![
                ("CFrame", pose.cframe.clone().into()),
                (
                    "EasingDirection",
                    EnumItem {
                        ty: "EasingDirection".to_owned(),
                        value: 0, // In
                    }
                    .into(),
                ),
                (
                    "EasingStyle",
                    EnumItem {
                        ty: "EasingStyle".to_owned(),
                        value: 0, // Linear
                    }
                    .into(),
                ),
            ];

            let pose_instance = InstanceBuilder::new("Pose")
                .with_name(pose.name.clone())
                .with_properties(pose_properties);

            // Temporarily insert under keyframe, we'll move them later
            let pose_ref = kfs.insert(keyframe_ref, pose_instance);
            pose_refs.insert(pose.name.clone(), pose_ref);
        }

        // Now organize them by hierarchy
        for pose in &keyframe.poses {
            if let Some(bone_info) = bone_infos.get(&pose.name) {
                if let Some(parent_name) = &bone_info.parent {
                    // If parent exists in this keyframe's poses, move this pose under the parent
                    if let (Some(&child_ref), Some(&parent_ref)) =
                        (pose_refs.get(&pose.name), pose_refs.get(parent_name))
                    {
                        kfs.transfer_within(child_ref, parent_ref);
                    }
                }
            }
        }
    }

    kfs
}
