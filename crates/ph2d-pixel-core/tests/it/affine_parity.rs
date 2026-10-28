//! **A moldura da transformação contra o MVP web** — `mvp_selection.json`, famílias `affine`
//! (decompor/compor, assentar, arrastar cada alça, trocar os números) e `xform` (a fonte
//! transformada pelo RotSprite do CENTRO, com e sem recorte, e os números da barra; o Fast é o do
//! Trial e os casos em que diverge do nearest do MVP estão contados). Os números são `f64` do JS: comparam-se a 1e-9 relativo; as células, ao byte.

use ph2d_pixel_core::Rect;
use ph2d_pixel_core::affine::{
    Affine, DragOpts, FieldPatch, Handle, Linear, Pivot, Point, Resample, transform_image,
};
use ph2d_pixel_core::transform::Image;
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/mvp_selection.json")).expect("json")
}

fn num(v: &Value) -> f64 {
    v.as_f64().expect("num")
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * 1f64.max(a.abs()).max(b.abs())
}

fn affine(v: &Value) -> Affine {
    Affine {
        a: num(&v["a"]),
        b: num(&v["b"]),
        c: num(&v["c"]),
        d: num(&v["d"]),
        e: num(&v["e"]),
        f: num(&v["f"]),
    }
}

fn same_affine(got: &Affine, want: &Value, what: &str) {
    let w = affine(want);
    let ok = [
        (got.a, w.a),
        (got.b, w.b),
        (got.c, w.c),
        (got.d, w.d),
        (got.e, w.e),
        (got.f, w.f),
    ]
    .iter()
    .all(|&(a, b)| close(a, b));
    assert!(ok, "{what}: {got:?} ≠ {w:?}");
}

fn same_linear(got: Linear, want: &Value, what: &str) {
    let ok = close(got.rot, num(&want["rot"]))
        && close(got.sx, num(&want["sx"]))
        && close(got.sy, num(&want["sy"]))
        && close(got.skew, num(&want["skew"]));
    assert!(ok, "{what}: {got:?} ≠ {want}");
}

fn point(v: &Value) -> Point {
    Point {
        x: num(&v["x"]),
        y: num(&v["y"]),
    }
}

fn pivot(v: &Value) -> Pivot {
    Pivot {
        u: num(&v["u"]),
        v: num(&v["v"]),
    }
}

fn handle(s: &str) -> Handle {
    match s {
        "move" => Handle::Move,
        "pivot" => Handle::Pivot,
        h if h.starts_with("rot-") => Handle::Rotate,
        h => {
            let (skew, side) = match h.strip_prefix("skew-") {
                Some(side) => (true, side),
                None => (false, h),
            };
            let (u, v) = match side {
                "nw" => (0, 0),
                "n" => (1, 0),
                "ne" => (2, 0),
                "e" => (2, 1),
                "se" => (2, 2),
                "s" => (1, 2),
                "sw" => (0, 2),
                "w" => (0, 1),
                other => panic!("alça {other}"),
            };
            if skew {
                Handle::Skew(u, v)
            } else {
                Handle::Scale(u, v)
            }
        }
    }
}

