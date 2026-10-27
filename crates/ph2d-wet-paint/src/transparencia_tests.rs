//! Gates da **Transparency** (dono 27/10, `1 − Alpha` do Principled; `oraculo/README.md` §Transparency): o neutro
//! ao bit, o relevo intacto, a cobertura de composição, a sombra que a tinta transparente projecta e a mistura
//! pela massa. O grão e a cor do papel na tela final: `ph2d-tool-painter`
//! `a_transparencia_deixa_ver_o_grao_e_a_cor_do_papel`.

use crate::grid::Grid;
use crate::material::MaterialCelula;
use crate::oleo_real::{CampoAltura, OleoReal, sombra_transparente};
use crate::render::{PigmentVisual, RenderLayer, render_pigment_row_oleo_com_sombra};

const W: usize = 48;

/// Uma pilha com relevo (estrias) no miolo, papel em volta.
fn tinta() -> Grid {
    let mut g = Grid::new(W, W);
    for y in 12..=36 {
        for x in 12..=36 {
            let i = x + y * g.s;
            g.sett[i] = 3000.0 + 1500.0 * (x as f32 * 0.9).sin();
            g.sett_rgb[i] = [40.0, 70.0, 200.0];
        }
    }
    g
}

fn oleo(t: f64) -> OleoReal {
    let mut o = OleoReal::default();
    o.material.transparencia = t;
    o
}

/// O RGBA e a sombra no papel da grade inteira.
fn render(g: &Grid, o: &OleoReal) -> (Vec<u8>, Vec<u8>) {
    let camadas = [RenderLayer {
        grid: g,
        opacity: 1.0,
        visible: true,
    }];
    let c = CampoAltura::de(&[(g, true)], o, 1, 1, 1, W, W);
    let (mut px, mut sombra) = (vec![0u8; W * W * 4], vec![0u8; W * W]);
    let mut feita = vec![false; W * W];
    for cy in 1..=W {
        render_pigment_row_oleo_com_sombra(
            None,
            &camadas,
            PigmentVisual::default(),
            o,
            &c,
            cy,
            1,
            W,
            &mut px[(cy - 1) * W * 4..cy * W * 4],
            &mut sombra[(cy - 1) * W..cy * W],
            &mut feita[(cy - 1) * W..cy * W],
        );
    }
    (px, sombra)
}

/// T 0 é a imagem de antes AO BIT: o campo do material alocado a NEUTRO (o ramo da célula) e o cartão a 0 dão os
/// mesmos bytes que a grade sem campo.
#[test]
fn a_transparencia_zero_e_a_imagem_de_antes() {
    let g = tinta();
    let mut com_campo = g.clone();
    crate::grid::material::garante(&mut com_campo);
    assert_eq!(render(&g, &oleo(0.0)), render(&com_campo, &oleo(0.0)));
}

/// ⭐ A Transparency só muda a cobertura de COMPOSIÇÃO: o relevo (as três alturas) fica ao bit; T 1 = alfa 0 e
/// nenhuma sombra no papel; T 0,5 = metade do alfa; a da CÉLULA (o pincel) vale o mesmo que a do cartão.
#[test]
fn a_transparencia_deixa_ver_sem_mexer_no_relevo() {
    let g = tinta();
    let (c0, c1) = (
        CampoAltura::de(&[(&g, true)], &oleo(0.0), 1, 1, 1, W, W),
        CampoAltura::de(&[(&g, true)], &oleo(1.0), 1, 1, 1, W, W),
    );
    for y in 1..=W as isize {
        for x in 1..=W as isize {
            assert_eq!(
                c0.at(x, y).to_bits(),
                c1.at(x, y).to_bits(),
                "forma ({x},{y})"
            );
            assert_eq!(c0.at_fina(x, y).to_bits(), c1.at_fina(x, y).to_bits());
            assert_eq!(c0.at_sombra(x, y).to_bits(), c1.at_sombra(x, y).to_bits());
        }
    }
    let (p0, s0) = render(&g, &oleo(0.0));
    let (pm, _) = render(&g, &oleo(0.5));
    let (p1, s1) = render(&g, &oleo(1.0));
    assert!(s0.iter().any(|&s| s > 0), "a tinta opaca projecta sombra");
    assert!(p1.chunks(4).all(|p| p[3] == 0), "T 1: a tinta não se vê");
    assert!(
        s1.iter().all(|&s| s == 0),
        "T 1: a tinta não projecta sombra"
    );
    let a = |p: &[u8]| p.chunks(4).map(|q| f64::from(q[3])).sum::<f64>();
    let r = a(&pm) / a(&p0);
    assert!((0.49..=0.51).contains(&r), "T 0,5 = metade do alfa: {r}");
    // A Transparency do PINCEL (por célula) com o cartão a 0.
    let mut cel = g.clone();
    for m in crate::grid::material::garante(&mut cel) {
        m.transparencia = 1.0;
    }
    assert_eq!(render(&cel, &oleo(0.0)), render(&g, &oleo(1.0)));
}

/// A lei da sombra sob a tinta transparente (`sombra_transparente`): o neutro ao bit, o Alpha 0 sem sombra, o papel
/// nu com a sombra que atravessa e o papel SOB a tinta com a luz que a atravessa.
#[test]
fn a_sombra_da_tinta_transparente_e_a_luz_que_a_atravessa() {
    for s in [0.0, 0.3, 0.8, 1.0] {
        for a in [0.0, 0.4, 1.0] {
            assert_eq!(sombra_transparente(s, a, 0.0).to_bits(), f64::to_bits(s));
            assert_eq!(sombra_transparente(s, a, 1.0), 0.0, "s {s} a {a}");
        }
        assert!((sombra_transparente(s, 0.0, 0.5) - s * 0.75).abs() < 1e-12);
        let sob = 1.0 - 0.5 * (1.0 - s * 0.75);
        assert!((sombra_transparente(s, 1.0, 0.5) - sob).abs() < 1e-12);
    }
}

/// A Transparency mistura-se pela MASSA como o resto do material (`grid::material::pousa`), e a dos dois lados
/// combina-se por `max` (a regra do Metallic).
#[test]
fn a_transparencia_mistura_pela_massa_e_combina_por_max() {
    let mut g = Grid::new(16, 12);
    let i = 40;
    let opaca = MaterialCelula::NEUTRO;
    let vidro = MaterialCelula {
        transparencia: 1.0,
        ..MaterialCelula::NEUTRO
    };
    crate::grid::material::pousa(&mut g, i, 0.0, opaca, 100.0, false);
    crate::grid::material::garante(&mut g);
    crate::grid::material::pousa(&mut g, i, 100.0, vidro, 300.0, false);
    assert_eq!(g.material_da_celula(i).transparencia, 0.75);
    let k = crate::oleo_real::MaterialCartao {
        transparencia: 0.4,
        ..crate::oleo_real::MaterialCartao::NEUTRO
    };
    assert_eq!(crate::material::efectivo(&vidro, &k).transparencia, 1.0);
    assert_eq!(crate::material::efectivo(&opaca, &k).transparencia, 0.4);
}
