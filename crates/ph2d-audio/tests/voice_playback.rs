//! A voice plays a known sample back at the expected values, and the render is
//! bit-for-bit deterministic given the same command sequence.

use ph2d_audio::{AudioEngine, AudioFormat, PlayParams, SampleData};

#[test]
fn plays_a_known_buffer_centered() {
    let sr = 48_000;
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(sr));

    // Mono ramp; source rate == output rate, so pitch 1 reads one source frame
    // per output frame (no resampling), gain 1, center pan.
    let src: Vec<f32> = (0..8).map(|i| i as f32 / 8.0).collect();
    let data = SampleData::from_interleaved(src.clone(), AudioFormat::mono(sr));
    engine.play(data, PlayParams::default()).unwrap();

    let mut out = vec![0.0f32; 16]; // 8 stereo frames
    renderer.render(&mut out, 8);

    // Center pan is equal-power (-3 dB per channel); mono → L == R.
    let g = std::f32::consts::FRAC_1_SQRT_2;
    for (i, &s) in src.iter().enumerate() {
        let expected = s * g;
        assert!((out[2 * i] - expected).abs() < 1e-4, "L frame {i}");
        assert!((out[2 * i + 1] - expected).abs() < 1e-4, "R frame {i}");
    }
    // 8 frames of an 8-frame sample rendered; it ends on the *next* block.
    assert_eq!(renderer.active_voices(), 1);
}

#[test]
fn gain_and_pan_scale_the_output() {
    let sr = 48_000;
    let (mut engine, mut renderer) = AudioEngine::new(AudioFormat::stereo(sr));
    let data = SampleData::from_interleaved(vec![1.0; 4], AudioFormat::mono(sr));
    // Hard-right pan → left silent, right full; gain 0.5.
    engine
        .play(
            data,
            PlayParams {
                gain: 0.5,
                pan: 1.0,
                ..PlayParams::default()
            },
        )
        .unwrap();

    let mut out = vec![0.0f32; 8];
    renderer.render(&mut out, 4);
    for f in 0..4 {
        assert!(out[2 * f].abs() < 1e-4, "hard-right → left silent");
        assert!((out[2 * f + 1] - 0.5).abs() < 1e-4, "right = sample * gain");
    }
}

#[test]
fn render_is_deterministic() {
    let render_once = || {
        let (mut e, mut r) = AudioEngine::new(AudioFormat::stereo(48_000));
        let src: Vec<f32> = (0..64)
            .map(|i| ((i * 7) % 13) as f32 / 13.0 - 0.5)
            .collect();
        let d = SampleData::from_interleaved(src, AudioFormat::mono(48_000));
        e.play(
            d,
            PlayParams {
                gain: 0.8,
                pan: 0.3,
                pitch: 1.0,
                looping: false,
                envelope: None,
                ..PlayParams::default()
            },
        )
        .unwrap();
        let mut out = vec![0.0f32; 128];
        r.render(&mut out, 64);
        out
    };
    assert_eq!(
        render_once(),
        render_once(),
        "identical inputs must yield identical output (HR-5-friendly render)"
    );
}
