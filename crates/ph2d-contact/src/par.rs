//! **A LEI DE UM PAR** — dados dois colisores e onde eles estão, quão fundo se tocam, para que
//! lado sair e ONDE a força age.
//!
//! Irmã do `lib.rs` pelo tecto de LOC (HR-18) e por ASSUNTO: ali mora a NUVEM (a grelha, o Jacobi
//! com média, a ordem das somas), aqui mora a geometria de DOIS. O ponto de cada par é o que
//! sustenta a rotação do doc 109 §6 — ver o cabeçalho do `lib.rs` para a fórmula que o consome.

use super::{Colisor, Contacto, EPS, Forma, dot, perp};

/// **O contacto de um par**: a normal de `a` para `b`, a profundidade e o ponto, ou `None` se não se
/// tocam. `eixo_x` escolhe o eixo de dois centros de disco COINCIDENTES (`true` ⇒ `x`, senão `y`) —
/// o desempate do `motion.collide`, que o solver tira da paridade do par.
pub fn contato(
    a: &Colisor,
    pa: [f32; 2],
    b: &Colisor,
    pb: [f32; 2],
    eixo_x: bool,
) -> Option<Contacto> {
    let (ca, cb) = (a.centro(pa), b.centro(pb));
    match (a.forma, b.forma) {
        (Forma::Disco(ra), Forma::Disco(rb)) => discos(ca, ra, cb, rb, eixo_x),
        (Forma::Caixa { meia: ma, eixo: ea }, Forma::Caixa { meia: mb, eixo: eb }) => {
            caixas(ca, ma, ea, cb, mb, eb)
        }
        // O ponto da caixa mais próximo dá a normal da CAIXA para o DISCO.
        (Forma::Caixa { meia, eixo }, Forma::Disco(r)) => disco_caixa(cb, r, ca, meia, eixo),
        (Forma::Disco(r), Forma::Caixa { meia, eixo }) => {
            disco_caixa(ca, r, cb, meia, eixo).map(|c| Contacto {
                normal: [-c.normal[0], -c.normal[1]],
                ..c
            })
        }
    }
}

fn discos(ca: [f32; 2], ra: f32, cb: [f32; 2], rb: f32, eixo_x: bool) -> Option<Contacto> {
    let min_dist = ra + rb;
    let min_d2 = min_dist * min_dist;
    let dx = cb[0] - ca[0];
    let dy = cb[1] - ca[1];
    let d2 = dx * dx + dy * dy;
    if d2 >= min_d2 {
        return None;
    }
    let (normal, penetracao) = if d2 > EPS {
        let d = d2.sqrt();
        ([dx / d, dy / d], min_dist - d)
    } else if eixo_x {
        ([1.0, 0.0], min_dist)
    } else {
        ([0.0, 1.0], min_dist)
    };
    // O ponto: sobre a normal, a meio do trecho sobreposto.
    let t = ra - penetracao * 0.5;
    Some(Contacto {
        normal,
        penetracao,
        ponto: [ca[0] + normal[0] * t, ca[1] + normal[1] * t],
    })
}

/// **Duas caixas orientadas** — o eixo separador de menor sobreposição dá a normal (de `a` para
/// `b`), e o recorte das faces dá o ponto.
fn caixas(
    ca: [f32; 2],
    ma: [f32; 2],
    ea: [f32; 2],
    cb: [f32; 2],
    mb: [f32; 2],
    eb: [f32; 2],
) -> Option<Contacto> {
    let d = [cb[0] - ca[0], cb[1] - ca[1]];
    let (va, vb) = (perp(ea), perp(eb));
    // `melhor` guarda também DE QUEM é o eixo: a face de referência é a dessa caixa.
    let mut melhor: Option<([f32; 2], f32, bool)> = None;
    for (eixo, de_a) in [(ea, true), (va, true), (eb, false), (vb, false)] {
        let dist = dot(d, eixo);
        let ra = ma[0] * dot(ea, eixo).abs() + ma[1] * dot(va, eixo).abs();
        let rb = mb[0] * dot(eb, eixo).abs() + mb[1] * dot(vb, eixo).abs();
        let sobra = ra + rb - dist.abs();
        if sobra <= 0.0 {
            return None;
        }
        if melhor.is_none_or(|(_, s, _)| sobra < s) {
            let n = if dist < 0.0 {
                [-eixo[0], -eixo[1]]
            } else {
                eixo
            };
            melhor = Some((n, sobra, de_a));
        }
    }
    let (normal, penetracao, de_a) = melhor?;
    // A face de REFERÊNCIA é a da caixa cujo eixo ganhou, e a normal dela aponta para a outra.
    let (cr, mr, er, ci, mi, ei, n_ref) = if de_a {
        (ca, ma, ea, cb, mb, eb, normal)
    } else {
        (cb, mb, eb, ca, ma, ea, [-normal[0], -normal[1]])
    };
    Some(Contacto {
        normal,
        penetracao,
        ponto: ponto_de_contacto(cr, mr, er, ci, mi, ei, n_ref),
    })
}

