//! As linhas da tabela: uma por campo da tela.

use super::*;
use ph2d_tool_registry::hash_node_id;

/// Atalho das linhas da tabela: `id`, `chave`, `cartao`, `faixa`, `passo`, `scrub`, `fabrica`, `ler`, `escrever`.
macro_rules! camada {
    ($id:literal, $k:literal, $c:ident, $f:expr, $p:expr, $s:expr, $fab:expr, $ler:expr, $esc:expr) => {
        Campo {
            id: hash_node_id($id),
            chave: $k,
            cartao: Cartao::$c,
            faixa: $f,
            passo: $p,
            scrub: $s,
            fabrica: $fab,
            onde: Onde::Camada {
                ler: $ler,
                escrever: $esc,
            },
        }
    };
}

pub const CAMPOS: &[Campo] = &[
    // ── O som ──
    Campo {
        id: hash_node_id("sfxlab_duration"),
        chave: "sfxlab.card.duration_ms",
        cartao: Cartao::Som,
        faixa: (1.0, MS_MAX),
        passo: 10.0,
        scrub: 1.0,
        fabrica: 1000.0,
        onde: Onde::Som {
            ler: |r| em_ms(r.seconds),
            escrever: |r, v| r.seconds = ms(v),
        },
    },
    Campo {
        id: hash_node_id("sfxlab_seed"),
        chave: "sfxlab.card.seed",
        cartao: Cartao::Som,
        faixa: (0.0, SEMENTE_BAIXA as f64),
        passo: 1.0,
        scrub: 1.0,
        fabrica: 1.0,
        onde: Onde::Som {
            ler: |r| (r.seed & SEMENTE_BAIXA) as f64,
            escrever: |r, v| {
                r.seed = (r.seed & !SEMENTE_BAIXA) | (v.round() as u64 & SEMENTE_BAIXA)
            },
        },
    },
    // ── A camada ──
    camada!(
        "sfxlab_gain",
        "sfxlab.card.gain_db",
        Camada,
        (DB_MIN, DB_MAX),
        1.0,
        0.1,
        0.0,
        |l| Some(db_de(l.gain)),
        |l, v| l.gain = ganho_de(v)
    ),
    camada!(
        "sfxlab_start",
        "sfxlab.card.start_ms",
        Camada,
        (0.0, MS_MAX),
        10.0,
        1.0,
        0.0,
        |l| Some(em_ms(l.start)),
        |l, v| l.start = ms(v)
    ),
    // ── Noise ──
    camada!(
        "sfxlab_noise_hz",
        "sfxlab.card.hz",
        Fonte,
        HZ,
        10.0,
        1.0,
        800.0,
        |l| filtrado(l).map(|n| f64::from(n.hz)),
        |l, v| if let Some(n) = ruido_mut(l) {
            n.hz = v as f32
        }
    ),
    camada!(
        "sfxlab_noise_hz_end",
        "sfxlab.card.hz_end",
        Fonte,
        HZ,
        10.0,
        1.0,
        800.0,
        |l| filtrado(l).map(|n| f64::from(n.hz_end)),
        |l, v| if let Some(n) = ruido_mut(l) {
            n.hz_end = v as f32
        }
    ),
    camada!(
        "sfxlab_noise_q",
        "sfxlab.card.q",
        Fonte,
        (0.05, 1000.0),
        0.1,
        0.01,
        1.0,
        |l| filtrado(l).map(|n| redondo(f64::from(n.q), CEM)),
        |l, v| if let Some(n) = ruido_mut(l) {
            n.q = v as f32
        }
    ),
    // ── Tone ──
    camada!(
        "sfxlab_tone_hz",
        "sfxlab.card.hz",
        Fonte,
        HZ,
        10.0,
        1.0,
        440.0,
        |l| tom(l).map(|t| f64::from(t.hz)),
        |l, v| if let Some(t) = tom_mut(l) {
            t.hz = v as f32
        }
    ),
    camada!(
        "sfxlab_tone_hz_end",
        "sfxlab.card.hz_end",
        Fonte,
        HZ,
        10.0,
        1.0,
        440.0,
        |l| tom(l).map(|t| f64::from(t.hz_end)),
        |l, v| if let Some(t) = tom_mut(l) {
            t.hz_end = v as f32
        }
    ),
    camada!(
        "sfxlab_tone_glide",
        "sfxlab.card.glide_ms",
        Fonte,
        (0.0, MS_MAX),
        10.0,
        1.0,
        0.0,
        |l| tom(l).map(|t| em_ms(t.glide)),
        |l, v| if let Some(t) = tom_mut(l) {
            t.glide = ms(v)
        }
    ),
    camada!(
        "sfxlab_fm_ratio",
        "sfxlab.card.fm_ratio",
        Fonte,
        (0.0, 1000.0),
        0.1,
        0.01,
        2.0,
        |l| fm(l).map(|f| redondo(f64::from(f.0), CEM)),
        |l, v| if let Some(f) = fm_mut(l) {
            *f.0 = v as f32
        }
    ),
    camada!(
        "sfxlab_fm_index",
        "sfxlab.card.fm_index",
        Fonte,
        (0.0, 1000.0),
        0.1,
        0.01,
        1.0,
        |l| fm(l).map(|f| redondo(f64::from(f.1), CEM)),
        |l, v| if let Some(f) = fm_mut(l) {
            *f.1 = v as f32
        }
    ),
    camada!(
        "sfxlab_fm_index_end",
        "sfxlab.card.fm_index_end",
        Fonte,
        (0.0, 1000.0),
        0.1,
        0.01,
        1.0,
        |l| fm(l).map(|f| redondo(f64::from(f.2), CEM)),
        |l, v| if let Some(f) = fm_mut(l) {
            *f.2 = v as f32
        }
    ),
    // ── Struck body ──
    camada!(
        "sfxlab_body_stretch",
        "sfxlab.card.stretch",
        Fonte,
        (1.0, 3.0),
        0.1,
        0.01,
        1.5,
        |l| esticada(l).map(|s| redondo(f64::from(s), CEM)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.law = ModeLaw::Stretched { stretch: v as f32 }
        }
    ),
    camada!(
        "sfxlab_body_base",
        "sfxlab.card.base_hz",
        Fonte,
        (MIN_BODY_HZ as f64, MAX_HZ as f64),
        10.0,
        1.0,
        420.0,
        |l| corpo(l).map(|b| f64::from(b.base_hz)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.base_hz = v as f32
        }
    ),
    camada!(
        "sfxlab_body_modes",
        "sfxlab.card.modes",
        Fonte,
        (1.0, MODOS_MAX),
        1.0,
        0.25,
        12.0,
        |l| corpo(l).map(|b| f64::from(b.modes)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.modes = v.round() as u32
        }
    ),
    camada!(
        "sfxlab_body_decay",
        "sfxlab.card.decay_ms",
        Fonte,
        (1.0, MS_MAX),
        10.0,
        1.0,
        800.0,
        |l| corpo(l).map(|b| em_ms(b.t60)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.t60 = ms(v)
        }
    ),
    camada!(
        "sfxlab_body_damping",
        "sfxlab.card.damping",
        Fonte,
        (0.0, 1000.0),
        0.1,
        0.01,
        0.1,
        |l| corpo(l).map(|b| redondo(f64::from(b.damping), CEM)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.damping = v as f32
        }
    ),
    camada!(
        "sfxlab_body_rolloff",
        "sfxlab.card.rolloff",
        Fonte,
        (0.0, 8.0),
        0.1,
        0.01,
        0.8,
        |l| corpo(l).map(|b| redondo(f64::from(b.rolloff), CEM)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.rolloff = v as f32
        }
    ),
    camada!(
        "sfxlab_body_contact",
        "sfxlab.card.contact_ms",
        Fonte,
        (CONTATO_MIN_MS, 1000.0),
        0.1,
        0.01,
        1.0,
        |l| corpo(l).map(|b| redondo(f64::from(b.contact_ms), CEM)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.contact_ms = v as f32
        }
    ),
    camada!(
        "sfxlab_body_noise",
        "sfxlab.card.noise_pct",
        Fonte,
        (0.0, CEM),
        1.0,
        0.25,
        20.0,
        |l| corpo(l).map(|b| redondo(f64::from(b.noise) * CEM, CEM)),
        |l, v| if let Some(b) = corpo_mut(l) {
            b.noise = (v / CEM) as f32
        }
    ),
    // ── Recording ──
    camada!(
        "sfxlab_rec_semitones",
        "sfxlab.card.semitones",
        Fonte,
        (-(MAX_SEMITONES as f64), MAX_SEMITONES as f64),
        1.0,
        0.1,
        0.0,
        |l| gravacao(l).map(|g| redondo(f64::from(g.semitones), CEM)),
        |l, v| if let Some(g) = gravacao_mut(l) {
            g.semitones = v as f32
        }
    ),
    camada!(
        "sfxlab_rec_offset",
        "sfxlab.card.offset_ms",
        Fonte,
        (0.0, f32::MAX as f64),
        10.0,
        1.0,
        0.0,
        |l| gravacao(l).map(|g| em_ms(g.offset)),
        |l, v| if let Some(g) = gravacao_mut(l) {
            g.offset = ms(v)
        }
    ),
    // ── Shape ──
    camada!(
        "sfxlab_attack",
        "sfxlab.card.attack_ms",
        Forma,
        (0.0, MS_MAX),
        1.0,
        0.25,
        1.0,
        |l| hit_de(&l.envelope).map(|t| em_ms(t.0)),
        |l, v| hit_escreve(l, |t| t.0 = ms(v))
    ),
    camada!(
        "sfxlab_decay",
        "sfxlab.card.decay_ms",
        Forma,
        (0.0, MS_MAX),
        10.0,
        1.0,
        400.0,
        |l| hit_de(&l.envelope).map(|t| em_ms(t.1)),
        |l, v| hit_escreve(l, |t| t.1 = ms(v))
    ),
    camada!(
        "sfxlab_close",
        "sfxlab.card.close_ms",
        Forma,
        (0.0, MS_MAX),
        1.0,
        0.25,
        10.0,
        |l| hit_de(&l.envelope).map(|t| em_ms(t.2)),
        |l, v| hit_escreve(l, |t| t.2 = ms(v))
    ),
];
