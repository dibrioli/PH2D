//! **A LEI DE UM PAR** — dados dois colisores e onde eles estão, quão fundo se tocam, para que
//! lado sair e ONDE a força age.
//!
//! Irmã do `lib.rs` pelo tecto de LOC (HR-18) e por ASSUNTO: ali mora a NUVEM (a grelha, o Jacobi
//! com média, a ordem das somas), aqui mora a geometria de DOIS. O ponto de cada par é o que
//! sustenta a rotação do doc 109 §6 — ver o cabeçalho do `lib.rs` para a fórmula que o consome.

use super::{Colisor, Contacto, EPS, Forma, dot, perp};

/// ⭐⭐⭐ **O ENCOSTO DE DOIS PONTOS** — o contacto de um par com o **TRECHO** onde eles se tocam,
/// e não só o meio dele (ordem do dono, 2026-09-15; mecanismo no doc 111 §5.10).
///
/// ## Porque UM ponto não chega
///
/// Uma caixa pousada **de face** sobre outra é, com um contacto pontual, um equilíbrio
/// **INSTÁVEL**: *um ponto não resiste a binário nenhum*. Medido na `=114`, o desalinho de um apoio
/// face-a-face cresce **`×1,4` por tique** desde `1,7°` até a peça tombar e a quina encravar nos
/// `45°` — que é o ponto fixo que o solver consegue segurar. Era o «salto» do 5.º report.
///
/// ## O que o segundo ponto compra, e o que ele NÃO compra
///
/// ⚠️⚠️ **Não é o segundo ponto: é a PROFUNDIDADE PRÓPRIA de cada um.** Com duas profundidades
/// iguais os dois pontos entregam exactamente o binário do ponto médio — *nada muda*. O que
/// endireita é a caixa **inclinada**, onde um canto está mais fundo que o outro: ali cada ponto
/// pede a sua correcção e a diferença entre elas **é** o binário restaurador que faltava.
///
/// ⛔ **Entre uma QUINA e uma face o contacto é um ponto POR GEOMETRIA**, e nada o desdobra — o
/// trecho tem comprimento zero. Isso não é uma limitação: os `45°` já são a configuração estável.
pub struct Manifesto {
    pontos: [Contacto; 2],
    n: usize,
}

impl Manifesto {
    /// Os pontos vivos — **um** ou **dois**, nunca zero (um `Manifesto` só existe se houver contacto).
    pub fn pontos(&self) -> &[Contacto] {
        &self.pontos[..self.n]
    }

    /// Um manifesto de um ponto só — o caso de toda forma que não é caixa-contra-caixa.
    fn unico(c: Contacto) -> Self {
        Self {
            pontos: [c, c],
            n: 1,
        }
    }
}

/// **O contacto de um par**: a normal de `a` para `b`, a profundidade e o ponto, ou `None` se não se
/// tocam. `eixo_x` escolhe o eixo de dois centros de disco COINCIDENTES (`true` ⇒ `x`, senão `y`) —
/// o desempate do `motion.collide`, que o solver tira da paridade do par.
///
/// ⚠️ **Esta é a leitura de UM ponto do [`manifesto`], não uma segunda lei** — de caixa contra
/// caixa ela é o MEIO do trecho, que é exactamente o que esta função sempre devolveu. Quem resolve
/// a pilha usa o manifesto; quem só precisa de saber *«tocam-se, e de que lado?»* (o colisor de
/// mundo, os censos) usa esta.
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

