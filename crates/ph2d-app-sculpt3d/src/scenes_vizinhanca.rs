//! **A CENA DOS EFEITOS DE VIZINHANÇA** (`=54`) — o desfoque, a nitidez e o
//! brilho do Painter sobre a peça, borrando NA SUPERFÍCIE (`docs/3D/30` §14).
//!
//! # ⚠️ A peça é a GROSSA da `=52` e o arame abre LIGADO
//!
//! O fenómeno a ver é a risca a desfocar por cima de uma ARESTA da malha sem
//! costura — e a aresta só se vê com o arame. Numa malha densa ela estaria em
//! toda a parte e em parte nenhuma.
//!
//! # ⚠️ A risca vem PINTADA, e a razão está escrita aqui
//!
//! O smoke é do DESFOQUE, não do pincel: a cor da base é um documento (como
//! abrir um ficheiro já pintado), oblíqua à grelha da esfera, escura sobre
//! claro, nos cinzentos que distinguem tons de ecrã de luz (nem `0` nem `255`).
//! O degrau é o `16x`: o arrasto do raio custa `≤ 7 ms` ali (§14), ao vivo.

use ph2d_sculpt3d::Verb;

/// `=54` — a cena dos **EFEITOS DE VIZINHANÇA**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `53`).
pub(crate) fn vizinhanca_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("54")
}

/// O degrau da tinta fina com que a cena abre (`16x`).
pub(crate) const NIVEL: u8 = 4;

/// O claro e o escuro da risca (sRGB8).
const CLARO: [u8; 4] = [214, 208, 196, 255];
const ESCURO: [u8; 4] = [38, 44, 70, 255];
/// A meia-largura da risca, nas unidades da peça (a esfera de raio `1`).
const MEIA_LARGURA: f32 = 0.06;

/// A cor da base num sítio da peça: a faixa em torno de um círculo máximo
/// oblíquo (atravessa arestas e faces em todos os ângulos) que cruza a FRENTE
/// da bola na diagonal, onde a câmara de abertura a vê.
pub(crate) fn cor_da_base(x: [f32; 3]) -> [u8; 4] {
    // ⟂ à frente da câmara de abertura (~(0,68; 0,25; 0,68)) e ao «direita + cima» do
    // ecrã: a risca passa no centro da bola, de cima-esquerda a baixo-direita.
    let n = [0.385_f32, 0.675, -0.630];
    let d = (x[0] * n[0] + x[1] * n[1] + x[2] * n[2]).abs();
    if d < MEIA_LARGURA { ESCURO } else { CLARO }
}

/// ⭐ **Arma a cena**: o pincel de pintura, o arame, a tinta fina a `16x` e a
/// base pintada com a risca.
pub(crate) fn arma(cena: &mut crate::Sculpt3dScene) {
    if !vizinhanca_scene() {
        return;
    }
    ph2d_panel_sculpt3d::state::switch_verb_parts(
        &mut cena.verb_slots,
        &mut cena.brush,
        &mut cena.radius_px,
        Verb::Paint,
    );
    cena.wireframe = true;
    cena.tinta_nivel = Some(NIVEL);
    if let Some(o) = cena.obj_mut() {
        pinta_a_risca(o);
    }
}

/// O plano a `16x`, a pilha de UMA camada e a risca pintada nela.
fn pinta_a_risca(o: &mut crate::SceneObject) {
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
    let px: Vec<[u8; 4]> = crate::vizinhanca_da_peca::posicoes(peca, mesh)
        .into_iter()
        .map(cor_da_base)
        .collect();
    if p.pinta_a_base(&px) {
        p.pinta_tinta(peca, Vec::new);
        let por_vertice = peca.plano_por_vertice().to_vec();
        stack.mesh_mut().colors_mut().copy_from_slice(&por_vertice);
        *tinta_suja = true;
    }
}

/// O roteiro da `=54`.
///
/// ⛔⛔ **O texto fica DENTRO do `eprintln!`** — a decisão da `=51`: numa `const`
/// ele perde a isenção do HR-15 e o censo de texto reprova-o.
pub(crate) fn announce() {
    if !vizinhanca_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =54 OS EFEITOS DE VIZINHANCA NA PECA (desfoque, nitidez, brilho)\n\
         [sculpt3d]    A bola ja' vem pintada: clara, com uma RISCA ESCURA que atravessa\n\
         [sculpt3d]    as linhas do arame (as arestas da malha). A fileira `Paint Detail`\n\
         [sculpt3d]    esta' em `16x`.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Na barra de cima, troque `IMG` por `PNTR` (o Painter). No painel\n\
         [sculpt3d]        do Painter abra o separador `Layers`: a lista mostra `Layer 1`.\n\
         [sculpt3d]    (2) Carregue no `+` dos ajustes e escolha `Gaussian Blur`.\n\
         [sculpt3d]        -> Nasce a linha `Gaussian Blur` no topo, com a barra `Radius`\n\
         [sculpt3d]           a zero. A bola ainda nao mudou.\n\
         [sculpt3d]    (3) Arraste a barra `Radius` devagar para a direita.\n\
         [sculpt3d]        -> A risca desfoca AO VIVO, enquanto arrasta. O numero da barra\n\
         [sculpt3d]           e' em % do tamanho da bola (no fim do curso, 2.5).\n\
         [sculpt3d]        -> Olhe onde a risca CRUZA uma linha do arame: o desfoque passa\n\
         [sculpt3d]           por cima dela sem costura -- nem mais claro nem mais escuro.\n\
         [sculpt3d]        -> Gire a bola (botao direito): o desfoque e' da propria bola.\n\
         [sculpt3d]    (4) `+` outra vez e escolha `Sharpen`; suba a barra `Amount`.\n\
         [sculpt3d]        -> A borda da risca ganha contorno, tambem sem costura.\n\
         [sculpt3d]    (5) `+` e escolha `Bloom`; suba a barra `Intensity`.\n\
         [sculpt3d]        -> A parte CLARA brilha e o brilho invade a risca escura.\n\
         [sculpt3d]    (6) `Ctrl+Z` varias vezes: os efeitos saem um a um e a bola volta\n\
         [sculpt3d]        a' risca nitida.\n\
         [sculpt3d]    -> No menu `+`, `Motion Blur`, `Chromatic Aberration` e `Halftone`\n\
         [sculpt3d]       aparecem APAGADOS: pedem uma direcao, um centro ou uma trama de\n\
         [sculpt3d]       imagem plana, e uma superficie nao tem nenhum dos tres.\n\
         [sculpt3d]    -> Acima de `64x` o `+` recusa estes efeitos com uma frase no\n\
         [sculpt3d]       painel: ali um unico passo levaria segundos."
    );
}

#[cfg(test)]
#[path = "scenes_vizinhanca_tests.rs"]
mod tests;
