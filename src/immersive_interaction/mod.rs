//! 沉浸式交互系统模块
//!
//! 提供手部追踪、眼动追踪、触觉反馈、语音识别和动作捕捉功能。
pub mod hand_tracking;
pub mod eye_tracking;
pub mod haptic_feedback;
pub mod voice_recognition;
pub mod motion_capture;
pub use hand_tracking::{HandTracking, HandTrackingConfig, HandPose, Gesture, MockHandData};
pub use eye_tracking::{EyeTracking, EyeTrackingConfig, GazePoint, FoveatedRegion};
pub use haptic_feedback::{HapticFeedback, HapticConfig, HapticPattern, HapticIntensity};
pub use voice_recognition::{VoiceRecognition, VoiceConfig, SpeechResult, VoiceCommand};
pub use motion_capture::{MotionCapture, MotionConfig, BodyPose, JointPosition};
use std::collections::{HashMap, BTreeMap};