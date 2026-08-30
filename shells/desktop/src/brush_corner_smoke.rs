//! **A cena das QUINAS do pincel de contorno** — `PH2D_BUILD_SMOKE=78` (plano 36, W5).
//!
//! ⭐⭐ **É a irmã da `=77`, e as duas existem por causa uma da outra.** Aquela é feita **só de
//! curvas suaves**, e a mensagem dela diz porquê: até 2026-08-30 um contorno com quina viva
//! mostrava as cópias a saltar no canto, e uma cena que exibisse isso estaria a ensinar um buraco
//! conhecido. Esta é a que prova que ele fechou — e por isso é feita **só de bicos**.
//!
//! # O que cada forma responde
//!
//! | forma | a pergunta |
//! |---|---|
//! | **quadrado** | a quina de 90°, o caso canónico |
//! | **estrela** | quinas **agudas** (~36°) e **reflexas** alternadas — o regime que mais dói |
//! | **triângulo** | três lados de comprimento igual e uma volta de 120° |
//! | **retângulo achatado** | lados de comprimentos **diferentes**, que é onde um avanço único
//!   para a volta inteira não consegue fechar em nenhum deles |
//!
//! ⚠️ **A quarta é a que separa as duas leis.** Num quadrado todos os lados medem o mesmo, então
//! um encaixe global e um por-lado dão quase a mesma coisa; num `12 × 4` eles divergem, e é ali
//! que se vê que o ritmo é **por lado**.

use ph2d_vec_scene::{
    BrushStroke, Paint, Rgba8, StrokePaint, StrokeSpec, VecPath, VecPathId, VecVertex,
};

/// O passo entre formas, em unidades de mundo.
const STEP: f64 = 3.2;
/// O «raio» de cada forma.
const RAIO: f64 = 1.25;
/// A largura da faixa do pincel — a arte tem esta altura.
const FAIXA: f64 = 0.30; // LITERAL-PX-OK: largura no domínio do documento
/// O contorno fino da arte.
const FIO: f64 = 0.02; // LITERAL-PX-OK: largura no domínio do documento

fn fio() -> StrokeSpec {
    StrokeSpec::new(Rgba8::new(35, 35, 45, 255), FIO)
}

/// A arte: um quadrilátero **assimétrico nos dois eixos**, o mesmo desenho da cena `=77` — para
/// que as duas se leiam como a mesma ferramenta em dois materiais.
fn arte(cx: f64, cy: f64) -> VecPath {
    VecPath {
        verts: [
            [cx - 0.50, cy - 0.10],
            [cx + 0.50, cy + 0.02],
            [cx - 0.10, cy + 0.30],
            [cx - 0.30, cy + 0.06],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(90, 190, 220, 255))),
        stroke: Some(fio()),
        ..VecPath::default()
    }
}

fn pincel(art: VecPathId) -> StrokeSpec {
    let mut s = StrokeSpec::new(Rgba8::new(60, 60, 80, 255), FAIXA);
    s.paint = StrokePaint::Brush(Box::new(BrushStroke {
        art: Some(art),
        fallback: Rgba8::new(60, 60, 80, 255),
        ..BrushStroke::default()
    }));
    s
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => select_hero(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;
    let x = |i: usize| -1.5 * STEP + (i as f64) * STEP;

    // ⭐ A ARTE nasce primeiro — as formas seguintes precisam do id dela.
    let art = scene.push_path(arte(x(1), -2.8));

    let mut formas = vec![
        // 1 — a quina de 90°, o caso canónico. **É o HERÓI** (já selecionado).
        ph2d_vec_scene::rectangle([x(0) - RAIO, -RAIO], [x(0) + RAIO, RAIO]),
        // 2 — quinas AGUDAS e REFLEXAS alternadas: o regime que mais dói.
        ph2d_vec_scene::star([x(1), 0.0], RAIO, RAIO, 5, 0.45),
        // 3 — três lados iguais, volta de 120°.
        ph2d_vec_scene::regular_polygon([x(2), 0.0], RAIO, RAIO, 3),
        // ⭐⭐ 4 — lados de comprimentos DIFERENTES: é aqui que um avanço único para a volta
        // inteira não fecha em lado nenhum, e o ritmo por-lado se vê.
        ph2d_vec_scene::rectangle(
            [x(3) - RAIO * 1.6, -RAIO * 0.5],
            [x(3) + RAIO * 1.6, RAIO * 0.5],
        ),
    ];
    for forma in &mut formas {
        forma.stroke = Some(pincel(art));
    }
    for forma in formas {
        scene.push_path(forma);
    }
}

/// Seleciona o RETÂNGULO — o painel abre com o chip **Brush** aceso e a secção *Brush* pintada.
///
/// ⚠️ **A primeira forma da cena é a ARTE**, então a selecção é pelo índice `1`.
fn select_hero(app: &mut crate::App) {
    let heroi: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().get(1).map(|p| p.id));
    if let Some(id) = heroi {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] AS QUINAS DO PINCEL (plano 36, W5). Quatro formas de BICOS, todas desenhadas com \
         a forma azul de baixo. \
         (1) QUADRADO (ja' selecionado): siga o contorno com o olho e passe por cada canto - a \
         arte tem de CHEGAR ao canto pelos dois lados, sem buraco e sem uma copia atravessada por \
         cima da esquina. \
         (2) ESTRELA: dez quinas, umas agudas e outras ao contrario - e' o caso mais dificil que \
         ha'. \
         (3) TRIANGULO: tres lados iguais. \
         (4) RETANGULO ACHATADO: os lados tem comprimentos DIFERENTES, e cada um fecha com o seu \
         proprio numero de copias. Compare o espacamento do lado longo com o do lado curto: eles \
         podem ser LIGEIRAMENTE diferentes, e isso e' correto - e' o que faz cada lado comecar e \
         acabar com uma copia inteira. \
         ⭐ COMPARE COM A CENA =77 (as curvas suaves): e' a mesma ferramenta. Antes de hoje esta \
         cena nao existia porque um contorno com bico mostrava as copias a saltar no canto. \
         ⚠️ COMO SABER QUE DEU ERRADO: um canto sem arte nenhuma (um buraco), ou uma copia \
         deitada POR CIMA da esquina, cortando o canto em vez de o contornar. \
         ⭐ E mexa: engrosse o Width na seccao Stroke, ou mexa no Spacing da seccao Brush - os \
         cantos tem de continuar limpos em qualquer valor."
    );
}

#[cfg(test)]
#[path = "brush_corner_smoke_tests.rs"]
mod tests;
