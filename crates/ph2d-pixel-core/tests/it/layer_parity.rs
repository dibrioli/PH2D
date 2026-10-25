//! **P3 contra o MVP web** — a fixtura `mvp_layers.json` (`docs/PixelArt/ferramentas/extrai_camadas_mvp.mjs`).
//!
//! O MVP é só de índices e tem duas leis que o Aseprite NÃO tem (recusadas, handoff P3): a
//! opacidade como cobertura Bayer 8×8 e o `clip` (máscara da camada de baixo). A paridade vale onde
//! as leis coincidem — camadas opacas sem clip: quem vence cada célula é a de cima visível que não
//! é transparente — e na estrutura das ops (ordem, visível, opacidade, trancada) a cada passo, a
//! cada desfazer e a cada refazer.

use ph2d_pixel_core::layers::{self, Place};
use ph2d_pixel_core::{
    Background, ColorMode, History, LayerId, NewSprite, Op, Patch, Rect, Sample, Sprite, apply, pick,
};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/mvp_layers.json")).expect("json")
}

fn n(v: &Value) -> u32 {
    v.as_u64().expect("int") as u32
}

/// Um sprite Indexed `w × h` com as camadas da fixtura (de baixo para cima) e as células dadas.
fn sprite_of(c: &Value) -> Sprite {
    let (w, h) = (n(&c["w"]), n(&c["h"]));
    let mut s = Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Indexed,
        background: Background::Transparent,
    })
    .expect("sprite");
    let t = n(&c["transparent"]) as u8;
    let colors = s.palette().to_vec();
    apply(&mut s, Op::Palette { colors, transparent: t }).expect("paleta");
    let fl = c["layers"].as_array().expect("layers");
    let mut ids = vec![s.layers()[0].id];
    for _ in 1..fl.len() {
        let top = *ids.last().expect("topo");
        let (op, id) = layers::new_layer(&mut s, top).expect("nova");
        apply(&mut s, op).expect("aplica");
        ids.push(id);
    }
    for (l, id) in fl.iter().zip(&ids) {
        let vis = l["visible"].as_bool().expect("vis");
        let op = layers::edit(&s, *id, |x| x.visible = vis);
        if let Some(op) = op {
            apply(&mut s, op).expect("vis");
        }
        if let Some(cells) = l["cells"].as_array() {
            let pixels: Vec<u8> = cells
                .iter()
                .map(|v| n(v) as u8)
                .collect();
            let r = Rect::new(0, 0, w, h);
            let p = Patch { layer: *id, frame: s.frames()[0].id, bbox: Some(r), rect: r, pixels };
            apply(&mut s, Op::Patch(p)).expect("patch");
        }
    }
    s
}

#[test]
fn the_top_visible_opaque_cell_wins_as_in_the_mvp() {
    let f = fx();
    let mut comparados = 0;
    for c in f["compose"].as_array().expect("compose") {
        let fl = c["layers"].as_array().expect("layers");
        // só onde as leis coincidem: nenhuma visível com opacidade < 255 ou clip
        let coincide = fl.iter().all(|l| {
            !l["visible"].as_bool().unwrap_or(false)
                || (n(&l["opacity"]) == 255 && !l["clip"].as_bool().unwrap_or(false))
        });
        if !coincide {
            continue;
        }
        let s = sprite_of(c);
        let w = n(&c["w"]);
        let frame = s.frames()[0].id;
        for (i, v) in c["out"].as_array().expect("out").iter().enumerate() {
            let (x, y) = ((i as u32 % w) as i32, (i as u32 / w) as i32);
            let got = pick(&s, frame, x, y, Sample::AllLayers).expect("dentro");
            // transparente = o índice transparente, nos dois lados
            let want = n(v);
            assert_eq!(got, want, "célula ({x},{y}) de {c}");
        }
        comparados += 1;
    }
    assert!(comparados >= 20, "a fixtura tem de dar pilhas comparáveis: {comparados}");
}

/// A estrutura comparável: `(id do MVP, visível, opacidade, trancada)` por camada, de baixo para cima.
type Linhas = Vec<(u32, bool, u8, bool)>;

