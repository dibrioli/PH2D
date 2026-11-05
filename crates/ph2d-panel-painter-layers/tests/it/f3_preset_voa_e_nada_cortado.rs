//! Fatia F3: a linha da lista de presets voa como o cartão do Kanban, e a fila «On switch» + Sync nunca
//! corta a opção mais longa nem o Sync (lei 5 e 7 de 05 §4).

use super::h1_os_cartoes::{Mesa, pincel};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_i18n::tr;
use ph2d_panel_painter_layers::{
    PainterLayersPanel, VistaPreset, VistaPresets, set_current_wet_presets,
};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::ids as p;
use std::collections::BTreeMap;
use std::sync::Arc;

const CHIP: ph2d_a11y::NodeId = p::PAINTER_WETPAINT_PRESET;
const SECAO_GRUPO: u8 = 2;
const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 2400.0);
/// Quanto o cursor anda para o lado, para o voo sair de cima da lista.
const AO_LADO: f32 = 600.0;
/// A marca de acento e o seu antialias no topo do vão.
const MARCA_PX: u32 = 3;
const RELS: [&str; 5] = [
    "WetPainter/Oil/A.ph2d-brush.png",
    "WetPainter/Oil/B.ph2d-brush.png",
    "WetPainter/Oil/C.ph2d-brush.png",
    "WetPainter/Oil/D.ph2d-brush.png",
    "WetPainter/Ink/E.ph2d-brush.png",
];

struct ComVista;

impl ComVista {
    /// Quatro presets de Oil e um de Ink, o primeiro activo.
    fn nova() -> Self {
        let nomes = [
            "Flat Bristle",
            "Round Bristle",
            "Palette Knife",
            "Filbert",
            "Pen",
        ];
        let entradas = RELS
            .iter()
            .zip(nomes)
            .enumerate()
            .map(|(i, (rel, nome))| VistaPreset {
                relativo: (*rel).to_owned(),
                nome: nome.to_owned(),
                categoria: if i < 4 { "Oil" } else { "Ink" }.to_owned(),
                favorito: false,
                apagavel: false,
            })
            .collect();
        set_current_wet_presets(Some(Arc::new(VistaPresets {
            entradas,
            ativo: Some(RELS[0].to_owned()),
            politica: 0,
            ilegiveis: 0,
            ordem_manual: false,
        })));
        Self
    }
}

impl Drop for ComVista {
    fn drop(&mut self) {
        set_current_wet_presets(None);
    }
}

fn linha(i: usize) -> ph2d_a11y::NodeId {
    p::painter_wetpaint_preset_option_id(SECAO_GRUPO, RELS[i])
}

fn lista_aberta() -> Mesa {
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mut mesa = Mesa::na_janela(true, b, false, JANELA);
    mesa.so_abertos(&[p::PAINTER_CARD_TOPO]);
    mesa.assenta();
    mesa.clica(CHIP);
    mesa.assenta();
    mesa
}

