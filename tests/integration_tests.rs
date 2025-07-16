//! Integration tests for anim2rbx

use std::collections::{HashMap, HashSet};

use anim2rbx::AnimationConverter;
use rbx_types::{CFrame, Matrix3, Vector3};

#[test]
fn test_convert_animation_file() {
    let converter = AnimationConverter::new(true, 1e-5);

    // This would need a test animation file
    // For now, just test that the converter can be created
    assert!(converter.filter_identical_bones);
    assert_eq!(converter.epsilon, 1e-5);
}

#[test]
fn test_default_settings() {
    let converter = AnimationConverter::default();

    // Test default values
    assert!(converter.filter_identical_bones);
    assert_eq!(converter.epsilon, 1e-5);
}

#[test]
fn test_custom_settings() {
    let converter = AnimationConverter::new(false, 0.001);

    // Test custom values
    assert!(!converter.filter_identical_bones);
    assert_eq!(converter.epsilon, 0.001);
}

#[test]
fn test_builder_pattern() {
    let converter = AnimationConverter::default()
        .with_filter_identical_bones(false)
        .with_epsilon(0.01);

    // Test builder pattern values
    assert!(!converter.filter_identical_bones);
    assert_eq!(converter.epsilon, 0.01);
}

#[test]
fn test_keyframe_creation() {
    use anim2rbx::{Keyframe, Pose};
    use rbx_types::{CFrame, Matrix3, Vector3};

    let pose = Pose {
        name: "TestBone".to_string(),
        cframe: CFrame::new(
            Vector3::new(0.0, 0.0, 0.0),
            Matrix3 {
                x: Vector3::new(1.0, 0.0, 0.0),
                y: Vector3::new(0.0, 1.0, 0.0),
                z: Vector3::new(0.0, 0.0, 1.0),
            },
        ),
    };

    let keyframe = Keyframe {
        time: 0.5,
        poses: vec![pose],
    };

    assert_eq!(keyframe.time, 0.5);
    assert_eq!(keyframe.poses.len(), 1);
    assert_eq!(keyframe.poses[0].name, "TestBone");
}

#[test]
fn test_cframe_components() {
    use anim2rbx::Pose;
    use rbx_types::{CFrame, Matrix3, Vector3};

    let position = Vector3::new(1.0, 2.0, 3.0);
    let orientation = Matrix3 {
        x: Vector3::new(1.0, 0.0, 0.0),
        y: Vector3::new(0.0, 1.0, 0.0),
        z: Vector3::new(0.0, 0.0, 1.0),
    };

    let cframe = CFrame::new(position, orientation);
    let pose = Pose {
        name: "TestBone".to_string(),
        cframe,
    };

    assert_eq!(pose.cframe.position.x, 1.0);
    assert_eq!(pose.cframe.position.y, 2.0);
    assert_eq!(pose.cframe.position.z, 3.0);
    assert_eq!(pose.cframe.orientation.x.x, 1.0);
    assert_eq!(pose.cframe.orientation.y.y, 1.0);
    assert_eq!(pose.cframe.orientation.z.z, 1.0);
}

#[test]
fn test_keyframe_sorting_and_validation() {
    use anim2rbx::{Keyframe, Pose};
    use rbx_types::{CFrame, Matrix3, Vector3};

    let identity_matrix = Matrix3 {
        x: Vector3::new(1.0, 0.0, 0.0),
        y: Vector3::new(0.0, 1.0, 0.0),
        z: Vector3::new(0.0, 0.0, 1.0),
    };

    let mut keyframes = vec![
        Keyframe {
            time: 1.0,
            poses: vec![Pose {
                name: "Bone1".to_string(),
                cframe: CFrame::new(Vector3::new(1.0, 0.0, 0.0), identity_matrix),
            }],
        },
        Keyframe {
            time: 0.5,
            poses: vec![Pose {
                name: "Bone1".to_string(),
                cframe: CFrame::new(Vector3::new(0.5, 0.0, 0.0), identity_matrix),
            }],
        },
        Keyframe {
            time: 0.0,
            poses: vec![Pose {
                name: "Bone1".to_string(),
                cframe: CFrame::new(Vector3::new(0.0, 0.0, 0.0), identity_matrix),
            }],
        },
    ];

    // Test that keyframes can be sorted by time
    keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

    assert_eq!(keyframes[0].time, 0.0);
    assert_eq!(keyframes[1].time, 0.5);
    assert_eq!(keyframes[2].time, 1.0);

    // Test that poses have valid data
    for keyframe in &keyframes {
        assert!(
            !keyframe.poses.is_empty(),
            "Keyframe should have at least one pose"
        );
        for pose in &keyframe.poses {
            assert!(!pose.name.is_empty(), "Pose name should not be empty");
            // Validate that position components are finite
            assert!(pose.cframe.position.x.is_finite());
            assert!(pose.cframe.position.y.is_finite());
            assert!(pose.cframe.position.z.is_finite());
        }
    }
}

