//! Os gates da **Shape Color Ramp na aquarela** (doc 46 §2-7) — a rampa entra no splat da COR
//! (`watercolor_accum_cor::RampaDaAguada`) e, no modo B&W, no peso das DUAS passadas.

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

/// Uma aguada azul de `(20,64)` a `(108,64)`, com a rampa como `arma` a deixar.
fn aguada_t(arma: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = tool(128, PaintMedia::Watercolor, 10.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    arma(&mut t);
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    for k in 1..=22 {
        #[allow(clippy::cast_precision_loss)]
        let x = 20.0 + 4.0 * k as f32;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
    t
}

fn aguada(arma: impl Fn(&mut PainterTool)) -> Vec<u8> {
    aguada_t(arma).canvas_rgba.to_vec()
}

fn rampa(cor: [f32; 4]) -> ColorRamp {
    ColorRamp::new(
        vec![RampStop::new(0.0, cor), RampStop::new(1.0, cor)],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

/// Texels onde o vermelho domina o azul por uma margem visível.
fn avermelhados(px: &[u8]) -> usize {
    px.chunks(4)
        .filter(|p| i32::from(p[0]) > i32::from(p[2]) + 40)
        .count()
}

/// **A RAMPA É A DONA DA COR NA AGUADA** — uma rampa vermelha sobre um pincel azul pinta uma aguada
/// vermelha (a lei do traço digital, `shape_colour_ramp_colourises_the_silhouette_when_grain_is_none`).
#[test]
fn a_rampa_da_shape_pinta_a_cor_da_aguada() {
    let sem = aguada(|_| {});
    // Sem textura no Shape a cor é da Falloff Color (prompt 38); a Texture Color não tem o que colorir.
    let so_textura = aguada(|t| {
        t.set_shape_color_ramp(rampa([1.0, 0.0, 0.0, 1.0]));
        t.set_shape_ramp_enabled(true);
        t.paint.shape_camadas.queda = false;
        t.set_shape_falloff_ramp(rampa([1.0, 0.0, 0.0, 1.0]));
    });
    assert_eq!(
        avermelhados(&so_textura),
        0,
        "sem textura no Shape a Texture Color não pode pintar a aguada"
    );
    let com = aguada(|t| {
        t.set_shape_falloff_ramp(rampa([1.0, 0.0, 0.0, 1.0]));
        t.set_shape_ramp_enabled(true);
        t.paint.shape_camadas.textura = false;
        t.paint.shape_camadas.queda = true;
        t.set_shape_falloff_ramp(rampa([1.0, 0.0, 0.0, 1.0]));
    });
    assert_eq!(
        avermelhados(&sem),
        0,
        "controlo: a aguada azul não tem vermelho"
    );
    let n = avermelhados(&com);
    assert!(
        n > 300,
        "a rampa vermelha não chegou à aguada ({n} texels vermelhos)"
    );
}

/// **No modo B&W a rampa é o TOM da silhueta da Shape** — a lei do traço digital (`stamp.rs`,
/// `remap_shape_value` sobre o valor CRU): com uma Shape procedural na aguada (Automatic desligado) o
/// valor dela é a DENSIDADE da ponta, e uma rampa de cinzento escuro baixa-a — a aguada sai mais
/// CLARA, na cor do pincel. Sem Shape o tom não tem silhueta a remapear, como no digital.
#[test]
fn o_tom_da_rampa_pesa_a_silhueta_da_shape() {
    let soma = |px: &[u8]| {
        px.chunks(4)
            .map(|p| u64::from(p[0]) + u64::from(p[1]))
            .sum::<u64>()
    };
    let shape = |t: &mut PainterTool| {
        t.paint.brush.watercolor_shape_auto = false;
        t.paint.brush.shape.kind = ph2d_painter_brush::TextureKind::Noise;
    };
    let tom = |t: &mut PainterTool| {
        t.set_shape_color_ramp(rampa([0.05, 0.05, 0.05, 1.0]));
        t.set_shape_ramp_enabled(true);
        t.paint.shape_color_ramp_bw = true;
    };
    let sem_t = aguada_t(shape);
    let com_t = aguada_t(|t| {
        shape(t);
        tom(t);
    });
    assert_ne!(
        sem_t.paint.stroke_density, com_t.paint.stroke_density,
        "o tom não chegou à densidade da ponta"
    );
    let (sem, com) = (sem_t.canvas_rgba.to_vec(), com_t.canvas_rgba.to_vec());
    assert_eq!(
        avermelhados(&com),
        0,
        "o tom não pode trocar a cor do pincel"
    );
    assert!(
        soma(&com) > soma(&sem) + 20_000,
        "o tom escuro não aclarou a aguada ({} contra {})",
        soma(&com),
        soma(&sem)
    );
    assert_eq!(
        aguada(|_| {}),
        aguada(tom),
        "sem Shape o tom remapeou alguma coisa — no digital ele não age sem silhueta"
    );
}

/// **O tom vale já no PRIMEIRO carimbo depois de mexer na rampa** — com uma ponta de IMAGEM o valor
/// da Shape é a própria cobertura, e a passada da cobertura corre ANTES da da cor: se só a da cor
/// garantisse a LUT do tom, o 1.º lote leria a LUT velha e a cobertura discordaria da cor.
#[test]
fn o_tom_vale_no_primeiro_carimbo_de_uma_ponta_de_imagem() {
    let pinga = |tom: bool| {
        let mut t = tool(64, PaintMedia::Watercolor, 12.0);
        t.paint.brush.watercolor_shape_auto = false;
        // Uma rampa de luminância a meio-tom: a ponta de imagem satura o miolo, o tom puxa-o para baixo.
        let mut lum = vec![0u8; 16 * 16];
        for (i, v) in lum.iter_mut().enumerate() {
            let (x, y) = (i % 16, i / 16);
            *v = if (4..12).contains(&x) && (4..12).contains(&y) {
                255
            } else {
                40
            };
        }
        t.set_brush_shape_image(lum, 16, 16);
        if tom {
            t.set_shape_color_ramp(rampa([0.02, 0.02, 0.02, 1.0]));
            t.set_shape_ramp_enabled(true);
            t.paint.shape_color_ramp_bw = true;
        }
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.paint.stroke_coverage.clone()
    };
    let (sem, com) = (pinga(false), pinga(true));
    assert!(
        sem.iter().any(|&c| c > 0),
        "controlo: a ponta de imagem molha o papel"
    );
    assert_ne!(
        sem, com,
        "o 1.º carimbo leu a LUT do tom velha — a cobertura ignorou o tom"
    );
}

/// O pincel do gate A8: Noise no Shape (View, Size 1,5), pincel vermelho, rampa azul → verde na
/// camada pedida (`None` = sem rampa), e `arma` (o campo varrido, pela PORTA do painel).
fn pincel_a8(camada: Option<bool>, arma: &dyn Fn(&mut PainterTool)) -> PainterTool {
    let mut t = arma_a8(papel_a8(), 30.0, camada, arma);
    t.on_canvas_pointer(cp([48.5, 48.5], PointerPhase::Down));
    t.on_canvas_pointer(cp([48.5, 48.5], PointerPhase::Up));
    t
}

fn papel_a8() -> PainterTool {
    const W: u32 = 96;
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (W * W * 4) as usize], W, W);
    t.set_paint_media(PaintMedia::Watercolor);
    t
}

/// Arma o pincel do A8 (de raio `raio`) sobre `t`, que pode já ter tinta.
fn arma_a8(
    mut t: PainterTool,
    raio: f32,
    camada: Option<bool>,
    arma: &dyn Fn(&mut PainterTool),
) -> PainterTool {
    use ph2d_painter_brush::texture::{TextureKind, TextureMapping};
    let azul_verde = ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 0.0, 1.0, 1.0]),
            RampStop::new(1.0, [0.0, 1.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    );
    let b = &mut t.paint.brush;
    b.radius_px = raio;
    b.color = [1.0, 0.0, 0.0];
    b.space_attenuation = false;
    b.watercolor_shape_auto = false;
    (b.falloff, b.hardness) = (ph2d_painter_brush::Falloff::Constant, 1.0);
    t.set_brush_shape_kind(TextureKind::Noise.to_u8());
    t.paint.brush.shape.mapping = TextureMapping::ViewPlane;
    t.paint.brush.shape.size = [1.5; 2];
    t.set_shape_color_ramp(azul_verde.clone());
    t.set_shape_ramp_enabled(camada.is_some());
    t.paint.shape_camadas.textura = camada == Some(true);
    t.paint.shape_camadas.queda = camada == Some(false);
    t.set_shape_falloff_ramp(azul_verde); // re-assa as LUTs
    arma(&mut t);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t
}

/// **A RAMPA LEVA A COR QUE A BROCHA APANHOU** (lista 26 A8, a metade aberta): sobre tinta que já está
/// no papel (molhada da sessão ou seca), a `Charge < 1` mistura a cor apanhada na cor do dab. A rampa
/// troca a cor do PINCEL — e é ela que se mistura, como a do pincel (`Apanha::mistura`). Antes, a rampa
/// trocava a cor do dab JÁ misturada: `|Δrgb|` médio do buffer da cor entre Charge `0,5` e `1`, sobre
/// uma poça amarela, era `0,2` (Texture) e `0,0` (Falloff) contra `9,1` sem rampa.
#[test]
fn a_rampa_leva_a_cor_que_a_brocha_apanhou() {
    let traco = |camada: Option<bool>, charge: f64, seca: bool| {
        let mut t = papel_a8();
        t.paint.brush.radius_px = 22.0;
        t.paint.brush.color = [1.0, 0.85, 0.0];
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        t.on_canvas_pointer(cp([26.0, 48.5], PointerPhase::Down));
        t.on_canvas_pointer(cp([26.0, 48.5], PointerPhase::Up));
        if seca {
            t.dry_session_now();
        }
        let poca: Vec<bool> = t.canvas_rgba.chunks(4).map(|p| p[3] > 60).collect();
        let arma = |t: &mut PainterTool| t.set_brush_wet_charge(charge as f32);
        let mut t = arma_a8(t, 14.0, camada, &arma);
        t.on_canvas_pointer(cp([4.0, 48.5], PointerPhase::Down));
        for k in 1..=22 {
            #[allow(clippy::cast_precision_loss)]
            t.on_canvas_pointer(cp([4.0 + 4.0 * k as f32, 48.5], PointerPhase::Move));
        }
        (t.paint.stroke_color.clone(), poca)
    };
    let mut leituras = Vec::new();
    for seca in [false, true] {
        for camada in [None, Some(true), Some(false)] {
            let ((cheia, poca), (meia, _)) = (traco(camada, 1.0, seca), traco(camada, 0.5, seca));
            let (mut s, mut n) = (0.0f64, 0u32);
            for ((a, b), _) in cheia
                .chunks(4)
                .zip(meia.chunks(4))
                .zip(&poca)
                .filter(|((a, b), p)| **p && a[3] >= 13 && b[3] >= 13)
            {
                s += (0..3).map(|c| f64::from(a[c].abs_diff(b[c]))).sum::<f64>() / 3.0;
                n += 1;
            }
            leituras.push((seca, camada, s / f64::from(n.max(1)), n));
        }
    }
    let fracas: Vec<String> = leituras
        .iter()
        .filter(|l| l.3 < 300 || l.2 < 4.0)
        .map(|(seca, camada, d, n)| {
            let poca = if *seca { "seca" } else { "molhada" };
            format!("poça {poca}, rampa {camada:?}: |Δrgb| {d:.1} em {n} texels")
        })
        .collect();
    assert!(
        fracas.is_empty(),
        "a apanha não tinge o depósito:\n{}",
        fracas.join("\n")
    );
}

/// Os valores fora do padrão de CADA campo da aquarela — `None` = um campo novo sem valores, e o
/// gate reprova (a tabela tem de crescer com a lista).
fn valores_fora_do_padrao(id: ph2d_a11y::NodeId) -> Option<&'static [f64]> {
    use crate::ids as i;
    let v: &'static [f64] = match id {
        x if x == i::PAINTER_WATERCOLOR_DRY_TIME => &[2.0, 60.0],
        x if x == i::PAINTER_WATERCOLOR_WET_PREVIEW => &[0.0, 1.0],
        x if x == i::PAINTER_WATERCOLOR_EDGE => &[0.0, 6.0],
        x if x == i::PAINTER_WATERCOLOR_SPREAD => &[1.0, 30.0],
        x if x == i::PAINTER_WATERCOLOR_GRANULATION => &[0.0, 1.0],
        x if x == i::PAINTER_WATERCOLOR_MIX => &[0.5, 1.0],
        x if x == i::PAINTER_WATERCOLOR_FILL => &[0.0, 1.0],
        x if x == i::PAINTER_WATERCOLOR_DEPTH => &[0.3, 4.0],
        x if x == i::PAINTER_WATERCOLOR_OPACITY => &[0.0, 1.0],
        x if x == i::PAINTER_WATERCOLOR_WARP => &[0.0, 20.0],
        x if x == i::PAINTER_WATERCOLOR_FLOW_SIZE => &[0.5, 4.0],
        x if x == i::PAINTER_WATERCOLOR_FLOW_ANGLE => &[45.0, 90.0],
        x if x == i::PAINTER_WATERCOLOR_PAPER_EDGE => &[0.5, 1.0],
        x if x == i::PAINTER_WATERCOLOR_SMUDGE => &[0.5, 1.0],
        x if x == i::PAINTER_WATERCOLOR_WET => &[0.5, 1.0],
        // O A8: QUALQUER Charge < 1 ligava o mixer e zerava a prioridade sobre o papel em branco.
        x if x == i::PAINTER_WATERCOLOR_CHARGE => &[0.99, 0.9, 0.5, 0.0],
        x if x == i::PAINTER_WATERCOLOR_DILUTION => &[0.5, 1.0],
        x if x == i::PAINTER_WATERCOLOR_PULL => &[0.5, 1.0],
        x if x == i::PAINTER_WATERCOLOR_PAPER_SIZE_X || x == i::PAINTER_WATERCOLOR_PAPER_SIZE_Y => {
            &[0.5, 3.0]
        }
        x if x == i::PAINTER_WATERCOLOR_PAPER_ANGLE => &[30.0, 90.0],
        x if x == i::PAINTER_WATERCOLOR_PAPER_OFFSET_X
            || x == i::PAINTER_WATERCOLOR_PAPER_OFFSET_Y =>
        {
            &[0.25, 0.75]
        }
        x if x == i::PAINTER_WATERCOLOR_PAPER_DEPTH => &[0.0, 2.0],
        x if i::PAINTER_WATERCOLOR_PAPER_PARAMS.contains(&x) => &[0.2, 0.8],
        _ => return None,
    };
    Some(v)
}

