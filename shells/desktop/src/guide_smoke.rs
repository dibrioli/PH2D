//! **A cena das GUIAS e da RÉGUA** — `PH2D_BUILD_SMOKE=45` (plano 25 §9, a W6.2).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modo nenhum** — a cicatriz que o `impasto_smoke` do Painter
//! prega: um smoke que arma o estado por baixo do pano pula justamente a costura que existe para
//! provar. As réguas já nascem à mostra porque esse é o **default do produto**, não um arranjo
//! da cena.
//!
//! O que ela monta:
//! - **duas guias já postas** (uma vertical, uma horizontal), para que mover e apagar sejam
//!   testáveis sem antes ter de criar — e para que o round-trip do save tenha o que carregar;
//! - **um retângulo encostado na guia vertical**, cujo lado direito fica a uma fração de
//!   unidade dela: é o par que prova a LEI do empate (a guia vence o vértice);
//! - **uma barra atravessada**, longe das guias, como controle: perto dela o ímã tem de ser o
//!   de sempre.

use ph2d_guides::Guide;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Largura do traço das referências, em unidades de mundo.
const STROKE_W: f64 = 0.05;

/// A guia VERTICAL da cena, em mundo.
const GUIDE_X: f64 = 1.0;
/// A guia HORIZONTAL da cena, em mundo.
const GUIDE_Y: f64 = 0.5;

/// A geometria, numa tabela — `(pontos, fechado, cor)`.
///
/// ⚠️ `const` e partilhada com a sonda de baixo de propósito: as distâncias que a mensagem
/// anuncia são MEDIDAS daqui, não escritas de memória. Uma cena que afirma um número que a
/// geometria dela não tem é a forma exata de um smoke que engana quem o corre.
type Piece = (&'static [[f64; 2]], bool, [u8; 3]);

const PIECES: &[Piece] = &[
    // O retângulo cujo lado direito quase toca a guia vertical: o par do empate.
    (
        &[[0.1, -0.4], [0.94, -0.4], [0.94, 0.4], [0.1, 0.4]],
        true,
        [220, 180, 90],
    ),
    // O controle: uma barra longe das duas guias.
    (
        &[[-1.8, -1.2], [-0.6, -1.2], [-0.6, -0.9], [-1.8, -0.9]],
        true,
        [130, 150, 200],
    ),
];

fn vertex(a: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor: a,
        in_handle: a,
        out_handle: a,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn poly(pts: &[[f64; 2]], closed: bool, rgb: [u8; 3]) -> VecPath {
    VecPath {
        verts: pts.iter().map(|p| vertex(*p)).collect(),
        closed,
        stroke: Some(StrokeSpec::new(
            Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
            STROKE_W,
        )),
        ..VecPath::default()
    }
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (pts, closed, rgb) in PIECES {
        gfx.vec_scene.push_path(poly(pts, *closed, *rgb));
    }
    gfx.guides.push(Guide::vertical(GUIDE_X));
    gfx.guides.push(Guide::horizontal(GUIDE_Y));
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    // A menor distância entre a guia vertical e um vértice do retângulo: é ela que torna o
    // empate alcançável com a mão, e é ela que o artista vai sentir.
    let gap = PIECES[0]
        .0
        .iter()
        .map(|p| (GUIDE_X - p[0]).abs())
        .fold(f64::INFINITY, f64::min);
    eprintln!(
        "[guides] cena montada: {} guia(s) — vertical x={GUIDE_X}, horizontal y={GUIDE_Y}; \
         {} formas; o lado direito do retângulo está a {gap:.2} unidades da guia vertical",
        gfx.guides.len(),
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[guides] REGUAS: {} (o default do produto)",
        if app.rulers_visible() {
            "à mostra"
        } else {
            "fora"
        }
    );
    eprintln!("[guides] o roteiro:");
    eprintln!("  1. CRIAR — arraste de dentro da faixa de cima para baixo: nasce uma guia");
    eprintln!("     HORIZONTAL. Da faixa da esquerda para a direita, uma VERTICAL.");
    eprintln!("  2. MOVER — pegue uma das duas que já estão lá e arraste. Um Ctrl+Z depois");
    eprintln!("     de soltar tem de desfazer o arrasto INTEIRO, num passo só.");
    eprintln!("  3. APAGAR — arraste uma guia de volta para QUALQUER uma das duas faixas.");
    eprintln!("  4. O IMÃ — com a ferramenta Vector, desenhe perto da guia vertical: o ponto");
    eprintln!("     pousa NELA, e a marca do encaixe é um quadradinho sobre a linha.");
    // O ponto de virada, DERIVADO da geometria: com a guia e o vértice dentro do limiar, quem
    // vence é quem estiver mais perto, e a guia leva o EMPATE — logo ela passa a ganhar a
    // partir do ponto médio entre os dois. Medido no motor: 0,970 para esta cena, que é
    // exatamente o que a lei prevê (a previsão e a medição concordando é a terceira
    // testemunha, não uma coincidência).
    let tie = (GUIDE_X + PIECES[0].0[1][0]) * 0.5;
    eprintln!(
        "     ⚠️ O lado do retângulo está a {gap:.2} dali. Passando de x={tie:.3} a GUIA vence \
         o vértice — ela é a restrição AUTORADA, ele é incidental."
    );
    eprintln!("  5. O LOCK — desligue 'Rulers' na seção Snap: as faixas somem, as guias FICAM");
    eprintln!("     visíveis e magnéticas, e nenhum arrasto as move. Religue e volta tudo.");
    eprintln!("  6. O ZERO — a régua conta a partir da origem da GRADE. Mude a origem no");
    eprintln!("     painel de Grid Snap e os rótulos têm de acompanhar, sem a rede sair do");
    eprintln!("     lugar em relação a eles.");
    eprintln!("  7. O ARQUIVO — Ctrl+S, Ctrl+O: as guias voltam onde estavam.");
}
