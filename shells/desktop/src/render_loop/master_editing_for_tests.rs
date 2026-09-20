//! ⚠️ **A MESMA porta do passe, alcançável dos gates de outro módulo** — a cadeia de
//! visibilidade do vetor lê a marca do `MasterPiece`, e o gate dela tem de a poder carimbar.
//! *Um segundo carimbo escrito à mão no teste seria a segunda resposta à mesma pergunta.*
//!
//! ⚠️ Ela saiu do `render_loop/mod.rs` na integração de 2026-09-20, por tecto de LOC (`607`
//! contra `600`) e **por responsabilidade**: aquele ficheiro é o ÍNDICE das fases do quadro — o
//! cabeçalho dele declara-o —, e era esta a única FUNÇÃO lá dentro.

pub(crate) fn master_editing_mark_for_tests(
    sim: &mut ph2d_ecs::SimWorld,
    selection: Option<u64>,
) -> bool {
    ph2d_app_components::master_editing::mark(sim, selection, &mut None).touched
}