/// ⭐⭐⭐ **O manifesto de um par** — o mesmo contacto de [`contato`], mas com os **DOIS** extremos
/// do trecho quando eles existem, cada um com a **sua** profundidade.
///
/// Só caixa-contra-caixa produz dois; todas as outras formas devolvem o ponto único que já
/// devolviam, embrulhado. ⚠️ **Nenhuma delas muda um bit** — ver [`Manifesto`].
pub fn manifesto(
    a: &Colisor,
    pa: [f32; 2],
    b: &Colisor,
    pb: [f32; 2],
    eixo_x: bool,
) -> Option<Manifesto> {
    if let (Forma::Caixa { meia: ma, eixo: ea }, Forma::Caixa { meia: mb, eixo: eb }) =
        (a.forma, b.forma)
    {
        return caixas_trecho(a.centro(pa), ma, ea, b.centro(pb), mb, eb);
    }
    contato(a, pa, b, pb, eixo_x).map(Manifesto::unico)
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
    let (normal, penetracao, de_a) = eixo_separador(ca, ma, ea, cb, mb, eb)?;
    let r = referencia(ca, ma, ea, cb, mb, eb, normal, de_a);
    Some(Contacto {
        normal,
        penetracao,
        ponto: ponto_de_contacto(r.cr, r.mr, r.er, r.ci, r.mi, r.ei, r.n_ref),
    })
}

/// **O eixo separador de menor sobreposição**: a normal de `a` para `b`, a penetração, e se o eixo
/// que ganhou é o da caixa `a` (⇒ a face de REFERÊNCIA é a dela).
///
/// ⚠️ **Uma lei, dois leitores** ([`caixas`] e [`caixas_trecho`]): escrita duas vezes, ela
/// discordaria no dia em que alguém mexesse no critério — e o ponto e o trecho passariam a falar
/// de eixos diferentes.
fn eixo_separador(
    ca: [f32; 2],
    ma: [f32; 2],
    ea: [f32; 2],
    cb: [f32; 2],
    mb: [f32; 2],
    eb: [f32; 2],
) -> Option<([f32; 2], f32, bool)> {
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
    melhor
}

/// Quem é a REFERÊNCIA e quem é a INCIDENTE, depois de o eixo ter ganho.
struct Ref {
    cr: [f32; 2],
    mr: [f32; 2],
    er: [f32; 2],
    ci: [f32; 2],
    mi: [f32; 2],
    ei: [f32; 2],
    /// A normal da face de referência — ela aponta para a caixa incidente.
    n_ref: [f32; 2],
}

#[expect(
    clippy::too_many_arguments,
    reason = "duas caixas sao 6 numeros mais o eixo"
)]
fn referencia(
    ca: [f32; 2],
    ma: [f32; 2],
    ea: [f32; 2],
    cb: [f32; 2],
    mb: [f32; 2],
    eb: [f32; 2],
    normal: [f32; 2],
    de_a: bool,
) -> Ref {
    // A face de REFERÊNCIA é a da caixa cujo eixo ganhou, e a normal dela aponta para a outra.
    if de_a {
        Ref {
            cr: ca,
            mr: ma,
            er: ea,
            ci: cb,
            mi: mb,
            ei: eb,
            n_ref: normal,
        }
    } else {
        Ref {
            cr: cb,
            mr: mb,
            er: eb,
            ci: ca,
            mi: ma,
            ei: ea,
            n_ref: [-normal[0], -normal[1]],
        }
    }
}

