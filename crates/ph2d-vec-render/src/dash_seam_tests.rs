//! ⭐ **O fecho de um tracejado cai DENTRO do último vão** — nenhum traço se emenda por trás do 1.º
//! (`docs/Vector Module/BUGS_vector.md` #42, a faixa da cena 24 do dono, 15/10).
//!
//! A régua é o que o traçador RECEBE: o `kurbo::dash` (o mesmo que o `vello::Scene::stroke` corre)
//! sobre o contorno cozido, com o padrão da porta única ([`ph2d_vec_scene::dash_fit::dash_lengths_for`]).
//! O CONTROLO é o padrão sem a folga ([`ph2d_vec_scene::dash_fit::fit`]): ali o arredondamento do
//! arco decide o fecho, e uma parte da população TEM de emendar — senão a régua não vê o defeito.

use ph2d_vec_scene::{DashPattern, Rgba8, ShapeKind, StrokeSpec, cook, dash_fit};
use ph2d_vector::{BezPath, PathEl, Point, Shape, dash};

/// O comprimento que o tracejado de `bp` põe ANTES do início do contorno, no traço que emenda o fim
/// ao começo — `None` se nenhum traço passa pelo início vindo de trás.
fn emenda(bp: &BezPath, d: &[f64]) -> Option<f64> {
    let Some(PathEl::MoveTo(inicio)) = bp.elements().first().copied() else {
        return None;
    };
    let (mut ultimo, mut acc, mut achou) = (Point::ZERO, 0.0, None);
    for el in dash(bp.iter(), 0.0, d) {
        let fim = match el {
            PathEl::MoveTo(p) => {
                (ultimo, acc) = (p, 0.0);
                continue;
            }
            PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => p,
            PathEl::ClosePath => continue,
        };
        let mut seg = BezPath::new();
        seg.move_to(ultimo);
        seg.push(el);
        acc += seg.perimeter(1e-12);
        ultimo = fim;
        if ultimo.distance(inicio) < 1e-12 && acc > 0.0 {
            achou = Some(acc);
        }
    }
    achou
}

/// A população: rectângulos de cantos vivos (as arestas são cúbicas DEGENERADAS — alças sobre os
/// nós), um round-rect e uma elipse, em vários tamanhos, com o padrão da cena 24 e dois de um par.
fn populacao() -> Vec<(String, ph2d_vec_scene::VecPath, StrokeSpec)> {
    let padroes = [
        DashPattern {
            pairs: [[4.0, 1.5], [0.0, 1.5], [0.0, 1.5]],
        },
        DashPattern::single(4.0, 1.5),
        DashPattern::single(0.001, 1.5),
    ];
    let mut v = Vec::new();
    for kind in [
        ShapeKind::Rectangle,
        ShapeKind::RoundRect,
        ShapeKind::Ellipse,
    ] {
        for i in 0..24 {
            let (w, h) = (0.4 + 0.113 * f64::from(i), 1.6 - 0.041 * f64::from(i));
            let mut path = cook(
                kind,
                [-w * 0.5, -h * 0.5],
                [w * 0.5, h * 0.5],
                &kind.defaults(),
            );
            for (j, pat) in padroes.iter().enumerate() {
                let mut s = StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.05);
                s.dash = Some(*pat);
                path.stroke = Some(s.clone());
                v.push((
                    format!("{kind:?} {w:.3}x{h:.3} padrão {j}"),
                    path.clone(),
                    s,
                ));
            }
        }
    }
    v
}

#[test]
fn o_fecho_de_um_tracejado_cai_dentro_do_ultimo_vao() {
    let (mut controlo, mut casos) = (std::collections::BTreeMap::<String, usize>::new(), 0);
    for (nome, path, s) in populacao() {
        let cooked = path.cooked();
        let bp = crate::build_bezpath(&cooked);
        let (total, fechado) = dash_fit::longest_contour(&cooked).expect("contorno");
        assert!(fechado, "{nome}");
        let sem_folga = dash_fit::fit(s.dash_lengths().expect("tracejado"), total, true);
        let familia = nome.split(' ').next().unwrap_or("").to_owned() + &nome[nome.len() - 1..];
        *controlo.entry(familia).or_default() +=
            usize::from(emenda(&bp, sem_folga.as_slice()).is_some());
        let d = dash_fit::dash_lengths_for(&cooked, &s).expect("tracejado");
        assert_eq!(
            emenda(&bp, d.as_slice()),
            None,
            "{nome}: o último traço emenda-se ao 1.º por trás do início — a lasca que o Vello pinta \
             como faixa"
        );
        casos += 1;
    }
    eprintln!("sem a folga, fechos que emendam (forma + padrão, de 24 cada): {controlo:?}");
    assert!(
        controlo.values().sum::<usize>() > 0,
        "CONTROLO: sem a folga nenhum dos {casos} casos emendou — a régua não vê o defeito"
    );
}

/// O caso do dono, exacto: o rectângulo da cena 24 (`1,6 × 1,6`, traço `0,05`, traço-ponto-ponto).
/// Sem a folga o fecho emenda uma lasca de `~3e-7` m ao 1.º traço; com ela, nada emenda.
#[test]
fn o_rectangulo_da_cena_24_nao_emenda_uma_lasca_no_fecho() {
    let mut path = cook(
        ShapeKind::Rectangle,
        [-0.8, -0.8],
        [0.8, 0.8],
        &ShapeKind::Rectangle.defaults(),
    );
    let mut s = StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 0.05);
    s.dash = Some(DashPattern {
        pairs: [[4.0, 1.5], [0.0, 1.5], [0.0, 1.5]],
    });
    path.stroke = Some(s.clone());
    let cooked = path.cooked();
    let bp = crate::build_bezpath(&cooked);
    let (total, _) = dash_fit::longest_contour(&cooked).expect("contorno");
    let sem_folga = dash_fit::fit(s.dash_lengths().expect("tracejado"), total, true);
    let lasca = emenda(&bp, sem_folga.as_slice());
    assert!(
        lasca.is_some_and(|l| l < 1e-6),
        "CONTROLO: sem a folga a lasca do fecho é sub-micrométrica ({lasca:?})"
    );
    let d = dash_fit::dash_lengths_for(&cooked, &s).expect("tracejado");
    assert_eq!(emenda(&bp, d.as_slice()), None);
}
