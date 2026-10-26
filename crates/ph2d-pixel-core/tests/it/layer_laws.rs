//! **As leis das camadas** (P3): cada acção do menu Layer é UM passo com inverso exacto; Merge Down e
//! Flatten não mudam a imagem; o grupo é pass-through (medido no Trial); a referência vê-se e não se
//! exporta; o formato guarda tudo; a árvore recusa o que a partiria.

use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::layers::{self, Place};
use ph2d_pixel_core::{
    Background, ColorMode, FrameId, History, Layer, LayerId, LayerKind, LayerSet, NewSprite, Op,
    OpError, Patch, Purpose, Rect, Sprite, apply, compose_rgba, compose_window,
};
use std::collections::BTreeMap;

fn bytes(s: &Sprite) -> Vec<u8> {
    write_library(&BTreeMap::from([(1, s.clone())]))
}

fn frame(s: &Sprite) -> FrameId {
    s.frames()[0].id
}

/// Pinta `rgba` no retângulo `r` da camada.
fn fill(s: &mut Sprite, l: LayerId, r: Rect, rgba: [u8; 4]) {
    let v = s.value_of_rgba(rgba).to_le_bytes();
    let bpp = s.mode().bpp();
    let pixels = (0..r.area()).flat_map(|_| v[..bpp].to_vec()).collect();
    let bbox = s.cel((l, frame(s))).map_or(r, |c| c.bbox.union(r));
    let p = Patch {
        layer: l,
        frame: frame(s),
        bbox: Some(bbox),
        rect: r,
        pixels,
    };
    apply(s, Op::Patch(p)).expect("patch");
}

/// 8 × 6 RGBA: Background branca, «a» vermelha meio transparente, um grupo com «b» azul em
/// Multiply e «c» verde a 50 %; a activa é «a».
fn cena() -> (Sprite, [LayerId; 5]) {
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 6,
        mode: ColorMode::Rgba,
        background: Background::White,
    })
    .expect("sprite");
    let bg = s.layers()[0].id;
    let (op, a) = layers::new_layer(&mut s, bg).expect("a");
    apply(&mut s, op).expect("a");
    fill(&mut s, a, Rect::new(0, 0, 5, 4), [200, 30, 40, 160]);
    let (op, g) = layers::new_group(&mut s, a).expect("g");
    apply(&mut s, op).expect("g");
    let (op, b) = layers::new_layer(&mut s, g).expect("b");
    apply(&mut s, op).expect("b");
    let op = layers::move_to(&s, b, Place::Into(g)).expect("b no grupo");
    apply(&mut s, op).expect("b");
    fill(&mut s, b, Rect::new(2, 1, 6, 5), [20, 60, 220, 255]);
    let op = layers::edit(&s, b, |l| l.blend = BlendMode::Multiply).expect("modo");
    apply(&mut s, op).expect("modo");
    let (op, c) = layers::new_layer(&mut s, b).expect("c");
    apply(&mut s, op).expect("c");
    fill(&mut s, c, Rect::new(4, 0, 4, 3), [30, 200, 60, 255]);
    let op = layers::edit(&s, c, |l| l.opacity = 128).expect("op");
    apply(&mut s, op).expect("op");
    (s, [bg, a, g, b, c])
}

fn image(s: &Sprite) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, frame(s), None, &mut out);
    out
}

