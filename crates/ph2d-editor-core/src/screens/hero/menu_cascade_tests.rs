//! A cascata de verdade, pelo ponteiro e pelo teclado reais sobre o quadro pintado.

use super::*;
use crate::interaction::dispatch::keymap as k;
use crate::screens::hero::menu_bar::{CASCADES, MENUS};
use bumpalo::Bump;
use ph2d_host::{Modifiers, PointerButton, PointerSource};
use ph2d_text::TextSystem;

const JANELA: Rect = Rect::new(0.0, 0.0, 1366.0, 900.0);

struct Palco {
    hero: HeroScreen,
    ts: TextSystem,
}

impl Palco {
    fn novo() -> Self {
        crate::test_support::ensure_panel_registry();
        let mut p = Self {
            hero: HeroScreen::new(NodeId(1)),
            ts: TextSystem::without_system_fonts(),
        };
        p.pinta();
        p
    }

    fn pinta(&mut self) {
        let mut scene = ph2d_vector::VectorScene::new();
        crate::screens::hero::paint_hero_screen(&mut self.hero, JANELA, &mut scene, &mut self.ts);
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) {
        let arena = Bump::new();
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs: Vec<WidgetEvent> = self.hero.handle_pointer(ev, &arena).to_vec();
        for e in evs {
            self.hero.apply_event(e);
        }
        self.pinta();
    }

