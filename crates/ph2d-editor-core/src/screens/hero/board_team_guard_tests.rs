//! As guardas da equipa (auditoria W8, `docs/MiroClone/06_equipa_e_saida.md`): cada uma foi um defeito
//! que as duas lentes acharam no fecho — o carregar que não é o primário, a ferramenta que fica armada,
//! o índice que aponta para outra mensagem depois de um apagar, o apagar de um clique, o cartão e os
//! painéis fora da área, a apresentação que deixava o chrome por baixo vivo. Pelo caminho do ECRÃ.

use super::*;
use crate::documents::team::{ExportChoice, Scope};
use crate::interaction::{KEY_ESCAPE, WidgetEvent};
use ph2d_board_edit::Tool;
use ph2d_board_model::ThreadId;
use ph2d_vector::{Brush, DrawOp};

impl T {
    /// A caixa `[x0, y0, x1, y1]` onde o índice responde `id` (varrida a 2 px), se responde.
    fn extent(&self, id: NodeId) -> Option<[f32; 4]> {
        let vp = self.hero.last_viewport;
        let mut out: Option<[f32; 4]> = None;
        let mut y = vp.y;
        while y < vp.y + vp.h {
            let mut x = vp.x;
            while x < vp.x + vp.w {
                if self.hero.hit_index.hit(x, y) == Some(id) {
                    let e = out.get_or_insert([x, y, x, y]);
                    *e = [e[0].min(x), e[1].min(y), e[2].max(x), e[3].max(y)];
                }
                x += 2.0;
            }
            y += 2.0;
        }
        out
    }

    fn down(&mut self, at: (f32, f32), button: PointerButton) {
        let (h, ts, c) = (&mut self.hero, &mut self.ts, &mut self.clock);
        board_view::pointer(h, inp(ts, c), PointerKind::Down, button, at.0, at.1, true);
        board_view::pointer(h, inp(ts, c), PointerKind::Up, button, at.0, at.1, true);
        self.paint();
    }

    fn arm(&mut self) {
        self.shell_click_bar(Item::Fac(FacItem::Comment));
        assert!(self.hero.documents.live.team.commenting);
    }

    fn thread_of(&mut self, at: [f64; 2], msgs: &[(&str, &str)]) -> ThreadId {
        let b = self.hero.documents.active_board_mut().unwrap();
        let (a, text) = msgs[0];
        let id = b.team.open(Pin::Point(at), a, text, 0).unwrap();
        for (i, (a, text)) in msgs.iter().enumerate().skip(1) {
            b.team.reply(id, a, text, i as i64);
        }
        self.paint();
        id
    }
}

/// K1 — com a ferramenta armada só o carregar PRIMÁRIO abre um fio: o direito e o `Espaço`+arrastar
/// continuam a ser da vista.
#[test]
fn only_the_primary_press_opens_a_thread() {
    let mut t = T::new();
    t.arm();
    let p = t.screen([0.0, 0.0]);
    t.down(p, PointerButton::Secondary);
    assert!(
        t.hero.documents.live.team.open.is_empty(),
        "o direito não abre"
    );
    board_keys::key(
        &mut t.hero,
        &mut t.ts,
        BoardKey::Space,
        Modifiers::default(),
        true,
        None,
        None,
    );
    t.down(p, PointerButton::Primary);
    assert!(
        t.hero.documents.live.team.open.is_empty(),
        "com o Espaço é deslocar"
    );
    board_keys::key(
        &mut t.hero,
        &mut t.ts,
        BoardKey::Space,
        Modifiers::default(),
        false,
        None,
        None,
    );
    t.down(p, PointerButton::Primary);
    assert_eq!(t.hero.documents.live.team.open.len(), 1);
}

/// K1 — escolher outra ferramenta (a tecla ou a barra) larga a de comentar; `Esc` no compositor também.
#[test]
fn another_tool_or_esc_in_the_composer_disarms_the_comment_tool() {
    let mut t = T::new();
    t.arm();
    t.key(BoardKey::Char('v'), Modifiers::default(), Some("v"));
    assert!(!t.hero.documents.live.team.commenting, "a tecla V");
    t.arm();
    t.shell_click_bar(Item::Tool(Tool::Select));
    assert!(!t.hero.documents.live.team.commenting, "a barra");
    t.arm();
    let p = t.screen([0.0, 0.0]);
    t.click_board(p);
    assert_eq!(t.hero.documents.live.team.open.len(), 1);
    t.hero_key(KEY_ESCAPE);
    let team = &t.hero.documents.live.team;
    assert!(
        team.open.is_empty() && !team.commenting,
        "Esc larga o cartão E a ferramenta"
    );
}

