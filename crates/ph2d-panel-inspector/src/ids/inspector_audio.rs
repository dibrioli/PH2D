//! **Os ids da secção AUDIO** (TOP-20 #4, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_timer`] e do [`super::inspector_action`].
//!
//! # ⚠️ Uma secção, DOIS corpos — e não duas secções
//!
//! Um objecto pode ter a FONTE, as ORELHAS, ou as duas. São dois componentes registados, mas uma
//! pergunta só para o artista — *«o que é que este objecto tem a ver com som?»* —, e o ADR-0166 diz
//! que o Inspector mostra **o que o objecto tem**. Duas secções fariam o caso comum (só a fonte)
//! pagar um cabeçalho vazio, e o caso das orelhas — um componente **sem campo nenhum** — teria uma
//! secção inteira para dizer uma linha.
//!
//! # ⚠️ Não há lista, e por isso não há linha aberta
//!
//! Ao contrário do `Timers` e do `SignalActions`, um objecto tem **UMA** fonte de som (ver o doc do
//! [`ph2d_ecs::AudioSource2D`]). ⇒ esta secção não precisa de estado de painel nenhum: ela lê o
//! snapshot e pinta os campos. É por isso que ela vive com as seções compartilhadas e não com as
//! que têm estado.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector_audio.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O ficheiro de som — o caminho. ⚠️ Ver o doc do [`ph2d_ecs::AudioSource2D`] para porque ele é um
/// caminho e não um id de asset.
pub const INSP_AUDIO_SOUND: NodeId = hash_node_id("insp_audio_sound");

/// `Browse…` — o diálogo de ficheiro.
///
/// ⚠️ **Ele existe porque não há `FieldKind::File`**: o campo ao lado é texto, e escrever um caminho
/// à mão não é um gesto que um artista faça. *A afordância que falta ao tipo do campo mora no botão.*
pub const INSP_AUDIO_BROWSE: NodeId = hash_node_id("insp_audio_browse");
pub const INSP_AUDIO_CREATE_SOUND: NodeId = hash_node_id("insp_audio_create_sound");

/// `Preview` — ouvir agora, sem esperar por um sinal.
///
/// ⚠️ **É o gesto que torna a secção APRENDÍVEL**: sem ele, o caminho mais curto entre *«anexei
/// isto»* e *«ouvi alguma coisa»* passa por escrever uma tabela de acções e um relógio.
pub const INSP_AUDIO_PREVIEW: NodeId = hash_node_id("insp_audio_preview");

/// `Stop` — cala o que este objecto tem a soar.
pub const INSP_AUDIO_STOP: NodeId = hash_node_id("insp_audio_stop");

/// Volume em decibéis.
pub const INSP_AUDIO_VOLUME: NodeId = hash_node_id("insp_audio_volume");

/// Multiplicador de tom.
pub const INSP_AUDIO_PITCH: NodeId = hash_node_id("insp_audio_pitch");

/// Repete para sempre.
pub const INSP_AUDIO_LOOP: NodeId = hash_node_id("insp_audio_loop");

/// Começa a tocar quando a cena abre. ⚠️ Nasce DESLIGADO — ver o doc do modelo.
pub const INSP_AUDIO_AUTOPLAY: NodeId = hash_node_id("insp_audio_autoplay");

/// A distância a partir da qual deixa de se ouvir, em metros.
pub const INSP_AUDIO_MAX_DIST: NodeId = hash_node_id("insp_audio_max_dist");

/// O expoente da queda com a distância.
pub const INSP_AUDIO_ATTENUATION: NodeId = hash_node_id("insp_audio_attenuation");

/// O raio dentro do qual o som não tem lado.
pub const INSP_AUDIO_RADIUS: NodeId = hash_node_id("insp_audio_radius");

/// Quanto do pan geométrico chega à saída.
pub const INSP_AUDIO_PANNING: NodeId = hash_node_id("insp_audio_panning");

/// Quantas vozes desta fonte podem soar ao mesmo tempo.
pub const INSP_AUDIO_POLYPHONY: NodeId = hash_node_id("insp_audio_polyphony");

/// **O barramento — o CHIP do seletor.** As entradas são [`INSP_AUDIO_BUS_OPT`].
pub const INSP_AUDIO_BUS_PICK: NodeId = hash_node_id("insp_audio_bus_pick");

