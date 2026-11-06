//! `PH2D_SFX_SMOKE=1` — **o caminho até o jogo do SFX Lab** (W1). O nível 2, com as moedas a tocar UM objeto
//! de três sons bem diferentes, o Audio Listener no jogador e um toque de Autoplay que só soa no Play.
//!
//! As moedas apontam para um objeto que fica: uma moeda sai da cena ao ser pega, e o sorteio dela sairia junto.
//!
//! `PH2D_SFX_SMOKE=2` — os sons renderizados pela receita: [`receitas`].
//!
//! `PH2D_SFX_SMOKE=3` — os mesmos sons, «Crate» (com Audio Source 2D e sem som) e a aba do SFX Lab aberta em «Metal Hit».

pub mod receitas;

use ph2d_ecs::{
    AudioSource2D, Entity, Name, PickStrategy, SignalActions, SignalVerb, SimWorld, SoundEntry,
    SoundSet,
};

/// O `Import…` de um som: o caminho vira o nome no projeto.
pub type ImportaSom = fn(&mut SimWorld, &std::path::Path) -> Result<String, String>;
/// O `+` do Inspector: anexa um componente pelo nome canónico aos bits do objeto.
pub type Anexa<'a> = &'a dyn Fn(&mut SimWorld, u64, &str) -> Result<(), String>;
/// Um vazio na raiz com o nome dado; os bits.
pub type Raiz = fn(&mut SimWorld, &str) -> u64;

/// A variável e os níveis.
pub const ROUTER: ph2d_app_host::SmokeRouter = ph2d_app_host::SmokeRouter {
    env: "PH2D_SFX_SMOKE",
    max_level: 3,
};

/// O nível do jogo de plataforma sobre o qual a cena monta.
pub const NIVEL_DO_JOGO: u32 = 2;
/// O nível da cena com os sons da receita.
pub const NIVEL_RECEITAS: u32 = 2;
/// A cena da tela do SFX Lab: os sons da receita, a aba aberta (`ph2d-app-sfxlab`) e um objeto sem som.
pub const NIVEL_ABA: u32 = 3;
/// O objeto sem som da cena da aba, à espera do Use in Game.
pub const SEM_SOM: &str = "Crate";
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

/// O nível `base`, ou o do jogo quando a cena do SFX Lab foi pedida.
#[must_use]
pub fn ou_nivel(base: Option<u32>) -> Option<u32> {
    base.or(nivel().map(|_| NIVEL_DO_JOGO))
}

/// `PH2D_ANIM_SMOKE=2`: a cena de animação com um marcador na `walk` e «Advanced» aberto, onde «Frame markers» mora.
#[must_use]
pub fn marcadores_do_anim_smoke() -> Option<ph2d_ecs::SpriteAnimMarkers> {
    std::env::var("PH2D_ANIM_SMOKE")
        .is_ok_and(|v| v.trim() == "2")
        .then(|| {
            ph2d_ecs::SpriteAnimMarkers(vec![ph2d_ecs::FrameMarker {
                tag: "walk".into(),
                step: 3,
                signal: "footstep".into(),
            }])
        })
}

/// O cartão do Inspector que abre sobre o objeto que o nível devolve.
#[must_use]
pub fn cartao_que_abre() -> ph2d_editor_core::NodeId {
    if nivel().is_some() {
        ph2d_editor_core::ids::INSP_LIVE_AUDIO_SECTION
    } else {
        ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION
    }
}

/// Uma fonte que se ouve igual em todo o nível: as moedas ficam longe umas das outras.
fn fonte_do_nivel(sound: String, autoplay: bool) -> AudioSource2D {
    AudioSource2D {
        sound,
        autoplay,
        panning_strength: 0.0,
        max_polyphony: 4,
        ..ph2d_sfx_lab::ligacao::fonte_sem_queda()
    }
}

/// Veste o nível 2 já montado: importa os três sons, cria os dois objetos e aponta as moedas. Devolve o
/// objeto dos sons.
pub fn veste(
    sim: &mut SimWorld,
    importa_som: ImportaSom,
    anexa: Anexa<'_>,
    raiz: Raiz,
    jogador: Entity,
) -> Entity {
    let nomes: Vec<String> = SONS
        .iter()
        .map(|p| importa_som(sim, std::path::Path::new(p)).unwrap_or_else(|m| panic!("{p}: {m}")))
        .collect();

    anexa(sim, jogador.to_bits(), "ph2d::ecs::AudioListener2D")
        .unwrap_or_else(|m| panic!("Audio Listener 2D: {m}"));

    let conjunto = Entity::from_bits(raiz(sim, CONJUNTO));
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

    let jingle = Entity::from_bits(raiz(sim, JINGLE));
    sim.world_mut()
        .entity_mut(jingle)
        .insert(fonte_do_nivel(nomes[1].clone(), true));

    aponta_as_moedas(sim, CONJUNTO);
    conjunto
}

/// Toda moeda deixa de tocar a si mesma e passa a tocar o objeto `alvo`.
fn aponta_as_moedas(sim: &mut SimWorld, alvo: &str) {
    let w = sim.world_mut();
    let moedas: Vec<Entity> = w
        .query::<(Entity, &Name)>()
        .iter(w)
        .filter(|(_, n)| n.as_str().starts_with("Coin ") && n.as_str() != alvo)
        .map(|(e, _)| e)
        .collect();
    for m in moedas {
        let mut moeda = w.entity_mut(m);
        moeda.remove::<AudioSource2D>();
        if let Some(mut t) = moeda.get_mut::<SignalActions>() {
            for a in &mut t.0 {
                if a.verb == SignalVerb::PlaySound {
                    a.target = alvo.into();
                }
            }
        }
    }
}

/// [`veste`] quando a cena foi pedida. Sem ela, o nível fica como está e o objeto é o jogador. Devolve o
/// objeto em que o Inspector abre.
pub fn veste_se_armada(
    sim: &mut SimWorld,
    importa_som: ImportaSom,
    anexa: Anexa<'_>,
    raiz: Raiz,
    jogador: Entity,
) -> Entity {
    match nivel() {
        Some(NIVEL_RECEITAS) => receitas::veste(sim, anexa, raiz, jogador),
        Some(NIVEL_ABA) => {
            receitas::veste(sim, anexa, raiz, jogador);
            // Tem o cartão de onde se cria o som, e nada a tocar ainda; ouve-se de qualquer ponto do nível.
            let caixa = Entity::from_bits(raiz(sim, SEM_SOM));
            sim.world_mut()
                .entity_mut(caixa)
                .insert(fonte_do_nivel(String::new(), true));
            caixa
        }
        Some(_) => veste(sim, importa_som, anexa, raiz, jogador),
        None => jogador,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_nivel_le_se_pela_porta_e_fora_da_faixa_e_inerte() {
        let com = |v: &'static str| move |k: &str| (k == ROUTER.env).then(|| v.to_owned());
        assert_eq!(nivel_in(com("1")), Some(1));
        assert_eq!(nivel_in(com(" 1 ")), Some(1));
        assert_eq!(nivel_in(com("2")), Some(2));
        assert_eq!(nivel_in(com("3")), Some(3));
        assert_eq!(nivel_in(com("0")), None);
        assert_eq!(nivel_in(com("4")), None);
        assert_eq!(nivel_in(com("x")), None);
        assert_eq!(nivel_in(|_| None), None);
    }
}
