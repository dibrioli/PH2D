//! **O painel só vale para o futuro** (W3b item 3, dono 27/10: «uma vez que o traço foi feito, se o usuário muda as
//! configurações no painel, não afeta a tinta já colocada no papel — passa a valer para os próximos traços»).
//! Doc: `docs/Painter/49_realshader/W3b_painel_so_no_futuro.md`.

use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::grid::aparencia::AparenciaCelula;
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob, KnobDef, Rebuild};

const W: usize = 200;
const H: usize = 150;
/// As linhas do canvas dos dois primeiros traços (y = 60 e 70, mais o raio e a água) e as do traço seguinte (y = 120).
const ANTIGOS: std::ops::Range<usize> = 0..95;
const SEGUINTE: std::ops::Range<usize> = 100..H;

fn pinta(t: &mut PainterTool, y: f32) {
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(p(30.0 + 3.5 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(170.0, PointerPhase::Up));
    for _ in 0..20 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A tela de prova: papel escolhido e impresso na tinta (Texture on Paint), relevo ligado e um traço MOLHADO sobre um
/// seco de outra tinta (a junção do filme sobre o assentado é onde a lei do ecrã morde).
fn tela(oleo: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
    t.paint.wetpaint.paper_visual = true;
    t.paint.wetpaint.knobs.set(Knob::Emboss, 1.5);
    t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 1200.0);
    if oleo {
        t.set_wet_oleo_real_ligado(true);
    }
    t.set_brush_color_srgb8([180, 30, 30]);
    pinta(&mut t, 60.0);
    t.wetpaint_dry_canvas();
    t.set_brush_color_srgb8([30, 60, 200]);
    pinta(&mut t, 70.0);
    t
}

fn recompoe(t: &mut PainterTool) -> Vec<u8> {
    t.wet_recomposite_full();
    t.canvas_rgba.to_vec()
}

fn linhas(c: &[u8], r: std::ops::Range<usize>) -> &[u8] {
    &c[r.start * W * 4..r.end * W * 4]
}

/// Um valor do knob longe do actual, dentro da faixa.
fn outro(def: &KnobDef, v: f64) -> f64 {
    let alvo = if (def.max - v).abs() > (v - def.min).abs() {
        def.max
    } else {
        def.min
    };
    v + (alvo - v) * 0.7
}

/// Os controlos do painel do pincel que não são knobs do registo (os factos que a sessão reconcilia).
type Muda = fn(&mut PainterTool);
const FACTOS: [(&str, Muda, Muda); 8] = [
    (
        "km_glaze",
        |t| t.paint.wetpaint.km_glaze ^= true,
        |t| t.paint.wetpaint.km_glaze ^= true,
    ),
    (
        "km_mixing",
        |t| t.paint.wetpaint.km_mixing ^= true,
        |t| t.paint.wetpaint.km_mixing ^= true,
    ),
    (
        "agua_viva",
        |t| t.set_wet_agua_viva(!t.wet_agua_viva()),
        |t| t.set_wet_agua_viva(!t.wet_agua_viva()),
    ),
    (
        "borda_que_pesa",
        |t| t.paint.wetpaint.borda_que_pesa = 1.0,
        |t| t.paint.wetpaint.borda_que_pesa = 0.0,
    ),
    (
        "opacidade",
        |t| t.paint.wetpaint.opacidade = 0.3,
        |t| t.paint.wetpaint.opacidade = 1.0,
    ),
    (
        "tilt",
        |t| t.paint.wetpaint.tilt_spoke ^= 1,
        |t| t.paint.wetpaint.tilt_spoke ^= 1,
    ),
    // A Paint Texture é do TRAÇO (dono 28/10), guardada na célula (`AparenciaCelula::textura`).
    (
        "paint_texture",
        |t| t.paint.wetpaint.paper_visual ^= true,
        |t| t.paint.wetpaint.paper_visual ^= true,
    ),
    // A caixa Real Oil é do TRAÇO (dono 28/10), guardada na célula (`OleoCelula::oleo`); o Depth e o Gloss são knobs
    // (`oilDepth`, `oilGloss`) e entram pela tabela.
    (
        "real_oil",
        |t| t.set_wet_oleo_real_ligado(t.wet_oleo_real().is_none()),
        |t| t.set_wet_oleo_real_ligado(t.wet_oleo_real().is_none()),
    ),
];

/// O knob é do PAPEL (a tela inteira, decisão do dono 27/10): o que refaz a folha.
fn do_papel(def: &KnobDef) -> bool {
    def.rebuild == Some(Rebuild::Paper)
}

/// ⭐ O GATE: cada controlo do pincel — TODO knob do registo (a tabela; knob novo entra sozinho) e os factos da sessão —
/// mudado depois do traço, com a tela inteira recomposta: nem um byte muda. Com e sem o Óleo Real.
#[test]
fn mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada() {
    for oleo in [false, true] {
        let mut t = tela(oleo);
        let base = recompoe(&mut t);
        assert!(
            base.chunks(4).any(|p| p != [255, 255, 255, 255]),
            "controlo: a tela tem tinta"
        );
        let mut mudam = Vec::new();
        for def in KNOB_DEFS.iter().filter(|d| !do_papel(d)) {
            let v = t.paint.wetpaint.knobs.get(def.knob);
            t.paint.wetpaint.knobs.set(def.knob, outro(def, v));
            if recompoe(&mut t) != base {
                mudam.push(def.key);
            }
            t.paint.wetpaint.knobs.set(def.knob, v);
        }
        for (nome, muda, volta) in FACTOS {
            muda(&mut t);
            if recompoe(&mut t) != base {
                mudam.push(nome);
            }
            volta(&mut t);
        }
        assert!(
            mudam.is_empty(),
            "óleo {oleo}: estes controlos do pincel mudaram a tinta já pousada: {mudam:?}"
        );
        assert_eq!(
            recompoe(&mut t),
            base,
            "óleo {oleo}: tudo de volta = a mesma tela"
        );
    }
}

/// Os knobs do papel são os três que refazem a folha (o dente mora no cartão Paper, fora do registo; a Paint Texture
/// é do TRAÇO desde 28/10 — `FACTOS`, cartão Wet Paint): a excepção do gate é pequena e nomeada.
#[test]
fn os_knobs_do_papel_sao_os_que_refazem_a_folha() {
    let papel: Vec<_> = KNOB_DEFS
        .iter()
        .filter(|d| do_papel(d))
        .map(|d| d.key)
        .collect();
    assert_eq!(papel, ["paperContrast", "paperFibres", "paperGrooves"]);
}

/// …e o traço SEGUINTE usa o valor novo: os três controlos guardados na célula mudam o traço que vem depois, e o
/// de antes fica ao byte.
#[test]
fn o_traco_seguinte_usa_o_valor_novo() {
    let controlos: [(&str, Muda); 4] = [
        ("paperVisibility", |t| {
            t.paint.wetpaint.knobs.set(Knob::PaperVisibility, 0.2)
        }),
        ("visualGrain", |t| {
            t.paint.wetpaint.knobs.set(Knob::VisualGrain, 2.5)
        }),
        ("km_glaze", |t| t.paint.wetpaint.km_glaze = true),
        // A tela pinta com ela ligada: desligá-la tira o papel da tinta dos seguintes.
        ("paint_texture", |t| t.paint.wetpaint.paper_visual = false),
    ];
    // O seguinte cruza o primeiro (o vidrado só morde onde um filme cobre outra tinta).
    let seguinte = |t: &mut PainterTool| {
        t.wetpaint_dry_canvas();
        t.set_brush_color_srgb8([200, 180, 20]);
        pinta(t, 120.0);
        t.wetpaint_dry_canvas();
        t.set_brush_color_srgb8([30, 60, 200]);
        pinta(t, 128.0);
        recompoe(t)
    };
    let sem = seguinte(&mut tela(false));
    for (nome, muda) in controlos {
        let mut t = tela(false);
        muda(&mut t);
        let com = seguinte(&mut t);
        // A MESMA história (secar, pintar ao lado) com e sem a mudança: a tinta de antes fica ao byte.
        assert_eq!(
            linhas(&com, ANTIGOS),
            linhas(&sem, ANTIGOS),
            "{nome}: o traço de antes mudou"
        );
        assert_ne!(
            linhas(&com, SEGUINTE),
            linhas(&sem, SEGUINTE),
            "{nome}: o traço seguinte não usou o valor novo"
        );
    }
}

/// Neutro ao bit: a fábrica é a [`AparenciaCelula::NEUTRA`], e pintar com ela não aloca o campo — o render lê
/// exactamente os valores de antes.
#[test]
fn a_fabrica_nao_aloca_a_aparencia() {
    let d = WetKnobs::DEFAULT;
    let n = AparenciaCelula::NEUTRA;
    assert_eq!(f64::from(n.papel), d.get(Knob::PaperVisibility));
    assert_eq!(f64::from(n.grao), d.get(Knob::VisualGrain));
    assert_eq!(n.vidrado, 0.0);
    assert_eq!(n.textura, 0.0, "a Paint Texture de fábrica é desligada");
    assert!(!PainterTool::default().paint.wetpaint.paper_visual);
    // A Live Water troca a velatura do filme que o ecrã lê da SESSÃO: hoje as duas são iguais. Se divergirem, a
    // velatura do filme passa a mudar a tinta pousada e tem de ir para a célula (`grid::aparencia`).
    assert_eq!(
        ph2d_wet_paint::colorops::VELATURA_DO_FILME,
        ph2d_wet_paint::colorops::VELATURA_DO_FILME_VIVA
    );
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    pinta(&mut t, 60.0);
    t.wet_bring_home();
    let sess = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let g = &sess.engine.layers[0].grid;
    assert!(
        g.sett.iter().chain(&g.susp).any(|&m| m > 0.0),
        "controlo: há tinta"
    );
    assert!(g.aparencia.is_empty(), "a fábrica alocou a aparência");
    // O custo de quem sai da fábrica: 16 B por célula (com a Paint Texture do traço, 28/10), contra os bytes que a grade já retém por célula.
    let base = g.bytes_retidos();
    let cells = g.cells;
    t.paint.wetpaint.knobs.set(Knob::PaperVisibility, 0.4);
    pinta(&mut t, 100.0);
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    assert_eq!(std::mem::size_of::<AparenciaCelula>(), 16);
    assert_eq!(
        g.bytes_retidos() - base,
        16 * cells,
        "o campo custa 16 B por célula"
    );
    eprintln!(
        "APARENCIA: grade {} B/célula, campo +16 B/célula",
        base as f64 / cells as f64
    );
}

/// O «Glaze layering» guardado na célula é a lei de velatura da sessão ao bit: uma tela pintada TODA com ele ligado sai
/// igual à da sessão cuja lei já tapa por velatura em toda célula que o rasto pousou — o ecrã e cada transferência
/// vertical (assentar, re-molhar, o Dry, a secagem) lêem o da célula pela mesma porta (`Ecra::com_vidrado`).
/// O pigmento que a ÁGUA leva a uma célula virgem não leva a aparência (a advecção não move o campo, como o relevo, o
/// Glaze e o material); a orla virgem que o rasto veste cobre-o — o resto (≤ 0,1 %) fica fora da comparação, contado.
#[test]
fn o_vidrado_da_celula_e_a_lei_de_velatura_ao_bit() {
    let pinta_tudo = |celula: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.set_brush_size_px(14.0);
        t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 1200.0);
        assert!(t.ensure_wet_session());
        if celula {
            t.paint.wetpaint.km_glaze = true;
        } else {
            let s = t.paint.wetpaint.session.as_mut().expect("a sessão");
            s.engine.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE_EM_VELATURA;
        }
        t.set_brush_color_srgb8([180, 30, 30]);
        pinta(&mut t, 60.0);
        t.wetpaint_dry_canvas();
        t.set_brush_color_srgb8([30, 60, 200]);
        pinta(&mut t, 70.0);
        for _ in 0..240 {
            t.on_tick(1000.0 / 60.0);
        }
        t.wetpaint_dry_canvas();
        t.set_brush_color_srgb8([200, 180, 20]);
        pinta(&mut t, 65.0);
        let tela = recompoe(&mut t);
        t.wet_bring_home();
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .engine
            .layers[0]
            .grid;
        let pousou: Vec<bool> = (0..W * H)
            .map(|k| {
                let i = (k % W + 1) + (k / W + 1) * g.s;
                ph2d_wet_paint::grid::aparencia::vidrado(&g.aparencia, i) == 1.0
            })
            .collect();
        (tela, pousou)
    };
    let (celula, pousou) = pinta_tudo(true);
    let (sessao, _) = pinta_tudo(false);
    let pintado = |c: &[u8], k: usize| c[k * 4..k * 4 + 4] != [255, 255, 255, 255];
    let fora = (0..W * H)
        .filter(|&k| pintado(&celula, k) && !pousou[k])
        .count();
    let dentro = (0..W * H).filter(|&k| pousou[k]).count();
    assert!(dentro > 2000, "controlo: o rasto pousou ({dentro} células)");
    let difs = (0..W * H)
        .filter(|&k| pousou[k] && celula[k * 4..k * 4 + 4] != sessao[k * 4..k * 4 + 4])
        .count();
    eprintln!("VIDRADO: {dentro} células pousadas, {fora} pintadas só pela água, {difs} diferem");
    // A orla virgem que a tinta veste (`grid::aparencia::veste_virgens`): sem ela, 1 060 das 6 549 células pintadas
    // (16 %) eram pigmento levado pela água a uma célula sem aparência — todas a 1–2 células do rasto.
    let pintadas = (0..W * H).filter(|&k| pintado(&celula, k)).count();
    assert!(
        fora * 1000 <= pintadas,
        "{fora} das {pintadas} células pintadas ficaram sem a aparência da tinta (a orla virgem não chegou)"
    );
    assert_eq!(difs, 0, "o vidrado da célula divergiu da lei de velatura");
}