fn mvp_rows(v: &Value) -> Linhas {
    v.as_array()
        .expect("snap")
        .iter()
        .map(|l| {
            (
                n(&l["id"]),
                l["visible"].as_bool().expect("v"),
                n(&l["opacity"]) as u8,
                l["locked"].as_bool().expect("l"),
            )
        })
        .collect()
}

fn our_rows(s: &Sprite, map: &[(u32, LayerId)]) -> Linhas {
    s.layers()
        .iter()
        .map(|l| {
            let m = map.iter().find(|(_, o)| *o == l.id).expect("mapeada").0;
            (m, l.visible, l.opacity, l.locked)
        })
        .collect()
}

#[test]
fn layer_ops_undo_and_redo_match_the_mvp_structure() {
    let f = fx();
    let seqs = f["ops"].as_array().expect("ops");
    assert!(seqs.len() >= 40);
    for seq in seqs {
        let mut s = Sprite::new(NewSprite {
            width: 2,
            height: 2,
            ..NewSprite::default()
        })
        .expect("sprite");
        let mut h = History::default();
        let mut map = vec![(1u32, s.layers()[0].id)];
        let ours = |mvp: u32, map: &[(u32, LayerId)]| map.iter().find(|(m, _)| *m == mvp).expect("id").1;
        for step in seq["steps"].as_array().expect("steps") {
            let op = match step["op"].as_str().expect("op") {
                "insert" => {
                    let i = n(&step["index"]) as usize;
                    let place = if i == 0 {
                        Place::Below(s.layers()[0].id)
                    } else {
                        Place::Above(s.layers()[i - 1].id)
                    };
                    let id = LayerId(s.reserve_id());
                    let new = ph2d_pixel_core::Layer::image(id, "x");
                    map.push((n(&step["id"]), id));
                    let mut ls = s.layers().to_vec();
                    let at = match place {
                        Place::Below(_) => 0,
                        Place::Above(a) => ls.iter().position(|l| l.id == a).expect("a") + 1,
                        Place::Into(_) => unreachable!(),
                    };
                    ls.insert(at, new);
                    Op::Layers(Box::new(ph2d_pixel_core::LayerSet { layers: ls, cels: Vec::new() }))
                }
                "delete" => layers::delete(&s, ours(n(&step["id"]), &map)).expect("apaga"),
                "move" => {
                    let (from, to) = (n(&step["from"]) as usize, n(&step["to"]) as usize);
                    let mut ls = s.layers().to_vec();
                    let l = ls.remove(from);
                    ls.insert(to, l);
                    Op::Layers(Box::new(ph2d_pixel_core::LayerSet { layers: ls, cels: Vec::new() }))
                }
                _ => {
                    let id = ours(n(&step["id"]), &map);
                    let p = &step["props"];
                    let mut ls = s.layers().to_vec();
                    let l = ls.iter_mut().find(|l| l.id == id).expect("l");
                    if let Some(v) = p["visible"].as_bool() {
                        l.visible = v;
                    }
                    if let Some(o) = p["opacity"].as_u64() {
                        l.opacity = o as u8;
                    }
                    if let Some(k) = p["locked"].as_bool() {
                        l.locked = k;
                    }
                    if let Some(nm) = p["name"].as_str() {
                        nm.clone_into(&mut l.name);
                    }
                    Op::Layers(Box::new(ph2d_pixel_core::LayerSet { layers: ls, cels: Vec::new() }))
                }
            };
            h.commit(&mut s, op).expect("commit");
            assert_eq!(our_rows(&s, &map), mvp_rows(&step["after"]), "passo {step}");
        }
        for u in seq["undos"].as_array().expect("undos") {
            assert!(h.undo(&mut s));
            assert_eq!(our_rows(&s, &map), mvp_rows(u), "desfazer");
        }
        for r in seq["redos"].as_array().expect("redos") {
            assert!(h.redo(&mut s));
            assert_eq!(our_rows(&s, &map), mvp_rows(r), "refazer");
        }
    }
}

#[test]
fn deleting_the_last_image_layer_is_refused_as_in_the_mvp() {
    let f = fx();
    assert!(f["limits"]["delete_last"].as_str().is_some(), "o MVP recusa");
    let s = Sprite::new(NewSprite::default()).expect("sprite");
    assert!(layers::delete(&s, s.layers()[0].id).is_none());
}
