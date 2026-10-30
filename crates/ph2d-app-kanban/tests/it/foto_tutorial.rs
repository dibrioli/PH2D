//! 📷 As fotos do TUTORIAL do smoke final (`docs/TrelloClone/tutorial/src/01_kanban.html`), para os
//! passos que a foto da janela não alcança por cliques: cada gesto entra pela porta real do hero
//! (como nos testes de costura) e a foto sai em `docs/TrelloClone/smoke/tutorial/<parte>_<passo>_<tema>.png`.
//! `#[ignore]`: a placa. `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --test it foto_tutorial -- --ignored`

use crate::duas_janelas::Duas;
use crate::j2_auto::toca_v;
use crate::verso::{cartao, ctrl, sem};
use crate::w6::{foca, tecla, toca};
use crate::{AREA, Mesa, centro};
use bumpalo::Bump;
use ph2d_app_kanban::KanbanDocs;
use ph2d_board_model::{BoardOp, Element, RichText, Shape, ShapeType};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::screens::hero::board_video::kanban as kb;
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, paint_hero_screen, surface_input,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_kanban_model::KanbanId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const TEMAS: [(Theme, &str); 2] = [(Theme::Light, "claro"), (Theme::Dark, "escuro")];

fn grava(nome: &str, (w, h): (u32, u32), scene: &VectorScene) {
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h))
        .expect("pass");
    let px = pass
        .render_and_readback(&gpu, scene.inner(), (w, h))
        .expect("readback");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/TrelloClone/smoke/tutorial");
    std::fs::create_dir_all(&dir).expect("pasta");
    let path = dir.join(format!("{nome}.png"));
    image::save_buffer(&path, &px, w, h, image::ExtendedColorType::Rgba8).expect("png");
    println!("foto: {}", path.display());
}

/// A cena `n` do smoke, no tema `theme`, com as fontes do sistema desde o 1.º gesto.
fn mesa(n: u8, theme: Theme) -> (Mesa, KanbanId) {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut m = Mesa::new();
    m.hero.theme = theme;
    m.ts = TextSystem::new();
    let id = ph2d_app_kanban::smoke::stage(&mut m.hero, n).expect("cena");
    m.quadros(2);
    (m, id)
}

/// A aba pintada (no rect da aba, como nos lotes J1/J2).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn foto(m: &mut Mesa, nome: &str) {
    m.quadros(20);
    let mut s = VectorScene::new();
    m.pinta(&mut s);
    grava(nome, (AREA.w as u32, AREA.h as u32), &s);
}

/// O ecrã inteiro do hero (o topo, os avisos, o Quadro, os menus).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn foto_hero(hero: &mut HeroScreen, ts: &mut TextSystem, nome: &str) {
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    let mut s = VectorScene::new();
    paint_hero_screen(hero, vp, &mut s, ts);
    let mut s = VectorScene::new();
    paint_hero_screen(hero, vp, &mut s, ts);
    grava(nome, (HERO_VIEWPORT_W as u32, HERO_VIEWPORT_H as u32), &s);
}

/// Rola o verso até o alvo que começa por `nome` estar à vista (sem clicar).
fn rola_v(m: &mut Mesa, id: KanbanId, nome: &str) {
    for _ in 0..60 {
        let r = m
            .docs()
            .probe_back_hits(id.0)
            .into_iter()
            .rev()
            .find(|(n, _)| n.starts_with(nome))
            .unwrap_or_else(|| panic!("o alvo {nome} não foi pintado no verso"))
            .1;
        if r.y >= AREA.y + 40.0 && r.y + r.h <= AREA.y + AREA.h - 40.0 {
            return;
        }
        let dy = if r.y < AREA.y + 40.0 { 16.0 } else { -16.0 };
        m.roda(AREA.w / 2.0, AREA.h / 2.0, dy);
    }
    panic!("o alvo {nome} nunca ficou à vista");
}

