//! **Os ids da PELE POR-WIDGET** (plano UI/UX W6.2) — irmão do [`super::vector`] pelo teto de LOC.
//!
//! O corte é por ASSUNTO: aqui mora *que widget do catálogo esta forma veste*.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;
use super::painter::fnv_node_id_runtime;

/// O cabeçalho da seção **Widget Skin**.
pub const VECTOR_SECTION_WIDGET: NodeId = hash_node_id("vector.section.widget");

/// **Wear a Widget** — veste a forma selecionada (nasce como Button).
///
/// ⚠️ Só é pintado para uma forma que **ainda não** veste: um botão que promove o já-promovido
/// seria um clique que não faz nada, e o artista aprenderia a não confiar nesta seção.
pub const VECTOR_WIDGET_WEAR: NodeId = hash_node_id("vector.widget.wear");

/// **Back to Drawing** — tira a pele e devolve a forma ao vetor. O simétrico exato do acima.
pub const VECTOR_WIDGET_REMOVE: NodeId = hash_node_id("vector.widget.remove");

/// **Bind Shape** — arma o conta-gotas que prende esta row a uma forma da cena (W8b.3).
///
/// ⚠️ Pintado **só para os tipos que dirigem** (`vec_widget_drive::bindable`): um `Button` produz
/// um evento e não um valor, e oferecer-lhe o vínculo daria um gesto que resolve e não faz nada.
pub const VECTOR_WIDGET_BIND: NodeId = hash_node_id("vector.widget.bind");

/// **Unbind** — solta a forma. Só existe quando há vínculo: um botão que solta o já-solto é o
/// clique-que-não-faz-nada que o irmão `WEAR` evita do outro lado.
pub const VECTOR_WIDGET_UNBIND: NodeId = hash_node_id("vector.widget.unbind");

/// Quantos chips de tipo a seção endereça.
///
/// ⚠️ **Teto de TABELA DE IDS, e ele diz de que recurso é** — o mesmo que o `MAX_VARIANT_VALUES`:
/// o `populate` regista os chips num laço e o roteador varre o mesmo intervalo. ⚠️ E aqui ele
/// **não pode** ficar abaixo do catálogo, ao contrário do irmão: os tipos que passassem daqui
/// ficariam **inalcançáveis** (não há um conta-gotas por trás como o Swap Main tem) — por isso o
/// gate exige `MAX_WIDGET_KINDS >= WidgetKind::ALL.len()`, e não apenas que os chips existam.
pub const MAX_WIDGET_KINDS: usize = 24;

/// **O chip do ÍCONE** — abre a lista de glifos de um `IconButton` (plano UI/UX W8b, §6.2).
///
/// ⚠️ Ele é um `Dropdown` e não uma segmentada, e a razão é o TAMANHO: são 136 glifos mais a
/// opção *Drawing*, e a segmentada dos tipos já cobra um teto de tabela de ids (`MAX_WIDGET_KINDS`)
/// por vinte e quatro. O popover é o irmão do dropdown de FONTES desta mesma janela — e pelo mesmo
/// motivo: **cada linha desenha a própria coisa**, porque um ícone se escolhe olhando.
pub const VECTOR_WIDGET_ICON_DD: NodeId = hash_node_id("vector.widget.icon_dd");

/// A linha `i` da lista de ícones aberta. `0` é **Drawing** (tirar a escolha); `1 + n` é
/// `IconId::all()[n]`.
///
/// ⚠️ Sem teto de tabela, e é por isso que ele é um dropdown: o id sai de um `format!` de runtime,
/// então a lista pode ter o tamanho do catálogo de glifos sem um `MAX_*` a decidir quantos são
/// alcançáveis. É o padrão do `vector_text_font_option_id`.
#[must_use]
pub fn vector_widget_icon_option_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.widget.iconopt.{i}"))
}

/// O chip do tipo `i` (índice em `WidgetKind::ALL`).
///
/// ⚠️ Derivado do **ÍNDICE de runtime**, nunca do código que viaja no documento: este id vive um
/// frame, e a mesma razão do `vector_variant_option_id` vale aqui.
#[must_use]
pub fn vector_widget_kind_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.widget.kind.{i}"))
}
