//! HR-7 — `editor=off` cortes 100% do código de editor.
//!
//! Verifies that a release-game build of `shells/desktop` contains
//! zero symbols matching `ph2d_tool_*`. The build runs with
//! `--no-default-features --features release-game` and we grep the
//! binary symbol table for forbidden names.
//!
//! PR 3 — Foundation. The shell crate does not yet declare a
//! `release-game` feature (introduced when tools start migrating in
//! PR 4 and the editor feature gate gets formalized). Until then this
//! test is `#[ignore]` to document the contract; it gets `un-ignored`
//! and wired into CI as part of PR 10 cleanup.
//!
//! Implementation, once unblocked:
//! 1. `cargo build -p ph2d-host-desktop --release --no-default-features --features release-game`
//! 2. Locate produced binary in `target/release/`.
//! 3. Run `nm` (Unix) / `dumpbin` (Windows) on it.
//! 4. Grep output for `ph2d_tool_`.
//! 5. Fail if any match found.
//!
//! Cross-platform handling: behind `#[cfg]` per-OS or shelling out to
//! a portable wrapper script. CI matrix gives us Linux + Mac + Win
//! coverage automatically.

#[test]
#[ignore = "PR 10 — un-ignore after release-game feature lands"]
fn release_build_has_no_tool_crate_symbols() {
    // Skeleton — see module doc for the contract this test enforces.
    // When un-ignoring:
    //   let output = std::process::Command::new("cargo")
    //       .args(["build", "-p", "ph2d-host-desktop", "--release",
    //              "--no-default-features", "--features", "release-game"])
    //       .output()
    //       .expect("cargo build should succeed");
    //   assert!(output.status.success(), "build failed: {}",
    //           String::from_utf8_lossy(&output.stderr));
    //   let bin = find_release_binary();
    //   let nm = std::process::Command::new("nm")
    //       .arg(&bin).output().expect("nm should run");
    //   let symbols = String::from_utf8_lossy(&nm.stdout);
    //   let leaks: Vec<&str> = symbols.lines()
    //       .filter(|l| l.contains("ph2d_tool_")).collect();
    //   assert!(leaks.is_empty(),
    //       "HR-7 violated: release-game binary contains tool symbols:\n{}",
    //       leaks.join("\n"));
    unreachable!("PR 10 will un-ignore and implement this test");
}