    fn centro(&self, id: NodeId) -> (f32, f32) {
        let r = self
            .hero
            .hit_index
            .rect_for(id)
            .unwrap_or_else(|| panic!("{id:?} não está no quadro"));
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    fn paira(&mut self, id: NodeId) {
        let (x, y) = self.centro(id);
        self.ponteiro(PointerKind::Move, x, y);
    }

    fn clica(&mut self, id: NodeId) {
        let (x, y) = self.centro(id);
        self.ponteiro(PointerKind::Move, x, y);
        self.ponteiro(PointerKind::Down, x, y);
        self.ponteiro(PointerKind::Up, x, y);
    }

    fn tecla(&mut self, keycode: u32, kind: KeyKind) {
        let arena = Bump::new();
        let ev = KeyEvent {
            keycode,
            modifiers: Modifiers::default(),
            kind,
            timestamp_ns: 0,
        };
        let evs: Vec<WidgetEvent> = self.hero.handle_key(ev, &arena).to_vec();
        for e in evs {
            self.hero.apply_event(e);
        }
        self.pinta();
    }

    fn toca(&mut self, keycode: u32) {
        self.tecla(keycode, KeyKind::Down);
        self.tecla(keycode, KeyKind::Up);
    }

    fn cadeia(&self) -> Vec<ContextMenuKind> {
        self.hero
            .store
            .menu_chain()
            .iter()
            .map(|r| r.kind)
            .collect()
    }
}

use ContextMenuKind as K;

/// O botão «Menu» abre os cinco; pairar numa linha abre o menu dela AO LADO e o pai fica.
#[test]
fn o_menu_abre_os_cinco_e_pairar_abre_o_filho_ao_lado() {
    let mut p = Palco::novo();
    p.clica(ids::MENUBAR_W);
    assert_eq!(p.cadeia(), [K::MenuBarRoot]);
    for (id, _, kind) in MENUS {
        p.paira(id);
        assert_eq!(p.cadeia(), [K::MenuBarRoot, kind], "pairar em {id:?}");
        let mae = p
            .hero
            .hit_index
            .rect_for(id)
            .expect("a linha-mãe fica pintada");
        let filho = p.hero.store.context_menu().expect("aberto");
        assert!(filho.x >= mae.x + mae.w, "o filho abre à direita da mãe");
        assert!(p.hero.store.menu_row_has_open_child(id), "a mãe fica acesa");
    }
}

/// Três níveis: Menu ▸ File ▸ Export, e a folha do fundo age e fecha tudo.
#[test]
fn tres_niveis_ficam_abertos_e_a_folha_fecha_a_cascata() {
    let mut p = Palco::novo();
    p.clica(ids::MENUBAR_W);
    p.paira(ids::MENUBAR_FILE);
    p.paira(ids::MENUBAR_FILE_NEW_MENU);
    assert_eq!(
        p.cadeia(),
        [K::MenuBarRoot, K::MenuBarFile, K::MenuBarFileNew]
    );
    // Uma irmã sem submenu fecha só o neto.
    p.paira(ids::CTX_MENU_SAVE);
    assert_eq!(p.cadeia(), [K::MenuBarRoot, K::MenuBarFile]);
    // A folha age: *New Image…* troca a cascata pelo diálogo dela.
    p.paira(ids::MENUBAR_FILE_NEW_MENU);
    p.clica(ids::MENUBAR_FILE_NEW);
    assert_eq!(p.cadeia(), [K::NewImageDialog]);
}

/// Clicar numa linha-cascata não fecha nada (o Down é do menu) e o segundo clique no «Menu» fecha.
#[test]
fn clicar_na_cascata_mantem_o_pai_e_o_botao_fecha() {
    let mut p = Palco::novo();
    p.clica(ids::MENUBAR_W);
    p.clica(ids::MENUBAR_WINDOW);
    assert_eq!(p.cadeia(), [K::MenuBarRoot, K::MenuBarWindow]);
    p.clica(ids::MENUBAR_WINDOW_PIXEL_MENU);
    assert_eq!(
        p.cadeia(),
        [K::MenuBarRoot, K::MenuBarWindow, K::MenuBarWindowPixel]
    );
    p.clica(ids::MENUBAR_W);
    assert!(p.cadeia().is_empty(), "o segundo clique no Menu fecha");
}

/// As Preferências empilham também: Menu ▸ Edit ▸ Preferences ▸ Pixels per meter.
#[test]
fn as_preferencias_abrem_em_cascata() {
    let mut p = Palco::novo();
    p.clica(ids::MENUBAR_W);
    p.paira(ids::MENUBAR_EDIT);
    p.paira(ids::MENUBAR_EDIT_PREFERENCES);
    p.paira(ids::CTX_MENU_SETTINGS_PPM);
    assert_eq!(
        p.cadeia(),
        [
            K::MenuBarRoot,
            K::MenuBarEdit,
            K::SettingsMenu,
            K::SettingsPpmSubmenu
        ]
    );
}

/// ⭐ Toda linha com «›» abre MESMO um nível a mais ao pairar — e nenhuma linha sem «›» o faz.
/// (Pairar re-toca o clique: uma linha de ação na lista do «›» seria executada só de passar.)
#[test]
fn toda_linha_com_seta_abre_um_nivel_e_so_elas() {
    let mut pais: Vec<(Vec<NodeId>, ContextMenuKind)> = vec![(vec![], K::MenuBarRoot)];
    let mut vistos = 0;
    while let Some((caminho, kind)) = pais.pop() {
        for id in level_rows(&Palco::novo().hero.store, kind) {
            if !opens_submenu(id) {
                continue;
            }
            let mut p = Palco::novo();
            p.clica(ids::MENUBAR_W);
            for passo in &caminho {
                p.paira(*passo);
            }
            let antes = p.cadeia();
            if p.hero.store.menu_row_is_unavailable(id) {
                continue;
            }
            p.paira(id);
            let depois = p.cadeia();
            assert_eq!(depois.len(), antes.len() + 1, "{id:?} tem «›» e não abriu");
            assert_eq!(depois[..antes.len()], antes[..], "{id:?} trocou o pai");
            vistos += 1;
            let mut c = caminho.clone();
            c.push(id);
            pais.push((c, *depois.last().expect("abriu")));
        }
    }
    // 5 menus + 7 submenus da onda 1 + Preferences + Theme + as 13 categorias das Preferências.
    assert_eq!(vistos, CASCADES.len() + 2 + 13);
}

/// A diagonal a caminho do filho passa por cima das irmãs sem trocar de submenu.
#[test]
fn a_diagonal_para_o_filho_nao_troca_de_submenu() {
    let mut p = Palco::novo();
    p.clica(ids::MENUBAR_W);
    p.paira(ids::MENUBAR_FILE);
    let (fx, fy) = p.centro(ids::MENUBAR_FILE);
    // Do File, em diagonal para dentro do menu File, passando sobre a linha Edit da raiz.
    let (ex, ey) = p.centro(ids::MENUBAR_EDIT);
    let alvo = p.centro(ids::CTX_MENU_SAVE);
    p.ponteiro(PointerKind::Move, fx, fy);
    let meio = (ex + (alvo.0 - fx) * 0.4, ey);
    assert_eq!(
        p.hero.hit_index.hit(meio.0, meio.1),
        Some(ids::MENUBAR_EDIT)
    );
    p.ponteiro(PointerKind::Move, meio.0, meio.1);
    assert_eq!(
        p.cadeia(),
        [K::MenuBarRoot, K::MenuBarFile],
        "trocou a meio"
    );
    // Um movimento VERTICAL sobre a irmã troca.
    p.ponteiro(PointerKind::Move, meio.0, meio.1 + 1.0);
    assert_eq!(p.cadeia(), [K::MenuBarRoot, K::MenuBarEdit]);
}

/// `Alt` sozinho abre e fecha o «Menu»; com outra tecla no meio, ou o ponteiro a mexer, não.
#[test]
fn o_alt_sozinho_abre_o_menu() {
    let mut p = Palco::novo();
    p.toca(ALT_KEYS[0]);
    assert_eq!(p.cadeia(), [K::MenuBarRoot]);
    p.toca(ALT_KEYS[0]);
    assert!(p.cadeia().is_empty());
    // Alt + outra tecla é um atalho.
    p.tecla(ALT_KEYS[0], KeyKind::Down);
    p.tecla(k::KEY_KEY_A, KeyKind::Down);
    p.tecla(k::KEY_KEY_A, KeyKind::Up);
    p.tecla(ALT_KEYS[0], KeyKind::Up);
    assert!(p.cadeia().is_empty(), "Alt+A abriu o menu");
    // Alt segurado com o ponteiro a mexer é um gesto da tela.
    p.tecla(ALT_KEYS[1], KeyKind::Down);
    p.ponteiro(PointerKind::Move, 600.0, 500.0);
    p.tecla(ALT_KEYS[1], KeyKind::Up);
    assert!(p.cadeia().is_empty(), "Alt de um gesto abriu o menu");
}

/// As setas andam pelos menus: ↓ ↓, → abre, ← volta, Enter escolhe, Esc fecha.
#[test]
fn as_setas_andam_pela_cascata() {
    let mut p = Palco::novo();
    p.toca(ALT_KEYS[0]);
    assert_eq!(p.hero.store.hot_id(), Some(ids::MENUBAR_FILE));
    p.toca(k::KEY_ARROW_DOWN);
    assert_eq!(p.hero.store.hot_id(), Some(ids::MENUBAR_EDIT));
    p.toca(k::KEY_ARROW_UP);
    p.toca(k::KEY_ARROW_RIGHT);
    assert_eq!(p.cadeia(), [K::MenuBarRoot, K::MenuBarFile]);
    assert_eq!(p.hero.store.hot_id(), Some(ids::MENUBAR_FILE_NEW_MENU));
    p.toca(k::KEY_ARROW_LEFT);
    assert_eq!(p.cadeia(), [K::MenuBarRoot]);
    assert_eq!(p.hero.store.hot_id(), Some(ids::MENUBAR_FILE));
    // ↑ na primeira dá a volta: Help; Enter abre-o; Enter em Commands… abre a busca.
    p.toca(k::KEY_ARROW_UP);
    p.toca(k::KEY_ENTER);
    assert_eq!(p.cadeia(), [K::MenuBarRoot, K::MenuBarHelp]);
    assert_eq!(p.hero.store.hot_id(), Some(ids::MENUBAR_HELP_COMMANDS));
    p.toca(k::KEY_ENTER);
    assert!(p.cadeia().is_empty());
    assert!(
        p.hero.store.command_palette_open(),
        "Commands… abre a busca"
    );
    p.hero.store.close_command_palette();
    p.toca(ALT_KEYS[0]);
    p.toca(k::KEY_ESCAPE);
    assert!(p.cadeia().is_empty(), "Esc fecha");
}

/// O campo «Find anything» abre a busca e pede à shell as linhas das famílias.
#[test]
fn o_campo_da_faixa_abre_a_busca() {
    let mut p = Palco::novo();
    p.clica(ids::MENUBAR_FIND);
    assert!(p.hero.store.command_palette_open());
    assert!(p.hero.store.take_global_palette_request());
    assert!(
        !p.hero.store.take_global_palette_request(),
        "o pedido toma-se uma vez"
    );
}

/// A seta do Play escolhe o palco: a linha joga lá e o palco fica lembrado (com a marca).
#[test]
fn a_seta_do_play_escolhe_o_palco() {
    use crate::game_stage::GameStage;
    let mut p = Palco::novo();
    p.clica(ids::MENUBAR_PLAY_MENU);
    assert_eq!(p.cadeia(), [K::MenuBarPlay]);
    p.clica(ids::MENUBAR_RUN_WINDOW);
    assert!(p.cadeia().is_empty());
    assert_eq!(p.hero.view.game_stage, GameStage::Window);
    assert_eq!(
        crate::screens::hero::menu_bar::module_is_on(&p.hero, ids::MENUBAR_RUN_WINDOW),
        Some(true)
    );
}
