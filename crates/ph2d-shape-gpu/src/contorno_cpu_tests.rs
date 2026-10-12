use super::*;
use crate::eixo::eixo;
use ph2d_vector::{Cap, Join, Point, Shape, Stroke};

/// A escala da fixtura: em pixels de ecrã (a lei é escrita neles), uma caneta de `40 px`.
const K: f64 = 20.0;

/// Um L aberto `(0,0) → (10,0) → (10,30)` (× [`K`]), caneta `2 K`, com o 1.º traço a acabar `0,3 K` DEPOIS da
/// quina (`[traço, vão] = [10,3; 19,4] K`: o ajuste do aberto é `1`, o 2.º traço acaba no fim).
fn l_tracejado(ponta: Cap) -> (BezPath, Stroke) {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((10.0 * K, 0.0));
    bp.line_to((10.0 * K, 30.0 * K));
    let s = Stroke::new(2.0 * K)
        .with_join(Join::Miter)
        .with_caps(ponta)
        .with_dashes(0.0, [10.3 * K, 19.4 * K]);
    (bp, s)
}

/// A distância de `p` ao trecho do 1.º traço: o L até `0,3` depois da quina.
fn ao_primeiro_traco(p: Point) -> f64 {
    let q = Point::new(p.x / K, p.y / K);
    let seg = |a: Point, b: Point| {
        let ab = b - a;
        let t = ((q - a).dot(ab) / ab.hypot2()).clamp(0.0, 1.0);
        (q - (a + ab * t)).hypot()
    };
    seg(Point::new(0.0, 0.0), Point::new(10.0, 0.0))
        .min(seg(Point::new(10.0, 0.0), Point::new(10.0, 0.3)))
}

