//! **A cena pronta para o smoke do PINCEL DE CONTORNO** — `PH2D_BUILD_SMOKE=77` (plano 36).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `texture_pattern_smoke`.
//!
//! ⭐⭐ **É a cena do MODELO B**, e a irmã dela (`=76`) é a do modelo A. As duas existem lado a lado
//! de propósito: um padrão é uma **TINTA que o contorno revela** (normativo em SVG 2, e num
//! tracejado os traços são *buracos* no papel de parede); um pincel é uma **ARTE que percorre a
//! linha**. Elas respondem ao contrário às mesmas três perguntas, e é por isso que todo aplicativo
//! sério entrega as duas ([plano 36 §1](../../docs/Vector%20Module/36_plano_pincel_de_contorno.md)).
//!
//! ⚠️ **A ARTE É UMA FORMA DO DOCUMENTO**, e fica visível ao lado: o pincel copia geometria, então
//! não há diálogo de ficheiro a abrir — o gesto é *Pick Shape…* e um clique na forma. Mexer nos nós
//! dela muda todas as cópias na hora.
//!
//! ⚠️ **AS CURVAS SÃO SUAVES de propósito.** As QUINAS são a wave seguinte (§4 W5, os quatro modos
//! do Illustrator medidos antes de escolher o nosso), e uma cena que as exibisse mostraria um
//! buraco conhecido como se fosse um defeito. *Uma cena de smoke escolhe o que deixa a feature
//! visível, não o que exercita mais código.*

use ph2d_vec_scene::{
    BrushStroke, Contour, FillRule, Paint, Rgba8, StrokePaint, StrokeSpec, VecPath, VecPathId,
    VecVertex,
};

/// O lado de cada forma, em unidades de mundo — a mesma régua da cena irmã (`=76`).
const BOX: f64 = 2.2;
/// O passo entre formas.
const STEP: f64 = 2.6;

/// A largura da faixa do pincel, em unidades de mundo.
///
/// ⚠️ **Ela É o tamanho da arte** (`altura = largura × Size`), e é isso que separa este modelo do
/// padrão: ali a largura decide a faixa e o motivo tem tamanho próprio; aqui a largura **é** o
/// motivo. O valor põe cada cópia a ~16 % do diâmetro da forma — grande o bastante para se ver
/// **o que** repete, pequeno o bastante para se ver **que** repete.
const FAIXA: f64 = 0.35; // LITERAL-PX-OK: largura no domínio do documento

/// A largura do contorno fino das formas que não são pincel (a arte, a moldura da demo).
const FIO: f64 = 0.02; // LITERAL-PX-OK: largura no domínio do documento

/// O tracejado da 2ª forma, em MÚLTIPLOS da largura (é assim que o `StrokeSpec` o guarda).
///
/// ⚠️ **Escolhido para caber MAIS DE UMA cópia por traço, e para caber mais de dois traços na
/// volta.** Com uma cópia só por traço, *"a arte reinicia em cada traço"* e *"há uma bolha em cada
/// traço"* desenham a mesma coisa — e o smoke deixaria de distinguir a lei que existe para provar.
///
/// ⚠️ **O número saiu da conta, não do gosto** (e a 1.ª escolha, `(3, 2)`, dava exactamente uma
/// cópia por traço): a arte mede `0,875` de largura depois de escalada para a faixa, a volta da
/// elipse mede `6,91`, e `(5, 2½)` põe **3 traços de 2 cópias** com um terço da volta vazio. O gate
/// `the_smoke_dash_carries_more_than_one_copy_per_dash` guarda as duas metades.
const TRACEJADO: (f64, f64) = (5.0, 2.5);

fn fio() -> StrokeSpec {
    StrokeSpec::new(Rgba8::new(35, 35, 45, 255), FIO)
}

/// ⭐ **A ARTE**: um quadrilátero assimétrico nos DOIS eixos.
///
/// ⚠️ **Nos dois**, e não só num: a assimetria em `x` é o que deixa ver a **Rotation** (o motivo
/// deita-se ou põe-se de pé sobre a curva), e a assimetria em `y` é o que deixa ver o **Flip** (a
/// arte passa para o outro lado da linha). Um losango simétrico esconderia as duas.
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

/// O traço-pincel que nomeia `art`.
fn pincel(art: VecPathId, dash: Option<(f64, f64)>, rotation_deg: f64, flip: bool) -> StrokeSpec {
    let mut s = StrokeSpec::new(Rgba8::new(60, 60, 80, 255), FAIXA);
    s.paint = StrokePaint::Brush(Box::new(BrushStroke {
        art: Some(art),
        // ⚠️ A `fallback` é a cor que a linha pinta **enquanto não há arte resolvida** — ela não é
        // a cor das cópias, que herdam o fill/stroke da própria arte (a lei do *Pattern Brush*:
        // a arte guarda as cores dela).
        fallback: Rgba8::new(60, 60, 80, 255),
        rotation_deg,
        flip,
        ..BrushStroke::default()
    }));
    s.dash = dash;
    s
}

