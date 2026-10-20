//! ⭐⭐ **A cena 3 do editor de scripts ensina o que diz** (`PH2D_CODE_SMOKE=3`, CodeEditor I15): o
//! roteiro das fotos (`ph2d_app_code::smoke_drive::STEPS`) conduzido pelas teclas reais da aba, sobre
//! a cena montada como a shell a monta, e cada passo confere o estado que o smoke do dono promete —
//! as formas paradas pelo erro, o «Did you mean amplitude?», a lista do `ph2d.`, o esqueleto, as
//! formas a girar, a caixinha do F2 e o 2 da estrela no nome novo. Precisa do `luau-lsp` e do StyLua
//! fixados (`bash scripts/luau-lsp-fetch.sh`, `bash scripts/stylua-fetch.sh`).

use ph2d_app_code::CodeDocs;
use ph2d_app_code::smoke_drive::{Conductor, Gesture, STEPS, drive};
use ph2d_app_components::code_smoke::{
    CENAS, ONDA_LUAU, ONDA_NAME, SHAPE_Y, STAR_AMPLITUDE, monta_em,
};
use ph2d_app_components::script_inspector::build_info;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::script_edits::InspectorScriptValue;
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost, ScriptId, ScriptValue};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecScene;
use std::time::{Duration, Instant};

/// O passo do relógio da família por quadro (as pausas do aviso e da lista contam-se nele).
const QUADRO_NS: u64 = 50_000_000;

struct Palco {
    sim: SimWorld,
    hero: HeroScreen,
    host: ScriptHost,
    toasts: ph2d_editor_core::ToastQueue,
    formas: Vec<Entity>,
    agora: u64,
}

impl Palco {
    fn novo() -> Self {
        let mut sim = SimWorld::new();
        let mut cena = VecScene::new();
        let mut mapa = VecEntityMap::new();
        let formas = monta_em(&mut sim, &mut cena, &mut mapa, 3);
        let mut hero = HeroScreen::new(NodeId(1));
        // O que a shell instala (`init.rs`): o assistente e o formatador fixados, por omissão.
        hero.documents
            .install_surface(Box::new(CodeDocs::default()));
        Self {
            sim,
            hero,
            host: ScriptHost::new().expect("vm"),
            toasts: ph2d_editor_core::ToastQueue::default(),
            formas,
            agora: 1_000_000_000_000,
        }
    }

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

    fn onda(&self) -> ScriptId {
        self.sim
            .world()
            .resource::<ProjectScripts>()
            .by_name(ONDA_NAME)
            .expect("onda.luau no projecto")
    }

    fn texto(&mut self) -> String {
        let id = self.onda();
        self.docs().text_of(id).expect("a aba abriu")
    }

    fn salvo(&self) -> String {
        let id = self.onda();
        self.sim
            .world()
            .resource::<ProjectScripts>()
            .get(id)
            .expect("script")
            .text
            .clone()
    }

    fn pose(&self, i: usize) -> Transform {
        *self
            .sim
            .world()
            .get::<Transform>(self.formas[i])
            .expect("pose")
    }

    /// A maior distância a `SHAPE_Y` de cada forma em `n` quadros.
    fn passeio(&mut self, n: usize) -> Vec<f32> {
        let mut max = vec![0.0f32; self.formas.len()];
        for _ in 0..n {
            self.quadro();
            for (i, m) in max.iter_mut().enumerate() {
                *m = m.max((self.pose(i).translation.y - SHAPE_Y).abs());
            }
        }
        max
    }

