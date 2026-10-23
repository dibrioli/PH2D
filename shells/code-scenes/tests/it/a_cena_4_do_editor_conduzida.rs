//! ⭐⭐ **A cena 4 do editor de scripts ensina o que diz** (`PH2D_CODE_SMOKE=4`, o acabamento da Onda
//! 2): o roteiro das fotos (`ph2d_app_code::smoke_drive::STEPS_4`) conduzido pelas portas reais da aba
//! (tecla, rato, clique, o Ctrl no `WidgetStore`), sobre a cena montada como a shell a monta e pintada
//! a cada quadro como a shell a pinta; cada passo confere o que o smoke do dono promete — o
//! sublinhado do Ctrl+rato no `set` e não no `ampiltude` (a mão também), a lista de problemas pela
//! gravidade, o clique e o F8 no culpado, e a sequência dos buracos fundidos. Precisa do `luau-lsp`
//! e do StyLua fixados (`bash scripts/luau-lsp-fetch.sh`, `bash scripts/stylua-fetch.sh`).

use ph2d_app_code::CodeDocs;
use ph2d_app_code::smoke_drive::{Conductor, Gesture, STEPS_4, drive};
use ph2d_app_components::code_smoke::{CENAS, PROBLEMAS_LUAU, PROBLEMAS_NAME, SHAPE_Y, monta_em};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::doc_surface::{DocumentSurface, PointerLook};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{BandTab, Severity};
use ph2d_script::{ProjectScripts, ScriptHost, ScriptId};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecScene;
use ph2d_vector::VectorScene;
use std::ops::Range;
use std::time::{Duration, Instant};

/// O passo do relógio da família por quadro (as pausas do aviso e da lista contam-se nele).
const QUADRO_NS: u64 = 50_000_000;

/// A área da aba (a foto é maior; o roteiro só pede que o código e a faixa caibam).
const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1400.0,
    h: 900.0,
};

/// O que o roteiro escreve no passo 6.
const ANINHADO: &str = "ph2d.set(ph2d.set(entity, \"x\", value), \"x\", value)";

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
        let formas = monta_em(&mut sim, &mut cena, &mut mapa, 4);
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

    /// Um quadro da shell: o relógio, a cena, a família, e a pintura da aba (o rato e o clique do
    /// roteiro miram o que ela pintou).
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
        let id = self.id();
        if self.docs().text_of(id).is_some() {
            let mut scene = VectorScene::new();
            let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
            self.docs()
                .paint(id.0, AREA, &mut scene, &mut ts, &Theme::default());
            self.ts = ts;
        }
        // As respostas do assistente, da gramática e do formatador chegam por threads.
        std::thread::sleep(Duration::from_millis(2));
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
            .by_name(PROBLEMAS_NAME)
            .expect("problemas.luau no projecto")
    }

    fn texto(&mut self) -> String {
        let id = self.id();
        self.docs().text_of(id).expect("a aba abriu")
    }

    fn selecao(&mut self) -> Vec<Range<usize>> {
        let id = self.id();
        self.docs().selection_of(id)
    }

    fn pose(&self, i: usize) -> Transform {
        *self
            .sim
            .world()
            .get::<Transform>(self.formas[i])
            .expect("pose")
    }

    /// O ponteiro sobre o carácter `ahead` bytes depois de `agulha`.
    fn ponteiro(&mut self, agulha: &str, ahead: usize) -> Option<PointerLook> {
        let id = self.id();
        let at = byte(agulha, ahead);
        let (x, y) = self.docs().glyph_center(id, at).expect("à vista");
        self.docs().pointer_look(id.0, x, y)
    }
}

/// O byte `ahead` depois da única `agulha` do script da cena.
fn byte(agulha: &str, ahead: usize) -> usize {
    assert_eq!(PROBLEMAS_LUAU.matches(agulha).count(), 1, "{agulha:?}");
    PROBLEMAS_LUAU.find(agulha).unwrap() + ahead
}

/// A palavra `w` dentro da única `agulha` do script da cena.
fn palavra(agulha: &str, w: &str) -> Range<usize> {
    let b = byte(agulha, agulha.find(w).unwrap());
    b..b + w.len()
}

/// A lista de problemas como a família a ordena: `(gravidade, texto, linha 1-based)`.
fn lista(p: &mut Palco) -> Vec<(Severity, String, usize)> {
    let id = p.id();
    p.docs().problem_list_of(id)
}

/// A linha 1-based do byte `b` do script da cena.
fn linha_de(b: usize) -> usize {
    PROBLEMAS_LUAU[..b].matches('\n').count() + 1
}

