//! **Os ids das secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do [`super::inspector_camera`].
//!
//! # ⚠️ DUAS secções e UM canal
//!
//! A `Factory` vive em quem fabrica; a `Lifetime` e o `DestroyOutside` vivem na **receita**, que é
//! outro objecto — logo um título só mentiria a um dos dois. Mas a plumbing (o snapshot, a enum de
//! edição, o dreno) é a mesma pergunta, e por isso é uma só. Ver
//! [`ph2d_editor_core::screens::hero::InspectorFactoryInfo`].
//!
//! # ⚠️ Não há lista, e por isso não há linha aberta
//!
//! Um objecto tem **uma** fábrica, **uma** vida e **um** fora-do-ecrã. ⇒ estas secções não precisam
//! de estado de painel nenhum: leem o snapshot e pintam os campos, como a da câmera.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O NOME da receita — ⚠️ o painel nunca vê um id (a referência durável desta casa é o nome).
pub const INSP_FACTORY_RECIPE: NodeId = hash_node_id("insp_factory_recipe");
/// O sinal que faz nascer. **Vazio = nunca**, e o painel di-lo.
pub const INSP_FACTORY_ON_SIGNAL: NodeId = hash_node_id("insp_factory_on_signal");
/// O segmentado do ONDE — **um id por opção** (`Here` · `Area` · `At Tag`).
///
/// ⚠️ **A POSIÇÃO no array é a tag do clique**, como em todo segmentado desta casa: reordenar
/// faria um clique escrever outro modo — e compila.
pub const INSP_FACTORY_WHERE: [NodeId; 3] = [
    hash_node_id("insp_factory_where_here"),
    hash_node_id("insp_factory_where_area"),
    hash_node_id("insp_factory_where_tag"),
];
/// A caixa da área, em metros.
pub const INSP_FACTORY_AREA_W: NodeId = hash_node_id("insp_factory_area_w");
/// A caixa da área, em metros.
pub const INSP_FACTORY_AREA_H: NodeId = hash_node_id("insp_factory_area_h");
/// A tag dos pontos de nascimento.
pub const INSP_FACTORY_TAG: NodeId = hash_node_id("insp_factory_tag");
/// Ao acaso, em vez de em roda-viva.
pub const INSP_FACTORY_PICK_RANDOM: NodeId = hash_node_id("insp_factory_pick_random");
/// Quantas de cada vez.
pub const INSP_FACTORY_BURST: NodeId = hash_node_id("insp_factory_burst");
/// Quantas podem estar vivas ao mesmo tempo (`0` = sem limite).
pub const INSP_FACTORY_ALIVE_MAX: NodeId = hash_node_id("insp_factory_alive_max");
/// Quantas ao todo na corrida (`0` = sem limite).
pub const INSP_FACTORY_TOTAL_MAX: NodeId = hash_node_id("insp_factory_total_max");
/// O sinal publicado quando nascem cópias.
pub const INSP_FACTORY_ON_SPAWNED: NodeId = hash_node_id("insp_factory_on_spawned");
/// O sinal publicado quando ela se esgota.
pub const INSP_FACTORY_ON_EXHAUSTED: NodeId = hash_node_id("insp_factory_on_exhausted");
/// A semente do sorteio — ⚠️ explícita, porque determinismo é lei da casa.
pub const INSP_FACTORY_SEED: NodeId = hash_node_id("insp_factory_seed");
/// ⭐ **A cópia sai apontada para onde a fábrica aponta** (o gatilho, 2026-09-18).
///
/// ⚠️ **APENDADO no fim, e a posição não é estilo:** os ids desta secção são lidos pelo censo
/// de registo, e o `INSP_FACTORY_WHERE` ao lado declara por escrito que a POSIÇÃO é a tag.
pub const INSP_FACTORY_AIM: NodeId = hash_node_id("insp_factory_aim");

/// A vida da cópia, em segundos. `0` **não mata**.
pub const INSP_LIFE_SECONDS: NodeId = hash_node_id("insp_life_seconds");
/// O sinal da morte (vazio = calada).
pub const INSP_LIFE_ON_DEATH: NodeId = hash_node_id("insp_life_on_death");
/// A folga do fora-do-ecrã, em metros.
pub const INSP_LIFE_OUTSIDE_MARGIN: NodeId = hash_node_id("insp_life_outside_margin");
