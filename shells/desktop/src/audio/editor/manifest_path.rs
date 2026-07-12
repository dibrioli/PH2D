//! Paths in a variation manifest: **relative to the manifest, and portable across machines.**
//!
//! A variation set is a list of clips plus a manifest that names them. Written naively, that
//! manifest records where the clips were *on the machine that saved it* — an absolute path,
//! with that machine's home directory and that OS's separator baked in. It works exactly once,
//! on exactly that computer.
//!
//! Which is not how this project is used. PH2D is deliberately **multi-machine** — the Mac
//! tests, the Linux box develops, the Windows box builds, and GitHub is the single source
//! (`docs/DevOps/MULTI_MACHINE_SETUP.md`). A manifest committed from Linux and opened on the
//! Mac would find nothing at `/home/enio/...`; one written on Windows would carry `\` into a
//! file the Mac then reads as part of a filename. The asset would be *silently* unavailable —
//! the entry survives, the row still paints, and only the audition is mysteriously dead.
//!
//! So two rules, and they are what make a manifest a portable artefact rather than a local one:
//!
//! 1. **Relative to the manifest's own directory.** Move the folder, mail the folder, commit the
//!    folder — the set still resolves, because it never referred to anything outside itself.
//! 2. **Forward slashes on the wire.** `/` is what a Unix path uses and what Windows *accepts*;
//!    `\` is what only Windows accepts. Writing `/` and converting on read is the one convention
//!    all three machines agree on.
//!
//! A clip that genuinely lives outside the manifest's tree (another volume, say) keeps its
//! absolute path — a `..` chain across a drive letter is not portable either, and pretending
//! otherwise would be worse than being honest about it.

use std::path::{Component, Path, PathBuf};

/// Rewrite `target` as a path relative to `base` (the manifest's directory), using `/` as the
/// separator.
///
/// Falls back to the absolute path when the two share no root — which on Windows means
/// different drives, and there is no relative path between those to be had.
pub(crate) fn to_manifest(target: &Path, base: &Path) -> String {
    let (t, b) = (normalise(target), normalise(base));
    if t.iter().next() != b.iter().next() {
        // Different roots (a different drive on Windows): no relative path exists.
        return slashes(&t);
    }
    let mut tc = t.components().peekable();
    let mut bc = b.components().peekable();
    // Walk off the shared prefix.
    while let (Some(a), Some(c)) = (tc.peek(), bc.peek()) {
        if a == c {
            tc.next();
            bc.next();
        } else {
            break;
        }
    }
    // Whatever is left of the base is how far UP we have to climb.
    let mut out = PathBuf::new();
    for _ in bc {
        out.push("..");
    }
    for c in tc {
        out.push(c);
    }
    if out.as_os_str().is_empty() {
        return String::new();
    }
    slashes(&out)
}

/// Resolve a manifest path back to something openable: relative entries hang off the
/// manifest's directory, absolute ones are taken as they are.
pub(crate) fn from_manifest(entry: &str, base: &Path) -> PathBuf {
    // A manifest written on Windows may carry `\`; a Unix filename may legitimately contain
    // one. Rule 2 says the wire format is `/`, so a `\` here is a separator from a Windows
    // machine, and treating it as such is what makes that file readable on this one.
    let cleaned = entry.replace('\\', "/");
    let p = Path::new(&cleaned);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base.join(p)
    }
}

/// Strip `.` components so a path compares cleanly. (`..` is left alone: it means something.)
fn normalise(p: &Path) -> PathBuf {
    p.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect()
}

/// The wire form: `/`, whatever this OS prefers.
fn slashes(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clip next to its manifest is named by its filename, and nothing else. This is the
    /// common case — the folder you mail to someone, or commit.
    #[test]
    fn a_clip_beside_the_manifest_is_just_its_name() {
        let base = Path::new("/home/enio/sfx");
        let target = Path::new("/home/enio/sfx/step_01.wav");
        assert_eq!(to_manifest(target, base), "step_01.wav");
    }

    /// A clip in a subfolder keeps the subfolder, and nothing above it.
    #[test]
    fn a_clip_below_the_manifest_keeps_only_what_is_below() {
        let base = Path::new("/home/enio/sfx");
        let target = Path::new("/home/enio/sfx/footsteps/gravel/step_01.wav");
        assert_eq!(to_manifest(target, base), "footsteps/gravel/step_01.wav");
    }

    /// A clip *beside* the manifest's folder climbs out with `..` — still relative, still
    /// portable, because everything is anchored to the manifest rather than to a machine.
    #[test]
    fn a_clip_outside_the_folder_climbs_out_relatively() {
        let base = Path::new("/home/enio/project/sets");
        let target = Path::new("/home/enio/project/audio/step_01.wav");
        assert_eq!(to_manifest(target, base), "../audio/step_01.wav");
    }

    /// **The round trip is what actually matters.** Save on one machine, load on another: the
    /// manifest is relative, the folder moved, and the clip is still found.
    #[test]
    fn the_set_survives_being_moved_to_another_machine() {
        // Saved on the Linux box…
        let saved_base = Path::new("/home/enio/Projects/game/sfx");
        let clip = Path::new("/home/enio/Projects/game/sfx/impacts/hit_03.wav");
        let wire = to_manifest(clip, saved_base);
        assert_eq!(
            wire, "impacts/hit_03.wav",
            "the manifest is not machine-free"
        );

        // …and opened on the Mac, where the whole folder lives somewhere else entirely.
        let mac_base = Path::new("/Users/enio/Desktop/game/sfx");
        let found = from_manifest(&wire, mac_base);
        assert_eq!(
            found,
            Path::new("/Users/enio/Desktop/game/sfx/impacts/hit_03.wav"),
            "the clip did not follow its manifest to the new machine"
        );
    }

    /// A manifest written on **Windows** reads on Unix: `\` is a separator on the wire, and
    /// `/` is the convention, so the backslashes are translated rather than swallowed into a
    /// filename.
    #[test]
    fn a_windows_manifest_reads_on_unix() {
        let base = Path::new("/home/enio/sfx");
        let found = from_manifest("footsteps\\gravel\\step_01.wav", base);
        assert_eq!(
            found,
            Path::new("/home/enio/sfx/footsteps/gravel/step_01.wav"),
            "the Windows separators were taken as part of the filename"
        );
    }

    /// An absolute entry stays absolute — a clip on another volume has no honest relative path,
    /// and inventing one would break it on the machine that saved it too.
    #[test]
    fn an_absolute_entry_is_left_alone() {
        let base = Path::new("/home/enio/sfx");
        let found = from_manifest("/mnt/library/rain.wav", base);
        assert_eq!(found, Path::new("/mnt/library/rain.wav"));
    }
}
