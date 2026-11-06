//! `PH2D_SFX_SMOKE=2` — **os sons da receita, para o dono ouvir sem tela** (W2). Cada receita de fábrica vira um objeto
//! da cena: selecionar na Hierarchy e apertar Preview toca o buffer que `render` devolveu, por dentro do projeto.
//! As moedas tocam os mesmos sons em sequência, no Play.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_encode::{BitDepth, encode_wav};
use ph2d_ecs::{Entity, PickStrategy, ProjectSounds, SimWorld, SoundEntry, SoundSet};
use ph2d_sfx_dsp::render;
use ph2d_sfx_recipe::{ENGINE_V1, Recipe, SAMPLE_RATE, SfxLibrary, Take, factory, frames_of};

use crate::{Anexa, Raiz};

/// O objeto das moedas, com todos os sons em sequência.
pub const CONJUNTO: &str = "Recipe Sounds";
/// Os corpos vibrantes vêm antes: é sobre eles que o ouvido do dono decide (kill-criterion de W2).
pub const ORDEM: [&str; 9] = [
    "Metal Hit",
    "Wood Knock",
    "Glass Ping",
    "Drum Thump",
    "Impact",
    "Whoosh",
    "Laser",
    "FM Bell",
    factory::RECORDED,
];
/// O material da camada de gravação: um som sintético do smoke de W1, no papel de gravação até existir uma de verdade.
pub const MATERIAL: &str = crate::SONS[2];

/// O material em `Take`, pela porta única de decodificação.
fn material() -> Take {
    let bytes = std::fs::read(MATERIAL).unwrap_or_else(|e| panic!("{MATERIAL}: {e}"));
    let dados =
        ph2d_audio_decode::decode_any::decode(&bytes).unwrap_or_else(|e| panic!("{MATERIAL}: {e}"));
    let f = dados.format();
    Take::new(
        f.sample_rate,
        f.channels.count() as u16,
        dados.samples().to_vec(),
    )
}

/// As receitas da cena: as de fábrica e a que toca o material, na ordem de [`ORDEM`], e a biblioteca delas.
#[must_use]
pub fn biblioteca() -> (Vec<(String, Recipe)>, SfxLibrary) {
    let mut lib = SfxLibrary::default();
    let take = material();
    let segundos = take.frames() as f32 / take.rate as f32 * 2.0f32.powf(5.0 / 12.0);
    let id = lib.takes.add(take);
    let mut todas: Vec<(String, Recipe)> = factory::all()
        .into_iter()
        .map(|(n, r)| (n.to_owned(), r))
        .collect();
    todas.push((
        factory::RECORDED.to_owned(),
        factory::recorded(id, segundos),
    ));
    todas.sort_by_key(|(n, _)| ORDEM.iter().position(|o| o == n));
    for (n, r) in &todas {
        lib.recipes.insert(n.clone(), r.clone());
    }
    (todas, lib)
}

/// O buffer da receita num WAV de `f32`: o caminho de qualquer som do projeto.
#[must_use]
pub fn em_wav(amostras: Vec<f32>) -> Vec<u8> {
    let dados = SampleData::from_interleaved(
        amostras,
        AudioFormat {
            sample_rate: SAMPLE_RATE,
            channels: ChannelLayout::Mono,
        },
    );
    encode_wav(&dados, BitDepth::Float32).unwrap_or_else(|e| panic!("wav: {e}"))
}

/// Veste o nível 2: renderiza cada receita, um objeto por som, e aponta as moedas. Devolve o primeiro objeto, em
/// que o Inspector abre.
pub fn veste(sim: &mut SimWorld, anexa: Anexa<'_>, raiz: Raiz, jogador: Entity) -> Entity {
    let (receitas, lib) = biblioteca();
    anexa(sim, jogador.to_bits(), "ph2d::ecs::AudioListener2D")
        .unwrap_or_else(|m| panic!("Audio Listener 2D: {m}"));

    let mut sons = sim
        .world_mut()
        .remove_resource::<ProjectSounds>()
        .unwrap_or_default();
    let mut primeiro = None;
    let mut nomes = Vec::new();
    for (nome, r) in &receitas {
        let pcm =
            render(r, r.seed, ENGINE_V1, &lib.takes).unwrap_or_else(|e| panic!("{nome}: {e}"));
        assert_eq!(pcm.samples.len(), frames_of(r.seconds), "{nome}");
        let arquivo = sons.add(&format!("{nome}.wav"), em_wav(pcm.samples));
        let objeto = Entity::from_bits(raiz(sim, nome));
        sim.world_mut()
            .entity_mut(objeto)
            .insert(crate::fonte_do_nivel(arquivo.clone(), false));
        primeiro.get_or_insert(objeto);
        nomes.push(arquivo);
    }
    sim.world_mut().insert_resource(sons);
    sim.world_mut().insert_resource(lib);

    let conjunto = Entity::from_bits(raiz(sim, CONJUNTO));
    sim.world_mut().entity_mut(conjunto).insert((
        crate::fonte_do_nivel(String::new(), false),
        SoundSet {
            sounds: nomes.iter().map(SoundEntry::new).collect(),
            strategy: PickStrategy::Sequence,
            no_repeat: false,
            pitch_jitter_semitones: 0.0,
            gain_jitter_db: 0.0,
        },
    ));
    crate::aponta_as_moedas(sim, CONJUNTO);
    primeiro.expect("a cena tem receitas")
}
