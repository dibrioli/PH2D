//! ⭐⭐⭐ **O GATE Nº1 DA ESPEC — o resíduo da costura, medido ONDE ELE MACHUCA.**
//!
//! ([`SPEC_restricoes_por_eliminacao.md` §5.1](../../../docs/3D/cleanroom/SPEC_restricoes_por_eliminacao.md))
//!
//! # ⛔⛔ As duas armadilhas que este ficheiro existe para não cair
//!
//! **1 — tautologia.** Depois de a costura ser eliminada, comparar a cópia derivada com
//! a fórmula que a derivou mede a fórmula contra si própria: fica verde para sempre,
//! sobre qualquer implementação, incluindo uma errada. ⇒ ⭐ **este gate mede à SAÍDA**,
//! pelo caminho que a extracção de facto percorre: o mapa por **canto**, com as
//! transições **re-derivadas dali** ([`ExtractReport::shift_residual`]).
//!
//! ⚠️ *E os dois sítios não são o mesmo, com prova:* na mesma corrida o arredondamento
//! declara a translação exactamente inteira (`0`) enquanto a extracção, re-derivando do
//! mapa por canto, mede até **`0,46`** de célula.
//!
//! **2 — «zero» contra «uma tolerância».** A lei promete zero **por construção**, e o
//! que se mede não é folga: é o erro de **avaliação** da substituição em vírgula
//! flutuante. ⇒ ⛔ nem `== 0.0`, nem uma tolerância de conforto: a barra **lê-se da
//! referência, pelo mesmo verificador**.
//!
//! # ⚠️⚠️ A PERGUNTA QUE ESTE GATE DEVOLVE (e a assunção sob que ele corre)
//!
//! Os mapas de referência são **`f64`** e fecham a `~3,5e-15`; o nosso [`ph2d_gridmap`]
//! guarda o mapa em **`f32`**, cujo chão é `|z|·2⁻²³ ≈ 1e-6`. ⇒ **a barra da referência
//! é inalcançável para nós por representação, não por algoritmo** — e a emenda que a
//! tornou «lida em vez de literal» não alcança essa metade.
//!
//! ⭐ **A forma que este gate usa, e que é a mesma lei sem a dependência da
//! representação:** cada mapa é comparado com o **chão da precisão em que foi
//! calculado**, e afirma-se que o nosso está tão perto do nosso chão quanto a referência
//! do dela. *Se o E preferir outra forma, é uma emenda — o número está medido aqui.*

mod support;

use ph2d_quadextract::{ExtractReport, extract};

/// O chão de representação de um mapa: `|z|·ε`.
fn floor_of(biggest: f64, eps: f64) -> f64 {
    // ⭐ `8` porque a substituição soma um punhado de termos; é o mesmo factor nos dois
    // lados da comparação, logo não escolhe o veredito.
    8.0 * biggest.max(1.0) * eps
}

/// A extracção de um `Mapa`, e o maior `|z|` que ele contém.
fn measure(m: &ph2d_quadextract::mapa::Mapa) -> (ExtractReport, f64) {
    let cm = ph2d_quadextract::CornerMap {
        pos: &m.pos,
        tris: &m.tris,
        uv: &m.uv,
    };
    let biggest =
        m.uv.iter()
            .flatten()
            .fold(0.0f64, |a, z| a.max(z[0].abs()).max(z[1].abs()));
    let (_, rep) = extract(&cm, None).expect("a referência tem de extrair");
    (rep, biggest)
}

/// ⭐⭐⭐ **A REFERÊNCIA FECHA AS COSTURAS DELA ao chão de `f64`** — é daqui que a barra
/// sai, medida pelo NOSSO verificador e não copiada de um literal.
#[test]
fn the_reference_maps_close_their_seams_at_the_floor_of_their_own_precision() {
    for (name, m) in [("gancho", support::hooked()), ("toro", support::torus())] {
        let (rep, biggest) = measure(&m);
        let floor = floor_of(biggest, f64::EPSILON);
        assert!(
            rep.interior_edges > 5_000,
            "{name}: a fixtura tem de conter costuras a sério — {} arestas interiores",
            rep.interior_edges
        );
        assert!(
            rep.shift_residual <= floor,
            "{name}: a referência mede {:.3e} de resíduo de translação, e o chão de \
             `f64` para |z| = {biggest:.1} é {floor:.3e} — se ela não está no chão dela, \
             a barra deste ficheiro não descreve o que diz descrever",
            rep.shift_residual
        );
        eprintln!(
            "{name}: referência {:.3e} | chão de f64 {:.3e} | razão {:.3}",
            rep.shift_residual,
            floor,
            rep.shift_residual / floor
        );
    }
}

/// ⭐⭐⭐ **E O NOSSO MAPA SOLDADO FECHA AS DELE ao chão de `f32`** — a mesma lei, a
/// mesma régua, a precisão que a nossa representação permite.
///
/// ⛔ **Prove-se por mutação:** com o caminho **penalizado** (a costura como termo de
/// energia) este mesmo número mede `0,46` de célula — cinco ordens de grandeza acima.
#[test]
#[ignore = "lento -- corre a cadeia inteira da casa"]
fn our_welded_map_closes_its_seams_at_the_floor_of_f32() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(&mesh, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = ph2d_gridmap::cut_along_patches(&mesh, &layout);
    let (combed, _) = ph2d_gridmap::comb_patches(&mesh, &layout, &cut);
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    let h = e[e.len() / 2];

    let (map, _) = ph2d_gridmap::round_welded(&mesh,
        &cut,
        &combed,
        ph2d_gridmap::Step::uniform(h),
        ph2d_gridmap::RoundOptions::default(),
        &singular,
    );
    let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
    let cm = ph2d_quadextract::CornerMap {
        pos: mesh.positions(),
        tris: &tris,
        uv: &uv,
    };
    let (_, rep) = extract(&cm, None).expect("a nossa cadeia tem de extrair");
    let biggest = uv
        .iter()
        .flatten()
        .fold(0.0f64, |a, z| a.max(z[0].abs()).max(z[1].abs()));
    // ⚠️ `f32::EPSILON` e não `f64::EPSILON`: o mapa foi CALCULADO em `f32`, e o
    // `corner_map` só o promove na saída. *Medir contra o chão de uma precisão que o
    // cálculo nunca teve seria uma barra sobre outra coisa.*
    let floor = floor_of(biggest, f64::from(f32::EPSILON));
    eprintln!(
        "nosso soldado: {:.3e} | chão de f32 {:.3e} | razão {:.3} | {} arestas interiores",
        rep.shift_residual,
        floor,
        rep.shift_residual / floor,
        rep.interior_edges
    );
    assert!(
        rep.interior_edges > 3_000,
        "a fixtura tem de conter costuras: {} arestas interiores",
        rep.interior_edges
    );
    assert!(
        rep.shift_residual <= floor,
        "o nosso resíduo de translação, medido no mapa por CANTO, foi {:.3e}, e o chão \
         de `f32` para |z| = {biggest:.1} é {floor:.3e}",
        rep.shift_residual
    );
}