fn confere(p: &mut Palco, passo: usize, buracos: &[Range<usize>]) {
    let id = p.id();
    let set = palavra("ph2d.set(self.id, \"y\"", "set");
    let sinn = palavra("math.sinn", "sinn");
    match passo {
        1 => {
            assert_eq!(
                p.docs().link_of(id),
                (Some(set.clone()), true),
                "o `set` sublinha"
            );
            assert_eq!(
                p.ponteiro("ph2d.set(self.id, \"y\"", 6),
                Some(PointerLook::Hand),
                "e o ponteiro é a mão"
            );
        }
        2 => {
            assert_eq!(p.docs().link_of(id), (None, false), "o `ampiltude` não");
            assert_eq!(
                p.docs().reach_settled_of(id),
                Some(false),
                "e o assistente já respondeu"
            );
            assert_eq!(p.ponteiro("self.ampiltude", 8), Some(PointerLook::Text));
        }
        3 => {
            assert_eq!(p.docs().band_tab_of(id), BandTab::Problems);
            let l = lista(p);
            let erro = linha_de(sinn.start);
            let antes = linha_de(byte("local sobra", 6));
            let depois = linha_de(byte("self.ampiltude", 5));
            assert!(
                antes < erro && erro < depois,
                "um aviso antes do erro, outro depois"
            );
            assert_eq!(
                l,
                [
                    (
                        Severity::Error,
                        "\u{201c}math\u{201d} has no \u{201c}sinn\u{201d}".into(),
                        erro
                    ),
                    (
                        Severity::Warning,
                        "\u{201c}sobra\u{201d} is never used".into(),
                        antes
                    ),
                    (
                        Severity::Warning,
                        "self.ampiltude is never set \u{2014} did you mean amplitude?".into(),
                        depois,
                    ),
                ],
                "o erro primeiro, depois os avisos pela linha (com a explicação de cada um)"
            );
            let banda = p.docs().band_of(id).expect("pintada");
            let pintadas: Vec<_> = banda
                .problems
                .iter()
                .map(|r| (r.severity, r.text.clone(), r.line))
                .collect();
            assert_eq!(pintadas, l, "a faixa pinta a mesma lista");
        }
        4 => {
            let culpado = palavra("math.sinn", "math.sinn");
            assert_eq!(
                p.selecao(),
                [culpado],
                "o clique selecciona o culpado do erro"
            );
            p.quadro();
            let banda = p.docs().band_of(id).expect("pintada");
            assert!(banda.problems[0].selected, "e a linha do erro fica marcada");
        }
        5 => {
            let sel = p.selecao();
            let sobra = palavra("local sobra", "sobra");
            assert_eq!(sel, [sobra], "o F8 vai para o 1.º aviso (o de cima)");
        }
        6 => {
            let t = p.texto();
            assert!(
                t.contains(&format!(
                    "  -- 3) escreva aqui, numa linha nova\n  {ANINHADO}\nend\n"
                )),
                "{t}"
            );
            assert!(
                p.docs().snippet_of(id).is_some(),
                "os buracos continuam vivos"
            );
            assert_eq!(buracos.len(), 3, "o de dentro + dois Tabs: {buracos:?}");
        }
        7 => {
            let t = p.texto();
            assert!(
                t.contains(&format!("  {ANINHADO}\nend\n")),
                "nenhum espaço entrou: {t}"
            );
            assert_eq!(
                p.docs().snippet_of(id),
                None,
                "a sessão acabou no fim de fora"
            );
            // A sequência inteira: os de dentro, o fim da chamada de dentro, os de fora, o fim.
            let b = t.find(ANINHADO).unwrap();
            let em = |w: &str, n: usize| -> Range<usize> {
                let i = ANINHADO.match_indices(w).nth(n).unwrap().0;
                b + i..b + i + w.len()
            };
            let fim_dentro = b + ANINHADO.find(')').unwrap() + 1;
            let fim = b + ANINHADO.len();
            assert_eq!(
                buracos,
                [
                    em("entity", 0),
                    em("x", 0),
                    em("value", 0),
                    fim_dentro..fim_dentro,
                    em("x", 1),
                    em("value", 1),
                    fim..fim,
                ],
                "a ordem do Tab nos buracos fundidos"
            );
        }
        _ => unreachable!("passo {passo}"),
    }
}

#[test]
fn a_cena_4_conduzida_passo_a_passo() {
    assert_eq!(
        CENAS,
        ph2d_app_code::smoke::CENAS,
        "as cenas contam-se nos dois lados"
    );
    for g in STEPS_4.iter().flat_map(|s| s.iter()) {
        if let Gesture::Go { needle, .. } | Gesture::Point { needle, .. } = g {
            let n = PROBLEMAS_LUAU.matches(needle).count();
            assert_eq!(n, 1, "a agulha {needle:?} existe uma vez");
        }
    }
    let mut p = Palco::novo();
    let id = p.id();
    // Passo 0 — como o dono a recebe: a aba aberta, as formas PARADAS pelo erro, os três problemas.
    for _ in 0..30 {
        p.quadro();
    }
    assert_eq!(p.texto(), PROBLEMAS_LUAU);
    for i in 0..p.formas.len() {
        assert_eq!(
            p.pose(i).translation.y,
            SHAPE_Y,
            "o erro pára as três (forma {i})"
        );
    }

    let mut c = Conductor::scene(STEPS_4, STEPS_4.len());
    let mut feitos = 0;
    let mut buracos: Vec<Range<usize>> = Vec::new();
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
        // Cada selecção por onde o Tab passa depois de o `set` de dentro entrar.
        if p.texto().contains(ANINHADO) && p.docs().completion_of(id).is_none() {
            let s = p.selecao();
            if let [r] = s.as_slice()
                && buracos.last() != Some(r)
            {
                buracos.push(r.clone());
            }
        }
        while feitos < c.steps_done() {
            feitos += 1;
            // A pintura do quadro em que o passo acabou (o que a foto mostra).
            p.quadro();
            confere(&mut p, feitos, &buracos);
        }
    }
    if c.failed() {
        let lista = p.docs().problem_list_of(id);
        let cursor = p.docs().cursor_of(id);
        let assistente = p.docs().assist_reason();
        panic!(
            "o roteiro desistiu no passo {} — cursor {cursor:?}, problemas {lista:?}, assistente \
             {assistente:?}, texto:\n{}",
            c.steps_done() + 1,
            p.texto()
        );
    }
    assert_eq!(feitos, STEPS_4.len());
}
