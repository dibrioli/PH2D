//! ⭐⭐ **O SUJEITO DE CADA ESCRITA** — os gates de *qual das duas tintas recebe* (plano 35, wave F).
//!
//! Irmão do [`super::tests`], e o corte é por RESPONSABILIDADE: aquele mede o que um comando faz
//! à LEI de um padrão (cadeado, ângulo, desfasamento, undo); este mede **em quem** ele escreve —
//! o preenchimento e o traço são secções independentes, e uma escrita no sujeito errado é um
//! defeito que nenhum gate de lei vê.

use super::tests::fill;
use super::*;
use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecVertex};

/// Uma forma com padrão no preenchimento (`no_fill`) e/ou no traço (`no_traco`).
///
/// ⚠️ **Os dois padrões são DIFERENTES** (`size` `[8,2]` contra `[3,5]`): com dois iguais, entregar
/// o padrão errado dá o resultado certo por acidente — a mesma armadilha que a chave do memo da
/// wave C teve de evitar.
fn cena_alvos(no_fill: bool, no_traco: bool) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId) {
    let mut scene = VecScene::default();
    let cor = Rgba8::new(1, 2, 3, 255);
    let mut s = ph2d_vec_scene::StrokeSpec::new(cor, 0.5);
    if no_traco {
        s.paint = ph2d_vec_scene::StrokePaint::Pattern(Box::new(PatternFill::new(
            PatternSource::Shape(2),
            [3.0, 5.0],
            cor,
        )));
    }
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: no_fill
            .then(|| Paint::Pattern(Box::new(fill())))
            .or_else(|| Some(Paint::solid(cor))),
        stroke: Some(s),
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, id)
}

/// ⭐⭐ **CADA SECÇÃO ESCREVE SÓ NA SUA TINTA** (gate nº 6 do plano 35 §4, na forma da wave F).
///
/// ⚠️ **O controle é a outra metade**: com o preenchimento aceso, o traço tem de ficar INTACTO. Sem
/// ele, uma implementação que escrevesse nos dois passaria.
#[test]
fn each_section_writes_only_its_own_paint() {
    let (mut scene, pen, id) = cena_alvos(true, true);
    let mut h = ph2d_vec_edit::History::default();
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Stroke,
        TexPatCmd::Angle(30.0),
    );
    assert!(
        (pattern_at(&scene, id, PatternSlot::Stroke)
            .expect("o traco tem padrao")
            .angle
            - 30f64.to_radians())
        .abs()
            < 1e-12,
        "o angulo nao entrou no TRACO - a seccao editou o outro sujeito"
    );
    assert_eq!(
        pattern_at(&scene, id, PatternSlot::Fill)
            .expect("o preenchimento tem padrao")
            .angle,
        0.0,
        "escrever no traco mexeu tambem no PREENCHIMENTO"
    );
    // E o simétrico, com o outro alvo aceso.
    apply(
        &mut scene,
        &mut h,
        &pen,
        PatternSlot::Fill,
        TexPatCmd::Angle(45.0),
    );
    assert!(
        (pattern_at(&scene, id, PatternSlot::Stroke)
            .expect("o traco tem padrao")
            .angle
            - 30f64.to_radians())
        .abs()
            < 1e-12,
        "escrever no preenchimento mexeu tambem no TRACO"
    );
}

/// ⚠️ **A troca de ARTE também honra o SUJEITO** — o botão *Source…* e o picker de forma da
/// secção do traço escrevem no traço, e não sempre no preenchimento.
#[test]
fn changing_the_art_honours_the_subject_too() {
    let (mut scene, _, id) = cena_alvos(true, true);
    let mut h = ph2d_vec_edit::History::default();
    assert!(set_source(
        &mut scene,
        &mut h,
        id,
        PatternSlot::Stroke,
        PatternSource::Shape(77),
        [77.0, 77.0],
    ));
    assert_eq!(
        pattern_at(&scene, id, PatternSlot::Stroke).map(|p| p.source),
        Some(PatternSource::Shape(77))
    );
    assert_eq!(
        pattern_at(&scene, id, PatternSlot::Fill).map(|p| p.source),
        Some(PatternSource::Shape(1)),
        "trocar a arte do traco trocou tambem a do preenchimento"
    );
}
