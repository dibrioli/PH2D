//! **O Ragged Edge com FLUXO e PAPEL** (2026-10-02): o padrão que desenha a borda, a parte dela que segue
//! o dente do Paper, e a faixa do Bleed nos vales. Plano e medições: BUGS_painter #31.

use super::super::media::PaintMedia;
use super::*;
use ph2d_painter_brush::TextureKind;

/// FNV-1a de 64 bits — estável entre versões do Rust (o `DefaultHasher` não promete isso).
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Um traço de aquarela horizontal no estado de fábrica do app. `ajusta` mexe no pincel antes do traço.
fn traco(w: u32, y: f32, ajusta: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * w * 4) as usize], w, w);
    t.set_paint_media(PaintMedia::Watercolor);
    t.paint.brush.color = [0.2, 0.3, 0.8];
    t.paint.brush.radius_px = 14.0;
    ajusta(&mut t);
    risca(&mut t, y, w);
    t
}

fn risca(t: &mut PainterTool, y: f32, w: u32) {
    let x1 = w as f32 - 24.0;
    t.on_canvas_pointer(cp([24.0, y], PointerPhase::Down));
    let mut x = 24.0;
    while x < x1 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        frame(t);
    }
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    frame(t);
}

/// As configurações que o caminho Classic tem de reproduzir AO BYTE.
/// Um ajuste do pincel antes do traço.
type Ajuste = Box<dyn Fn(&mut PainterTool)>;

fn configs_classicas() -> Vec<(&'static str, Ajuste)> {
    vec![
        ("fábrica", Box::new(|_| {})),
        ("ragged 24", Box::new(|t| t.paint.brush.warp = 24.0)),
        (
            "ragged 48 sem AA",
            Box::new(|t| {
                t.paint.brush.warp = 48.0;
                t.paint.brush.smooth_edges = false;
            }),
        ),
        (
            "ragged 24 Paper Rough",
            Box::new(|t| {
                t.paint.brush.warp = 24.0;
                t.set_brush_paper_kind(TextureKind::PaperRough.to_u8());
            }),
        ),
        (
            "ladrilho",
            Box::new(|t| {
                t.paint.brush.warp = 24.0;
                t.paint.tiling = [true, true];
            }),
        ),
    ]
}

/// ⭐ **O Classic e o Paper Edge a 0 são o caminho de HOJE, ao byte** — gravado ANTES da wave
/// (commit `b03bbfba2`). Inclui a sessão de DOIS donos com Ragged diferentes (o campo de estilo da #18).
///
/// **Mutação que tem de sangrar:** trocar o `warp_offset` do Classic pela tabela dos padrões.
#[test]
fn o_classic_e_o_paper_edge_zero_sao_o_byte_de_hoje() {
    let mut lidos = Vec::new();
    for (nome, ajusta) in configs_classicas() {
        lidos.push((nome, fnv(&traco(128, 64.0, ajusta).canvas_rgba)));
    }
    let mut dois = traco(128, 52.0, |t| t.paint.brush.warp = 8.0);
    dois.paint.brush.warp = 40.0;
    risca(&mut dois, 76.0, 128);
    lidos.push(("dois donos", fnv(&dois.canvas_rgba)));
    for (nome, h) in &lidos {
        eprintln!("FLOW-BASE {nome}: {h:#018x}");
    }
    let esperado: [(&str, u64); 6] = BASE;
    for ((nome, h), (_, e)) in lidos.iter().zip(esperado) {
        assert_eq!(*h, e, "{nome}: o caminho Classic mudou de byte");
    }
}

const BASE: [(&str, u64); 6] = [
    ("fábrica", 0xa571_f120_d9eb_f288),
    ("ragged 24", 0x1860_91ef_4ed9_b617),
    ("ragged 48 sem AA", 0xac14_b804_34ed_b932),
    ("ragged 24 Paper Rough", 0x321f_923b_7fd3_b060),
    ("ladrilho", 0x8b25_8d8a_18ec_fef2),
    ("dois donos", 0xb52d_61bf_7582_3c3b),
];

