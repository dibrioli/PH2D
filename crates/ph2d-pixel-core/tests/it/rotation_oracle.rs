//! **Giro fora de 90° e inclinação contra o Aseprite Trial** — `oracle_trial_rotation.json` (blocos
//! cheios de ids distintos: cada célula da saída diz de que célula da fonte veio) e os `xform_` de
//! `oracle_trial_selection.json` (a forma de 19 ids, pelo caminho levantar → transformar → assentar).
//!
//! Lei da *Fast Rotation* (`affine::transform_image`, `Resample::Fast`): os quatro cantos da
//! moldura exacta vão ao `floor`; a fonte estica-se sobre esse quadrilátero inteiro; cada célula
//! copia a célula da fonte sob o seu centro, e o empate resolve-se um nada à esquerda. O que a lei
//! não explica é RESÍDUO DECLARADO (caso → células), nunca tabela copiada.

use std::collections::BTreeMap;

use ph2d_pixel_core::Rect;
use ph2d_pixel_core::affine::{Affine, Linear, Resample, transform_image};
use ph2d_pixel_core::canvas::{self, SizeMethod};
use ph2d_pixel_core::edit;
use ph2d_pixel_core::transform::{Image, lift};
use serde_json::Value;

use crate::selection_oracle::{cases, fx, rgba_sprite};
use crate::transform_oracle::{lifted_box, run, sz_diff, xf_ids, xf_sprite};

fn rot_fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_trial_rotation.json")).expect("json")
}

/// O campo do Trial (ângulo anti-horário; inclinação positiva = o lado de BAIXO para a direita).
/// Os campos `w`/`h` são da forma 7×5 de `xf.ase`.
fn linear(fields: &Value) -> Linear {
    let angle = fields["angle"].as_f64().unwrap_or(0.0);
    let skew = fields["skew"].as_f64().unwrap_or(0.0);
    let sx = fields["w"].as_f64().map_or(1.0, |w| w / 7.0);
    let sy = fields["h"].as_f64().map_or(1.0, |h| h / 5.0);
    Linear::from_screen_ccw(angle, sx, sy, -skew)
}

fn method(c: &Value) -> Resample {
    if c["algorithm"] == "rotsprite" {
        Resample::RotSprite
    } else {
        Resample::Fast
    }
}

fn ids(c: &Value) -> Vec<Vec<i64>> {
    serde_json::from_value(c["ids"].clone()).expect("ids")
}

fn cells_off(got: &[Vec<i64>], want: &[Vec<i64>]) -> usize {
    got.iter()
        .flatten()
        .zip(want.iter().flatten())
        .filter(|(a, b)| a != b)
        .count()
}

/// Um caso de bloco: a fonte `w × h` com ids `1..=w·h`, transformada em volta do centro.
fn block_off(f: &Value, c: &Value) -> usize {
    let b = &f["inputs"][c["block"].as_str().expect("block")]["box"];
    let v: Vec<i64> = serde_json::from_value(b.clone()).expect("box");
    let (bx, by, bw, bh) = (v[0] as i32, v[1] as i32, v[2] as u32, v[3] as u32);
    let src = Image::from_values(bw, bh, 1, &(1..=bw * bh).collect::<Vec<_>>());
    let t = Affine::around_center(linear(&c["fields"]), Rect::new(bx, by, bw, bh));
    let (r, img) = transform_image(&src, &t, method(c), 0, None).expect("transform");
    let want = ids(c);
    let mut got = vec![vec![0i64; want[0].len()]; want.len()];
    for y in 0..img.h {
        for x in 0..img.w {
            let (gx, gy) = (r.x + x as i32, r.y + y as i32);
            let id = img.get(x, y);
            if id != 0
                && gy >= 0
                && gx >= 0
                && (gy as usize) < got.len()
                && (gx as usize) < got[0].len()
            {
                got[gy as usize][gx as usize] = i64::from(id);
            }
        }
    }
    cells_off(&got, &want)
}

/// Um `xform_` da forma `xf.ase`: levantar, transformar, assentar.
fn xf_off(f: &Value, c: &Value) -> usize {
    let mut s = xf_sprite(f);
    let key = (s.layers()[0].id, s.frames()[0].id);
    let original = lifted_box(&s);
    let fl = lift(&s, key, Some(&original)).expect("lift");
    let t = Affine::around_center(linear(&c["fields"]), fl.rect());
    let moved = edit::transform(&fl, &t, method(c), 0).expect("t");
    let op = edit::commit(&s, key, &original, &moved, 0);
    run(&mut s, op);
    cells_off(&xf_ids(f, &s), &ids(c))
}

/// Todos os casos de giro/inclinação de um algoritmo: `nome → células diferentes`.
fn measure(alg: &str) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    let f = fx();
    for (k, c) in cases(&f, &format!("xform_{alg}_")) {
        if c["fields"]["w"].is_u64() && alg != "fast" {
            continue; // a escala do Trial é nearest nos dois algoritmos: `transform_oracle`
        }
        out.insert(k.clone(), xf_off(&f, c));
    }
    let r = rot_fx();
    for (k, c) in cases(&r, &format!("rot_{alg}_")) {
        out.insert(k.clone(), block_off(&r, c));
    }
    out
}

