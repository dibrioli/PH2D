//! ⭐ A faixa muda (olho fechado, dono 10/11): a strip e as alças nas cores ORIGINAIS, meio
//! apagadas por um fator só (`desbote`); os estados iguais aos da faixa viva; o cursor tinge-se da
//! cor de REPOUSO da faixa viva.

use super::*;
use ph2d_tokens::TIMELINE_PECA_MUDA_ALPHA as K;
use ph2d_vector::Shape;

fn snap_mudo() -> TimelineViewSnapshot {
    let mut snap = snap_duas_pecas();
    snap.lanes[0].muted = true;
    snap
}

/// Onde a peça mora (`[0.5, 2.5] s` × 200 px/s: `x ∈ [200, 600]`), com folga para a sombra: o fundo
/// da faixa (até 900) e a coluna dos nomes (o olho muda lá, de propósito) ficam de fora.
const PECA: (f64, f64) = (198.0, 602.0);

/// Cada cor sólida (e cada parada de degradê, cada texto) que a strip pinta, pela ordem.
fn tintas_do_tempo(ops: &[DrawOp]) -> Vec<Color> {
    let mut out = Vec::new();
    for op in ops {
        match op {
            DrawOp::Fill {
                brush,
                path,
                transform,
                ..
            }
            | DrawOp::Stroke {
                brush,
                path,
                transform,
                ..
            } => {
                let b = (*transform * path.clone()).bounding_box();
                if b.x0 < PECA.0 || b.x1 > PECA.1 {
                    continue;
                }
                match brush {
                    Brush::Solid(c) => out.push(*c),
                    Brush::Gradient(g) => {
                        out.extend(g.stops.iter().map(|s| s.color.to_alpha_color()))
                    }
                    Brush::Image(_) => {}
                }
            }
            DrawOp::Glyphs(g) if g.transform.translation().x >= PECA.0 => out.push(g.brush),
            _ => {}
        }
    }
    out
}

fn rgba(c: Color) -> [u8; 4] {
    let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let k = c.components;
    [u(k[0]), u(k[1]), u(k[2]), u8::MAX]
}

/// O ângulo de matiz OKLCh (graus) e o croma de `c`.
fn matiz(c: Color) -> (f64, f64) {
    let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let k = c.components;
    let (_, ch, h) = ph2d_tokens::srgb_to_oklch(u(k[0]), u(k[1]), u(k[2]));
    (h, ch)
}

/// ⭐ **Corpo, contorno, ícone, nome, mistura e CADA alça com a sombra: o mesmo matiz, a opacidade
/// × `K`** — em todo tema, cor a cor, pela mesma ordem de pintura. *Mutações: o fade das alças em
/// `Border` (o cinza de antes) ⇒ matiz muda; os cantos a pleno ⇒ a opacidade não é × `K`.*
#[test]
fn a_faixa_muda_tem_as_cores_originais_meio_apagadas() {
    let mut pior_matiz: f64 = 0.0;
    for theme in Theme::ALL {
        let viva =
            tintas_do_tempo(&pinta_faixa_de(theme, true, &snap_duas_pecas(), |_| {}, |_| ()).0);
        let muda = tintas_do_tempo(&pinta_faixa_de(theme, true, &snap_mudo(), |_| {}, |_| ()).0);
        assert_eq!(viva.len(), muda.len(), "{theme:?}: as mesmas partes");
        for (i, (v, m)) in viva.iter().zip(&muda).enumerate() {
            let (a, b) = (v.components, m.components);
            assert_eq!(a[..3], b[..3], "{theme:?} #{i}: a cor original");
            assert!(
                (b[3] - a[3] * K).abs() < 1e-5,
                "{theme:?} #{i}: {} ≠ {} × K",
                b[3],
                a[3]
            );
            let ((hv, cv), (hm, _)) = (matiz(*v), matiz(*m));
            if cv > 0.02 {
                let d = (hv - hm).abs().min(360.0 - (hv - hm).abs());
                pior_matiz = pior_matiz.max(d);
            }
        }
        let tem = |c: Color| muda.contains(&c.multiply_alpha(K));
        let alcas = [
            ("canto esticar", resolve(ColorToken::Success, theme)),
            ("canto cortar", resolve(ColorToken::Danger, theme)),
            ("fade", resolve(ColorToken::TimelinePlayhead, theme)),
        ];
        for (nome, c) in alcas {
            assert!(tem(c), "{theme:?}: {nome} na cor dela, desbotada");
        }
        let p = crate::pecas_cartoes::PecaDesenho {
            tinta: crate::pecas_cartoes::tinta_do_strip(
                &snap_mudo().lanes[0].strips[0],
                &snap_mudo(),
            ),
            icon: ph2d_editor_core::icons::IconId::Film,
            nome: "",
            extra: None,
            muda: false,
            escolhida: false,
            theme,
        };
        let c = crate::pecas_cartoes::cores(&p);
        for (nome, cor) in [
            ("topo", c.topo),
            ("pé", c.pe),
            ("contorno", c.borda),
            ("nome", c.texto),
        ] {
            assert!(tem(cor), "{theme:?}: o {nome} da peça desbotado");
        }
    }
    eprintln!("pior matiz viva × muda: {pior_matiz:.4}°");
    assert!(pior_matiz < 1.0);
}

/// Na faixa muda os estados são os da viva: hover acende (desbotado), aperto e arrasto a pleno (o
/// que a mão segura vê-se); e o cursor tinge-se da cor de REPOUSO da faixa viva, nunca da apagada.
#[test]
fn na_faixa_muda_as_alcas_tem_os_estados_e_o_cursor_a_cor_viva() {
    let theme = Theme::Forge;
    let snap = snap_mudo();
    let azul = acento(theme);
    let pinta_com = |snap: &TimelineViewSnapshot, id: NodeId, aperta: bool| {
        let prepara = |s: &mut WidgetStore| {
            if aperta {
                s.set_active(Some(id));
            } else {
                s.set_hot(Some(id));
            }
        };
        pinta_faixa_de(theme, true, snap, prepara, |s| s.timeline_cursor_tint(id))
    };
    for (edge, repouso) in [
        (5u8, resolve(ColorToken::Success, theme)),
        (1, resolve(ColorToken::Danger, theme)),
        (EASE_IN_CODE, resolve(ColorToken::TimelinePlayhead, theme)),
    ] {
        let id = id_de(&snap, edge);
        let (h, ..) = pinta_com(&snap, id, false);
        let quente = cor(EstadoAlca::Hover, repouso, theme).multiply_alpha(K);
        assert!(
            cores(&h).contains(&quente.components),
            "edge {edge}: o hover desbotado"
        );
        let (p, _, _, tinta_m) = pinta_com(&snap, id, true);
        assert!(
            cores(&p).contains(&azul.components),
            "edge {edge}: o aperto a pleno"
        );
        let (_, _, _, tinta_v) = pinta_com(&snap_duas_pecas(), id, true);
        let (_, _, _, hover_m) = pinta_com(&snap, id, false);
        assert_eq!(tinta_m, tinta_v, "edge {edge}: o cursor da faixa viva");
        assert_eq!(hover_m, tinta_v, "edge {edge}: o cursor no hover também");
        assert_eq!(
            tinta_m,
            Some(rgba(repouso)),
            "edge {edge}: a cor de repouso"
        );
    }
}
