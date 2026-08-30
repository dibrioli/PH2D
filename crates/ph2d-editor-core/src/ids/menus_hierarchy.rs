//! **Os ids do menu de contexto de uma LINHA da Hierarquia** — irmão de [`super::menus`].
//!
//! ⚠️ O corte é por RESPONSABILIDADE e segue o precedente do [`super::menus_timeline`]: este menu é
//! o maior do app (25 linhas) e cresce sozinho sempre que um módulo ganha um verbo por-objecto. O
//! ficheiro único passou o tecto de 700 LOC ao ganhar o par `Group`/`Ungroup` (2026-08-30).
//!
//! ⚠️ **Os ids continuam a resolver-se como `ids::CTX_MENU_HIER_*`** — o `pub use` no [`super`] não
//! muda o endereço de nenhum chamador, e nenhum é um número: são hashes do próprio nome, então o
//! corte não pode colidir com nada.

use super::*;

/// Hierarchy row context menu: "Use as Brush **Grain**" — load the right-clicked sprite's pixels as the
/// brush Grain (texture) image (shown only for image/sprite rows; Enio 2026-06-24). The id keeps the
/// legacy `texture` slug for wire stability; the label is "Use as Brush Grain".
pub const CTX_MENU_HIER_USE_AS_BRUSH_TEXTURE: NodeId =
    hash_node_id("ctx_menu_hier_use_as_brush_texture");

/// Hierarchy row context menu: "Use as Brush **Shape**" — load the right-clicked sprite's pixels as the
/// brush Shape (silhouette tip) image (Enio 2026-06-25; the Shape slot replaces the falloff).
pub const CTX_MENU_HIER_USE_AS_BRUSH_SHAPE: NodeId =
    hash_node_id("ctx_menu_hier_use_as_brush_shape");

/// Hierarchy row context menu: "Use as **Watercolor Paper**" — install the right-clicked layer/group's
/// pixels as the watercolor paper (Grain slot, canvas-anchored), so the wash granulates against it
/// (`docs/Painter/10…` §5). A Group tags its composited children.
pub const CTX_MENU_HIER_USE_AS_PAPER: NodeId = hash_node_id("ctx_menu_hier_use_as_paper");

/// Hierarchy row context menu: "Use as **Granulation**" — like [`CTX_MENU_HIER_USE_AS_PAPER`] but as a
/// stronger mineral-settling map (pigment pools harder in the layer's valleys).
pub const CTX_MENU_HIER_USE_AS_GRANULATION: NodeId =
    hash_node_id("ctx_menu_hier_use_as_granulation");

// M14.6 F: per-row Hierarchy context menu entries. Triggered by a
// secondary (right-button) click on any hierarchy row in live mode;
// `ContextMenuKind::HierarchyRow { row }` carries the target row's
// NodeId so dispatch can attach the action to the right entity when
// any of these ids fires.
pub const CTX_MENU_HIER_DUPLICATE: NodeId = hash_node_id("ctx_menu_hier_duplicate");
pub const CTX_MENU_HIER_DELETE: NodeId = hash_node_id("ctx_menu_hier_delete");
pub const CTX_MENU_HIER_RESET_TRANSFORM: NodeId = hash_node_id("ctx_menu_hier_reset_transform");
/// ⭐ **Devolver uma INSTÂNCIA à receita** (ADR-0164 / F4.4) — apaga as excepções que o artista
/// fez nesta cópia, e o sync volta a propagar o mestre no quadro seguinte.
pub const CTX_MENU_HIER_REVERT_TO_MASTER: NodeId = hash_node_id("ctx_menu_hier_revert_to_master");
/// ⭐ **A seleção vira RECEITA** (ADR-0164 / F4.5) — e uma instância fica no lugar dela.
pub const CTX_MENU_HIER_MAKE_COMPONENT: NodeId = hash_node_id("ctx_menu_hier_make_component");
/// ⭐ **Instanciar** a receita escolhida (ADR-0164 / F4.5).
pub const CTX_MENU_HIER_INSTANTIATE: NodeId = hash_node_id("ctx_menu_hier_instantiate");
/// ⭐ **Instanciar LIGADO** (Enio, 2026-08-27) — o `Alt+D` do Blender: a cópia divide a ARTE da
/// receita, então editar a tinta ou o desenho dela sobe e chega a todas as irmãs.
pub const CTX_MENU_HIER_INSTANTIATE_LINKED: NodeId =
    hash_node_id("ctx_menu_hier_instantiate_linked");
