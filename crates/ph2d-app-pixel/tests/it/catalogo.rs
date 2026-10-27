//! **P6e pelo caminho real** (dono 27/10: *«traga todas as outras shapes presentes em vector para
//! pixel mantendo os mesmos gizmos maravilhosos»*): TODA forma do catálogo do Vector nasce viva com as
//! alças dele e assenta num passo; a aberta (linha, arco, espiral…) assenta como traço de uma célula,
//! sem interior, mesmo com o «Fill» ligado.

use std::collections::BTreeSet;

use ph2d_app_pixel::{Tool, formas, viva};
use ph2d_editor_core::pixel_panel::PixelShape;
use ph2d_tool_vector::shape_handles::handles_of;
use ph2d_tool_vector::shapes::{ALL_GROUPS, SHAPES};
use ph2d_vec_scene::ShapeKind;
use winit::keyboard::KeyCode;

use crate::edit::canvas;
use crate::viva::Mesa;

const CLEAR: [u8; 4] = [0, 0, 0, 0];
const A: (i32, i32) = (3, 3);
const B: (i32, i32) = (28, 24);

/// As células pintadas na caixa da forma (com uma de folga).
fn pintadas(m: &Mesa) -> BTreeSet<(i32, i32)> {
    let mut out = BTreeSet::new();
    for y in A.1 - 1..=B.1 + 1 {
        for x in A.0 - 1..=B.0 + 1 {
            if m.v((x, y)) != CLEAR {
                out.insert((x, y));
            }
        }
    }
    out
}

#[test]
fn every_vector_shape_is_born_live_with_the_vector_handles_and_settles_as_one_step() {
    assert_eq!(PixelShape::ALL.len(), SHAPES.len(), "o catálogo inteiro");
    let mut com_alcas = 0;
    for s in PixelShape::ALL {
        let mut m = Mesa::nova();
        m.app.options.shape_fill = true;
        m.desenha(Tool::Shape(s), A, B);
        let v = m.app.live_shape().unwrap_or_else(|| panic!("{s:?}: viva"));
        assert_eq!(v.forma(), s);
        let c = canvas(&m.sim, &m.cam, m.bits);
        let alcas = viva::handles_on_screen(&m.app, &c, m.key);
        assert_eq!(
            alcas.len(),
            handles_of(s.kind()).len(),
            "{s:?}: as alças do Vector"
        );
        if !alcas.is_empty() {
            assert!(
                alcas.iter().any(Option::is_some),
                "{s:?}: nenhuma alça à vista"
            );
            com_alcas += 1;
        }
        assert!(pintadas(&m).is_empty(), "{s:?}: largar não grava");
        m.tecla(KeyCode::Enter);
        assert!(m.app.live_shape().is_none(), "{s:?}: o Enter assenta");
        assert!(!pintadas(&m).is_empty(), "{s:?}: assentou em células");
        assert!(m.app.undo(m.key), "{s:?}");
        assert!(pintadas(&m).is_empty(), "{s:?}: o desfazer leva-a toda");
        assert!(!m.app.undo(m.key), "{s:?}: um passo só");
    }
    assert!(com_alcas >= 40, "controlo: só {com_alcas} formas com alças");
}

#[test]
fn an_open_shape_settles_as_a_one_cell_stroke_even_with_fill_on() {
    let abertas: Vec<_> = PixelShape::ALL
        .into_iter()
        .filter(|s| !s.kind().is_closed())
        .collect();
    assert_eq!(abertas.len(), 5, "as abertas do Vector");
    for s in abertas
        .into_iter()
        .chain([PixelShape::new(ShapeKind::Heart)])
    {
        let mut m = Mesa::nova();
        m.app.options.shape_fill = true;
        m.desenha(Tool::Shape(s), A, B);
        m.tecla(KeyCode::Enter);
        let v = m.app.options.shapes.get(s);
        let (traco, dentro) = formas::cells(s, &v, A, B, true);
        let traco: BTreeSet<_> = traco.into_iter().collect();
        if s.kind().is_closed() {
            // O controlo: a fechada enche.
            assert!(!dentro.is_empty(), "{s:?}: o Fill enche a fechada");
            continue;
        }
        assert!(dentro.is_empty(), "{s:?}: a aberta não tem interior");
        assert_eq!(pintadas(&m), traco, "{s:?}: só o traço, de uma célula");
        // Aberta: o traço não volta ao começo (o polígono fecharia a corda).
        let path = formas::cook_local(s, [12.0, 10.0], &v.values());
        let fechados = ph2d_vec_scene::trim_tool::contours_of(&path)
            .filter(|c| c.closed)
            .count();
        assert_eq!(fechados, 0, "{s:?}: o Vector cozinha-a aberta");
    }
}

#[test]
fn scene_13_samples_every_catalog_family_and_leaves_a_live_gear_with_its_handles() {
    use ph2d_app_pixel::smoke_catalogo::{GEAR, SETTLED, caixa};
    let grupo = |k: ShapeKind| SHAPES.iter().find(|d| d.kind == k).map(|d| d.group);
    for g in ALL_GROUPS {
        assert!(
            SETTLED.iter().any(|(k, ..)| grupo(*k) == Some(*g)),
            "a família {g:?} não tem amostra"
        );
    }
    assert!(
        SETTLED.iter().any(|(k, ..)| !k.is_closed()),
        "uma aberta, em traço"
    );
    let mut m = Mesa::cena(13);
    let gear = PixelShape::new(ShapeKind::Gear);
    assert_eq!(m.app.tool, Tool::Shape(gear));
    assert_eq!(m.app.live_shape().map(viva::Viva::forma), Some(gear));
    let c = canvas(&m.sim, &m.cam, m.bits);
    let alcas = viva::handles_on_screen(&m.app, &c, m.key);
    assert!(
        alcas.iter().flatten().count() >= 1,
        "as alças da engrenagem: {alcas:?}"
    );
    for (i, (k, cor, _)) in SETTLED.into_iter().enumerate() {
        let (a, b) = caixa(i);
        let achou = (a.1..=b.1).any(|y| (a.0..=b.0).any(|x| m.v((x, y)) == cor));
        assert!(achou, "{k:?} assentou na caixa {i}");
    }
    // A engrenagem tem o furo do eixo no meio (a do Vector): contam-se as células dela na caixa.
    let ((a, b), cor) = ((GEAR.0, GEAR.1), GEAR.2);
    let dela = |m: &Mesa| {
        (a.1..=b.1)
            .flat_map(|y| (a.0..=b.0).map(move |x| (x, y)))
            .filter(|&c| m.v(c) == cor)
            .count()
    };
    assert_eq!(dela(&m), 0, "a engrenagem ainda não é pixel");
    m.tecla(KeyCode::Enter);
    let area = usize::try_from((b.0 - a.0 + 1) * (b.1 - a.1 + 1)).expect("caixa");
    assert!(
        dela(&m) > area / 3,
        "assentou cheia: {} de {area}",
        dela(&m)
    );
}
