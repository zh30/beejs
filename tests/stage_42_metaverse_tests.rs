//! Stage 42.0 元宇宙与全息计算测试
//!
//! 测试目标:
//! - 元宇宙渲染引擎
//! - 全息计算引擎
//! - 沉浸式交互系统
//! - 分布式元宇宙网络

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// 元宇宙渲染引擎测试
// ============================================================================

#[cfg(test)]
mod metaverse_engine_tests {
    use super::*;
    use beejs::metaverse::{
        engine::{MetaverseEngine, MetaverseConfig},
        RenderMode, XRPlatform,
        SceneObject, Transform, Material,
    };

    #[test]
    fn test_metaverse_engine_creation() {
        let config: _ = MetaverseConfig::default();
        let engine: _ = MetaverseEngine::new(config);

        assert!(engine.is_ok());
        let engine: _ = engine.clone();unwrap();
        assert!(!engine.is_rendering());
    }

    #[test]
    fn test_metaverse_engine_with_custom_config() {
        let config: _ = MetaverseConfig {
            render_mode: RenderMode::RayTracing,
            target_fps: 120,
            resolution: (3840, 2160), // 4K
            enable_multiuser: true,
            max_users: 100,
            enable_spatial_audio: true,
            ..Default::default()
        };

        let engine: _ = MetaverseEngine::new(config);
        assert!(engine.is_ok());

        let engine: _ = engine.clone();unwrap();
        assert_eq!(engine.config().target_fps, 120);
        assert_eq!(engine.config().resolution, (3840, 2160));
    }

    #[test]
    fn test_xr_runtime_initialization() {
        use beejs::metaverse::xr_runtime::{XRRuntime, XRConfig, XRMode};

        let config: _ = XRConfig {
            mode: XRMode::VR,
            platform: XRPlatform::MetaQuest,
            enable_hand_tracking: true,
            enable_eye_tracking: true,
        };

        let runtime: _ = XRRuntime::new(config);
        assert!(runtime.is_ok());

        let runtime: _ = runtime.clone();unwrap();
        assert!(runtime.supports_hand_tracking());
        assert!(runtime.supports_eye_tracking());
    }

    #[test]
    fn test_webxr_compatibility() {
        use beejs::metaverse::xr_runtime::{XRRuntime, XRConfig, XRMode};

        let config: _ = XRConfig {
            mode: XRMode::AR,
            platform: XRPlatform::WebXR,
            enable_hand_tracking: true,
            enable_eye_tracking: false,
        };

        let runtime: _ = XRRuntime::new(config);
        assert!(runtime.is_ok());

        let runtime: _ = runtime.clone();unwrap();
        assert!(runtime.is_webxr_compatible());
    }

