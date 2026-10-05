//! A dobra do relevo através dos ajustes de vizinhança.

use super::*;
use crate::layers::GroupLayer;
use ph2d_painter_effects::adjustments::{AdjustWindow, GaussianBlurParams, SharpenParams};
use std::collections::BTreeMap;

const W: u32 = 24;
const H: u32 = 16;
const N: usize = (W * H) as usize;

/// Planos sintéticos: `(altura, cobertura)` por camada, e máscaras.
#[derive(Default)]
struct Planos {
    camadas: BTreeMap<LayerId, (Vec<f32>, Vec<f32>)>,
    mascaras: BTreeMap<LayerId, Vec<f32>>,
}

impl ReliefSamples for Planos {
    fn len(&self) -> usize {
        N
    }
    fn has(&self, id: LayerId) -> bool {
        self.camadas.contains_key(&id)
    }
    fn at(&self, id: LayerId, i: usize) -> (f32, f32) {
        let (h, c) = &self.camadas[&id];
        (h[i], c[i])
    }
    fn mask(&self, id: LayerId) -> Option<Vec<f32>> {
        self.mascaras.get(&id).cloned()
    }
}

fn grelha() -> AdjustWindow {
    AdjustWindow::full(W, H)
}

/// Uma pincelada: uma lomba com degraus em `x0..x1`, tinta cheia lá dentro.
fn lomba(x0: u32, x1: u32, alto: f32) -> (Vec<f32>, Vec<f32>) {
    let mut h = vec![0.0; N];
    let mut c = vec![0.0; N];
    for y in 0..H {
        for x in x0..x1 {
            let i = (y * W + x) as usize;
            h[i] = alto * (1.0 + (y % 3) as f32);
            c[i] = 1.0;
        }
    }
    (h, c)
}

/// A dobra POR AMOSTRA de sempre (a do `ReliefFields::height_at`).
fn por_amostra(s: &LayerStack, p: &Planos) -> Vec<f32> {
    let ids: Vec<LayerId> = s
        .relief_layers_bottom_up()
        .into_iter()
        .filter(|id| p.has(*id))
        .collect();
    (0..N)
        .map(|i| {
            let cmax = ids
                .iter()
                .map(|&id| p.at(id, i).1.clamp(0.0, 1.0))
                .fold(0.0f32, f32::max);
            let mut h = RELIEF_FOLD_SEED;
            for &id in &ids {
                let l = s.get(id).expect("camada");
                let (a, c) = p.at(id, i);
                h = fold_relief_step(
                    h,
                    a,
                    l.impasto_depth,
                    l.impasto_composite,
                    c.clamp(0.0, 1.0),
                    cmax,
                );
            }
            h
        })
        .collect()
}

fn gaussiano(s: &mut LayerStack, raio: f32) -> LayerId {
    let id = s
        .add_adjustment(AdjustmentKind::GaussianBlur)
        .expect("ajuste");
    s.adjustment_mut(id).expect("ajuste").params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: raio });
    id
}

fn borra(v: &[f32], raio: f32) -> Vec<f32> {
    let mut b = v.to_vec();
    ph2d_painter_effects::adjustments::separable_blur_scalar(raio, &mut b, grelha());
    b
}

/// ⭐ Sem ajuste que aja, a dobra através é a POR AMOSTRA ao bit (`Add`, `Level`, profundidade
/// negativa, uma escondida) — as duas portas dizem o mesmo; o corpo é o máximo das coberturas.
#[test]
fn sem_ajuste_a_dobra_e_a_por_amostra_ao_bit() {
    let mut s = LayerStack::new();
    let a = s.add_raster("a", W, H).expect("a");
    let b = s.add_raster("b", W, H).expect("b");
    let c = s.add_raster("c", W, H).expect("c");
    let d = s.add_raster("d", W, H).expect("d");
    s.set_impasto_composite(b, ReliefComposite::Level);
    s.set_impasto_depth(c, -0.6);
    s.set_visible(d, false);
    let mut p = Planos::default();
    p.camadas.insert(a, lomba(2, 14, 0.5));
    p.camadas.insert(b, lomba(8, 20, 0.3));
    p.camadas.insert(c, lomba(5, 9, 0.7));
    p.camadas.insert(d, lomba(0, 24, 9.0));
    // Um ajuste que não age (o Brilho) não muda o plano a seguir.
    s.add_adjustment(AdjustmentKind::Bloom).expect("brilho");
    let plan = s.relief_plan();
    assert!(!relief_plan_filters(&plan), "o Brilho não age no relevo");
    let (h, corpo) = fold_relief_through(&plan, &p, &grelha()).expect("há relevo");
    let quer = por_amostra(&s, &p);
    assert_eq!(
        h.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        quer.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
    );
    for (i, &got) in corpo.iter().enumerate() {
        let m = [a, b, c]
            .iter()
            .map(|id| p.at(*id, i).1)
            .fold(0.0f32, f32::max);
        assert_eq!(got, m, "o corpo é o máximo das visíveis");
    }
}

