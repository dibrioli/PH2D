//! **Os ids da §5 9-Slice do Inspector** (spec
//! [`03_inspector_secoes.md`](../../../../docs/Sprite_projeto/03_inspector_secoes.md) §3.5).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão de
//! [`super::inspector_sampling`], [`super::inspector_joint`] e [`super::inspector_player`].
//!
//! A seção declarada em 2026-05 e construída em **2026-08-21**: até essa data
//! `git grep -c SliceNine` dava **0** em todo o repositório, e a auditoria
//! ([`20_auditoria_do_inspector`](../../../../docs/Sprite_projeto/20_auditoria_do_inspector_2026-08-21.md) §6)
//! mediu-a como uma das três seções da spec que nunca nasceram.
//!
//! **A POSIÇÃO NO ARRAY É A TAG** — a mesma lei da §9: o despacho deriva a tag de
//! `position(|&o| o == id)` e a shell fecha com `from_tag`. ⛔ Nunca reordene nenhum destes
//! arrays; a ordem **é** o contrato, e há gate a prendê-la.

use super::*;

/// §5 9-Slice — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] emparelhado com o
/// ponto de cor abaixo, e é isso — **uma linha** — que o faz nascer vivo nas quatro faces
/// (dobra · ponto · despacho do ponto · menu de contorno).
pub const INSP_LIVE_SLICE_SECTION: NodeId = hash_node_id("insp_live_slice_section");
/// §5 9-Slice — ponto de cor do cabeçalho.
pub const INSP_LIVE_SLICE_COLOR: NodeId = hash_node_id("insp_live_slice_color");

/// **Draw Mode** — um segmento por variante de `SliceDrawMode`, tags `0..=2`.
pub const INSP_SLICE_MODE: [NodeId; 3] = [
    hash_node_id("insp_slice_mode_simple"),
    hash_node_id("insp_slice_mode_sliced"),
    hash_node_id("insp_slice_mode_tiled"),
];

/// **Tile Mode** global — `Continuous` / `Whole`, tags `0..=1`.
///
/// ⛔ Houve aqui um terceiro, `Adaptive`, com um slider `Stretch Value` — retirado em 2026-08-22
/// porque o mecanismo não podia funcionar (o motivo, medido, está em `ph2d_ecs::SliceTileMode`).
pub const INSP_SLICE_TILE_MODE: [NodeId; 2] = [
    hash_node_id("insp_slice_tile_continuous"),
    hash_node_id("insp_slice_tile_whole"),
];

/// As quatro bordas, em pixels da fonte: **`[left, top, right, bottom]`**.
///
/// ⚠️ A ordem é a do campo `SliceNine::borders`, e o despacho indexa por `position`. Trocar dois
/// destes ids faria o artista arrastar «esquerda» e ver a borda de cima mexer — e compila.
pub const INSP_SLICE_BORDER: [NodeId; 4] = [
    hash_node_id("insp_slice_border_l"),
    hash_node_id("insp_slice_border_t"),
    hash_node_id("insp_slice_border_r"),
    hash_node_id("insp_slice_border_b"),
];

/// Tamanho alvo em metros, `[x, y]`. `0` = herda o tamanho do sprite.
pub const INSP_SLICE_SIZE: [NodeId; 2] = [
    hash_node_id("insp_slice_size_x"),
    hash_node_id("insp_slice_size_y"),
];

/// **A grelha 3×3 dos modos por-região** — oito células, na ordem de `SliceRegion::ALL`
/// (TL · T · TR · L · R · BL · B · BR).
///
/// ⚠️ **É um CYCLER, não um segmented, e a escolha é de desenho.** A spec pedia «8 × Dropdown»;
/// oito dropdowns de quatro opções são 32 alvos e ~8 linhas num painel estreito. Uma grelha 3×3
/// em que cada célula mostra a inicial do seu modo e cicla ao clique ocupa três linhas, tem oito
/// alvos — e **parece a coisa que edita**. O miolo da grelha não é clicável: ele obedece a
/// `Fill Center`, que é uma lei própria.
pub const INSP_SLICE_REGION: [NodeId; 8] = [
    hash_node_id("insp_slice_region_tl"),
    hash_node_id("insp_slice_region_t"),
    hash_node_id("insp_slice_region_tr"),
    hash_node_id("insp_slice_region_l"),
    hash_node_id("insp_slice_region_r"),
    hash_node_id("insp_slice_region_bl"),
    hash_node_id("insp_slice_region_b"),
    hash_node_id("insp_slice_region_br"),
];

/// **A célula do MIOLO na grelha 3×3** — a nona, que não é uma das oito da moldura.
///
/// ⚠️ Ela cicla só **Stretch → Repeat → Mirror**: apagar o miolo é o [`INSP_SLICE_FILL_CENTER`],
/// e duas portas para o mesmo estado divergem.
pub const INSP_SLICE_CENTRE: NodeId = hash_node_id("insp_slice_centre");

/// `Fill Center` — o miolo desenha-se, ou a moldura fica oca.
pub const INSP_SLICE_FILL_CENTER: NodeId = hash_node_id("insp_slice_fill_center");

/// **«+ Add 9-Slice»** — anexa o componente. ⚠️ Anexar é INERTE (`SliceNine::INERT`): não muda
/// um pixel. Um botão que abre uma seção não pode ser uma edição destrutiva disfarçada.
pub const INSP_SLICE_ADD: NodeId = hash_node_id("insp_slice_add");
/// **«× Remove 9-Slice»** — retira o componente.
pub const INSP_SLICE_REMOVE: NodeId = hash_node_id("insp_slice_remove");
