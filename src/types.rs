//! Type definitions for animation data structures

use rbx_types::CFrame;
use russimp::Matrix4x4;

/// Information about a node in the animation hierarchy
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// The rest/bind pose transformation matrix for this node
    pub rest_transform: Matrix4x4,
    /// The name of the parent node, if any
    pub parent: Option<String>,
}

/// A pose for a specific bone at a specific time
#[derive(Debug, Clone)]
pub struct Pose {
    /// The name of the bone this pose applies to
    pub name: String,
    /// The CFrame transformation for this pose
    pub cframe: CFrame,
}

/// A keyframe containing poses for multiple bones at a specific time
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// The time of this keyframe in seconds
    pub time: f64,
    /// The poses for all animated bones at this time
    pub poses: Vec<Pose>,
}

/// Configuration options for animation conversion
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Whether to filter out bones with identical poses across all keyframes
    pub filter_identical_bones: bool,
    /// Epsilon value for floating-point comparisons
    pub epsilon: f32,
    /// Default ticks per second if not specified in the animation
    pub default_ticks_per_second: f64,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            filter_identical_bones: true,
            epsilon: 1e-5,
            default_ticks_per_second: 24.0,
        }
    }
}
