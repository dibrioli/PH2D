//! **As receitas de fábrica:** cada uma exercita um kernel e tem um hash dourado. Elas servem de fixture e de
//! prova de ouvido; não são presets para o usuário.

use crate::{
    Body, Envelope, ModeLaw, Noise, NoiseFilter, Recipe, Recording, Source, TakeId, Tone, Wave,
};

const SEMENTE: u64 = 0x5F3;

fn corpo(law: ModeLaw, base_hz: f32, modes: u32, t60: f32) -> Body {
    Body {
        law,
        base_hz,
        modes,
        t60,
        damping: 0.0,
        rolloff: 1.0,
        contact_ms: 1.0,
        noise: 0.0,
    }
}

fn whoosh() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 0.8);
    r.add(
        1.6,
        Source::Noise(Noise {
            filter: NoiseFilter::Band,
            hz: 300.0,
            hz_end: 4_000.0,
            q: 1.2,
        }),
        Envelope::hit(0.15, 0.6, 0.04),
    );
    r
}

fn laser() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 0.45);
    r.add(
        0.4,
        Source::Tone(Tone {
            wave: Wave::Saw,
            hz: 3_200.0,
            hz_end: 180.0,
            glide: 0.25,
        }),
        Envelope::hit(0.001, 0.4, 0.02),
    );
    r
}

fn metal_hit() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 2.0);
    r.add(
        1.6,
        Source::Body(Body {
            damping: 0.02,
            rolloff: 0.6,
            contact_ms: 0.4,
            noise: 0.3,
            ..corpo(ModeLaw::Bar, 420.0, 16, 1.6)
        }),
        Envelope::default(),
    );
    r
}

fn wood_knock() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 0.5);
    r.add(
        1.8,
        Source::Body(Body {
            damping: 1.5,
            rolloff: 1.0,
            contact_ms: 1.5,
            noise: 0.2,
            ..corpo(ModeLaw::Bar, 300.0, 8, 0.18)
        }),
        Envelope::default(),
    );
    r
}

fn glass_ping() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 1.2);
    r.add(
        1.2,
        Source::Body(Body {
            damping: 0.1,
            rolloff: 0.9,
            contact_ms: 0.2,
            noise: 0.1,
            ..corpo(ModeLaw::Stretched { stretch: 2.1 }, 1_900.0, 10, 0.9)
        }),
        Envelope::default(),
    );
    r
}

fn drum_thump() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 0.8);
    r.add(
        1.6,
        Source::Body(Body {
            damping: 0.4,
            rolloff: 0.7,
            contact_ms: 4.0,
            noise: 0.15,
            ..corpo(ModeLaw::Membrane, 95.0, 16, 0.5)
        }),
        Envelope::default(),
    );
    r
}

fn fm_bell() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 2.0);
    r.add(
        0.5,
        Source::Tone(Tone {
            wave: Wave::Fm {
                ratio: 1.41,
                index: 5.0,
                index_end: 0.5,
            },
            hz: 440.0,
            hz_end: 440.0,
            glide: 1.5,
        }),
        Envelope::hit(0.002, 1.9, 0.05),
    );
    r
}

/// As quatro camadas de um golpe de jogo: estalo, corpo, cauda e peso.
fn impact() -> Recipe {
    let mut r = Recipe::new(SEMENTE, 1.2);
    r.add(
        0.35,
        Source::Noise(Noise {
            filter: NoiseFilter::High,
            hz: 3_000.0,
            hz_end: 3_000.0,
            q: 0.7,
        }),
        Envelope::hit(0.0005, 0.03, 0.005),
    );
    r.add(
        0.35,
        Source::Body(Body {
            damping: 0.05,
            rolloff: 0.8,
            contact_ms: 0.8,
            noise: 0.4,
            ..corpo(ModeLaw::Bar, 180.0, 16, 0.9)
        }),
        Envelope::default(),
    );
    r.add(
        0.2,
        Source::Noise(Noise {
            filter: NoiseFilter::Low,
            hz: 1_400.0,
            hz_end: 200.0,
            q: 0.7,
        }),
        Envelope::hit(0.002, 0.9, 0.05),
    );
    r.add(
        0.35,
        Source::Tone(Tone {
            wave: Wave::Sine,
            hz: 110.0,
            hz_end: 42.0,
            glide: 0.12,
        }),
        Envelope::hit(0.001, 0.5, 0.02),
    );
    r
}

/// As receitas que não pedem material.
#[must_use]
pub fn all() -> Vec<(&'static str, Recipe)> {
    vec![
        ("Whoosh", whoosh()),
        ("Laser", laser()),
        ("Metal Hit", metal_hit()),
        ("Wood Knock", wood_knock()),
        ("Glass Ping", glass_ping()),
        ("Drum Thump", drum_thump()),
        ("FM Bell", fm_bell()),
        ("Impact", impact()),
    ]
}

/// O nome da receita sobre o material.
pub const RECORDED: &str = "Recording Down";

/// Um material cinco semitons abaixo, com um fechamento para não terminar em estalo.
#[must_use]
pub fn recorded(take: TakeId, seconds: f32) -> Recipe {
    let mut r = Recipe::new(SEMENTE, seconds);
    r.add(
        0.8,
        Source::Recording(Recording {
            take,
            semitones: -5.0,
            offset: 0.0,
        }),
        Envelope::hit(0.001, seconds * 0.9, seconds * 0.05),
    );
    r
}
