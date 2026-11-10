//! Gates do campo [`super::OleoCelula`] (dono 28/10: Depth, Gloss e a caixa Real Oil são do TRAÇO) e a MEDIDA da
//! parede entre traços de Depth diferentes / com e sem óleo (doc `W3b_painel_so_no_futuro.md` §Depth, Gloss e Real
//! Oil por traço).

use super::*;
use crate::oleo_real::{CampoAltura, OleoReal};

const W: usize = 120;
const H: usize = 40;

/// Duas faixas de tinta lado a lado (x `20..=59` e `60..=99`, y `10..=30`) com o mesmo pigmento e o óleo `a` / `b`.
fn duas(a: OleoCelula, b: OleoCelula) -> Grid {
    let mut g = Grid::new(W, H);
    g.oleo = CampoEsparso::novo(OleoCelula::NEUTRA, g.cells);
    for y in 10..=30 {
        for x in 20..=99 {
            let i = x + y * g.s;
            g.sett[i] = 6000.0;
            g.oleo[i] = if x < 60 { a } else { b };
        }
    }
    g
}

/// A forma `v` (a altura que a normal e a cor lêem) ao longo da linha do meio, e a mesma com o Depth aplicado DEPOIS
/// do desfoque (a alternativa recusada): `(antes, depois)`.
fn perfis(a: OleoCelula, b: OleoCelula) -> (Vec<f64>, Vec<f64>) {
    let o = OleoReal::default();
    let g = duas(a, b);
    let antes = CampoAltura::de(&[(&g, true)], &o, 1, 1, 1, W, H);
    let mut neutra = g.clone();
    let mut v = neutra.oleo.para_vec();
    for c in &mut v {
        if c.oleo > 0.0 {
            *c = OleoCelula {
                oleo: 1.0,
                profundidade: 1.0,
                brilho: c.brilho,
                soluvel: 0.0,
                meio: 0.0,
                corpo: 0.0,
                pincel_de_oleo: true,
            };
        }
    }
    neutra.oleo = campo_de(&v);
    let liso = CampoAltura::de(&[(&neutra, true)], &o, 1, 1, 1, W, H);
    let y = 20isize;
    let a_ = (1..=W as isize).map(|x| antes.at(x, y)).collect();
    let d_ = (1..=W as isize)
        .map(|x| liso.at(x, y) * ganho(&g.oleo, x as usize + y as usize * g.s))
        .collect();
    (a_, d_)
}

/// O maior salto entre células vizinhas na fronteira (x `50..=70`) e se o perfil é MONÓTONO ali (sem ressalto).
fn parede(v: &[f64]) -> (f64, bool) {
    let z = &v[49..70];
    let salto = z
        .windows(2)
        .map(|p| (p[1] - p[0]).abs())
        .fold(0.0, f64::max);
    let sobe = z.windows(2).all(|p| p[1] >= p[0] - 1e-9);
    let desce = z.windows(2).all(|p| p[1] <= p[0] + 1e-9);
    (salto, sobe || desce)
}

/// ⭐ **A parede entre traços de Depth diferentes** (e entre tinta com e sem óleo): o Depth escala a massa ANTES do
/// desfoque — a fronteira é a rampa do desfoque, monótona (sem ressalto) e sem degrau; o Depth aplicado DEPOIS do
/// desfoque faz um degrau de uma célula do tamanho da diferença (medido e impresso).
#[test]
fn a_parede_entre_traços_de_depth_diferentes_e_uma_rampa_sem_degrau() {
    let um = OleoCelula {
        oleo: 1.0,
        profundidade: 1.0,
        brilho: 1.0,
        soluvel: 0.0,
        meio: 0.0,
        corpo: 0.0,
        pincel_de_oleo: true,
    };
    for (nome, b) in [
        (
            "Depth 1 | 3",
            OleoCelula {
                profundidade: 3.0,
                ..um
            },
        ),
        ("óleo | sem óleo", OleoCelula::NEUTRA),
    ] {
        let (antes, depois) = perfis(um, b);
        let (sa, ma) = parede(&antes);
        let (sd, _) = parede(&depois);
        eprintln!(
            "PAREDE {nome}: antes do desfoque salto {sa:.3} px monótona {ma} · depois do desfoque salto {sd:.3} px"
        );
        assert!(ma, "{nome}: a parede tem ressalto");
        assert!(
            sa * 3.0 < sd,
            "{nome}: a rampa ({sa}) não é mais suave que o degrau ({sd})"
        );
    }
}

