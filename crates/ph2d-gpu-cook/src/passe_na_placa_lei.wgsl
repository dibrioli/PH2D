// O PASSE DE SEPARACAO na placa (doc 130) -- a lei de `ph2d-contact`, termo a termo.
// Cada funcao `pnp_*` tem o nome da porta da CPU que porta; a ordem das operacoes e' a dela.
// Este texto e' a BIBLIOTECA: nao declara bindings, cada kernel a recebe a' frente do corpo.

const PNP_EPS: f32 = 1e-9;
const PNP_FMAX: f32 = 3.4028235e38;

struct U { n: u32, num_buckets: u32, repouso: f32, tem: u32, bi: u32, bo: u32, varreduras: u32, inverte: u32 };

// Colisor: `kind` 0 nenhum, 1 disco (raio em m.x), 2 caixa (meias em m).
struct Col { kind: u32, m: vec2<f32>, eixo: vec2<f32>, desvio: vec2<f32> };
struct Cont { ok: bool, n: vec2<f32>, pen: f32, ponto: vec2<f32> };
struct Man { np: u32, a: Cont, b: Cont };
struct Sep { ok: bool, n: vec2<f32>, sobra: f32, de_a: bool };
struct Ref { cr: vec2<f32>, mr: vec2<f32>, er: vec2<f32>, ci: vec2<f32>, mi: vec2<f32>, ei: vec2<f32>, n_ref: vec2<f32> };
struct Seg { ok: bool, p1: vec2<f32>, p2: vec2<f32> };

fn pnp_finito(x: f32) -> bool { return x == x && abs(x) <= PNP_FMAX; }
fn pnp_dot(a: vec2<f32>, b: vec2<f32>) -> f32 { return a.x * b.x + a.y * b.y; }
fn pnp_cruz(a: vec2<f32>, b: vec2<f32>) -> f32 { return a.x * b.y - a.y * b.x; }
fn pnp_perp(e: vec2<f32>) -> vec2<f32> { return vec2<f32>(0.0 - e.y, e.x); }
fn pnp_zero(v: vec2<f32>) -> bool { return v.x == 0.0 && v.y == 0.0; }

// `ph2d_motion_kit::trig::sin_cycles` -- o seno parabolico corrigido, a MESMA polinomial.
fn pnp_sin_ciclos(fase: f32) -> f32 {
    let f = fase - floor(fase);
    var p = 0.0;
    if (f < 0.5) {
        let w = f * 2.0;
        p = 4.0 * w * (1.0 - w);
    } else {
        let w = (f - 0.5) * 2.0;
        p = -4.0 * w * (1.0 - w);
    }
    return 0.225 * (p * abs(p) - p) + p;
}

// `eixo_de`
fn pnp_eixo_de(graus: f32) -> vec2<f32> {
    if (graus == 0.0) { return vec2<f32>(1.0, 0.0); }
    let fase = graus / 360.0;
    let c = pnp_sin_ciclos(fase + 0.25);
    let s = pnp_sin_ciclos(fase);
    let inv = 1.0 / sqrt(c * c + s * s);
    return vec2<f32>(c * inv, s * inv);
}

fn pnp_gira(v: vec2<f32>, e: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(v.x * e.x - v.y * e.y, v.x * e.y + v.y * e.x);
}

// `Colisor::centro`
fn pnp_centro(c: Col, p: vec2<f32>) -> vec2<f32> {
    if (pnp_zero(c.desvio)) { return p; }
    return vec2<f32>(p.x + c.desvio.x, p.y + c.desvio.y);
}

// `Colisor::alcance`
fn pnp_alcance(c: Col) -> f32 {
    var longe = c.m.x;
    if (c.kind == 2u) { longe = sqrt(c.m.x * c.m.x + c.m.y * c.m.y); }
    if (pnp_zero(c.desvio)) { return longe; }
    return longe + sqrt(c.desvio.x * c.desvio.x + c.desvio.y * c.desvio.y);
}

fn pnp_nada() -> Cont { return Cont(false, vec2<f32>(0.0), 0.0, vec2<f32>(0.0)); }