/// ⭐⭐ Um Gaussiano borra o relevo das camadas POR BAIXO dele, e só esse: a de cima entra depois,
/// intacta. CONTROLO: sem o ajuste a lomba de baixo tem degraus.
#[test]
fn um_gaussiano_borra_so_o_relevo_por_baixo_dele() {
    let mut s = LayerStack::new();
    let baixo = s.add_raster("baixo", W, H).expect("baixo");
    let g = gaussiano(&mut s, 4.0);
    let cima = s.add_raster("cima", W, H).expect("cima");
    let mut p = Planos::default();
    let (hb, cb) = lomba(2, 10, 0.5);
    let (hc, cc) = lomba(14, 22, 0.5);
    p.camadas.insert(baixo, (hb.clone(), cb.clone()));
    p.camadas.insert(cima, (hc.clone(), cc.clone()));
    let plan = s.relief_plan();
    assert!(relief_plan_filters(&plan));
    let (h, corpo) = fold_relief_through(&plan, &p, &grelha()).expect("relevo");
    let (bh, bc) = (borra(&hb, 4.0), borra(&cb, 4.0));
    for i in 0..N {
        assert!(
            (h[i] - (bh[i] + hc[i])).abs() < 1e-5,
            "amostra {i}: {} contra o de baixo borrado mais o de cima {}",
            h[i],
            bh[i] + hc[i]
        );
        assert!((corpo[i] - bc[i].max(cc[i])).abs() < 1e-6);
    }
    s.set_visible(g, false);
    let (h0, _) = fold_relief_through(&s.relief_plan(), &p, &grelha()).expect("relevo");
    // Nas colunas da camada de BAIXO (a de cima entra intacta, com os degraus dela).
    let degrau = |v: &[f32]| {
        (0..N - W as usize)
            .filter(|i| (*i as u32 % W) < 10)
            .map(|i| (v[i + W as usize] - v[i]).abs())
            .fold(0.0f32, f32::max)
    };
    assert!(
        degrau(&h0) > 2.0 * degrau(&h),
        "CONTROLO: o desfoque amaciou os degraus ({} → {})",
        degrau(&h0),
        degrau(&h)
    );
}

/// A opacidade e a máscara do ajuste pesam-no: metade é a média, a máscara a zero é o relevo de
/// antes; a máscara invertida troca os lados.
#[test]
fn a_opacidade_e_a_mascara_pesam_o_ajuste() {
    let mut s = LayerStack::new();
    let baixo = s.add_raster("baixo", W, H).expect("baixo");
    let g = gaussiano(&mut s, 3.0);
    let mut p = Planos::default();
    let (hb, cb) = lomba(4, 12, 0.5);
    p.camadas.insert(baixo, (hb.clone(), cb));
    s.adjustment_mut(g).expect("g").opacity = 0.5;
    let (h, _) = fold_relief_through(&s.relief_plan(), &p, &grelha()).expect("relevo");
    let bh = borra(&hb, 3.0);
    for i in 0..N {
        assert!((h[i] - 0.5 * (hb[i] + bh[i])).abs() < 1e-5, "amostra {i}");
    }
    s.adjustment_mut(g).expect("g").opacity = 1.0;
    // A pilha só cria máscaras sob uma camada de pintura; o ajuste aponta para ela.
    let portadora = s.add_raster("portadora", W, H).expect("portadora");
    let m = s.add_mask(portadora).expect("máscara");
    s.adjustment_mut(g).expect("g").mask = Some(m.0);
    let metade: Vec<f32> = (0..N)
        .map(|i| if (i as u32 % W) < W / 2 { 0.0 } else { 1.0 })
        .collect();
    p.mascaras.insert(m, metade.clone());
    let (h, _) = fold_relief_through(&s.relief_plan(), &p, &grelha()).expect("relevo");
    for i in 0..N {
        let quer = if metade[i] > 0.0 { bh[i] } else { hb[i] };
        assert!((h[i] - quer).abs() < 1e-5, "amostra {i} com a máscara");
    }
    if let Some(LayerKind::Mask(mk)) = s.get_mut(m).map(|l| &mut l.kind) {
        mk.inverted = true;
    }
    let (h, _) = fold_relief_through(&s.relief_plan(), &p, &grelha()).expect("relevo");
    for i in 0..N {
        let quer = if metade[i] > 0.0 { hb[i] } else { bh[i] };
        assert!(
            (h[i] - quer).abs() < 1e-5,
            "amostra {i} com a máscara invertida"
        );
    }
}

