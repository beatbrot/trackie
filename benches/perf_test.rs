use assert_cmd::prelude::*;
use bencher::{benchmark_group, benchmark_main, Bencher};
use rand::Rng;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;

fn empty_status_bench(c: &mut Bencher) {
    let t = TestDirectory::create();

    c.iter(|| cmd_quiet(&t).arg("status").status());
}

fn non_empty_status_bench(c: &mut Bencher) {
    let t = TestDirectory::create();

    cmd_quiet(&t).arg("status").status().unwrap();

    cmd_quiet(&t).arg("start").arg("foo").assert().success();

    c.iter(|| cmd_quiet(&t).arg("status").status());
}

benchmark_group!(
    benches,
    empty_status_bench,
    non_empty_status_bench,
);
benchmark_main!(benches);

pub fn cmd(td: &TestDirectory) -> Command {
    let mut r = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    r.env("TRACKIE_CONFIG", td.path.join("trackie.json"));
    r
}

pub fn cmd_quiet(td: &TestDirectory) -> Command {
    let mut r = cmd(&td);
    r.stdin(Stdio::null());
    r.stdout(Stdio::null());
    r
}

pub struct TestDirectory {
    pub path: PathBuf,
}

impl TestDirectory {
    pub fn create() -> Self {
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
