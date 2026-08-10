//! **Os ids do AUTO LAYOUT** (plano UI/UX W2, ADR-0153) — irmão de [`super::vector_frame`] pelo
//! teto de 700 LOC, e o corte é o mesmo: aqui mora tudo o que a moldura que EMPILHA precisa.
//!
//! # Duas metades, e elas são perguntas independentes
//!
//! A seção *Layout* pinta dois blocos, e cada um responde a uma pergunta que o outro não faz:
//!
//! - **a moldura DISPÕE** (direção · vão · recuo · alinhamento) — o `VecLayout` do pai;
//! - **o filho se COMPORTA** (Grow/Shrink) — o `VecLayoutItem`.
//!
//! ⚠️ Elas **coexistem**, e é isso que uma moldura ANINHADA torna visível: ela empilha os próprios
//! filhos *e* é um item no fluxo do pai. Colapsá-las num bloco só faria a metade do item
//! desaparecer exactamente onde ela é mais útil.

use ph2d_a11y::NodeId;

use crate::ids::hash_node_id;

/// O cabeçalho da seção **Layout** (só com uma moldura ou um filho de fluxo selecionado).
pub const VECTOR_SECTION_LAYOUT: NodeId = hash_node_id("vector.section.layout");

/// **Off** — a moldura deixa de empilhar; os filhos voltam a ficar onde o artista os pôs.
///
/// ⚠️ Ele é o primeiro chip do MESMO rádio da direção, e não um interruptor à parte, porque
/// *"esta moldura flui?"* e *"em que direção?"* são **uma** pergunta (é o `display` do CSS:
/// `block` contra `flex; flex-direction: row`). Dois controlos admitiriam o estado *"não flui, mas
/// em coluna"*, que não quer dizer nada — e é o estado em que um deles fica quando alguém esquece
/// de limpar o outro.
///
/// ⚠️ **Off REMOVE o componente**, então vão/recuo/alinhamento se perdem (é o que o Figma faz ao
/// tirar o auto layout). O undo os devolve; guardá-los num componente inerte seria manter no
/// arquivo uma regra que ninguém honra.
pub const VECTOR_LAYOUT_DIR_OFF: NodeId = hash_node_id("vector.layout.dir.off");
/// Ver [`VECTOR_LAYOUT_DIR_OFF`] — em linha.
pub const VECTOR_LAYOUT_DIR_ROW: NodeId = hash_node_id("vector.layout.dir.row");
/// Ver [`VECTOR_LAYOUT_DIR_OFF`] — em coluna.
pub const VECTOR_LAYOUT_DIR_COL: NodeId = hash_node_id("vector.layout.dir.col");
/// Ver [`VECTOR_LAYOUT_DIR_OFF`] — em linha, quebrando quando não cabe.
pub const VECTOR_LAYOUT_DIR_WRAP: NodeId = hash_node_id("vector.layout.dir.wrap");
/// Ver [`VECTOR_LAYOUT_DIR_OFF`] — em **grade**, com [`VECTOR_LAYOUT_COLUMNS`] colunas iguais.
///
/// ⚠️ Ele é o 5º chip do MESMO rádio, e não um interruptor à parte, pela razão que o
/// [`VECTOR_LAYOUT_DIR_OFF`] já escreve: *"esta moldura flui, e como?"* é **uma** pergunta.
///
/// ⚠️ **A ausência dele era MEDIDA e a medição envelheceu.** Este doc dizia que a feature `grid`
/// do motor triplica o custo de build *"por um modo que nada honraria"* — a razão de build era um
/// A/B que nunca reconstruía o `taffy` (são ~0,47 s absolutos, uma vez por build limpo, 0,03% de
/// uma corrida de CI), e a metade que importava — *nada honraria* — deixou de ser verdade no
/// momento em que a fatia, o painel e o gesto passaram a honrá-lo.
pub const VECTOR_LAYOUT_DIR_GRID: NodeId = hash_node_id("vector.layout.dir.grid");

