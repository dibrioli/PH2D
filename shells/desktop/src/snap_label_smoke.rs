//! **A cena do RÓTULO DE DISTÂNCIA** — `PH2D_BUILD_SMOKE=72` (plano 25 §9, o último
//! item da tabela da W6).
//!
//! # A cena tem DUAS superfícies de propósito
//!
//! ⚠️ A pergunta desta wave não é *"aparece um número?"* — é **"as duas superfícies
//! que imprimem comprimento dizem o MESMO número?"**. Por isso a cena não mostra a
//! ficha sozinha: ela deixa a **RÉGUA** à vista (ela vive com a ferramenta Vector em
//! mãos, W6.2) e põe a distância a medir num valor que a régua também sabe dizer.
//!
//! Antes desta wave as duas discordavam por um fator de `pixels_per_meter` inteiro,
//! e nenhuma das duas sabia da outra: o Inspector e o painel de Grid Snap convertiam
//! para a unidade do artista, e a régua **não conseguia** — `paint_rulers` nem sequer
//! recebia as settings.
//!
//! # O que a cena NÃO arma
//!
//! Nada do gesto: nem o encaixe, nem o arrasto, nem a unidade. Encaixar É a wave; um
//! smoke que arma o estado por baixo do pano pula exatamente a costura que existe
//! para provar (a lei que o `impasto_smoke` prega e que esta linha honrou em `=70` e
//! `=71`).

use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Meia-largura de cada quadrado, em mundo.
const HALF: f64 = 0.5;
/// O vão VERTICAL entre os centros — o número que a ficha tem de dizer.
///
/// Escolhido redondo na unidade que o artista lê por default (100 px/m ⇒ **150 px**):
/// um número redondo é o que torna a comparação com a régua uma leitura, não uma conta.
const GAP_Y: f64 = 1.5;
/// O afastamento horizontal inicial — longe o bastante para que o encaixe seja um
/// gesto, e não a posição em que a cena já nasce.
const START_DX: f64 = 2.5;

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
    // A ÂNCORA, no centro: ela não se move, e é com ela que a outra vai alinhar.
    gfx.vec_scene
        .push_path(tint(rectangle([-HALF, -HALF], [HALF, HALF]), BLUE));
    // A VIAJANTE, deslocada nos dois eixos: alinhar o X é o gesto, e o vão em Y é o
    // que sobra para a ficha medir.
    gfx.vec_scene.push_path(tint(
        rectangle(
            [START_DX - HALF, GAP_Y - HALF],
            [START_DX + HALF, GAP_Y + HALF],
        ),
        AMBER,
    ));
    // Select: é o modo em que o gizmo move a forma, e é onde o snap de objeto vive.
    app.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
}

/// A mensagem — com os números MEDIDOS da própria cena e das settings VIVAS.
fn announce(app: &crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let n = gfx.vec_scene.paths().len();
    // As settings VIVAS — a mesma fonte que a régua e a ficha leem no frame.
    let display = gfx
        .hero_screen
        .as_ref()
        .map_or_else(ph2d_editor::LengthDisplay::default, |h| {
            ph2d_editor::LengthDisplay::of(&h.project)
        });
    // O que a ficha vai dizer, pela MESMA porta que a desenha.
    let expect = format!("{} {}", display.text(GAP_Y, 0.1), display.suffix());
    eprintln!("[snap-label-smoke] cena montada: {n} formas, modo SELECT.");
    eprintln!(
        "[snap-label-smoke] escala do projeto: {:.0} px/m, unidade {}.",
        display.pixels_per_meter,
        display.suffix()
    );
    eprintln!(
        "[snap-label-smoke] o vao VERTICAL entre os centros e' {GAP_Y} de mundo = **{expect}**."
    );
    eprintln!(
        "[snap-label-smoke] (!) se as formas nao forem 2, PARE: a cena perdeu a premissa e o \
         passo 2 nao tem com o que alinhar."
    );
    eprintln!("[snap-label-smoke] o roteiro:");
    eprintln!(
        "  1. Olhe a REGUA (faixas de cima e da esquerda). Com a escala acima ela numera em \
         **{}** -- os tracos ficam onde sempre estiveram, so' o numero mudou de unidade.",
        display.suffix()
    );
    eprintln!(
        "  2. Arraste o quadrado AMBAR para a esquerda ate' ele alinhar com o AZUL. Sai a linha \
         tracejada de sempre -- e agora uma FICHA com o numero no meio dela."
    );
    eprintln!(
        "     A ficha tem de dizer **{expect}**, e o traco da regua tem de concordar. As casas \
         decimais saem do que UM PIXEL de tela distingue neste zoom -- entao em metros ela traz \
         os CENTIMETROS."
    );
    eprintln!(
        "  3. Arraste devagar ate' quase encostar os dois quadrados: quando o segmento fica \
         curto demais para se ver, o numero SOME (ele mede o que se ve, e duas cruzes coladas \
         nao tem segmento entre elas)."
    );
    eprintln!(
        "  4. O CONTROLE, e e' o passo que prova a wave: menu **Settings > Unit > Meters**. A \
         REGUA e a FICHA tem de mudar JUNTAS (150 px vira 1,5 m). Se so' uma mudar, sao duas \
         portas outra vez."
    );
    eprintln!(
        "     (!) E a FICHA tem de trazer os CENTIMETROS -- **1.50 m**, nunca \"2 m\". As casas \
         saem de UM PIXEL de tela; o passo dos tracos da regua vale 1 m neste zoom e arredondaria \
         a medicao para o metro inteiro."
    );
    eprintln!(
        "  5. De volta em Pixels, confira o Inspector: a posicao do quadrado ambar em Y le' o \
         mesmo numero que a regua marca na altura dele."
    );
}