#[test]
fn test_converter_epsilon_precision() {
    let epsilon_high = 0.1;
    let epsilon_low = 0.001;

    let converter_high = AnimationConverter::new(true, epsilon_high);
    let converter_low = AnimationConverter::new(true, epsilon_low);

    assert_eq!(converter_high.epsilon, epsilon_high);
    assert_eq!(converter_low.epsilon, epsilon_low);

    // Test that different epsilon values are preserved
    assert_ne!(converter_high.epsilon, converter_low.epsilon);
}

#[test]
fn test_builder_pattern_chaining() {
    let converter = AnimationConverter::default()
        .with_filter_identical_bones(false)
        .with_epsilon(0.001)
        .with_filter_identical_bones(true) // Should overwrite previous value
        .with_epsilon(0.002); // Should overwrite previous value

    assert!(converter.filter_identical_bones);
    assert_eq!(converter.epsilon, 0.002);
}

#[test]
fn test_extreme_epsilon_values() {
    // Test very small epsilon
    let converter_tiny = AnimationConverter::new(true, f32::EPSILON);
    assert_eq!(converter_tiny.epsilon, f32::EPSILON);

    // Test larger epsilon
    let converter_large = AnimationConverter::new(false, 1.0);
    assert_eq!(converter_large.epsilon, 1.0);
    assert!(!converter_large.filter_identical_bones);
}

#[test]
fn test_multiple_poses_per_keyframe() {
    use anim2rbx::{Keyframe, Pose};
    use rbx_types::{CFrame, Matrix3, Vector3};

    let identity_matrix = Matrix3 {
        x: Vector3::new(1.0, 0.0, 0.0),
        y: Vector3::new(0.0, 1.0, 0.0),
        z: Vector3::new(0.0, 0.0, 1.0),
    };

    let poses = vec![
        Pose {
            name: "LeftArm".to_string(),
            cframe: CFrame::new(Vector3::new(-1.0, 0.0, 0.0), identity_matrix),
        },
        Pose {
            name: "RightArm".to_string(),
            cframe: CFrame::new(Vector3::new(1.0, 0.0, 0.0), identity_matrix),
        },
        Pose {
            name: "Head".to_string(),
            cframe: CFrame::new(Vector3::new(0.0, 1.0, 0.0), identity_matrix),
        },
    ];

    let keyframe = Keyframe { time: 0.0, poses };

    assert_eq!(keyframe.poses.len(), 3);
    assert_eq!(keyframe.poses[0].name, "LeftArm");
    assert_eq!(keyframe.poses[1].name, "RightArm");
    assert_eq!(keyframe.poses[2].name, "Head");

    // Verify unique bone names
    let bone_names: HashSet<_> = keyframe.poses.iter().map(|p| &p.name).collect();
    assert_eq!(bone_names.len(), 3, "All bone names should be unique");
}

