//! ⭐⭐ **O PÉ NA SUPERFÍCIE DE REFERÊNCIA** — o ponto mais próximo, e o que olha para o
//! mesmo lado.
//!
//! Irmão de [`crate`] por RESPONSABILIDADE: o laço principal decide **onde** um vértice
//! quer estar; este módulo decide **onde ele pousa**. ⚠️ É público porque o F5 o reusa, e
//! duas implementações de *"o ponto mais próximo da superfície"* divergiriam exactamente
//! onde ninguém olha.

use ph2d_mesh::Mesh;

use crate::dot;

/// **O PONTO MAIS PRÓXIMO da superfície de referência.**
///
/// ⚠️ **É público porque o F5 o reusa**, e duas implementações de *"o ponto mais
/// próximo da superfície"* divergiriam exatamente onde ninguém olha — na parte
/// esparsa do modelo, que é onde o raio de busca decide tudo.
///
/// ⚠️ **O raio de busca DOBRA até achar face.** Um raio fixo devolve zero faces
/// sobre uma parte esparsa do modelo, e a projeção vira um no-op **silencioso** —
/// o Laplaciano então roda sem freio e a peça encolhe.
pub fn project_onto(mesh: &Mesh, p: [f32; 3], seed_radius: f32) -> [f32; 3] {
    project_facing(mesh, p, seed_radius, None)
}

/// **O PONTO MAIS PRÓXIMO QUE OLHA PARA O MESMO LADO.**
///
/// ⭐⭐ **Dentro de um vinco CÔNCAVO o ponto mais próximo pode estar do outro lado
/// da dobra**, e é aí que a malha rasga. A razão é geométrica e não numérica: o
/// eixo medial de uma concavidade **encosta na superfície**, então dois pontos a
/// milímetros um do outro têm pés opostos. Um deles atravessa a dobra, e a face
/// entre os dois vira uma lasca — a fenda que a foto de 2026-08-22 mostra colada à
/// borda da orelha.
///
/// ⚠️ **A cura clássica não é um raio menor: é uma DIREÇÃO.** Com `facing =
/// Some(n)`, uma face candidata só entra se a normal dela concordar com `n`. Sem
/// candidato nenhum que concorde, ela **cai para o mais próximo** — porque uma
/// projeção que falha é pior que uma que erra de lado: o Laplaciano roda sem freio
/// e a peça encolhe.
///
/// ⚠️ **O limiar é `0`, e não um cosseno afinado.** *Do mesmo lado* é uma pergunta
/// de sinal; qualquer número acima disso seria uma segunda escolha a defender, e a
/// superfície de referência é um poliedro cujas normais saltam de faceta em faceta.
#[must_use]
pub fn project_facing(
    mesh: &Mesh,
    p: [f32; 3],
    seed_radius: f32,
    facing: Option<[f32; 3]>,
) -> [f32; 3] {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    let diag = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    let mut radius = seed_radius.max(1.0e-6);
    let mut hits: Vec<u32> = Vec::new();
    loop {
        hits.clear();
        mesh.octree().faces_in_sphere(p, radius, &mut hits);
        if !hits.is_empty() || radius > diag {
            break;
        }
        radius *= 2.0;
    }
    if hits.is_empty() {
        return p;
    }
    let (verts, faces) = (mesh.positions(), mesh.faces());
    let normals = mesh.face_normals();
    let (mut best, mut best_p) = (f32::INFINITY, p);
    // ⚠️ **O de RECURSO é acumulado na mesma passagem**, e não numa segunda: uma
    // segunda varredura pagaria a consulta ao octree outra vez, e o caminho sem
    // candidato concordante é justamente o caro.
    let (mut any, mut any_p) = (f32::INFINITY, p);
    for &f in &hits {
        let v = faces[f as usize].verts();
        let agrees = facing.is_none_or(|n| {
            normals
                .get(f as usize)
                .is_none_or(|m| n[0].mul_add(m[0], n[1].mul_add(m[1], n[2] * m[2])) > 0.0)
        });
        for k in 1..v.len() - 1 {
            let q = closest_on_triangle(
                p,
                verts[v[0] as usize],
                verts[v[k] as usize],
                verts[v[k + 1] as usize],
            );
            let d = sub(q, p);
            let dist = dot(d, d);
            if dist < any {
                any = dist;
                any_p = q;
            }
            if agrees && dist < best {
                best = dist;
                best_p = q;
            }
        }
    }
    if best.is_finite() { best_p } else { any_p }
}

/// O ponto do triângulo mais próximo de `p` — as sete regiões de Voronoi.
fn closest_on_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1.mul_add(d4, -(d3 * d2));
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return axpy(v, ab, a);
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5.mul_add(d2, -(d1 * d6));
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return axpy(w, ac, a);
    }
    let va = d3.mul_add(d6, -(d5 * d4));
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return axpy(w, sub(c, b), b);
    }
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    [
        w.mul_add(ac[0], v.mul_add(ab[0], a[0])),
        w.mul_add(ac[1], v.mul_add(ab[1], a[1])),
        w.mul_add(ac[2], v.mul_add(ab[2], a[2])),
    ]
}

fn axpy(t: f32, d: [f32; 3], o: [f32; 3]) -> [f32; 3] {
    [
        t.mul_add(d[0], o[0]),
        t.mul_add(d[1], o[1]),
        t.mul_add(d[2], o[2]),
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