/// Os pontos de uma grelha à volta da quina que ficam a `≤ 0,95` do 1.º traço (dentro da união
/// verdadeira de qualquer ponta e junta) e que `contorno` deixa por pintar (enrolamento `0`).
fn buracos_na_quina(contorno: &BezPath) -> usize {
    let mut n = 0;
    for i in 0..=120 {
        for j in 0..=120 {
            let p = Point::new(
                K * (8.5 + 3.0 * f64::from(i) / 120.0),
                K * (-1.5 + 3.0 * f64::from(j) / 120.0),
            );
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
    eixo(&bp, &s, 0.1, &mut pecas);
    let mut lei = BezPath::new();
    contorno_do_eixo(&pecas, &AfimDaCopia::IDENTIDADE, true, &mut lei);
    assert_eq!(
        buracos_na_quina(&lei),
        0,
        "a lei da placa deixou pontos por pintar na quina"
    );
}

/// O afim de uma cópia leva o eixo ao ecrã: uma cópia deslocada desenha os MESMOS polígonos deslocados.
#[test]
fn a_lei_no_ecra_e_a_local_pelo_afim() {
    let (bp, s) = l_tracejado(Cap::Round);
    let mut pecas = Vec::new();
    eixo(&bp, &s, 0.1, &mut pecas);
    let mut local = BezPath::new();
    contorno_do_eixo(&pecas, &AfimDaCopia::IDENTIDADE, true, &mut local);
    let mut ecra = BezPath::new();
    let m = AfimDaCopia {
        t: [3.0, -1.0],
        ..AfimDaCopia::IDENTIDADE
    };
    contorno_do_eixo(&pecas, &m, true, &mut ecra);
    let a: Vec<_> = local.elements().to_vec();
    let b: Vec<_> = ecra.elements().to_vec();
    assert_eq!(a.len(), b.len(), "o número de vértices muda com o afim");
    let longe = a.iter().zip(&b).any(|(x, y)| match (x, y) {
        (
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p),
            ph2d_vector::PathEl::MoveTo(q) | ph2d_vector::PathEl::LineTo(q),
        ) => (Point::new(p.x + 3.0, p.y - 1.0) - *q).hypot() > 1.0e-3,
        _ => false,
    });
    assert!(!longe, "a lei no ecrã não é a local pelo afim");
}

/// O `copia_de` do shader: o nível mais grosso com o erro `≤ 0,25 px`, e a conformidade.
#[test]
fn o_nivel_e_a_conformidade_sao_os_do_shader() {
    let tol: [f32; LEVELS] = std::array::from_fn(|k| 0.1 * 0.25_f32.powi(k as i32));
    assert_eq!(
        nivel_da_copia(&tol, [2.0, 0.0, 0.0, 2.0]),
        NivelDaCopia {
            nivel: 0,
            conforme: true
        }
    );
    let n = nivel_da_copia(&tol, [40.0, 0.0, 0.0, 10.0]);
    assert_eq!(
        n,
        NivelDaCopia {
            nivel: 2,
            conforme: false
        }
    );
    assert!(
        nivel_da_copia(&tol, [0.0, 3.0, -3.0, 0.0]).conforme,
        "uma rotação é conforme"
    );
    assert!((caneta_de([40.0, 0.0, 0.0, 10.0]) - 20.0).abs() < 1.0e-5);
}

/// Um V de pernas `10 K` com a quina na origem, ângulo interno `phi` (graus), o bico para `+y`;
/// caneta `2 K`, junta em esquadria, limite `4`.
fn v_agudo(phi: f64) -> (BezPath, Stroke) {
    let (s, c) = (phi.to_radians() * 0.5).sin_cos();
    let l = 10.0 * K;
    let mut bp = BezPath::new();
    bp.move_to((-l * s, -l * c));
    bp.line_to((0.0, 0.0));
    bp.line_to((l * s, -l * c));
    (bp, Stroke::new(2.0 * K).with_join(Join::Miter).with_miter_limit(4.0))
}

/// A altura do bico: o último ponto pintado subindo a bissectriz (`x = 0`) desde a quina.
fn topo(contorno: &BezPath) -> f64 {
    (0..=24_000)
        .map(|i| f64::from(i) * 0.005)
        .filter(|&y| contorno.winding(Point::new(0.0, y)) != 0)
        .fold(0.0, f64::max)
}

/// ⭐ (F13 E) **A placa do Motion corta o bico como o resto da casa** (SVG 2 `miter-clip`): além do
/// limite o topo fica a `limite × ½w` = `4 K` — contínuo, sem o salto para o chanfro (o fixture do
/// resvg, doc 19 §2); dentro dele, o bico inteiro. Oráculo: `miter_clip::stroke_outline` (o traço
/// conforme da mesma placa e o render da cena). Controlo: com o limite NEGATIVO (`PH2D_MITER_CLIP=0`)
/// o chanfro de antes — o topo cai abaixo de `K`.
#[test]
fn a_placa_corta_o_bico_alem_do_limite_como_o_oraculo() {
    for phi in [60.0, 30.0, 28.0, 20.0, 10.0, 1.0] {
        let (bp, s) = v_agudo(phi);
        let mut pecas = Vec::new();
        eixo(&bp, &s, 0.1, &mut pecas);
        let mut lei = BezPath::new();
        contorno_do_eixo(&pecas, &AfimDaCopia::IDENTIDADE, false, &mut lei);
        let oraculo = ph2d_vec_render::miter_clip::stroke_outline(&bp, &s, 0.01);
        let (t, o) = (topo(&lei), topo(&oraculo));
        let inteiro = K / (phi.to_radians() * 0.5).sin();
        let quer = if inteiro <= 4.0 * K { inteiro } else { 4.0 * K };
        assert!((t - quer).abs() < 0.05 && (o - quer).abs() < 0.05, "{phi}°: placa {t}, oráculo {o}, lei {quer}");
        if inteiro > 4.0 * K {
            for it in &mut pecas {
                it.limite_esquadria = -it.limite_esquadria;
            }
            let mut antes = BezPath::new();
            contorno_do_eixo(&pecas, &AfimDaCopia::IDENTIDADE, false, &mut antes);
            assert!(topo(&antes) < K, "{phi}°: o chanfro de antes não voltou com o limite negativo");
        }
    }
}
