//! ⭐⭐ **A cena 5 do editor de scripts ensina o que diz** (`PH2D_CODE_SMOKE=5`, a página de ajuda,
//! G10 de `docs/CodeEditor/04_pagina_de_ajuda.md` §6): o roteiro das fotos
//! (`ph2d_app_code::smoke_drive::STEPS_5`) conduzido pelas portas reais da aba (tecla, rato, clique,
//! o Ctrl no `WidgetStore`), sobre a cena montada como a shell a monta e com a aba ACTIVA pintada a
//! cada quadro como a shell a pinta; cada passo confere o que o smoke do dono promete — a mão no
//! `sin`, a página `math` no `sin` (assinatura e exemplo pintados), o link na mesma página que só
//! rola, o Alt+← que volta ao SCRIPT (o link na mesma página não empilha, como no Godot 4.7.2), o
//! F12 na `ph2d` com os três argumentos e o exemplo, o `PoseField` que leva à `types`, o Alt+← que
//! volta ao `set` na mesma rolagem, a contagem do Ctrl+F, o Ctrl+W, e nada sobre `local`. Precisa
//! do `luau-lsp` fixado (`bash scripts/luau-lsp-fetch.sh`).

use ph2d_app_code::smoke_drive::{Conductor, Gesture, STEPS_5, drive};
use ph2d_app_code::{CodeDocs, member_id};
use ph2d_app_components::code_smoke::{AJUDA_LUAU, AJUDA_NAME, CENAS, SHAPE_X, monta_em};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::doc_surface::{DocumentSurface, PointerLook};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::HelpProbe;
use ph2d_script::{ProjectScripts, ScriptHost, ScriptId};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecScene;
use ph2d_vector::VectorScene;
use std::ops::Range;
use std::time::{Duration, Instant};

/// O passo do relógio da família por quadro.
const QUADRO_NS: u64 = 50_000_000;

/// A área da aba (a da foto é maior; a página do `set` tem de caber com os argumentos e o exemplo).
const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1400.0,
    h: 900.0,
};

struct Palco {
    sim: SimWorld,
    hero: HeroScreen,
    host: ScriptHost,
    toasts: ph2d_editor_core::ToastQueue,
    ts: TextSystem,
    formas: Vec<Entity>,
    agora: u64,
}

impl Palco {
    fn novo() -> Self {
        let mut sim = SimWorld::new();
        let mut cena = VecScene::new();
        let mut mapa = VecEntityMap::new();
        let formas = monta_em(&mut sim, &mut cena, &mut mapa, 5);
        let mut hero = HeroScreen::new(NodeId(1));
        // O que a shell instala (`init.rs`): o assistente e o formatador fixados, por omissão.
        hero.documents
            .install_surface(Box::new(CodeDocs::default()));
        Self {
            sim,
            hero,
            host: ScriptHost::new().expect("vm"),
            toasts: ph2d_editor_core::ToastQueue::default(),
            ts: TextSystem::without_system_fonts(),
            formas,
            agora: 1_000_000_000_000,
        }
    }

    /// Um quadro da shell: o relógio, a cena, a família, e a pintura da aba ACTIVA (o rato e os
    /// cliques do roteiro miram o que ela pintou).
    fn quadro(&mut self) {
        self.agora += QUADRO_NS;
        self.hero.store.note_clock(u128::from(self.agora));
        self.host.scene_sync(self.sim.world_mut());
        let _ = self.host.scene_tick(&mut self.sim, 1.0 / 60.0);
        ph2d_app_code::frame(
            &mut self.sim,
            Some(&mut self.host),
            &mut self.hero,
            &mut self.toasts,
            false,
        );
        self.hero.documents.follow_surfaces();
        if let Some(tab) = self.ativa() {
            let mut scene = VectorScene::new();
            let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
            self.docs()
                .paint(tab, AREA, &mut scene, &mut ts, &Theme::default());
            self.ts = ts;
        }
        // As respostas do assistente e da gramática chegam por threads.
        std::thread::sleep(Duration::from_millis(2));
    }