/// ⭐ **Destacar** — a instância deixa de seguir a receita (ADR-0164 / F4.5).
pub const CTX_MENU_HIER_DETACH: NodeId = hash_node_id("ctx_menu_hier_detach");
/// ⭐ **Aplicar ao mestre** — a excepção vira o padrão (ADR-0164 / F4.5).
pub const CTX_MENU_HIER_APPLY_TO_MASTER: NodeId = hash_node_id("ctx_menu_hier_apply_to_master");
pub const CTX_MENU_HIER_ADD_CHILD: NodeId = hash_node_id("ctx_menu_hier_add_child");
// ⭐⭐ Os ids do menu de um CARTÃO da biblioteca vivem no irmão `menus_asset.rs` — ver o cabeçalho
// dele para a lei da tabela plana e para quem redige as recusas.
/// ⭐⭐⭐ **Agrupar** (Enio, 2026-08-30) — a seleção passa a ser **um objeto** na Hierarquia.
///
/// ⚠️ **O verbo já existia e era invisível.** `Ctrl+G` / `Ctrl+Shift+G` agrupam e desagrupam desde
/// sempre (`input_dispatch::vec_group`), e **nenhum botão, menu ou rótulo do app os nomeia** — é a
/// lei do repo aplicada à UI: *uma ferramenta que nenhum passo escrito chama pelo nome morre.* O
/// pedido do Enio foi *"criar a feature"*, e o que de facto faltava era **alcançá-la**.
pub const CTX_MENU_HIER_GROUP: NodeId = hash_node_id("ctx_menu_hier_group");
/// ⭐ **Desagrupar** — o gémeo, e ele tem de estar ao lado: um verbo destrutivo-de-estrutura cujo
/// inverso não se vê deixa o artista com medo de usar o primeiro.
pub const CTX_MENU_HIER_UNGROUP: NodeId = hash_node_id("ctx_menu_hier_ungroup");
/// M14.7 polish: per-row "Rename..." entry. Opens inline rename
/// mode (the row's name turns into a TextInput).
pub const CTX_MENU_HIER_RENAME: NodeId = hash_node_id("ctx_menu_hier_rename");
/// Enio 2026-05-27: "Merge Sprites" entry — flattens the current
/// multi-selection (≥ 2 sprites) into a single new Individual-texture
/// sprite at the union bounding box, then despawns the originals.
/// Always shown in the HierarchyRow menu; the drain handler emits a
/// toast when fewer than 2 sprites are selected (silent no-op
/// otherwise feels broken).
pub const CTX_MENU_HIER_MERGE_SPRITES: NodeId = hash_node_id("ctx_menu_hier_merge_sprites");

/// Enio, 2026-08-21: **"Merge to Layers"** — a mesma fusão do vizinho, mas cada sprite de origem
/// fica também numa **camada** do documento do Painter (plano `docs/Sprite_projeto/18` W10).
///
/// ⚠️ **Vizinho do "Merge Sprites" de propósito, e a diferença é uma só:** aquele achata e não há
/// volta; este achata **e guarda como separar outra vez**. Ler os dois seguidos é o que torna a
/// escolha óbvia no momento de a fazer — que é o único momento em que ela ainda é possível.
pub const CTX_MENU_HIER_MERGE_TO_LAYERS: NodeId = hash_node_id("ctx_menu_hier_merge_to_layers");
/// Enio 2026-08-19: "Pack into Sheet" — **cria** a folha com os sprites da seleção dentro.
///
/// ⚠️ **Ele CRIA, e só isso.** Já fez duas coisas conforme o alvo (com uma folha selecionada,
/// re-arranjava) — vide [`CTX_MENU_HIER_ARRANGE_SHEET`], que é esse verbo posto ao nome. Clicá-lo
/// sobre uma folha agora recusa e aponta para lá, em vez de fazer calado uma coisa diferente da
/// que o rótulo promete.
///
/// A criação abre primeiro o modal de resolução ([`CTX_MENU_SHEET_SIZES`]); a folha só nasce no
/// Create. Semântica de seleção idêntica à do "Merge Sprites" vizinho: leva a seleção inteira
/// quando a linha clicada faz parte dela, e só essa linha quando não faz.
pub const CTX_MENU_HIER_PACK_SHEET: NodeId = hash_node_id("ctx_menu_hier_pack_sheet");

