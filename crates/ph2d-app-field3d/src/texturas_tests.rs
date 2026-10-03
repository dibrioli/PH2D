//! A textura de cada folha, a matriz mundo → folha, os ficheiros e a semente da cena.

use super::*;
use crate::materials::colour_row_tests::three_balls;

fn fixtura(n: &str) -> String {
    format!("{}/../ph2d-triplanar/fixtures/{n}", env!("CARGO_MANIFEST_DIR"))
}

/// ⭐ **A matriz leva o mundo de volta à folha** (giro, escala e deslocamento).
#[test]
fn a_matriz_leva_o_mundo_a_folha() {
    let (s, c) = 0.4f32.sin_cos();
    let x = ph2d_field::Xform {
        translation: [0.3, -0.2, 1.1],
        rotation: [0.0, s, 0.0, c],
        scale: 1.7,
    };
    let m = mundo_para_folha(x);
    for p in [[0.1, 0.2, 0.3], [-1.0, 0.5, 2.0]] {
        let w = x.apply(p);
        let f: [f32; 3] =
            std::array::from_fn(|i| m[i][0] * w[0] + m[i][1] * w[1] + m[i][2] * w[2] + m[i][3]);
        for i in 0..3 {
            assert!((f[i] - p[i]).abs() < 1e-5, "{p:?} → {f:?}");
        }
    }
}

/// ⭐ **A ordem é a dos materiais**, e a fonte resolve-se.
#[test]
fn a_ordem_e_a_dos_materiais() {
    let (mut sim, grupo, folhas) = three_balls();
    sim.world_mut().entity_mut(folhas[1]).insert(FieldTexture {
        source: 3,
        ..FieldTexture::default()
    });
    let v = das_folhas(sim.world(), grupo);
    assert_eq!(v.len(), crate::materials::folhas(sim.world(), grupo).len());
    assert!(v[0].is_none() && v[2].is_none());
    let f = v[1].as_ref().expect("a do meio");
    assert_eq!(f.fonte, Fonte::Embarcada(2));
    assert!((f.tile - 1.8).abs() < 1e-6, "o ladrilho 0 é o tamanho real da pedra");
    let sem_cor = FieldTexture {
        source: ph2d_field::TEXTURE_FROM_FILE,
        ..FieldTexture::default()
    };
    assert_eq!(fonte_de(&sem_cor), None, "de ficheiro sem a cor não é textura");
}

/// ⭐ **Um ficheiro importado vira mapas `1024²`**; um que não existe diz porquê.
#[test]
fn os_ficheiros_viram_mapas_ou_dizem_porque() {
    let (m, aspecto) = carrega(&Fonte::Ficheiros {
        cor: fixtura("teste_colorida.png"),
        normal: fixtura("teste_normal.png"),
        rugosidade: String::new(),
    })
    .expect("os PNG de teste");
    assert_eq!(m.cor.lado(), ph2d_triplanar::LADO);
    assert!(m.tem_normal && !m.tem_rugosidade);
    assert!((aspecto - 1.0).abs() < 1e-6);
    let erro = carrega(&Fonte::Ficheiros {
        cor: "/nao/existe.png".into(),
        normal: String::new(),
        rugosidade: String::new(),
    });
    assert!(erro.is_err(), "um ficheiro que sumiu não pode virar textura");
}

/// ⭐ **A semente da cena planta-se uma vez e gasta-se.**
#[test]
fn a_semente_planta_e_gasta() {
    let (mut sim, grupo, folhas) = three_balls();
    semeia(Some(vec![
        None,
        Some(FieldTexture {
            source: 1,
            ..FieldTexture::default()
        }),
    ]));
    planta(sim.world_mut(), grupo);
    assert!(sim.world().get::<FieldTexture>(folhas[0]).is_none());
    assert_eq!(sim.world().get::<FieldTexture>(folhas[1]).map(|t| t.source), Some(1));
    sim.world_mut().entity_mut(folhas[1]).remove::<FieldTexture>();
    planta(sim.world_mut(), grupo);
    assert!(
        sim.world().get::<FieldTexture>(folhas[1]).is_none(),
        "a semente replantou por cima do artista"
    );
}
