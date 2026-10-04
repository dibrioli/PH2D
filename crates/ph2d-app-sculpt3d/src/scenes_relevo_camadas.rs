//! **A CENA DO RELEVO POR CAMADA** (`=55`) — a profundidade e o `Add`/`Level`
//! de cada camada do Painter sobre a peça (`docs/3D/30` §15, a W4).
//!
//! # ⚠️ As duas camadas vêm PINTADAS, e a razão está escrita aqui
//!
//! O smoke é do RELEVO POR CAMADA, não do pincel: a base é clara com uma TEXTURA
//! de riscas horizontais em relevo; a `Layer 2`, por cima, é uma faixa vertical
//! cor de terra com uma LOMBA lisa. Onde a faixa cruza as riscas está o que se
//! aprende: baixar a profundidade da de cima achata a lomba e deixa as riscas;
//! o `Level` faz a lomba ENTERRAR as riscas por baixo dela. O degrau é o `16x`.

use ph2d_sculpt3d::Verb;

/// `=55` — a cena do **RELEVO POR CAMADA**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `54`).
pub(crate) fn relevo_camadas_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("55")
}

/// O degrau da tinta fina com que a cena abre (`16x`).
pub(crate) const NIVEL: u8 = 4;

const CLARO: [u8; 4] = [214, 208, 196, 255];
const TERRA: [u8; 4] = [178, 92, 60, 255];
/// As riscas da base: altura e período, nas unidades da peça (a esfera de raio `1`).
const RISCA_ALTURA: f32 = 0.012;
const RISCA_PERIODO: f32 = 0.08;
/// A faixa de cima: meia-largura e a altura da lomba no meio dela.
const FAIXA_MEIA_LARGURA: f32 = 0.18;
const LOMBA: f32 = 0.03;
/// A normal do plano da faixa: vertical, pela frente da câmara de abertura
/// (~`(0,68; 0,25; 0,68)`).
const FAIXA_N: [f32; 3] = [
    -std::f32::consts::FRAC_1_SQRT_2,
    0.0,
    std::f32::consts::FRAC_1_SQRT_2,
];

/// O relevo `[altura, corpo]` da base num sítio: riscas horizontais em toda a bola.
pub(crate) fn relevo_da_base(x: [f32; 3]) -> [f32; 2] {
    let fase = x[1] / RISCA_PERIODO * std::f32::consts::TAU;
    [RISCA_ALTURA * (0.5 + 0.5 * fase.sin()), 1.0]
}

/// A cor e o relevo da `Layer 2` num sítio: dentro da faixa, terra e a lomba;
/// fora, nada.
pub(crate) fn faixa(x: [f32; 3]) -> ([u8; 4], [f32; 2]) {
    let d = (x[0] * FAIXA_N[0] + x[1] * FAIXA_N[1] + x[2] * FAIXA_N[2]).abs();
    // Só a frente da bola: o plano da faixa corta-a também atrás.
    let frente = x[0] + x[2] > 0.0;
    if !frente || d >= FAIXA_MEIA_LARGURA {
        return ([0; 4], [0.0; 2]);
    }
    let t = d / FAIXA_MEIA_LARGURA;
    (TERRA, [LOMBA * (1.0 - t * t), 1.0])
}

/// ⭐ **Arma a cena**: o pincel de pintura, a tinta fina a `16x` e as duas
/// camadas pintadas.
pub(crate) fn arma(cena: &mut crate::Sculpt3dScene) {
    if !relevo_camadas_scene() {
        return;
    }
    ph2d_panel_sculpt3d::state::switch_verb_parts(
        &mut cena.verb_slots,
        &mut cena.brush,
        &mut cena.radius_px,
        Verb::Paint,
    );
    cena.tinta_nivel = Some(NIVEL);
    if let Some(o) = cena.obj_mut() {
        pinta_as_camadas(o);
    }
}