/// **Quantas colunas** — pintado só com a direção em *Grid*, porque só ela o lê.
///
/// ⚠️ Ele é a mesma cerca do vão TRANSVERSAL, que só o `Wrap` pinta: um número que o artista edita
/// e que não move um pixel é o controlo morto que esta política existe para impedir. E o VALOR
/// sobrevive à troca de direção (ele mora no `VecLayout`, não no variante), então ir a `Row` e
/// voltar devolve a grade que estava lá.
pub const VECTOR_LAYOUT_COLUMNS: NodeId = hash_node_id("vector.layout.columns");

/// O vão no eixo PRINCIPAL (entre um filho e o seguinte).
pub const VECTOR_LAYOUT_GAP_MAIN: NodeId = hash_node_id("vector.layout.gap.main");
/// O vão no eixo TRANSVERSAL — só o `Wrap` o usa (entre uma linha e a seguinte).
pub const VECTOR_LAYOUT_GAP_CROSS: NodeId = hash_node_id("vector.layout.gap.cross");

/// **Padding: All** — um campo só, que escreve os quatro lados.
///
/// ⚠️ O par All/Each **troca os campos pintados** em vez de acender um cadeado sobre quatro que
/// se movem juntos: quatro campos que espelham o mesmo número não dizem em qual se digita, e o
/// artista descobre por tentativa. É a forma do Figma.
pub const VECTOR_LAYOUT_PAD_ALL_MODE: NodeId = hash_node_id("vector.layout.pad.all_mode");
/// Ver [`VECTOR_LAYOUT_PAD_ALL_MODE`] — os quatro lados, cada um por si.
pub const VECTOR_LAYOUT_PAD_EACH_MODE: NodeId = hash_node_id("vector.layout.pad.each_mode");
/// O campo único do modo *All*.
pub const VECTOR_LAYOUT_PAD_ALL: NodeId = hash_node_id("vector.layout.pad.all");
/// Recuo do TOPO (modo *Each*).
pub const VECTOR_LAYOUT_PAD_T: NodeId = hash_node_id("vector.layout.pad.t");
/// Recuo da DIREITA (modo *Each*).
pub const VECTOR_LAYOUT_PAD_R: NodeId = hash_node_id("vector.layout.pad.r");
/// Recuo da BASE (modo *Each*).
pub const VECTOR_LAYOUT_PAD_B: NodeId = hash_node_id("vector.layout.pad.b");
/// Recuo da ESQUERDA (modo *Each*).
pub const VECTOR_LAYOUT_PAD_L: NodeId = hash_node_id("vector.layout.pad.l");

/// Alinhamento no eixo TRANSVERSAL — começo.
pub const VECTOR_LAYOUT_ALIGN_START: NodeId = hash_node_id("vector.layout.align.start");
/// Ver [`VECTOR_LAYOUT_ALIGN_START`].
pub const VECTOR_LAYOUT_ALIGN_CENTER: NodeId = hash_node_id("vector.layout.align.center");
/// Ver [`VECTOR_LAYOUT_ALIGN_START`].
pub const VECTOR_LAYOUT_ALIGN_END: NodeId = hash_node_id("vector.layout.align.end");
/// Ver [`VECTOR_LAYOUT_ALIGN_START`] — estica o filho para preencher a travessa.
pub const VECTOR_LAYOUT_ALIGN_STRETCH: NodeId = hash_node_id("vector.layout.align.stretch");

/// Distribuição no eixo PRINCIPAL — começo.
pub const VECTOR_LAYOUT_JUSTIFY_START: NodeId = hash_node_id("vector.layout.justify.start");
/// Ver [`VECTOR_LAYOUT_JUSTIFY_START`].
pub const VECTOR_LAYOUT_JUSTIFY_CENTER: NodeId = hash_node_id("vector.layout.justify.center");
/// Ver [`VECTOR_LAYOUT_JUSTIFY_START`].
pub const VECTOR_LAYOUT_JUSTIFY_END: NodeId = hash_node_id("vector.layout.justify.end");
/// Ver [`VECTOR_LAYOUT_JUSTIFY_START`] — a sobra vai para os VÃOS.
pub const VECTOR_LAYOUT_JUSTIFY_BETWEEN: NodeId = hash_node_id("vector.layout.justify.between");
/// Ver [`VECTOR_LAYOUT_JUSTIFY_START`] — a sobra vai para os vãos E para as pontas.
pub const VECTOR_LAYOUT_JUSTIFY_AROUND: NodeId = hash_node_id("vector.layout.justify.around");

