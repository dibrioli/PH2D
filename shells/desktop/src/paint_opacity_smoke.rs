//! ⭐⭐⭐ **A CENA DA OPACIDADE das duas tintas** — `PH2D_BUILD_SMOKE=79` (plano 36, W6).
//!
//! # O que ela prova, e por que precisa de DUAS fileiras
//!
//! A barra *Opacity* tem uma casa diferente em cada espécie de tinta, e até 2026-08-30 **duas
//! delas não tinham consumidor**: numa estampa a barra do preenchimento não era sequer escrita
//! (a guarda que protegia o padrão de ser esmagado também o deixava sem porta de escrita), e num
//! pincel ela era escrita e **ninguém a lia** — as cópias saíam sempre opacas.
//!
//! | fileira | tinta | a barra que a comanda |
//! |---|---|---|
//! | **cima** | `Paint::Pattern` (estampa no preenchimento) | *Fill > Opacity* |
//! | **baixo** | `StrokePaint::Brush` (arte que percorre a linha) | *Stroke > Opacity* |
//!
//! ⚠️ **A barra CLARA por trás de cada fileira é load-bearing.** Sobre um fundo neutro, «mais
//! transparente» e «mais escuro» desenham-se quase igual, e a cena provaria o defeito tão bem
//! quanto a cura. Com uma faixa clara atrás, o que se vê é a faixa a **atravessar** a arte.
//!
//! ⚠️ **As três colunas são a MESMA tinta em três opacidades** — `100 %`, `50 %`, `15 %`. É a
//! metade estática (vê-se antes de tocar em nada); a metade viva é o artista arrastar a barra e
//! ver a forma seguir, sem a estampa virar cor chapada nem o pincel virar linha lisa.

use ph2d_vec_pattern::{PatternMode, TileKind};
use ph2d_vec_scene::{
    BrushStroke, Paint, PatternFill, PatternSource, Rgba8, StrokePaint, StrokeSpec, VecPath,
    VecPathId, VecVertex,
};

/// O lado da arte da estampa, em pixels.
const ART: u32 = 32;
/// O lado de cada forma, em unidades de mundo.
const BOX: f64 = 1.8;
/// O passo entre colunas.
const STEP: f64 = 2.4;
/// A altura (em `y`) de cada fileira, e a da arte do pincel.
///
/// ⚠️ **Os três números são um ENQUADRAMENTO, não gosto.** A cena inteira tem de caber na caixa que
/// a câmera de omissão mostra, e a referência medida é a [`crate::texture_pattern_smoke`] (`=76`),
/// que ocupa `y ∈ [−4,1 ; 1,1]` e é smokada. ⛔ Uma cena que nasce meio fora do quadro lê-se como
/// *"faltam formas"*, e o artista não tem como distinguir isso de um defeito da feature.
///
/// ⚠️ **O topo é da FAIXA CLARA, não das formas** — ela é `1,2 ×` a altura delas, e a 1.ª redacção
/// destes três números esqueceu-o: a fileira de cima cabia e a faixa por trás dela saía `0,58` fora.
/// O gate `the_whole_scene_fits_inside_the_box_a_shipped_sibling_proves_visible` apanhou-o.
const ROW_PAT: f64 = 0.0;
const ROW_BRUSH: f64 = -2.35;
const ROW_ART: f64 = -3.9;
/// A largura do contorno fino que toda forma leva.
const FIO: f64 = 0.02; // LITERAL-PX-OK: largura no domínio do documento
/// A largura da faixa do pincel — a arte tem esta altura.
const FAIXA: f64 = 0.34; // LITERAL-PX-OK: largura no domínio do documento

/// ⭐ **As três opacidades da cena, em alfa de `0..=255`.** `100 %` é o controlo: se a coluna da
/// esquerda também desvanecer, a cura escureceu tudo por sistema em vez de obedecer à barra.
const ALFAS: [u8; 3] = [255, 128, 38];

/// ⛔ **Toda forma desta cena nasce COM contorno** — a ferramenta de forma escreve sempre um, e uma
/// cena montada por código que não o faça mede um objecto que o produto nunca produz (a lição que a
/// `=76` pagou com um report).
fn fio(a: u8) -> StrokeSpec {
    StrokeSpec::new(Rgba8::new(35, 35, 45, a), FIO)
}