/// Agarra a linha `de` e leva o cursor à metade de cima (`depois` = de baixo) da linha `para`.
fn arrasta(mesa: &mut Mesa, de: usize, para: usize, depois: bool) -> (f32, f32) {
    let r = mesa.rect(linha(de)).expect("a linha agarrada");
    let t = mesa.rect(linha(para)).expect("a linha alvo");
    let ev = mesa.ponteiro(PointerKind::Down, r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = mesa.aplica(ev);
    let y = t.y + t.h * if depois { 0.75 } else { 0.25 };
    let ev = mesa.ponteiro(PointerKind::Move, r.x + r.w * 0.5, y);
    let _ = mesa.aplica(ev);
    (r.x + r.w * 0.5, y)
}

fn tinta(mesa: &mut Mesa) -> BTreeMap<u32, i32> {
    mesa.host.store_mut().set_hover_live(CHIP, 0.0);
    let dados = mesa
        .host
        .paint_draw_data::<PainterLayersPanel>(&mut mesa.st, mesa.janela);
    let mut contagem = BTreeMap::new();
    for cor in dados {
        *contagem.entry(cor).or_insert(0) += 1;
    }
    contagem
}

/// A cor como o Vello a grava nos dados de desenho.
fn gravada(t: ColorToken) -> u32 {
    resolve(t, Theme::default())
        .premultiply()
        .to_rgba8()
        .to_u32()
}

/// Quantos desenhos de `t` a mais em `b`.
fn a_mais(a: &BTreeMap<u32, i32>, b: &BTreeMap<u32, i32>, t: ColorToken) -> i32 {
    let c = gravada(t);
    b.get(&c).unwrap_or(&0) - a.get(&c).unwrap_or(&0)
}

fn letras(mesa: &mut Mesa) -> u32 {
    let (st, j) = (&mut mesa.st, mesa.janela);
    mesa.host
        .paint_and_count_geometry::<PainterLayersPanel>(st, j)
        .0
}

/// A meio do arrasto: a marca, as duas sombras do voo e as mesmas letras, porque o vão tirou o nome que a
/// linha na mão pôs. Os alvos não dançam com a ordem prevista, e largar leva o pedido de sempre.
#[test]
fn the_dragged_preset_row_flies_opens_a_gap_and_drops_as_before() {
    let _v = ComVista::nova();
    let mut mesa = lista_aberta();
    let parada = tinta(&mut mesa);
    let letras_paradas = letras(&mut mesa);
    let antes: Vec<_> = (0..5).map(|i| mesa.rect(linha(i)).unwrap()).collect();

    let (x, y) = arrasta(&mut mesa, 1, 3, false);
    mesa.assenta();
    let voando = tinta(&mut mesa);
    assert_eq!(a_mais(&parada, &voando, ColorToken::Accent), 1, "a marca");
    assert_eq!(
        a_mais(&parada, &voando, ColorToken::KanbanShadowFlight),
        1,
        "sombra"
    );
    assert_eq!(
        a_mais(&parada, &voando, ColorToken::KanbanFlightNear),
        1,
        "sombra perto"
    );
    assert_eq!(
        letras(&mut mesa),
        letras_paradas,
        "o vão tira o nome que voa"
    );

    mesa.quadro();
    for (i, r) in antes.iter().enumerate() {
        assert_eq!(mesa.rect(linha(i)), Some(*r), "a linha {i} no arrasto");
    }

    let ev = mesa.ponteiro(PointerKind::Up, x, y);
    let acoes = mesa.aplica(ev).join(" ");
    assert!(
        acoes.contains(RELS[1]) && acoes.contains(RELS[3]),
        "{acoes}"
    );
    assert!(acoes.contains("before"), "{acoes}");
}

/// Noutra secção a ponte recusa a queda: o voo vê-se, a marca não.
#[test]
fn a_preset_row_over_another_category_shows_no_drop_mark() {
    let _v = ComVista::nova();
    let mut mesa = lista_aberta();
    let parada = tinta(&mut mesa);
    arrasta(&mut mesa, 1, 4, false);
    mesa.assenta();
    let voando = tinta(&mut mesa);
    assert_eq!(a_mais(&parada, &voando, ColorToken::Accent), 0, "sem marca");
    assert_eq!(
        a_mais(&parada, &voando, ColorToken::KanbanShadowFlight),
        1,
        "voa"
    );
}

/// A grade do visor (8 anéis e 12 raios) na tinta que se vê no cartão; o painel de antes fica como era.
#[test]
fn the_tilt_grid_reads_on_the_card_ink() {
    let mut b = pincel(PaintMedia::WetPaint, "brush").0;
    b.wet_tilt_on = true;
    let mut mesa = Mesa::na_janela(true, b, false, JANELA);
    mesa.so_abertos(&[p::PAINTER_WETPAINT_SECTION]);
    mesa.assenta();
    let t = tinta(&mut mesa);
    let n = t
        .get(&gravada(ColorToken::TextDisabled))
        .copied()
        .unwrap_or(0);
    println!("MEDIDA TextDisabled: {n}");
    assert!(n >= 20, "a grade: {n} desenhos");
    let mut antes = Mesa::na_janela(false, b, false, JANELA);
    antes.assenta();
    let t = tinta(&mut antes);
    let borda = t.get(&gravada(ColorToken::Border)).copied().unwrap_or(0);
    println!("MEDIDA painel de antes, Border: {borda}");
    assert!(borda >= 20, "a grade do painel de antes: {borda} desenhos");
}

/// A fila On switch + Sync: cada opção da lista e o Sync cabem inteiros, em qualquer largura.
#[test]
fn the_on_switch_row_never_cuts_its_longest_option_nor_sync() {
    let _v = ComVista::nova();
    let mut texto = ph2d_text::TextSystem::without_system_fonts();
    let longa = [
        "panel.painter_layers.wetpaint.policy_volumes_and_mode",
        "panel.painter_layers.wetpaint.policy_volumes",
        "panel.painter_layers.wetpaint.policy_none",
    ]
    .iter()
    .map(|k| texto.prefix_width(tr(k), m::VALUE_PX).ceil())
    .fold(0.0_f32, f32::max);
    let pede = longa + m::FIELD_PAD_R * 2.0 + m::ICON_GLYPH;
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    for w in [1100.0, 1600.0, 2400.0, 3840.0] {
        let mut mesa = Mesa::na_janela(true, b, false, Rect::new(0.0, 0.0, w, 2400.0));
        mesa.so_abertos(&[p::PAINTER_CARD_TOPO]);
        mesa.assenta();
        let lista = mesa
            .rect(p::PAINTER_WETPAINT_PRESET_POLICY)
            .expect("On switch");
        let sync = mesa.rect(p::PAINTER_BRUSH_SYNC).expect("Sync");
        println!("MEDIDA janela {w}: a lista dá {} e pede {pede}", lista.w);
        assert!(lista.w >= pede, "janela {w}: {} < {pede}", lista.w);
        assert!(
            sync.w >= texto.prefix_width(tr("card.painter.brush.sync"), m::VALUE_PX),
            "janela {w}: o Sync {}",
            sync.w
        );
    }
}

/// O vão é fundo da lista, liso, mesmo com um controlo por baixo: o voo fica ao lado, para não o tapar.
#[test]
fn the_gap_is_list_background_over_the_control_below() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("sem GPU: gate de placa saltado");
        return;
    };
    let _v = ComVista::nova();
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mut fechada = Mesa::na_janela(true, b, false, JANELA);
    fechada.so_abertos(&[p::PAINTER_CARD_TOPO]);
    fechada.assenta();
    let controlo = fechada
        .rect(p::PAINTER_WETPAINT_PRESET_EXPORT)
        .expect("Export");

    let mut mesa = lista_aberta();
    // Largada antes da linha 3: o vão fica no lugar da linha 2.
    let vao = mesa.rect(linha(2)).expect("o lugar do vão");
    assert!(
        vao.y < controlo.y + controlo.h && controlo.y < vao.y + vao.h,
        "o vão {vao:?} não passa por cima do controlo {controlo:?}"
    );
    let (x, y) = arrasta(&mut mesa, 1, 3, false);
    let ev = mesa.ponteiro(PointerKind::Move, x - AO_LADO, y);
    let _ = mesa.aplica(ev);
    mesa.assenta();
    let (px, _) = super::h10_fotos::pixeis(&gpu, &mut mesa);
    let cor = |x: f32, y: f32| {
        let i = (y as usize * JANELA.w as usize + x as usize) * 4;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    };
    let topo = mesa.rect(linha(0)).expect("a 1.ª linha");
    let fundo = cor(vao.x + vao.w - m::SEG_INSET, topo.y - m::SEG_INSET);
    let mut outros = 0;
    let (x0, x1) = (vao.x as u32 + 1, (vao.x + vao.w) as u32 - 1);
    // Por baixo da marca de acento, até à linha seguinte.
    let (y0, y1) = (vao.y as u32 + MARCA_PX, (vao.y + vao.h) as u32 - 1);
    for yy in y0..y1 {
        for xx in x0..x1 {
            outros += u32::from(cor(xx as f32, yy as f32) != fundo);
        }
    }
    println!("MEDIDA fundo {fundo:?}, píxeis do vão diferentes: {outros}");
    assert_eq!(outros, 0, "o vão não é o fundo da lista");
}

