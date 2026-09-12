//! ⭐⭐⭐ **A OPACIDADE ALCANÇA AS TRÊS TINTAS** (plano 36, W6) — os gates da PONTE.
//!
//! O motor já responde (`ph2d-vec-scene/src/brush_opacity_tests.rs`); aqui prova-se o outro lado da
//! costura: que a barra *Opacity* do painel **chega** a cada espécie de tinta, e que quem DETECTA a
//! mudança concorda com quem a GRAVA.
//!
//! # Uma opacidade, uma casa
//!
//! | tinta | onde a opacidade vive | quem a escreve |
//! |---|---|---|
//! | `Solid` (traço e preenchimento) | a alfa da cor | o caminho de sempre |
//! | `Pattern` (traço) | `PatternFill::alpha`, `fallback.a` em sincronia | [`super::StrokeStyle::onto`] |
//! | `Pattern` (**preenchimento**) | idem | [`super::style::apply_fill_colour`] ⬅ **a W6** |
//! | `Brush` (traço) | `fallback.a`, e mais nada | [`super::StrokeStyle::onto`] |
//!
//! ⚠️ **As duas metades de um preenchimento de padrão são independentes**, e uma sem a outra é
//! pior que nenhuma: escrever sem SEMEAR faz a primeira mexida jogar na forma a alfa da forma
//! **anterior**; semear sem escrever deixa a barra a mostrar o valor certo e a não o mudar.

use super::style::{apply_fill_colour, seed_fill_from_paint};
use super::{StrokeStyle, restyle_selected_strokes};
use ph2d_vec_scene::{
    BrushStroke, Paint, PatternFill, PatternSource, Rgba8, StrokePaint, StrokeSpec, VecPath,
    VecPathId, VecScene, VecVertex,
};

const W: f64 = 0.05;

fn style(c: Rgba8) -> StrokeStyle {
    StrokeStyle {
        color: c,
        cap: ph2d_vec_scene::LineCap::Butt,
        join: ph2d_vec_scene::LineJoin::Miter,
        align: ph2d_vec_scene::StrokeAlign::Centre,
        dash: None,
        marker_start: ph2d_vec_scene::Marker::None,
        marker_end: ph2d_vec_scene::Marker::None,
        marker_scale: 1.0,
        marker_round: 0.0,
    }
}

/// Uma cena com uma forma cujo TRAÇO é um pincel.
fn cena_com_traco_de_pincel() -> (VecScene, VecPathId) {
    let mut s = StrokeSpec::new(Rgba8::new(10, 20, 30, 255), W);
    s.paint = StrokePaint::Brush(Box::new(BrushStroke {
        art: Some(VecPathId::from(1u64)),
        fallback: Rgba8::new(10, 20, 30, 255),
        ..BrushStroke::default()
    }));
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([1.0, 0.0])],
        stroke: Some(s),
        ..VecPath::default()
    });
    (scene, id)
}

fn pincel_de(scene: &VecScene, id: VecPathId) -> Option<BrushStroke> {
    match &scene.path(id)?.stroke.as_ref()?.paint {
        StrokePaint::Brush(b) => Some((**b).clone()),
        _ => None,
    }
}

/// Um preenchimento de PADRÃO, opaco, com a cor de recurso `c`.
fn padrao(c: Rgba8) -> Paint {
    Paint::Pattern(Box::new(PatternFill::new(
        PatternSource::Shape(VecPathId::from(1u64)),
        [1.0, 1.0],
        c,
    )))
}

fn padrao_de(fill: &Option<Paint>) -> Option<&PatternFill> {
    match fill.as_ref()? {
        Paint::Pattern(p) => Some(p),
        _ => None,
    }
}

// ─────────────────────────────── o TRAÇO de pincel ───────────────────────────────

/// ⭐⭐⭐ **A barra *Opacity* alcança um traço de PINCEL** — irmã do gate do padrão, e a mesma lei.
///
/// ⚠️ **A metade que faltava não era esta escrita, era o CONSUMIDOR.** A alfa já chegava à
/// `fallback` desde a W1; o que não existia era alguém a lê-la para desvanecer as cópias, e é isso
/// que a W6 acrescentou no motor. Este gate prende a metade da ponte para que ela não se perca
/// numa reescrita futura do `onto`.
#[test]
fn the_stroke_opacity_reaches_a_brushed_stroke() {
    let (mut scene, id) = cena_com_traco_de_pincel();
    let meia = Rgba8::new(11, 22, 33, 128);
    restyle_selected_strokes(&mut scene, &[id], &style(meia), None);
    let b = pincel_de(&scene, id).expect("o pincel sobrevive");
    assert_eq!(
        b.fallback.a, 128,
        "a barra Opacity nao alcanca o pincel - ela anda e nao muda um pixel"
    );
    assert_eq!(b.fallback, meia, "a cor de recurso tambem tem de a seguir");
    assert!(
        b.art.is_some(),
        "a cor TROCOU a tinta - a porta de sair do pincel e' a fileira Type, e so' ela"
    );
    // CONTROLO: opaco devolve o pincel a cheio — senão o gate ficaria verde sobre uma porta que
    // escreve a alfa uma vez e nunca mais a levanta.
    restyle_selected_strokes(&mut scene, &[id], &style(Rgba8::new(11, 22, 33, 255)), None);
    assert_eq!(
        pincel_de(&scene, id)
            .expect("o pincel sobrevive")
            .fallback
            .a,
        255,
        "a opacidade nao volta a subir"
    );
}