    fn ativa(&self) -> Option<u64> {
        self.hero.documents.surface_active().map(|t| t.id)
    }

    fn docs(&mut self) -> &mut CodeDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<CodeDocs>()
            .expect("família instalada")
    }

    fn id(&self) -> ScriptId {
        self.sim
            .world()
            .resource::<ProjectScripts>()
            .by_name(AJUDA_NAME)
            .expect("ajuda.luau no projecto")
    }

    fn texto(&mut self) -> String {
        let id = self.id();
        self.docs().text_of(id).expect("a aba abriu")
    }

    fn cursor(&mut self) -> usize {
        let id = self.id();
        self.docs().cursor_of(id).expect("aberta").1
    }

    fn x(&self, i: usize) -> f32 {
        self.sim
            .world()
            .get::<Transform>(self.formas[i])
            .expect("pose")
            .translation
            .x
    }

    /// A página `page` é a aba activa (e a da fila da família com esse título).
    fn na(&mut self, page: &str) -> HelpProbe {
        let tab = self.docs().help_tab_id(page);
        assert!(tab.is_some(), "a página {page} não está aberta");
        assert_eq!(self.ativa(), tab, "a aba activa não é a página {page}");
        let probe = self.docs().help_probe(page).expect("pintada");
        let titulo = self
            .docs()
            .tabs()
            .into_iter()
            .find(|t| Some(t.id) == tab)
            .expect("na fila")
            .title;
        assert_eq!(probe.title, titulo, "o título pintado é o da aba");
        probe
    }

    fn abertas(&mut self) -> Vec<&'static str> {
        self.docs().help_tabs().iter().map(|t| t.0).collect()
    }

    /// O ponteiro sobre o carácter `ahead` bytes depois de `agulha` (a aba do script).
    fn ponteiro(&mut self, agulha: &str, ahead: usize) -> Option<PointerLook> {
        let id = self.id();
        let (x, y) = self
            .docs()
            .glyph_center(id, byte(agulha, ahead))
            .expect("à vista");
        self.docs().pointer_look(id.0, x, y)
    }
}

/// O byte `ahead` depois da única `agulha` do script da cena.
fn byte(agulha: &str, ahead: usize) -> usize {
    assert_eq!(AJUDA_LUAU.matches(agulha).count(), 1, "{agulha:?}");
    AJUDA_LUAU.find(agulha).unwrap() + ahead
}

/// A palavra `w` dentro da única `agulha` do script da cena.
fn palavra(agulha: &str, w: &str) -> Range<usize> {
    let b = byte(agulha, agulha.find(w).unwrap());
    b..b + w.len()
}

/// Alguma linha pintada contém `s`.
fn pintou(p: &HelpProbe, s: &str) -> bool {
    p.lines.iter().any(|l| l.contains(s))
}

fn topo(p: &HelpProbe, page: &str, member: &str) {
    assert_eq!(
        p.top_member,
        member_id(page, member),
        "a página {page} não está no {member}: {:#?}",
        p.lines
    );
}

/// O que se conferiu num passo e é preciso noutro: o cursor do Ctrl+clique e do F12, e a rolagem
/// da `ph2d` no `set`.
#[derive(Default)]
struct Memoria {
    cursor_sin: usize,
    cursor_set: usize,
    rolagem_set: f32,
}