/// ⭐⭐⭐ **O TRECHO onde duas caixas se tocam, com a profundidade de CADA extremo.**
///
/// A face incidente é recortada contra os lados da face de referência — exactamente o mesmo
/// recorte que o [`ponto_de_contacto`] já fazia para tirar o meio. O que muda é o que se faz com
/// ele: em vez de colapsar em `(clo + chi) / 2`, cada extremo sobrevive **com a distância dele à
/// face de referência**.
///
/// ⚠️⚠️ **É a profundidade própria que endireita, não o número de pontos** (ver [`Manifesto`]):
/// de chapa, os dois extremos estão à MESMA profundidade e entregam o binário do meio; inclinada,
/// o canto mais fundo pede mais correcção, e a diferença é o restaurador.
///
/// ⛔ Um extremo que não penetra (`profundidade <= 0`) **não é um contacto** e cai fora. Se os dois
/// caírem, sobra o ponto único de sempre — *uma quina contra uma face tem trecho de comprimento
/// zero, e isso é geometria, não uma falha do recorte*.
fn caixas_trecho(
    ca: [f32; 2],
    ma: [f32; 2],
    ea: [f32; 2],
    cb: [f32; 2],
    mb: [f32; 2],
    eb: [f32; 2],
) -> Option<Manifesto> {
    let (normal, penetracao, de_a) = eixo_separador(ca, ma, ea, cb, mb, eb)?;
    let r = referencia(ca, ma, ea, cb, mb, eb, normal, de_a);
    let unico = || {
        Manifesto::unico(Contacto {
            normal,
            penetracao,
            ponto: ponto_de_contacto(r.cr, r.mr, r.er, r.ci, r.mi, r.ei, r.n_ref),
        })
    };
    let Some((q1, q2)) = trecho_recortado(&r) else {
        return Some(unico());
    };
    // A profundidade de um ponto: quanto ele está para DENTRO da face de referência, medida na
    // normal dela. O plano da face passa a `meia_ref` do centro, ao longo de `n_ref`.
    let vr = perp(r.er);
    let meia_ref = r.mr[0] * dot(r.er, r.n_ref).abs() + r.mr[1] * dot(vr, r.n_ref).abs();
    let fundo = |q: [f32; 2]| meia_ref - dot([q[0] - r.cr[0], q[1] - r.cr[1]], r.n_ref);
    let mut pontos = [Contacto {
        normal,
        penetracao,
        ponto: q1,
    }; 2];
    let mut n = 0;
    for q in [q1, q2] {
        let d = fundo(q);
        if d > 0.0 {
            pontos[n] = Contacto {
                normal,
                penetracao: d,
                ponto: q,
            };
            n += 1;
        }
    }
    if n == 0 {
        return Some(unico());
    }
    Some(Manifesto { pontos, n })
}

/// Os DOIS extremos da face incidente depois de recortada contra a de referência, em mundo — ou
/// `None` se a face incidente for degenerada (um trecho de comprimento nulo).
fn trecho_recortado(r: &Ref) -> Option<([f32; 2], [f32; 2])> {
    let (p1, p2) = face_incidente(r.ci, r.mi, r.ei, r.n_ref);
    let tang = perp(r.n_ref);
    let vr = perp(r.er);
    let meia_tang = r.mr[0] * dot(r.er, tang).abs() + r.mr[1] * dot(vr, tang).abs();
    let t1 = dot([p1[0] - r.cr[0], p1[1] - r.cr[1]], tang);
    let t2 = dot([p2[0] - r.cr[0], p2[1] - r.cr[1]], tang);
    let (lo, hi) = (t1.min(t2), t1.max(t2));
    let (clo, chi) = (lo.max(-meia_tang), hi.min(meia_tang));
    if chi <= clo {
        return None;
    }
    let denom = t2 - t1;
    if denom.abs() <= EPS {
        return None;
    }
    let em = |t: f32| {
        let u = (t - t1) / denom;
        [p1[0] + (p2[0] - p1[0]) * u, p1[1] + (p2[1] - p1[1]) * u]
    };
    Some((em(clo), em(chi)))
}

/// **A face INCIDENTE**: a da outra caixa cuja normal é mais anti-paralela a `n`, com os dois
/// vértices dela. ⚠️ **Uma lei, dois leitores** — [`ponto_de_contacto`] e [`trecho_recortado`].
fn face_incidente(ci: [f32; 2], mi: [f32; 2], ei: [f32; 2], n: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    let vi = perp(ei);
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
    (
        [
            centro[0] - lado[0] * meia_lado,
            centro[1] - lado[1] * meia_lado,
        ],
        [
            centro[0] + lado[0] * meia_lado,
            centro[1] + lado[1] * meia_lado,
        ],
    )
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
    let vr = perp(er);
    // ⚠️ A face incidente sai da PORTA partilhada com o [`trecho_recortado`] — escrita duas vezes,
    // o ponto e o trecho podiam passar a falar de faces diferentes.
    let (p1, p2) = face_incidente(ci, mi, ei, n);
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