/// O plano a `16x`, a pilha, a base com as riscas e a `Layer 2` com a faixa.
fn pinta_as_camadas(o: &mut crate::SceneObject) {
    let crate::objects::SceneObject {
        stack,
        tinta,
        tinta_parqueada,
        pilha,
        pilha_parqueada,
        tinta_suja,
        ..
    } = o;
    let mesh = stack.mesh();
    crate::tinta_da_peca::pilha::garante_com_pilha(
        mesh,
        tinta,
        tinta_parqueada,
        pilha,
        pilha_parqueada,
        Some(NIVEL),
        u64::MAX,
    );
    let (Some(peca), Some(p)) = (tinta.as_mut(), pilha.as_mut()) else {
        return;
    };
    let xs = crate::vizinhanca_da_peca::posicoes(peca, mesh);
    let Some(base) = p.base() else {
        return;
    };
    let claro = vec![CLARO; xs.len()];
    let riscas = xs.iter().map(|&x| relevo_da_base(x)).collect();
    let (cor, lomba): (Vec<[u8; 4]>, Vec<[f32; 2]>) = xs.iter().map(|&x| faixa(x)).unzip();
    let nome = ph2d_i18n::tr_with("app.sculpt3d.pilha_da_peca.camada_nova", &[("n", &2)]);
    let ok = p.pinta_camada(base, &claro, Some(riscas))
        && p.nova_camada(&nome)
            .is_ok_and(|cima| p.pinta_camada(cima, &cor, Some(lomba)));
    if ok {
        p.pinta_tinta(peca, Vec::new);
        let por_vertice = peca.plano_por_vertice().to_vec();
        stack.mesh_mut().colors_mut().copy_from_slice(&por_vertice);
        *tinta_suja = true;
    }
}

/// O roteiro da `=55`.
///
/// ⛔⛔ **O texto fica DENTRO do `eprintln!`** — a decisão da `=51`: numa `const`
/// ele perde a isenção do HR-15 e o censo de texto reprova-o.
pub(crate) fn announce() {
    if !relevo_camadas_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =55 O RELEVO POR CAMADA NA PECA (profundidade e Add/Level)\n\
         [sculpt3d]    A bola ja' vem pintada em DUAS camadas: a de baixo e' clara com\n\
         [sculpt3d]    RISCAS horizontais em relevo; a de cima e' uma FAIXA vertical cor\n\
         [sculpt3d]    de terra com uma LOMBA lisa, a meio da frente. `16x`.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Na barra de cima, troque `IMG` por `PNTR` (o Painter). No painel\n\
         [sculpt3d]        do Painter abra o separador `Layers`: `Layer 2` em cima, `Layer 1`\n\
         [sculpt3d]        em baixo, cada uma com uma TERCEIRA linha: a barra da profundidade\n\
         [sculpt3d]        (`100%`) e o botao `Add`.\n\
         [sculpt3d]    (2) Na linha da `Layer 2`, arraste a barra da profundidade devagar\n\
         [sculpt3d]        para a esquerda, ate' `0%`.\n\
         [sculpt3d]        -> A LOMBA da faixa achata AO VIVO; as riscas da camada de baixo\n\
         [sculpt3d]           ficam iguais, tambem por baixo da faixa. A cor nao muda.\n\
         [sculpt3d]        -> Abaixo de `0%` a lomba vira um SULCO.\n\
         [sculpt3d]    (3) `Ctrl+Z`: a lomba volta inteira.\n\
         [sculpt3d]    (4) Carregue no `Add` da `Layer 2`: passa a `Level`.\n\
         [sculpt3d]        -> Dentro da faixa as riscas SOMEM: a lomba lisa enterra a textura\n\
         [sculpt3d]           de baixo. Fora da faixa, as riscas ficam.\n\
         [sculpt3d]    (5) Na linha da `Layer 1`, baixe a profundidade: as riscas achatam\n\
         [sculpt3d]        fora da faixa; dentro dela (com `Level`) nada muda.\n\
         [sculpt3d]    (6) `Ctrl+Z` varias vezes: tudo volta ao que abriu."
    );
}

#[cfg(test)]
#[path = "scenes_relevo_camadas_tests.rs"]
mod tests;