#[allow(clippy::too_many_lines)]
fn confere(p: &mut Palco, passo: usize, mem: &mut Memoria) {
    let id = p.id();
    match passo {
        1 => {
            assert_eq!(
                p.docs().link_of(id),
                (Some(palavra("math.sin(", "sin")), true),
                "o `sin` sublinha"
            );
            assert_eq!(
                p.ponteiro("math.sin(", 6),
                Some(PointerLook::Hand),
                "e o ponteiro é a mão"
            );
            assert!(p.abertas().is_empty(), "o Ctrl+rato não abre nada");
        }
        2 => {
            let pr = p.na("math");
            topo(&pr, "math", "sin");
            assert!(
                pintou(&pr, "math.sin(n: number): number"),
                "a assinatura do sin: {:#?}",
                pr.lines
            );
            assert!(
                pintou(&pr, "print(math.sin(math.pi / 2)) --> 1"),
                "o exemplo do sin: {:#?}",
                pr.lines
            );
            assert!(pintou(&pr, "The angle, in radians."), "o argumento `n`");
            assert_eq!(p.abertas(), ["math"]);
            let math = p.docs().help_tab_id("math").unwrap();
            assert_eq!(p.docs().nav_can(math), (true, false), "‹ aceso, › apagado");
            mem.cursor_sin = p.cursor();
            assert!(
                palavra("math.sin(", "sin").contains(&mem.cursor_sin)
                    || mem.cursor_sin == palavra("math.sin(", "sin").end,
                "o Ctrl+clique pôs o cursor no `sin`: {}",
                mem.cursor_sin
            );
        }
        3 => {
            let pr = p.na("math");
            topo(&pr, "math", "noise");
            assert_eq!(p.abertas(), ["math"], "a mesma aba");
            let math = p.docs().help_tab_id("math").unwrap();
            assert_eq!(
                p.docs().nav_can(math),
                (true, false),
                "o link na mesma página não empilhou"
            );
        }
        4 => {
            assert_eq!(p.ativa(), Some(id.0), "o Alt+← volta ao SCRIPT");
            assert_eq!(p.cursor(), mem.cursor_sin, "no cursor do Ctrl+clique");
            assert_eq!(p.abertas(), ["math"], "a página fica aberta");
        }
        5 => {
            let pr = p.na("ph2d");
            topo(&pr, "ph2d", "set");
            for (o_que, s) in [
                ("o argumento entity", "The object to change, usually"),
                ("o argumento field", "Which part of the pose:"),
                (
                    "o argumento value",
                    "The new value. It must be a real number:",
                ),
                ("o exemplo", "ph2d.set(self.id, \"x\", x + self.speed * dt)"),
            ] {
                assert!(pintou(&pr, s), "{o_que} não foi pintado: {:#?}", pr.lines);
            }
            mem.cursor_set = p.cursor();
            assert_eq!(
                mem.cursor_set,
                byte("ph2d.set(self.id", 6),
                "o F12 não mexe no cursor"
            );
            mem.rolagem_set = pr.scroll;
            assert_eq!(p.abertas(), ["math", "ph2d"]);
        }
        6 => {
            let pr = p.na("types");
            topo(&pr, "types", "PoseField");
            assert_eq!(p.abertas(), ["math", "ph2d", "types"], "uma aba por página");
            let types = p.docs().help_tab_id("types").unwrap();
            assert_eq!(p.docs().nav_can(types), (true, false));
        }
        7 => {
            let pr = p.na("ph2d");
            topo(&pr, "ph2d", "set");
            assert!(
                (pr.scroll - mem.rolagem_set).abs() < 0.5,
                "a MESMA rolagem: {} × {}",
                pr.scroll,
                mem.rolagem_set
            );
            let ph2d = p.docs().help_tab_id("ph2d").unwrap();
            assert_eq!(p.docs().nav_can(ph2d), (true, true), "‹ e › acesos");
        }
        8 => {
            let _ = p.na("ph2d");
            let plano = p.docs().help_plain("ph2d").expect("o texto da página");
            let n = plano.to_lowercase().matches("entity").count();
            assert!(n >= 2, "controlo: a página tem `entity` ({n})");
            assert_eq!(
                p.docs().help_find_count_text("ph2d").as_deref(),
                Some(format!("1 of {n}").as_str()),
                "a contagem da busca"
            );
            println!("[cena 5] Ctrl+F «entity» na página ph2d: 1 of {n}");
        }
        9 => {
            assert_eq!(p.ativa(), Some(id.0), "o Ctrl+W volta ao script");
            assert_eq!(p.docs().help_tab_id("ph2d"), None, "a página fechou");
            assert_eq!(p.abertas(), ["math", "types"], "as outras ficam");
            assert_eq!(p.cursor(), mem.cursor_set, "no cursor do F12");
        }
        10 => {
            // Nada: nem sublinhado, nem mão, nem página — em quadros com tempo para uma resposta.
            let abertas = p.abertas();
            for _ in 0..25 {
                p.quadro();
                assert_eq!(p.docs().link_of(id), (None, false), "`local` sublinhou");
                assert_eq!(
                    p.ponteiro("local m = math", 2),
                    Some(PointerLook::Text),
                    "`local` deu a mão"
                );
                assert_eq!(p.docs().reach_settled_of(id), None, "perguntou por `local`");
                assert_eq!(p.abertas(), abertas);
                assert_eq!(p.ativa(), Some(id.0));
            }
        }
        _ => unreachable!("passo {passo}"),
    }
}

