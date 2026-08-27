//! Os gates da autoria do padrão (plano 33, W5).

use super::*;
use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecVertex};

fn scene_with(f: PatternFill) -> (VecScene, ph2d_vec_edit::PenTool, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(f))),
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, id)
}

fn fill() -> PatternFill {
    // ⚠️ Arte NÃO quadrada de propósito: um `size` `[8, 2]` é o único que mostra se o aspecto
    // sobreviveu a um Size novo. Com `[4, 4]` toda aritmética errada passa.
    let mut f = PatternFill::new(
        PatternSource::Shape(1),
        [8.0, 2.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f.kind = TileKind::Grid;
    f
}

fn pattern_of(scene: &VecScene, id: VecPathId) -> PatternFill {
    match scene.path(id).and_then(|p| p.fill.as_ref()) {
        Some(Paint::Pattern(p)) => (**p).clone(),
        _ => panic!("a forma deixou de ter padrao"),
    }
}

/// ⭐ **Mudar o Size preserva o ASPECTO da arte.** O painel autora UM número e o documento guarda
/// DOIS — oferecer os dois lados deixaria o artista esmagar a imagem sem querer.
#[test]
fn setting_the_size_keeps_the_arts_aspect() {
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(&mut scene, &mut h, &pen, TexPatCmd::Size(4.0));
    let p = pattern_of(&scene, id);
    assert!(
        (longer_side(p.size) - 4.0).abs() < 1e-9,
        "o lado maior nao virou 4: {:?}",
        p.size
    );
    assert!(
        (p.size[0] / p.size[1] - 4.0).abs() < 1e-9,
        "o aspecto 4:1 nao sobreviveu: {:?}",
        p.size
    );
}

/// **Cada mudança é UM passo de undo, e um valor repetido NÃO é passo nenhum.**
///
/// ⚠️ Sem a comparação, o slider a re-publicar o mesmo número faria todo quadro virar um passo — o
/// defeito que o `canonicalize` do editor curou para o mundo inteiro.
#[test]
fn a_repeated_value_records_no_undo_step() {
    let (mut scene, pen, _) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(&mut scene, &mut h, &pen, TexPatCmd::Angle(30.0));
    let after_first = h.undo_len();
    apply(&mut scene, &mut h, &pen, TexPatCmd::Angle(30.0));
    assert_eq!(
        h.undo_len(),
        after_first,
        "o mesmo valor gravou um passo espurio"
    );
    apply(&mut scene, &mut h, &pen, TexPatCmd::Angle(31.0));
    assert_eq!(h.undo_len(), after_first + 1, "um valor NOVO tem de gravar");
}

/// **Os índices do painel e os enums do documento são a MESMA lista, nos dois sentidos.**
///
/// ⚠️ Uma tradução escrita só num sentido é onde um chip passa a acender no reticulado errado — e
/// isso lê-se como *"o painel mostra Brick e o desenho é Hex"*, que é um report sem causa aparente.
#[test]
fn the_panel_indices_round_trip_through_the_document_enums() {
    for k in [
        TileKind::Grid,
        TileKind::BrickRow,
        TileKind::BrickCol,
        TileKind::Hex,
    ] {
        let (mut scene, pen, id) = scene_with(fill());
        let mut h = ph2d_vec_edit::History::default();
        apply(&mut scene, &mut h, &pen, TexPatCmd::Tile(tile_index(k)));
        assert_eq!(
            pattern_of(&scene, id).kind,
            k,
            "ida e volta partiu em {k:?}"
        );
    }
    for m in [PatternMode::Tile, PatternMode::Mirror, PatternMode::Clamp] {
        let (mut scene, pen, id) = scene_with(fill());
        let mut h = ph2d_vec_edit::History::default();
        apply(&mut scene, &mut h, &pen, TexPatCmd::Mode(mode_index(m)));
        assert_eq!(
            pattern_of(&scene, id).mode,
            m,
            "ida e volta partiu em {m:?}"
        );
    }
}

/// ⚠️ **O denominador é INTEIRO e nunca desce abaixo de 1.** Um `1/0` seria uma divisão por zero no
/// assador, e um `1/2,7` um desfasamento que nenhum reticulado exprime.
#[test]
fn the_offset_denominator_is_a_whole_number_and_never_zero() {
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(&mut scene, &mut h, &pen, TexPatCmd::OffsetDenom(2.7));
    assert_eq!(pattern_of(&scene, id).offset_denom, 3);
    apply(&mut scene, &mut h, &pen, TexPatCmd::OffsetDenom(-5.0));
    assert_eq!(pattern_of(&scene, id).offset_denom, 1);
}

/// **O ângulo do painel é em GRAUS e o documento guarda RADIANOS** — a conversão vive numa porta só.
#[test]
fn the_angle_crosses_from_degrees_to_radians_in_one_door() {
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    apply(&mut scene, &mut h, &pen, TexPatCmd::Angle(90.0));
    assert!(
        (pattern_of(&scene, id).angle - std::f64::consts::FRAC_PI_2).abs() < 1e-12,
        "90 graus nao viraram pi/2"
    );
}

/// ⚠️ **Uma forma SEM padrão não é tocada** — a secção nem sobe para ela, mas o comando pode chegar
/// por um caminho que o painel não controla (um atalho, um replay), e um `Paint::Solid` que virasse
/// padrão por causa de um slider seria uma edição que o artista não pediu.
#[test]
fn a_shape_without_a_pattern_is_left_alone() {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(9, 9, 9, 255))),
        ..VecPath::default()
    });
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[id]);
    let mut h = ph2d_vec_edit::History::default();
    apply(&mut scene, &mut h, &pen, TexPatCmd::Size(4.0));
    assert_eq!(h.undo_len(), 0, "gravou um passo sobre uma forma solida");
    assert!(matches!(
        scene.path(id).and_then(|p| p.fill.as_ref()),
        Some(Paint::Solid(_))
    ));
}