/// **O ponto onde duas caixas se tocam** — o MEIO do trecho da face incidente que cai dentro da
/// face de referência.
///
/// ⚠️ **A média, e não o vértice mais fundo:** de chapa uma sobre a outra os dois vértices estão à
/// mesma profundidade, e escolher um deles dá binário a uma pilha parada — ela tomba sozinha.
fn ponto_de_contacto(
    cr: [f32; 2],
    mr: [f32; 2],
    er: [f32; 2],
    ci: [f32; 2],
    mi: [f32; 2],
    ei: [f32; 2],
    n: [f32; 2],
) -> [f32; 2] {
    let (vr, vi) = (perp(er), perp(ei));
    // A face INCIDENTE: a da outra caixa cuja normal é mais anti-paralela a `n`.
    let faces = [
        (ei, mi[0], mi[1]),
        (vi, mi[1], mi[0]),
        ([-ei[0], -ei[1]], mi[0], mi[1]),
        ([-vi[0], -vi[1]], mi[1], mi[0]),
    ];
    let (face_n, meia_fora, meia_lado) = faces
        .into_iter()
        .min_by(|a, b| dot(a.0, n).total_cmp(&dot(b.0, n)))
        .unwrap_or(faces[0]);
    let lado = perp(face_n);
    let centro = [ci[0] + face_n[0] * meia_fora, ci[1] + face_n[1] * meia_fora];
    let p1 = [
        centro[0] - lado[0] * meia_lado,
        centro[1] - lado[1] * meia_lado,
    ];
    let p2 = [
        centro[0] + lado[0] * meia_lado,
        centro[1] + lado[1] * meia_lado,
    ];
    // Recortar contra os lados da face de referência, ao longo da tangente dela.
    let tang = perp(n);
    let meia_tang = mr[0] * dot(er, tang).abs() + mr[1] * dot(vr, tang).abs();
    let t1 = dot([p1[0] - cr[0], p1[1] - cr[1]], tang);
    let t2 = dot([p2[0] - cr[0], p2[1] - cr[1]], tang);
    let (lo, hi) = (t1.min(t2), t1.max(t2));
    let (clo, chi) = (lo.max(-meia_tang), hi.min(meia_tang));
    let meio = if chi >= clo {
        (clo + chi) * 0.5
    } else {
        (lo + hi) * 0.5
    };
    let denom = t2 - t1;
    let u = if denom.abs() > EPS {
        (meio - t1) / denom
    } else {
        0.5
    };
    [p1[0] + (p2[0] - p1[0]) * u, p1[1] + (p2[1] - p1[1]) * u]
}

/// **Um disco contra uma caixa** — a normal da CAIXA para o DISCO, a profundidade e o ponto.
///
/// ⚠️ Os dois ramos são geometricamente diferentes e ambos necessários: **fora**, a distância ao
/// ponto mais próximo decide e arredonda as quinas; **dentro**, não há direcção «para fora» única, e
/// sai-se pela face de MENOR penetração (sem este ramo, um disco que atravessou a face num tique
/// grande ficaria preso).
pub fn disco_caixa(
    cd: [f32; 2],
    r: f32,
    cc: [f32; 2],
    meia: [f32; 2],
    eixo: [f32; 2],
) -> Option<Contacto> {
    let v = perp(eixo);
    let d = [cd[0] - cc[0], cd[1] - cc[1]];
    let (lx, ly) = (dot(d, eixo), dot(d, v));
    let (hx, hy) = (meia[0], meia[1]);
    let (qx, qy) = (lx.clamp(-hx, hx), ly.clamp(-hy, hy)); // CLAMP-OK: meias validadas >= 0
    let (ex, ey) = (lx - qx, ly - qy);
    let e2 = ex * ex + ey * ey;
    let mundo = |l: [f32; 2]| {
        [
            cc[0] + l[0] * eixo[0] + l[1] * v[0],
            cc[1] + l[0] * eixo[1] + l[1] * v[1],
        ]
    };
    let dir = |l: [f32; 2]| [l[0] * eixo[0] + l[1] * v[0], l[0] * eixo[1] + l[1] * v[1]];
    if e2 > EPS {
        let e = e2.sqrt();
        if e >= r {
            return None;
        }
        // O ponto é o da CAIXA mais próximo do disco — é ali que ela o toca.
        return Some(Contacto {
            normal: dir([ex / e, ey / e]),
            penetracao: r - e,
            ponto: mundo([qx, qy]),
        });
    }
    let (px, py) = (hx - lx.abs(), hy - ly.abs());
    let (nl, face) = if px < py {
        let s = if lx < 0.0 { -1.0 } else { 1.0 };
        ([s, 0.0], [s * hx, ly])
    } else {
        let s = if ly < 0.0 { -1.0 } else { 1.0 };
        ([0.0, s], [lx, s * hy])
    };
    Some(Contacto {
        normal: dir(nl),
        penetracao: px.min(py) + r,
        ponto: mundo(face),
    })
}
