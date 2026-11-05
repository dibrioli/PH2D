//! `PH2D_SFX_SMOKE=1` — **o caminho até o jogo do SFX Lab** (W1). O nível 2, com as moedas a tocar UM objeto
//! de três sons bem diferentes, o Audio Listener no jogador e um toque de Autoplay que só soa no Play.
//!
//! As moedas apontam para um objeto que fica: uma moeda sai da cena ao ser pega, e o sorteio dela sairia junto.

use ph2d_ecs::{
    AudioSource2D, Entity, Name, PickStrategy, SignalActions, SignalVerb, SimWorld, SoundEntry,
    SoundSet,
};

use crate::smoke::Portas;

/// A variável e os níveis.
pub const ROUTER: ph2d_app_host::SmokeRouter = ph2d_app_host::SmokeRouter {
    env: "PH2D_SFX_SMOKE",
    max_level: 1,
};

/// O nível do jogo de plataforma sobre o qual a cena monta.
pub const NIVEL_DO_JOGO: u32 = 2;
/// O objeto dos três sons: o que se seleciona na Hierarchy.
pub const CONJUNTO: &str = "Coin Sounds";
/// O objeto do toque de Autoplay.
pub const JINGLE: &str = "Start Jingle";
/// A variação por disparo, em semitons e em dB: o bastante para o ouvido notar sem trocar de som.
pub const VARIA_ALTURA: f32 = 2.0;
pub const VARIA_VOLUME: f32 = 2.0;
/// Os arquivos dos sons (`docs/SfxLab/ferramentas/gera_sons_smoke.py`); o nome no projeto é o do arquivo.
pub const SONS: [&str; 3] = [
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/SfxLab/arte/sons/coin_low.wav"
    ),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/SfxLab/arte/sons/coin_high.wav"
    ),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/SfxLab/arte/sons/coin_noise.wav"
    ),
];

/// O nível pedido em `get`.
#[must_use]
pub fn nivel_in(get: impl Fn(&str) -> Option<String>) -> Option<u32> {
    get(ROUTER.env)?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=ROUTER.max_level).contains(n))
}

#[must_use]
pub fn nivel() -> Option<u32> {
    static NIVEL: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *NIVEL.get_or_init(|| nivel_in(|k| std::env::var(k).ok()))
}

/// Uma fonte que se ouve igual em todo o nível: as moedas ficam longe umas das outras.
fn fonte_do_nivel(sound: String, autoplay: bool) -> AudioSource2D {
    AudioSource2D {
        sound,
        autoplay,
        max_distance: ph2d_ecs::AUDIO_MAX_DISTANCE_M,
        attenuation: 0.0,
        panning_strength: 0.0,
        max_polyphony: 4,
        ..AudioSource2D::default()
    }
}

/// Veste o nível 2 já montado: importa os três sons, cria os dois objetos e aponta as moedas.
pub fn veste(sim: &mut SimWorld, portas: Portas<'_>, jogador: Entity) {
    let nomes: Vec<String> = SONS
        .iter()
        .map(|p| {
            (portas.importa_som)(sim, std::path::Path::new(p))
                .unwrap_or_else(|m| panic!("{p}: {m}"))
        })
        .collect();

    (portas.menu.anexa)(sim, jogador.to_bits(), "ph2d::ecs::AudioListener2D")
        .unwrap_or_else(|m| panic!("Audio Listener 2D: {m}"));

    let conjunto = Entity::from_bits((portas.menu.raiz)(sim, CONJUNTO));
    sim.world_mut().entity_mut(conjunto).insert((
        fonte_do_nivel(String::new(), false),
        SoundSet {
            sounds: nomes.iter().map(SoundEntry::new).collect(),
            strategy: PickStrategy::Shuffle,
            no_repeat: true,
            pitch_jitter_semitones: VARIA_ALTURA,
            gain_jitter_db: VARIA_VOLUME,
        },
    ));

    let jingle = Entity::from_bits((portas.menu.raiz)(sim, JINGLE));
    sim.world_mut()
        .entity_mut(jingle)
        .insert(fonte_do_nivel(nomes[1].clone(), true));

    let w = sim.world_mut();
    let moedas: Vec<Entity> = w
        .query::<(Entity, &Name)>()
        .iter(w)
        .filter(|(_, n)| n.as_str().starts_with("Coin ") && n.as_str() != CONJUNTO)
        .map(|(e, _)| e)
        .collect();
    for m in moedas {
        let mut moeda = w.entity_mut(m);
        moeda.remove::<AudioSource2D>();
        if let Some(mut t) = moeda.get_mut::<SignalActions>() {
            for a in &mut t.0 {
                if a.verb == SignalVerb::PlaySound {
                    a.target = CONJUNTO.into();
                }
            }
        }
    }
}
