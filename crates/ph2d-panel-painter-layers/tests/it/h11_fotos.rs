//! As fotos e o custo por quadro do fecho do painel: `--ignored`, por nome. Fotos: `PH2D_FOTO_DIR=<pasta>`, `h11_foto`.
//! Custo: `h11_custo_por_quadro`, com as variantes no mesmo processo, intercaladas, e o mínimo por bloco
//! (`docs/DevOps/MEDIR_VELOCIDADE.md`).

use super::h1_os_cartoes::{Mesa, pincel};
use super::h10_fotos::foto;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::FALLBACK_BRUSH;
use ph2d_panel_painter_layers::cards::force_cards_on_this_thread;
use ph2d_panel_painter_layers::state::{
    set_current_brush, set_current_dock_shows_layers, set_current_layers,
};
use ph2d_tool_painter::{
    AdjustmentKind, AdjustmentParams, BrushSettings, LayerKind, LayerStack, PaintMedia,
    TextureKind, TextureLayer,
};
use std::time::Instant;

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 2400.0);

/// Uma pilha com estas espécies, por cima de uma base; as curvas com um ponto, para se ver a pega.
fn pilha(especies: &[AdjustmentKind]) -> LayerStack {
    let mut s = LayerStack::new();
    let _ = s.add_raster("Base", 8, 8).expect("nasce");
    for k in especies {
        let a = s.add_adjustment(*k).expect("o ajuste nasce");
        if let Some(LayerKind::Adjustment(adj)) = s.get_mut(a).map(|l| &mut l.kind)
            && let AdjustmentParams::Curves(c) = &mut adj.params
        {
            c.points_rgb.points.push([0.5, 0.3]);
        }
    }
    s
}

fn com_textura() -> LayerStack {
    let mut s = pilha(&[]);
    let spec = TextureLayer {
        kind: TextureKind::COUNT - 2,
        ramp_enabled: true,
        ..TextureLayer::default()
    };
    let t = s.add_texture(spec).expect("a textura nasce");
    s.set_active(t);
    s
}

#[test]
#[ignore = "fotos: PH2D_FOTO_DIR=<pasta> e --ignored"]
fn h11_foto() {
    use AdjustmentKind as K;
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    let camadas = [
        (
            "h11_curva_levels_degrade",
            true,
            pilha(&[K::Curves, K::Levels, K::GradientMap]),
        ),
        (
            "h11_mixer_seletiva",
            true,
            pilha(&[K::ChannelMixer, K::SelectiveColor]),
        ),
        (
            "h11_pb_balanco_brilho",
            true,
            pilha(&[K::BlackAndWhite, K::ColorBalance, K::Bloom]),
        ),
        ("h11_textura", true, com_textura()),
        (
            "h11_antes_ajustes",
            false,
            pilha(&[K::Curves, K::Levels, K::GradientMap]),
        ),
    ];
    for (nome, cartoes, s) in camadas {
        let mut m = Mesa::na_janela(cartoes, FALLBACK_BRUSH, true, JANELA);
        set_current_layers(Some(s));
        m.assenta();
        foto(&gpu, &dir, nome, &mut m);
        set_current_layers(None);
    }
    let mut m = Mesa::na_janela(false, pincel(PaintMedia::Digital, "brush").0, false, JANELA);
    foto(&gpu, &dir, "h11_antes_pincel", &mut m);
}

struct Variante {
    nome: &'static str,
    cartoes: bool,
    brush: BrushSettings,
    camadas: Option<LayerStack>,
    mesa: Mesa,
    minimo: f64,
}

impl Variante {
    fn nova(
        nome: &'static str,
        cartoes: bool,
        brush: BrushSettings,
        camadas: Option<LayerStack>,
    ) -> Self {
        let mut mesa = Mesa::na_janela(cartoes, brush, camadas.is_some(), JANELA);
        set_current_layers(camadas.clone());
        mesa.assenta();
        Self {
            nome,
            cartoes,
            brush,
            camadas,
            mesa,
            minimo: f64::MAX,
        }
    }

    fn bloco(&mut self, quadros: u32) {
        force_cards_on_this_thread(Some(self.cartoes));
        set_current_brush(Some(self.brush));
        set_current_dock_shows_layers(self.camadas.is_some());
        set_current_layers(self.camadas.clone());
        let t = Instant::now();
        for _ in 0..quadros {
            self.mesa.quadro();
        }
        let us = t.elapsed().as_secs_f64() * 1e6 / f64::from(quadros);
        self.minimo = self.minimo.min(us);
    }
}

/// O custo do painel por quadro, cartões × `PH2D_CARDS=0`. A pilha é a mesma nas duas caras.
#[test]
#[ignore = "medida: --ignored --nocapture, perfil smoke"]
fn h11_custo_por_quadro() {
    use AdjustmentKind as K;
    let cheia = pilha(&[K::Curves, K::Levels, K::GradientMap, K::ChannelMixer]);
    let molhado = pincel(PaintMedia::WetPaint, "brush").0;
    let digital = pincel(PaintMedia::Digital, "brush").0;
    let mut v = vec![
        Variante::nova("pincel digital, cartões", true, digital, None),
        Variante::nova("pincel digital, antes", false, digital, None),
        Variante::nova("pincel molhado, cartões", true, molhado, None),
        Variante::nova("pincel molhado, antes", false, molhado, None),
        Variante::nova(
            "4 ajustes, cartões",
            true,
            FALLBACK_BRUSH,
            Some(cheia.clone()),
        ),
        Variante::nova("4 ajustes, antes", false, FALLBACK_BRUSH, Some(cheia)),
        Variante::nova(
            "camada Texture, cartões",
            true,
            FALLBACK_BRUSH,
            Some(com_textura()),
        ),
        Variante::nova(
            "camada Texture, antes",
            false,
            FALLBACK_BRUSH,
            Some(com_textura()),
        ),
    ];
    let n = v.len();
    for bloco in 0..24 {
        for i in 0..n {
            v[(i + bloco) % n].bloco(10);
        }
    }
    set_current_layers(None);
    for x in &v {
        println!("custo {}: {:.0} us/quadro (mínimo)", x.nome, x.minimo);
    }
}