/// Uma faixa larga (r = 30) num canvas de 256² — a régua das bordas.
fn faixa(ajusta: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
    t.set_paint_media(PaintMedia::Watercolor);
    t.paint.brush.color = [0.2, 0.3, 0.8];
    t.paint.brush.radius_px = 30.0;
    ajusta(&mut t);
    t.on_canvas_pointer(cp([40.0, 128.0], PointerPhase::Down));
    let mut x = 40.0;
    while x < 216.0 {
        x += 3.0;
        t.on_canvas_pointer(cp([x, 128.0], PointerPhase::Move));
        frame(&mut t);
    }
    t.on_canvas_pointer(cp([x, 128.0], PointerPhase::Up));
    frame(&mut t);
    t
}

/// O desvio (px) da borda de CIMA da faixa, coluna a coluna no miolo do traço.
fn desvio_da_borda(t: &PainterTool) -> f64 {
    let tops: Vec<f64> = (70..186)
        .map(|cx| {
            (0..256usize)
                .find(|&y| t.canvas_rgba[(y * 256 + cx) * 4] < 245)
                .unwrap_or(256) as f64
        })
        .collect();
    let m = tops.iter().sum::<f64>() / tops.len() as f64;
    (tops.iter().map(|v| (v - m).powi(2)).sum::<f64>() / tops.len() as f64).sqrt()
}

/// Cada padrão desenha OUTRA borda que a do Classic, e nenhum a rasga: o salto médio entre colunas
/// vizinhas fica na ordem do Classic (medido 2026-10-02: Classic `0,38` · padrões `0,38..0,72` px, e
/// antes da normalização da escala `1,43..2,57`).
///
/// ⚠️ A mutação da escala (`escala_do_flow` → `1`) NÃO sangra aqui — a Size `1` sem normalização dá
/// manchas maiores, que não rasgam; quem a apanha é o `todo_padrao_tem_a_forca_e_a_escala_do_classic`.
/// Este gate protege contra o rasgo (o caso que a 1.ª lei, `−∇T`, produzia: `1,73 px`).
#[test]
fn cada_padrao_desenha_outra_borda_sem_rasgar() {
    let classic = faixa(|t| t.paint.brush.warp = 24.0);
    for &k in &ph2d_painter_brush::FLOW_KINDS[1..] {
        let t = faixa(|t| {
            t.paint.brush.warp = 24.0;
            t.set_brush_edge_flow_kind(k.to_u8());
        });
        assert_ne!(
            t.canvas_rgba, classic.canvas_rgba,
            "{k:?}: a borda é a do Classic"
        );
        let tops: Vec<f64> = (70..186)
            .map(|cx| {
                (0..256usize)
                    .find(|&y| t.canvas_rgba[(y * 256 + cx) * 4] < 245)
                    .unwrap_or(256) as f64
            })
            .collect();
        let salto = tops.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / 115.0;
        assert!(
            salto < 1.0,
            "{k:?}: a borda rasga — salto médio {salto:.2} px entre colunas"
        );
    }
}

/// ⭐ O Paper Edge põe o DENTE do papel na borda: sem Ragged, o desvio segue a rugosidade do papel
/// (medido: nenhum `0,45` · Hot Press `1,77` · Rough `2,59` px), e a `0` os papéis não mudam a borda.
///
/// **Mutação que tem de sangrar:** apagar a parcela `papel` do `Fonte::desloca`.
#[test]
fn o_paper_edge_poe_o_dente_do_papel_na_borda() {
    let com = |papel: TextureKind, pe: f32| {
        desvio_da_borda(&faixa(|t| {
            t.paint.brush.warp = 0.0;
            t.set_brush_paper_kind(papel.to_u8());
            t.set_brush_paper_edge(pe);
        }))
    };
    let (rough0, hot0) = (
        com(TextureKind::PaperRough, 0.0),
        com(TextureKind::PaperHot, 0.0),
    );
    let (rough, hot) = (
        com(TextureKind::PaperRough, 1.0),
        com(TextureKind::PaperHot, 1.0),
    );
    assert!(
        (rough0 - hot0).abs() < 0.2,
        "a 0 o papel não toca na borda ({rough0:.2} × {hot0:.2})"
    );
    assert!(
        rough > rough0 + 1.5,
        "Rough a 1 tem de quebrar a borda ({rough:.2} contra {rough0:.2})"
    );
    assert!(
        rough > hot * 1.25,
        "o Rough tem de quebrar MAIS que o Hot ({rough:.2} × {hot:.2})"
    );
}