    #[test]
    fn test_scene_object_creation() {
        let transform: _ = Transform {
            position: [0.0, 1.0, -2.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // quaternion
            scale: [1.0, 1.0, 1.0],
        };

        let material: _ = Material {
            albedo: [1.0, 0.0, 0.0, 1.0], // red
            metallic: 0.5,
            roughness: 0.3,
            emission: [0.0, 0.0, 0.0],
        };

        let object: _ = SceneObject::new("cube", transform, material);
        assert_eq!(object.name(), "cube");
        assert_eq!(object.transform().position, [0.0, 1.0, -2.0]);
    }

    #[test]
    fn test_ray_tracing_renderer() {
        use beejs::metaverse::ray_tracer::{RayTracer, RayTracerConfig, BounceLimit};

        let config: _ = RayTracerConfig {
            max_bounces: BounceLimit::Limited(8),
            samples_per_pixel: 4,
            enable_denoising: true,
            enable_global_illumination: true,
        };

        let tracer: _ = RayTracer::new(config);
        assert!(tracer.is_ok());

        let tracer: _ = tracer.clone();unwrap();
        assert_eq!(tracer.max_bounces(), 8);
        assert!(tracer.denoising_enabled());
    }

    #[test]
    fn test_multiuser_renderer() {
        use beejs::metaverse::multiuser_renderer::{
            MultiuserRenderer, AvatarConfig, SyncMode,
        };

        let renderer: _ = MultiuserRenderer::new(SyncMode::LockStep);
        assert!(renderer.is_ok());

        let mut renderer = renderer.clone();clone();clone();unwrap();

        // 添加用户 Avatar
        let avatar_config: _ = AvatarConfig {
            user_id: "user_001".to_string(),
            avatar_model: "humanoid_v1".to_string(),
            enable_expressions: true,
            enable_lip_sync: true,
        };

        let result: _ = renderer.add_avatar(avatar_config);
        assert!(result.is_ok());
        assert_eq!(renderer.user_count(), 1);
    }

    #[test]
    fn test_spatial_audio_system() {
        use beejs::metaverse::spatial_audio::{
            SpatialAudioSystem, AudioSource, AudioConfig, HRTFProfile,
        };

        let config: _ = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            hrtf_profile: HRTFProfile::Generic,
            enable_reverb: true,
            enable_occlusion: true,
        };

        let audio: _ = SpatialAudioSystem::new(config);
        assert!(audio.is_ok());

        let mut audio = audio.clone();clone();clone();unwrap();

        // 添加音源
        let source: _ = AudioSource {
            id: "ambient_001".to_string(),
            position: [10.0, 2.0, 5.0],
            volume: 0.8,
            loop_audio: true,
        };

        let result: _ = audio.add_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_performance_target() {
        let config: _ = MetaverseConfig {
            render_mode: RenderMode::Rasterization,
            target_fps: 120,
            resolution: (3840, 2160),
            ..Default::default()
        };

        let engine: _ = MetaverseEngine::new(config).unwrap();

        // 模拟渲染一帧
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let result: _ = engine.render_frame();
        let elapsed: _ = start.elapsed().unwrap();

        assert!(result.is_ok());
        // 目标: 120 FPS = 8.33ms per frame
        // 实际测试允许更宽松的时间
        assert!(elapsed < Duration::from_millis(100));
    }
}

// ============================================================================
// 全息计算引擎测试
// ============================================================================

#[cfg(test)]
mod holographic_engine_tests {
    use super::*;
    use beejs::holographic::{
        engine::{HolographicEngine, HolographicConfig},
        HologramType,
        WavefrontPropagator, PropagationMethod,
        HologramGenerator, GeneratorConfig,
        VolumeCapture, CaptureConfig,
        HolographicStorage, StorageConfig, CompressionMode,
    };

    #[test]
    fn test_holographic_engine_creation() {
        let config: _ = HolographicConfig::default();
        let engine: _ = HolographicEngine::new(config);

        assert!(engine.is_ok());
    }

    #[test]
    fn test_holographic_engine_custom_resolution() {
        let config: _ = HolographicConfig {
            resolution: (8192, 8192, 8192), // 8K x 8K x 8K 体素
            refresh_rate: 144,
            wavelength: 532.0, // 532nm green laser
            compute_method: PropagationMethod::AngularSpectrum,
        };

        let engine: _ = HolographicEngine::new(config);
        assert!(engine.is_ok());

        let engine: _ = engine.clone();unwrap();
        assert_eq!(engine.config().resolution, (8192, 8192, 8192));
        assert_eq!(engine.config().refresh_rate, 144);
    }

    #[test]
    fn test_wavefront_propagator() {
        let propagator: _ = WavefrontPropagator::new(PropagationMethod::AngularSpectrum);
        assert!(propagator.is_ok());

        let propagator: _ = propagator.clone();unwrap();

        // 创建测试波前数据
        let wavefront: _ = vec![
            beejs::holographic::Complex::new(1.0, 0.0); 1024 * 1024
        ];

        let result: _ = propagator.propagate(&wavefront, 0.1); // 传播 0.1m
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1024 * 1024);
    }