fn rect(cx: f64, cy: f64, hx: f64, hy: f64) -> Vec<VecVertex> {
    [
        [cx - hx, cy - hy],
        [cx + hx, cy - hy],
        [cx + hx, cy + hy],
        [cx - hx, cy + hy],
    ]
    .map(VecVertex::corner)
    .to_vec()
}

/// A arte da estampa: xadrez de duas cores com uma barra, **opaca por inteiro**.
///
/// ⚠️ **Sem quadrante transparente, ao contrário da [`crate::texture_pattern_smoke`]** — ali o alfa
/// da arte é o sujeito; aqui ele seria uma **segunda** fonte de transparência a somar-se à que a
/// cena mede, e não haveria como ler qual delas moveu o pixel.
fn art_rgba() -> Vec<u8> {
    let mut px = Vec::with_capacity((ART * ART * 4) as usize);
    for y in 0..ART {
        for x in 0..ART {
            let c = if y < ART / 6 {
                [230u8, 140, 60, 255]
            } else if (x / 8 + y / 8) % 2 == 0 {
                [70, 120, 210, 255]
            } else {
                [235, 232, 225, 255]
            };
            px.extend_from_slice(&c);
        }
    }
    px
}

/// A lei da estampa, com a opacidade `a` na casa dela — **e a `fallback` em sincronia**, que é
/// exactamente o que a ponte do painel escreve.
fn estampa(source: PatternSource, cx: f64, a: u8) -> Paint {
    let half = BOX * 0.5;
    let mut f = PatternFill::new(source, [BOX / 3.0, BOX / 3.0], Rgba8::new(90, 90, 110, a));
    f.kind = TileKind::Grid;
    f.mode = PatternMode::Tile;
    // O canto da FORMA, como a autoria real ancora (`texture_pattern_pick::default_placement`).
    f.origin = [cx - half, ROW_PAT - half];
    f.alpha = f32::from(a) / 255.0;
    Paint::Pattern(Box::new(f))
}

/// A arte do pincel: um quadrilátero assimétrico nos dois eixos — o mesmo desenho das cenas `=77`
/// e `=78`, para as três se lerem como a mesma ferramenta.
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
        stroke: Some(fio(255)),
        ..VecPath::default()
    }
}

