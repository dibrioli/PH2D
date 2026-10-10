//! ⭐ A cena 10 ENSINA o que acontece (a lição da W5b), e cada lição conduz-se pelo caminho do ECRÃ que
//! ela manda (a lição da W7: a cena 9 tinha uma lição falsa que o gate não via, porque chamava o editor
//! e não o painel) — a tecla `C` e o clique no quadro pela porta da shell, o texto e o `Enter` pelo
//! despacho do hero, o menu da aba aberto e a linha carregada, o botão Present do canto — na ORDEM REAL
//! (o quadro de uma lição é o da seguinte).

use super::*;
use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::documents::team::{Format, IoRequest};
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, KEY_ENTER, KEY_ESCAPE};
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::board_nav::team::{Hit, TeamButton, node};
use ph2d_editor_core::screens::hero::board_nav::{NavButton, button_node};
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// A área do quadro no ecrã, `[x, y, w, h]` (a conta do `board_view`).
fn area(r: Rect) -> [f64; 4] {
    [
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.w),
        f64::from(r.h),
    ]
}

struct Screen {
    hero: HeroScreen,
    ts: TextSystem,
    clock: u128,
    l: Lessons,
}

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

impl Screen {
    fn new() -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut hero = HeroScreen::new(NodeId(1));
        let mut set = BoardSet::default();
        let id = set.create(default_name(1));
        let l = build(set.get_mut(id).unwrap());
        document_tabs::load(&mut hero, set);
        hero.documents.activate(Some(id));
        board_nav::show(&mut hero, false, false);
        let mut s = Self {
            hero,
            ts: TextSystem::without_system_fonts(),
            clock: 1_000_000_000_000,
            l,
        };
        s.paint();
        s.paint();
        s
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.ts);
    }

    fn find(&self, id: NodeId) -> Option<(f32, f32)> {
        let vp = self.hero.last_viewport;
        let mut hits = Vec::new();
        let mut y = vp.y;
        while y < vp.y + vp.h {
            let mut x = vp.x;
            while x < vp.x + vp.w {
                if self.hero.hit_index.hit(x, y) == Some(id) {
                    hits.push((x, y));
                }
                x += 2.0;
            }
            y += 2.0;
        }
        let n = hits.len() as f32;
        (!hits.is_empty()).then(|| {
            (
                hits.iter().map(|p| p.0).sum::<f32>() / n,
                hits.iter().map(|p| p.1).sum::<f32>() / n,
            )
        })
    }

    /// Um clique num controlo PELA ORDEM DA SHELL: o quadro primeiro, o resto ao despacho.
    fn click(&mut self, id: NodeId, what: &str) {
        let (x, y) = self
            .find(id)
            .unwrap_or_else(|| panic!("{what} não está pintado"));
        let on_canvas =
            self.hero.chrome_panel_at(x, y).is_none() && self.hero.chrome_hit(x, y).is_none();
        assert!(!on_canvas, "{what} é um controlo, não o quadro");
        let arena = Bump::new();
        for kind in [PointerKind::Down, PointerKind::Up] {
            self.clock += 1_000_000_000;
            let inp = Input {
                text: &mut self.ts,
                mods: Modifiers::default(),
                now_ns: self.clock,
            };
            if !board_view::pointer(
                &mut self.hero,
                inp,
                kind,
                PointerButton::Primary,
                x,
                y,
                false,
            ) {
                let ev = self.hero.handle_pointer(ptr(kind, x, y), &arena).to_vec();
                for e in ev {
                    self.hero.apply_event(e);
                }
            }
        }
        self.paint();
    }

    /// Um clique no QUADRO em `world`.
    fn click_board(&mut self, world: [f64; 2]) {
        let a = area(self.hero.last_canvas);
        let p = self
            .hero
            .documents
            .active_board()
            .unwrap()
            .camera
            .to_screen(a, world);
        let (x, y) = (p[0] as f32, p[1] as f32);
        assert!(
            self.hero.chrome_hit(x, y).is_none(),
            "nenhum controlo sobre a nota"
        );
        for kind in [PointerKind::Down, PointerKind::Up] {
            self.clock += 1_000_000_000;
            let inp = Input {
                text: &mut self.ts,
                mods: Modifiers::default(),
                now_ns: self.clock,
            };
            board_view::pointer(
                &mut self.hero,
                inp,
                kind,
                PointerButton::Primary,
                x,
                y,
                true,
            );
        }
        self.paint();
    }

    fn board_key(&mut self, k: BoardKey, text: Option<&str>) {
        let m = Modifiers::default();
        board_keys::key(&mut self.hero, &mut self.ts, k, m, true, text, None);
        board_keys::key(&mut self.hero, &mut self.ts, k, m, false, None, None);
        self.paint();
    }

    fn hero_key(&mut self, keycode: u32) {
        let arena = Bump::new();
        let e = KeyEvent {
            keycode,
            modifiers: Modifiers::default(),
            kind: KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = self.hero.handle_key(e, &arena).to_vec();
        for e in ev {
            self.hero.apply_event(e);
        }
        self.paint();
    }

    fn write(&mut self, s: &str) {
        for ch in s.chars() {
            let arena = Bump::new();
            let ev = self.hero.handle_text_input(ch, &arena).to_vec();
            for e in ev {
                self.hero.apply_event(e);
            }
        }
        self.hero_key(KEY_ENTER);
    }

    fn tab_menu(&mut self, row: NodeId, what: &str) {
        let board = self.hero.documents.active().unwrap().0;
        self.hero.store.open_context_menu(ContextMenuRequest {
            x: 200.0,
            y: 40.0,
            kind: ContextMenuKind::BoardTab { board },
        });
        self.paint();
        self.click(row, what);
    }

    fn board(&self) -> &ph2d_board_model::Board {
        self.hero.documents.active_board().unwrap()
    }
}