/// ⭐ Um ajuste dentro de um GRUPO borra o que o grupo fez ao relevo, não o de fora — como o
/// compositor faz à cor do grupo. CONTROLO: o mesmo ajuste fora do grupo borra tudo.
#[test]
fn dentro_de_um_grupo_borra_so_o_do_grupo() {
    let mut s = LayerStack::new();
    let fora = s.add_raster("fora", W, H).expect("fora");
    let grupo = s.add_group("grupo").expect("grupo");
    let dentro = s.add_raster("dentro", W, H).expect("dentro");
    assert!(s.move_into_group(dentro, grupo));
    let g = gaussiano(&mut s, 3.0);
    assert!(s.move_into_group(g, grupo));
    assert!(
        matches!(&s.get(grupo).expect("g").kind, LayerKind::Group(GroupLayer { children, .. }) if children.len() == 2)
    );
    let mut p = Planos::default();
    let (hf, cf) = lomba(2, 8, 0.4);
    let (hd, cd) = lomba(12, 20, 0.6);
    p.camadas.insert(fora, (hf.clone(), cf.clone()));
    p.camadas.insert(dentro, (hd.clone(), cd.clone()));
    let (h, corpo) = fold_relief_through(&s.relief_plan(), &p, &grelha()).expect("relevo");
    let (bd, bcd) = (borra(&hd, 3.0), borra(&cd, 3.0));
    for i in 0..N {
        assert!(
            (h[i] - (hf[i] + bd[i])).abs() < 1e-5,
            "amostra {i}: {}",
            h[i]
        );
        assert!((corpo[i] - cf[i].max(bcd[i])).abs() < 1e-6);
    }
    // CONTROLO: fora do grupo, borra as duas.
    let mut s2 = LayerStack::new();
    let f2 = s2.add_raster("fora", W, H).expect("f");
    let d2 = s2.add_raster("dentro", W, H).expect("d");
    gaussiano(&mut s2, 3.0);
    let mut p2 = Planos::default();
    p2.camadas.insert(f2, (hf.clone(), cf));
    p2.camadas.insert(d2, (hd.clone(), cd));
    let (h2, _) = fold_relief_through(&s2.relief_plan(), &p2, &grelha()).expect("relevo");
    let bf = borra(&hf, 3.0);
    let difere = (0..N).filter(|&i| (h2[i] - h[i]).abs() > 1e-4).count();
    assert!(difere > 0 && (0..N).all(|i| (h2[i] - (bf[i] + bd[i])).abs() < 1e-4));
}

/// A Nitidez é a máscara de nitidez sobre a altura: um campo constante fica constante, um degrau
/// ganha contraste; o corpo fica em `0..1`.
#[test]
fn a_nitidez_afia_a_altura_e_o_corpo_fica_em_0_a_1() {
    let mut s = LayerStack::new();
    let baixo = s.add_raster("baixo", W, H).expect("baixo");
    let n = s.add_adjustment(AdjustmentKind::Sharpen).expect("nitidez");
    s.adjustment_mut(n).expect("n").params = AdjustmentParams::Sharpen(SharpenParams {
        amount: 1.5,
        radius: 2.0,
        mask_edges: false,
    });
    let mut p = Planos::default();
    let (hb, cb) = lomba(6, 18, 0.5);
    p.camadas.insert(baixo, (hb.clone(), cb));
    let (h, corpo) = fold_relief_through(&s.relief_plan(), &p, &grelha()).expect("relevo");
    let max0 = hb.iter().copied().fold(0.0f32, f32::max);
    let max1 = h.iter().copied().fold(0.0f32, f32::max);
    assert!(
        max1 > max0 * 1.05,
        "o degrau não ganhou contraste: {max0} → {max1}"
    );
    assert!(corpo.iter().all(|c| (0.0..=1.0).contains(c)));
    let mut p = Planos::default();
    p.camadas.insert(baixo, (vec![0.25; N], vec![1.0; N]));
    let (h, _) = fold_relief_through(&s.relief_plan(), &p, &grelha()).expect("relevo");
    assert!(
        h.iter().all(|x| (x - 0.25).abs() < 1e-6),
        "constante não fica constante"
    );
}

/// O Brilho e as Sombras/Realces são de TOM; o Gaussiano e a Nitidez agem; o Motion não é de
/// vizinhança. Um ajuste escondido, com opacidade `0` ou raio `0` não entra no plano.
#[test]
fn quem_age_no_relevo_e_quem_nao() {
    assert_eq!(
        relief_effect(AdjustmentKind::GaussianBlur),
        ReliefEffect::Acts
    );
    assert_eq!(relief_effect(AdjustmentKind::Sharpen), ReliefEffect::Acts);
    assert_eq!(relief_effect(AdjustmentKind::Bloom), ReliefEffect::Tone);
    assert_eq!(
        relief_effect(AdjustmentKind::ShadowsHighlights),
        ReliefEffect::Tone
    );
    assert_eq!(relief_effect(AdjustmentKind::MotionBlur), ReliefEffect::Not);
    let mut s = LayerStack::new();
    s.add_raster("baixo", W, H).expect("baixo");
    let g = gaussiano(&mut s, 2.0);
    assert!(
        relief_plan_filters(&s.relief_plan()),
        "CONTROLO: o Gaussiano entra"
    );
    s.adjustment_mut(g).expect("g").visible = false;
    assert!(!relief_plan_filters(&s.relief_plan()));
    s.adjustment_mut(g).expect("g").visible = true;
    s.adjustment_mut(g).expect("g").opacity = 0.0;
    assert!(!relief_plan_filters(&s.relief_plan()));
    s.adjustment_mut(g).expect("g").opacity = 1.0;
    s.adjustment_mut(g).expect("g").params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 0.0 });
    assert!(!relief_plan_filters(&s.relief_plan()));
}