#[test]
fn a_cena_5_conduzida_passo_a_passo() {
    assert_eq!(
        CENAS,
        ph2d_app_code::smoke::CENAS,
        "as cenas contam-se nos dois lados"
    );
    for g in STEPS_5.iter().flat_map(|s| s.iter()) {
        if let Gesture::Go { needle, .. }
        | Gesture::Point { needle, .. }
        | Gesture::Click { needle, .. } = g
        {
            let n = AJUDA_LUAU.matches(needle).count();
            assert_eq!(n, 1, "a agulha {needle:?} existe uma vez");
        }
    }
    let mut p = Palco::novo();
    let id = p.id();
    // Passo 0 — como o dono a recebe: a aba aberta, sem problemas, e as três formas a balançar.
    let fim = Instant::now() + Duration::from_secs(30);
    loop {
        p.quadro();
        if p.docs().assist_diagnostics(id).1.is_some() {
            break;
        }
        assert!(
            Instant::now() < fim,
            "o assistente não verificou o script: {:?}",
            p.docs().assist_reason()
        );
    }
    for _ in 0..30 {
        p.quadro();
    }
    assert_eq!(p.texto(), AJUDA_LUAU);
    assert_eq!(p.ativa(), Some(id.0));
    let problemas = p.docs().problems_of(id);
    assert!(
        problemas.is_empty(),
        "o script da cena não tem problemas: {problemas:?}"
    );
    for (i, &x0) in SHAPE_X.iter().enumerate().take(3) {
        #[allow(clippy::cast_possible_truncation)]
        let d = (p.x(i) - x0 as f32).abs();
        assert!(d > 0.01, "a forma {i} não balançou ({d})");
    }
    #[allow(clippy::cast_possible_truncation)]
    let quadrado = SHAPE_X[3] as f32;
    assert!((p.x(3) - quadrado).abs() < f32::EPSILON, "o quadrado fica");

    let mut c = Conductor::scene(STEPS_5, STEPS_5.len());
    let mut feitos = 0;
    let mut mem = Memoria::default();
    let prazo = Instant::now() + Duration::from_secs(120);
    while !c.finished() {
        assert!(
            Instant::now() < prazo,
            "o roteiro parou no passo {}",
            c.steps_done() + 1
        );
        p.quadro();
        drive(p.docs(), &mut c);
        if let Some(held) = c.ctrl() {
            p.hero.store.set_cmd_held(held);
        }
        while feitos < c.steps_done() {
            feitos += 1;
            // A pintura do quadro em que o passo acabou (o que a foto mostra).
            p.quadro();
            confere(&mut p, feitos, &mut mem);
        }
    }
    if c.failed() {
        let ativa = p.ativa();
        let abertas = p.abertas();
        let cursor = p.docs().cursor_of(id);
        let assistente = p.docs().assist_reason();
        panic!(
            "o roteiro desistiu no passo {} — activa {ativa:?}, páginas {abertas:?}, cursor \
             {cursor:?}, assistente {assistente:?}",
            c.steps_done() + 1,
        );
    }
    assert_eq!(feitos, STEPS_5.len());
}