#[test]
fn test_bone_info_hierarchy() {
    use anim2rbx::NodeInfo;

    let root_bone = NodeInfo {
        rest_transform: russimp::Matrix4x4 {
            a1: 1.0,
            a2: 0.0,
            a3: 0.0,
            a4: 0.0,
            b1: 0.0,
            b2: 1.0,
            b3: 0.0,
            b4: 0.0,
            c1: 0.0,
            c2: 0.0,
            c3: 1.0,
            c4: 0.0,
            d1: 0.0,
            d2: 0.0,
            d3: 0.0,
            d4: 1.0,
        },
        parent: None,
    };

    let child_bone = NodeInfo {
        rest_transform: russimp::Matrix4x4 {
            a1: 1.0,
            a2: 0.0,
            a3: 0.0,
            a4: 1.0, // Translated by 1 unit in X
            b1: 0.0,
            b2: 1.0,
            b3: 0.0,
            b4: 0.0,
            c1: 0.0,
            c2: 0.0,
            c3: 1.0,
            c4: 0.0,
            d1: 0.0,
            d2: 0.0,
            d3: 0.0,
            d4: 1.0,
        },
        parent: Some("RootBone".to_string()),
    };

    assert!(root_bone.parent.is_none());
    assert!(child_bone.parent.is_some());
    assert_eq!(child_bone.parent.as_ref().unwrap(), "RootBone");

    // Test that transformation matrices are different
    assert_ne!(root_bone.rest_transform.a4, child_bone.rest_transform.a4);
}

#[test]
fn test_invalid_file_path_handling() {
    let converter = AnimationConverter::new(true, 1e-5);

    // Test with non-existent file
    let result = converter.convert_file_to_keyframes("nonexistent_file.fbx");
    assert!(result.is_err(), "Should return error for non-existent file");

    let result = converter.convert_file_to_weakdom("nonexistent_file.fbx");
    assert!(result.is_err(), "Should return error for non-existent file");
}

#[test]
fn test_empty_keyframe_sequence() {
    use anim2rbx::Keyframe;

    let empty_keyframes: Vec<Keyframe> = vec![];
    let bone_infos = HashMap::new();

    let converter = AnimationConverter::new(true, 1e-5);
    let weakdom = converter.keyframes_to_weakdom(&empty_keyframes, &bone_infos);

    // Should create a valid WeakDom even with empty data
    assert!(weakdom.root_ref().is_some());
}

#[test]
fn test_floating_point_precision() {
    use anim2rbx::{Keyframe, Pose};
    use rbx_types::{CFrame, Matrix3, Vector3};

    // Test with high precision floating point values
    let precise_position = Vector3::new(1.23456789012345, -9.87654321098765, 0.000000123456789);

    let identity_matrix = Matrix3 {
        x: Vector3::new(1.0, 0.0, 0.0),
        y: Vector3::new(0.0, 1.0, 0.0),
        z: Vector3::new(0.0, 0.0, 1.0),
    };

    let pose = Pose {
        name: "PrecisionBone".to_string(),
        cframe: CFrame::new(precise_position, identity_matrix),
    };

    let keyframe = Keyframe {
        time: 0.123456789,
        poses: vec![pose],
    };

    // Verify precision is maintained
    assert!((keyframe.time - 0.123456789).abs() < f64::EPSILON);
    assert!((keyframe.poses[0].cframe.position.x - 1.23456789012345).abs() < f32::EPSILON);
    assert!((keyframe.poses[0].cframe.position.y + 9.87654321098765).abs() < f32::EPSILON);
}

#[test]
fn test_api_consistency() {
    // Ensure all three API patterns produce equivalent results
    let default_converter = AnimationConverter::default();
    let param_converter = AnimationConverter::new(true, 1e-5);
    let builder_converter = AnimationConverter::default()
        .with_filter_identical_bones(true)
        .with_epsilon(1e-5);

    // All should have identical settings
    assert_eq!(
        default_converter.filter_identical_bones,
        param_converter.filter_identical_bones
    );
    assert_eq!(default_converter.epsilon, param_converter.epsilon);
    assert_eq!(
        param_converter.filter_identical_bones,
        builder_converter.filter_identical_bones
    );
    assert_eq!(param_converter.epsilon, builder_converter.epsilon);
}

mod utils_tests {
    use super::*;

