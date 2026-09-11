//! With no voices playing, the mixer must produce pure silence into whatever
//! (possibly garbage) buffer the device hands it.

use ph2d_audio::{AudioEngine, AudioFormat};

#[test]
fn no_voices_is_silence() {
    let (_engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(48_000));
    // Device buffer arrives with stale garbage; render must overwrite it fully.
    let mut out = vec![0.123f32; 512]; // 256 stereo frames
    renderer.render(&mut out, 256);

    assert!(
        out.iter().all(|&s| s.abs() < f32::EPSILON),
        "no voices → every output sample is zero"
    );
    assert_eq!(renderer.active_voices(), 0);
}
