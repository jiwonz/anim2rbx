//! # anim2rbx
//!
//! A library for converting animation files to Roblox KeyframeSequence format.
//!
//! This library provides functionality to:
//! - Parse animation files using Assimp
//! - Extract bone hierarchies and keyframe data
//! - Convert to Roblox-compatible KeyframeSequence format
//! - Filter and optimize animation data

use std::collections::HashMap;

use anyhow::Result;
use rbx_dom_weak::WeakDom;
use russimp::scene::Scene;

pub mod types;
pub mod converter;
pub mod utils;

pub use types::*;
pub use converter::*;

/// Main library API for converting animation files to KeyframeSequence
pub struct AnimationConverter {
    /// Whether to filter out bones with identical poses across all keyframes
    pub filter_identical_bones: bool,
    /// Epsilon value for floating-point comparisons
    pub epsilon: f32,
}

impl Default for AnimationConverter {
    fn default() -> Self {
        Self {
            filter_identical_bones: true,
            epsilon: 1e-5,
        }
    }
}

impl AnimationConverter {
    /// Create a new AnimationConverter with custom settings
    pub fn new(filter_identical_bones: bool, epsilon: f32) -> Self {
        Self {
            filter_identical_bones,
            epsilon,
        }
    }

    /// Builder method to set whether to filter identical bones
    pub fn with_filter_identical_bones(mut self, enabled: bool) -> Self {
        self.filter_identical_bones = enabled;
        self
    }

    /// Builder method to set the epsilon value for floating-point comparisons
    pub fn with_epsilon(mut self, epsilon: f32) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// Convert an animation file to keyframes
    pub fn convert_file_to_keyframes(&self, file_path: &str) -> Result<Vec<Keyframe>> {
        let scene = Scene::from_file(file_path, vec![])?;
        Ok(self.convert_scene_to_keyframes(&scene))
    }

    /// Convert an Assimp Scene to keyframes
    pub fn convert_scene_to_keyframes(&self, scene: &Scene) -> Vec<Keyframe> {
        let bone_infos = utils::get_bone_infos(scene);
        let mut keyframes = self.extract_keyframes(scene, &bone_infos);

        if self.filter_identical_bones {
            self.filter_identical_poses(&mut keyframes);
        }

        keyframes
    }

    /// Convert keyframes to a Roblox WeakDom KeyframeSequence
    pub fn keyframes_to_weakdom(&self, keyframes: &[Keyframe], bone_infos: &HashMap<String, NodeInfo>) -> WeakDom {
        converter::create_keyframe_sequence_dom(keyframes, bone_infos)
    }

    /// Convert an animation file directly to a Roblox WeakDom KeyframeSequence
    pub fn convert_file_to_weakdom(&self, file_path: &str) -> Result<WeakDom> {
        let scene = Scene::from_file(file_path, vec![])?;
        let bone_infos = utils::get_bone_infos(&scene);
        let keyframes = self.convert_scene_to_keyframes(&scene);
        Ok(self.keyframes_to_weakdom(&keyframes, &bone_infos))
    }

    fn extract_keyframes(&self, scene: &Scene, bone_infos: &HashMap<String, NodeInfo>) -> Vec<Keyframe> {
        converter::extract_keyframes_from_scene(scene, bone_infos)
    }

    fn filter_identical_poses(&self, keyframes: &mut Vec<Keyframe>) {
        converter::filter_identical_bone_poses(keyframes, self.epsilon);
    }
}