/// `PH2D_FOTO_DIR=<pasta>`, `--ignored`: a meio do arrasto, a fila On switch e o visor do Tilt.
#[test]
#[ignore = "fotos: PH2D_FOTO_DIR=<pasta> e --ignored"]
fn f3_foto() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    let _v = ComVista::nova();
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mut fechada = Mesa::na_janela(true, b, false, JANELA);
    fechada.so_abertos(&[p::PAINTER_CARD_TOPO]);
    fechada.assenta();
    super::h10_fotos::foto(&gpu, &dir, "f3_on_switch", &mut fechada);
    let mut mesa = lista_aberta();
    super::h10_fotos::foto(&gpu, &dir, "f3_on_switch_lista_aberta", &mut mesa);
    arrasta(&mut mesa, 1, 3, false);
    mesa.assenta();
    super::h10_fotos::foto(&gpu, &dir, "f3_preset_a_voar", &mut mesa);
    let (x, y) = (
        mesa.janela.w * 0.5,
        mesa.rect(linha(2)).map_or(0.0, |r| r.y + r.h * 1.25),
    );
    let ev = mesa.ponteiro(PointerKind::Move, x, y);
    let _ = mesa.aplica(ev);
    mesa.assenta();
    super::h10_fotos::foto(&gpu, &dir, "f3_preset_vao_livre", &mut mesa);
    let mut fora = lista_aberta();
    arrasta(&mut fora, 1, 4, false);
    fora.assenta();
    super::h10_fotos::foto(&gpu, &dir, "f3_preset_fora_da_secao", &mut fora);

    let mut tilt = pincel(PaintMedia::WetPaint, "brush").0;
    tilt.wet_tilt_on = true;
    let mut m = Mesa::na_janela(true, tilt, false, JANELA);
    m.so_abertos(&[p::PAINTER_WETPAINT_SECTION]);
    m.assenta();
    super::h10_fotos::foto(&gpu, &dir, "f3_tilt", &mut m);
    let mut mascara = pincel(PaintMedia::Digital, "brush").0;
    mascara.is_mask = true;
    let mut m = Mesa::na_janela(true, mascara, false, JANELA);
    m.so_abertos(&[p::PAINTER_MASK_SECTION]);
    m.assenta();
    super::h10_fotos::foto(&gpu, &dir, "f3_mascara", &mut m);
}