/// O neutro: a tinta com o Real Oil e o Depth/Gloss de fábrica dá o MESMO campo, ao bit, que a chamada de antes
/// (sem o campo: toda a tinta é óleo de Depth 1).
#[test]
fn o_oleo_de_fabrica_e_o_campo_de_antes_ao_bit() {
    let um = OleoCelula {
        oleo: 1.0,
        profundidade: 1.0,
        brilho: 1.0,
        soluvel: 0.0,
        meio: 0.0,
        corpo: 0.0,
        pincel_de_oleo: true,
    };
    let g = duas(um, um);
    let mut antes = g.clone();
    antes.oleo = sem_campo();
    let o = OleoReal::default();
    let c = CampoAltura::de(&[(&g, true)], &o, 1, 1, 1, W, H);
    let d = CampoAltura::de(&[(&antes, true)], &o, 1, 1, 1, W, H);
    for y in 1..=H as isize {
        for x in 1..=W as isize {
            for (a, b) in [
                (c.at(x, y), d.at(x, y)),
                (c.at_fina(x, y), d.at_fina(x, y)),
                (c.at_sombra(x, y), d.at_sombra(x, y)),
            ] {
                assert_eq!(a.to_bits(), b.to_bits(), "({x}, {y})");
            }
            if g.sett[x as usize + y as usize * g.s] > 0.0 {
                assert_eq!(
                    (
                        c.brilho_em(x as usize, y as usize),
                        c.oleo_em(x as usize, y as usize)
                    ),
                    (1.0, 1.0)
                );
            }
        }
    }
}

/// O pouso: a NEUTRA (sem óleo) não aloca; a tinta com óleo aloca e mistura-se pela massa; a orla virgem veste-se;
/// o desfazer que deixa a grade sem óleo LARGA o campo; o custo é 16 B por célula (o `soluvel` da J10, 84df135c02).
#[test]
fn o_campo_nasce_com_o_oleo_mistura_se_e_larga_se_sem_ele() {
    assert_eq!(std::mem::size_of::<OleoCelula>(), 28); // + meio e corpo (E10 A2) + pincel_de_oleo (E10e1, 1 B + 3 de alinhamento)
    let mut g = Grid::new(32, 32);
    let i = 10 + 10 * g.s;
    pousa(&mut g, i, 0.0, OleoCelula::NEUTRA, (5.0, 0.0));
    assert!(g.oleo.is_empty(), "a tinta sem óleo alocou o campo");
    let fundo = OleoCelula {
        oleo: 1.0,
        profundidade: 3.0,
        brilho: 0.0,
        soluvel: 0.0,
        meio: 0.0,
        corpo: 0.0,
        pincel_de_oleo: true,
    };
    pousa(&mut g, i, 0.0, fundo, (5.0, 0.0));
    g.sett[i] = 5.0;
    assert_eq!(da_celula(&g.oleo, i), fundo);
    pousa(&mut g, i, 5.0, OleoCelula::NEUTRA, (5.0, 0.0));
    let m = da_celula(&g.oleo, i);
    assert!(
        (m.oleo - 0.5).abs() < 1e-6 && (m.profundidade - 2.0).abs() < 1e-6,
        "{m:?}"
    );
    assert!(
        !m.pincel_de_oleo,
        "o Real Oil da célula é o da última mão (sobrescreve, E10e1)"
    );
    veste_virgens(&mut g, (10, 10, 10, 10), fundo);
    assert_eq!(da_celula(&g.oleo, i + 1), fundo, "a orla virgem");
    assert_eq!(da_celula(&g.oleo, i), m, "a célula pintada não se veste");
    restaura_janela(&mut g, (1, 1, 32, 32), &[]);
    assert!(g.oleo.is_empty(), "o desfazer sem óleo não largou o campo");
}

/// B12.3: o 1.º dab com o Real Oil (raio 30) aloca os pedaços das linhas dele, não a tela (era `plano_cheio`).
#[test]
fn o_primeiro_dab_com_oleo_aloca_o_campo_das_linhas_dele() {
    let mut g = Grid::new(3840, 600);
    let nova = OleoCelula {
        oleo: 1.0,
        pincel_de_oleo: true,
        ..OleoCelula::NEUTRA
    };
    let (cx, cy, r) = (1900i32, 300i32, 30i32);
    for y in cy - r..=cy + r {
        for x in cx - r..=cx + r {
            if (x - cx).pow(2) + (y - cy).pow(2) <= r * r {
                let i = x as usize + y as usize * g.s;
                pousa(&mut g, i, 0.0, nova, (1.0, 0.0));
            }
        }
    }
    let pedaco = super::super::campo_esparso::PEDACO;
    let denso = g.cells * std::mem::size_of::<OleoCelula>();
    let teto = (((2 * r + 1) as usize * g.s).div_ceil(pedaco) + 1)
        * pedaco
        * std::mem::size_of::<OleoCelula>();
    let vivos = bytes_alocados(&g);
    assert!(
        vivos > 0 && vivos <= teto && vivos * 8 < denso,
        "{vivos} B: as linhas do dab cabem em {teto} B, o denso {denso} B"
    );
    assert!(g.oleo_real);
    assert_eq!(da_celula(&g.oleo, cx as usize + cy as usize * g.s), nova);
    assert_eq!(da_celula(&g.oleo, 3 + 3 * g.s), OleoCelula::NEUTRA);
}
