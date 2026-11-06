//! O que nasce: o som novo e as camadas do «Add layer». Os números são o ponto de partida de cada fonte, audível de
//! primeira; quem os muda é o artista, nos cartões.

use ph2d_sfx_recipe::{
    Body, Envelope, ModeLaw, Noise, NoiseFilter, Recipe, Recording, Source, TakeId, Tone, Wave,
};

/// A duração de um som novo.
const SEGUNDOS: f32 = 1.0;
const QUEDA: f32 = 0.4;

/// O que o «Add layer» oferece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fonte {
    Noise,
    Tone,
    Body,
    Recording,
}

impl Fonte {
    pub const TODAS: [Self; 4] = [Self::Noise, Self::Tone, Self::Body, Self::Recording];
}

/// FNV-1a do nome: o mesmo nome dá a mesma semente em qualquer máquina.
#[must_use]
pub fn semente_do_nome(nome: &str) -> u64 {
    nome.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Um corpo vibrante, audível de primeira.
#[must_use]
pub fn som_novo(nome: &str) -> Recipe {
    let mut r = Recipe::new(semente_do_nome(nome), SEGUNDOS);
    acrescenta(&mut r, Fonte::Body, None);
    r
}

/// `None` só no `Recording` sem material.
pub fn acrescenta(
    r: &mut Recipe,
    fonte: Fonte,
    take: Option<TakeId>,
) -> Option<ph2d_sfx_recipe::LayerId> {
    let queda = Envelope::hit(0.001, QUEDA.min(r.seconds), 0.01);
    Some(match fonte {
        Fonte::Noise => r.add(
            0.5,
            Source::Noise(Noise {
                filter: NoiseFilter::Band,
                hz: 800.0,
                hz_end: 800.0,
                q: 1.0,
            }),
            queda,
        ),
        Fonte::Tone => r.add(
            0.5,
            Source::Tone(Tone {
                wave: Wave::Sine,
                hz: 440.0,
                hz_end: 440.0,
                glide: 0.0,
            }),
            queda,
        ),
        Fonte::Body => r.add(
            1.0,
            Source::Body(Body {
                law: ModeLaw::Bar,
                base_hz: 420.0,
                modes: 12,
                t60: 0.8,
                damping: 0.1,
                rolloff: 0.8,
                contact_ms: 1.0,
                noise: 0.2,
            }),
            queda,
        ),
        Fonte::Recording => r.add(
            0.8,
            Source::Recording(Recording {
                take: take?,
                semitones: 0.0,
                offset: 0.0,
            }),
            Envelope::default(),
        ),
    })
}