/// Uma onda ABERTA — o contorno sem emenda, em que as duas pontas são o sujeito.
fn onda(cx: f64, cy: f64, half: f64) -> Vec<VecVertex> {
    let pts: Vec<[f64; 2]> = (0..=8)
        .map(|i| {
            let t = f64::from(i) / 8.0;
            let x = cx - half + t * half * 2.0;
            let y = cy + (t * std::f64::consts::TAU).sin() * half * 0.45;
            [x, y]
        })
        .collect();
    ph2d_vec_scene::smooth_polyline(&pts, 1.0 / 3.0)
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
    let half = BOX * 0.5;
    let x = |i: usize| -2.0 * STEP + (i as f64) * STEP;

    // ⭐ A ARTE nasce PRIMEIRO — as formas seguintes precisam do id dela. Ela fica visível e
    // editável: é uma forma do documento como qualquer outra.
    let art = scene.push_path(arte(x(1), -3.0));

    // 1 — o caso base: a arte corre pelo contorno de uma elipse. **É o HERÓI** (já selecionado).
    let mut e = ph2d_vec_scene::ellipse([x(0), 0.0], half, half);
    e.stroke = Some(pincel(art, None, 0.0, false));
    scene.push_path(e);

    // ⭐⭐ 2 — **O TRACEJADO**: a arte REINICIA em cada traço (a lei do *Pattern Brush*, e a
    // pergunta do Enio ao juntar as duas coisas). Os vãos ficam VAZIOS — nenhuma cópia lá dentro.
    let mut d = ph2d_vec_scene::ellipse([x(1), 0.0], half, half);
    d.stroke = Some(pincel(art, Some(TRACEJADO), 0.0, false));
    scene.push_path(d);

    // 3 — um contorno ABERTO. Sem emenda, mas com duas PONTAS: o encaixe põe cópias inteiras de
    // ponta a ponta, em vez de deixar meia cópia pendurada.
    scene.push_path(VecPath {
        verts: onda(x(2), 0.0, half),
        closed: false,
        stroke: Some(pincel(art, None, 0.0, false)),
        ..VecPath::default()
    });

    // ⚠️ 4 — um COMPOSTO: **cada contorno recebe as suas cópias e fecha exactamente**. O tracejado
    // não pode fazer isto (ele tem um par `[traço, vão]` só para o caminho inteiro, fitado ao anel
    // mais longo); o encaixe do avanço não herda essa limitação.
    let mut c = ph2d_vec_scene::ellipse([x(3), 0.0], half, half);
    let furo = ph2d_vec_scene::ellipse([x(3), 0.0], half * 0.5, half * 0.5);
    c.subpaths.push(Contour {
        verts: furo.verts,
        closed: true,
    });
    c.fill_rule = FillRule::EvenOdd;
    c.stroke = Some(pincel(art, None, 0.0, false));
    scene.push_path(c);

    // ⭐ 5 — a MESMA arte, de pé e do outro lado: `Rotation = 90` e `Flip` ligado. Está aqui para o
    // Enio ver o que os dois botões fazem ANTES de lhes tocar.
    let mut r = ph2d_vec_scene::ellipse([x(3), -3.0], half, half);
    r.stroke = Some(pincel(art, None, 90.0, true));
    scene.push_path(r);
}

/// Seleciona a PRIMEIRA elipse — o painel abre com o chip **Brush** aceso e a secção *Brush*
/// pintada.
///
/// ⚠️ **A primeira forma da cena é a ARTE, não o herói** (ela nasce primeiro porque as outras
/// precisam do id dela) — então a selecção é pelo índice `1`, e não por `first()`. *Um herói
/// escolhido por posição envelhece na primeira forma que se acrescente antes dele.*
fn select_hero(app: &mut crate::App) {
    let heroi: Option<VecPathId> = app
        .gfx
        .as_ref()
        .and_then(|g| g.vec_scene.paths().get(1).map(|p| p.id));
    if let Some(id) = heroi {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] PINCEL DE CONTORNO (plano 36). A forma da ESQUERDA ja' esta' selecionada, e na \
         seccao Stroke a fileira **Type** mostra tres opcoes: Solid | Pattern | **Brush** - com a \
         terceira acesa. \
         (1) O CIRCULO DA ESQUERDA: o contorno dele nao e' uma linha, sao COPIAS da forma azul de \
         baixo, uma atras da outra, cada uma virada para a curva. \
         (2) O SEGUNDO CIRCULO tem TRACEJADO: a arte REINICIA em cada traco, e os vaos ficam \
         VAZIOS. E' a resposta 'sim' a' pergunta 'nao posso usar o dash com pattern?'. \
         (3) A ONDA: um contorno ABERTO - as copias cabem inteiras de ponta a ponta. \
         (4) O ANEL: um contorno com FURO - o de fora e o de dentro recebem cada um as suas \
         copias, e cada um FECHA (nao ha' uma copia curta encostada a uma longa na emenda). \
         (5) EM BAIXO A' DIREITA: a MESMA arte com Rotation = 90 e Flip ligado - de pe' e do outro \
         lado da linha. \
         ⭐ A ARTE E' A FORMA AZUL EM BAIXO, no meio. Pegue na ferramenta Node e mexa nos nos dela: \
         TODAS as copias mudam na hora. \
         ⭐ NA SECCAO **Brush** do painel: Size (o tamanho da arte, que segue a largura do traco), \
         Spacing (perto ou longe), Rotation, Offset (para dentro ou para fora da linha) e Flip. \
         O botao 'Change Shape...' arma o gesto: clique nele e depois clique noutra forma da cena \
         - ela passa a ser a arte. \
         ⭐ ENGROSSE a barra Width da seccao Stroke: a ARTE cresce junto. (No modelo da cena =76, \
         que e' a estampa, engrossar NAO muda o tamanho do motivo - sao duas leis, e as duas estao \
         certas.) \
         ⚠️ AS QUINAS VIVAS ainda nao sao tratadas: por isso esta cena e' feita de curvas suaves. \
         Um quadrado com pincel mostra copias a saltar nos cantos - e' a wave seguinte."
    );
}

#[cfg(test)]
#[path = "brush_smoke_tests.rs"]
mod tests;