/// ⭐ **A casa da opacidade de um pincel é a alfa da cor de RECURSO** — e mais nada. Um campo
/// `alpha` próprio como o do padrão seria uma segunda casa para o mesmo número; ali ele existe
/// porque o amostrador quer um `f32`, e um pincel não tem amostrador.
fn pincel(art: VecPathId, a: u8) -> StrokeSpec {
    let cor = Rgba8::new(60, 60, 80, a);
    let mut s = StrokeSpec::new(cor, FAIXA);
    s.paint = StrokePaint::Brush(Box::new(BrushStroke {
        art: Some(art),
        fallback: cor,
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
    let source = PatternSource::Image(gfx.asset_db.insert_image_rgba8(ART, ART, art_rgba()));
    populate(&mut gfx.vec_scene, source);
}

/// ⭐⭐ **A CENA, sem a `App`** — extraída para que os gates meçam **o que o produto empilha** e não
/// uma reconstrução paralela.
///
/// ⚠️ A `=78` prova a mesma lei por reconstrução, e é ali que ela é fraca: um gate que rebuild a
/// fixtura mede o que ele próprio escreveu. Aqui a ordem de empilhamento (as faixas por baixo) e a
/// escolha do herói são propriedades da CENA, e só se medem se a cena for alcançável.
pub(crate) fn populate(scene: &mut ph2d_vec_scene::VecScene, source: PatternSource) {
    let half = BOX * 0.5;
    let x = |i: usize| -STEP + (i as f64) * STEP;

    // ⭐⭐ **AS DUAS FAIXAS CLARAS, primeiro** — sem elas «transparente» e «escuro» desenham-se
    // igual, e a cena aprovaria o defeito. Elas entram antes de tudo para ficarem POR BAIXO.
    for cy in [ROW_PAT, ROW_BRUSH] {
        scene.push_path(VecPath {
            verts: rect(0.0, cy, STEP * 1.55, half * 1.2),
            closed: true,
            fill: Some(Paint::Solid(Rgba8::new(238, 236, 230, 255))),
            ..VecPath::default()
        });
    }

    // ⭐ A ARTE do pincel nasce antes das formas que a nomeiam.
    let art = scene.push_path(arte(x(1), ROW_ART));

    for (i, a) in ALFAS.into_iter().enumerate() {
        // Fileira de cima — a ESTAMPA. O HERÓI é a primeira (opaca).
        scene.push_path(VecPath {
            verts: rect(x(i), ROW_PAT, half, half),
            closed: true,
            fill: Some(estampa(source, x(i), a)),
            stroke: Some(fio(255)),
            ..VecPath::default()
        });
        // Fileira de baixo — o PINCEL. ⚠️ Sem preenchimento: o sujeito é a faixa.
        scene.push_path(VecPath {
            verts: rect(x(i), ROW_BRUSH, half, half * 0.55),
            closed: true,
            fill: None,
            stroke: Some(pincel(art, a)),
            ..VecPath::default()
        });
    }
}

/// ⭐⭐ **O HERÓI é DERIVADO, nunca um índice** — *a primeira forma com estampa*.
///
/// ⚠️ As faixas claras e a arte do pincel nascem antes das formas, então o índice literal seria `3`
/// hoje e outro amanhã: acrescentar uma faixa faria a cena abrir com o painel na secção errada, sem
/// erro nenhum. É a mesma armadilha que as quatro constantes `SHAPES.len() − N` do catálogo 3D
/// custaram, e a cura é a mesma — *perguntar pelo FACTO em vez de contar posições*.
pub(crate) fn hero_of(scene: &ph2d_vec_scene::VecScene) -> Option<VecPathId> {
    scene
        .paths()
        .iter()
        .find(|p| matches!(p.fill, Some(Paint::Pattern(_))))
        .map(|p| p.id)
}

/// Seleciona a ESTAMPA opaca — o painel abre com a secção *Fill* pintada e a barra *Opacity* no
/// topo, que é o primeiro gesto que a mensagem pede.
fn select_hero(app: &mut crate::App) {
    let heroi = app.gfx.as_ref().and_then(|g| hero_of(&g.vec_scene));
    if let Some(id) = heroi {
        app.vec_pen.select_many(&[id]);
    }
    eprintln!(
        "[smoke] A OPACIDADE DAS DUAS TINTAS (plano 36, W6). Duas fileiras de tres formas sobre \
         faixas claras -- a faixa por tras e' o que deixa a transparencia visivel. \
         EM CIMA: a mesma ESTAMPA em 100%, 50% e 15%. \
         EM BAIXO: o mesmo PINCEL (a forma azul la' de baixo a percorrer a linha) nas mesmas tres. \
         (1) OLHE antes de tocar: as tres colunas tem de ficar cada vez mais claras, com a faixa \
         branca a APARECER atraves' delas. A da esquerda e' o controlo: ela tem de estar cheia. \
         (2) A forma de cima a' esquerda ja' esta' selecionada. Na seccao Fill, arraste a barra \
         OPACITY: a estampa tem de desvanecer ao vivo -- e continuar a ser uma ESTAMPA, nunca virar \
         uma cor chapada. \
         (3) Clique numa forma da fileira de BAIXO e arraste a barra OPACITY da seccao STROKE: as \
         copias da arte tem de desvanecer -- e continuar copias, nunca virar uma linha lisa. \
         (4) Arraste qualquer uma das duas ate' ao FUNDO e volte a subir: a forma tem de \
         reaparecer exactamente como estava. \
         ⚠️ COMO SABER QUE DEU ERRADO: a barra anda e a forma nao muda; a estampa vira uma cor \
         solida; o pincel vira uma linha sem arte; ou levar ao fundo e voltar nao devolve o que la' \
         estava."
    );
}

#[cfg(test)]
#[path = "paint_opacity_smoke_tests.rs"]
mod tests;