/// A fração da cor que NÃO é a do pincel (vermelho) sobre os texels pintados — `None` = nada pintado.
fn fracao_da_rampa(px: &[u8]) -> Option<f32> {
    let (mut s, mut n) = (0.0, 0u32);
    for p in px.chunks(4).filter(|p| p[3] >= 13) {
        let (r, g, b) = (f32::from(p[0]), f32::from(p[1]), f32::from(p[2]));
        s += 1.0 - r / (r + g + b).max(1.0);
        n += 1;
    }
    (n > 0).then(|| s / n as f32)
}

/// **A RAMPA DA AGUADA SOBREVIVE A CADA CAMPO DO PAINEL** (lista 26 A8, smoke do dono 15/10): com a
/// Texture Color ou a Falloff Color do Shape ligadas, nenhum campo da aquarela, a nenhum valor, pode
/// devolver a tela SEM rampa. O A8: a `Charge < 1` liga o mixer, a prioridade do dab vira
/// `pickup × w` e sobre o papel em branco `w = 0` — o buffer da cor ficava vazio e o composto caía na
/// cor do pincel. Dilution `1` não pinta com nem sem rampa: é o único caso em branco admitido.
#[test]
fn a_rampa_da_aguada_sobrevive_a_cada_campo_do_painel() {
    use crate::ids as i;
    use ph2d_editor_core::tool::PanelEvent;
    let sem_valores = i::PAINTER_WATERCOLOR_FIELDS
        .iter()
        .filter(|id| valores_fora_do_padrao(**id).is_none())
        .count();
    assert_eq!(
        sem_valores, 0,
        "campos da aquarela sem valores na tabela do gate"
    );
    let (mut casos, mut falhas, mut em_branco) = (0, Vec::new(), 0);
    for id in i::PAINTER_WATERCOLOR_FIELDS {
        for &v in valores_fora_do_padrao(id).unwrap_or(&[]) {
            let arma = |t: &mut PainterTool| {
                assert!(t.route_brush_watercolor_event(&PanelEvent::SetValue(id, v)));
            };
            for textura in [true, false] {
                casos += 1;
                let com = pincel_a8(Some(textura), &arma).canvas_rgba.to_vec();
                let sem = pincel_a8(None, &arma).canvas_rgba.to_vec();
                match (fracao_da_rampa(&com), fracao_da_rampa(&sem)) {
                    (None, None) if id == i::PAINTER_WATERCOLOR_DILUTION && v == 1.0 => {
                        em_branco += 1;
                    }
                    (Some(c), Some(s)) if com != sem && c > s + 0.15 => {}
                    (c, s) => falhas.push(format!(
                        "{id:?}={v} {}: rampa {c:.3?} × sem {s:.3?}{}",
                        if textura { "Texture" } else { "Falloff" },
                        if com == sem { " (IGUAL AO BYTE)" } else { "" }
                    )),
                }
            }
        }
    }
    assert_eq!(
        em_branco, 2,
        "a Dilution 1 deixou de ser o único caso em branco"
    );
    assert!(
        falhas.is_empty(),
        "{} de {casos} casos perderam a rampa:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// Um carimbo (`traco = false`) ou um traço curto de Voronoi na aguada, Automatic desligado, pincel
/// azul; `camadas` = `(Texture, Falloff)` da Shape Color com a rampa verde → vermelho (`None` = sem
/// rampa). Devolve a ferramenta com o buffer da cor (a prévia, `rampa(v)`) e a tela.
fn voronoi_p39(camadas: Option<(bool, bool)>, traco: bool) -> (PainterTool, Vec<u8>) {
    use ph2d_painter_brush::texture::TextureKind;
    let verde_vermelho = ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    );
    let mut t = tool(128, PaintMedia::Watercolor, 40.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.toggle_brush_watercolor_shape_auto();
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    if let Some((textura, queda)) = camadas {
        t.set_shape_color_ramp(verde_vermelho.clone());
        t.set_shape_falloff_ramp(verde_vermelho);
        t.set_shape_ramp_enabled(true);
        t.paint.shape_camadas.textura = textura;
        t.paint.shape_camadas.queda = queda;
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    for k in 1..=if traco { 4 } else { 0 } {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(cp([64.0 + 2.0 * k as f32, 64.0], PointerPhase::Move));
    }
    let cor = t.paint.stroke_color.clone();
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Up));
    (t, cor)
}

/// **O PADRÃO QUE A TEXTURE COLOR RECOLHE EM COR NÃO RAREIA O PIGMENTO DA AGUADA** (2.º smoke do dono,
/// 15/10, P39: Voronoi + Shape Color, a prévia verde no miolo das células e a tela creme/branca ali).
/// O valor do Voronoi indexava a rampa E era a densidade da ponta: o miolo (`v ≈ 0`, a ponta verde da
/// rampa) não tinha pigmento. A lei do digital (prompt 38): o padrão vira cor e deixa de recortar — na
/// aguada, deixa de ser a densidade. Medido antes da cura (dab · traço): G−R da tela onde a prévia é
/// verde `6,3 · 7,0`, escuro `12 · 17`, correlação prévia × tela `0,38 · 0,35`; depois `30 · 27`,
/// `47 · 53`, `0,94 · 0,65`.
#[test]
fn o_padrao_que_vira_cor_nao_rareia_o_pigmento_da_aguada() {
    for traco in [false, true] {
        let (t, cor) = voronoi_p39(Some((true, true)), traco);
        let tela = &t.canvas_rgba;
        let (mut n, mut gr, mut escuro) = (0u32, 0.0f64, 0.0f64);
        let (mut sx, mut sy, mut sxx, mut syy, mut sxy, mut m) = (0.0f64, 0.0, 0.0, 0.0, 0.0, 0.0);
        for (c, p) in cor
            .chunks(4)
            .zip(tela.chunks(4))
            .filter(|(c, _)| c[3] >= 128)
        {
            let x = f64::from(c[1]) - f64::from(c[0]);
            let y = f64::from(p[1]) - f64::from(p[0]);
            if x.abs() <= 60.0 {
                continue; // nem verde nem vermelho na prévia
            }
            (sx, sy, sxx, syy, sxy, m) = (
                sx + x,
                sy + y,
                sxx + x * x,
                syy + y * y,
                sxy + x * y,
                m + 1.0,
            );
            if x > 60.0 {
                n += 1;
                gr += y;
                escuro += 255.0 - p[..3].iter().map(|&v| f64::from(v)).sum::<f64>() / 3.0;
            }
        }
        let corr =
            (m * sxy - sx * sy) / ((m * sxx - sx * sx) * (m * syy - sy * sy)).sqrt().max(1e-9);
        let (gr, escuro) = (gr / f64::from(n.max(1)), escuro / f64::from(n.max(1)));
        assert!(
            n >= 300,
            "controlo: a prévia tem miolos verdes ({n} texels)"
        );
        assert!(
            gr >= 18.0 && escuro >= 30.0 && corr >= 0.55,
            "traço {traco}: onde a prévia é verde a tela não é (G−R {gr:.1}, escuro {escuro:.1}, \
             correlação {corr:.3})"
        );
        let cheia = t
            .paint
            .stroke_coverage
            .iter()
            .zip(&t.paint.stroke_density)
            .all(|(&c, &d)| c == 0 || d == 255);
        assert!(
            cheia,
            "o padrão recolhido em cor ainda é a densidade da ponta"
        );
    }
    // Controlos: sem a Texture Color (só a Falloff, ou sem rampa) o padrão CONTINUA a densidade.
    for camadas in [Some((false, true)), None] {
        let (t, _) = voronoi_p39(camadas, false);
        let rala = t
            .paint
            .stroke_coverage
            .iter()
            .zip(&t.paint.stroke_density)
            .filter(|&(&c, &d)| c > 128 && d < 128)
            .count();
        assert!(
            rala > 300,
            "{camadas:?}: o Voronoi deixou de ser a densidade sem a Texture Color ({rala} texels ralos)"
        );
    }
}

/// O pincel azul do P39b, a aguada de um carimbo em `(64, 64)` de raio 40 armada por `arma`.
fn aguada_p39b(arma: &dyn Fn(&mut PainterTool)) -> (PainterTool, Vec<u8>) {
    let mut t = tool(128, PaintMedia::Watercolor, 40.0);
    t.set_brush_color_srgb8(AZUL_P39B);
    arma(&mut t);
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    let cor = t.paint.stroke_color.clone();
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Up));
    (t, cor)
}