/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"clamp deixa tudo em branco"*.**
///
/// O `Clamp` desenha UMA cópia e estica a borda pelo resto. Com a cópia do tamanho de um ladrilho,
/// no canto, quase toda a forma é borda esticada — um borrão chapado, nunca a imagem. Escolher
/// `Clamp` passa a **ENQUADRAR** a cópia na forma, cobrindo-a.
#[test]
fn switching_to_clamp_frames_the_single_copy_over_the_shape() {
    let (mut scene, pen, id) = scene_with(fill());
    let mut h = ph2d_vec_edit::History::default();
    // A forma mede 10x10; o padrão nasce a 8x2 na origem.
    apply(&mut scene, &mut h, &pen, TexPatCmd::Mode(2));
    let p = pattern_of(&scene, id);
    assert_eq!(p.mode, PatternMode::Clamp);
    assert_eq!(
        p.origin,
        [0.0, 0.0],
        "a copia tem de nascer no canto da forma"
    );
    assert!(
        p.size[0] >= 10.0 - 1e-9 && p.size[1] >= 10.0 - 1e-9,
        "a copia nao COBRE a forma de 10x10: {:?}",
        p.size
    );
    assert!(
        (p.size[0] / p.size[1] - 4.0).abs() < 1e-9,
        "o aspecto 4:1 da arte nao sobreviveu ao enquadramento: {:?}",
        p.size
    );
    // ⚠️ Controlo: os OUTROS modos não enquadram — enquadrar sempre destruiria o ladrilho que o
    // artista afinou no instante em que ele espreitasse o Mirror.
    let (mut scene2, pen2, id2) = scene_with(fill());
    let mut h2 = ph2d_vec_edit::History::default();
    apply(&mut scene2, &mut h2, &pen2, TexPatCmd::Mode(1));
    assert_eq!(
        pattern_of(&scene2, id2).size,
        [8.0, 2.0],
        "o Mirror enquadrou"
    );
}