/// ⚠️ **O detector concorda com o escritor num traço de pincel** — sem isto o restyle re-dispara
/// todo quadro, e **cada quadro vira um passo de undo**.
#[test]
fn the_detector_agrees_with_the_writer_on_a_brushed_stroke() {
    let (mut scene, id) = cena_com_traco_de_pincel();
    let ficha = style(Rgba8::new(11, 22, 33, 128));
    restyle_selected_strokes(&mut scene, &[id], &ficha, None);
    let spec = scene
        .path(id)
        .and_then(|p| p.stroke.clone())
        .expect("o traco existe");
    assert!(
        !ficha.differs_from(&spec),
        "o detector continua a ver diferenca depois de escrever - o restyle corre todo quadro"
    );
}

// ────────────────────── o PREENCHIMENTO de padrão (as duas metades) ──────────────────────

/// ⭐⭐⭐ **A barra *Fill Opacity* alcança um preenchimento de PADRÃO.**
///
/// ⚠️ **A guarda que a bloqueava estava CERTA e incompleta.** *"Um pick só substitui um Solid /
/// vazio; nunca esmaga um gradiente"* (Enio, 2026-07-08) mantinha o padrão a salvo de ser trocado
/// por uma cor — e, na mesma linha, deixava-o sem NENHUM caminho de escrita. *Uma cerca que protege
/// uma tinta de ser destruída não é a mesma coisa que uma tinta ser editável.*
#[test]
fn the_fill_opacity_reaches_a_patterned_fill() {
    let mut fill = Some(padrao(Rgba8::new(9, 9, 9, 255)));
    let meia = Rgba8::new(77, 88, 99, 128);
    assert!(
        apply_fill_colour(&mut fill, true, meia),
        "o pick nao mexeu no padrao"
    );
    let p = padrao_de(&fill).expect("o padrao SOBREVIVE ao pick");
    assert!(
        (p.alpha - 128.0 / 255.0).abs() < 1e-6,
        "a barra Fill Opacity nao alcanca o padrao (alpha={}) - ela anda e nao muda um pixel",
        p.alpha
    );
    assert_eq!(
        p.fallback, meia,
        "a cor de recurso ficou fora de sincronia - o instante pre-resolucao mostraria outra coisa"
    );
    // CONTROLO: volta a subir.
    apply_fill_colour(&mut fill, true, Rgba8::new(77, 88, 99, 255));
    assert!((padrao_de(&fill).expect("sobrevive").alpha - 1.0).abs() < 1e-6);
}

/// ⭐⭐ **ALFA ZERO NÃO APAGA UM PADRÃO** — ele fica invisível, e volta.
///
/// ⚠️ **Num preenchimento SÓLIDO alfa zero significa «sem preenchimento»** e a tinta desaparece —
/// é a convenção que a ponte já usa nos dois sentidos, e não é desta wave mudá-la. Num padrão a
/// mesma leitura destruiria a grade, o ladrilho, a rotação e a fonte por um arrasto acidental até
/// ao fundo da barra. *Uma convenção herdada aplica-se onde ela não custa nada; onde custa, ela é a
/// pergunta.*
#[test]
fn dragging_a_patterns_opacity_to_zero_hides_it_instead_of_deleting_it() {
    let mut fill = Some(padrao(Rgba8::new(9, 9, 9, 255)));
    apply_fill_colour(&mut fill, true, Rgba8::new(9, 9, 9, 0));
    let p = padrao_de(&fill).expect("o padrao SOBREVIVE a opacidade zero");
    assert!(p.alpha.abs() < 1e-6, "invisivel, mas la'");
    apply_fill_colour(&mut fill, true, Rgba8::new(9, 9, 9, 255));
    assert!(
        (padrao_de(&fill).expect("sobrevive").alpha - 1.0).abs() < 1e-6,
        "o padrao nao volta - o arrasto ate' ao fundo foi destrutivo"
    );
    // CONTROLO: num SÓLIDO a mesma alfa zero continua a apagar o preenchimento.
    let mut solido = Some(Paint::solid(Rgba8::new(9, 9, 9, 255)));
    apply_fill_colour(&mut solido, true, Rgba8::new(9, 9, 9, 0));
    assert!(solido.is_none(), "a convencao do solido mudou sem se pedir");
}