#[test]
fn decompose_compose_settle_drag_and_fields_match_the_mvp() {
    let f = fx();
    let cs = f["affine"].as_array().expect("affine");
    for (k, c) in cs.iter().enumerate() {
        let (w, h) = (num(&c["w"]) as u32, num(&c["h"]) as u32);
        let t = affine(&c["t"]);
        let dec = t.decompose();
        same_linear(dec, &c["decompose"], &format!("{k} decompose"));
        same_affine(
            &Affine::compose(dec, t.e, t.f),
            &c["compose"],
            &format!("{k} compose"),
        );
        same_affine(&t.settle(), &c["settle"], &format!("{k} settle"));
        let o = &c["opts"];
        let opt = DragOpts {
            keep_ratio: o["keepRatio"].as_bool().expect("k"),
            from_center: o["fromCenter"].as_bool().expect("c"),
            snap: o["snap"].as_bool().expect("s"),
        };
        let (dt, dp) = t.drag(
            pivot(&c["pivot"]),
            (w, h),
            handle(c["handle"].as_str().expect("h")),
            (point(&c["from"]), point(&c["to"])),
            opt,
        );
        same_affine(&dt, &c["drag"]["t"], &format!("{k} drag {}", c["handle"]));
        let wp = pivot(&c["drag"]["pivot"]);
        assert!(close(dp.u, wp.u) && close(dp.v, wp.v), "{k} pivô");
        let p = &c["patch"];
        let patch = FieldPatch {
            x: p["x"].as_f64(),
            y: p["y"].as_f64(),
            w: p["w"].as_f64(),
            h: p["h"].as_f64(),
            rot: p["rot"].as_f64(),
            skew: p["skew"].as_f64(),
        };
        let keep = c["keep_ratio"].as_bool().expect("kr");
        let wf = t.with_fields(pivot(&c["pivot"]), w, h, patch, keep);
        same_affine(&wf, &c["with_fields"], &format!("{k} with_fields {p}"));
    }
    assert_eq!(cs.len(), 80);
}

fn rect(v: &Value) -> Option<Rect> {
    (!v.is_null()).then(|| {
        Rect::new(
            num(&v["x"]) as i32,
            num(&v["y"]) as i32,
            num(&v["w"]) as u32,
            num(&v["h"]) as u32,
        )
    })
}

fn values(v: &Value) -> Vec<u32> {
    v.as_array()
        .expect("arr")
        .iter()
        .map(|x| x.as_u64().expect("u") as u32)
        .collect()
}

#[test]
fn transform_image_fast_and_rotsprite_and_the_bar_fields_match_the_mvp() {
    let f = fx();
    let cs = f["xform"].as_array().expect("xform");
    let mut fast_off = Vec::new();
    for (k, c) in cs.iter().enumerate() {
        let (w, h) = (num(&c["w"]) as u32, num(&c["h"]) as u32);
        let src = Image::from_values(w, h, 1, &values(&c["data"]));
        let t = affine(&c["t"]);
        let clip = rect(&c["clip"]);
        for (name, m) in [
            ("nearest", Resample::Fast),
            ("rotsprite", Resample::RotSprite),
        ] {
            let got = transform_image(&src, &t, m, 0, clip);
            match &c[name] {
                Value::Null => assert!(got.is_none(), "{k} {name}"),
                want => {
                    let (r, img) = got.unwrap_or_else(|| panic!("{k} {name} None"));
                    let same = Some(r) == rect(want) && img.values() == values(&want["data"]);
                    if m == Resample::Fast {
                        // o Fast é o do Trial (`rotation_oracle`): fora dos giros de 90° o
                        // nearest do MVP (centro da célula sobre a moldura assentada) diverge
                        if !same {
                            fast_off.push(k);
                        }
                        continue;
                    }
                    assert_eq!(Some(r), rect(want), "{k} {name} caixa");
                    assert_eq!(img.values(), values(&want["data"]), "{k} {name}");
                }
            }
        }
        let fl = t.fields(w, h);
        let wf = &c["fields"];
        assert_eq!(
            (fl.x, fl.y),
            (num(&wf["x"]) as i32, num(&wf["y"]) as i32),
            "{k}"
        );
        assert!(
            close(fl.w, num(&wf["w"]))
                && close(fl.h, num(&wf["h"]))
                && close(fl.rot, num(&wf["rot"]))
                && close(fl.skew, num(&wf["skew"])),
            "{k} números {fl:?}"
        );
    }
    assert_eq!(cs.len(), 50);
    assert_eq!(
        fast_off, MVP_FAST_CONFLICTS,
        "os casos em que o Trial e o MVP divergem"
    );
}

/// Os casos `xform` (índice) em que o Fast do Trial e o nearest do MVP dão células diferentes:
/// 47 de 50 (só 4, 13 e 25 coincidem) — o MVP amostra a moldura ASSENTADA sem levar os cantos ao
/// `floor`; o Trial ganha nos pixels (`rotation_oracle`, também as 3 escalas pelos campos).
const MVP_FAST_CONFLICTS: [usize; 47] = [
    0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26, 27, 28,
    29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
];
