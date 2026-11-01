//! ⭐ GEF 19 E4 — o Flip na ordem: o plano parte as faixas em BAIXO · PRINCIPAL · CIMA, e cada faixa
//! desenha-se UMA vez. *Mutações: `main.1 = i + 1` (o Flip de cima desenhado duas vezes, ou nenhuma) ·
//! sem sprites `(0, len)` (tudo na metade de baixo E na de cima — o vetor colado duas vezes).*
use super::plan_frame;
use ph2d_render::draw_bands::{Family, FrameOrder};

fn ordem(fams: &[Family]) -> FrameOrder {
    let mut o = FrameOrder::default();
    for (r, f) in fams.iter().enumerate() {
        let r = r as u32;
        match f {
            Family::Flip => o.record_flip(r, 100 + u64::from(r)),
            Family::Vector => o.record(r, Some(u64::from(r))),
            _ => o.record(r, None),
        }
    }
    o
}

#[test]
fn every_band_draws_once_and_the_flip_above_the_last_sprites_is_the_main_one() {
    use Family::{Flip as F, Sprite as S, Vector as V};
    for (fams, main) in [
        (&[S, F][..], (0, 2)),
        (&[F, S][..], (1, 2)),
        (&[S, V, F][..], (0, 1)),
        (&[V, F][..], (1, 2)),
        (&[F, S, F, V][..], (1, 3)),
        (&[V][..], (0, 0)),
    ] {
        let p = plan_frame(&ordem(fams));
        assert_eq!(p.main, main, "{fams:?}");
        let n = p.bands.len();
        assert!(p.main.0 <= p.main.1 && p.main.1 <= n);
        let em_main: Vec<_> = p.bands[p.main.0..p.main.1]
            .iter()
            .map(|b| b.family)
            .collect();
        assert!(em_main.iter().filter(|f| **f == S).count() <= 1);
        assert_eq!(
            p.flip_main.map(|b| b.family),
            em_main.contains(&F).then_some(F),
            "{fams:?}"
        );
    }
}

/// As duas metades desenham a faixa Flip que lhes cabe (o passe principal já desenhou a dele).
#[test]
fn both_halves_paste_their_flip_bands() {
    let src = include_str!("present_bands.rs");
    assert_eq!(
        src.matches("cola_o_flip(\n").count(),
        3,
        "a definição e as duas metades"
    );
    assert!(src.contains("passe_de_baixo(&plan.bands[..plan.main.0])"));
    assert!(src.contains("let from = plan.main.1;"));
    let fx = include_str!("present.rs");
    assert!(fx.contains(".flip_main") && fx.contains("quadro.desenha(&ids, game_rt);"));
}
