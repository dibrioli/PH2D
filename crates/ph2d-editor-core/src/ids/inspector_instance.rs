//! Os ids da seção **Component** do Inspector (ADR-0164 / F5).

use super::hash_node_id;
use ph2d_a11y::NodeId;

// ⛔ **Havia um `INSP_INSTANCE_SECTION` e ele SAIU** (Enio, 2026-08-27): a superfície deixou de ser
// uma seção e passou a ser um CARTÃO no topo, que não tem cabeçalho, nem recolher, nem âncora de
// nota — logo não tem o que registar. *Um id que nada regista é um id que nada resolve.*

/// ⭐ **Limpar as excepções SEM ALVO** (F5.3).
///
/// ⚠️ **Um botão, e não uma limpeza automática:** a lei do *«unused overrides»* do Unity é que
/// elas **nunca** se apagam sozinhas — sair por causa de um `Delete` no mestre é perder trabalho
/// do artista em silêncio. ⇒ o gesto existe, e é explícito.
pub const INSP_INSTANCE_CLEAR_ORPHANS: NodeId = hash_node_id("insp_instance_clear_orphans");

/// ⭐⭐ **Quantas VARIANTES a fileira de chips endereça** (F5, critério 2).
///
/// ⚠️ **Teto de TABELA DE IDS, e ele diz de que recurso é** — o mesmo idioma do
/// [`super::chrome::vector_components::MAX_VARIANT_VALUES`]: o `populate` regista este intervalo
/// num laço e o roteador varre o mesmo. **Não é teto do catálogo**: uma família pode ter as
/// variantes que quiser, e as que passam daqui continuam a existir e a funcionar.
///
/// ⚠️ E o excedente **é escrito** no cartão, nunca truncado em silêncio.
pub const MAX_INSTANCE_VARIANTS: usize = 8;

/// Os chips da fileira **Variant** do cartão.
///
/// ⚠️ **Um chip por variante, e não um dropdown**: uma família tem tipicamente duas a quatro
/// versões, e a fileira mostra-as todas ao mesmo tempo — que é o que deixa o artista **ver** o
/// catálogo em vez de o abrir. É a mesma escolha que a fileira de variants do vetor.
///
/// ⚠️ **Tabela `const`, e não um hash em tempo de execução** — é o idioma dos irmãos deste
/// directório (`INSP_SAMPLE_FILTER`, `INSP_PLAYER_MODE_IDS`), e é o que deixa o censo de ids do
/// `hit_indexed_ids_are_registered` **ver** estes oito.
pub const INSP_INSTANCE_VARIANT: [NodeId; MAX_INSTANCE_VARIANTS] = [
    hash_node_id("insp_instance_variant_0"),
    hash_node_id("insp_instance_variant_1"),
    hash_node_id("insp_instance_variant_2"),
    hash_node_id("insp_instance_variant_3"),
    hash_node_id("insp_instance_variant_4"),
    hash_node_id("insp_instance_variant_5"),
    hash_node_id("insp_instance_variant_6"),
    hash_node_id("insp_instance_variant_7"),
];