/// As notas-lição cabem: o texto não as obriga a crescer (a altura de nascença), e todo o lançamento
/// está à vista ao abrir (o enquadramento de nascença com a margem dos painéis).
#[test]
fn the_lessons_fit_and_the_launch_is_in_view() {
    let s = Screen::new();
    let b = s.board();
    for id in s.l.lessons {
        assert_eq!(
            b.doc.get(id).unwrap().h,
            STICKY_SIDE,
            "a lição {} cresceu",
            id.0
        );
    }
    let a = area(s.hero.last_canvas);
    for f in s.l.frames {
        let e = b.doc.get(f).unwrap();
        for p in [[e.x, e.y], [e.x + e.w, e.y + e.h]] {
            let q = b.camera.to_screen(a, p);
            assert!(
                q[0] >= a[0] && q[1] >= a[1] && q[0] <= a[0] + a[2] && q[1] <= a[1] + a[3],
                "{q:?}"
            );
        }
    }
    assert!(
        s.find(node(Hit::Bubble(s.l.rui))).is_some(),
        "a bolha do Rui à vista"
    );
    let pin = b.team.thread(s.l.rui).unwrap().pin;
    assert!(
        matches!(pin, ph2d_board_model::Pin::Element { id, .. } if id == s.l.builds),
        "na nota dos builds"
    );
    assert!(
        s.find(node(Hit::Bubble(s.l.ana))).is_none(),
        "a da Ana está resolvida"
    );
}

/// ⭐ As cinco lições, na ordem real, pelo ecrã.
#[test]
fn every_team_lesson_does_what_it_says() {
    let mut s = Screen::new();
    // 1 — `C`, clicar numa nota, escrever, Enter; a bolha azul do Rui: ler e responder.
    s.board_key(BoardKey::Char('c'), Some("c"));
    let reviews = s.board().doc.live().find(|e| {
        e.shape()
            .is_some_and(|x| x.text.as_str() == ph2d_i18n::tr("board.smoke.team.n_reviews"))
    });
    let at = reviews.unwrap().center();
    s.click_board(at);
    s.write("Needs a reviewer rota");
    assert_eq!(s.board().team.threads.len(), 3, "um fio novo");
    let me = s.hero.documents.author().to_owned();
    assert!(
        s.board()
            .team
            .thread(s.l.rui)
            .unwrap()
            .unread_for(if me.is_empty() { "Me" } else { &me })
    );
    s.click(node(Hit::Bubble(s.l.rui)), "a bolha do Rui");
    s.write("CI, mostly");
    let rui = s.board().team.thread(s.l.rui).unwrap();
    assert_eq!(rui.messages.len(), 2, "respondido");
    s.hero_key(KEY_ESCAPE);
    s.board_key(BoardKey::Escape, None);
    assert!(!s.hero.documents.team_session().commenting);
    // 2 — Versions…, Save; Restore abre a «Kick-off» num quadro NOVO (sem a última ideia).
    let original = s.hero.documents.active().unwrap();
    s.tab_menu(ph2d_editor_core::ids::CTX_MENU_BOARD_VERSIONS, "Versions…");
    s.hero_key(KEY_ENTER);
    assert_eq!(s.board().team.versions.len(), 2);
    s.click(node(Hit::Restore(0)), "Restore da Kick-off");
    let restored = s.hero.documents.active().unwrap();
    assert_ne!(restored, original);
    let small = ph2d_i18n::tr("board.smoke.team.n_small");
    let has_small = |b: &ph2d_board_model::Board| {
        b.doc
            .live()
            .any(|e| e.shape().is_some_and(|x| x.text.as_str() == small))
    };
    assert!(
        !has_small(s.board()),
        "a Kick-off é de antes da última ideia"
    );
    assert!(
        has_small(s.hero.documents.boards().get(original).unwrap()),
        "o original não muda"
    );
    s.hero.documents.activate(Some(original));
    s.paint();
    s.paint();
    // 3 — Present: três diapositivos, as setas andam, Esc pára.
    s.click(button_node(NavButton::Present), "Present");
    let show = |s: &Screen| s.hero.documents.team_session().show.clone();
    assert_eq!(show(&s).unwrap().slides, s.l.frames.to_vec());
    s.board_key(BoardKey::Right, None);
    s.board_key(BoardKey::Right, None);
    assert_eq!(show(&s).unwrap().at, 2);
    s.board_key(BoardKey::Escape, None);
    assert!(show(&s).is_none());
    // 4 — Export…, PDF, Export… (o pedido à família) — e o que ela escreveria.
    s.tab_menu(ph2d_editor_core::ids::CTX_MENU_BOARD_EXPORT, "Export…");
    s.click(node(Hit::Button(TeamButton::ExportPdf)), "PDF");
    s.click(node(Hit::Button(TeamButton::ExportGo)), "Export…");
    let Some(IoRequest::Export {
        choice, selection, ..
    }) = s.hero.documents.take_io()
    else {
        panic!("o Export… não pediu o ficheiro");
    };
    assert_eq!(choice.format, Format::Pdf);
    let path = std::env::temp_dir().join(format!("ph2d_w8_scene10_{}.pdf", std::process::id()));
    let b = s.board().clone();
    let files = crate::io::write_export(&b, choice, &selection, None, &mut s.ts, &path).unwrap();
    let pdf = std::fs::read(&files[0]).unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert_eq!(
        text.matches("/Type /Page").count() - text.matches("/Type /Pages").count(),
        3,
        "um frame por página"
    );
    // 5 — Open an Excalidraw file…: o pedido, e o exemplo da lição abre.
    s.tab_menu(
        ph2d_editor_core::ids::CTX_MENU_BOARD_IMPORT,
        "Open an Excalidraw file…",
    );
    assert_eq!(s.hero.documents.take_io(), Some(IoRequest::Import));
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/MiroClone/example.excalidraw");
    let (name, imp) = crate::io::import_file(&example).unwrap();
    assert_eq!(name, "example");
    assert!(imp.doc.live_len() >= 10);
    assert_eq!(
        imp.skipped.get("image"),
        Some(&1),
        "a imagem salta e conta-se"
    );
}