// `par::discos`
fn pnp_discos(ca: vec2<f32>, ra: f32, cb: vec2<f32>, rb: f32, eixo_x: bool) -> Cont {
    let min_dist = ra + rb;
    let min_d2 = min_dist * min_dist;
    let dx = cb.x - ca.x;
    let dy = cb.y - ca.y;
    let d2 = dx * dx + dy * dy;
    if (d2 >= min_d2) { return pnp_nada(); }
    var n = vec2<f32>(0.0, 1.0);
    var pen = min_dist;
    if (d2 > PNP_EPS) {
        let d = sqrt(d2);
        n = vec2<f32>(dx / d, dy / d);
        pen = min_dist - d;
    } else if (eixo_x) {
        n = vec2<f32>(1.0, 0.0);
    }
    let t = ra - pen * 0.5;
    return Cont(true, n, pen, vec2<f32>(ca.x + n.x * t, ca.y + n.y * t));
}

// `par::eixo_separador`
fn pnp_eixo_separador(ca: vec2<f32>, ma: vec2<f32>, ea: vec2<f32>, cb: vec2<f32>, mb: vec2<f32>, eb: vec2<f32>) -> Sep {
    let d = vec2<f32>(cb.x - ca.x, cb.y - ca.y);
    let va = pnp_perp(ea);
    let vb = pnp_perp(eb);
    var eixos = array<vec2<f32>, 4>(ea, va, eb, vb);
    var melhor = Sep(false, vec2<f32>(0.0), 0.0, true);
    for (var k = 0u; k < 4u; k = k + 1u) {
        let eixo = eixos[k];
        let dist = pnp_dot(d, eixo);
        let ra = ma.x * abs(pnp_dot(ea, eixo)) + ma.y * abs(pnp_dot(va, eixo));
        let rb = mb.x * abs(pnp_dot(eb, eixo)) + mb.y * abs(pnp_dot(vb, eixo));
        let sobra = ra + rb - abs(dist);
        if (sobra <= 0.0) { return Sep(false, vec2<f32>(0.0), 0.0, true); }
        if (!melhor.ok || sobra < melhor.sobra) {
            var n = eixo;
            if (dist < 0.0) { n = vec2<f32>(-eixo.x, -eixo.y); }
            melhor = Sep(true, n, sobra, k < 2u);
        }
    }
    return melhor;
}

// `par::referencia`
fn pnp_referencia(ca: vec2<f32>, ma: vec2<f32>, ea: vec2<f32>, cb: vec2<f32>, mb: vec2<f32>, eb: vec2<f32>, normal: vec2<f32>, de_a: bool) -> Ref {
    if (de_a) { return Ref(ca, ma, ea, cb, mb, eb, normal); }
    return Ref(cb, mb, eb, ca, ma, ea, vec2<f32>(-normal.x, -normal.y));
}

// `f32::total_cmp(a, b) == Less` para os valores que um `dot` de eixos unitarios da' (sem NaN):
// so' o par `-0 < +0` difere do `<`.
fn pnp_menor_total(a: f32, b: f32) -> bool {
    if (a < b) { return true; }
    return a == b && bitcast<u32>(a) == 0x80000000u && bitcast<u32>(b) == 0u;
}

// `par::face_incidente` -- o `min_by` fica com o PRIMEIRO minimo.
fn pnp_face_incidente(ci: vec2<f32>, mi: vec2<f32>, ei: vec2<f32>, n: vec2<f32>) -> Seg {
    let vi = pnp_perp(ei);
    var normais = array<vec2<f32>, 4>(ei, vi, vec2<f32>(-ei.x, -ei.y), vec2<f32>(-vi.x, -vi.y));
    var fora = array<f32, 4>(mi.x, mi.y, mi.x, mi.y);
    var lado_m = array<f32, 4>(mi.y, mi.x, mi.y, mi.x);
    var k_min = 0u;
    var d_min = pnp_dot(normais[0], n);
    for (var k = 1u; k < 4u; k = k + 1u) {
        let d = pnp_dot(normais[k], n);
        if (pnp_menor_total(d, d_min)) { k_min = k; d_min = d; }
    }
    let face_n = normais[k_min];
    let lado = pnp_perp(face_n);
    let centro = vec2<f32>(ci.x + face_n.x * fora[k_min], ci.y + face_n.y * fora[k_min]);
    let ml = lado_m[k_min];
    return Seg(true,
        vec2<f32>(centro.x - lado.x * ml, centro.y - lado.y * ml),
        vec2<f32>(centro.x + lado.x * ml, centro.y + lado.y * ml));
}

