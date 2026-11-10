//! P13: efeitos de camada, rampas e ciclos no `.ase` (desenho: `docs/PixelArt/P13_plano.md` §12
//! «C + R4»). Na escrita o cel dono sai ASSADO (outros apps vêem o contorno) com os bytes crus em
//! `ph2d.cru`; a camada leva `ph2d.efeitos`; o sprite `ph2d.ramps` e `ph2d.cycles` (o
//! `ph2d-aseprite` não modela o user data do chunk de paleta). Na leitura o cru volta, o efeito
//! renasce e as propriedades saem: ida-e-volta exacta. Tudo no mapa do artista (chave `""`).

use std::borrow::Cow;

use ph2d_pixel_core::filter::outline::{Background, Outline, Place};
use ph2d_pixel_core::{
    Cel, CelKey, Cycle, CycleDirection, DocOp, EfeitoDeCamada, LayerId, Op, Owner, Patch, PropMap,
    PropValue, Rect, Sprite, aplicar_efeitos, apply,
};

pub const EFEITOS_PROP: &str = "ph2d.efeitos";
pub const CRU_PROP: &str = "ph2d.cru";
pub const RAMPS_PROP: &str = "ph2d.ramps";
pub const CYCLES_PROP: &str = "ph2d.cycles";

fn poe(s: &mut Sprite, owner: Owner, chave: &str, v: PropValue) {
    let Some(u) = s.user_of(owner) else {
        return;
    };
    let mut u = u.clone();
    u.props
        .entry(String::new())
        .or_default()
        .insert(chave.to_owned(), v);
    let data = Box::new(u);
    let _ = apply(s, Op::UserData { owner, data });
}

/// Tira a propriedade do mapa do artista, só se `le` a entende (senão fica, como veio).
fn tira<T>(
    s: &mut Sprite,
    owner: Owner,
    chave: &str,
    le: impl Fn(&PropValue) -> Option<T>,
) -> Option<T> {
    let mut u = s.user_of(owner)?.clone();
    let m = u.props.get_mut("")?;
    let v = le(m.get(chave)?)?;
    m.remove(chave);
    if m.is_empty() {
        u.props.remove("");
    }
    let data = Box::new(u);
    apply(s, Op::UserData { owner, data }).ok()?;
    Some(v)
}

fn bytes(b: &[u8]) -> PropValue {
    PropValue::Vector(b.iter().map(|&x| PropValue::U8(x)).collect())
}

fn bytes_de(v: &PropValue) -> Option<Vec<u8>> {
    let PropValue::Vector(v) = v else {
        return None;
    };
    v.iter()
        .map(|x| match x {
            PropValue::U8(b) => Some(*b),
            _ => None,
        })
        .collect()
}

fn cor_de(v: Option<&PropValue>) -> Option<[u8; 4]> {
    bytes_de(v?)?.try_into().ok()
}