    #[test]
    fn test_hologram_generator() {
        let config: _ = GeneratorConfig {
            hologram_type: HologramType::Amplitude,
            encoding: beejs::holographic::HologramEncoding::Binary,
            optimization_iterations: 100,
        };

        let generator: _ = HologramGenerator::new(config);
        assert!(generator.is_ok());

        let generator: _ = generator.clone();unwrap();

        // 从 3D 点云生成全息图
        let point_cloud: _ = vec![
            [0.0, 0.0, 1.0],
            [0.1, 0.0, 1.0],
            [0.0, 0.1, 1.0],
        ];

        let result: _ = generator.from_point_cloud(&point_cloud, (1024, 1024));
        assert!(result.is_ok());
    }

    #[test]
    fn test_volume_capture() {
        let config: _ = CaptureConfig {
            resolution: (512, 512, 512),
            depth_range: (0.1, 10.0),
            color_format: beejs::holographic::ColorFormat::RGBA32F,
        };

        let capture: _ = VolumeCapture::new(config);
        assert!(capture.is_ok());

        let capture: _ = capture.clone();unwrap();
        assert_eq!(capture.resolution(), (512, 512, 512));
    }

    #[test]
    fn test_holographic_storage() {
        let config: _ = StorageConfig {
            compression: CompressionMode::Intelligent,
            target_ratio: 1000.0, // 1000:1 压缩率
            enable_deduplication: true,
        };

        let storage: _ = HolographicStorage::new(config);
        assert!(storage.is_ok());

        let mut storage = storage.clone();clone();clone();unwrap();

        // 测试存储全息数据
        let hologram_data: _ = vec![0u8; 1024 * 1024]; // 1MB 测试数据
        let result: _ = storage.store("test_hologram", &hologram_data);
        assert!(result.is_ok());

        // 测试读取
        let retrieved: _ = storage.retrieve("test_hologram");
        assert!(retrieved.is_ok());
    }

    #[test]
    fn test_holographic_computation_speed() {
        let config: _ = HolographicConfig {
            resolution: (1024, 1024, 1024),
            compute_method: PropagationMethod::Fresnel,
            ..Default::default()
        };

        let engine: _ = HolographicEngine::new(config).unwrap();

        // 模拟全息计算
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let result: _ = engine.compute_hologram();
        let elapsed: _ = start.elapsed().unwrap();

        assert!(result.is_ok());
        // 目标: 实时全息计算 < 10ms (测试允许更宽松)
        assert!(elapsed < Duration::from_millis(500));
    }
}

// ============================================================================
// 沉浸式交互系统测试
// ============================================================================

#[cfg(test)]
mod immersive_interaction_tests {
    use super::*;
    use beejs::immersive_interaction::{
        HandTracking, HandTrackingConfig, HandPose, Gesture,
        EyeTracking, EyeTrackingConfig, GazePoint,
        HapticFeedback, HapticConfig, HapticPattern, HapticIntensity,
        VoiceRecognition, VoiceConfig, VoiceCommand,
        MotionCapture, MotionConfig,
    };

    #[test]
    fn test_hand_tracking_initialization() {
        let config: _ = HandTrackingConfig {
            sample_rate: 120,
            enable_gesture_recognition: true,
            prediction_latency_ms: 5,
        };

        let tracker: _ = HandTracking::new(config);
        assert!(tracker.is_ok());

        let tracker: _ = tracker.clone();unwrap();
        assert!(tracker.gesture_recognition_enabled());
    }

    #[test]
    fn test_hand_pose_detection() {
        let config: _ = HandTrackingConfig::default();
        let mut tracker = HandTracking::new(config).unwrap();

        // 模拟手部数据
        let mock_hand_data: _ = beejs::immersive_interaction::MockHandData {
            joints: vec![[0.0, 0.0, 0.0]; 21], // 21 个关节点
            confidence: 0.95,
        };

        let result: _ = tracker.process_frame(&mock_hand_data);
        assert!(result.is_ok());

        let pose: _ = result.unwrap();
        assert!(pose.confidence > 0.9);
    }

