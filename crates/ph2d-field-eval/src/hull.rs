//! ⭐⭐ **A GEOMETRIA CONVEXA da região** (W59) — o casco em `(u, v)`, e as duas operações que o
//! põem no sítio.
//!
//! Irmão do [`crate`] por responsabilidade (teto de LOC): ali mora *o que se especializa*, aqui *que
//! forma tem a região*. ⚠️ A ordem das duas operações é **load-bearing** — ver [`hull_uv`].

/// ⭐⭐⭐ **O CASCO EM `(u, v)` DA REGIÃO** (W59) — a pegada real do tubo no plano do perfil.
///
/// ⚠️ **Por que não a caixa:** o corte guarda toda aresta a menos de `dmax`, e o `dmax` cresce com o
/// **diâmetro** da região. A caixa de um tubo de viés tem uma diagonal muito maior que o tubo —
/// medido, ela guarda `1,21×`–`1,28×` mais arestas do que a forma real (câmera de viés).
///
/// ⚠️ **Ele é recortado pela caixa local e INFLADO pela mesma folga que ela levou.** A caixa que
/// chega já vem ∩ com a peça e com a folga da sonda da normal somada; o casco sai dos cantos CRUS,
/// então as duas coisas têm de lhe ser feitas à mão, senão ele seria mais apertado do que a região
/// que a marcha de facto avalia — e uma região apertada demais **fura a peça**.
///
/// ⚠️ A inflação empurra cada **lado** para fora: o offset verdadeiro é arredondado nas quinas, e o
/// polígono de lados empurrados **contém-no**. *Conservador é o único lado seguro.*
pub(crate) fn hull_uv(pts: &[[f32; 3]], lo: [f32; 2], hi: [f32; 2]) -> Vec<[f32; 2]> {
    if pts.len() < 3 {
        return Vec::new();
    }
    // A folga que a caixa levou, lida da própria caixa: o que ela tem além dos pontos.
    let (mut plo, mut phi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for p in pts {
        for k in 0..2 {
            plo[k] = plo[k].min(p[k]);
            phi[k] = phi[k].max(p[k]);
        }
    }
    let pad = (0..2)
        .map(|k| (plo[k] - lo[k]).max(hi[k] - phi[k]))
        .fold(0.0f32, f32::max)
        .max(0.0);
    let flat: Vec<[f32; 2]> = pts.iter().map(|p| [p[0], p[1]]).collect();
    // ⚠️ **INFLA e só depois RECORTA, e a ordem é load-bearing.** ⛔ Ao contrário — recortar pela
    // caixa e inflar a seguir — o casco espeta **para fora** dela por até `pad`, e aí ele deixa de
    // estar contido na caixa: um gate de **monotonia** apanhou isso (uma região guardava MAIS
    // arestas com o casco do que com a caixa, o que é impossível para um subconjunto). Nesta ordem
    // ele é ⊆ caixa por construção, e continua a conter a região (a caixa contém-na).
    let inflated = inflate_convex(&convex_hull(flat), pad);
    clip_convex_to_rect(&inflated, lo, hi)
}

/// ⚠️ Só para o gate: o casco que a especialização de um `Extrude` de facto usa.
#[doc(hidden)]
#[must_use]
pub fn probe_hull_uv(pts: &[[f32; 3]], lo: [f32; 2], hi: [f32; 2]) -> Vec<[f32; 2]> {
    hull_uv(pts, lo, hi)
}

/// ⚠️ Só para o gate: este ponto está dentro do polígono convexo?
#[doc(hidden)]
#[must_use]
pub fn probe_in_hull(p: [f32; 2], hull: &[[f32; 2]]) -> bool {
    if hull.len() < 3 {
        return false;
    }
    let (mut pos, mut neg) = (false, false);
    for i in 0..hull.len() {
        let (a, b) = (hull[i], hull[(i + 1) % hull.len()]);
        let c = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        pos |= c > 0.0;
        neg |= c < 0.0;
    }
    !(pos && neg)
}

/// Casco convexo 2D (Andrew monotone chain), anti-horário.
fn convex_hull(mut pts: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    pts.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: [f32; 2], a: [f32; 2], b: [f32; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut out: Vec<[f32; 2]> = Vec::with_capacity(pts.len() * 2);
    for pass in 0..2u8 {
        let start = out.len();
        let seq: Vec<[f32; 2]> = if pass == 0 {
            pts.clone()
        } else {
            pts.iter().rev().copied().collect()
        };
        for p in seq {
            while out.len() >= start + 2 && cross(out[out.len() - 2], out[out.len() - 1], p) <= 0.0
            {
                out.pop();
            }
            out.push(p);
        }
        out.pop();
    }
    out
}

/// Corta o polígono convexo contra um rectângulo (Sutherland–Hodgman).
fn clip_convex_to_rect(poly: &[[f32; 2]], lo: [f32; 2], hi: [f32; 2]) -> Vec<[f32; 2]> {
    let mut cur = poly.to_vec();
    for (axis, bound, keep_ge) in [
        (0usize, lo[0], true),
        (0, hi[0], false),
        (1, lo[1], true),
        (1, hi[1], false),
    ] {
        if cur.is_empty() {
            break;
        }
        let inside = |p: &[f32; 2]| {
            if keep_ge {
                p[axis] >= bound
            } else {
                p[axis] <= bound
            }
        };
        let mut out: Vec<[f32; 2]> = Vec::with_capacity(cur.len() + 1);
        for i in 0..cur.len() {
            let (a, b) = (cur[i], cur[(i + 1) % cur.len()]);
            let (ia, ib) = (inside(&a), inside(&b));
            if ia {
                out.push(a);
            }
            if ia != ib && (b[axis] - a[axis]).abs() > f32::EPSILON {
                let t = (bound - a[axis]) / (b[axis] - a[axis]);
                out.push([a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]);
            }
        }
        cur = out;
    }
    cur
}

/// Empurra cada lado do polígono convexo para fora por `pad` e re-intersecta — ver [`hull_uv`].
fn inflate_convex(poly: &[[f32; 2]], pad: f32) -> Vec<[f32; 2]> {
    if poly.len() < 3 || pad <= 0.0 {
        return poly.to_vec();
    }
    let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for p in poly {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let big = (hi[0] - lo[0]).max(hi[1] - lo[1]) + 4.0 * pad + 1.0;
    let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let mut cur = vec![
        [c[0] - big, c[1] - big],
        [c[0] + big, c[1] - big],
        [c[0] + big, c[1] + big],
        [c[0] - big, c[1] + big],
    ];
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let e = [b[0] - a[0], b[1] - a[1]];
        let len = e[0].hypot(e[1]);
        if len <= f32::EPSILON {
            continue;
        }
        // A normal EXTERIOR de um polígono anti-horário é `(e.y, −e.x)`.
        let nrm = [e[1] / len, -e[0] / len];
        let off = [a[0] + nrm[0] * pad, a[1] + nrm[1] * pad];
        let side = |p: &[f32; 2]| (p[0] - off[0]) * nrm[0] + (p[1] - off[1]) * nrm[1];
        let mut out: Vec<[f32; 2]> = Vec::with_capacity(cur.len() + 1);
        for j in 0..cur.len() {
            let (u, v) = (cur[j], cur[(j + 1) % cur.len()]);
            let (su, sv) = (side(&u), side(&v));
            if su <= 0.0 {
                out.push(u);
            }
            if (su <= 0.0) != (sv <= 0.0) {
                let t = su / (su - sv);
                out.push([u[0] + t * (v[0] - u[0]), u[1] + t * (v[1] - u[1])]);
            }
        }
        cur = out;
        if cur.is_empty() {
            break;
        }
    }
    cur
}