#[test]
fn every_layer_action_is_one_step_whose_undo_restores_the_bytes() {
    let (s0, [bg, a, g, b, c]) = cena();
    type Acao = Box<dyn Fn(&mut Sprite) -> Option<Op>>;
    let acoes: Vec<(&str, Acao)> = vec![
        (
            "new layer",
            Box::new(move |s| layers::new_layer(s, a).map(|r| r.0)),
        ),
        (
            "new group",
            Box::new(move |s| layers::new_group(s, c).map(|r| r.0)),
        ),
        ("delete group", Box::new(move |s| layers::delete(s, g))),
        (
            "rename",
            Box::new(move |s| layers::edit(s, a, |l| "x".clone_into(&mut l.name))),
        ),
        (
            "move",
            Box::new(move |s| layers::move_to(s, a, Place::Above(g))),
        ),
        (
            "move out",
            Box::new(move |s| layers::move_to(s, c, Place::Above(g))),
        ),
        (
            "duplicate group",
            Box::new(move |s| layers::duplicate(s, g).map(|r| r.0)),
        ),
        ("merge down", Box::new(move |s| layers::merge_down(s, a))),
        (
            "merge in group",
            Box::new(move |s| layers::merge_down(s, c)),
        ),
        ("flatten", Box::new(|s| layers::flatten(s, false))),
        ("flatten visible", Box::new(|s| layers::flatten(s, true))),
        (
            "via copy",
            Box::new(move |s| {
                let f = frame(s);
                layers::via(s, a, f, Rect::new(1, 1, 3, 2), false).map(|r| r.0)
            }),
        ),
        (
            "via cut",
            Box::new(move |s| {
                let f = frame(s);
                layers::via(s, b, f, Rect::new(0, 0, 4, 4), true).map(|r| r.0)
            }),
        ),
        (
            "paste",
            Box::new(move |s| {
                let f = frame(s);
                let clip = layers::copy(s, (a, f), s.bounds())?;
                layers::paste_as_new(s, c, f, &clip, true).map(|r| r.0)
            }),
        ),
    ];
    let _ = bg;
    for (nome, acao) in acoes {
        let mut s = s0.clone();
        let mut h = History::default();
        let op = acao(&mut s).unwrap_or_else(|| panic!("{nome}: a acção aplica-se"));
        let antes = bytes(&s);
        h.commit(&mut s, op)
            .unwrap_or_else(|e| panic!("{nome}: {e:?}"));
        assert_ne!(bytes(&s), antes, "{nome}: mudou algo");
        let depois = bytes(&s);
        assert!(h.undo(&mut s), "{nome}");
        assert_eq!(bytes(&s), antes, "{nome}: o desfazer devolve os bytes");
        assert!(h.redo(&mut s), "{nome}");
        assert_eq!(bytes(&s), depois, "{nome}: o refazer também");
    }
}

#[test]
fn merge_down_and_flatten_keep_the_image() {
    let (s0, [_, a, _, _, c]) = cena();
    let want = image(&s0);
    let mut s = s0.clone();
    apply(&mut s, layers::merge_down(&s0, a).expect("merge")).expect("merge");
    assert_eq!(image(&s), want, "Merge Down sobre a Background");
    // Sobre uma camada com modo próprio a fusão MUDA a imagem (a de cima passa a misturar-se com o
    // modo da de baixo) — é a semântica do Merge Down; o passo continua um só e reversível.
    let mut s = s0.clone();
    apply(&mut s, layers::merge_down(&s0, c).expect("merge")).expect("merge");
    assert!(s.layer(c).is_none(), "a de cima sai");
    let mut s = s0.clone();
    let op = layers::flatten(&mut s, false).expect("flatten");
    apply(&mut s, op).expect("flatten");
    assert_eq!(s.layers().len(), 1, "uma só");
    assert!(s.layers()[0].background, "com Background, fica Background");
    assert_eq!(image(&s), want, "a imagem não muda");
}

#[test]
fn a_group_passes_its_layers_through_and_its_mode_is_not_used() {
    // Trial, g*/gm*.ase: o mesmo composto com as camadas na raiz, e o modo do grupo não entra.
    let (s0, [_, _, g, b, c]) = cena();
    let want = image(&s0);
    let mut raiz = s0.clone();
    for id in [b, c] {
        let op = layers::move_to(&raiz, id, Place::Above(g)).expect("sobe");
        apply(&mut raiz, op).expect("sobe");
    }
    let op = layers::move_to(&raiz, c, Place::Above(b)).expect("ordem");
    apply(&mut raiz, op).expect("ordem");
    assert_eq!(image(&raiz), want, "o grupo é pass-through");
    let mut modo = s0.clone();
    let op = layers::edit(&modo, g, |l| l.blend = BlendMode::Difference).expect("modo do grupo");
    apply(&mut modo, op).expect("modo");
    assert_eq!(image(&modo), want, "o modo do grupo não entra");
    let mut escondido = s0.clone();
    let op = layers::edit(&escondido, g, |l| l.visible = false).expect("olho");
    apply(&mut escondido, op).expect("olho");
    assert!(
        !escondido.is_shown(c),
        "o grupo escondido esconde os filhos"
    );
    assert_ne!(image(&escondido), want);
}

