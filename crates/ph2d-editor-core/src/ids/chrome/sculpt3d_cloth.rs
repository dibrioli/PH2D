//! ⭐⭐⭐ **OS IDS DO TECIDO** — o pincel de tecido, o filtro de tecido e os dois
//! controlos que o report de 2026-09-08 pediu.
//!
//! Irmão (`#[path]`) do [`super::sculpt3d`], cortado por ASSUNTO. ⚠️ **O `pub use
//! sculpt3d::*` do `chrome/mod.rs` continua a ser a única porta** — nenhum
//! caminho de chamador muda, e `ids::SCULPT3D_CLOTH_MASS` resolve como sempre.
//!
//! ⚠️ **A família do tecido é a maior deste painel e a mais nova** (`23` ids
//! entre chips, pistas e interruptores), e é a única cuja pergunta de
//! visibilidade se parte em DUAS: os do PINCEL aparecem com o verbo `Cloth` na
//! mão, os do FILTRO com a **lei** de tecido escolhida. *Duas perguntas
//! diferentes sobre a mesma palavra é exactamente o que merece um ficheiro.*
//!
//! ⚠️ **O gatilho do corte foi o `architecture_workspace_file_loc_cap`** (`732`
//! de um tecto de `700`), e ele estava **latente**: mora em
//! `ph2d-editor-core/tests/`, e nenhum fechamento por `--bins` o alcança.

use super::super::{NodeId, hash_node_id};

