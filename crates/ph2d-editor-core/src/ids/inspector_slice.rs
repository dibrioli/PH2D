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

// ⛔ Houve aqui um **«× Remove 9-Slice»**, retirado em 2026-08-22 pela pergunta do Enio: *«o
// botao xRemove 9-slice ainda faz sentido?»*. Medido, não fazia — um sprite **sem** o componente
// e um **com ele desligado** desenham igual, gravam igual e, depois de o botão sair, mostram a
// mesma seção. Ele era a segunda porta para «desligado», e a caixa já é a primeira.
//
// ⚠️ O «estado inalcançável» que isto cria (voltar a não-ter o componente) **não custa nada
// observável** — é a definição de equivalente. O que custava era o botão: uma ação sem efeito
// visível, a terceira desta seção depois do slider e do `Simple`.