/// A faixa escura da borda escurece nos VALES do papel também com Rewet: o termo molhado puxava para
/// os picos, contra a granulação e o Tooth (o mesmo defeito do #30, na borda).
///
/// **Mutação que tem de sangrar:** voltar o `ragged` do `watercolor_render` a `(paper_h − 0,5)`.
#[test]
fn com_rewet_a_faixa_escura_assenta_nos_vales() {
    let t = faixa(|t| {
        t.paint.brush.warp = 0.0;
        t.set_brush_paper_kind(TextureKind::PaperRough.to_u8());
        t.set_brush_wet_rewet(1.0);
    });
    let p = t.paint.brush.paper;
    let rot = ph2d_painter_brush::texture::angle_basis(p.angle_deg);
    let (mut hs, mut ds) = (Vec::new(), Vec::new());
    for cx in 70..186_i64 {
        let topo = (0..256_i64)
            .find(|&y| t.canvas_rgba[((y * 256 + cx) * 4) as usize] < 245)
            .unwrap_or(256);
        // A faixa: os 5 texels logo abaixo da borda.
        for y in topo + 1..topo + 6 {
            let i = ((y * 256 + cx) * 4) as usize;
            let px = &t.canvas_rgba[i..i + 3];
            ds.push(255.0 - px.iter().map(|&c| f64::from(c)).sum::<f64>() / 3.0);
            hs.push(f64::from(
                ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                    &p,
                    cx,
                    y,
                    None,
                    rot,
                    [0.0, 0.0],
                ),
            ));
        }
    }
    let n = hs.len() as f64;
    let (mh, md) = (hs.iter().sum::<f64>() / n, ds.iter().sum::<f64>() / n);
    let cov: f64 = hs.iter().zip(&ds).map(|(h, d)| (h - mh) * (d - md)).sum();
    let r = cov
        / (hs.iter().map(|h| (h - mh).powi(2)).sum::<f64>()
            * ds.iter().map(|d| (d - md).powi(2)).sum::<f64>())
        .sqrt();
    eprintln!("FLOW-REWET correlação faixa×altura = {r:.3}");
    assert!(
        r < -0.2,
        "a faixa escura tem de assentar nos VALES (correlação {r:.3})"
    );
}

/// ⭐ Trocar o Flow para o traço seguinte NÃO reforma o traço anterior da mesma sessão molhada — o
/// estilo da borda é por dono, como o papel (#13).
///
/// **Mutação que tem de sangrar:** ler o `edge_flow` do pincel vivo em vez do dono (`Estilo::de(cur)`
/// para todos no `EdgeFlow::build`).
#[test]
fn trocar_o_flow_nao_reforma_o_traco_anterior() {
    let sessao = |flow_b: TextureKind| {
        // Afastados mais que o raio de suavização da junção (8 px): A ocupa ~2..78, B ~122..198.
        let mut t = traco(200, 40.0, |t| t.paint.brush.warp = 24.0);
        t.set_brush_edge_flow_kind(flow_b.to_u8());
        risca(&mut t, 160.0, 200);
        t
    };
    let a = sessao(TextureKind::None);
    let b = sessao(TextureKind::Wood);
    let linhas = |t: &PainterTool| t.canvas_rgba[0..(90 * 200 * 4)].to_vec();
    assert_eq!(
        linhas(&a),
        linhas(&b),
        "o traço de cima mudou quando o de baixo trocou de Flow"
    );
    assert_ne!(
        a.canvas_rgba, b.canvas_rgba,
        "o traço de baixo tem de ter outra borda"
    );
}