#[test]
fn a_reference_layer_shows_but_is_not_exported_nor_flattened() {
    let (mut s, [_, a, _, _, _]) = cena();
    let f = frame(&s);
    let clip = layers::copy(&s, (a, f), s.bounds()).expect("copia");
    let top = s.layers().last().expect("topo").id;
    let (op, r) = layers::paste_as_new(&mut s, top, f, &clip, true).expect("referência");
    apply(&mut s, op).expect("ref");
    let ver = compose_window(&s, f, None, s.bounds(), Purpose::View);
    let exp = compose_window(&s, f, None, s.bounds(), Purpose::Export);
    assert_ne!(ver, exp, "a vista mostra a referência; a exportação não");
    let op = layers::flatten(&mut s, false).expect("flatten");
    apply(&mut s, op).expect("flatten");
    assert!(
        s.layer(r).is_some_and(|l| l.reference),
        "a referência fica de fora"
    );
}

#[test]
fn every_layer_field_survives_the_format() {
    let (mut s, [_, a, g, _, _]) = cena();
    let op = layers::edit(&s, a, |l| {
        l.locked = true;
        l.reference = true;
        l.opacity = 7;
        l.blend = BlendMode::Divide;
    })
    .expect("campos");
    apply(&mut s, op).expect("campos");
    let op = layers::edit(&s, g, |l| l.kind = LayerKind::Group { collapsed: true }).expect("g");
    apply(&mut s, op).expect("g");
    let back = read_library(&bytes(&s)).expect("lê");
    assert_eq!(back[&1], s, "o mesmo documento");
    assert_eq!(bytes(&back[&1]), bytes(&s), "os mesmos bytes");
}

#[test]
fn the_tree_refuses_what_would_break_it() {
    let (s, [bg, a, g, b, _]) = cena();
    let tenta = |f: &dyn Fn(&mut Vec<Layer>)| {
        let mut ls = s.layers().to_vec();
        f(&mut ls);
        let mut t = s.clone();
        apply(
            &mut t,
            Op::Layers(Box::new(LayerSet {
                layers: ls,
                cels: Vec::new(),
            })),
        )
    };
    let pai = |ls: &mut Vec<Layer>, id: LayerId, p: Option<LayerId>| {
        ls.iter_mut().find(|l| l.id == id).expect("l").parent = p;
    };
    assert!(
        matches!(tenta(&|ls| pai(ls, g, Some(g))), Err(OpError::BadLayers(_))),
        "ciclo"
    );
    assert!(
        matches!(tenta(&|ls| pai(ls, b, Some(a))), Err(OpError::BadLayers(_))),
        "pai sem ser grupo"
    );
    assert!(
        matches!(tenta(&|ls| ls.swap(0, 1)), Err(OpError::BadLayers(_))),
        "Background no fundo"
    );
    assert!(
        matches!(
            tenta(&|ls| ls.retain(|l| l.id != a)),
            Err(OpError::BadCel(_))
        ),
        "tirar uma camada sem os cels dela deixaria cels órfãos"
    );
    let mut t = s.clone();
    let cel = s.cel((a, frame(&s))).cloned();
    let r = apply(
        &mut t,
        Op::Layers(Box::new(LayerSet {
            layers: s.layers().to_vec(),
            cels: vec![((g, frame(&s)), cel)],
        })),
    );
    assert!(
        matches!(r, Err(OpError::BadCel(_))),
        "um grupo não tem cels"
    );
    assert!(
        layers::move_to(&s, bg, Place::Above(a)).is_none(),
        "a Background não sai do fundo"
    );
    assert!(
        layers::move_to(&s, g, Place::Into(g)).is_none(),
        "um grupo não entra em si"
    );
}

#[test]
fn paste_between_modes_goes_through_the_color() {
    let (s, [_, a, ..]) = cena();
    let f = frame(&s);
    let clip = layers::copy(&s, (a, f), s.bounds()).expect("copia");
    let mut idx = Sprite::new(NewSprite {
        width: 8,
        height: 6,
        mode: ColorMode::Indexed,
        background: Background::Transparent,
    })
    .expect("idx");
    let top = idx.layers()[0].id;
    let fi = frame(&idx);
    let (op, id) = layers::paste_as_new(&mut idx, top, fi, &clip, false).expect("cola");
    apply(&mut idx, op).expect("cola");
    let v = idx.value_at((id, fi), 0, 0);
    assert_eq!(
        v,
        idx.value_of_rgba([200, 30, 40, 160]),
        "a cor mais próxima da paleta"
    );
}
