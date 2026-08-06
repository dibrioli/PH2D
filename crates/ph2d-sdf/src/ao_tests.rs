//! Os gates do AO assado.
//!
//! ⚠️ **O oráculo é a FORMA, nunca a fórmula.** Um gate que recomputasse
//! `k·d/t` estaria a afirmar que o código é ele mesmo; estes perguntam o que a
//! geometria obriga — um convexo isolado enxerga o céu, o aro interno de um
//! toro enxerga a parede oposta — e por isso conseguem falhar.

use super::*;
use ph2d_mesh::{Mesh, shapes};

/// O campo pronto de uma malha, do jeito que um bake o construiria.
fn field_of(mesh: &Mesh, res: u32) -> VoxelField {
    let mut f = VoxelField::for_bounds(mesh.bounds(), res);
    f.voxelize(mesh);
    f.flood_fill();
    f
}

fn mean(xs: &[f32]) -> f32 {
    xs.iter().sum::<f32>() / xs.len() as f32
}

// ---------------------------------------------------------------- o controle

/// Uma esfera não tem o que a ocluda, então ela enxerga o céu quase inteiro — e
/// o *quase* é o viés rasante que o módulo nomeia, medido aqui em vez de
/// prometido em prosa.
#[test]
fn um_convexo_isolado_enxerga_o_ceu() {
    let mesh = shapes::uv_sphere(48, 72, 1.0);
    let field = field_of(&mesh, 96);
    let params = AoParams::for_bounds(mesh.bounds());
    let ao = bake_ao(&field, mesh.positions(), mesh.normals(), params);

    let m = mean(&ao);
    println!(
        "esfera: AO medio {m:.4}  min {:.4}",
        ao.iter().copied().fold(f32::INFINITY, f32::min)
    );
    assert!(
        m > 0.90,
        "um convexo isolado tem de enxergar o ceu; medido {m:.4}"
    );
    // O teto é `1` por construção (o `clamp`), e o piso é o viés rasante: se
    // esta metade quebrar, alguem tirou o `clamp` ou o campo saiu do avesso.
    assert!(ao.iter().all(|&v| (0.0..=1.0).contains(&v)));
}

/// ⚠️ **O viés rasante ENCOLHE quando os cones sobem** — a propriedade que o
/// módulo afirma, e a razão de ele ser um viés aceitável em vez de um erro.
/// Um gate que só medisse um ponto de operação não veria a tendência.
#[test]
fn o_vies_rasante_encolhe_com_mais_cones() {
    let mesh = shapes::uv_sphere(32, 48, 1.0);
    let field = field_of(&mesh, 64);
    let base = AoParams::for_bounds(mesh.bounds());

    let poucos = mean(&bake_ao(
        &field,
        mesh.positions(),
        mesh.normals(),
        AoParams { cones: 8, ..base },
    ));
    let muitos = mean(&bake_ao(
        &field,
        mesh.positions(),
        mesh.normals(),
        AoParams { cones: 64, ..base },
    ));
    println!("vies rasante: 8 cones {poucos:.4} -> 64 cones {muitos:.4}");
    assert!(
        muitos > poucos,
        "mais cones tem de aproximar o ceu aberto: 8 -> {poucos:.4}, 64 -> {muitos:.4}"
    );
}

// ------------------------------------------------------- a propriedade do canal

/// O aro **interno** de um toro enxerga a parede oposta através do furo; o
/// **externo** enxerga o céu. É a razão de o canal existir, e é geometria — não
/// há como um bake correto inverter isto.
#[test]
fn o_aro_interno_de_um_toro_ve_menos_ceu_que_o_externo() {
    let mesh = shapes::torus(64, 32, 1.0, 0.5);
    let field = field_of(&mesh, 128);
    // ⚠️ O alcance do default é fração da caixa e **não atravessa o furo**; a
    // fixture tem de CONTER o fenômeno, então aqui ele é aberto o bastante para
    // a parede oposta estar no alcance.
    let params = AoParams {
        radius: 1.0,
        ..AoParams::for_bounds(mesh.bounds())
    };
    let ao = bake_ao(&field, mesh.positions(), mesh.normals(), params);

    // ⚠️ A fixture olha os dois EQUADORES, não duas metades: o tubo inteiro
    // dilui o fenomeno com as laterais, que nao veem nem o furo nem o ceu
    // aberto. Medir na banda larga reporta a banda, nao o aro.
    let (mut dentro, mut fora) = (Vec::new(), Vec::new());
    for (i, p) in mesh.positions().iter().enumerate() {
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        if p[2].abs() > 0.12 {
            continue;
        }
        if r < 0.62 {
            dentro.push(ao[i]);
        } else if r > 1.38 {
            fora.push(ao[i]);
        }
    }
    assert!(
        !dentro.is_empty() && !fora.is_empty(),
        "a fixture perdeu os dois aros"
    );

    let (d, f) = (mean(&dentro), mean(&fora));
    println!("toro: aro interno {d:.4}  aro externo {f:.4}");
    assert!(
        f - d > 0.05,
        "o aro interno tem de enxergar menos ceu que o externo: interno {d:.4}, externo {f:.4}"
    );
}

// --------------------------------------------------- o infinito que vira NaN