/// Quanto o filho selecionado toma da SOBRA (o espaçador de uma barra de ferramentas).
pub const VECTOR_LAYOUT_ITEM_GROW: NodeId = hash_node_id("vector.layout.item.grow");
/// Quanto o filho selecionado CEDE quando falta espaço.
pub const VECTOR_LAYOUT_ITEM_SHRINK: NodeId = hash_node_id("vector.layout.item.shrink");

/// **Width: Fixed | Hug** — o par que decide se a moldura tem o tamanho que o artista desenhou ou
/// o tamanho do que está DENTRO dela (o *Hug contents* do Figma).
///
/// ⚠️ **Dois chips, e não três.** O Figma oferece *Fill* no mesmo seletor, mas *Fill* é uma relação
/// com o PAI (quanto da sobra eu tomo) e vive na linha Grow, que já existe. Juntá-los aqui daria um
/// seletor cujo terceiro chip não faz nada numa moldura de topo — e o artista descobriria por
/// tentativa.
///
/// ⚠️ Ele é o gémeo do par **Width: Auto | Fixed** que o TEXTO já tem (`vector_text.rs`): o mesmo
/// vocabulário, a mesma pergunta, dois objectos diferentes.
pub const VECTOR_LAYOUT_SIZE_W_FIXED: NodeId = hash_node_id("vector.layout.size.w.fixed");
/// Ver [`VECTOR_LAYOUT_SIZE_W_FIXED`] — a largura sai do conteúdo.
pub const VECTOR_LAYOUT_SIZE_W_HUG: NodeId = hash_node_id("vector.layout.size.w.hug");
/// Ver [`VECTOR_LAYOUT_SIZE_W_FIXED`], no outro eixo.
pub const VECTOR_LAYOUT_SIZE_H_FIXED: NodeId = hash_node_id("vector.layout.size.h.fixed");
/// Ver [`VECTOR_LAYOUT_SIZE_W_FIXED`], no outro eixo.
pub const VECTOR_LAYOUT_SIZE_H_HUG: NodeId = hash_node_id("vector.layout.size.h.hug");

/// **Piso da largura.** `0` = sem piso.
///
/// ⚠️ Zero significa *ausência*, e não um limite de zero — é o que permite estes quatro campos
/// serem números simples em vez de quatro pares `Auto | Fixed`. A leitura é honesta nos dois: um
/// piso de zero não restringe nada (o conteúdo já mede ≥ 0), e um TETO de zero não é um limite, é
/// um desaparecimento.
pub const VECTOR_LAYOUT_MIN_W: NodeId = hash_node_id("vector.layout.min.w");
/// Teto da largura. `0` = sem teto — ver [`VECTOR_LAYOUT_MIN_W`].
pub const VECTOR_LAYOUT_MAX_W: NodeId = hash_node_id("vector.layout.max.w");
/// Piso da altura. `0` = sem piso — ver [`VECTOR_LAYOUT_MIN_W`].
pub const VECTOR_LAYOUT_MIN_H: NodeId = hash_node_id("vector.layout.min.h");
/// Teto da altura. `0` = sem teto — ver [`VECTOR_LAYOUT_MIN_W`].
pub const VECTOR_LAYOUT_MAX_H: NodeId = hash_node_id("vector.layout.max.h");

/// **Este filho sai do fluxo** — o *Absolute position* do Figma.
///
/// ⚠️ Marcado, ele fica com a pose que o artista lhe deu (continua filho: anda com o pai e é
/// recortado por ele), e as linhas **Grow/Shrink deixam de ser pintadas** — quem não está no fluxo
/// não reparte sobra nenhuma, e oferecer os dois números ali seria o controlo morto que a política
/// de UI deste repo existe para impedir.
pub const VECTOR_LAYOUT_ITEM_ABSOLUTE: NodeId = hash_node_id("vector.layout.item.absolute");
