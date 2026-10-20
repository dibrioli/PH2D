//! Onda 2, item 5 — a vista da página de ajuda (`docs/CodeEditor/04_pagina_de_ajuda.md` §3.3, §5):
//! só a faixa à vista se pinta, a âncora do membro vai ao topo, cada alvo responde ao clique, a
//! coluna do índice sai na área estreita, o leitor de ecrã e as cores pelos tokens.

use crate::ts;
use ph2d_a11y::{AkNodeId, NodeId, Role};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::widget::section_cards::CardDepth;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::help::{Block, HelpDoc, Span, SpanKind};
use ph2d_panel_code::{HelpFrame, HelpHit, HelpView, Ink};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Brush, DrawOp, VectorScene};
use std::collections::BTreeSet;

const MEMBROS: u32 = 40;
/// O número do link do corpo do membro `i`.
const LINK: u32 = 1_000;
const ULTIMO: u32 = 99;
const BASE: NodeId = NodeId(7_000);

fn prosa(t: &str) -> Span {
    Span::new(t, SpanKind::Prose)
}

fn codigo(t: &str, ink: Ink) -> Span {
    Span::new(t, SpanKind::Code(ink))
}

/// Uma página com ~200 blocos, todas as linhas distintas.
fn pagina() -> HelpDoc {
    let mut blocks = vec![
        Block::Title("math".into()),
        Block::Strong(vec![prosa("Mathematical functions.")]),
        Block::Heading("Functions".into()),
    ];
    for i in 0..MEMBROS {
        blocks.push(Block::Sig {
            spans: vec![
                codigo(&format!("math.f{i}"), Ink::Function),
                codigo("(x: number): number", Ink::Plain),
            ],
            member: i,
            first: i == 0,
        });
        blocks.push(Block::Body(vec![
            prosa(&format!("Gives f{i} back. Read about ")),
            codigo("math.floor", Ink::Function).link(LINK + i),
            prosa(" here."),
        ]));
        blocks.push(Block::Code {
            lines: vec![
                vec![codigo(&format!("local a{i} = math.f{i}(1)"), Ink::Plain)],
                vec![codigo(&format!("print(a{i}) --> {i}"), Ink::Plain)],
            ],
            copy: i,
        });
        blocks.push(Block::Gap);
    }
    blocks.push(Block::Heading("Constants".into()));
    for i in 0..30 {
        blocks.push(Block::Para(vec![prosa(&format!(
            "Constant number {i} of the page."
        ))]));
    }
    // O último membro, curto, no fim da página: a âncora dele nunca chega ao topo.
    blocks.push(Block::Sig {
        spans: vec![codigo("math.last()", Ink::Function)],
        member: ULTIMO,
        first: false,
    });
    HelpDoc {
        key: 1,
        title: "math".into(),
        blocks,
        sections: vec!["Functions".into(), "Constants".into()],
    }
}

fn quadro(doc: &HelpDoc) -> HelpFrame<'_> {
    HelpFrame {
        doc,
        back: true,
        forward: false,
        hover: None,
        focus_link: None,
        selection: None,
        find: None,
        matches: &[],
        current: None,
    }
}

fn pinta(v: &mut HelpView, f: &HelpFrame<'_>, ts: &mut TextSystem, area: Rect) -> VectorScene {
    let mut scene = VectorScene::new();
    scene.record();
    v.paint(f, area, &mut scene, ts, &Theme::default());
    scene
}