fn mapa(campos: impl IntoIterator<Item = (&'static str, PropValue)>) -> PropValue {
    PropValue::Map(campos.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}

fn efeito_prop(e: &EfeitoDeCamada) -> PropValue {
    let EfeitoDeCamada::Outline(o) = e;
    let bits = o.matrix.iter().flatten().enumerate();
    let matriz = bits.fold(0_u16, |a, (i, &b)| a | (u16::from(b) << i));
    let mut m = vec![
        ("kind", PropValue::Str("outline".into())),
        ("color", bytes(&o.color)),
        ("matrix", PropValue::U16(matriz)),
        ("inside", PropValue::Bool(o.place == Place::Inside)),
        ("tiled", PropValue::Bool(o.tiled)),
    ];
    if let Background::Color(c) = o.background {
        m.push(("background", bytes(&c)));
    }
    mapa(m)
}

fn efeito_de(v: &PropValue) -> Option<EfeitoDeCamada> {
    let PropValue::Map(m) = v else {
        return None;
    };
    let flag = |k: &str| match m.get(k) {
        Some(PropValue::Bool(b)) => Some(*b),
        _ => None,
    };
    if m.get("kind") != Some(&PropValue::Str("outline".into())) {
        return None;
    }
    let Some(PropValue::U16(bits)) = m.get("matrix") else {
        return None;
    };
    let mut matrix = [[false; 3]; 3];
    for (i, c) in matrix.iter_mut().flatten().enumerate() {
        *c = bits & (1 << i) != 0;
    }
    let background = match m.get("background") {
        None => Background::Mask,
        v => Background::Color(cor_de(v)?),
    };
    Some(EfeitoDeCamada::Outline(Outline {
        color: cor_de(m.get("color"))?,
        background,
        matrix,
        place: if flag("inside")? {
            Place::Inside
        } else {
            Place::Outside
        },
        tiled: flag("tiled")?,
    }))
}

fn cru_prop(c: &Cel) -> PropValue {
    let b = c.bbox;
    let rect = PropValue::Rect(b.x, b.y, b.w as i32, b.h as i32);
    mapa([("rect", rect), ("bytes", bytes(&c.data))])
}

fn cru_de(v: &PropValue) -> Option<(Rect, Vec<u8>)> {
    let PropValue::Map(m) = v else {
        return None;
    };
    let Some(&PropValue::Rect(x, y, w, h)) = m.get("rect") else {
        return None;
    };
    let (w, h) = (u32::try_from(w).ok()?, u32::try_from(h).ok()?);
    Some((Rect { x, y, w, h }, bytes_de(m.get("bytes")?)?))
}

fn u32_de(m: &PropMap, k: &str) -> Option<u32> {
    match m.get(k)? {
        PropValue::U32(v) => Some(*v),
        _ => None,
    }
}

/// O sprite que vai ao ficheiro: o próprio se não há nada da P13, senão um clone assado e anotado.
pub(super) fn assa(s: &Sprite) -> Cow<'_, Sprite> {
    let pal = s.ramps();
    if pal.ramps.is_empty() && s.layers().iter().all(|l| l.efeitos.is_empty()) {
        return Cow::Borrowed(s);
    }
    let mut c = s.clone();
    if !pal.ramps.is_empty() {
        let rampas = pal.ramps.iter().map(|r| {
            mapa([
                ("name", PropValue::Str(r.name.clone())),
                ("indices", bytes(&r.indices)),
            ])
        });
        poe(
            &mut c,
            Owner::Sprite,
            RAMPS_PROP,
            PropValue::Vector(rampas.collect()),
        );
        let ciclos: Vec<PropValue> = pal
            .cycles
            .iter()
            .filter_map(|y| {
                let pos = pal.ramps.iter().position(|r| r.id == y.ramp)? as u32;
                Some(mapa([
                    ("ramp", PropValue::U32(pos)),
                    ("period_ms", PropValue::U32(y.periodo_ms)),
                    ("direction", PropValue::U8(y.direcao.code())),
                ]))
            })
            .collect();
        if !ciclos.is_empty() {
            poe(
                &mut c,
                Owner::Sprite,
                CYCLES_PROP,
                PropValue::Vector(ciclos),
            );
        }
    }
    for l in s.layers().iter().filter(|l| !l.efeitos.is_empty()) {
        let efeitos = l.efeitos.iter().map(efeito_prop).collect();
        poe(
            &mut c,
            Owner::Layer(l.id),
            EFEITOS_PROP,
            PropValue::Vector(efeitos),
        );
        let Some(Op::Batch(ops)) = aplicar_efeitos(&c, l.id) else {
            continue;
        };
        let assados: Vec<CelKey> = ops
            .iter()
            .filter_map(|op| match op {
                Op::Patch(p) => Some((p.layer, p.frame)),
                _ => None,
            })
            .collect();
        if apply(&mut c, Op::Batch(ops)).is_err() {
            continue;
        }
        // Depois do assado: o Patch que alarga a caixa refaz o cel.
        for key in assados {
            if let Some(cel) = s.cel(key) {
                poe(&mut c, Owner::Cel(key), CRU_PROP, cru_prop(cel));
            }
        }
    }
    Cow::Owned(c)
}

/// O inverso de [`assa`] sobre o sprite lido: repõe rampas, ciclos, efeitos e os cels crus.
pub(super) fn repoe(s: &mut Sprite) {
    let rampa = |v: &PropValue| {
        let PropValue::Map(m) = v else {
            return None;
        };
        let Some(PropValue::Str(name)) = m.get("name") else {
            return None;
        };
        Some((name.clone(), bytes_de(m.get("indices")?)?))
    };
    let lista = |v: &PropValue| match v {
        PropValue::Vector(v) => Some(v.clone()),
        _ => None,
    };
    if let Some(v) = tira(s, Owner::Sprite, RAMPS_PROP, lista) {
        let mut ids = Vec::new();
        for (name, indices) in v.iter().filter_map(rampa) {
            if apply(s, Op::Doc(Box::new(DocOp::AddRamp { name, indices }))).is_ok() {
                ids.extend(s.ramps().ramps.last().map(|r| r.id));
            }
        }
        for m in tira(s, Owner::Sprite, CYCLES_PROP, lista).unwrap_or_default() {
            let PropValue::Map(m) = m else { continue };
            let Some(&ramp) = u32_de(&m, "ramp").and_then(|p| ids.get(p as usize)) else {
                continue;
            };
            let dir = match m.get("direction") {
                Some(PropValue::U8(d)) => CycleDirection::from_code(*d),
                _ => None,
            };
            let (Some(periodo_ms), Some(direcao)) = (u32_de(&m, "period_ms"), dir) else {
                continue;
            };
            let cycle = Some(Cycle {
                ramp,
                periodo_ms,
                direcao,
            });
            let _ = apply(s, Op::Doc(Box::new(DocOp::SetCycle { ramp, cycle })));
        }
    }
    let camadas: Vec<LayerId> = s.layers().iter().map(|l| l.id).collect();
    for layer in camadas {
        let todos = |v: &PropValue| lista(v)?.iter().map(efeito_de).collect::<Option<Vec<_>>>();
        if let Some(efeitos) = tira(s, Owner::Layer(layer), EFEITOS_PROP, todos) {
            let _ = apply(
                s,
                Op::Doc(Box::new(DocOp::SetLayerEffects { layer, efeitos })),
            );
        }
    }
    let bpp = s.mode().bpp();
    let cels: Vec<CelKey> = s.entries().map(|(k, _)| *k).collect();
    for key in cels {
        let certo = |v: &PropValue| cru_de(v).filter(|(r, px)| px.len() == r.area() * bpp);
        if let Some((rect, pixels)) = tira(s, Owner::Cel(key), CRU_PROP, certo) {
            let (layer, frame) = key;
            let bbox = Some(rect);
            let _ = apply(
                s,
                Op::Patch(Patch {
                    layer,
                    frame,
                    bbox,
                    rect,
                    pixels,
                }),
            );
        }
    }
}