/// **As entradas do seletor de barramento**, uma por `AudioBus::ALL`.
///
/// ⚠️ **A posição é a tag** — a mesma lei do [`super::inspector_action::INSP_ACTION_VERB`], e
/// reordenar isto mudaria o barramento de toda cena já gravada **sem dar erro**.
pub const INSP_AUDIO_BUS_OPT: [NodeId; 4] = [
    hash_node_id("insp_audio_bus_sfx"),
    hash_node_id("insp_audio_bus_music"),
    hash_node_id("insp_audio_bus_voice"),
    hash_node_id("insp_audio_bus_master"),
];

/// **A tabela de `N` ids de linha derivados de `base`**: uma fileira por posição.
pub(crate) const fn tabela_de_linha<const N: usize>(base: NodeId) -> [NodeId; N] {
    const PASSO: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut out = [NodeId(0); N];
    let mut i = 0;
    while i < N {
        out[i] = NodeId(base.0 ^ (i as u64 + 1).wrapping_mul(PASSO));
        i += 1;
    }
    out
}

/// **Quantas linhas de som têm id.** O recurso é a tabela compilada, nunca o motor: a lista guarda e sorteia
/// mais do que isso, e o cartão avisa do que não mostra. Sobe mudando o número: nada mais depende dele.
///
/// Medido 05/11 (`diag_o_custo_da_lista_cheia`, `--profile smoke`, mínimo de 30 pinturas do Inspector):
///
/// | linhas | pintura | altura da lista |
/// |---|---|---|
/// | 0 | 201 µs | 0 |
/// | 8 | 394 µs | 272 px |
/// | 64 | 1 557 µs | 2 176 px, duas janelas de 1040 |
///
/// Registar o painel inteiro com as tabelas: 1,1 ms, uma vez.
pub const MAX_SOUND_SET_ROWS: usize = 64;

/// **Quantos sons do projeto o «Pick» oferece** (ids das linhas do popover, que rola). O recurso é a mesma tabela
/// compilada, dentro do 1,1 ms de registo medido acima; o que passa disso o cartão avisa.
pub const MAX_SOUND_OFFER: usize = 256;

/// Ligar/desligar a linha `i`.
pub const INSP_AUDIO_SET_ON: [NodeId; MAX_SOUND_SET_ROWS] =
    tabela_de_linha(hash_node_id("insp_audio_set_on"));
/// O peso da linha `i`.
pub const INSP_AUDIO_SET_WEIGHT: [NodeId; MAX_SOUND_SET_ROWS] =
    tabela_de_linha(hash_node_id("insp_audio_set_weight"));
/// Remover a linha `i`.
pub const INSP_AUDIO_SET_DEL: [NodeId; MAX_SOUND_SET_ROWS] =
    tabela_de_linha(hash_node_id("insp_audio_set_del"));
/// O CHIP do «Pick». O store guarda o `open`; as opções vêm do snapshot.
pub const INSP_AUDIO_PICK: NodeId = hash_node_id("insp_audio_pick");
/// As linhas do popover do «Pick», por posição em `InspectorSoundSet::offer`.
pub const INSP_AUDIO_PICK_OPT: [NodeId; MAX_SOUND_OFFER] =
    tabela_de_linha(hash_node_id("insp_audio_pick_opt"));
/// O CHIP do «Mode».
pub const INSP_AUDIO_MODE: NodeId = hash_node_id("insp_audio_mode");
/// As linhas do «Mode», uma por `PickStrategy::ALL`. ⚠️ A posição é a tag.
pub const INSP_AUDIO_MODE_OPT: [NodeId; 3] = [
    hash_node_id("insp_audio_mode_random"),
    hash_node_id("insp_audio_mode_sequence"),
    hash_node_id("insp_audio_mode_shuffle"),
];
/// «No repeat».
pub const INSP_AUDIO_NO_REPEAT: NodeId = hash_node_id("insp_audio_no_repeat");
/// Variação de altura por disparo, em semitons.
pub const INSP_AUDIO_PITCH_JITTER: NodeId = hash_node_id("insp_audio_pitch_jitter");
/// Variação de volume por disparo, em dB.
pub const INSP_AUDIO_GAIN_JITTER: NodeId = hash_node_id("insp_audio_gain_jitter");
