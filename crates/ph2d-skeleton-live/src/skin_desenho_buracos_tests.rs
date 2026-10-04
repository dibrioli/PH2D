//! A5-b — os buracos que o traço engole fecham-se.
//!
//! ⚠️ A régua conta os buracos do DESENHO (contornos de sentido contrário ao maior) e o raio
//! inscrito de cada um numa grelha PRÓPRIA (`64 × 64`, não a `24 × 24` da lei).

use crate::barra_da_cena_tests_support::osso;
use crate::skin_desenho::Leis;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

/// A largura do traço da barra da cena `=5` (`0,75 · 0,06`).
const LARGURA: f64 = 0.75 * 0.06;

/// A barra *Zig Zag* da cena `=5` (`4,5 × 0,75`, três ossos), presa recta e dobrada em S a `graus`,
/// desenhada numa thread NOVA (o memo do quadro é por thread) com ou sem a lei.
fn zig_zag_em_s(graus: f32, sem_fecho: bool) -> VecPath {
    std::thread::scope(|s| {
        s.spawn(|| {
            super::SEM_FECHO.with(|c| c.set(sem_fecho));
            let mut sim = ph2d_ecs::SimWorld::default();
            let mut scene = ph2d_vec_scene::VecScene::new();
            let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
            let mut barra = ph2d_vec_scene::cook(
                ph2d_vec_scene::ShapeKind::RoundRect,
                [-2.25, -0.375],
                [2.25, 0.375],
                &[0.375],
            );
            barra.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
                ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
                LARGURA,
            ));
            barra.effects = vec![FxEntry::new(PathEffect::ZigZag(
                ph2d_vec_scene::fx_zigzag::ZigZagSpec {
                    amplitude: 6.0,
                    ridges: 24.0,
                    ..Default::default()
                },
            ))];
            let id = scene.push_path(barra);
            ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
            let passo = (4.5 - 0.75) / 3.0;
            #[expect(clippy::cast_possible_truncation, reason = "metros de uma cena")]
            let p32 = passo as f32;
            let b1 = osso(&mut sim, "b1", [-1.875, 0.0], passo, None);
            let b2 = osso(&mut sim, "b2", [p32, 0.0], passo, Some(b1));
            let b3 = osso(&mut sim, "b3", [p32, 0.0], passo, Some(b2));
            assert_eq!(
                crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], Some(b1)),
                1
            );
            for (b, g) in [(b2, graus), (b3, -graus)] {
                sim.world_mut()
                    .get_mut::<ph2d_ecs::Transform>(b)
                    .expect("Transform")
                    .rotation = g.to_radians();
            }
            let e = ph2d_ecs::Entity::from_bits(map[&id]);
            assert!(sim.world().get::<SkinBind>(e).is_some(), "presa");
            crate::skin_live::recook_leis(&sim, &mut scene.clone(), Leis::do_ambiente())
                .remove(&id)
                .expect("desenho")
                .forma
        })
        .join()
        .expect("thread do desenho")
    })
}

/// O raio inscrito de cada BURACO do desenho, em larguras do traço.
fn buracos(p: &VecPath) -> Vec<f64> {
    let pl: Vec<Vec<[f64; 2]>> = (0..p.contour_count())
        .filter_map(|c| p.contour(c))
        .filter(|(v, f)| *f && v.len() > 1)
        .map(|(v, _)| super::polilinha(v))
        .collect();
    let areas: Vec<f64> = pl.iter().map(|q| super::area(q)).collect();
    let maior = areas
        .iter()
        .copied()
        .max_by(|a, b| a.abs().total_cmp(&b.abs()))
        .unwrap_or(0.0);
    pl.iter()
        .zip(&areas)
        .filter(|(_, a)| **a * maior < 0.0)
        .map(|(q, _)| {
            let (lo, hi) = q.iter().fold(([f64::MAX; 2], [f64::MIN; 2]), |(l, h), p| {
                (
                    [l[0].min(p[0]), l[1].min(p[1])],
                    [h[0].max(p[0]), h[1].max(p[1])],
                )
            });
            let mut r = 0.0_f64;
            for i in 0..64 {
                for j in 0..64 {
                    let s = [
                        (hi[0] - lo[0]).mul_add((f64::from(i) + 0.5) / 64.0, lo[0]),
                        (hi[1] - lo[1]).mul_add((f64::from(j) + 0.5) / 64.0, lo[1]),
                    ];
                    if crate::skin_desenho::frente::tests::dentro(q, s) {
                        r = r.max(crate::skin_desenho::frente::tests::dist_pol(q, s));
                    }
                }
            }
            r / LARGURA
        })
        .collect()
}

