use std::time{SystemTime, UNIX_EPOCH, Duration};
//! Stage 76 Phase 3: CLI 集成测试
//! 测试 Profile 子命令、交互式性能查看器、报告导出功能

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use clap::Parser;
    use beejs::cli::commands{CliApp, SubCommand, ProfileCommand};

    /// 测试场景：Profile 命令参数解析
    mod profile_command_parsing {
        use super::*;

        #[test]
        fn test_profile_command_basic() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                    assert!(!profile.detailed);
                    assert!(!profile.interactive);
                    assert_eq!(profile.output_format, "text");
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_command_with_detailed() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--detailed"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert!(profile.detailed);
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_command_with_interactive() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--interactive"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert!(profile.interactive);
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_command_with_output_format() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--format", "json"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.output_format, "json");
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_command_with_output_dir() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--dir", "/tmp/profiles"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.output_dir, Some(PathBuf::from("/tmp/profiles")));
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_command_with_duration() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--duration", "30"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.duration, 30);
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_command_with_sampling_rate() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--sampling-rate", "1000"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.sampling_rate, 1000);
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                }
                _ => panic!("Expected Profile command"),
            }
        }
    }

    /// 测试场景：Profile 命令组合参数
    mod profile_command_combinations {
        use super::*;

        #[test]
        fn test_profile_all_options() {
            let cli: _ = CliApp::parse_from(&[
                "beejs",
                "profile",
                "test.js",
                "--detailed",
                "--interactive",
                "--format", "json",
                "--dir", "/tmp/profiles",
                "--duration", "60",
                "--sampling-rate", "500",
                "--", "--arg1", "value1"
            ]);

            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert!(profile.detailed);
                    assert!(profile.interactive);
                    assert_eq!(profile.output_format, "json");
                    assert_eq!(profile.output_dir, Some(PathBuf::from("/tmp/profiles")));
                    assert_eq!(profile.duration, 60);
                    assert_eq!(profile.sampling_rate, 500);
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                    assert_eq!(profile.args, vec!["--arg1".to_string(), "value1".to_string()]);
                }
                _ => panic!("Expected Profile command"),
            }
        }
    }

    /// 测试场景：边界条件
    mod profile_edge_cases {
        use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

        #[test]
        fn test_profile_with_zero_sampling_rate() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--sampling-rate", "0"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.sampling_rate, 0);
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_with_custom_output_format() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js", "--format", "json"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.output_format, "json");
                }
                _ => panic!("Expected Profile command"),
            }
        }

        #[test]
        fn test_profile_minimal_args() {
            let cli: _ = CliApp::parse_from(&["beejs", "profile", "test.js"]);
            match cli.command {
                Some(SubCommand::Profile(profile)) => {
                    assert_eq!(profile.script, PathBuf::from("test.js"));
                    assert_eq!(profile.args.len(), 0);
                    assert_eq!(profile.duration, 10);
                    assert_eq!(profile.sampling_rate, 100);
                    assert_eq!(profile.output_format, "text");
                }
                _ => panic!("Expected Profile command"),
            }
        }
    }
}