#[test]
#[ignore = "a placa: fotos do tutorial"]
fn foto_tutorial_parte_1_listas() {
    for (theme, tag) in TEMAS {
        // 1.3 — o ‹ › abre a REVIEW; o «…» dela abre «List actions».
        let (mut m, id) = mesa(1, theme);
        let review = m.lista(id, "Review");
        let (_, abrir, _) = m.docs().probe_list_buttons(id.0, review).expect("botões");
        m.clica(centro(abrir));
        m.quadros(30);
        let menu = m.docs().probe_list_buttons(id.0, review).expect("botões").2;
        m.clica(centro(menu));
        foto(&mut m, &format!("1_3_{tag}"));
        // 1.4 — «Add another list» → «QA»; «Add a card» nela → «Rig the fox».
        let (mut m, id) = mesa(1, theme);
        for _ in 0..20 {
            m.roda(600.0, AREA.h - 20.0, -16.0);
        }
        let add = m.docs().probe_rect(id.0, "add_list").expect("add_list");
        m.clica(centro(add));
        m.escreve("QA");
        m.enter();
        m.tecla(BoardKey::Escape, sem(), None);
        m.quadros(30);
        let qa = m.lista(id, "QA");
        let (add, _, _) = m.docs().probe_list_buttons(id.0, qa).expect("botões");
        m.clica(centro(add));
        m.escreve("Rig the fox");
        m.enter();
        m.tecla(BoardKey::Escape, sem(), None);
        foto(&mut m, &format!("1_4_{tag}"));
    }
}