/// **Como o pincel de TECIDO deforma** — `ph2d_sculpt3d::ClothMode::ALL`.
///
/// ⚠️ **O tamanho se CONTA, não se escolhe**, pela mesma razão do falloff acima:
/// o seam `the_panel_offers_every_cloth_mode_the_engine_has` compara este array
/// com o `ALL` do motor, então um modo novo que não passe por aqui nasce
/// inalcançável e o gate fica vermelho em vez de o chip sumir em silêncio.
///
/// ⚠️ **A fileira só é desenhada com o verbo Cloth na mão** — os outros 22
/// verbos não têm o que dizer sobre ela.
pub const SCULPT3D_CLOTH_MODE: [NodeId; 8] = [
    hash_node_id("sculpt3d.cloth_mode.0"),
    hash_node_id("sculpt3d.cloth_mode.1"),
    hash_node_id("sculpt3d.cloth_mode.2"),
    hash_node_id("sculpt3d.cloth_mode.3"),
    hash_node_id("sculpt3d.cloth_mode.4"),
    hash_node_id("sculpt3d.cloth_mode.5"),
    hash_node_id("sculpt3d.cloth_mode.6"),
    hash_node_id("sculpt3d.cloth_mode.7"),
];
/// **Que pedaço da malha entra na simulação** — `ph2d_sculpt3d::ClothArea::ALL`.
pub const SCULPT3D_CLOTH_AREA: [NodeId; 3] = [
    hash_node_id("sculpt3d.cloth_area.0"),
    hash_node_id("sculpt3d.cloth_area.1"),
    hash_node_id("sculpt3d.cloth_area.2"),
];
/// **A FORMA ESPACIAL do peso da força do tecido** —
/// `ph2d_sculpt3d::ClothForceFalloff::ALL`. ⚠️ O tamanho se CONTA: o censo
/// `the_panel_offers_every_cloth_knob_the_engine_has` compara-o com o `ALL`.
pub const SCULPT3D_CLOTH_FORCE_FALLOFF: [NodeId; 2] = [
    hash_node_id("sculpt3d.cloth_force_falloff.0"),
    hash_node_id("sculpt3d.cloth_force_falloff.1"),
];
/// ***Pin Simulation Boundary*** — só existe na área *Local* (espec §2.3).
pub const SCULPT3D_CLOTH_PIN: NodeId = hash_node_id("sculpt3d.cloth_pin");
/// ***Use Collisions*** — o pano pára nas outras peças da cena (espec §5.6).
pub const SCULPT3D_CLOTH_COLLISIONS: NodeId = hash_node_id("sculpt3d.cloth_collisions");
/// ***Persistent*** — a construção lê a base congelada (espec §6.4).
pub const SCULPT3D_CLOTH_PERSISTENT: NodeId = hash_node_id("sculpt3d.cloth_persistent");
/// ***Set Persistent Base*** — congela as posições de agora.
///
/// ⚠️ **Ele é um COMANDO e o vizinho é um interruptor**, e os dois existem
/// porque nenhum basta: ligar a opção sem base gravada é um no-op exacto, e
/// gravar a base sem a opção ligada não muda nada. *Um controlo sozinho aqui
/// seria um botão que o artista carrega e não vê acontecer nada.*
pub const SCULPT3D_CLOTH_SET_BASE: NodeId = hash_node_id("sculpt3d.cloth_set_base");
/// ⭐⭐ ***Collisions*** do FILTRO de tecido — o pano bate nas outras peças.
///
/// ⚠️ **Nasce desligada, e o preço é a razão**: `2,6×` a `6,1×` o custo de um
/// dab, e no filtro a peça inteira é o pior caso.
pub const SCULPT3D_CFILTER_COLLISIONS: NodeId = hash_node_id("sculpt3d.cfilter_collisions");
/// ⭐⭐ ***Force Axis*** do filtro de tecido — os três interruptores `X`/`Y`/`Z`
/// que **só a Escala** lê (espec §7).
///
/// ⚠️ **Um array e não três constantes soltas**: o índice `i` nomeia o eixo `i`,
/// e é essa unificação que impede um interruptor rotulado `Y` de escrever o `Z`
/// — o defeito que os sete chips do `FilterKind` já pagaram uma vez.
pub const SCULPT3D_CFILTER_AXIS: [NodeId; 3] = [
    hash_node_id("sculpt3d.cfilter_axis_x"),
    hash_node_id("sculpt3d.cfilter_axis_y"),
    hash_node_id("sculpt3d.cfilter_axis_z"),
];
/// ⭐⭐⭐ ***Cloth Quality*** do PINCEL — quantas varreduras de relaxação por passo.
///
/// ⚠️ **O alvo FIXA isto em `5` e não o oferece.** Ver
/// [`ph2d_sculpt3d::ClothFilterProps::sweeps`] para a tabela medida do que ele
/// compra e do teto.
pub const SCULPT3D_CLOTH_SWEEPS: NodeId = hash_node_id("sculpt3d.cloth_sweeps");
/// Chip ligado a [`SCULPT3D_CLOTH_SWEEPS`].
pub const SCULPT3D_CLOTH_SWEEPS_NUM: NodeId = hash_node_id("sculpt3d.cloth_sweeps_num");
/// ⭐⭐ ***Mass* do FILTRO de tecido** — dele, e não do pincel.
pub const SCULPT3D_CFILTER_MASS: NodeId = hash_node_id("sculpt3d.cfilter_mass");
/// Chip ligado a [`SCULPT3D_CFILTER_MASS`].
pub const SCULPT3D_CFILTER_MASS_NUM: NodeId = hash_node_id("sculpt3d.cfilter_mass_num");
/// ⭐⭐ ***Damping* do FILTRO** — ⚠️ omissão `0` e faixa a começar em `0`, ao
/// contrário da do pincel (`0,01`), que o filtro nunca alcançava.
pub const SCULPT3D_CFILTER_DAMPING: NodeId = hash_node_id("sculpt3d.cfilter_damping");
/// Chip ligado a [`SCULPT3D_CFILTER_DAMPING`].
pub const SCULPT3D_CFILTER_DAMPING_NUM: NodeId = hash_node_id("sculpt3d.cfilter_damping_num");
/// ⭐⭐ ***Plasticity* do FILTRO** — ⚠️ o alvo fixa-a em `0`; aqui é controlo.
pub const SCULPT3D_CFILTER_PLASTICITY: NodeId = hash_node_id("sculpt3d.cfilter_plasticity");
/// Chip ligado a [`SCULPT3D_CFILTER_PLASTICITY`].
pub const SCULPT3D_CFILTER_PLASTICITY_NUM: NodeId = hash_node_id("sculpt3d.cfilter_plasticity_num");
/// ⭐⭐⭐ ***Quality* do FILTRO** — as varreduras que o alvo fixa em `5`.
pub const SCULPT3D_CFILTER_SWEEPS: NodeId = hash_node_id("sculpt3d.cfilter_sweeps");
/// Chip ligado a [`SCULPT3D_CFILTER_SWEEPS`].
pub const SCULPT3D_CFILTER_SWEEPS_NUM: NodeId = hash_node_id("sculpt3d.cfilter_sweeps_num");
/// ⭐⭐⭐ ***Stretch Limit* do FILTRO** — quanto o pano estica antes de trancar.
/// ⚠️ O alvo não tem este número: sem ele o pano é um elástico sem fim.
pub const SCULPT3D_CFILTER_STRETCH: NodeId = hash_node_id("sculpt3d.cfilter_stretch");
/// Chip ligado a [`SCULPT3D_CFILTER_STRETCH`].
pub const SCULPT3D_CFILTER_STRETCH_NUM: NodeId = hash_node_id("sculpt3d.cfilter_stretch_num");
/// ⭐⭐⭐ ***Preserve Volume* do FILTRO** — a peça mantém o volume que tinha.
/// ⚠️ Só liga em peça FECHADA, e cancela a Escala e o Inflate por construção.
pub const SCULPT3D_CFILTER_VOLUME: NodeId = hash_node_id("sculpt3d.cfilter_volume");
/// Chip ligado a [`SCULPT3D_CFILTER_VOLUME`].
pub const SCULPT3D_CFILTER_VOLUME_NUM: NodeId = hash_node_id("sculpt3d.cfilter_volume_num");
/// ***Simulation Limit* `L`** — quantos raios a área simulada alcança.
pub const SCULPT3D_CLOTH_LIMIT: NodeId = hash_node_id("sculpt3d.cloth_limit");
/// Chip ligado a [`SCULPT3D_CLOTH_LIMIT`].
pub const SCULPT3D_CLOTH_LIMIT_NUM: NodeId = hash_node_id("sculpt3d.cloth_limit_num");
/// ***Simulation Falloff* `F`** — onde, dentro do limite, a banda começa.
pub const SCULPT3D_CLOTH_FALLOFF: NodeId = hash_node_id("sculpt3d.cloth_falloff");
/// Chip ligado a [`SCULPT3D_CLOTH_FALLOFF`].
pub const SCULPT3D_CLOTH_FALLOFF_NUM: NodeId = hash_node_id("sculpt3d.cloth_falloff_num");
/// ***Cloth Mass*** — ganho inverso sobre o passo de tempo.
pub const SCULPT3D_CLOTH_MASS: NodeId = hash_node_id("sculpt3d.cloth_mass");
/// Chip ligado a [`SCULPT3D_CLOTH_MASS`].
pub const SCULPT3D_CLOTH_MASS_NUM: NodeId = hash_node_id("sculpt3d.cloth_mass_num");
/// ***Cloth Damping*** — a fracção de velocidade perdida por passo.
pub const SCULPT3D_CLOTH_DAMPING: NodeId = hash_node_id("sculpt3d.cloth_damping");
/// Chip ligado a [`SCULPT3D_CLOTH_DAMPING`].
pub const SCULPT3D_CLOTH_DAMPING_NUM: NodeId = hash_node_id("sculpt3d.cloth_damping_num");
/// ***Soft Body Plasticity* `ρ`** — quanto a forma se lembra do que foi deformado.
pub const SCULPT3D_CLOTH_PLASTICITY: NodeId = hash_node_id("sculpt3d.cloth_plasticity");
/// Chip ligado a [`SCULPT3D_CLOTH_PLASTICITY`].
pub const SCULPT3D_CLOTH_PLASTICITY_NUM: NodeId = hash_node_id("sculpt3d.cloth_plasticity_num");