/// ⭐⭐⭐ **GATE — no *Zig Zag* dobrado a `100°`/`110°`/`120°` nenhum buraco que o traço cobre fica**,
/// e os que se vêem ficam todos. ⛔ **O CONTROLO:** sem a lei há buracos assim (MEDIDO na `=5`).
#[test]
fn os_buracos_que_o_traco_cobre_fecham_e_os_outros_ficam() {
    let (mut sem_lei, mut vistos) = (0, 0);
    for graus in [100f32, 110.0, 120.0] {
        let (antes, depois) = (
            buracos(&zig_zag_em_s(graus, true)),
            buracos(&zig_zag_em_s(graus, false)),
        );
        println!("  {graus}°: raios sem a lei {antes:.2?} · com {depois:.2?}");
        let engolidos = |v: &[f64]| v.iter().filter(|r| **r < 0.5).count();
        let grandes = |v: &[f64]| {
            let mut g: Vec<String> = v
                .iter()
                .filter(|r| **r >= 0.5)
                .map(|r| format!("{r:.3}"))
                .collect();
            g.sort();
            g
        };
        assert_eq!(
            engolidos(&depois),
            0,
            "a {graus}° ficou um buraco que o traço cobre"
        );
        assert_eq!(
            grandes(&antes),
            grandes(&depois),
            "a {graus}° um buraco que se vê mudou"
        );
        sem_lei += engolidos(&antes);
        vistos += grandes(&depois).len();
    }
    assert!(
        sem_lei > 0,
        "o CONTROLO: sem a lei não há buraco que o traço cubra"
    );
    assert!(
        vistos > 0,
        "o CONTROLO: nenhum buraco que se vê na varredura"
    );
}

/// Um quadrado `[-1, 1]²` com um buraco quadrado de meio-lado `h` no centro, e o traço da barra.
fn quadrado_com_buraco(h: Option<f64>) -> VecPath {
    let c = |pts: &[[f64; 2]]| ph2d_vec_scene::Contour {
        verts: pts
            .iter()
            .map(|&p| ph2d_vec_scene::VecVertex::corner(p))
            .collect(),
        closed: true,
    };
    let fora = c(&[[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]);
    VecPath {
        verts: fora.verts,
        closed: true,
        subpaths: h
            .map(|h| c(&[[-h, -h], [-h, h], [h, h], [h, -h]]))
            .into_iter()
            .collect(),
        stroke: Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 0, 255),
            LARGURA,
        )),
        ..VecPath::default()
    }
}

/// ⭐⭐ **GATE — só o buraco NOVO e pequeno sai:** o que a fonte já tinha fica (o artista desenhou-o),
/// e o novo que se vê fica. ⛔ **O CONTROLO:** o novo e pequeno sai mesmo.
#[test]
fn so_o_buraco_novo_que_o_traco_cobre_sai() {
    let pequeno = 0.01; // meio-lado `0,01` < meia largura `0,0225`
    let mut u = quadrado_com_buraco(Some(pequeno));
    super::fecha_os_buracos_que_o_traco_engole(&mut u, &quadrado_com_buraco(Some(pequeno)));
    assert_eq!(u.contour_count(), 2, "o buraco do artista saiu");
    let mut u = quadrado_com_buraco(Some(pequeno));
    super::fecha_os_buracos_que_o_traco_engole(&mut u, &quadrado_com_buraco(None));
    assert_eq!(
        u.contour_count(),
        1,
        "o CONTROLO: o buraco novo e pequeno ficou"
    );
    let mut u = quadrado_com_buraco(Some(0.05));
    super::fecha_os_buracos_que_o_traco_engole(&mut u, &quadrado_com_buraco(None));
    assert_eq!(u.contour_count(), 2, "um buraco novo que se vê saiu");
}