#[test]
#[ignore = "a placa: fotos do tutorial"]
fn foto_tutorial_parte_2_verso() {
    for (theme, tag) in TEMAS {
        // 2.3 — a descrição com negrito, uma caixa da checklist, o comentário «@Rui olá».
        let (mut m, id) = mesa(4, theme);
        let c = cartao(&m, id, "Animate shot 12");
        toca_v(&mut m, id, "DescEdit");
        m.escreve(" Then ");
        m.tecla(BoardKey::Char('b'), ctrl(), Some("b"));
        m.escreve("bold");
        m.tecla(BoardKey::Enter, ctrl(), None);
        m.quadro();
        let k = m.kanban(id);
        let item = k
            .items
            .values()
            .filter(|i| !i.done && !i.meta.deleted)
            .find(|i| {
                k.checklists
                    .get(&i.checklist)
                    .is_some_and(|cl| cl.card == c)
            })
            .map(|i| i.id)
            .expect("um item por marcar");
        toca_v(&mut m, id, &format!("ItemBox({item:?})"));
        toca_v(&mut m, id, "CommentArea");
        m.escreve("@Rui");
        m.quadro();
        m.enter();
        m.escreve("olá");
        toca_v(&mut m, id, "CommentSave");
        rola_v(&mut m, id, "DescEdit");
        foto(&mut m, &format!("2_3a_{tag}"));
        rola_v(&mut m, id, "CommentArea");
        for _ in 0..16 {
            m.roda(AREA.w / 2.0, AREA.h / 2.0, -16.0);
        }
        foto(&mut m, &format!("2_3b_{tag}"));
        // 2.5 — uma imagem largada no verso aberto entra nos anexos.
        let (mut m, id) = mesa(4, theme);
        let img = image::RgbaImage::from_fn(320, 180, |x, y| {
            #[allow(clippy::cast_possible_truncation)]
            image::Rgba([(x * 255 / 320) as u8, 120, (y * 255 / 180) as u8, 255])
        });
        let dir = std::env::temp_dir().join(format!("ph2d-foto-tutorial-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("pasta");
        let p = dir.join("fox_pose.png");
        img.save(&p).expect("png");
        assert!(surface_input::drop_files(
            &mut m.hero,
            AREA.x + 2.0,
            AREA.y + AREA.h - 2.0,
            std::slice::from_ref(&p)
        ));
        m.quadros(4);
        rola_v(&mut m, id, "AttOpen(");
        foto(&mut m, &format!("2_5_{tag}"));
    }
}

#[test]
#[ignore = "a placa: fotos do tutorial"]
fn foto_tutorial_parte_4_filtro_e_atalhos() {
    for (theme, tag) in TEMAS {
        // 4.2 — «Clear all», depois «/» e «#Animation due:week».
        let (mut m, id) = mesa(5, theme);
        toca(&mut m, id, "ClearAll");
        m.tecla(BoardKey::Escape, sem(), None);
        m.quadro();
        tecla(&mut m, '/');
        m.escreve("#Animation due:week");
        foto(&mut m, &format!("4_2_{tag}"));
        // 4.3 — o E num cartão (edição rápida) e o «Mirror» → «Select destination».
        let (mut m, id) = mesa(5, theme);
        toca(&mut m, id, "ClearAll");
        m.tecla(BoardKey::Escape, sem(), None);
        m.quadro();
        let c = cartao(&m, id, "Design the fox");
        foca(&mut m, id, c);
        tecla(&mut m, 'e');
        foto(&mut m, &format!("4_3a_{tag}"));
        toca(&mut m, id, "Quick(Mirror)");
        foto(&mut m, &format!("4_3b_{tag}"));
        // 4.5 — «Card aging…» e «Create from template».
        let (mut m, id) = mesa(6, theme);
        let menu = m.docs().probe_rect(id.0, "menu").expect("menu");
        m.clica(centro(menu));
        let linha = m
            .docs()
            .probe_menu_rows(id.0)
            .into_iter()
            .find(|(n, _)| n == "Aging")
            .expect("Card aging…")
            .1;
        m.clica(centro(linha));
        foto(&mut m, &format!("4_5a_{tag}"));
        m.tecla(BoardKey::Escape, sem(), None);
        m.quadro();
        let backlog = m.lista(id, "Backlog");
        let r = m
            .docs()
            .probe_template_button(id.0, backlog)
            .expect("ícone");
        m.clica(centro(r));
        foto(&mut m, &format!("4_5b_{tag}"));
        // 4.6 — «?».
        let (mut m, _) = mesa(6, theme);
        tecla(&mut m, '?');
        foto(&mut m, &format!("4_6_{tag}"));
    }
}

#[test]
#[ignore = "a placa: fotos do tutorial"]
fn foto_tutorial_parte_7_botao_de_quadro() {
    for (theme, tag) in TEMAS {
        let (mut m, id) = mesa(8, theme);
        m.tecla(BoardKey::Escape, sem(), None);
        m.quadro();
        foto(&mut m, &format!("7_4a_{tag}"));
        let r = m
            .docs()
            .probe_w6_hits(id.0)
            .into_iter()
            .find(|(n, _)| n.starts_with("BoardButton("))
            .expect("o botão de quadro no cabeçalho")
            .1;
        m.clica(centro(r));
        foto(&mut m, &format!("7_4b_{tag}"));
    }
}

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ph2d-kanban-import/tests/fixtures/trello_board_synthetic.json"
);

#[test]
#[ignore = "a placa: fotos do tutorial"]
fn foto_tutorial_parte_10_importar_e_exportar() {
    for (theme, tag) in TEMAS {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut m = Mesa::new();
        m.hero.theme = theme;
        m.ts = TextSystem::new();
        m.quadro();
        // 10.2 — File → «Import from Trello…» → o ficheiro (sem pasta de anexos).
        KanbanDocs::answer_next_dialogs(vec![Some(FIXTURE.into()), None]);
        assert!(
            m.hero
                .apply_event(WidgetEvent::Click(KanbanDocs::CMD_IMPORT))
        );
        m.quadros(2);
        ph2d_app_kanban::service(&mut m.hero);
        foto(&mut m, &format!("10_2_{tag}"));
        // 10.3 — File → «Export Kanban…» → CSV: o aviso no topo.
        m.tecla(BoardKey::Escape, sem(), None);
        m.quadro();
        let dir = std::env::temp_dir().join(format!("ph2d-foto-tutorial-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("pasta");
        KanbanDocs::answer_next_dialogs(vec![Some(dir.join("lua.csv"))]);
        assert!(
            m.hero
                .apply_event(WidgetEvent::Click(KanbanDocs::CMD_EXPORT))
        );
        m.quadros(2);
        ph2d_app_kanban::service(&mut m.hero);
        m.quadros(2);
        println!("aviso: {:?}", m.hero.documents.told());
        let mut ts = std::mem::replace(&mut m.ts, TextSystem::without_system_fonts());
        foto_hero(&mut m.hero, &mut ts, &format!("10_3_{tag}"));
    }
}

#[test]
#[ignore = "a placa: fotos do tutorial"]
fn foto_tutorial_parte_11_ligacoes() {
    for (theme, tag) in TEMAS {
        // 11.2 — «Scene object»: o catálogo do projecto, e a ligação escolhida.
        let mut c = ph2d_kanban_links::Catalog::default();
        c.objects.insert(7, "Hero".into());
        c.scripts.insert(3, "jump.luau".into());
        c.macros.insert("Walk cycle".into());
        ph2d_editor_core::kanban_links::publish_scene(c);
        let (mut m, id) = mesa(8, theme);
        toca_v(&mut m, id, "Side(SceneObject)");
        foto(&mut m, &format!("11_2a_{tag}"));
        crate::verso::toca(&mut m, id, "Pop(Scene(0))");
        rola_v(&mut m, id, "LinkOpen(");
        foto(&mut m, &format!("11_2b_{tag}"));
    }
}

/// O Quadro com o Kanban (a cena 1), no tema `theme`, com as fontes do sistema.
fn duas(theme: Theme) -> (Duas, KanbanId, ph2d_board_model::ElementId) {
    let (mut d, id, el) = Duas::new();
    d.hero.theme = theme;
    d.ts = TextSystem::new();
    d.quadros(2);
    (d, id, el)
}

fn foto_d(d: &mut Duas, nome: &str) {
    d.quadros(20);
    foto_hero(&mut d.hero, &mut d.ts, nome);
}

/// Down + Up sobre o alvo `id` do hero (o menu de contexto), pelo despacho.
fn clica_alvo(d: &mut Duas, id: NodeId) {
    let vp = d.hero.last_viewport;
    let mut at = None;
    let mut y = vp.y;
    'fora: while y < vp.y + vp.h {
        let mut x = vp.x;
        while x < vp.x + vp.w {
            if d.hero.hit_index.hit(x, y) == Some(id) {
                at = Some((x, y));
                break 'fora;
            }
            x += 2.0;
        }
        y += 2.0;
    }
    let (x, y) = at.expect("o alvo está pintado");
    let ptr = |kind| PointerEvent {
        x: x + 4.0,
        y: y + 4.0,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let arena = Bump::new();
    let mut events = d
        .hero
        .handle_pointer(ptr(PointerKind::Down), &arena)
        .to_vec();
    events.extend_from_slice(d.hero.handle_pointer(ptr(PointerKind::Up), &arena));
    for e in events {
        d.hero.apply_event(e);
    }
    d.quadros(2);
}

#[test]
#[ignore = "a placa: fotos do tutorial"]
fn foto_tutorial_parte_3_e_11_no_quadro() {
    for (theme, tag) in TEMAS {
        // 3.2 — 1.º clique seleciona, 2.º entra; um cartão a meio do arrasto lá dentro.
        let (mut d, id, el) = duas(theme);
        board_view::select(&mut d.hero, [el]);
        d.quadros(2);
        let probe = d.docs().probe_canvas(el.0).expect("canvas");
        let mut rs: Vec<Rect> = probe.lists.iter().map(|l| l.1).collect();
        rs.sort_by(|a, b| a.x.total_cmp(&b.x));
        let x = rs[0].x + rs[0].w;
        let vao = d.ecra(el, Rect::new(x, rs[0].y + 20.0, rs[1].x - x, 8.0));
        d.clica(vao);
        assert_eq!(kb::active(&d.hero), Some(el), "o 2.º clique entra");
        let backlog = d.lista(id, "Backlog");
        let card = d
            .hero
            .documents
            .kanbans()
            .get(id)
            .unwrap()
            .cards_in(backlog)[0]
            .id;
        let probe = d.docs().probe_canvas(el.0).expect("canvas");
        let r = probe.cards.iter().find(|c| c.0 == card).unwrap().1;
        let de = d.ecra(el, Rect::new(r.x, r.y + 4.0, r.w, 8.0));
        let doing = d.lista(id, "To Do");
        let lr = probe.lists.iter().find(|l| l.0 == doing).unwrap().1;
        let para = d.ecra(el, Rect::new(lr.x, lr.y + 120.0, lr.w * 0.6, 8.0));
        d.rato(PointerKind::Down, de);
        for i in 1..=12u8 {
            let t = f32::from(i) / 12.0;
            d.move_((de.0 + (para.0 - de.0) * t, de.1 + (para.1 - de.1) * t));
            d.quadro();
        }
        foto_d(&mut d, &format!("3_2_{tag}"));

        // 11.3 — uma nota ao lado do Kanban: botão direito → «Convert to card» → «Convert to card in…».
        let (mut d, _, el) = duas(theme);
        let nota = {
            let b = d.hero.documents.active_board_mut().expect("quadro");
            let k = b.doc.get(el).expect("kanban").clone();
            let nid = b.doc.mint_id();
            let fill = ph2d_board_model::Rgba(ph2d_board_model::STICKY_COLORS[2]);
            let shape = Shape {
                kind: ShapeType::Sticky,
                style: ph2d_board_model::Style::new(Some(fill), None, fill.readable_ink()),
                text: RichText::plain("Fix the jump\nThe hero falls"),
            };
            let bx = [k.x + k.w + 80.0, k.y + 40.0, 199.0, 199.0];
            let e = Element::new_shape(nid, b.doc.z_on_top(), shape, bx);
            let _ = BoardOp::Put(e).apply(&mut b.doc);
            b.camera.center_x = bx[0] + 99.5;
            b.camera.center_y = bx[1] + 99.5;
            nid
        };
        d.quadros(3);
        let cv = d.hero.last_canvas;
        let (cx, cy) = (cv.x + cv.w / 2.0, cv.y + cv.h / 2.0);
        let input = Input {
            text: &mut d.ts,
            mods: Modifiers::default(),
            now_ns: d.agora,
        };
        board_view::pointer(
            &mut d.hero,
            input,
            PointerKind::Down,
            PointerButton::Secondary,
            cx,
            cy,
            true,
        );
        d.quadros(2);
        foto_d(&mut d, &format!("11_3a_{tag}"));
        clica_alvo(&mut d, ph2d_editor_core::ids::CTX_MENU_BOARD_EL_TO_CARD);
        foto_d(&mut d, &format!("11_3b_{tag}"));
        let _ = nota;

        // 11.4 — o verso de um cartão deste Kanban (que está num Quadro): «Convert to sticky note».
        let (mut d, id, _) = duas(theme);
        let c = d
            .hero
            .documents
            .kanbans()
            .get(id)
            .unwrap()
            .cards
            .values()
            .find(|c| c.title == "Write the logline")
            .unwrap()
            .id;
        d.docs().open(id);
        d.quadros(2);
        d.docs().open_back(id.0, c);
        d.quadros(2);
        foto_d(&mut d, &format!("11_4a_{tag}"));
        let r = d
            .docs()
            .probe_back_hits(id.0)
            .into_iter()
            .rev()
            .find(|(n, _)| n == "Side(ToNote)")
            .expect("«Convert to sticky note»")
            .1;
        d.clica(centro(r));
        d.quadros(4);
        let vivo = |d: &Duas| {
            d.hero
                .documents
                .kanbans()
                .get(id)
                .unwrap()
                .cards
                .get(&c)
                .is_some_and(|x| !x.meta.deleted)
        };
        assert!(
            d.hero.documents.active_board().is_some() && !vivo(&d),
            "11.4: a nota no Quadro"
        );
        foto_d(&mut d, &format!("11_4b_{tag}"));
        let _ = sem();
    }
}