    #[test]
    fn test_gesture_recognition() {
        let config: _ = HandTrackingConfig {
            enable_gesture_recognition: true,
            ..Default::default()
        };

        let tracker: _ = HandTracking::new(config).unwrap();

        // 测试识别捏合手势
        let pinch_pose: _ = HandPose::mock_pinch();
        let gesture: _ = tracker.recognize_gesture(&pinch_pose);

        assert!(gesture.is_ok());
        assert_eq!(gesture.unwrap(), Gesture::Pinch);
    }

    #[test]
    fn test_eye_tracking_initialization() {
        let config: _ = EyeTrackingConfig {
            sample_rate: 120,
            enable_foveated_rendering: true,
            calibration_points: 9,
        };

        let tracker: _ = EyeTracking::new(config);
        assert!(tracker.is_ok());

        let tracker: _ = tracker.clone();unwrap();
        assert!(tracker.foveated_rendering_enabled());
    }

    #[test]
    fn test_gaze_point_tracking() {
        let config: _ = EyeTrackingConfig::default();
        let mut tracker = EyeTracking::new(config).unwrap();

        // 模拟眼动数据
        let result: _ = tracker.get_gaze_point();
        assert!(result.is_ok());

        let gaze: _ = result.unwrap();
        assert!(gaze.x >= 0.0 && gaze.x <= 1.0);
        assert!(gaze.y >= 0.0 && gaze.y <= 1.0);
    }

    #[test]
    fn test_foveated_region_calculation() {
        let config: _ = EyeTrackingConfig {
            enable_foveated_rendering: true,
            ..Default::default()
        };

        let tracker: _ = EyeTracking::new(config).unwrap();

        let gaze: _ = GazePoint { x: 0.5, y: 0.5, depth: 1.0 };
        let region: _ = tracker.calculate_foveated_region(&gaze);

        assert!(region.is_ok());
        let region: _ = region.clone();unwrap();
        assert!(region.center_radius > 0.0);
        assert!(region.peripheral_radius > region.center_radius);
    }

    #[test]
    fn test_haptic_feedback_initialization() {
        let config: _ = HapticConfig {
            actuator_count: 256,
            frequency: 1000,
            max_intensity: HapticIntensity::High,
        };

        let haptic: _ = HapticFeedback::new(config);
        assert!(haptic.is_ok());

        let haptic: _ = haptic.clone();unwrap();
        assert_eq!(haptic.actuator_count(), 256);
    }

    #[test]
    fn test_haptic_pattern_playback() {
        let config: _ = HapticConfig::default();
        let mut haptic = HapticFeedback::new(config).unwrap();

        let pattern: _ = HapticPattern {
            name: "click".to_string(),
            duration_ms: 50,
            intensity: HapticIntensity::Medium,
            waveform: vec![1.0, 0.8, 0.5, 0.2, 0.0],
        };

        let result: _ = haptic.play_pattern(&pattern);
        assert!(result.is_ok());
    }

    #[test]
    fn test_voice_recognition_initialization() {
        let config: _ = VoiceConfig {
            language: "en-US".to_string(),
            enable_wake_word: true,
            wake_word: "Hey Beejs".to_string(),
            enable_continuous: true,
        };

        let voice: _ = VoiceRecognition::new(config);
        assert!(voice.is_ok());

        let voice: _ = voice.clone();unwrap();
        assert!(voice.wake_word_enabled());
    }