const AZUL_P39B: [u8; 3] = [30, 60, 220];

/// A rampa da Falloff Color de UMA cor (sRGB em bytes) com alfa `a` — Automatic ligado, sem Shape.
fn queda_lisa(rgb: [u8; 3], a: f32) -> impl Fn(&mut PainterTool) {
    let lin = rgb.map(|c| ((f32::from(c) / 255.0 + 0.055) / 1.055).powf(2.4));
    move |t: &mut PainterTool| {
        t.set_shape_falloff_ramp(rampa([lin[0], lin[1], lin[2], a]));
        t.set_shape_ramp_enabled(true);
    }
}

/// A absorvância de um texel (`−ln`, por canal) normalizada — a DIRECÇÃO da cor de um pigmento.
fn absorvancia(p: &[u8]) -> [f64; 3] {
    let a: [f64; 3] = core::array::from_fn(|c| -(f64::from(p[c].max(1)) / 255.0).ln());
    let n = a.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-9);
    a.map(|v| v / n)
}

fn cosseno(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(&b).map(|(x, y)| x * y).sum()
}

/// No anel `dn ∈ [0,7; 0,95]` do carimbo: (fração dos texels pintados cuja cor está mais perto da do
/// buffer — a rampa — que da do pincel, tinta média = `255 − lum`).
fn anel_p39b(t: &PainterTool, cor: &[u8]) -> (f64, f64) {
    let pincel = absorvancia(&AZUL_P39B);
    let (mut perto, mut n, mut tinta) = (0u32, 0u32, 0.0f64);
    for (i, p) in t.canvas_rgba.chunks(4).enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let (x, y) = ((i % 128) as f32 + 0.5 - 64.0, (i / 128) as f32 + 0.5 - 64.0);
        let dn = (x * x + y * y).sqrt() / 40.0;
        let lum = p[..3].iter().map(|&v| f64::from(v)).sum::<f64>() / 3.0;
        if !(0.7..=0.95).contains(&dn) || lum > 250.0 || cor[i * 4 + 3] == 0 {
            continue;
        }
        let (c, rampa) = (absorvancia(p), absorvancia(&cor[i * 4..i * 4 + 3]));
        n += 1;
        tinta += 255.0 - lum;
        if cosseno(c, rampa) > cosseno(c, pincel) {
            perto += 1;
        }
    }
    (
        f64::from(perto) / f64::from(n.max(1)),
        tinta / f64::from(n.max(1)),
    )
}

