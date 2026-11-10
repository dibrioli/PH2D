//! Pedidos do dono 10/11 sobre a letra da aba, lidos na cena (fonte, corpo, eixo do peso, posição):
//! o nome na lista e as entradas do «In this script» na letra exacta de «alive» (a variável do
//! quadro parado); o «Who has what» no título dos cartões; Console | Problems na letra da faixa
//! do «Paused».

use super::{GRADE, quem};
use ph2d_a11y::NodeId;
use ph2d_code_buffer::Buffer;
use ph2d_code_debugcard::{CardState, DebugCard, ValueKind, VarRow};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_panel_code::bancada::{Aim, BancadaView};
use ph2d_panel_code::debug_card::DebugCardView;
use ph2d_panel_code::{
    Band, BandTab, CodeView, ConsoleRow, ConsoleSource, ConsoleView, Ink, LineCache, ListHit,
    ListRow, ScriptList, ScriptListView,
};
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ROW_H_PX, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// Uma corrida de glifos: (blob da fonte, índice, corpo, eixos, ids, (x, y) de cada glifo).
type Corrida = (u64, u32, f32, Vec<i16>, Vec<u32>, Vec<(f32, f32)>);

fn corridas(scene: &VectorScene) -> Vec<Corrida> {
    let r = &scene.inner().encoding().resources;
    r.glyph_runs
        .iter()
        .map(|g| {
            let [tx, ty] = g.transform.translation;
            let gl = &r.glyphs[g.glyphs.clone()];
            (
                g.font.data.id(),
                g.font.index,
                g.font_size,
                r.normalized_coords[g.normalized_coords.clone()].to_vec(),
                gl.iter().map(|x| x.id).collect(),
                gl.iter().map(|x| (tx + x.x, ty + x.y)).collect(),
            )
        })
        .collect()
}

/// A letra de uma corrida (fonte, índice, corpo, eixos) — o que «a mesma fonte» quer dizer.
fn letra(c: &Corrida) -> (u64, u32, f32, Vec<i16>) {
    (c.0, c.1, c.2, c.3.clone())
}

/// Os ids de `text` moldado na mono do código.
fn ids_mono(ts: &mut TextSystem, text: &str) -> Vec<u32> {
    let mut cache = LineCache::default();
    let px = ph2d_text::displayed_font_px(TypeToken::Base.px());
    let layout = cache.get(ts, text, &[], px);
    let mut ids = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            if let ph2d_text::PositionedLayoutItem::GlyphRun(run) = item {
                ids.extend(run.glyphs().map(|g| g.id));
            }
        }
    }
    ids
}

/// Os ids de `text` na letra da interface em `weight`.
fn ids_ui(ts: &mut TextSystem, text: &str, px: f32, weight: FontWeight) -> Vec<u32> {
    let mut sc = VectorScene::new();
    let ink = Color::from_rgba8(255, 255, 255, 255);
    ph2d_editor_core::paint::paint_text_in_weight(
        ts,
        &mut sc,
        text,
        [0.0, 0.0],
        px,
        f32::INFINITY,
        ink,
        weight,
    );
    corridas(&sc).into_iter().flat_map(|c| c.4).collect()
}

fn achar<'a>(cena: &'a [Corrida], ids: &[u32], onde: &str) -> &'a Corrida {
    cena.iter()
        .find(|c| c.4 == ids)
        .unwrap_or_else(|| panic!("{onde}: nenhuma corrida com estes glifos"))
}

/// Um `CodeView` com a mono medida (uma pintura de aquecimento do código, como na aba).
fn medido(ts: &mut TextSystem) -> CodeView {
    let mut code = CodeView::new();
    let buf = Buffer::new("a\n");
    let aba = Rect::new(0.0, 0.0, 600.0, 400.0);
    let _ = crate::pinta(&mut code, &buf, ts, aba, &crate::Deco::default());
    code
}

/// O quadro parado com `alive` nas variáveis.
fn quadro(ts: &mut TextSystem) -> Vec<Corrida> {
    let c = DebugCard {
        state: CardState::Paused,
        line: 18,
        function: "update".into(),
        locals: vec![VarRow {
            key: "alive".into(),
            name: "alive".into(),
            depth: 0,
            value: "true".into(),
            kind: ValueKind::Boolean,
            field_count: 0,
            expandable: false,
            old: None,
        }],
        can_step: true,
        ..DebugCard::default()
    };
    let (mut scene, mut v) = (VectorScene::new(), DebugCardView::default());
    let aba = Rect::new(0.0, 0.0, 1100.0, 900.0);
    let h = v.height(&c, aba);
    let r = Rect::new(0.0, aba.h - h, aba.w, h);
    let ids = [NodeId(1), NodeId(2)];
    let mut code = medido(ts);
    v.paint(&c, ids, r, &mut code, &mut scene, ts, Theme::default());
    corridas(&scene)
}