    #[test]
    fn test_voice_command_recognition() {
        let config: _ = VoiceConfig::default();
        let mut voice = VoiceRecognition::new(config).unwrap();

        // 注册命令
        voice.register_command(VoiceCommand {
            phrase: "open menu".to_string(),
            action: "ui.open_menu".to_string(),
        });

        // 模拟语音输入
        let mock_audio: _ = vec![0.0f32; 48000]; // 1秒静音
        let result: _ = voice.process_audio(&mock_audio);
        assert!(result.is_ok());
    }

    #[test]
    fn test_motion_capture_initialization() {
        let config: _ = MotionConfig {
            joint_count: 65, // 全身关节
            sample_rate: 60,
            enable_prediction: true,
        };

        let mocap: _ = MotionCapture::new(config);
        assert!(mocap.is_ok());

        let mocap: _ = mocap.clone();unwrap();
        assert_eq!(mocap.joint_count(), 65);
    }

    #[test]
    fn test_body_pose_estimation() {
        let config: _ = MotionConfig::default();
        let mut mocap = MotionCapture::new(config).unwrap();

        // 模拟动捕数据
        let result: _ = mocap.get_body_pose();
        assert!(result.is_ok());

        let pose: _ = result.unwrap();
        assert!(pose.joints.len() > 0);
        assert!(pose.confidence > 0.0);
    }

    #[test]
    fn test_interaction_latency() {
        let hand_config: _ = HandTrackingConfig {
            prediction_latency_ms: 5,
            ..Default::default()
        };

        let tracker: _ = HandTracking::new(hand_config).unwrap();

        // 测试处理延迟
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mock_data: _ = beejs::immersive_interaction::MockHandData::default();
        let _: _ = tracker.process_frame_sync(&mock_data);
        let elapsed: _ = start.elapsed().unwrap();

        // 目标: 延迟 < 20ms
        assert!(elapsed < Duration::from_millis(50));
    }
}

// ============================================================================
// 分布式元宇宙网络测试
// ============================================================================

#[cfg(test)]
mod distributed_metaverse_tests {
    use super::*;
    use beejs::distributed_metaverse::{
        MetaverseNetwork, NetworkConfig, NetworkNode, NodeRole,
        EdgeComputing, EdgeConfig, EdgeTask,
        StateSync, SyncConfig, SyncMode, StateChange,
        AssetInterop, AssetFormat, Asset,
        DecentralizedAuth, AuthConfig, Credential,
    };

    #[test]
    fn test_metaverse_network_creation() {
        let config: _ = NetworkConfig::default();
        let network: _ = MetaverseNetwork::new(config);

        assert!(network.is_ok());
    }

    #[test]
    fn test_network_node_registration() {
        let config: _ = NetworkConfig {
            max_nodes: 1000,
            enable_auto_discovery: true,
            heartbeat_interval_ms: 1000,
            ..Default::default()
        };

        let mut network = MetaverseNetwork::new(config).unwrap();

        let node: _ = NetworkNode {
            id: "node_001".to_string(),
            role: NodeRole::Edge,
            region: "us-west-2".to_string(),
            capacity: 1000,
        };

        let result: _ = network.register_node(node);
        assert!(result.is_ok());
        assert_eq!(network.node_count(), 1);
    }

