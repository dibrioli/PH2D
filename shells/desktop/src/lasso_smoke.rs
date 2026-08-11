//! **A cena do LAÇO** — `PH2D_BUILD_SMOKE=71` (plano 25 §9).
//!
//! O plano nomeava o laço como *"wave dela"* e a wave dos nós de várias formas (`=70`) era o
//! pré-requisito declarado: um laço que varre os nós de duas formas não significa nada enquanto a
//! seleção só souber guardar os de uma.
//!
//! # A cena é uma FILEIRA ALTERNADA, e é a premissa que a torna capaz de reprovar
//!
//! ⚠️ Num par de formas separadas, **um retângulo faz tudo o que o laço faz** — e um smoke sobre
//! essa cena aprovaria um laço que fosse a caixa envolvente do próprio caminho. A cena aqui são
//! **seis formas alternadas** (azul · âmbar · azul · âmbar · …) numa fileira, e o pedido é
//! *"selecione os nós das TRÊS AZUIS e de nenhuma âmbar"*: nenhum retângulo separa esse conjunto,
//! porque cada azul tem uma âmbar entre ela e a seguinte. É o caso de uso inteiro numa figura.
//!
//! # O que esta cena NÃO arma
//!
//! Nada da seleção, e nada da forma do marquee — nem o chip, nem o Ctrl. O gesto de laçar É a
//! wave; um smoke que arma o estado por baixo do pano pula exatamente a costura que existe para
//! provar (a lei que o `impasto_smoke` prega, e que esta linha já honrou em `=70`).

use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Quantas formas a fileira tem — ímpar de propósito, para começar e acabar em AZUL.
const N: usize = 5;
/// A meia-largura de cada quadrado.
const HALF: f64 = 0.45;
/// O passo entre centros — folgado, para o laço poder serpentear entre elas.
const STEP: f64 = 1.6;
/// Azul = o alvo; âmbar = o que não pode entrar.
const BLUE: [u8; 3] = [86, 132, 214];
const AMBER: [u8; 3] = [214, 150, 70];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        8 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let x0 = -(N as f64 - 1.0) * STEP * 0.5;
    for k in 0..N {
        let cx = x0 + k as f64 * STEP;
        // As ÍMPARES são âmbar, e sobem um pouco: sem o degrau, uma faixa horizontal fina já
        // separaria as azuis, e o retângulo voltaria a bastar.
        let (rgb, cy) = if k % 2 == 0 {
            (BLUE, 0.0)
        } else {
            (AMBER, 0.35)
        };
        gfx.vec_scene.push_path(tint(
            rectangle([cx - HALF, cy - HALF], [cx + HALF, cy + HALF]),
            rgb,
        ));
    }
    app.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let n = gfx.vec_scene.paths().len();
    let nodes: usize = gfx.vec_scene.paths().iter().map(|p| p.total_verts()).sum();
    let blue = N.div_ceil(2);
    eprintln!("[lasso-smoke] cena montada: {n} formas, {nodes} nos, modo NODE.");
    eprintln!(
        "[lasso-smoke] {blue} AZUIS (alvo) alternadas com {} AMBAR (nao podem entrar).",
        N - blue
    );
    eprintln!(
        "[lasso-smoke] (!) se as formas nao forem {N}, ou os nos nao forem {}, PARE: a cena perdeu \
         a premissa e o passo 2 deixa de exigir um laco.",
        N * 4
    );
    eprintln!("[lasso-smoke] o roteiro (a ferramenta VECTOR ja' esta' no modo Node):");
    eprintln!(
        "  1. Arraste do VAZIO. Sai o retangulo de sempre — e agora ha' um par `Marquee: Box | \
         Lasso` logo abaixo da fileira TOOL."
    );
    eprintln!("  2. Clique **Lasso** e SERPENTEIE em volta das {blue} azuis, pulando as ambar.");
    eprintln!(
        "     Os {} nos azuis acendem em CIANO e NENHUM ambar acende. (!) se as ambar entrarem, o \
         laco esta' a usar a CAIXA do caminho, e nao o caminho.",
        blue * 4
    );
    eprintln!("  3. Arraste um dos nos acesos: as tres azuis acompanham, as ambar ficam.");
    eprintln!(
        "  4. Volte o chip para **Box** e segure **Ctrl** ao comecar o arrasto: sai o LACO na \
         mesma. Solte o Ctrl e sai o retangulo. O chip nao mudou."
    );
    eprintln!(
        "  5. Com nos ja' escolhidos, **Shift+Ctrl**+arraste um segundo laco: ele SOMA (e o \
         primeiro conjunto nao se apaga)."
    );
    eprintln!(
        "  6. O CONTROLE: com o chip em Box e sem Ctrl, tudo tem de estar exatamente como sempre \
         esteve — caixa, aditivo com Shift, clique no vazio a desselecionar."
    );
}