    fn proprio_da_estrela(&self) -> Vec<(String, ScriptValue)> {
        let s = self
            .sim
            .world()
            .get::<LuauScript>(self.formas[1])
            .expect("script na estrela");
        s.own.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// O que o Inspector mostra da estrela escolhida: `(nome, valor, próprio)`.
    fn inspector_da_estrela(&self) -> Vec<(String, Option<f64>, bool)> {
        let info = build_info(
            &self.sim,
            Some(&self.host),
            self.formas[1].to_bits(),
            1,
            true,
        )
        .expect("a estrela tem o cartão Script");
        info.props
            .into_iter()
            .map(|p| {
                let v = match p.value {
                    InspectorScriptValue::Number(v) => Some(v),
                    _ => None,
                };
                (p.name, v, p.own)
            })
            .collect()
    }
}

fn confere(p: &mut Palco, passo: usize) {
    let id = p.onda();
    match passo {
        1 => {
            let (titulos, _, previa) = p.docs().actions_of(id).expect("o Alt+Enter abriu");
            assert_eq!(titulos, ["Did you mean amplitude?"]);
            assert_eq!(
                previa,
                ["ph2d.set(self.id, \"y\", self.base_y + s * self.amplitude)"]
            );
        }
        2 => {
            let t = p.texto();
            assert!(t.contains("s * self.amplitude)"), "{t}");
            assert_eq!(p.salvo(), t, "o Ctrl+S levou a correcção à cena");
            let m = p.passeio(90);
            assert!(
                m[0] > 0.5 && m[0] <= 1.0 + 1e-3,
                "o círculo sobe ~1 m: {m:?}"
            );
            assert!(m[1] > 1.5, "a estrela sobe ~2 m (o valor dela): {m:?}");
            assert_eq!(m[3], 0.0, "o quadrado não se mexe");
        }
        3 => {
            let (lista, _) = p.docs().completion_of(id).expect("a lista abriu");
            let mut api: Vec<&str> = ph2d_script::api_table::API
                .iter()
                .filter(|e| e.table == "ph2d")
                .map(|e| e.ident)
                .collect();
            let mut vista: Vec<&str> = lista.iter().map(String::as_str).collect();
            api.sort_unstable();
            vista.sort_unstable();
            assert_eq!(vista, api, "a lista do `ph2d.` é a API inteira");
        }
        4 => {
            assert!(
                p.docs().snippet_of(id).is_some(),
                "os buracos estão activos"
            );
            let t = p.texto();
            assert!(t.contains("  ph2d.set(entity, \"x\", value)\n"), "{t}");
        }
        5 => {
            let t = p.texto();
            assert!(
                t.contains("  ph2d.set(self.id, \"rotation\", self.t)\nend\n"),
                "{t}"
            );
            assert_eq!(p.salvo(), t);
            let antes = p.pose(1).rotation;
            p.passeio(30);
            assert!((p.pose(1).rotation - antes).abs() > 0.3, "a estrela gira");
            assert_eq!(p.pose(3).rotation, 0.0, "o quadrado não");
        }
        6 => {
            let (campo, ajuda, razao) = p.docs().rename_of(id).expect("a caixinha abriu");
            assert_eq!(campo.as_deref(), Some("amplitude"));
            assert!(ajuda.contains("1 object"), "{ajuda}");
            assert_eq!(razao, None);
        }
        7 => {
            let t = p.texto();
            assert!(t.contains("ph2d.property(\"height\", 1.0"), "{t}");
            assert!(t.contains("s * self.height)"), "{t}");
            assert!(
                !t.contains("\"amplitude\"") && !t.contains("self.amplitude"),
                "{t}"
            );
            assert_eq!(p.salvo(), t);
            assert_eq!(
                p.proprio_da_estrela(),
                [("height".to_owned(), ScriptValue::Number(STAR_AMPLITUDE))],
                "o 2 da estrela foi com o nome"
            );
            p.quadro();
            let insp = p.inspector_da_estrela();
            assert!(
                insp.contains(&("height".into(), Some(STAR_AMPLITUDE), true)),
                "o Inspector mostra-o como próprio: {insp:?}"
            );
            assert!(!insp.iter().any(|(n, ..)| n == "amplitude"), "{insp:?}");
        }
        _ => unreachable!("passo {passo}"),
    }
}

#[test]
fn a_cena_3_conduzida_passo_a_passo() {
    assert_eq!(
        CENAS,
        ph2d_app_code::smoke::CENAS,
        "as cenas contam-se nos dois lados"
    );
    for g in STEPS.iter().flat_map(|s| s.iter()) {
        if let Gesture::Go { needle, .. } = g {
            assert_eq!(
                ONDA_LUAU.matches(needle).count(),
                1,
                "a agulha {needle:?} existe uma vez"
            );
        }
    }
    let mut p = Palco::novo();
    // Passo 0 — como o dono a recebe: a aba aberta, as formas PARADAS pelo erro, a estrela com 2.
    let m = p.passeio(30);
    assert!(m.iter().all(|d| *d == 0.0), "o erro pára as três: {m:?}");
    assert_eq!(p.texto(), ONDA_LUAU);
    assert!(
        p.inspector_da_estrela()
            .contains(&("amplitude".into(), Some(STAR_AMPLITUDE), true)),
        "{:?}",
        p.inspector_da_estrela()
    );

    let mut c = Conductor::new(STEPS.len());
    let mut feitos = 0;
    let prazo = Instant::now() + Duration::from_secs(120);
    while !c.finished() {
        assert!(
            Instant::now() < prazo,
            "o roteiro parou no passo {}",
            c.steps_done() + 1
        );
        p.quadro();
        drive(p.docs(), &mut c);
        while feitos < c.steps_done() {
            feitos += 1;
            confere(&mut p, feitos);
        }
    }
    if c.failed() {
        let id = p.onda();
        let lista = p.docs().completion_of(id);
        let cursor = p.docs().cursor_of(id);
        let assistente = p.docs().assist_reason();
        panic!(
            "o roteiro desistiu no passo {} — cursor {cursor:?}, lista {lista:?}, assistente \
             {assistente:?}, texto:\n{}",
            c.steps_done() + 1,
            p.texto()
        );
    }
    assert_eq!(feitos, STEPS.len());
}
