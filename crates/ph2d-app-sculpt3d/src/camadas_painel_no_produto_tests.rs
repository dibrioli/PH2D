//! ⭐⭐⭐⭐ **O PAINEL DE LAYERS SOBRE A PEÇA, PELO CAMINHO DO PRODUTO** (`docs/3D/30` §4 — a W3) —
//! irmão (`#[path]`) do [`super`], com o arnês dele. Pede a placa (`#[ignore]`).
//!
//! A corrente inteira, sem atalho: o painel REAL pintado (`MockPanelHost::paint`), o clique e o
//! arrasto REAIS sobre o que ele registou (`click_at` / `drag_at` → despacho → `apply_event` →
//! barramento), o barramento entregue ao Painter (`handle_panel_event`, como a shell), o quadro da
//! costura (`painter_na_malha::quadro`, que drena os pedidos para a porta da pilha) e o `Ctrl+Z`
//! pela tecla da escultura.

use super::{amostras, cena_52, tecla};
use crate::painter_na_malha::{entrega, quadro};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::{PointerPhase, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::PainterLayersPanelState;
use ph2d_tool_painter::ids::{PAINTER_LAYERS_ADD, PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{LayerId, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

/// O painel de Layers, pintado e conduzido como no app.
struct Painel {
    host: MockPanelHost,
    st: PainterLayersPanelState,
}

impl Painel {
    fn novo() -> Self {
        Self {
            host: MockPanelHost::with_panel_and_shared_chrome::<PainterLayersPanel>(),
            st: PainterLayersPanelState,
        }
    }

    /// O que a ponte do Painter publica a cada quadro, e a pintura do painel.
    fn pinta(&mut self, p: &PainterTool) -> Vec<(NodeId, Rect)> {
        use ph2d_panel_painter_layers as pl;
        pl::set_current_dock_shows_layers(true);
        pl::set_current_layers(p.panel_layers().cloned());
        pl::set_current_layers_on_piece(p.panel_shows_the_piece());
        pl::set_current_piece_refusal(p.piece_layer_refusal().map(str::to_owned));
        pl::set_current_selection(p.panel_selection());
        self.host
            .paint::<PainterLayersPanel>(&mut self.st, Rect::new(0.0, 0.0, 1600.0, 900.0))
    }

    /// O rectângulo que a pintura registou para `id`.
    fn onde(&mut self, p: &PainterTool, id: NodeId) -> Rect {
        self.pinta(p)
            .into_iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| r)
            .unwrap_or_else(|| panic!("{id:?} não está no ecrã do painel"))
    }

    /// O gesto atravessa o painel e o barramento chega ao Painter (o que a shell faz).
    fn entrega(&mut self, p: &mut PainterTool, eventos: Vec<WidgetEvent>) {
        assert!(!eventos.is_empty(), "o gesto não produziu evento nenhum");
        for ev in eventos {
            self.host
                .apply_panel_event::<PainterLayersPanel>(&mut self.st, ev);
        }
        let mut chegou = false;
        for a in self.host.drained_actions() {
            if let EditorAction::ToolPanelEvent(e) = a {
                p.handle_panel_event(e);
                chegou = true;
            }
        }
        assert!(chegou, "o painel não pôs nada no barramento");
    }

    fn clica(&mut self, p: &mut PainterTool, id: NodeId) {
        let r = self.onde(p, id);
        let ev = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.entrega(p, ev);
    }

    fn arrasta(&mut self, p: &mut PainterTool, id: NodeId, de: f32, para: f32) {
        let r = self.onde(p, id);
        let y = r.y + r.h * 0.5;
        let ev = self.host.drag_at(r.x + r.w * de, y, r.x + r.w * para, y);
        self.entrega(p, ev);
    }
}

fn painter_vermelho() -> PainterTool {
    let mut p = PainterTool::default();
    p.set_brush_color_srgb8([255, 0, 0]);
    p.set_brush_strength(1.0);
    p.set_brush_size_px(24.0);
    p
}

/// Um traço do Painter pelo caminho da shell, com quadros a meio.
fn traco(s: &mut crate::Sculpt3dScene, p: &mut PainterTool, x0: f32) {
    assert!(
        entrega(s, p, x0, 350.0, 1.0, PointerPhase::Down),
        "o pen-down"
    );
    for k in 1..=8u8 {
        entrega(
            s,
            p,
            x0 + 6.0 * f32::from(k),
            350.0,
            1.0,
            PointerPhase::Move,
        );
        quadro(Some(&mut *s), Some(&mut *p));
    }
    entrega(s, p, x0 + 54.0, 350.0, 1.0, PointerPhase::Up);
}

fn bits(a: &[[f32; 3]]) -> Vec<[u32; 3]> {
    a.iter().map(|c| c.map(f32::to_bits)).collect()
}

fn activa(s: &crate::Sculpt3dScene) -> (usize, Option<LayerId>, Option<f32>) {
    let p = s.objects[s.active]
        .pilha
        .as_ref()
        .expect("a peça tem pilha");
    let a = p.pilha().active();
    let op = a.and_then(|a| p.pilha().get(a)).map(|l| l.opacity);
    (p.pilha().len(), a, op)
}

/// ⭐⭐⭐⭐ **GATE (seam, W3) — «nova camada → pintar → baixar a opacidade → `Ctrl+Z`» MUDA A COR DA
/// PEÇA E VOLTA AO BIT**, e cada `Ctrl+Z` seguinte desfaz um passo (o arrasto inteiro é UM) até à
/// peça de antes — e o `Ctrl+Shift+Z` refá-los todos.
///
/// ⛔ CONTROLO: antes do primeiro quadro o painel não mostra a peça (a tela não está presa).
#[test]
#[ignore = "precisa de adaptador"]
fn o_painel_de_camadas_muda_a_cor_da_peca_e_o_ctrl_z_a_devolve_ao_bit() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_vermelho();
    let mut painel = Painel::novo();
    assert!(
        !p.panel_shows_the_piece(),
        "o CONTROLO: sem a tela, a pilha da peça não aparece"
    );

    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(
        p.panel_layers().map(|l| l.len()),
        Some(1),
        "o painel mostra a pilha da peça"
    );
    let virgem = amostras(&s);

    // 1. Nova camada — o «+» do painel.
    painel.clica(&mut p, PAINTER_LAYERS_ADD);
    quadro(Some(&mut s), Some(&mut p));
    let (n, cima, _) = activa(&s);
    assert_eq!(n, 2, "o «+» criou uma camada na PEÇA");
    let cima = cima.expect("a nova é a activa");
    assert_eq!(
        bits(&amostras(&s)),
        bits(&virgem),
        "uma camada transparente não muda a peça"
    );

    // 2. Pintar — o traço pousa na camada nova.
    traco(&mut s, &mut p, 420.0);
    s.sync_mesh(&gpu);
    quadro(Some(&mut s), Some(&mut p));
    let pintada = amostras(&s);
    assert_ne!(
        bits(&pintada),
        bits(&virgem),
        "o CONTROLO: o traço pintou a peça"
    );

    // 3. Baixar a opacidade — arrastar o slider da linha dela, em DOIS quadros (um arrasto atravessa
    //    quadros; o `Ctrl+Z` desfá-lo inteiro).
    let opacidade = painter_layer_widget_id(cima.0, PainterLayerWidget::Opacity);
    painel.arrasta(&mut p, opacidade, 0.98, 0.6);
    quadro(Some(&mut s), Some(&mut p));
    let meio = activa(&s).2;
    painel.arrasta(&mut p, opacidade, 0.6, 0.2);
    quadro(Some(&mut s), Some(&mut p));
    assert!(
        meio > activa(&s).2,
        "os dois passos chegaram: {meio:?} → {:?}",
        activa(&s).2
    );
    let (_, _, op) = activa(&s);
    assert!(
        op.is_some_and(|o| o < 0.5),
        "a opacidade da camada desceu: {op:?}"
    );
    let baixa = amostras(&s);
    assert_ne!(
        bits(&baixa),
        bits(&pintada),
        "baixar a opacidade mudou a COR DA PEÇA"
    );
    assert_eq!(
        p.panel_layers()
            .and_then(|l| l.get(cima))
            .map(|l| l.opacity),
        op,
        "o painel mostra o que a peça tem"
    );

    // 4. Ctrl+Z — o arrasto inteiro é UM passo, e a peça volta AO BIT.
    assert!(tecla(&mut s, false));
    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(
        bits(&amostras(&s)),
        bits(&pintada),
        "o Ctrl+Z não devolveu a peça ao bit"
    );
    assert_eq!(activa(&s).2, Some(1.0));
    assert_eq!(
        p.panel_layers()
            .and_then(|l| l.get(cima))
            .map(|l| l.opacity),
        Some(1.0),
        "o painel segue o desfazer"
    );
    // … o traço …
    assert!(tecla(&mut s, false));
    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(bits(&amostras(&s)), bits(&virgem));
    // … e a camada nova.
    assert!(tecla(&mut s, false));
    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(activa(&s).0, 1, "o Ctrl+Z tirou a camada");
    assert_eq!(bits(&amostras(&s)), bits(&virgem));

    // E o refazer devolve os três, ao bit.
    for _ in 0..3 {
        assert!(tecla(&mut s, true));
        quadro(Some(&mut s), Some(&mut p));
    }
    assert_eq!(
        bits(&amostras(&s)),
        bits(&baixa),
        "o refazer devolveu a peça ao bit"
    );
}

/// ⭐⭐ **GATE — a activa que não é de pintura: o pen-down recusa e o painel DIZ porquê**; o
/// controlo é a mesma peça com a camada de pintura escolhida, onde o traço pousa.
#[test]
#[ignore = "precisa de adaptador"]
fn com_um_ajuste_activo_o_traco_recusa_e_o_painel_diz_porque() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu);
    let mut p = painter_vermelho();
    let mut painel = Painel::novo();
    quadro(Some(&mut s), Some(&mut p));
    let virgem = amostras(&s);
    let (_, base, _) = activa(&s);
    // Um ajuste (Invert) — e a linha dele escolhida como activa.
    p.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
        ph2d_tool_painter::ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
        ph2d_tool_painter::AdjustmentKind::ALL
            .iter()
            .position(|k| *k == ph2d_tool_painter::AdjustmentKind::Invert)
            .expect("Invert")
            .to_string(),
    ));
    quadro(Some(&mut s), Some(&mut p));
    let pilha = s.objects[s.active]
        .pilha
        .as_ref()
        .expect("pilha")
        .pilha()
        .clone();
    let ajuste = pilha.root()[0];
    painel.clica(
        &mut p,
        painter_layer_widget_id(ajuste.0, PainterLayerWidget::Row),
    );
    quadro(Some(&mut s), Some(&mut p));
    assert_eq!(activa(&s).1, Some(ajuste));
    let invertida = amostras(&s);
    assert_ne!(
        bits(&invertida),
        bits(&virgem),
        "o ajuste vivo mudou a peça"
    );

    assert!(
        !entrega(&mut s, &mut p, 420.0, 350.0, 1.0, PointerPhase::Down),
        "o pen-down sobre um ajuste activo recusa"
    );
    assert!(p.piece_layer_refusal().is_some(), "e o painel tem a frase");
    assert_eq!(bits(&amostras(&s)), bits(&invertida), "a peça não mudou");

    // ⛔ CONTROLO: a base escolhida, o traço pousa.
    painel.clica(
        &mut p,
        painter_layer_widget_id(base.expect("base").0, PainterLayerWidget::Row),
    );
    quadro(Some(&mut s), Some(&mut p));
    traco(&mut s, &mut p, 420.0);
    quadro(Some(&mut s), Some(&mut p));
    assert_ne!(
        bits(&amostras(&s)),
        bits(&invertida),
        "com a base activa o traço pousa"
    );
    assert!(p.piece_layer_refusal().is_none(), "e a frase sai");
}

