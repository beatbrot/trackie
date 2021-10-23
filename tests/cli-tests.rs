use assert_cmd::Command;
use rand::Rng;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

#[test]
fn test_error_if_no_args() {
    cmd(&TestDirectory::create())
        .assert()
        .stderr(predicates::str::contains("status"))
        .stderr(predicates::str::contains("report"))
        .stderr(predicates::str::contains("start"))
        .stderr(predicates::str::contains("stop"))
        .code(2);
}

#[test]
fn test_status_output() {
    let t = TestDirectory::create();

    cmd(&t).arg("status").assert().failure();

    cmd(&t).arg("start").arg("foo").assert().success();
    assert!(t.path.exists());

    cmd(&t).arg("status").assert().success();
}

#[test]
fn test_status_format() {
    let t = TestDirectory::create();

    cmd(&t)
        .arg("status")
        .arg("--fallback")
        .arg("bar")
        .assert()
        .stdout("bar\n")
        .failure();

    cmd(&t).arg("start").arg("foo").assert().success();

    cmd(&t)
        .arg("status")
        .arg("-f")
        .arg("%p")
        .assert()
        .stdout("foo\n");
}

#[test]
fn test_report_json_output() -> Result<(), Box<dyn Error>> {
    let t = TestDirectory::create();

    cmd(&t).arg("start").arg("foo").ok()?;
    cmd(&t).arg("stop").ok()?;

    let out = String::from_utf8(cmd(&t).arg("report").arg("--json").output()?.stdout)?;
    assert!(out.contains("{"));
    assert!(out.contains("}"));
    Ok(())
}

fn cmd(td: &TestDirectory) -> Command {
    let mut r = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    r.env("TRACKIE_CONFIG", td.path.join("trackie.json"));
    r
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn create() -> Self {
        let mut r = rand::thread_rng();
        let n: u64 = r.gen();

        let path = PathBuf::from_str(".")
            .unwrap()
            .join("target")
            .join("test-data")
            .join(n.to_string());

        std::fs::create_dir_all(path.clone()).unwrap();
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.path.clone()).unwrap();
        assert!(!self.path.exists(), "Could not delete directory")
    }
}