/// K4 — apagar pede o 2.º clique; e a edição em curso segue a SUA mensagem quando outra sai.
#[test]
fn delete_asks_twice_and_the_edit_follows_its_message() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    let id = t.thread_of(
        [0.0, 0.0],
        &[("Ana", "one"), ("Ana", "two"), ("Ana", "three")],
    );
    t.shell_click_id(team_node(Hit::Bubble(id)), "a bolha");
    t.shell_click_id(team_node(Hit::Edit(2)), "editar a 3.ª");
    t.shell_click_id(team_node(Hit::Delete(1)), "apagar a 2.ª");
    assert_eq!(
        t.team().thread(id).unwrap().messages.len(),
        3,
        "o 1.º clique só arma"
    );
    t.shell_click_id(team_node(Hit::Delete(1)), "confirmar");
    assert_eq!(t.team().thread(id).unwrap().messages.len(), 2);
    assert_eq!(
        t.hero.documents.live.team.editing,
        Some((id, 1)),
        "a 3.ª é agora a 2.ª"
    );
    // O clique no lixo levou o foco: o artista volta ao campo e publica.
    let field = crate::ids::BOARD_COMMENT_INPUT;
    board_nav::open_field(&mut t.hero.store, field, "three!".into());
    t.hero.apply_event(WidgetEvent::Submit(field));
    let m = &t.team().thread(id).unwrap().messages;
    assert_eq!((m[0].text.as_str(), m[1].text.as_str()), ("one", "three!"));
    assert_eq!(m.len(), 2, "editou, não respondeu");
}

/// R2 — apagar uma versão pede o 2.º clique; o renomear segue a SUA versão.
#[test]
fn a_version_delete_asks_twice_and_the_rename_follows_its_version() {
    let mut t = T::new();
    {
        let b = t.hero.documents.active_board_mut().unwrap();
        for n in ["a", "b", "c"] {
            b.save_version(n.into(), 0);
        }
    }
    t.tab_menu(crate::ids::CTX_MENU_BOARD_VERSIONS);
    t.hero
        .apply_event(WidgetEvent::DoubleClick(team_node(Hit::RenameVersion(2))));
    t.paint();
    t.shell_click_id(team_node(Hit::DropVersion(0)), "apagar a 1.ª");
    assert_eq!(t.team().versions.len(), 3, "o 1.º clique só arma");
    t.shell_click_id(team_node(Hit::DropVersion(0)), "confirmar");
    assert_eq!(t.team().versions.len(), 2);
    // O campo tem o nome da versão a renomear («c»): esvazia-se e escreve-se o novo.
    board_nav::open_field(
        &mut t.hero.store,
        crate::ids::BOARD_VERSION_INPUT,
        String::new(),
    );
    t.write_and_enter("C2");
    let names: Vec<_> = t.team().versions.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["b", "C2"]);
}

/// K4 — um fio comprido num alvo perto do canto: o cartão fica DENTRO da área e mostra as últimas
/// mensagens (a primeira, minha, fica acima de «N earlier» — sem botões pintados).
#[test]
fn a_long_thread_card_stays_inside_the_area() {
    let mut t = T::new();
    t.hero.documents.set_author("Ana".into());
    let canvas = t.hero.last_canvas;
    let a = board_view::area_of(canvas);
    let world = {
        let b = t.hero.documents.active_board().unwrap();
        b.camera.to_world(
            a,
            [
                f64::from(canvas.x + canvas.w) - 40.0,
                f64::from(canvas.y + canvas.h / 2.0),
            ],
        )
    };
    let long = "a long message that wraps over a few lines of the card to make it tall";
    let msgs: Vec<(&str, &str)> = (0..30).map(|_| ("Ana", long)).collect();
    let id = t.thread_of(world, &msgs);
    t.shell_click_id(team_node(Hit::Bubble(id)), "a bolha");
    let card = t
        .extent(team_node(Hit::Button(TeamButton::CardSurface)))
        .expect("o cartão");
    assert!(
        card[0] >= canvas.x
            && card[1] >= canvas.y
            && card[2] <= canvas.x + canvas.w
            && card[3] <= canvas.y + canvas.h,
        "{card:?} fora de {canvas:?}"
    );
    assert!(
        t.find(team_node(Hit::Edit(29))).is_some(),
        "a última à vista"
    );
    assert!(
        t.find(team_node(Hit::Edit(0))).is_none(),
        "a primeira fica nas de antes"
    );
}

/// R2/K8 — os painéis param por cima das filas de botões do canto; o que não cabe pagina.
#[test]
fn a_long_panel_stops_above_the_corner_buttons_and_pages() {
    let mut t = T::new();
    {
        let b = t.hero.documents.active_board_mut().unwrap();
        for n in 0..60 {
            b.save_version(format!("v{n}"), 0);
        }
    }
    t.tab_menu(crate::ids::CTX_MENU_BOARD_VERSIONS);
    let panel = t
        .extent(team_node(Hit::Button(TeamButton::VersionsSurface)))
        .unwrap();
    let corner = t.extent(button_node(NavButton::Present)).unwrap();
    assert!(panel[3] < corner[1], "{panel:?} tapa o canto {corner:?}");
    assert!(
        t.find(team_node(Hit::Restore(59))).is_some(),
        "a mais nova à vista"
    );
    assert!(
        t.find(team_node(Hit::Button(TeamButton::PageUp))).is_none(),
        "nada antes"
    );
    t.btn(TeamButton::PageDown);
    assert!(
        t.find(team_node(Hit::Restore(59))).is_none(),
        "a página andou"
    );
    t.btn(TeamButton::PageUp);
    assert!(t.find(team_node(Hit::Restore(59))).is_some());
}