// `par::ponto_de_contacto`
fn pnp_ponto_de_contacto(r: Ref) -> vec2<f32> {
    let vr = pnp_perp(r.er);
    let f = pnp_face_incidente(r.ci, r.mi, r.ei, r.n_ref);
    let tang = pnp_perp(r.n_ref);
    let meia_tang = r.mr.x * abs(pnp_dot(r.er, tang)) + r.mr.y * abs(pnp_dot(vr, tang));
    let t1 = pnp_dot(vec2<f32>(f.p1.x - r.cr.x, f.p1.y - r.cr.y), tang);
    let t2 = pnp_dot(vec2<f32>(f.p2.x - r.cr.x, f.p2.y - r.cr.y), tang);
    let lo = min(t1, t2);
    let hi = max(t1, t2);
    let clo = max(lo, -meia_tang);
    let chi = min(hi, meia_tang);
    var meio = (lo + hi) * 0.5;
    if (chi >= clo) { meio = (clo + chi) * 0.5; }
    let denom = t2 - t1;
    var t = 0.5;
    if (abs(denom) > PNP_EPS) { t = (meio - t1) / denom; }
    return vec2<f32>(f.p1.x + (f.p2.x - f.p1.x) * t, f.p1.y + (f.p2.y - f.p1.y) * t);
}

// `par::trecho_recortado`
fn pnp_trecho_recortado(r: Ref) -> Seg {
    let f = pnp_face_incidente(r.ci, r.mi, r.ei, r.n_ref);
    let tang = pnp_perp(r.n_ref);
    let vr = pnp_perp(r.er);
    let meia_tang = r.mr.x * abs(pnp_dot(r.er, tang)) + r.mr.y * abs(pnp_dot(vr, tang));
    let t1 = pnp_dot(vec2<f32>(f.p1.x - r.cr.x, f.p1.y - r.cr.y), tang);
    let t2 = pnp_dot(vec2<f32>(f.p2.x - r.cr.x, f.p2.y - r.cr.y), tang);
    let clo = max(min(t1, t2), -meia_tang);
    let chi = min(max(t1, t2), meia_tang);
    if (chi <= clo) { return Seg(false, vec2<f32>(0.0), vec2<f32>(0.0)); }
    let denom = t2 - t1;
    if (abs(denom) <= PNP_EPS) { return Seg(false, vec2<f32>(0.0), vec2<f32>(0.0)); }
    let u1 = (clo - t1) / denom;
    let u2 = (chi - t1) / denom;
    return Seg(true,
        vec2<f32>(f.p1.x + (f.p2.x - f.p1.x) * u1, f.p1.y + (f.p2.y - f.p1.y) * u1),
        vec2<f32>(f.p1.x + (f.p2.x - f.p1.x) * u2, f.p1.y + (f.p2.y - f.p1.y) * u2));
}

fn pnp_unico(c: Cont) -> Man { return Man(1u, c, c); }

// `par::caixas_trecho`
fn pnp_caixas_trecho(ca: vec2<f32>, ma: vec2<f32>, ea: vec2<f32>, cb: vec2<f32>, mb: vec2<f32>, eb: vec2<f32>) -> Man {
    let s = pnp_eixo_separador(ca, ma, ea, cb, mb, eb);
    if (!s.ok) { return Man(0u, pnp_nada(), pnp_nada()); }
    let r = pnp_referencia(ca, ma, ea, cb, mb, eb, s.n, s.de_a);
    let unico = Cont(true, s.n, s.sobra, pnp_ponto_de_contacto(r));
    let q = pnp_trecho_recortado(r);
    if (!q.ok) { return pnp_unico(unico); }
    let vr = pnp_perp(r.er);
    let meia_ref = r.mr.x * abs(pnp_dot(r.er, r.n_ref)) + r.mr.y * abs(pnp_dot(vr, r.n_ref));
    var pontos = array<vec2<f32>, 2>(q.p1, q.p2);
    var out = Man(0u, unico, unico);
    for (var k = 0u; k < 2u; k = k + 1u) {
        let p = pontos[k];
        let d = meia_ref - pnp_dot(vec2<f32>(p.x - r.cr.x, p.y - r.cr.y), r.n_ref);
        if (d > 0.0) {
            let c = Cont(true, s.n, d, p);
            if (out.np == 0u) { out.a = c; } else { out.b = c; }
            out.np = out.np + 1u;
        }
    }
    if (out.np == 0u) { return pnp_unico(unico); }
    return out;
}

