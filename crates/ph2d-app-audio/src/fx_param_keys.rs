//! **As CHAVES dos parâmetros do rack** — uma `const` por palavra, partilhada por todos os efeitos
//! que a usam (`Freq`, `Mix`, `Q`…).
//!
//! ⚠️ A chave é a IDENTIDADE do parâmetro dentro de um efeito: os presets de fábrica sobrepõem
//! valores por ela (`ovr(p::CUTOFF, 90.0)`), e o painel pinta `TextKey::tr`. Antes de 2026-09-16 a
//! identidade era o próprio rótulo inglês — traduzi-lo mudaria qual parâmetro um preset toca.

use ph2d_i18n::TextKey;

pub const CUTOFF: TextKey = TextKey::new("audio.fx.param.cutoff");
pub const Q: TextKey = TextKey::new("audio.fx.param.q");
pub const FREQ: TextKey = TextKey::new("audio.fx.param.freq");
pub const GAIN: TextKey = TextKey::new("audio.fx.param.gain");
pub const DEPTH: TextKey = TextKey::new("audio.fx.param.depth");
pub const HARMONICS: TextKey = TextKey::new("audio.fx.param.harmonics");
pub const THRESHOLD: TextKey = TextKey::new("audio.fx.param.threshold");
pub const RATIO: TextKey = TextKey::new("audio.fx.param.ratio");
pub const ATTACK: TextKey = TextKey::new("audio.fx.param.attack");
pub const RELEASE: TextKey = TextKey::new("audio.fx.param.release");
pub const CEILING: TextKey = TextKey::new("audio.fx.param.ceiling");
pub const TARGET: TextKey = TextKey::new("audio.fx.param.target");
pub const AMOUNT: TextKey = TextKey::new("audio.fx.param.amount");
pub const SPEED: TextKey = TextKey::new("audio.fx.param.speed");
pub const SUSTAIN: TextKey = TextKey::new("audio.fx.param.sustain");
pub const DRIVE: TextKey = TextKey::new("audio.fx.param.drive");
pub const TONE: TextKey = TextKey::new("audio.fx.param.tone");
pub const BITS: TextKey = TextKey::new("audio.fx.param.bits");
pub const DOWNSAMPLE: TextKey = TextKey::new("audio.fx.param.downsample");
pub const WIDTH: TextKey = TextKey::new("audio.fx.param.width");
pub const DELAY: TextKey = TextKey::new("audio.fx.param.delay");
pub const RESONANCE: TextKey = TextKey::new("audio.fx.param.resonance");
pub const MIX: TextKey = TextKey::new("audio.fx.param.mix");
pub const BASE: TextKey = TextKey::new("audio.fx.param.base");
pub const SENS: TextKey = TextKey::new("audio.fx.param.sens");
pub const ROOM: TextKey = TextKey::new("audio.fx.param.room");
pub const DAMP: TextKey = TextKey::new("audio.fx.param.damp");
pub const TAIL: TextKey = TextKey::new("audio.fx.param.tail");
pub const TIME: TextKey = TextKey::new("audio.fx.param.time");
pub const FEEDBACK: TextKey = TextKey::new("audio.fx.param.feedback");
pub const RATE: TextKey = TextKey::new("audio.fx.param.rate");
pub const SMOOTH: TextKey = TextKey::new("audio.fx.param.smooth");
pub const DETUNE: TextKey = TextKey::new("audio.fx.param.detune");
pub const SEMITONES: TextKey = TextKey::new("audio.fx.param.semitones");
pub const SHIFT: TextKey = TextKey::new("audio.fx.param.shift");
pub const VOICE_1: TextKey = TextKey::new("audio.fx.param.voice_1");
pub const VOICE_2: TextKey = TextKey::new("audio.fx.param.voice_2");
pub const CARRIER: TextKey = TextKey::new("audio.fx.param.carrier");
pub const BANDS: TextKey = TextKey::new("audio.fx.param.bands");
pub const BREATH: TextKey = TextKey::new("audio.fx.param.breath");
pub const GRAIN: TextKey = TextKey::new("audio.fx.param.grain");
pub const SCATTER: TextKey = TextKey::new("audio.fx.param.scatter");
pub const PITCH: TextKey = TextKey::new("audio.fx.param.pitch");
pub const SENSITIVITY: TextKey = TextKey::new("audio.fx.param.sensitivity");
