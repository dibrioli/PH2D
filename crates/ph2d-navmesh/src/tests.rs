//! Os gates da construção (os do oráculo do Godot e a varredura contra o oráculo exacto vivem em
//! `tests/it/`).

use ph2d_nav::Polyanya;

use crate::{Corner, Params, Shape, build};

fn caixa(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
}

#[test]
fn a_esquadria_da_a_malha_do_godot_na_cena_da_sonda() {
    // A cena da sonda da pesquisa: 400×300, um quadrado (150,100)–(250,200), raio 10. O Godot assou
    // 4 polígonos com o furo em (140,90)–(260,210) e o contorno em (10,10)–(390,290).
    let p = Params {
        agent_radius: 10.0,
        corner: Corner::Miter,
        ..Params::default()
    };
    let b = build(
        &caixa(0.0, 0.0, 400.0, 300.0),
        &[Shape::Convex(caixa(150.0, 100.0, 250.0, 200.0))],
        &p,
    )
    .expect("constrói");
    let area = 380.0 * 280.0 - 120.0 * 120.0;
    assert!(
        (b.mesh.area() - area).abs() < 1e-6,
        "{} contra {area}",
        b.mesh.area()
    );
    for &v in &[
        [140.0, 90.0],
        [260.0, 90.0],
        [260.0, 210.0],
        [140.0, 210.0],
        [10.0, 10.0],
        [390.0, 290.0],
    ] {
        assert!(
            b.mesh
                .verts()
                .iter()
                .any(|&q| (q[0] - v[0]).abs() < 1e-9 && (q[1] - v[1]).abs() < 1e-9),
            "o vértice {v:?} do Godot não está na malha"
        );
    }
}

#[test]
fn o_recuo_redondo_anda_mais_e_o_caminho_e_mais_curto() {
    let reg = caixa(0.0, 0.0, 400.0, 300.0);
    let obs = [Shape::Convex(caixa(150.0, 100.0, 250.0, 200.0))];
    let miter = build(
        &reg,
        &obs,
        &Params {
            agent_radius: 10.0,
            corner: Corner::Miter,
            ..Params::default()
        },
    )
    .expect("constrói");
    let round = build(
        &reg,
        &obs,
        &Params {
            agent_radius: 10.0,
            ..Params::default()
        },
    )
    .expect("constrói");
    assert!(
        round.mesh.area() > miter.mesh.area() + 1.0,
        "{} vs {}",
        round.mesh.area(),
        miter.mesh.area()
    );
    let mut s = Polyanya::new();
    let a = s
        .find_path(&miter.mesh, [50.0, 150.0], [350.0, 160.0])
        .expect("caminho")
        .length;
    let b = s
        .find_path(&round.mesh, [50.0, 150.0], [350.0, 160.0])
        .expect("caminho")
        .length;
    assert!(b < a - 0.5, "redondo {b} contra esquadria {a}");
}

#[test]
fn uma_passagem_mais_estreita_que_o_diametro_fecha() {
    // Duas paredes com um vão de 1,0 m: um agente de raio 0,4 passa, um de raio 0,6 não.
    let reg = caixa(0.0, 0.0, 10.0, 10.0);
    let obs = [
        Shape::Convex(caixa(4.0, 0.0, 5.0, 4.5)),
        Shape::Convex(caixa(4.0, 5.5, 5.0, 10.0)),
    ];
    let mut s = Polyanya::new();
    let pequeno = build(
        &reg,
        &obs,
        &Params {
            agent_radius: 0.4,
            ..Params::default()
        },
    )
    .expect("constrói");
    let grande = build(
        &reg,
        &obs,
        &Params {
            agent_radius: 0.6,
            ..Params::default()
        },
    )
    .expect("constrói");
    assert_eq!(pequeno.mesh.island_count(), 1);
    assert!(s.find_path(&pequeno.mesh, [1.0, 5.0], [9.0, 5.0]).is_ok());
    assert_eq!(
        grande.mesh.island_count(),
        2,
        "o vão fecha e parte a região em duas"
    );
    assert_eq!(
        s.find_path(&grande.mesh, [1.0, 5.0], [9.0, 5.0]),
        Err(ph2d_nav::NoPath::Unreachable)
    );
}

#[test]
fn obstaculos_sobrepostos_e_que_se_tocam_fundem() {
    let reg = caixa(0.0, 0.0, 20.0, 20.0);
    let obs = [
        Shape::Convex(caixa(5.0, 5.0, 10.0, 10.0)),
        Shape::Convex(caixa(8.0, 8.0, 13.0, 13.0)),
        Shape::Convex(caixa(13.0, 5.0, 15.0, 13.0)),
        Shape::Circle {
            center: [3.0, 15.0],
            radius: 1.0,
        },
        Shape::Capsule {
            a: [15.0, 17.0],
            b: [18.0, 17.0],
            radius: 0.5,
        },
    ];
    let b = build(
        &reg,
        &obs,
        &Params {
            agent_radius: 0.5,
            ..Params::default()
        },
    )
    .expect("constrói");
    assert_eq!(b.mesh.island_count(), 1);
    assert!(b.mesh.area() > 0.0);
}

#[test]
fn a_regiao_toda_tapada_da_uma_malha_vazia_e_nao_um_erro() {
    let reg = caixa(0.0, 0.0, 1.0, 1.0);
    let b = build(
        &reg,
        &[Shape::Convex(caixa(-1.0, -1.0, 2.0, 2.0))],
        &Params::default(),
    )
    .expect("constrói");
    assert_eq!(b.mesh.polys().len(), 0);
    let b = build(
        &reg,
        &[],
        &Params {
            agent_radius: 0.6,
            ..Params::default()
        },
    )
    .expect("constrói");
    assert_eq!(
        b.mesh.polys().len(),
        0,
        "a região mais estreita que o diâmetro some"
    );
}
