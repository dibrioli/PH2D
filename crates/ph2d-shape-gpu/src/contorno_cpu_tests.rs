use super::*;
use crate::eixo::eixo;
use ph2d_vector::{Cap, Join, Point, Shape, Stroke};

/// Um L aberto `(0,0) → (10,0) → (10,30)`, caneta `2`, com o 1.º traço a acabar `0,3` DEPOIS da quina
/// (`[traço, vão] = [10,3; 19,4]`: o ajuste do aberto é `1`, o 2.º traço acaba no fim).
fn l_tracejado(ponta: Cap) -> (BezPath, Stroke) {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((10.0, 0.0));
    bp.line_to((10.0, 30.0));
    let s = Stroke::new(2.0)
        .with_join(Join::Miter)
        .with_caps(ponta)
        .with_dashes(0.0, [10.3, 19.4]);
    (bp, s)
}

/// A distância de `p` ao trecho do 1.º traço: o L até `0,3` depois da quina.
fn ao_primeiro_traco(p: Point) -> f64 {
    let seg = |a: Point, b: Point| {
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.hypot2()).clamp(0.0, 1.0);
        (p - (a + ab * t)).hypot()
    };
    seg(Point::new(0.0, 0.0), Point::new(10.0, 0.0)).min(seg(Point::new(10.0, 0.0), Point::new(10.0, 0.3)))
}

/// Os pontos de uma grelha à volta da quina que ficam a `≤ 0,95` do 1.º traço (dentro da união
/// verdadeira de qualquer ponta e junta) e que `contorno` deixa por pintar (enrolamento `0`).
fn buracos_na_quina(contorno: &BezPath) -> usize {
    let mut n = 0;
    for i in 0..=120 {
        for j in 0..=120 {
            let p = Point::new(8.5 + 3.0 * f64::from(i) / 120.0, -1.5 + 3.0 * f64::from(j) / 120.0);
            if ao_primeiro_traco(p) <= 0.95 && contorno.winding(p) == 0 {
                n += 1;
            }
        }
    }
    n
}

/// ⭐⭐ **O pedaço rente depois de uma quina é a UNIÃO verdadeira** — todo ponto a menos de meia caneta do
/// traço (pontas redondas: a soma de Minkowski do traço com o disco) fica pintado. ⚠️ O controlo da
/// mordida (o traçador do Vello, na placa gráfica) vive no arnês de GPU do Motion: o `expand_stroke` da casa
/// não morde neste desenho (varrido em `30°`–`170°` × `0,05`–`0,8` além da quina, doc 121 §9.18 C).
#[test]
fn o_pedaco_rente_depois_de_uma_quina_e_a_uniao_verdadeira() {
    let (bp, s) = l_tracejado(Cap::Round);
    let mut pecas = Vec::new();
    eixo(&bp, &s, 0.001, &mut pecas);
    let mut lei = BezPath::new();
    contorno_do_eixo(&pecas, &AfimDaCopia::IDENTIDADE, 0.01, true, &mut lei);
    assert_eq!(buracos_na_quina(&lei), 0, "a lei da placa deixou pontos por pintar na quina");
}

/// O afim de uma cópia leva a lei do espaço local para o ecrã: a cópia `× 2` (caneta `2`, pixel `1`) é a
/// local (pixel `0,5`) escalada — os mesmos polígonos.
#[test]
fn a_lei_no_ecra_e_a_local_pelo_afim() {
    let (bp, s) = l_tracejado(Cap::Round);
    let mut pecas = Vec::new();
    eixo(&bp, &s, 0.001, &mut pecas);
    let mut local = BezPath::new();
    contorno_do_eixo(&pecas, &AfimDaCopia::IDENTIDADE, 0.5, true, &mut local);
    let mut ecra = BezPath::new();
    let m = AfimDaCopia {
        lin: [2.0, 0.0, 0.0, 2.0],
        t: [3.0, -1.0],
        caneta: 2.0,
    };
    contorno_do_eixo(&pecas, &m, 1.0, true, &mut ecra);
    let a: Vec<_> = local.elements().to_vec();
    let b: Vec<_> = ecra.elements().to_vec();
    assert_eq!(a.len(), b.len(), "o número de vértices muda com o afim");
    let longe = a.iter().zip(&b).any(|(x, y)| match (x, y) {
        (ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p), ph2d_vector::PathEl::MoveTo(q) | ph2d_vector::PathEl::LineTo(q)) => {
            (Point::new(2.0 * p.x + 3.0, 2.0 * p.y - 1.0) - *q).hypot() > 1.0e-4
        }
        _ => false,
    });
    assert!(!longe, "a lei no ecrã não é a local pelo afim");
}

/// O `copia_de` do shader: o nível mais grosso com o erro `≤ 0,25 px`, e a conformidade.
#[test]
fn o_nivel_e_a_conformidade_sao_os_do_shader() {
    let tol: [f32; LEVELS] = std::array::from_fn(|k| 0.1 * 0.25_f32.powi(k as i32));
    assert_eq!(nivel_da_copia(&tol, [2.0, 0.0, 0.0, 2.0]), NivelDaCopia { nivel: 0, conforme: true });
    let n = nivel_da_copia(&tol, [40.0, 0.0, 0.0, 10.0]);
    assert_eq!(n, NivelDaCopia { nivel: 2, conforme: false });
    assert!(nivel_da_copia(&tol, [0.0, 3.0, -3.0, 0.0]).conforme, "uma rotação é conforme");
    assert!((caneta_de([40.0, 0.0, 0.0, 10.0]) - 20.0).abs() < 1.0e-5);
}