/// ⛔⛔ **UM GRADIENTE CONTINUA INTOCÁVEL** — a cerca de 2026-07-08, com um gate por cima.
///
/// A W6 abriu uma porta de escrita para o padrão; se ela tivesse sido escrita como *"tudo o que não
/// é sólido recebe a cor"*, teria reaberto a regressão que aquele report fechou.
#[test]
fn a_gradient_fill_is_still_never_clobbered_by_a_pick() {
    let grad = Paint::Linear {
        start: [0.0, 0.0],
        end: [1.0, 0.0],
        stops: vec![
            ph2d_vec_scene::GradientStop::new(0.0, Rgba8::new(255, 0, 0, 255)),
            ph2d_vec_scene::GradientStop::new(1.0, Rgba8::new(0, 0, 255, 255)),
        ],
    };
    let mut fill = Some(grad.clone());
    assert!(
        !apply_fill_colour(&mut fill, true, Rgba8::new(1, 2, 3, 128)),
        "o pick diz ter mudado um gradiente"
    );
    assert_eq!(fill, Some(grad), "o gradiente foi esmagado por um pick");
}

/// ⚠️ **Um caminho ABERTO não recebe preenchimento** — a guarda `closed` da ponte, preservada.
#[test]
fn an_open_path_takes_no_fill() {
    let mut fill = None;
    assert!(!apply_fill_colour(
        &mut fill,
        false,
        Rgba8::new(1, 2, 3, 255)
    ));
    assert!(fill.is_none());
}

/// ⭐⭐⭐ **A OUTRA METADE: escolher uma forma de padrão SEMEIA a barra com a opacidade dela.**
///
/// ⛔⛔ **Sem isto a metade de cima é PIOR que o buraco.** O `seed_style_from_selection` deixava um
/// padrão passar (`Some(_) => {}`), então a ferramenta ficava com a cor e a alfa da forma
/// ANTERIOR; com a porta de escrita aberta, a primeira mexida em qualquer controlo jogaria essa
/// alfa alheia no padrão — e o artista veria a forma saltar de opacidade sem ter tocado na barra.
///
/// ⚠️ **A semente lê `alpha`, e não `fallback.a`.** As duas ficam em sincronia por esta ponte, mas
/// um documento gravado antes da W6 tem `alpha` autorado noutro sítio e a `fallback` opaca — e o
/// que se DESENHA é o `alpha`. *Semear pelo campo que não se vê poria a barra a mentir sobre o
/// ficheiro do artista.*
#[test]
fn selecting_a_patterned_fill_seeds_the_opacity_from_what_is_drawn() {
    let mut pat = PatternFill::new(
        PatternSource::Shape(VecPathId::from(1u64)),
        [1.0, 1.0],
        Rgba8::new(70, 80, 90, 255), // a `fallback` OPACA, como num ficheiro pré-W6
    );
    pat.alpha = 0.25; // e o que de facto se desenha
    let semeada = seed_fill_from_paint(Some(&Paint::Pattern(Box::new(pat))));
    assert_eq!(
        semeada,
        Some([70, 80, 90, 64]),
        "a semente leu a `fallback` em vez do que se desenha - a barra mentiria sobre o ficheiro"
    );
    // CONTROLO: um sólido continua a semear a própria cor, e um gradiente continua QUIETO.
    assert_eq!(
        seed_fill_from_paint(Some(&Paint::solid(Rgba8::new(1, 2, 3, 4)))),
        Some([1, 2, 3, 4])
    );
    assert_eq!(seed_fill_from_paint(None), Some([0, 0, 0, 0]));
    assert_eq!(
        seed_fill_from_paint(Some(&Paint::Linear {
            start: [0.0, 0.0],
            end: [1.0, 0.0],
            stops: Vec::new(),
        })),
        None,
        "um gradiente tem alca propria - esmaga-lo numa cor so' seria a selecao a destruir autoria"
    );
}

/// ⚠️ **A semente e a escrita fecham o CICLO**: semear de um padrão e voltar a escrever o que se
/// semeou não muda nada. Sem isto, escolher uma forma e não lhe tocar registaria um passo de undo.
#[test]
fn seeding_a_pattern_and_writing_it_back_changes_nothing() {
    let mut pat = PatternFill::new(
        PatternSource::Shape(VecPathId::from(1u64)),
        [1.0, 1.0],
        Rgba8::new(70, 80, 90, 200),
    );
    pat.alpha = 200.0 / 255.0;
    let mut fill = Some(Paint::Pattern(Box::new(pat)));
    let semeada = seed_fill_from_paint(fill.as_ref()).expect("um padrao semeia");
    assert!(
        !apply_fill_colour(
            &mut fill,
            true,
            Rgba8::new(semeada[0], semeada[1], semeada[2], semeada[3])
        ),
        "escrever de volta o que se acabou de semear diz ter mudado - todo quadro vira undo"
    );
}
