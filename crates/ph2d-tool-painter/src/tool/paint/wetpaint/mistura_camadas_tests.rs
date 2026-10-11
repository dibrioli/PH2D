//! **N camadas de amarelo sobre azul SECO** (smoke do dono 10/10, prompt 15: *«o amarelo sobre o azul produz
//! preto e por mais que se coloque tinta amarela, nunca para de reagir… e o amarelo nunca chega a cobrir»*).
//!
//! A cena da bancada do Rebelle (`ferramentas/camadas.py`, a mesma geometria deslocada de (+200, +120)): uma
//! referência amarela no branco; a mancha azul (9 traços, Dry canvas); 1…10 passadas de amarelo em serpentina
//! sobre o miolo (x 285–315, y 185–215), com a tinta MOLHADA entre passadas (0,5 s) ou SECA (Dry canvas).
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};

pub(super) const W: usize = 600;
pub(super) const H: usize = 400;
pub(super) const AZUL: [u8; 3] = [30, 100, 220];
pub(super) const AMARELO: [u8; 3] = [245, 205, 25];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn media(px: &[u8], x0: usize, x1: usize, y0: usize, y1: usize) -> [f64; 3] {
    let mut s = [0.0; 3];
    let mut n = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            for (c, v) in s.iter_mut().enumerate() {
                *v += f64::from(px[(y * W + x) * 4 + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

fn lin(c: f64) -> f64 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub(super) fn lum(c: [f64; 3]) -> f64 {
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

/// O que o motor guarda no miolo: a cor e a massa do assentado e do suspenso (médias das células).
#[derive(Clone, Copy, Debug)]
pub(super) struct Motor {
    pub sett_rgb: [f64; 3],
    pub sett: f64,
    pub susp_rgb: [f64; 3],
    pub susp: f64,
}

/// Uma leitura do miolo: o ecrã, o pixel mais escuro da zona do amarelo (o 2.º percentil da luminância, x
/// 240–360, y 155–245) e o motor.
#[derive(Clone, Copy, Debug)]
pub(super) struct Miolo {
    pub ecra: [f64; 3],
    pub escuro: f64,
    pub motor: Motor,
}

/// As condições do dono que a cena pode ligar: a mistura K–M («Pigment mixing (K-M)» do Tuning), a tela
/// inclinada (o anel do Tilt, `None` = direita), a Heavy Edge, a Live Water, o Smooth Edges, o tamanho do
/// pincel e quantas passadas cabem num só traço (esfregar).
#[derive(Clone, Copy, Debug)]
pub(super) struct Cond {
    /// A Opacity do Wet Paint (`None` = a do preset, o produto).
    pub opacidade: Option<f32>,
    pub km: bool,
    pub tilt: Option<u8>,
    pub borda: f64,
    pub viva: bool,
    pub suave: f32,
    pub tamanho: f32,
    pub por_traco: usize,
}

pub(super) const SIMPLES: Cond = Cond {
    opacidade: None,
    km: false,
    tilt: None,
    borda: 0.0,
    viva: false,
    suave: 0.0,
    tamanho: 30.0,
    por_traco: 1,
};

fn escuro(px: &[u8]) -> f64 {
    let mut l: Vec<f64> = Vec::new();
    for y in 155..245 {
        for x in 240..360 {
            let i = (y * W + x) * 4;
            l.push(lum([px[i], px[i + 1], px[i + 2]].map(f64::from)));
        }
    }
    l.sort_by(f64::total_cmp);
    l[l.len() / 50]
}

fn le_motor(t: &PainterTool) -> Motor {
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("sessão")
        .engine
        .layers[0]
        .grid;
    let r = (W as f64 / g.w as f64).round().max(1.0);
    let (c0, c1) = ((285.0 / r) as usize + 1, (315.0 / r) as usize + 1);
    let (l0, l1) = ((185.0 / r) as usize + 1, (215.0 / r) as usize + 1);
    let mut m = Motor {
        sett_rgb: [0.0; 3],
        sett: 0.0,
        susp_rgb: [0.0; 3],
        susp: 0.0,
    };
    let mut n = 0.0;
    for cy in l0..l1 {
        for cx in c0..c1 {
            let i = cx + cy * g.s;
            for c in 0..3 {
                m.sett_rgb[c] += f64::from(g.sett_rgb[i][c]);
                m.susp_rgb[c] += f64::from(g.susp_rgb[i][c]);
            }
            m.sett += f64::from(g.sett[i]);
            m.susp += f64::from(g.susp[i]);
            n += 1.0;
        }
    }
    m.sett_rgb = m.sett_rgb.map(|v| v / n);
    m.susp_rgb = m.susp_rgb.map(|v| v / n);
    m.sett /= n;
    m.susp /= n;
    m
}

fn le(t: &PainterTool) -> Miolo {
    Miolo {
        ecra: media(&t.canvas_rgba, 285, 315, 185, 215),
        escuro: escuro(&t.canvas_rgba),
        motor: le_motor(t),
    }
}

fn traco(t: &mut PainterTool, pts: &[[f32; 2]]) {
    traco_n(t, pts, 1);
}

/// O MESMO caminho `n` vezes sem levantar o pincel (esfregar).
fn traco_n(t: &mut PainterTool, pts: &[[f32; 2]], n: usize) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for k in 0..n {
        let ida: Vec<[f32; 2]> = if k % 2 == 0 {
            pts.to_vec()
        } else {
            pts.iter().rev().copied().collect()
        };
        for &p in &ida[1..] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            t.on_tick(8.0);
        }
    }
    let fim = if n % 2 == 1 {
        pts[pts.len() - 1]
    } else {
        pts[0]
    };
    t.on_canvas_pointer(cp(fim, PointerPhase::Up));
}

fn linha(a: [f32; 2], b: [f32; 2]) -> Vec<[f32; 2]> {
    let n = ((b[0] - a[0]).abs().max((b[1] - a[1]).abs()) / 3.0).max(1.0) as usize;
    (0..=n)
        .map(|k| {
            let f = k as f32 / n as f32;
            [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
        })
        .collect()
}

fn serpentina() -> Vec<[f32; 2]> {
    let mut pts: Vec<[f32; 2]> = Vec::new();
    for (j, y) in [170.0, 190.0, 210.0, 230.0].into_iter().enumerate() {
        let (x0, x1) = if j % 2 == 0 {
            (250.0, 350.0)
        } else {
            (350.0, 250.0)
        };
        if let Some(&ult) = pts.last() {
            pts.extend(linha(ult, [x0, y]).into_iter().skip(1));
            pts.extend(linha([x0, y], [x1, y]).into_iter().skip(1));
        } else {
            pts.extend(linha([x0, y], [x1, y]));
        }
    }
    pts
}

fn ticks(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

fn seca(t: &mut PainterTool) {
    ticks(t, 60);
    assert!(
        t.route_brush_wetpaint_event(&PanelEvent::Click(crate::ids::PAINTER_WETPAINT_DRYCANVAS))
    );
    ticks(t, 30);
}

/// A trajectória: `[0]` = o azul seco antes do amarelo, `[k]` = depois da passada `k`, e no fim `fim`
/// (+2 s) e `seco` (Dry canvas). `ajusta` mexe no motor depois de a sessão nascer (as variantes).
pub(super) struct Trajectoria {
    pub amarelo: [f64; 3],
    pub passadas: Vec<Miolo>,
    pub fim: Miolo,
    pub seco: Miolo,
    /// As telas (RGBA) de cada passada e do fim seco — só com `PH2D_SONDA_OUT` (as pranchas).
    pub telas: Vec<Vec<u8>>,
}

pub(super) fn cena(
    seco_entre: bool,
    passadas: usize,
    cond: Cond,
    ajusta: &dyn Fn(&mut ph2d_wet_paint::painter::Engine),
) -> Trajectoria {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    // A física mede-se no carimbo do motor de sempre: a W2 só re-rotula o número do painel.
    t.set_wet_largura_crua(1.0);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_wet_largura_crua(1.0);
    t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    t.paint.wetpaint.tilt_on = false;
    t.set_brush_size_px(30.0);
    t.set_wet_agua_viva(cond.viva);
    t.set_wet_borda_que_pesa(cond.borda);
    t.set_wet_smooth_edges(cond.suave);
    t.paint.wetpaint.km_mixing = cond.km;
    if let Some(o) = cond.opacidade {
        t.paint.wetpaint.opacidade = o;
    }
    assert!(t.ensure_wet_session());
    // O tick leva as opções autoradas ao motor (o reconcile); só depois as variantes mexem nele.
    ticks(&mut t, 1);
    ajusta(&mut t.paint.wetpaint.session.as_mut().expect("sessão").engine);
    t.set_brush_color_srgb8(AMARELO);
    traco(&mut t, &linha([100.0, 360.0], [500.0, 360.0]));
    ticks(&mut t, 30);
    t.set_brush_color_srgb8(AZUL);
    for y in (120..=280).step_by(20) {
        traco(&mut t, &linha([150.0, y as f32], [450.0, y as f32]));
        ticks(&mut t, 4);
    }
    seca(&mut t);
    let guarda = std::env::var("PH2D_SONDA_OUT").is_ok();
    let mut telas = Vec::new();
    let mut out = vec![le(&t)];
    if guarda {
        telas.push(t.canvas_rgba.to_vec());
    }
    t.set_brush_color_srgb8(AMARELO);
    if let Some(anel) = cond.tilt {
        t.paint.wetpaint.tilt_on = true;
        t.paint.wetpaint.tilt_ring = anel;
    }
    t.set_brush_size_px(cond.tamanho);
    let s = serpentina();
    for _ in 0..passadas {
        traco_n(&mut t, &s, cond.por_traco);
        if seco_entre {
            seca(&mut t);
        } else {
            ticks(&mut t, 30);
        }
        out.push(le(&t));
        if guarda {
            telas.push(t.canvas_rgba.to_vec());
        }
    }
    ticks(&mut t, 120);
    let fim = le(&t);
    seca(&mut t);
    if guarda {
        telas.push(t.canvas_rgba.to_vec());
    }
    Trajectoria {
        amarelo: media(&t.canvas_rgba, 280, 320, 355, 366),
        passadas: out,
        fim,
        seco: le(&t),
        telas,
    }
}

pub(super) fn r(c: [f64; 3]) -> [i32; 3] {
    c.map(|v| v.round() as i32)
}

/// **Quanto do azul ficou ESCONDIDO** — a fracção do caminho azul → amarelo no canal VERMELHO, em luz linear:
/// `0` = o azul seco do miolo, `1` = o amarelo como ele sai sozinho no branco (com a Opacity a tinta é
/// diluída e nunca chega à cor escolhida). Só o vermelho: é a luz que o azul absorve e o amarelo devolve. ⛔
/// O verde junto (a régua de antes) confunde cobrir com CONCENTRAR — o Rebelle molhado acumula até o amarelo
/// puro (G 205) abaixo do próprio traço (G 241) e a média dava-lhe 0,46 à 10.ª; pelo vermelho: 0,30 · 0,75 à
/// 3.ª · 0,89 à 10.ª, seco 0,96 (`fixtures/rebelle_camadas/`).
pub(super) fn para_o_amarelo(azul: [f64; 3], y: [f64; 3], c: [f64; 3]) -> f64 {
    (lin(c[0]) - lin(azul[0])) / (lin(y[0]) - lin(azul[0]))
}

/// **A barra (o Rebelle 8.3.4, `fixtures/rebelle_camadas/`, a mesma cena):** (a) nada fica mais escuro que o
/// azul — o miolo nunca abaixo da luminância do azul seco e o 2.º percentil da zona nunca abaixo de
/// `0,8×` dela (o Rebelle: o miolo sobe desde a 1.ª passada nos dois modos); (b) converge — depois de 10
/// passadas o miolo está a `≥ 0,7` do caminho para o amarelo e não recuou da 1.ª (o Rebelle cobre à 3.ª
/// molhado e à 3.ª seco). `so_a` = só a (a): a K–M com a tinta molhada sobre o azul seco re-dissolvido
/// fica verde, e isso é a lei do Staining (05 §4b «Mistura»).
pub(super) fn barra(nome: &str, tr: &Trajectoria, so_a: bool) -> Result<(), String> {
    let azul = tr.passadas[0].ecra;
    let la = lum(azul);
    let mut f = Vec::new();
    let todas = tr.passadas[1..].iter().chain([&tr.fim, &tr.seco]);
    for (k, m) in todas.enumerate() {
        if lum(m.ecra) < la || m.escuro < 0.8 * la {
            f.push(format!(
                "{nome} passo {}: miolo {:?} lum {:.3} · 2.º percentil {:.3} (azul {:.3})",
                k + 1,
                r(m.ecra),
                lum(m.ecra),
                m.escuro,
                la
            ));
        }
    }
    if !so_a {
        let (p1, p10) = (
            para_o_amarelo(azul, tr.amarelo, tr.passadas[1].ecra),
            para_o_amarelo(azul, tr.amarelo, tr.passadas[10].ecra),
        );
        if p10 < 0.7 || p10 < p1 {
            f.push(format!(
                "{nome}: depois de 10 passadas a {p10:.2} do amarelo (1.ª {p1:.2}; barra ≥ 0,70)"
            ));
        }
    }
    if f.is_empty() {
        Ok(())
    } else {
        Err(f.join(" · "))
    }
}

pub(super) const TUDO: Cond = Cond {
    opacidade: None,
    km: false,
    tilt: Some(4),
    borda: 1.0,
    viva: true,
    suave: 1.0,
    tamanho: 60.0,
    por_traco: 3,
};

/// **10 camadas de amarelo sobre azul SECO convergem sem preto, na trajectória do Rebelle** — pelos dois
/// misturadores (o de fábrica e o «Pigment mixing (K-M)»), com a tinta molhada entre passadas (a cena
/// simples e a do dono: Live Water, tela inclinada, Heavy Edge, Smooth Edges, pincel grande a esfregar) e
/// seca entre passadas. ⚠️ O CONTROLO está dentro: a lei de antes (a velatura por cima da K–M) na cena do
/// dono tem de reprovar.
#[test]
fn n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto() {
    let km = |c: Cond| Cond { km: true, ..c };
    // O produto de antes pintava com a cor CHEIA (sem a Opacity).
    let cheia = |c: Cond| Cond {
        opacidade: Some(1.0),
        ..c
    };
    // As cenas são independentes: uma por thread (140 s em série no `ci-test` passava o tecto de 180 s do
    // nextest sob a suíte inteira, 11/10).
    let (controlo, f) = std::thread::scope(|s| {
        let controlo = s.spawn(|| {
            // As DUAS leis de antes: a velatura por cima da K–M e o assentar que não preserva o ecrã (cada
            // cura sozinha já tira o preto).
            let antes = |e: &mut ph2d_wet_paint::painter::Engine| {
                e.sim.glaze = 0.47;
                e.sim.lei = ph2d_wet_paint::assenta::Lei::Porto;
                e.sim.molhada_escurece = 1.0;
                e.tuning
                    .set(ph2d_wet_paint::tuning::Knob::PigmentPerDab, 800.0);
            };
            barra("controlo", &cena(false, 10, cheia(km(TUDO)), &antes), true)
        });
        // ⚠️ O controlo «o pincel apanha o assentado» saiu (10/10): a régua do canal VERMELHO contra o
        // amarelo como sai sozinho já não separa esse caso (o pincel sujo fica justificado pela medição da
        // W1e: rasto − azul ≈ 0 no Rebelle).
        let cenas: Vec<_> = [
            ("fábrica, molhado", false, SIMPLES, false),
            ("fábrica, cena do dono", false, TUDO, false),
            ("fábrica, seco", true, SIMPLES, false),
            ("K–M, molhado", false, km(SIMPLES), true),
            ("K–M, cena do dono", false, km(TUDO), false),
            ("K–M, seco", true, km(SIMPLES), false),
        ]
        .into_iter()
        .map(|(nome, seco, cond, so_a)| {
            s.spawn(move || barra(nome, &cena(seco, 10, cond, &|_| {}), so_a))
        })
        .collect();
        let f: Vec<String> = cenas
            .into_iter()
            .filter_map(|h| h.join().expect("a cena entrou em pânico").err())
            .collect();
        (controlo.join().expect("o controlo entrou em pânico"), f)
    });
    assert!(
        controlo.is_err(),
        "CONTROLO: a velatura por cima da K–M passou a barra — a régua não discrimina"
    );
    assert!(
        f.is_empty(),
        "a mistura em camadas falhou a barra do Rebelle: {}",
        f.join(" | ")
    );
}

/// **Sonda (prompt 15): a trajectória do miolo por condição e variante**, no MESMO processo — a tabela H
/// do 05 §4b «Mistura». Filtros: `PH2D_SONDA_SO=<variante,…>` · `PH2D_SONDA_COND=<condição,…>` ·
/// Uma passada de amarelo sobre o azul seco: o miolo com o rato EM BAIXO e depois do Up (+1 · +30 · +120 ·
/// +600 ticks · Dry canvas), com as caixas do Tuning «Pigment mixing (K-M)» (`km`) e «Glaze layering (K-M)»
/// (`glaze`) e a Live Water (`viva`). `ajusta` mexe no motor depois do reconcile.
pub(super) fn duas_etapas(
    km: bool,
    glaze: bool,
    viva: bool,
    ajusta: &dyn Fn(&mut ph2d_wet_paint::painter::Engine),
) -> Vec<[f64; 3]> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_largura_crua(1.0);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_wet_largura_crua(1.0);
    t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    t.paint.wetpaint.tilt_on = false;
    t.set_brush_size_px(30.0);
    t.set_wet_agua_viva(viva);
    t.paint.wetpaint.km_mixing = km;
    t.paint.wetpaint.km_glaze = glaze;
    assert!(t.ensure_wet_session());
    ticks(&mut t, 1);
    ajusta(&mut t.paint.wetpaint.session.as_mut().expect("sessão").engine);
    t.set_brush_color_srgb8(AZUL);
    for y in (120..=280).step_by(20) {
        traco(&mut t, &linha([150.0, y as f32], [450.0, y as f32]));
        ticks(&mut t, 4);
    }
    seca(&mut t);
    t.set_brush_color_srgb8(AMARELO);
    let s = serpentina();
    t.on_canvas_pointer(cp(s[0], PointerPhase::Down));
    for &q in &s[1..] {
        t.on_canvas_pointer(cp(q, PointerPhase::Move));
        t.on_tick(8.0);
    }
    for _ in 0..6 {
        t.on_tick(1000.0 / 60.0);
    }
    let miolo = |t: &PainterTool| media(&t.canvas_rgba, 285, 315, 185, 215);
    let mut out = vec![miolo(&t)];
    t.on_canvas_pointer(cp(s[s.len() - 1], PointerPhase::Up));
    let mut feito = 0;
    for alvo in [1, 30, 120, 240, 360, 480, 600] {
        ticks(&mut t, alvo - feito);
        feito = alvo;
        if std::env::var("PH2D_SONDA_DIAG").is_ok() {
            let m = le_motor(&t);
            println!(
                "    +{alvo} ecrã {:?} | sett {:?} m {:.0} | susp {:?} m {:.0}",
                r(miolo(&t)),
                r(m.sett_rgb),
                m.sett,
                r(m.susp_rgb),
                m.susp
            );
        }
        if [1, 30, 120, 600].contains(&alvo) {
            out.push(miolo(&t));
        }
    }
    seca(&mut t);
    out.push(miolo(&t));
    out
}

pub(super) fn delta(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|c| (a[c] - b[c]).powi(2)).sum::<f64>().sqrt()
}

/// **O TOM mudou?** — o seno do ângulo entre os perfis de absorção das duas leituras
/// (`colorops::diferenca_de_pigmento`): não muda quando a tinta só CLAREIA ao secar (a escolha do dono 10/10,
/// como no Rebelle), muda quando vira outra cor (o amarelo a verde-escuro).
pub(super) fn tom(a: [f64; 3], b: [f64; 3]) -> f64 {
    ph2d_wet_paint::colorops::diferenca_de_pigmento(a, b)
}

/// **O traço não muda de TOM ao secar** (dono 10/10: «Pigment mixing + Glaze layering pintando sobre tinta
/// seca ainda tem uma aparência em 2 etapas — isso não pode acontecer»; e depois, escolhido por ele: a tinta
/// molhada parece mais forte e CLAREIA ao secar, como no Rebelle). Amarelo sobre azul seco, o miolo com o
/// rato em baixo contra +1 · +30 · +120 · +600 ticks e o Dry canvas, nas 4 combinações das caixas × Live
/// Water: o TOM (o seno do ângulo dos perfis de absorção, [`tom`]) não passa de `0,15` — medido 0,06 com a
/// água parada e 0,12 com a Live Water; o Rebelle 0,05 (o cruzamento da W1e); a lei de antes 0,27. ⚠️ O
/// CONTROLO está dentro: a lei de antes (`Lei::Porto`, a velatura e a tinta carregada) reprova.
#[test]
fn o_traco_nao_muda_de_tom_ao_secar() {
    let antes = |e: &mut ph2d_wet_paint::painter::Engine| {
        e.sim.lei = ph2d_wet_paint::assenta::Lei::Porto;
        e.sim.glaze = 0.47;
        e.sim.glaze_filme = 0.20;
        e.sim.molhada_escurece = 1.0;
        e.tuning
            .set(ph2d_wet_paint::tuning::Knob::PigmentPerDab, 800.0);
    };
    let pior = |v: &[[f64; 3]]| v[1..].iter().map(|c| tom(v[0], *c)).fold(0.0, f64::max);
    // Uma condição por thread (independentes; o gancho `CAIXAS` é por thread).
    let (c, f) = std::thread::scope(|s| {
        let controlo = s.spawn(|| pior(&duas_etapas(true, true, false, &antes)));
        let conds: Vec<_> = [false, true]
            .into_iter()
            .flat_map(|viva| {
                [(false, false), (true, false), (false, true), (true, true)]
                    .map(|(km, glaze)| (km, glaze, viva))
            })
            .map(|(km, glaze, viva)| {
                s.spawn(move || {
                    let v = duas_etapas(km, glaze, viva, &|_| {});
                    let p = pior(&v);
                    (p > 0.15).then(|| {
                        format!(
                            "K–M {km} · Glaze {glaze} · Live {viva}: em baixo {:?} → seco {:?}, tom {p:.3}",
                            r(v[0]),
                            r(v[5])
                        )
                    })
                })
            })
            .collect();
        let f: Vec<String> = conds
            .into_iter()
            .filter_map(|h| h.join().expect("a condição entrou em pânico"))
            .collect();
        (controlo.join().expect("o controlo entrou em pânico"), f)
    });
    assert!(
        c > 0.15,
        "CONTROLO: a lei de antes ficou no tom ({c:.3}) — a régua não discrimina"
    );
    assert!(
        f.is_empty(),
        "o traço muda de tom ao secar: {}",
        f.join(" | ")
    );
}