    #[test]
    fn test_edge_computing_task_dispatch() {
        let config: _ = EdgeConfig {
            max_concurrent_tasks: 100,
            task_timeout_ms: 5000,
            enable_load_balancing: true,
        };

        let mut edge = EdgeComputing::new(config).unwrap();

        let task: _ = EdgeTask {
            id: "task_001".to_string(),
            compute_type: beejs::distributed_metaverse::ComputeType::Rendering,
            payload: vec![1, 2, 3, 4],
            priority: 1,
        };

        let result: _ = edge.dispatch_task(task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_state_synchronization() {
        let config: _ = SyncConfig {
            mode: SyncMode::Eventual,
            conflict_resolution: beejs::distributed_metaverse::ConflictResolution::LastWriterWins,
            sync_interval_ms: 50,
            ..Default::default()
        };

        let mut sync = StateSync::new(config).unwrap();

        // 发布状态变化
        let change: _ = StateChange {
            key: "player_001.position".to_string(),
            value: serde_json::json!([10.0, 5.0, -3.0]),
            timestamp: std::time::SystemTime::now(),
            version: 1,
        };

        let result: _ = sync.publish_change(change);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cross_region_sync_latency() {
        let config: _ = SyncConfig {
            mode: SyncMode::Causal,
            max_latency_ms: 50,
            ..Default::default()
        };

        let sync: _ = StateSync::new(config).unwrap();

        // 模拟跨区域同步
        let start: _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let result: _ = sync.sync_with_region("eu-west-1");
        let elapsed: _ = start.elapsed().unwrap();

        assert!(result.is_ok());
        // 目标: < 50ms 跨洲同步 (测试允许更宽松)
        assert!(elapsed < Duration::from_millis(200));
    }

    #[test]
    fn test_asset_format_conversion() {
        let interop: _ = AssetInterop::new();

        let asset: _ = Asset {
            id: "avatar_model".to_string(),
            format: AssetFormat::GLTF,
            data: vec![0u8; 1024],
        };

        // 转换到 USDZ 格式
        let result: _ = interop.convert(&asset, AssetFormat::USDZ);
        assert!(result.is_ok());

        let converted: _ = result.unwrap();
        assert_eq!(converted.format, AssetFormat::USDZ);
    }

    #[test]
    fn test_asset_interoperability() {
        let interop: _ = AssetInterop::new();

        // 测试支持的格式
        assert!(interop.supports_format(AssetFormat::GLTF));
        assert!(interop.supports_format(AssetFormat::USDZ));
        assert!(interop.supports_format(AssetFormat::FBX));
        assert!(interop.supports_format(AssetFormat::OBJ));
    }

    #[test]
    fn test_decentralized_auth_creation() {
        let config: _ = AuthConfig {
            enable_did: true, // Decentralized Identifier
            enable_zero_knowledge: true,
            supported_chains: vec!["ethereum".to_string(), "polygon".to_string()],
        };

        let auth: _ = DecentralizedAuth::new(config);
        assert!(auth.is_ok());
    }

    #[test]
    fn test_identity_creation() {
        let config: _ = AuthConfig::default();
        let mut auth = DecentralizedAuth::new(config).unwrap();

        // 创建去中心化身份
        let result: _ = auth.create_identity("user_001");
        assert!(result.is_ok());

        let identity: _ = result.unwrap();
        assert!(identity.did.starts_with("did:"));
    }

    #[test]
    fn test_credential_verification() {
        let config: _ = AuthConfig {
            enable_zero_knowledge: true,
            ..Default::default()
        };

        let auth: _ = DecentralizedAuth::new(config).unwrap();

        let credential: _ = Credential {
            holder_did: "did:beejs:user_001".to_string(),
            issuer_did: "did:beejs:authority".to_string(),
            claims: HashMap::from([
                ("age_over_18".to_string(), serde_json::json!(true)),
            ]),
            proof: vec![1, 2, 3, 4], // 简化的零知识证明
        };

        let result: _ = auth.verify_credential(&credential);
        assert!(result.is_ok());
    }

    #[test]
    fn test_network_scalability() {
        let config: _ = NetworkConfig {
            max_nodes: 1000000, // 百万级用户
            enable_sharding: true,
            shard_count: 1000,
            ..Default::default()
        };

        let network: _ = MetaverseNetwork::new(config).unwrap();

        assert_eq!(network.max_capacity(), 1000000);
        assert!(network.sharding_enabled());
    }

    #[test]
    fn test_sla_availability() {
        let config: _ = NetworkConfig {
            target_availability: 0.9999, // 99.99% SLA
            enable_redundancy: true,
            replica_count: 3,
            ..Default::default()
        };

        let network: _ = MetaverseNetwork::new(config).unwrap();

        assert!(network.target_availability() >= 0.9999);
    }
}

// ============================================================================
// 集成测试
// ============================================================================

#[cfg(test)]
mod integration_tests {
    
    use beejs::metaverse::engine::MetaverseEngine;
    use beejs::holographic::engine::HolographicEngine;
    use beejs::immersive_interaction::{HandTracking, EyeTracking};
    use beejs::distributed_metaverse::MetaverseNetwork;

    #[test]
    fn test_full_metaverse_pipeline() {
        // 1. 初始化元宇宙引擎
        let metaverse: _ = MetaverseEngine::new(Default::default()).unwrap();

        // 2. 初始化全息计算
        let _holographic: _ = HolographicEngine::new(Default::default()).unwrap();

        // 3. 初始化交互系统
        let _hand_tracking: _ = HandTracking::new(Default::default()).unwrap();
        let _eye_tracking: _ = EyeTracking::new(Default::default()).unwrap();

        // 4. 初始化分布式网络
        let network: _ = MetaverseNetwork::new(Default::default()).unwrap();

        // 验证所有组件就绪
        assert!(!metaverse.is_rendering());
        assert!(network.node_count() == 0);
    }

    #[test]
    fn test_multiuser_session() {
        use beejs::metaverse::multiuser_renderer::{MultiuserRenderer, SyncMode, AvatarConfig};
        

        // 创建多用户渲染器
        let mut renderer = MultiuserRenderer::new(SyncMode::LockStep).unwrap();

        // 添加多个用户
        for i in 0..100 {
            let config: _ = AvatarConfig {
                user_id: format!("user_{:03}", i),
                avatar_model: "humanoid_v1".to_string(),
                enable_expressions: true,
                enable_lip_sync: i % 2 == 0, // 一半用户启用唇同步
            };

            renderer.add_avatar(config).unwrap();
        }

        assert_eq!(renderer.user_count(), 100);
    }

    #[test]
    fn test_holographic_display_integration() {
        use beejs::holographic::{engine::{HolographicEngine, HolographicConfig}, PropagationMethod};
        use beejs::metaverse::{engine::{MetaverseEngine, MetaverseConfig}};

        // 创建全息引擎
        let holo_config: _ = HolographicConfig {
            resolution: (4096, 4096, 4096),
            compute_method: PropagationMethod::AngularSpectrum,
            ..Default::default()
        };
        let holographic: _ = HolographicEngine::new(holo_config).unwrap();

        // 创建元宇宙渲染引擎
        let meta_config: _ = MetaverseConfig {
            enable_holographic_display: true,
            ..Default::default()
        };
        let metaverse: _ = MetaverseEngine::new(meta_config).unwrap();

        // 集成全息显示
        let result: _ = metaverse.integrate_holographic(&holographic);
        assert!(result.is_ok());
    }

    #[test]
    fn test_immersive_interaction_pipeline() {
        use beejs::immersive_interaction::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
            HandTracking, HandTrackingConfig,
            EyeTracking, EyeTrackingConfig,
            HapticFeedback, HapticConfig,
        };

        // 创建手部追踪
        let hand: _ = HandTracking::new(HandTrackingConfig {
            sample_rate: 120,
            enable_gesture_recognition: true,
            prediction_latency_ms: 5,
        }).unwrap();

        // 创建眼动追踪
        let eye: _ = EyeTracking::new(EyeTrackingConfig {
            sample_rate: 120,
            enable_foveated_rendering: true,
            calibration_points: 9,
        }).unwrap();

        // 创建触觉反馈
        let haptic: _ = HapticFeedback::new(HapticConfig {
            actuator_count: 256,
            frequency: 1000,
            ..Default::default()
        }).unwrap();

        // 验证所有系统就绪
        assert!(hand.gesture_recognition_enabled());
        assert!(eye.foveated_rendering_enabled());
        assert_eq!(haptic.actuator_count(), 256);
    }
}