/// A sequência inteira pelo painel: o `SelectOption` do Flow chega à ferramenta e muda a borda; o Size,
/// o Angle e o Paper Edge chegam pelo `SetValue`.
#[test]
fn o_flow_escolhido_pelo_painel_muda_a_borda() {
    use ph2d_editor_core::tool::PanelEvent;
    let clouds = TextureKind::Clouds.to_u8();
    let classic = faixa(|t| t.paint.brush.warp = 24.0);
    let t = faixa(|t| {
        t.paint.brush.warp = 24.0;
        t.handle_panel_event(PanelEvent::SelectOption(
            crate::ids::PAINTER_WATERCOLOR_FLOW_KIND,
            clouds.to_string(),
        ));
        t.handle_panel_event(PanelEvent::SetValue(
            crate::ids::PAINTER_WATERCOLOR_FLOW_SIZE,
            2.0,
        ));
        t.handle_panel_event(PanelEvent::SetValue(
            crate::ids::PAINTER_WATERCOLOR_FLOW_ANGLE,
            30.0,
        ));
        t.handle_panel_event(PanelEvent::SetValue(
            crate::ids::PAINTER_WATERCOLOR_PAPER_EDGE,
            0.5,
        ));
    });
    let f = t.paint.brush.edge_flow;
    assert_eq!(f.kind, TextureKind::Clouds);
    assert_eq!(f.size, [2.0, 2.0]);
    assert_eq!(f.angle_deg, 30);
    assert_eq!(t.paint.brush.paper_edge, 0.5);
    assert_ne!(t.canvas_rgba, classic.canvas_rgba);
    let s = t.brush_settings();
    assert_eq!(
        (s.flow_kind, s.flow_size, s.flow_angle, s.paper_edge),
        (clouds, 2.0, 30, 0.5)
    );
}

/// Os limites dos controles novos.
#[test]
fn os_controles_do_flow_respeitam_os_limites() {
    let mut t = PainterTool::default();
    t.set_brush_edge_flow_size(99.0);
    assert_eq!(
        t.paint.brush.edge_flow.size,
        [ph2d_painter_brush::FLOW_SIZE_MAX; 2]
    );
    t.set_brush_edge_flow_size(0.0);
    assert_eq!(
        t.paint.brush.edge_flow.size,
        [ph2d_painter_brush::FLOW_SIZE_MIN; 2]
    );
    t.set_brush_paper_edge(3.0);
    assert_eq!(t.paint.brush.paper_edge, 1.0);
    t.set_brush_edge_flow_angle(400.0);
    assert_eq!(t.paint.brush.edge_flow.angle_deg, 40);
    assert_eq!(
        ph2d_painter_brush::BrushSpec::default().edge_flow.kind,
        TextureKind::None
    );
}