/// 🔎 **SONDA — o preço de ARRASTAR a opacidade, de ponta a ponta** (`docs/3D/30` §7, a W3:
/// *«se passar de um quadro, a W1b é a onda seguinte»*). Por degrau da peça da lição, com 3 camadas:
/// cada passo = os pedidos do quadro pela porta + a recomposição (desde a W1b, `docs/3D/30` §13: na
/// CPU só o prefixo dos vértices) + o `sync_mesh` (a pilha composta e achatada NA PLACA, até ela
/// acabar).
#[test]
#[ignore = "sonda: precisa de adaptador e imprime a tabela"]
fn diag_o_preco_de_arrastar_a_opacidade_de_ponta_a_ponta() {
    use ph2d_tool_painter::PieceLayerOp;
    use std::time::Instant;
    let gpu = gpu_or_skip!();
    for k in 3u8..=8 {
        let mut s = cena_52(&gpu.device);
        s.tinta_nivel = Some(k);
        s.sync_mesh(&gpu);
        let mut p = painter_vermelho();
        quadro(Some(&mut s), Some(&mut p));
        for _ in 0..2 {
            p.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
                PAINTER_LAYERS_ADD,
            ));
            quadro(Some(&mut s), Some(&mut p));
        }
        traco(&mut s, &mut p, 420.0);
        quadro(Some(&mut s), Some(&mut p));
        s.sync_mesh(&gpu);
        let n = amostras(&s).len();
        let (_, cima, _) = activa(&s);
        let cima = cima.expect("cima");
        let gesto = painter_layer_widget_id(cima.0, PainterLayerWidget::Opacity);
        let (mut cpu, mut placa) = (Vec::new(), Vec::new());
        for q in 0..20u32 {
            let mut nova = p.panel_layers().expect("pilha").clone();
            nova.set_opacity(cima, 1.0 - q as f32 * 0.04);
            let t = Instant::now();
            let recusa = s.aplica_pedidos_da_pilha(vec![PieceLayerOp::Metadata {
                stack: nova,
                gesture: Some(gesto),
            }]);
            cpu.push(t.elapsed().as_secs_f64() * 1e3);
            assert!(recusa.is_none());
            quadro(Some(&mut s), Some(&mut p));
            let t = Instant::now();
            s.sync_mesh(&gpu);
            gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
            placa.push(t.elapsed().as_secs_f64() * 1e3);
        }
        cpu.sort_by(f64::total_cmp);
        placa.sort_by(f64::total_cmp);
        eprintln!(
            "degrau {k} ({}x) · {n} amostras · um passo do arrasto: porta+recompor (CPU) mediana \
             {:.2} ms · pior {:.2} ms | sync_mesh (compor na placa) mediana {:.2} ms · pior {:.2} ms",
            1u32 << k,
            cpu[10],
            cpu[19],
            placa[10],
            placa[19]
        );
    }
}