/// ⚠️ **RED-FIRST:** o campo carrega `±INFINITY`, e `0.0 * inf` é `NaN`. Um peso
/// da trilinear cai em zero **exatamente sobre as amostras da grade**, que é
/// onde uma marcha de esfera mais pousa — então sem o domador do
/// [`VoxelField::sample`] isto sai `NaN` e contamina o AO inteiro em silêncio.
#[test]
fn a_amostragem_nunca_devolve_nan_nem_infinito() {
    let mesh = shapes::uv_sphere(24, 36, 1.0);
    let field = field_of(&mesh, 48);
    let dims = field.dims();
    let o = field.origin();
    let s = field.step();

    // Varre EXATAMENTE os pontos de amostra (onde as frações são zero) e
    // também o meio das células, mais o lado de fora dos dois lados.
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                for shift in [0.0f32, 0.5] {
                    let p = [
                        o[0] + (i as f32 + shift) * s,
                        o[1] + (j as f32 + shift) * s,
                        o[2] + (k as f32 + shift) * s,
                    ];
                    let v = field.sample(p);
                    assert!(v.is_finite(), "amostra nao-finita em {p:?}: {v}");
                }
            }
        }
    }
    for p in [
        [-1e9f32, 0.0, 0.0],
        [1e9, 0.0, 0.0],
        [f32::NAN, 0.0, 0.0],
        [0.0, f32::INFINITY, 0.0],
    ] {
        assert!(
            field.sample(p).is_finite(),
            "amostra nao-finita fora da grade em {p:?}"
        );
    }
}

/// E o AO herdado disso também é finito — a metade que o consumidor vê.
#[test]
fn o_ao_e_sempre_finito_e_normalizado() {
    let mesh = shapes::torus(32, 16, 1.0, 0.4);
    let field = field_of(&mesh, 64);
    let ao = bake_ao(
        &field,
        mesh.positions(),
        mesh.normals(),
        AoParams::for_bounds(mesh.bounds()),
    );
    assert!(ao.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
    assert_eq!(ao.len(), mesh.vert_count());
}

// ------------------------------------------------------------ as duas bases

/// A base de Duff é ortonormal para TODA normal, inclusive o polo `(0,0,−1)`
/// onde a construção ingênua divide por zero.
#[test]
fn a_base_de_duff_e_ortonormal_ate_no_polo() {
    let normais = [
        [0.0f32, 0.0, 1.0],
        [0.0, 0.0, -1.0], // o polo que mata a versao ingenua
        [1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.577_35, 0.577_35, 0.577_35],
        [-0.577_35, -0.577_35, -0.577_35],
    ];
    for n in normais {
        let (t1, t2) = basis(n);
        let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        for (nome, v) in [("t1", t1), ("t2", t2)] {
            assert!(
                (dot(v, v) - 1.0).abs() < 1e-4,
                "{nome} nao unitario em {n:?}: {:.6}",
                dot(v, v)
            );
            assert!(
                dot(v, n).abs() < 1e-4,
                "{nome} nao perpendicular a normal em {n:?}: {:.6}",
                dot(v, n)
            );
        }
        assert!(
            dot(t1, t2).abs() < 1e-4,
            "t1 e t2 nao sao perpendiculares em {n:?}"
        );
    }
}

/// ⚠️ As direções são distribuídas por **COSSENO**, não uniformemente — e a
/// diferença é observável sem reimplementar a fórmula: a média de `cos θ` é
/// `2/3` no cosseno e `1/2` no uniforme. É isso que faz a média simples dos
/// cones ser o integral de AO em vez de precisar de um peso na soma.
#[test]
fn as_direcoes_sao_distribuidas_por_cosseno() {
    let dirs = cone_directions(4096);
    let media_z = dirs.iter().map(|d| d[2]).sum::<f32>() / dirs.len() as f32;
    println!("media de cos(theta): {media_z:.4} (cosseno = 0,6667; uniforme = 0,5)");
    assert!(
        (media_z - 2.0 / 3.0).abs() < 0.01,
        "a distribuicao nao e por cosseno: media de cos(theta) = {media_z:.4}"
    );
    // Todas no hemisfério certo e unitárias.
    for d in &dirs {
        assert!(d[2] >= 0.0, "direcao no hemisferio errado: {d:?}");
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-3, "direcao nao unitaria: {len:.6}");
    }
}

/// A abertura do cone é função da CONTAGEM: mais cones, cone mais fino.
#[test]
fn a_abertura_do_cone_sai_da_contagem() {
    let k = |cones| {
        AoParams {
            cones,
            ..AoParams::for_bounds(Aabb::default())
        }
        .cone_k()
    };
    let (k8, k32, k128) = (k(8), k(32), k(128));
    println!("k: 8 cones {k8:.3}  32 {k32:.3}  128 {k128:.3}");
    assert!(k8 < k32 && k32 < k128, "k tem de crescer com os cones");
    assert!(
        k8.is_finite() && k(1).is_finite(),
        "k tem de ser finito ate em 1 cone"
    );
}

// -------------------------------------------------------------- determinismo

/// Duas corridas dão os MESMOS bytes. Sem isto o canal não pode ser assado,
/// comparado nem gateado — e é a condição que o ADR-0109 cobra de qualquer
/// coisa que venha a rodar em paralelo depois.
#[test]
fn o_bake_e_deterministico() {
    let mesh = shapes::torus(24, 12, 1.0, 0.4);
    let field = field_of(&mesh, 48);
    let params = AoParams::for_bounds(mesh.bounds());
    let a = bake_ao(&field, mesh.positions(), mesh.normals(), params);
    let b = bake_ao(&field, mesh.positions(), mesh.normals(), params);
    assert_eq!(a.to_vec(), b.to_vec(), "o bake nao e reproduzivel");
}