/// O que a lei da Fast Rotation ainda não explica (caso → células), medido: TODAS são empates
/// exactos (o centro sobre uma linha da grelha da fonte) — 36 de 785 empates; os outros 749 seguem
/// o «um nada à esquerda». A aritmética do Trial que os decide não foi achada (handoff P6g).
const FAST_RESIDUAL: [(&str, usize); 10] = [
    ("rot_fast_r58_a150", 2),
    ("rot_fast_r58_a53", 2),
    ("rot_fast_r58_a60", 2),
    ("rot_fast_r75_a100", 3),
    ("rot_fast_r86_a-37", 5),
    ("rot_fast_r86_a-60", 2),
    ("rot_fast_r86_a143", 5),
    ("rot_fast_r86_a150", 3),
    ("rot_fast_r86_a60", 2),
    ("rot_fast_r86_a72", 10),
];

#[test]
fn fast_rotation_and_skew_match_the_trial_cell_for_cell() {
    let got = measure("fast");
    let off: BTreeMap<String, usize> = got
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(k, n)| (k.clone(), *n))
        .collect();
    let want: BTreeMap<String, usize> = FAST_RESIDUAL
        .iter()
        .map(|(k, n)| ((*k).to_owned(), *n))
        .collect();
    assert_eq!(off, want, "o resíduo declarado da Fast Rotation mudou");
    assert_eq!(
        got.len(),
        86,
        "14 da forma xf (10 giros, inclinação, 3 escalas) + 72 de blocos"
    );
}

/// O RotSprite (o EPX×8 do MVP lido no centro): o padrão do Trial NÃO foi achado — ele encolhe a
/// cobertura (31–39 de 48 células num bloco 8×6) e amostra diferente do Fast. Recusas medidas
/// (handoff P6g): EPX×8 + nearest com qualquer translação; a lei do Fast a K = 1, 2, 4, 8 com
/// qualquer sub-amostra (melhor: K = 1, 282 células nos 18 blocos); moda / limiar de opacos na
/// redução 8×8 (≥ 467). A distância medida fica gravada para que mudar o RotSprite se veja.
const ROTSPRITE_GAP: [(&str, usize); 28] = [
    ("rot_rotsprite_r75_a-30", 37),
    ("rot_rotsprite_r75_a100", 35),
    ("rot_rotsprite_r75_a15", 26),
    ("rot_rotsprite_r75_a150", 20),
    ("rot_rotsprite_r75_a26", 32),
    ("rot_rotsprite_r75_a45", 20),
    ("rot_rotsprite_r75_a60", 33),
    ("rot_rotsprite_r75_a8", 41),
    ("rot_rotsprite_r75_k20", 16),
    ("rot_rotsprite_r86_a-30", 31),
    ("rot_rotsprite_r86_a100", 46),
    ("rot_rotsprite_r86_a15", 33),
    ("rot_rotsprite_r86_a150", 49),
    ("rot_rotsprite_r86_a26", 38),
    ("rot_rotsprite_r86_a45", 44),
    ("rot_rotsprite_r86_a60", 31),
    ("rot_rotsprite_r86_a8", 55),
    ("rot_rotsprite_r86_k20", 36),
    ("size_rotsprite_12x8", 72),
    ("size_rotsprite_25x13", 135),
    ("size_rotsprite_36x24", 264),
    ("xform_rotsprite_rot-30", 22),
    ("xform_rotsprite_rot135", 25),
    ("xform_rotsprite_rot15", 16),
    ("xform_rotsprite_rot30", 11),
    ("xform_rotsprite_rot33", 10),
    ("xform_rotsprite_rot45", 13),
    ("xform_rotsprite_skew20", 8),
];

#[test]
fn rotsprite_rotation_gap_to_the_trial_is_the_declared_one() {
    let mut got: BTreeMap<String, usize> = measure("rotsprite")
        .into_iter()
        .filter(|(k, _)| !k.ends_with("rot90"))
        .collect();
    let f = fx();
    for (k, c) in cases(&f, "size_rotsprite_") {
        let (w, h) = (
            c["w"].as_u64().expect("w") as u32,
            c["h"].as_u64().expect("h") as u32,
        );
        if w % 24 == 0 && h % 16 == 0 {
            continue; // nearest a 2×: `transform_oracle`
        }
        let mut s = rgba_sprite(f["inputs"]["sz_rows_rgba"].as_array().expect("sz"));
        let op = canvas::sprite_size(&s, w, h, SizeMethod::RotSprite);
        run(&mut s, op);
        got.insert(k.clone(), sz_diff(&f, &s, c));
    }
    let want: BTreeMap<String, usize> = ROTSPRITE_GAP
        .iter()
        .map(|(k, n)| ((*k).to_owned(), *n))
        .collect();
    assert_eq!(got, want, "a distância do RotSprite ao Trial mudou");
}
