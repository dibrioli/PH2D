//! Gates da F14b — o raio arrastado até ao canto É zero; o bico dos balões tem ângulo; as cúspides
//! de ângulo zero são a lista escrita. Irmão pelo tecto de LOC.

use super::tests::cooker;
use super::*;
use ph2d_vec_scene::ALL_SHAPES;

/// ⭐ (F14b, o dono: «Radius 0» desenhava orelhas no traço) **O ⊙ de arredondar levado à quina
/// guarda ZERO, não uma fracção**: em toda alça `Spot::Rounding` do catálogo, a mão sobre o ponto
/// onde o raio mínimo põe a alça devolve exactamente o mínimo. Antes: `1e-8`–`2e-5` px (a busca
/// desempatava a favor do valor do press; a fila mostrava «0»). Controlo: a mão 2 px trilho adentro
/// guarda um raio, e o encaixe nunca vai ao MÁXIMO do campo.
#[test]
fn a_rounding_dragged_to_the_corner_is_exactly_zero() {
    let half = [50.0, 47.7];
    let mut checked = 0;
    for &k in ALL_SHAPES {
        let ck = cooker(k, half);
        for (n, h) in handles_of(k).iter().enumerate() {
            let (Spot::Rounding { .. }, Law::Nearest, [i]) = (h.spot, h.law, h.fields) else {
                continue;
            };
            let v = k.defaults();
            let Some((lo, hi, false)) = range_world(k, *i, 1.0) else {
                continue;
            };
            let mut z = v;
            z[*i] = lo;
            let mut one = v;
            one[*i] = lo + 2.0;
            let (Some(zero), Some(two)) = (
                handle_point(h, &z, half, &ck),
                handle_point(h, &one, half, &ck),
            ) else {
                continue;
            };
            let to = |p: [f64; 2]| {
                let q = handle_grip(k, h, &v, half, 1.0, 18.0, &ck).expect("grip");
                let mut d = HandleDrag::start(n, v, q, q, half);
                drag(k, &mut d, p, Mods::default(), half, 1.0, &ck)[*i]
            };
            assert!(
                to(zero) == lo,
                "{k:?} alça {n}: a quina guardou {:e}",
                to(zero)
            );
            let far = to(two);
            assert!(
                far > lo + 0.5 && far < hi,
                "{k:?} alça {n}: 2 px adentro guardou {far}"
            );
            checked += 1;
        }
    }
    assert!(checked >= 20, "controlo: só {checked} alças de arredondar");
}

/// O ângulo INTERNO (graus) de cada vértice de `p` com chegada e saída — os extremos de um contorno
/// aberto não têm.
fn inner_angles(p: &VecPath) -> Vec<f64> {
    let d = |a: [f64; 2], b: [f64; 2]| {
        let (x, y) = (b[0] - a[0], b[1] - a[1]);
        let l = x.hypot(y);
        (l > 1e-9).then(|| [x / l, y / l])
    };
    let mut out = Vec::new();
    let mut run = |vs: &[ph2d_vec_scene::VecVertex], closed: bool| {
        let n = vs.len();
        for i in 0..n {
            if !closed && (i == 0 || i + 1 == n) {
                continue;
            }
            let (pv, v, nx) = (&vs[(i + n - 1) % n], &vs[i], &vs[(i + 1) % n]);
            let ent = d(v.in_handle, v.anchor)
                .or_else(|| d(pv.out_handle, v.anchor))
                .or_else(|| d(pv.anchor, v.anchor));
            let sai = d(v.anchor, v.out_handle)
                .or_else(|| d(v.anchor, nx.in_handle))
                .or_else(|| d(v.anchor, nx.anchor));
            if let (Some(e), Some(s)) = (ent, sai) {
                out.push(
                    (-e[0] * s[0] - e[1] * s[1])
                        .clamp(-1.0, 1.0)
                        .acos()
                        .to_degrees(),
                );
            }
        }
    };
    run(&p.verts, p.closed);
    for s in &p.subpaths {
        run(&s.verts, s.closed);
    }
    out
}

/// O padrão e cada campo varrido na faixa inteira (9 valores).
pub(super) fn sweep(k: ShapeKind) -> Vec<ShapeValues> {
    let mut out = vec![k.defaults()];
    for i in shapes::shown_fields(k) {
        let f = &shapes::desc(k).fields[i];
        out.extend((0..=8).map(|s| {
            let mut v = k.defaults();
            v[i] = f.min + (f.max - f.min) * f64::from(s) / 8.0;
            v
        }));
    }
    out
}

/// O ângulo abaixo do qual o *Miter* (limite 4) corta o bico: `2·asin(1/4)`.
const MITER_CUT_DEG: f64 = 28.955;

/// ⭐ (F14b, o dono: «nenhuma opção do painel lateral produz uma ponta pontuda») **O bico dos dois
/// balões de fala abre acima do corte do *Miter* por omissão, e nunca é agulha**: o padrão mede 42°
/// (rectângulo) e 35° (oval); em toda a varredura dos campos o menor é 8° (o rabo fino de base
/// mínima, a 0,05 da caixa) — antes, 0° em TODO valor.
#[test]
fn the_speech_tips_have_an_angle() {
    for k in [ShapeKind::SpeechRect, ShapeKind::SpeechOval] {
        let ck = cooker(k, [50.0, 47.7]);
        let def = inner_angles(&ck(&k.defaults()));
        let tip = def.iter().copied().fold(f64::INFINITY, f64::min);
        assert!(
            tip > MITER_CUT_DEG + 3.0,
            "{k:?}: o bico por omissão mede {tip:.1}°"
        );
        for v in sweep(k) {
            let m = inner_angles(&ck(&v))
                .into_iter()
                .fold(f64::INFINITY, f64::min);
            assert!(m > 5.0, "{k:?} {v:?}: bico de {m:.2}° (agulha)");
        }
    }
}

/// ⭐ (F14b) **As cúspides de ângulo ZERO do catálogo são a lista escrita** (padrão e cada campo
/// varrido): a estrela e o polígono no *Curve* ±1 (doc 19 §5), os chifres da lua (o terminador
/// astronómico, tangente ao limbo — `symbols::moon`) e o bico da chave (o `{` do OOXML). Uma PONTA
/// nova de ângulo zero (como era o rabo dos balões) aparece aqui antes de chegar ao dono.
#[test]
fn the_zero_angle_cusps_are_the_written_list() {
    use ShapeKind as K;
    let found: Vec<ShapeKind> = ALL_SHAPES
        .iter()
        .copied()
        .filter(|&k| {
            let ck = cooker(k, [100.0, 100.0]);
            sweep(k)
                .iter()
                .any(|v| inner_angles(&ck(v)).iter().any(|&a| a < 0.25))
        })
        .collect();
    assert_eq!(found, [K::Polygon, K::Star, K::Brace, K::Moon]);
}