/// Um painel do canto de cada vez: o exportar fecha a lista.
#[test]
fn the_corner_panels_are_one_at_a_time() {
    let mut t = T::new();
    t.shell_click_id(button_node(NavButton::Comments), "Comments");
    assert!(t.hero.documents.live.team.list_open);
    t.tab_menu(crate::ids::CTX_MENU_BOARD_EXPORT);
    let team = &t.hero.documents.live.team;
    assert!(team.export.is_some() && !team.list_open);
}

/// E2 — «Selection» sem nada seleccionado exporta o quadro (a área que o diálogo mostra acesa).
#[test]
fn a_selection_export_without_a_selection_is_the_board() {
    let mut t = T::new();
    t.note_at([0.0, 0.0], "x");
    t.tab_menu(crate::ids::CTX_MENU_BOARD_EXPORT);
    t.hero.documents.live.team.export = Some(ExportChoice {
        scope: Scope::Selection,
        ..ExportChoice::default()
    });
    t.paint();
    t.btn(TeamButton::ExportGo);
    match t.hero.documents.take_io() {
        Some(IoRequest::Export { choice, .. }) => assert_eq!(choice.scope, Scope::Board),
        other => panic!("{other:?}"),
    }
}

/// P1/P3/P6 — a apresentação cobre a janela: nenhum painel por baixo recebe o ponteiro, o F9 é
/// recusado, um frame sem fundo é papel, e sair devolve a ferramenta de comentar armada.
#[test]
fn presenting_owns_the_window_and_gives_the_tool_back() {
    let mut t = T::new();
    let f = t.frame_at([0.0, 0.0, 400.0, 300.0], "One");
    {
        let b = t.hero.documents.active_board_mut().unwrap();
        let mut el = b.doc.get(f).unwrap().clone();
        if let ph2d_board_model::ElementKind::Frame(fr) = &mut el.kind {
            fr.style = Style::new(None, None, Rgba([0, 0, 0, 255]));
        }
        BoardOp::Put(el).apply(&mut b.doc);
    }
    t.arm();
    t.shell_click_id(button_node(NavButton::Present), "Present");
    assert!(board_nav::team::show::showing(&t.hero));
    // Um painel publicado sob o ponto (o que um painel do chrome faz a cada quadro): a apresentação
    // tapa-o (prova de mutação W8: sem painel nenhum ali, a pergunta era vazia).
    let panel = NodeId(0x7e57_0001);
    let under = Rect::new(0.0, 0.0, 200.0, 200.0);
    t.hero.store.set_panel_rect(panel, under);
    assert!(t.hero.chrome_panel_at(10.0, 10.0).is_none());
    assert_eq!(t.hero.toggle_legacy_chrome(), None, "F9 recusado");
    let mut scene = VectorScene::new();
    scene.record();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    let ((), seen) = crate::text_elide::elisao::medindo(|| {
        paint_hero_screen(&mut t.hero, vp, &mut scene, &mut t.ts);
    });
    // A foto dos estados viu «1/3 ·…»: o contador sai inteiro (o `onde` do censo é o do pintor
    // centrado, por isso o contador acha-se pelo texto).
    let counter: Vec<_> = seen.iter().filter(|m| m.texto.contains("Esc")).collect();
    assert!(!counter.is_empty(), "o contador foi medido");
    for m in counter {
        assert!(m.coube(), "{:?} pintado {:?}", m.texto, m.pintado);
    }
    let paper = scene.take_record().iter().any(|op| {
        matches!(op, DrawOp::Fill { brush: Brush::Solid(c), .. } if *c == ph2d_board_render::EXPORT_PAPER)
    });
    assert!(paper, "o diapositivo sem fundo é papel");
    t.key(BoardKey::Escape, Modifiers::default(), None);
    assert!(
        t.hero.documents.live.team.commenting,
        "a ferramenta volta armada"
    );
    t.hero.store.set_panel_rect(panel, under);
    assert_eq!(
        t.hero.chrome_panel_at(10.0, 10.0),
        Some(panel),
        "o controlo: fora da apresentação o painel responde"
    );
}

/// P1 — todos os frames escondidos: o aviso diz isso (não «adicione um frame»).
#[test]
fn present_with_every_frame_hidden_says_so() {
    let mut t = T::new();
    let f = t.frame_at([0.0, 0.0, 400.0, 300.0], "One");
    t.hero
        .documents
        .active_board_mut()
        .unwrap()
        .team
        .hidden_slides
        .insert(f);
    t.shell_click_id(button_node(NavButton::Present), "Present");
    assert!(!board_nav::team::show::showing(&t.hero));
    assert_eq!(
        t.hero.documents.live.notice.map(|n| n.0),
        Some("board.present.all_hidden")
    );
}