/// ⭐ **O MESMO traço preto na base e numa camada nova por cima dá a MESMA peça, a um degrau**
/// (report do dono, 03/10: *«o traço feito na camada 2 tem qualidade menor»*; ADR-0177, P1 gate b).
/// A cena A pinta a base; a B pinta uma camada nova. O pincel mistura em tons de ecrã, e as camadas
/// juntam-se no mesmo espaço — antes juntavam-se em luz e a B era `srgb(lin(antes)·(1 − w))`, até
/// `0,286` (73 degraus) fora da A. Imprime também o resíduo contra essa previsão antiga, com
/// `w = 1 − A/antes` (pincel preto).
#[test]
#[ignore = "precisa de adaptador"]
fn o_traco_numa_camada_nova_e_o_traco_na_base() {
    use ph2d_color::srgb::{linear_to_srgb_unit, srgb_to_linear_unit};
    let gpu = gpu_or_skip!();
    let pinta = |camada_nova: bool| {
        let mut s = cena_52(&gpu.device);
        s.sync_mesh(&gpu);
        let mut p = PainterTool::default();
        p.set_brush_color_srgb8([0, 0, 0]);
        p.set_brush_strength(1.0);
        p.set_brush_size_px(24.0);
        quadro(Some(&mut s), Some(&mut p));
        if camada_nova {
            p.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
                PAINTER_LAYERS_ADD,
            ));
            quadro(Some(&mut s), Some(&mut p));
        }
        s.sync_mesh(&gpu);
        let antes = amostras(&s);
        traco(&mut s, &mut p, 420.0);
        quadro(Some(&mut s), Some(&mut p));
        s.sync_mesh(&gpu);
        (antes, amostras(&s), activa(&s).0)
    };
    let (antes, a, ca) = pinta(false);
    let (antes_b, b, cb) = pinta(true);
    assert_eq!(antes, antes_b, "as duas cenas partem da mesma peça");
    eprintln!("camadas: A {ca} · B {cb} · amostras {}", a.len());
    let (mut tocadas, mut fora_da_luz, mut fora_da_a) = (0usize, Vec::new(), Vec::new());
    for i in 0..a.len() {
        let w = (0..3)
            .map(|c| {
                if antes[i][c] > 0.05 {
                    1.0 - a[i][c] / antes[i][c]
                } else {
                    0.0
                }
            })
            .fold(0.0f32, f32::max)
            .clamp(0.0, 1.0);
        if w < 1e-4 && (0..3).all(|c| (b[i][c] - antes[i][c]).abs() < 1e-4) {
            continue;
        }
        tocadas += 1;
        let prev = (0..3)
            .map(|c| linear_to_srgb_unit(srgb_to_linear_unit(antes[i][c]) * (1.0 - w)))
            .collect::<Vec<_>>();
        let d_luz = (0..3)
            .map(|c| (b[i][c] - prev[c]).abs())
            .fold(0.0f32, f32::max);
        let d_a = (0..3)
            .map(|c| (b[i][c] - a[i][c]).abs())
            .fold(0.0f32, f32::max);
        fora_da_luz.push((d_luz, i, w));
        fora_da_a.push((d_a, i, w));
    }
    let resume = |nome: &str, v: &mut Vec<(f32, usize, f32)>| {
        v.sort_by(|x, y| y.0.total_cmp(&x.0));
        let n = v.len().max(1);
        let acima = |t: f32| v.iter().filter(|x| x.0 > t).count();
        eprintln!(
            "{nome}: pior {:.3} · mediana {:.4} · > 4/255: {} · > 16/255: {} · > 64/255: {} (de {n})",
            v.first().map_or(0.0, |x| x.0),
            v[n / 2].0,
            acima(4.0 / 255.0),
            acima(16.0 / 255.0),
            acima(64.0 / 255.0)
        );
        for (d, i, w) in v.iter().take(6) {
            eprintln!(
                "   amostra {i}: dif {d:.3} · w {w:.3} · antes {:?} · A {:?} · B {:?}",
                antes[*i], a[*i], b[*i]
            );
        }
    };
    eprintln!("amostras tocadas: {tocadas}");
    resume("B contra a previsão em LUZ", &mut fora_da_luz);
    resume("B contra A", &mut fora_da_a);
    let pior = fora_da_a.first().map_or(0.0, |x| x.0);
    assert!(tocadas > 100, "o traço tocou só {tocadas} amostras");
    assert!(
        pior <= 1.0 / 255.0 + 1e-4,
        "o traço numa camada nova difere do traço na base em {:.1} degraus",
        pior * 255.0
    );
}