fn meio(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

const LARGA: Rect = Rect::new(0.0, 0.0, 1000.0, 400.0);

#[test]
fn so_a_faixa_a_vista_se_pinta_e_a_pagina_mostra_as_linhas_seguintes() {
    let doc = pagina();
    let (mut v, mut ts) = (HelpView::new(), ts());
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    let p1 = v.probe();
    let lh = v.metrics().line_h;
    assert!(lh > 0.0);
    let cabem = (LARGA.h / lh).ceil() as usize + 1;
    assert!(
        p1.height > 5.0 * LARGA.h,
        "controlo: a página é muito maior que a vista ({})",
        p1.height
    );
    assert!(
        !p1.lines.is_empty() && p1.lines.len() <= cabem,
        "{} linhas pintadas, cabem {cabem}",
        p1.lines.len()
    );
    assert!(
        p1.runs <= p1.lines.len() * 4,
        "{} trechos em {} linhas",
        p1.runs,
        p1.lines.len()
    );
    assert_eq!(p1.lines[0], "math", "o título no topo");
    assert_eq!(p1.title, "math");
    v.page(true);
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    let p2 = v.probe();
    assert!(p2.scroll > 0.0);
    assert!(!p2.lines.is_empty() && p2.lines.len() <= cabem);
    let ultima = p1.lines.last().unwrap();
    for l in &p1.lines {
        assert!(
            l == ultima || !p2.lines.contains(l),
            "a linha «{l}» ficou à vista depois de uma página"
        );
    }
    let depois = p2
        .lines
        .iter()
        .position(|l| !p1.lines.contains(l))
        .expect("linhas novas");
    assert!(
        depois <= 1,
        "a página continua onde a anterior acabou: {:?} → {:?}",
        p1.lines,
        p2.lines
    );
}

#[test]
fn ir_ao_membro_poe_a_assinatura_no_topo_e_prende_no_fim() {
    let doc = pagina();
    let (mut v, mut ts) = (HelpView::new(), ts());
    // Pedido antes da primeira pintura: a página ainda não está disposta.
    v.scroll_to_member(20);
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    let p = v.probe();
    assert_eq!(p.top_member, Some(20));
    assert_eq!(p.lines[0], "math.f20(x: number): number");
    // Depois de disposta, a resposta é imediata.
    v.scroll_to_member(7);
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    assert_eq!(
        (v.probe().top_member, v.probe().lines[0].as_str()),
        (Some(7), "math.f7(x: number): number")
    );
    assert_eq!(v.probe().section, Some(0), "o índice marca «Functions»");
    // O último membro não chega ao topo: a rolagem prende no fim.
    v.scroll_to_member(ULTIMO);
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    let fim = v.scroll();
    v.scroll_by(LARGA.h);
    assert_eq!(v.scroll(), fim, "já estava no fim");
    v.end();
    assert_eq!(v.scroll(), fim);
    assert!(fim < v.probe().height);
    assert_eq!(v.probe().section, Some(1), "no fim, a secção «Constants»");
    v.home();
    assert_eq!(v.scroll(), 0.0);
    v.scroll_to_section(1);
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    assert_eq!(v.probe().lines[0], "Constants");
}

#[test]
fn o_clique_acerta_cada_alvo() {
    let doc = pagina();
    let (mut v, mut ts) = (HelpView::new(), ts());
    v.scroll_to_member(3);
    let mut f = quadro(&doc);
    pinta(&mut v, &f, &mut ts, LARGA);
    let p = v.probe();
    let (n, r, texto) = p
        .links
        .iter()
        .find(|l| l.0 == LINK + 3)
        .expect("o link do membro 3 à vista")
        .clone();
    assert_eq!((n, texto.as_str()), (LINK + 3, "math.floor"));
    assert_eq!(v.link_rect(LINK + 3), Some(r));
    let (x, y) = meio(r);
    assert_eq!(v.hit(&mut ts, x, y), Some(HelpHit::Link(LINK + 3)));
    // A prosa antes do link: a palavra «about».
    let Some(HelpHit::Text(b)) = v.hit(&mut ts, r.x - 16.0, y) else {
        panic!("texto")
    };
    let plain = v.plain();
    let ini = plain[..b]
        .rfind(|c: char| !c.is_alphanumeric())
        .map_or(0, |i| i + 1);
    let fim = plain[b..]
        .find(|c: char| !c.is_alphanumeric())
        .map_or(plain.len(), |i| b + i);
    assert_eq!(&plain[ini..fim], "about");
    // O botão de copiar o exemplo 3 e o texto dele.
    let c = v
        .target_rect(HelpHit::Copy(3))
        .expect("o botão de copiar à vista");
    let (x, y) = meio(c);
    assert_eq!(v.hit(&mut ts, x, y), Some(HelpHit::Copy(3)));
    assert_eq!(
        v.example_text(3).as_deref(),
        Some("local a3 = math.f3(1)\nprint(a3) --> 3")
    );
    // O cabeçalho e o índice.
    let (x, y) = meio(v.target_rect(HelpHit::Back).unwrap());
    assert_eq!(v.hit(&mut ts, x, y), Some(HelpHit::Back));
    let (fx, fy) = meio(v.target_rect(HelpHit::Forward).unwrap());
    assert_eq!(
        v.hit(&mut ts, fx, fy),
        None,
        "› sem para onde ir não responde"
    );
    f.forward = true;
    pinta(&mut v, &f, &mut ts, LARGA);
    assert_eq!(v.hit(&mut ts, fx, fy), Some(HelpHit::Forward));
    let (x, y) = meio(v.target_rect(HelpHit::Section(1)).unwrap());
    assert_eq!(v.hit(&mut ts, x, y), Some(HelpHit::Section(1)));
    // A ordem do Tab: um por link, pela página.
    let ordem = v.links_in_order();
    assert_eq!(ordem, (0..MEMBROS).map(|i| LINK + i).collect::<Vec<_>>());
}

#[test]
fn estreita_sem_indice_e_nada_sai_da_area() {
    let doc = pagina();
    let (mut v, mut ts) = (HelpView::new(), ts());
    let larga = Rect::new(100.0, 50.0, 1000.0, 300.0);
    pinta(&mut v, &quadro(&doc), &mut ts, larga);
    assert!(
        v.target_rect(HelpHit::Section(0)).is_some(),
        "controlo: na área larga há índice"
    );
    let estreita = Rect::new(100.0, 50.0, 420.0, 300.0);
    let mut scene = pinta(&mut v, &quadro(&doc), &mut ts, estreita);
    assert_eq!(v.target_rect(HelpHit::Section(0)), None);
    let p = v.probe();
    assert!(!p.lines.is_empty());
    for (_, r, _) in &p.links {
        assert!(
            r.x >= estreita.x && r.x + r.w <= estreita.x + estreita.w,
            "{r:?}"
        );
    }
    // Tudo o que se pinta fica dentro do recorte da área: o 1.º pedido abre-o, o último fecha-o.
    let ops = scene.take_record();
    let Some(DrawOp::Layer { clip, .. }) = ops.first() else {
        panic!("{:?}", ops.first())
    };
    let bb = ph2d_vector::Shape::bounding_box(clip);
    assert_eq!((bb.x0, bb.y0, bb.x1, bb.y1), (100.0, 50.0, 520.0, 350.0));
    let mut fundo = 0i32;
    for (i, op) in ops.iter().enumerate() {
        match op {
            DrawOp::Layer { .. } => fundo += 1,
            DrawOp::Pop => fundo -= 1,
            _ => {}
        }
        assert!(
            fundo > 0 || i + 1 == ops.len(),
            "o pedido {i} fica fora do recorte da área"
        );
    }
    assert_eq!(fundo, 0);
}

#[test]
fn o_leitor_de_ecra_le_a_pagina_e_o_link_do_teclado() {
    let doc = pagina();
    let (mut v, mut ts) = (HelpView::new(), ts());
    v.scroll_to_member(3);
    let mut f = quadro(&doc);
    f.focus_link = Some(LINK + 3);
    pinta(&mut v, &f, &mut ts, LARGA);
    let nos = v.a11y(BASE);
    let (id, raiz) = &nos[0];
    assert_eq!(
        (*id, raiz.role(), raiz.label()),
        (BASE, Role::Document, Some("math"))
    );
    let ids: BTreeSet<NodeId> = nos.iter().map(|n| n.0).collect();
    assert_eq!(ids.len(), nos.len(), "ids únicos");
    let link = nos
        .iter()
        .find(|(_, n)| n.role() == Role::Link && n.label() == Some("math.floor"))
        .expect("um link");
    let focado = v.a11y_link_id(BASE, LINK + 3);
    assert!(ids.contains(&focado));
    assert_eq!(raiz.active_descendant(), Some(AkNodeId::from(focado)));
    assert!(link.1.bounds().is_some());
    assert!(
        nos.iter()
            .any(|(_, n)| n.role() == Role::Heading
                && n.label() == Some("math.f3(x: number): number"))
    );
    assert!(
        nos.iter()
            .any(|(_, n)| n.role() == Role::Button && n.label() == Some("Copy example"))
    );
    assert!(
        nos.iter()
            .any(|(_, n)| n.role() == Role::List && n.children().len() == 2)
    );
    // Cada filho nomeado existe.
    for (_, n) in &nos {
        for k in n.children() {
            assert!(
                nos.iter().any(|(i, _)| AkNodeId::from(*i) == *k),
                "filho {k:?} sem nó"
            );
        }
    }
    // Controlo: sem foco, nenhum descendente activo.
    f.focus_link = None;
    pinta(&mut v, &f, &mut ts, LARGA);
    assert_eq!(v.a11y(BASE)[0].1.active_descendant(), None);
}

/// ⚠️ A gravação da cena vê os retângulos (`fill_rect`); os glifos e as quinas redondas vão pela
/// cena directa, e esses ficam com o censo das fontes (`no_literal_color`, `ph2d-editor-core`).
#[test]
fn cada_cor_pintada_vem_de_um_token() {
    let doc = pagina();
    let theme = Theme::default();
    let mut aceites: Vec<ph2d_vector::Color> =
        ColorToken::ALL.iter().map(|t| resolve(*t, theme)).collect();
    aceites.extend(
        ColorToken::ALL
            .iter()
            .map(|t| ph2d_editor_core::widget::resolve_opaque(*t, theme)),
    );
    aceites.extend([
        CardDepth::Section.fill(theme),
        CardDepth::Subsection.fill(theme),
    ]);
    let (mut v, mut ts) = (HelpView::new(), ts());
    v.scroll_to_member(3);
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    // A seleção, as ocorrências e o link sob o rato, todos À VISTA.
    let (l3, l4) = (
        v.link_byte(LINK + 3).unwrap(),
        v.link_byte(LINK + 4).unwrap(),
    );
    let m = [l3..l3 + 4, l4..l4 + 4];
    let mut f = quadro(&doc);
    f.hover = Some(HelpHit::Link(LINK + 3));
    f.focus_link = Some(LINK + 4);
    f.selection = Some(l3 - 10..l3 - 2);
    f.matches = &m;
    f.current = Some(1);
    let mut scene = pinta(&mut v, &f, &mut ts, LARGA);
    let mut vistas = Vec::new();
    for op in &scene.take_record() {
        let c = match op {
            DrawOp::Fill {
                brush: Brush::Solid(c),
                ..
            }
            | DrawOp::Stroke {
                brush: Brush::Solid(c),
                ..
            } => *c,
            DrawOp::Glyphs(g) => g.brush,
            _ => continue,
        };
        assert!(aceites.contains(&c), "uma cor sem token: {c:?} em {op:?}");
        vistas.push(c);
    }
    // Controlo: a gravação viu as marcas e o sublinhado.
    for t in [
        ColorToken::Selection,
        ColorToken::InfoSoft,
        ColorToken::WarnSoft,
        ColorToken::Accent,
    ] {
        assert!(
            vistas.contains(&resolve(t, theme)),
            "{t:?} não se viu: {vistas:?}"
        );
    }
}