/// **A BORDA ESCURECIDA DA AGUADA SAI NA COR DA RAMPA** (2.º smoke do dono, P39: o aro laranja era a
/// cor do pincel). O buffer pousa a cor da rampa com alfa = peso × alfa da cor, e o composto puxava a
/// cor para a do pincel por esse alfa — na borda o peso é baixo. Com a rampa dona da cor
/// (`WetStrokeStyle::deposito_dono`) o pigmento é o do buffer onde ele tem tinta.
#[test]
fn a_borda_da_aguada_sai_na_cor_da_rampa() {
    use ph2d_painter_brush::texture::TextureKind;
    let voronoi = |t: &mut PainterTool| {
        t.toggle_brush_watercolor_shape_auto();
        t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
        let vv = ColorRamp::new(
            vec![
                RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
                RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        );
        t.set_shape_color_ramp(vv.clone());
        t.set_shape_falloff_ramp(vv);
        t.set_shape_ramp_enabled(true);
    };
    let laranja = queda_lisa([230, 120, 20], 1.0);
    for (nome, arma) in [
        ("Voronoi", &voronoi as &dyn Fn(&mut PainterTool)),
        ("Falloff laranja", &laranja),
    ] {
        let (t, cor) = aguada_p39b(arma);
        let (perto, _) = anel_p39b(&t, &cor);
        assert!(
            perto >= 0.9,
            "{nome}: só {perto:.3} do anel tem a cor da rampa — o resto puxa para o pincel"
        );
    }
    // Controlo: a rampa da COR DO PINCEL é a aguada sem rampa — a força da borda não muda (±5 %).
    let (sem, cs) = aguada_p39b(&|_| {});
    let (com, cc) = aguada_p39b(&queda_lisa(AZUL_P39B, 1.0));
    let ((_, ts), (_, tc)) = (anel_p39b(&sem, &cs), anel_p39b(&com, &cc));
    assert!(
        (tc / ts - 1.0).abs() <= 0.05,
        "a rampa da cor do pincel mudou a tinta da borda: {tc:.1} contra {ts:.1}"
    );
}

/// **UMA PARAGEM TRANSPARENTE DA RAMPA PINTA MENOS TINTA** — no digital o alfa da cor escala a
/// cobertura; na aguada escala a densidade do pigmento, e a cor continua a da rampa (antes o alfa
/// baixo puxava a cor para a do pincel e a tinta ficava igual).
#[test]
fn a_paragem_transparente_da_rampa_pinta_menos_tinta() {
    let tinta = |t: &PainterTool| {
        t.canvas_rgba
            .chunks(4)
            .map(|p| 255.0 - p[..3].iter().map(|&v| f64::from(v)).sum::<f64>() / 3.0)
            .sum::<f64>()
    };
    let verde = [40, 200, 60];
    let (opaca, _) = aguada_p39b(&queda_lisa(verde, 1.0));
    let (rala, cor) = aguada_p39b(&queda_lisa(verde, 0.3));
    let r = tinta(&rala) / tinta(&opaca);
    assert!(r <= 0.6, "o alfa 0,3 da rampa deixou {r:.3} da tinta");
    let (perto, _) = anel_p39b(&rala, &cor);
    assert!(
        perto >= 0.9,
        "a paragem transparente pinta a cor do pincel ({perto:.3})"
    );
}