/// `text` pintado por quem pinta «alive», centrado na linha `r`: a posição de cada glifo.
fn como_alive(ts: &mut TextSystem, text: &str, x: f32, r: Rect) -> Vec<(f32, f32)> {
    let mut sc = VectorScene::new();
    let (row, inf) = ((x, r.y, r.h), f32::INFINITY);
    let mut view = medido(ts);
    view.paint_code_value(ts, &mut sc, Theme::default(), text, Ink::Hint, row, inf);
    corridas(&sc).into_iter().flat_map(|c| c.5).collect()
}

/// (1a, 1b) O nome na lista de scripts e uma entrada do «In this script» saem na fonte, no corpo
/// e no peso de «alive», centrados na linha pela mesma conta; os cliques continuam a cair nelas.
#[test]
fn o_nome_do_script_e_as_entradas_tem_a_letra_de_alive() {
    let mut ts = TextSystem::without_system_fonts();
    let quadro = quadro(&mut ts);
    let alive = letra(achar(&quadro, &ids_mono(&mut ts, "alive"), "«alive»"));
    // A lista: o nome «alive.luau».
    let filtro = Buffer::new("");
    let linhas = [ListRow {
        id: 1,
        name: "alive.luau",
        dirty: false,
        current: false,
        users: 1,
    }];
    let lista = ScriptList {
        open: true,
        filter: &filtro,
        focus: None,
        rows: &linhas,
        renaming: None,
        notice: None,
        hover_count: None,
    };
    let (mut l, mut s) = (ScriptListView::default(), VectorScene::new());
    let la = Rect::new(0.0, 0.0, 240.0, ScriptListView::height_for(1));
    l.paint(&lista, la, &mut s, &mut ts, &Theme::default());
    let row = l.target_rect(ListHit::Row(1)).expect("a linha é alvo");
    let cena = corridas(&s);
    let nome = achar(&cena, &ids_mono(&mut ts, "alive.luau"), "lista");
    assert_eq!(
        letra(nome),
        alive,
        "o nome na lista não tem a letra de «alive»"
    );
    let x0 = nome.5[0].0;
    assert_eq!(
        nome.5,
        como_alive(&mut ts, "alive.luau", x0, row),
        "centrar"
    );
    let (cx, cy) = (row.x + row.w * 0.25, row.y + row.h * 0.5);
    assert_eq!(
        l.hit(&mut ts, cx, cy),
        Some(ListHit::Row(1)),
        "o clique na linha"
    );
    assert!(l.fit_w() >= ph2d_panel_code::mono_label::width(&mut ts, "alive.luau"));
    // O «In this script»: a entrada `update`.
    let (who, mut v, mut s) = (quem(), BancadaView::default(), VectorScene::new());
    let area = Rect::new(0.0, 0.0, 300.0, 600.0);
    v.paint_outline(&who, area, &mut s, &mut ts, &Theme::default());
    let cena = corridas(&s);
    let up = achar(&cena, &ids_mono(&mut ts, "update"), "entrada");
    assert_eq!(letra(up), alive, "a entrada não tem a letra de «alive»");
    let r = v.target(Aim::Entry(20)).expect("a entrada `update` é alvo");
    assert_eq!(up.5, como_alive(&mut ts, "update", up.5[0].0, r), "centrar");
    // A secção «Functions» fica na letra da interface.
    let titulo = ids_ui(
        &mut ts,
        tr("code.bancada.outline.functions"),
        TypeToken::Sm.px(),
        FontWeight::NORMAL,
    );
    assert!(
        cena.iter().any(|c| c.4 == titulo),
        "a secção fica como estava"
    );
}