/// Enio, 2026-08-21: *"exportação nos vários formatos suportados"* — **"Export Image…"**, a porta
/// que faltava para os 16 exportadores que a engine já tinha registados e que nenhum gesto
/// alcançava (plano `docs/Sprite_projeto/18` W9).
///
/// ⚠️ **Vizinho do "Export Sheet" de propósito, e a distinção está nos nomes:** aquele escreve a
/// FOLHA (png + json, o par que o Aseprite lê); este escreve **uma sprite**, no formato que a
/// extensão escolhida nomear. Ler os dois seguidos é o que torna a diferença óbvia.
pub const CTX_MENU_HIER_EXPORT_IMAGE: NodeId = hash_node_id("ctx_menu_hier_export_image");
/// Enio 2026-08-19: "Remove from Sheet" — a saída. A peça deixa de ser filha da folha e volta a
/// ser um objeto de raiz, **onde está** (a preservação de mundo do reparent trata disso).
///
/// ⚠️ **É a segunda porta de um gesto que já existia**, não um mecanismo novo: arrastar a linha
/// para fora da folha na hierarquia sempre fez isto. O Enio nomeou as duas — *"o usuário deve
/// acabar com seu parentesco na hierarchy ou com o menu do botão direito usar a opção de retirar
/// da sheet"* — e a razão de a segunda existir é a descoberta: um menu diz o que se pode fazer;
/// um arrasto só responde a quem já sabe. O trabalho é o MESMO caminho
/// (`HierReparentIntent { new_parent: None }`), de propósito.
pub const CTX_MENU_HIER_REMOVE_FROM_SHEET: NodeId = hash_node_id("ctx_menu_hier_remove_from_sheet");
/// Enio 2026-08-19: "Auto-Arrange Pieces" — re-encaixa os filhos DENTRO da resolução da folha.
///
/// ⚠️ **Este verbo já existia, escondido dentro do [`CTX_MENU_HIER_PACK_SHEET`]:** aquele item
/// fazia duas coisas conforme o alvo — criava com sprites, re-arranjava com uma folha. Parecia
/// economia («a pergunta é sempre *arrume isto*») e não era: um verbo que só se descobre por ter
/// selecionado a coisa certa **não está no menu**, está escondido nele. O Enio pediu-o pelo nome,
/// e esse pedido é a prova.
///
/// Agora cada item faz UMA coisa e diz porquê quando não pode: o Pack recusa uma folha e aponta
/// para aqui; este recusa o que não for folha e aponta para lá.
pub const CTX_MENU_HIER_ARRANGE_SHEET: NodeId = hash_node_id("ctx_menu_hier_arrange_sheet");
/// "Bake Sheet" — as peças deixam de ser N imagens e passam a ser N janelas para UMA textura
/// (plano `docs/Sprite_projeto/17` §7.3, W5.2).
///
/// ⚠️ Muda o DOCUMENTO (é um passo de undo) e não escreve ficheiro nenhum. O irmão
/// [`CTX_MENU_HIER_EXPORT_SHEET`] faz o contrário: escreve os ficheiros e não toca na cena.
pub const CTX_MENU_HIER_BAKE_SHEET: NodeId = hash_node_id("ctx_menu_hier_bake_sheet");
/// "Export Sheet" — grava `<nome>.png` + `<nome>.json` (formato Aseprite) ao lado do projeto.
///
/// ⚠️ **Compõe, mas NÃO reata.** Uma exportação que mudasse a cena faria um pedido de ficheiro
/// virar uma edição, e o artista descobriria pelo undo — o pior sítio para descobrir.
pub const CTX_MENU_HIER_EXPORT_SHEET: NodeId = hash_node_id("ctx_menu_hier_export_sheet");
