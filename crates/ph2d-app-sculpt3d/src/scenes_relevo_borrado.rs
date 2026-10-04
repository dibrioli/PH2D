//! **A CENA DO RELEVO BORRADO** (`=56`) — o Gaussiano por cima de camadas com relevo borra a cor
//! E o relevo delas (`docs/3D/30` §20, decisão do dono de 04/10).
//!
//! ⚠️ Ela abre com a pintura da `=55` (riscas em relevo na base, a faixa com a lomba por cima) e um
//! Gaussiano no topo com o raio a `0`, como o menu o cria: abre NÍTIDA, e o que se aprende é o
//! arrasto do raio — a borda da faixa e as riscas amaciam juntas.

use ph2d_sculpt3d::Verb;
use ph2d_tool_painter::AdjustmentKind;

/// `=56` — a cena do **RELEVO BORRADO**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `55`).
pub(crate) fn relevo_borrado_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("56")
}

/// ⭐ **Arma a cena**: a `=55` e o Gaussiano por cima.
pub(crate) fn arma(cena: &mut crate::Sculpt3dScene) {
    if !relevo_borrado_scene() {
        return;
    }
    ph2d_panel_sculpt3d::state::switch_verb_parts(
        &mut cena.verb_slots,
        &mut cena.brush,
        &mut cena.radius_px,
        Verb::Paint,
    );
    cena.tinta_nivel = Some(super::relevo_camadas::NIVEL);
    if let Some(o) = cena.obj_mut() {
        super::relevo_camadas::pinta_as_camadas(o);
        if let Some(p) = o.pilha.as_mut() {
            poe_o_desfoque(p, o.stack.mesh());
        }
    }
}

/// O Gaussiano no topo da pilha da peça, nas unidades dela.
pub(crate) fn poe_o_desfoque(p: &mut crate::pilha_da_peca::PilhaDaPeca, mesh: &ph2d_mesh::Mesh) {
    let _ = p.novo_ajuste(
        AdjustmentKind::GaussianBlur,
        crate::vizinhanca_da_peca::unidades(mesh),
    );
}

/// O roteiro da `=56`.
///
/// ⛔⛔ **O texto fica DENTRO do `eprintln!`** — a decisão da `=51`: numa `const`
/// ele perde a isenção do HR-15 e o censo de texto reprova-o.
pub(crate) fn announce() {
    if !relevo_borrado_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =56 O DESFOQUE AMACIA TAMBEM O RELEVO\n\
         [sculpt3d]    A bola vem como na =55: a de baixo clara com RISCAS em relevo, a de\n\
         [sculpt3d]    cima uma FAIXA cor de terra com uma LOMBA. Por cima de tudo, um\n\
         [sculpt3d]    efeito `Gaussian Blur` com o raio a zero (ainda nitido). `16x`.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Na barra de cima, troque `IMG` por `PNTR` (o Painter) e abra o\n\
         [sculpt3d]        separador `Layers`: no topo, a linha `Gaussian Blur`.\n\
         [sculpt3d]    (2) Na barra do raio do `Gaussian Blur`, arraste devagar para a direita.\n\
         [sculpt3d]        -> A borda da faixa fica macia E as riscas e a lomba amaciam AO VIVO:\n\
         [sculpt3d]           a luz deixa de desenhar as riscas finas.\n\
         [sculpt3d]    (3) `Ctrl+Z`: riscas e lomba voltam nitidas.\n\
         [sculpt3d]    (4) No olho do `Gaussian Blur`: escondido, tudo nitido; visivel, macio.\n\
         [sculpt3d]    Deu errado se: so' a cor amacia e as riscas ficam riscadas; ou o\n\
         [sculpt3d]    `Ctrl+Z` nao devolve as riscas."
    );
}

#[cfg(test)]
#[path = "scenes_relevo_borrado_tests.rs"]
mod tests;