/// 📷 As fotografias dos ESTADOS da cena 10 que a foto da janela não alcança (a tela virtual não aceita
/// cliques sintéticos): o fio aberto, as versões, o exportar e a apresentação — pela placa, a cena do
/// ecrã inteira. Corre à mão e LÊ-SE cada foto:
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-board --lib fotos_dos_estados -- --ignored --nocapture
/// ```
#[test]
#[ignore = "fotografias: correm-se à mão com a placa"]
fn fotos_dos_estados_da_cena_10() {
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("placa");
    let size = (HERO_VIEWPORT_W as u32, HERO_VIEWPORT_H as u32);
    let mut pass =
        ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, size).unwrap();
    let dir = std::env::temp_dir().join("ph2d_w8_fotos");
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = Screen::new();
    let mut shot = |s: &mut Screen, name: &str| {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut s.hero, vp, &mut scene, &mut s.ts);
        let px = pass.render_and_readback(&gpu, scene.inner(), size).unwrap();
        let img = ph2d_board_export::png::Image {
            width: size.0,
            height: size.1,
            rgba: px,
        };
        let path = dir.join(format!("{name}.png"));
        std::fs::write(&path, ph2d_board_export::png::encode(&img)).unwrap();
        println!("foto: {}", path.display());
    };
    shot(&mut s, "0_abertura");
    s.click(node(Hit::Bubble(s.l.rui)), "a bolha do Rui");
    shot(&mut s, "1_fio_do_rui");
    s.write("CI, mostly");
    s.hero_key(KEY_ESCAPE);
    s.click(button_node(NavButton::Comments), "Comments");
    shot(&mut s, "2_lista");
    s.click(button_node(NavButton::Comments), "Comments");
    s.tab_menu(ph2d_editor_core::ids::CTX_MENU_BOARD_VERSIONS, "Versions…");
    shot(&mut s, "3_versoes");
    s.hero_key(KEY_ESCAPE);
    s.click(node(Hit::Button(TeamButton::VersionsClose)), "fechar");
    s.tab_menu(ph2d_editor_core::ids::CTX_MENU_BOARD_EXPORT, "Export…");
    shot(&mut s, "4_exportar");
    s.click(node(Hit::Button(TeamButton::ExportClose)), "fechar");
    s.hero.motion.set_reduced_motion(true);
    s.click(button_node(NavButton::Present), "Present");
    shot(&mut s, "5_apresentar_1");
    s.board_key(BoardKey::Right, None);
    shot(&mut s, "6_apresentar_2");
}