/// (2) «Who has what» no título dos cartões da coluna (MAIÚSCULAS espaçadas, corpo e cor da
/// lista), na mesma altura; a contagem encosta à largura do título novo.
#[test]
fn o_titulo_who_has_what_e_o_dos_cartoes() {
    use ph2d_editor_core::panel::cards::banda::metrics as m;
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let (who, mut ts) = (quem(), TextSystem::without_system_fonts());
    let mut v = BancadaView::default();
    let mut s = VectorScene::new();
    let theme = Theme::default();
    v.paint_bench(
        &who,
        GRADE,
        None,
        &mut CodeView::new(),
        &mut s,
        &mut ts,
        &theme,
    );
    let cena = corridas(&s);
    let mut o = VectorScene::new();
    v.paint_outline(
        &who,
        Rect::new(0.0, 0.0, 300.0, 600.0),
        &mut o,
        &mut ts,
        &theme,
    );
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    // A referência: o título pintado pela porta dos cartões no sítio do da grelha.
    let g = ph2d_tokens::Spacing::Sm.px();
    let title = tr("code.bancada.grid.title").to_uppercase();
    let y = GRADE.y + g + ph2d_editor_core::paint::folga_do_texto(ROW_H_PX, m::LIST_TITLE_PX);
    let mut r = VectorScene::new();
    let ink = ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::KanbanListFg, theme);
    ph2d_editor_core::paint::paint_text_title_elided_tracked(
        &mut ts,
        &mut r,
        &title,
        [GRADE.x + g, y],
        m::LIST_TITLE_PX,
        f32::INFINITY,
        ink,
        m::LIST_TITLE_TRACKING_EM,
    );
    let refe = corridas(&r);
    assert_eq!(refe.len(), 1, "controlo: uma corrida");
    let achada = achar(&cena, &refe[0].4, "«WHO HAS WHAT»");
    assert_eq!(achada, &refe[0], "o título não é o dos cartões");
    let card = tr("code.bancada.outline.title").to_uppercase();
    let mut c = VectorScene::new();
    ph2d_editor_core::paint::paint_text_title_elided_tracked(
        &mut ts,
        &mut c,
        &card,
        [0.0, 0.0],
        m::LIST_TITLE_PX,
        f32::INFINITY,
        ink,
        m::LIST_TITLE_TRACKING_EM,
    );
    let ids_card = corridas(&c).remove(0).4;
    let outline = corridas(&o);
    let cartao = achar(&outline, &ids_card, "«IN THIS SCRIPT»");
    assert_eq!(letra(achada), letra(cartao), "a letra do «IN THIS SCRIPT»");
    // A contagem começa depois do título medido como se pinta.
    let tw = ph2d_editor_core::paint::title_tracked_width(
        &mut ts,
        &title,
        m::LIST_TITLE_PX,
        m::LIST_TITLE_TRACKING_EM,
    );
    let depois = GRADE.x + g + tw;
    let lx = achada.5.last().map_or(0.0, |p| p.0);
    let contagem = cena
        .iter()
        .filter(|c| c.4 != achada.4 && (c.5[0].1 - achada.5[0].1).abs() < 2.0)
        .map(|c| c.5[0].0)
        .find(|x| *x > lx)
        .expect("a contagem ao lado");
    assert!(
        contagem >= depois,
        "a contagem {contagem} entra no título ({depois})"
    );
    assert!(
        contagem < depois + ph2d_tokens::Spacing::Lg.px(),
        "{contagem} × {depois}"
    );
}

struct Vazia;
impl ConsoleSource for Vazia {
    fn len(&self) -> usize {
        0
    }
    fn row(&self, _: usize) -> ConsoleRow<'_> {
        unreachable!("vazia")
    }
}

/// (3) Console | Problems na letra da faixa do quadro parado: o activo no peso do «Paused», o
/// outro no do corpo da faixa («line»), no mesmo corpo; os alvos medem o que se pinta.
#[test]
fn os_separadores_tem_a_letra_da_faixa_do_paused() {
    let mut ts = TextSystem::without_system_fonts();
    let quadro = quadro(&mut ts);
    let sm = TypeToken::Sm.px();
    let semi = FontWeight::SEMI_BOLD;
    let paused = tr("code.debug.paused");
    let paused = letra(achar(
        &quadro,
        &ids_ui(&mut ts, paused, sm, semi),
        "«Paused»",
    ));
    let word = tr("code.debug.line_word");
    let normal = FontWeight::NORMAL;
    let corpo = letra(achar(&quadro, &ids_ui(&mut ts, word, sm, normal), "«line»"));
    assert_ne!(paused, corpo, "controlo: a sonda separa os pesos");
    for (tab, activo, outro) in [(BandTab::Console, 0, 1), (BandTab::Problems, 1, 0)] {
        let band = Band {
            open: true,
            tab,
            console: &Vazia,
            problem_count: 0,
            problems: &[],
            current: None,
            tab_width: 4,
        };
        let (mut c, mut s) = (ConsoleView::default(), VectorScene::new());
        let faixa = Rect::new(0.0, 0.0, 800.0, ROW_H_PX * 6.0);
        let mut view = CodeView::new();
        c.paint(band, faixa, &mut view, &mut s, &mut ts, &Theme::default());
        let cena = corridas(&s);
        let p = c.probe();
        let (la, lo) = (&p.tabs[activo].0, &p.tabs[outro].0);
        let a = achar(&cena, &ids_ui(&mut ts, la, sm, semi), "activo");
        let o = achar(&cena, &ids_ui(&mut ts, lo, sm, normal), "o outro");
        assert_eq!(letra(a), paused, "{tab:?}: o activo «{la}»");
        assert_eq!(letra(o), corpo, "{tab:?}: o outro «{lo}»");
        let ra = p.tabs[activo].1;
        let fim = a.5.last().map_or(0.0, |q| q.0);
        assert!(
            fim < ra.x + ra.w && a.5[0].0 >= ra.x,
            "{tab:?}: o texto no alvo"
        );
    }
}
