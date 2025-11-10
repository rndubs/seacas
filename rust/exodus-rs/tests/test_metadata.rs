//! Tests for Phase 2: Metadata operations (QA and Info records)

use exodus_rs::{mode, CreateMode, CreateOptions, ExodusFile, InitParams, QaRecord};
use tempfile::NamedTempFile;

#[test]
fn test_qa_records_write_read() {
    let tmp = NamedTempFile::new().unwrap();

    // Write QA records
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "QA Records Test".into(),
            num_dim: 3,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

        let qa_records = vec![
            QaRecord {
                code_name: "MyApp".into(),
                code_version: "1.0.0".into(),
                date: "2025/01/15".into(),
                time: "10:30:00".into(),
            },
            QaRecord {
                code_name: "PostProcessor".into(),
                code_version: "2.5.1".into(),
                date: "2025/01/16".into(),
                time: "14:45:30".into(),
            },
        ];

        file.put_qa_records(&qa_records).unwrap();
    }

    // Read QA records
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let qa_records = file.qa_records().unwrap();

        assert_eq!(qa_records.len(), 2);

        assert_eq!(qa_records[0].code_name, "MyApp");
        assert_eq!(qa_records[0].code_version, "1.0.0");
        assert_eq!(qa_records[0].date, "2025/01/15");
        assert_eq!(qa_records[0].time, "10:30:00");

        assert_eq!(qa_records[1].code_name, "PostProcessor");
        assert_eq!(qa_records[1].code_version, "2.5.1");
        assert_eq!(qa_records[1].date, "2025/01/16");
        assert_eq!(qa_records[1].time, "14:45:30");
    }
}

#[test]
fn test_qa_records_empty() {
    let tmp = NamedTempFile::new().unwrap();

    // Write without QA records
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "No QA Test".into(),
            num_dim: 2,
            num_nodes: 2,
            ..Default::default()
        })
        .unwrap();
    }

    // Read QA records (should be empty)
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let qa_records = file.qa_records().unwrap();
        assert_eq!(qa_records.len(), 0);
    }
}

#[test]
fn test_qa_records_single() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Single QA Test".into(),
            num_dim: 3,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();

        let qa_records = vec![QaRecord {
            code_name: "TestApp".into(),
            code_version: "0.1.0".into(),
            date: "2025/01/01".into(),
            time: "00:00:00".into(),
        }];

        file.put_qa_records(&qa_records).unwrap();
    }

    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let qa_records = file.qa_records().unwrap();

        assert_eq!(qa_records.len(), 1);
        assert_eq!(qa_records[0].code_name, "TestApp");
        assert_eq!(qa_records[0].code_version, "0.1.0");
    }
}

#[test]
fn test_qa_records_max_length() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Max Length QA Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();

        // 32 characters exactly (max allowed)
        let max_str = "12345678901234567890123456789012";
        assert_eq!(max_str.len(), 32);

        let qa_records = vec![QaRecord {
            code_name: max_str.into(),
            code_version: max_str.into(),
            date: max_str.into(),
            time: max_str.into(),
        }];

        file.put_qa_records(&qa_records).unwrap();
    }

    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let qa_records = file.qa_records().unwrap();

        assert_eq!(qa_records.len(), 1);
        assert_eq!(qa_records[0].code_name.len(), 32);
    }
}

#[test]
fn test_qa_records_too_long() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    file.init(&InitParams {
        title: "Too Long Test".into(),
        num_dim: 2,
        num_nodes: 1,
        ..Default::default()
    })
    .unwrap();

    // 33 characters (one too many)
    let too_long = "123456789012345678901234567890123";
    assert_eq!(too_long.len(), 33);

    let qa_records = vec![QaRecord {
        code_name: too_long.into(),
        code_version: "1.0".into(),
        date: "2025/01/01".into(),
        time: "12:00:00".into(),
    }];

    let result = file.put_qa_records(&qa_records);
    assert!(result.is_err());
}

#[test]
fn test_info_records_write_read() {
    let tmp = NamedTempFile::new().unwrap();

    // Write info records
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Info Records Test".into(),
            num_dim: 3,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

        let info_records = vec![
            "Generated by mesh generator v2.0".to_string(),
            "Contact: support@example.com".to_string(),
            "Mesh quality: good".to_string(),
        ];

        file.put_info_records(&info_records).unwrap();
    }

    // Read info records
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let info_records = file.info_records().unwrap();

        assert_eq!(info_records.len(), 3);
        assert_eq!(info_records[0], "Generated by mesh generator v2.0");
        assert_eq!(info_records[1], "Contact: support@example.com");
        assert_eq!(info_records[2], "Mesh quality: good");
    }
}

#[test]
fn test_info_records_empty() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "No Info Test".into(),
            num_dim: 2,
            num_nodes: 2,
            ..Default::default()
        })
        .unwrap();
    }

    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let info_records = file.info_records().unwrap();
        assert_eq!(info_records.len(), 0);
    }
}

#[test]
fn test_info_records_max_length() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Max Length Info Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();

        // 80 characters exactly (max allowed)
        let max_line = "12345678901234567890123456789012345678901234567890123456789012345678901234567890";
        assert_eq!(max_line.len(), 80);

        let info_records = vec![max_line.to_string()];

        file.put_info_records(&info_records).unwrap();
    }

    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let info_records = file.info_records().unwrap();

        assert_eq!(info_records.len(), 1);
        assert_eq!(info_records[0].len(), 80);
    }
}

#[test]
fn test_info_records_too_long() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    file.init(&InitParams {
        title: "Too Long Info Test".into(),
        num_dim: 2,
        num_nodes: 1,
        ..Default::default()
    })
    .unwrap();

    // 81 characters (one too many)
    let too_long = "123456789012345678901234567890123456789012345678901234567890123456789012345678901";
    assert_eq!(too_long.len(), 81);

    let info_records = vec![too_long.to_string()];

    let result = file.put_info_records(&info_records);
    assert!(result.is_err());
}

#[test]
fn test_qa_and_info_combined() {
    let tmp = NamedTempFile::new().unwrap();

    // Write both QA and info records
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Combined Test".into(),
            num_dim: 3,
            num_nodes: 8,
            ..Default::default()
        })
        .unwrap();

        let qa_records = vec![QaRecord {
            code_name: "TestApp".into(),
            code_version: "1.0.0".into(),
            date: "2025/01/15".into(),
            time: "12:00:00".into(),
        }];

        let info_records = vec![
            "Test mesh file".to_string(),
            "Created for unit testing".to_string(),
        ];

        file.put_qa_records(&qa_records).unwrap();
        file.put_info_records(&info_records).unwrap();
    }

    // Read both
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        let qa_records = file.qa_records().unwrap();
        assert_eq!(qa_records.len(), 1);
        assert_eq!(qa_records[0].code_name, "TestApp");

        let info_records = file.info_records().unwrap();
        assert_eq!(info_records.len(), 2);
        assert_eq!(info_records[0], "Test mesh file");
        assert_eq!(info_records[1], "Created for unit testing");
    }
}