/// **O que o Flow e o Paper Edge cobram por quadro** — MOVE + o fecho do quadro (desde 2026-08-02 a
/// recomposição da aquarela é devida ao `paint_tick`, então medir só o move mede outra coisa). 2048²,
/// r = 100, Ragged 24, mediana de 20 quadros. A 1.ª e a 2.ª linha são a MESMA configuração: a diferença
/// entre elas é o piso de ruído da sonda. Rode em `--release` (o número de DEBUG não vale nada).
#[test]
#[ignore = "measurement, not a gate"]
fn measure_what_the_edge_flow_costs() {
    const SIZE: u32 = 2048;
    const R: f32 = 100.0;
    let quadro_ms = |ajusta: &dyn Fn(&mut PainterTool)| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        t.set_paint_media(PaintMedia::Watercolor);
        t.paint.brush.radius_px = R;
        t.paint.brush.warp = 24.0;
        ajusta(&mut t);
        let mid = (SIZE / 2) as f32;
        t.on_canvas_pointer(cp([R + 20.0, mid], PointerPhase::Down));
        frame(&mut t);
        let mut v = Vec::new();
        for i in 1..=20 {
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([R + 20.0 + 40.0 * i as f32, mid], PointerPhase::Move));
            frame(&mut t);
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let k = |k: TextureKind| k.to_u8();
    type Linha<'a> = (&'a str, Box<dyn Fn(&mut PainterTool)>);
    let linhas: Vec<Linha> = vec![
        ("Classic", Box::new(|_| {})),
        ("Classic (controlo)", Box::new(|_| {})),
        (
            "Flow Clouds",
            Box::new(move |t| t.set_brush_edge_flow_kind(k(TextureKind::Clouds))),
        ),
        (
            "Flow Musgrave",
            Box::new(move |t| t.set_brush_edge_flow_kind(k(TextureKind::Musgrave))),
        ),
        (
            "Flow Voronoi",
            Box::new(move |t| t.set_brush_edge_flow_kind(k(TextureKind::Voronoi))),
        ),
        (
            "Paper Edge 1 (Rough)",
            Box::new(move |t| {
                t.set_brush_paper_kind(k(TextureKind::PaperRough));
                t.set_brush_paper_edge(1.0);
            }),
        ),
        (
            "Clouds + Paper Edge 1",
            Box::new(move |t| {
                t.set_brush_edge_flow_kind(k(TextureKind::Clouds));
                t.set_brush_paper_kind(k(TextureKind::PaperRough));
                t.set_brush_paper_edge(1.0);
            }),
        ),
    ];
    let base = quadro_ms(&|_| {});
    println!(
        "\n{:<26} {:>10} {:>10}",
        "configuração", "quadro ms", "vs Classic"
    );
    for (nome, f) in &linhas {
        let ms = quadro_ms(f.as_ref());
        println!("{nome:<26} {ms:>10.3} {:>10.3}", ms - base);
    }
}

/// Uma luminância de listras diagonais — um mapa de fluxo com DIREÇÃO, que se reconhece na borda.
fn listras(w: u32, h: u32) -> Vec<u8> {
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as f32, (i / w) as f32);
            (127.5 + 127.0 * ((x + y) / 9.0).sin()) as u8
        })
        .collect()
}

/// ⭐ **"Use as Flow"**: a porta do menu instala a camada como o mapa do Flow, e a borda passa a segui-lo.
/// Sem a imagem carregada, um Flow `Image` cai no Classic AO BYTE (a régua gravada antes da wave).
///
/// **Mutação que tem de sangrar:** o `use_layers_as` mandar o `Fluxo` para o papel.
#[test]
fn usar_a_camada_como_flow_desenha_a_borda_por_ela() {
    use crate::tool::UsoDaCamada;
    let classic = traco(128, 64.0, |t| t.paint.brush.warp = 24.0);
    let com_imagem = traco(128, 64.0, |t| {
        t.paint.brush.warp = 24.0;
        t.use_layers_as(UsoDaCamada::Fluxo, listras(128, 128), 128, 128);
    });
    assert_eq!(com_imagem.paint.brush.edge_flow.kind, TextureKind::Image);
    assert!(
        com_imagem.paint.flow_map.imagem.is_some(),
        "a imagem do Flow não foi guardada"
    );
    assert_ne!(
        com_imagem.canvas_rgba, classic.canvas_rgba,
        "a borda ignorou o mapa"
    );
    let sem_imagem = traco(128, 64.0, |t| {
        t.paint.brush.warp = 24.0;
        t.paint.brush.edge_flow.kind = TextureKind::Image;
    });
    assert_eq!(
        fnv(&sem_imagem.canvas_rgba),
        BASE[1].1,
        "um Flow Image sem imagem tem de ser o Classic ao byte"
    );
}