/// O dente do papel é da TELA (dono 27/10): escolher um pincel não o muda — nenhum preset de fábrica, e um preset
/// antigo que traga `paper_depth` é lido e o valor IGNORADO. Gravar um preset não o leva.
#[test]
fn escolher_um_pincel_nao_muda_o_dente_da_tela() {
    const DENTE: f32 = 0.37;
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let fabrica = ph2d_brush_presets::Biblioteca::carregar(&raiz, None).entradas;
    assert!(fabrica.len() > 40, "controlo: a fábrica carregou");
    let com_dente = fabrica
        .iter()
        .filter(|e| e.preset.parametros.brush.contains_key("paper_depth"))
        .count();
    let mut valores: std::collections::BTreeMap<String, usize> = Default::default();
    for e in &fabrica {
        if let Some(v) = e.preset.parametros.brush.get("paper_depth") {
            *valores.entry(v.to_string()).or_default() += 1;
        }
    }
    eprintln!(
        "DENTE: {com_dente} dos {} presets de fábrica trazem paper_depth: {valores:?}",
        fabrica.len()
    );
    let mut antigo = fabrica[0].preset.clone();
    antigo
        .parametros
        .brush
        .insert("paper_depth".into(), serde_json::json!(1.9));
    for p in fabrica.iter().map(|e| &e.preset).chain([&antigo]) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_brush_paper_depth(DENTE);
        let _ = t.preset_aplicar(p);
        assert_eq!(
            t.paint.brush.paper_depth, DENTE,
            "o pincel {:?} mudou o dente do papel da tela",
            p.parametros.name
        );
    }
    // Gravar (também sobre o preset antigo que o trazia) não o leva.
    let t = PainterTool::default();
    let gravado = t.wet_preset_capturar(&antigo.parametros).0;
    assert!(
        !gravado.brush.contains_key("paper_depth"),
        "o preset de pincel gravou o dente do papel"
    );
}
