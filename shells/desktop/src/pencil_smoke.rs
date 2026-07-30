//! **A cena pronta para o smoke do LÁPIS** — `PH2D_BUILD_SMOKE=40`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Esta cena NÃO arma o modo Pencil**, e é deliberado: o gesto que o smoke tem de provar
//! começa no CHIP do painel. Armar o modo em código pularia exatamente a costura que ele existe
//! para exercer — a cicatriz que o `impasto_smoke` do Painter prega (*"o smoke que arma estado por
//! baixo da mesa pula justamente o seam que devia provar"*) e que um dos smokes dele contradizia.
//!
//! O que ela dá é **uma REFERÊNCIA na tela**: dois traços produzidos pelo motor a partir de uma
//! mão sintética (o mesmo `Pencil` que o dedo do artista vai dirigir), para ele comparar o que a
//! própria mão produz — e para provar que o módulo está no binário.
//!
//! **Números MEDIDOS** — o par que esta cena desenha (240 amostras, ±0,22 px de tremor):
//! `Fidelity 0,5 px` → **44 nós** (o detalhista, em cima) · `8 px` → **11 nós** (o liso, embaixo).
//! O caminho é o MESMO nos dois; a única diferença é o *Fidelity*. (A tabela do motor, medida
//! sobre um S maior, vive no doc de `ph2d_vec_edit::pencil::DEFAULT_FIDELITY_PX`: 32/9/4 nós a
//! 0,5/2/8 px, com desvio 1,04/0,77/3,55 px.)

use ph2d_vec_edit::Pencil;
use ph2d_vec_scene::{Rgba8, VecPathId};

/// Metade da largura do S de referência, em unidades de mundo.
const HALF_W: f64 = 1.8;
/// Amplitude do S.
const AMP: f64 = 0.9;
/// Amostras da mão sintética (a ordem de grandeza de um arrasto real de ~2 s).
const SAMPLES: usize = 240;
/// Amplitude do tremor da mão sintética, em unidades de MUNDO (≈0,22 px de tela na câmera
/// default). **MEDIDO** (`tmp` de calibração, o par que a cena desenha): com 0,001 o Fidelity
/// `0,5 px` dá **44 nós** e o `8 px` dá **11** — detalhista contra liso, e nenhum dos dois
/// degenerado. Com 0,006 (o 1º palpite) o lado detalhista saía com **189 nós**: uma referência
/// que ensinaria o artista a coisa errada, porque o tremor era maior que a própria tolerância.
const TREMOR: f64 = 0.001;

/// A "mão": um S com tremor determinístico (nada de `rand` — a cena tem de ser a mesma toda vez).
fn hand(dy: f64) -> Vec<[f64; 2]> {
    (0..SAMPLES)
        .map(|i| {
            let t = i as f64 / (SAMPLES - 1) as f64;
            let f = i as f64;
            [
                -HALF_W + 2.0 * HALF_W * t + TREMOR * (f * 1.9).sin(),
                dy + AMP * (t * std::f64::consts::TAU).sin() + TREMOR * (f * 2.7).cos(),
            ]
        })
        .collect()
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

/// Dirige o `Pencil` pelo caminho do PRODUTO (press → moves → release) duas vezes, com o
/// *Fidelity* nas duas pontas da faixa medida.
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    // Um pixel de tela vale ~0,0045 de mundo na câmera default (4 unidades de altura em ~900 px);
    // o valor exato não importa para a cena, mas ele TEM de ser o mesmo nos dois traços, senão o
    // par comparado teria duas variáveis.
    let px_to_world = 0.0045;
    for (fidelity, dy, rgb) in [
        (0.5_f64, 1.1_f64, [70u8, 150, 220]),
        (8.0, -1.1, [220, 150, 90]),
    ] {
        let mut pencil = Pencil::default();
        // O estilo do traço de referência é escolhido aqui (a cena não tem tool publicada ainda):
        // largura visível e a cor que identifica o par.
        pencil.set_style(ph2d_vec_edit::PenStyle {
            stroke: Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
            stroke_w_px: 3.0,
            ..ph2d_vec_edit::PenStyle::default()
        });
        pencil.set_fidelity_px(fidelity);
        let path = hand(dy);
        pencil.on_press(&mut gfx.vec_scene, path[0], px_to_world);
        for &p in &path[1..] {
            pencil.on_drag(&mut gfx.vec_scene, p);
        }
        pencil.on_release(&mut gfx.vec_scene);
    }
}

/// Conta os nós que cada referência de fato tem (o número que a mensagem afirma sai da CENA, não
/// de uma tabela escrita à mão) e imprime o roteiro.
fn announce(app: &mut crate::App) {
    let counts: Vec<(VecPathId, usize)> = app
        .gfx
        .as_ref()
        .map(|g| {
            g.vec_scene
                .paths()
                .iter()
                .map(|p| (p.id, p.verts.len()))
                .collect()
        })
        .unwrap_or_default();
    let nodes: Vec<String> = counts.iter().map(|(_, n)| n.to_string()).collect();
    eprintln!(
        "[smoke] pencil: 2 tracos de REFERENCIA produzidos pelo motor a partir da MESMA mao \
         sintetica -- AZUL (acima) com Fidelity 0,5 px e LARANJA (abaixo) com 8 px. Nos: [{}]. \
         AGORA O GESTO: (1) na fileira TOOL do painel Vector clique **Pencil** -- se o chip nao \
         existir ou nao acender, PARE; (2) arraste no canvas: a curva tem de aparecer DESDE o \
         toque e seguir a mao (o ajuste e' ao vivo, nao ha troca no soltar); (3) solte: o traco \
         fica SELECIONADO; (4) Ctrl+Z desfaz o traco INTEIRO num passo; (5) comece um traco e \
         aperte o BOTAO DIREITO no meio: ele desaparece sem deixar rastro; (6) um CLIQUE sem \
         arrastar nao deixa nada; (7) troque para **Node** e confira que os nos sao poucos e \
         editaveis (o azul da tela mostra o extremo detalhista, o laranja o extremo liso).",
        nodes.join(", ")
    );
}