    #[test]
    fn test_approx_equal_vec3_exact() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(1.0, 2.0, 3.0);

        assert!(anim2rbx::utils::approx_equal_vec3(&v1, &v2, 1e-6));
        assert!(anim2rbx::utils::approx_equal_vec3(&v1, &v2, 0.0)); // Exact match should work with zero epsilon
    }

    #[test]
    fn test_approx_equal_vec3_within_epsilon() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(1.0001, 2.0001, 3.0001);

        assert!(anim2rbx::utils::approx_equal_vec3(&v1, &v2, 0.001));
        assert!(!anim2rbx::utils::approx_equal_vec3(&v1, &v2, 0.00001));
    }

    #[test]
    fn test_approx_equal_vec3_outside_epsilon() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(1.1, 2.1, 3.1);

        assert!(!anim2rbx::utils::approx_equal_vec3(&v1, &v2, 0.05));
        assert!(anim2rbx::utils::approx_equal_vec3(&v1, &v2, 0.15));
    }

    #[test]
    fn test_approx_equal_matrix3() {
        let m1 = Matrix3 {
            x: Vector3::new(1.0, 0.0, 0.0),
            y: Vector3::new(0.0, 1.0, 0.0),
            z: Vector3::new(0.0, 0.0, 1.0),
        };

        let m2 = Matrix3 {
            x: Vector3::new(1.0001, 0.0001, 0.0001),
            y: Vector3::new(0.0001, 1.0001, 0.0001),
            z: Vector3::new(0.0001, 0.0001, 1.0001),
        };

        assert!(anim2rbx::utils::approx_equal_matrix3(&m1, &m2, 0.001));
        assert!(!anim2rbx::utils::approx_equal_matrix3(&m1, &m2, 0.00001));
    }

    #[test]
    fn test_approx_equal_cframe() {
        let cframe1 = CFrame::new(
            Vector3::new(1.0, 2.0, 3.0),
            Matrix3 {
                x: Vector3::new(1.0, 0.0, 0.0),
                y: Vector3::new(0.0, 1.0, 0.0),
                z: Vector3::new(0.0, 0.0, 1.0),
            },
        );

        let cframe2 = CFrame::new(
            Vector3::new(1.0001, 2.0001, 3.0001),
            Matrix3 {
                x: Vector3::new(1.0001, 0.0001, 0.0001),
                y: Vector3::new(0.0001, 1.0001, 0.0001),
                z: Vector3::new(0.0001, 0.0001, 1.0001),
            },
        );

        assert!(anim2rbx::utils::approx_equal_cframe(
            &cframe1, &cframe2, 0.001
        ));
        assert!(!anim2rbx::utils::approx_equal_cframe(
            &cframe1, &cframe2, 0.00001
        ));
    }

    #[test]
    fn test_approx_equal_edge_cases() {
        // Test with negative values
        let v1 = Vector3::new(-1.0, -2.0, -3.0);
        let v2 = Vector3::new(-1.0001, -2.0001, -3.0001);
        assert!(anim2rbx::utils::approx_equal_vec3(&v1, &v2, 0.001));

        // Test with very small values
        let v3 = Vector3::new(1e-10, 1e-10, 1e-10);
        let v4 = Vector3::new(2e-10, 2e-10, 2e-10);
        assert!(anim2rbx::utils::approx_equal_vec3(&v3, &v4, 1e-9));

        // Test with mixed large and small values
        let v5 = Vector3::new(1000.0, 0.001, -1000.0);
        let v6 = Vector3::new(1000.1, 0.002, -1000.1);
        assert!(anim2rbx::utils::approx_equal_vec3(&v5, &v6, 0.15));
    }

    #[test]
    fn test_zero_epsilon_strict_equality() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(1.0, 2.0, 3.0);
        let v3 = Vector3::new(1.0000001, 2.0, 3.0); // Tiny difference

        assert!(anim2rbx::utils::approx_equal_vec3(&v1, &v2, 0.0));
        assert!(!anim2rbx::utils::approx_equal_vec3(&v1, &v3, 0.0));
    }
}