// `par::caixas` -- o contacto unico (so' o filtro do repouso o pede).
fn pnp_caixas(ca: vec2<f32>, ma: vec2<f32>, ea: vec2<f32>, cb: vec2<f32>, mb: vec2<f32>, eb: vec2<f32>) -> Cont {
    let s = pnp_eixo_separador(ca, ma, ea, cb, mb, eb);
    if (!s.ok) { return pnp_nada(); }
    let r = pnp_referencia(ca, ma, ea, cb, mb, eb, s.n, s.de_a);
    return Cont(true, s.n, s.sobra, pnp_ponto_de_contacto(r));
}

// `par::disco_caixa`
fn pnp_disco_caixa(cd: vec2<f32>, r: f32, cc: vec2<f32>, meia: vec2<f32>, eixo: vec2<f32>) -> Cont {
    let v = pnp_perp(eixo);
    let d = vec2<f32>(cd.x - cc.x, cd.y - cc.y);
    let lx = pnp_dot(d, eixo);
    let ly = pnp_dot(d, v);
    let hx = meia.x;
    let hy = meia.y;
    let qx = clamp(lx, -hx, hx);
    let qy = clamp(ly, -hy, hy);
    let ex = lx - qx;
    let ey = ly - qy;
    let e2 = ex * ex + ey * ey;
    if (e2 > PNP_EPS) {
        let e = sqrt(e2);
        if (e >= r) { return pnp_nada(); }
        let nl = vec2<f32>(ex / e, ey / e);
        return Cont(true,
            vec2<f32>(nl.x * eixo.x + nl.y * v.x, nl.x * eixo.y + nl.y * v.y),
            r - e,
            vec2<f32>(cc.x + qx * eixo.x + qy * v.x, cc.y + qx * eixo.y + qy * v.y));
    }
    let px = hx - abs(lx);
    let py = hy - abs(ly);
    var nl = vec2<f32>(0.0, 1.0);
    var face = vec2<f32>(lx, hy);
    if (px < py) {
        var s = 1.0;
        if (lx < 0.0) { s = -1.0; }
        nl = vec2<f32>(s, 0.0);
        face = vec2<f32>(s * hx, ly);
    } else {
        var s = 1.0;
        if (ly < 0.0) { s = -1.0; }
        nl = vec2<f32>(0.0, s);
        face = vec2<f32>(lx, s * hy);
    }
    return Cont(true,
        vec2<f32>(nl.x * eixo.x + nl.y * v.x, nl.x * eixo.y + nl.y * v.y),
        min(px, py) + r,
        vec2<f32>(cc.x + face.x * eixo.x + face.y * v.x, cc.y + face.x * eixo.y + face.y * v.y));
}

// `par::contato`
fn pnp_contato(a: Col, pa: vec2<f32>, b: Col, pb: vec2<f32>, eixo_x: bool) -> Cont {
    let ca = pnp_centro(a, pa);
    let cb = pnp_centro(b, pb);
    if (a.kind == 1u && b.kind == 1u) { return pnp_discos(ca, a.m.x, cb, b.m.x, eixo_x); }
    if (a.kind == 2u && b.kind == 2u) { return pnp_caixas(ca, a.m, a.eixo, cb, b.m, b.eixo); }
    if (a.kind == 2u) { return pnp_disco_caixa(cb, b.m.x, ca, a.m, a.eixo); }
    var c = pnp_disco_caixa(ca, a.m.x, cb, b.m, b.eixo);
    c.n = vec2<f32>(-c.n.x, -c.n.y);
    return c;
}

// `par::manifesto`
fn pnp_manifesto(a: Col, pa: vec2<f32>, b: Col, pb: vec2<f32>, eixo_x: bool) -> Man {
    if (a.kind == 2u && b.kind == 2u) {
        return pnp_caixas_trecho(pnp_centro(a, pa), a.m, a.eixo, pnp_centro(b, pb), b.m, b.eixo);
    }
    let c = pnp_contato(a, pa, b, pb, eixo_x);
    if (!c.ok) { return Man(0u, c, c); }
    return pnp_unico(c);
}

// A celula de `p` na grelha de lado `lado`, e o balde dela (o hash do `grid.rs`).
fn pnp_celula(p: vec2<f32>, lado: f32) -> vec2<i32> {
    return vec2<i32>(i32(floor(p.x / lado)), i32(floor(p.y / lado)));
}
fn pnp_balde(c: vec2<i32>, nb: u32) -> u32 {
    return bitcast<u32>((c.x * 73856093) ^ (c.y * 19349663)) & (nb - 1u);
}
