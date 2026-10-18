//! Os testes da família do editor de scripts — tudo pelo caminho do produto: o pedido do cartão no
//! mundo, o `frame` da família, as teclas e o rato pela porta do hero (`board_keys`/`board_view`), a
//! cena pelo `ScriptHost` real.

mod a11y;
mod abrir;
mod acoes;
mod assistente;
mod bancada;
mod botoes;
mod busca;
mod completar;
mod console;
mod cores;
mod definicao;
mod despacho;
mod dicas;
mod dicas_de_tipo;
mod dobras;
mod esqueletos;
mod explicacoes;
mod gramatica;
mod linha_longa;
mod lista;
mod onda1;
mod oraculo_lsp;
mod paleta;
mod projecto;
mod renomear;
mod salvar;
mod teclado;

use ph2d_app_code::CodeDocs;
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_script::{LuauScript, ProjectScripts, ScriptEditRequests, ScriptHost, ScriptId};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

pub const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1000.0,
    h: 700.0,
};

pub struct Mesa {
    pub sim: SimWorld,
    pub hero: HeroScreen,
    pub host: Option<ScriptHost>,
    pub ts: TextSystem,
    /// Os avisos que a família deu (o ecrã os mostra).
    pub toasts: ph2d_editor_core::ToastQueue,
    /// O ficheiro do projecto (`None` = nunca gravado, como a cena do `PH2D_CODE_SMOKE`).
    pub ficheiro: Option<std::path::PathBuf>,
    /// Quantos diálogos «Save as» a shell teria aberto ([`Mesa::shell_drena`]).
    pub dialogos: usize,
}

impl Mesa {
    /// Um projeto com os scripts `(nome, texto)` e um hero com a família instalada.
    pub fn new(scripts: &[(&str, &str)]) -> (Self, Vec<ScriptId>) {
        let mut sim = SimWorld::new();
        let ids = {
            let mut lib = sim.world_mut().get_resource_or_init::<ProjectScripts>();
            scripts.iter().map(|(n, t)| lib.insert(n, *t)).collect()
        };
        let mut hero = HeroScreen::new(NodeId(1));
        // Sem assistente de código por omissão: o `luau-lsp` real arrancaria em cada teste. Os do
        // assistente ligam o seu ([`Mesa::com_assistente`]).
        let mut docs = CodeDocs::default();
        docs.set_intel_server(ph2d_app_code::IntelServer::Off);
        // O registo das explicações vistas fica na memória (nunca no ficheiro de quem corre o teste).
        docs.set_seen_file(None);
        hero.documents.install_surface(Box::new(docs));
        (
            Self {
                sim,
                hero,
                host: None,
                ts: TextSystem::without_system_fonts(),
                toasts: ph2d_editor_core::ToastQueue::default(),
                ficheiro: None,
                dialogos: 0,
            },
            ids,
        )
    }

    /// Com a VM, e `n` objectos a correr o script `id`.
    pub fn com_cena(mut self, id: ScriptId, n: u64) -> Self {
        let w = self.sim.world_mut();
        for sid in 1..=n {
            w.spawn((
                Transform::IDENTITY,
                StableId(sid + id.0 * 100),
                LuauScript::library(id),
            ));
        }
        if self.host.is_none() {
            self.host = Some(ScriptHost::new().expect("vm"));
        }
        self
    }

    /// Com este assistente de código (antes de abrir a 1.ª aba).
    pub fn com_assistente(mut self, server: ph2d_app_code::IntelServer) -> Self {
        self.docs().set_intel_server(server);
        self
    }

    /// Um quadro da família (e o hero segue o pedido de foco, como no início da pintura).
    pub fn quadro(&mut self) {
        ph2d_app_code::frame(
            &mut self.sim,
            self.host.as_mut(),
            &mut self.hero,
            &mut self.toasts,
            self.ficheiro.is_some(),
        );
        self.hero.documents.follow_surfaces();
    }

    /// O dreno do menu Ficheiro da shell (`drain_project_io` → `save_target`, gate
    /// `the_first_save_asks_and_the_next_ones_do_not`): «Save» com ficheiro grava em silêncio (aqui,
    /// o blob dos scripts no ficheiro); «Save» sem ficheiro ou «Save as» abre o diálogo (conta-se).
    pub fn shell_drena(&mut self) {
        let asked = std::mem::take(&mut self.hero.file_menu);
        if !(asked.save || asked.save_as) {
            return;
        }
        match self.ficheiro.as_ref().filter(|_| !asked.save_as) {
            Some(p) => {
                let bytes = self.sim.world().resource::<ProjectScripts>().to_bytes();
                std::fs::write(p, bytes).expect("grava");
            }
            None => self.dialogos += 1,
        }
    }

    /// Um quadro da cena (o `sync` recarrega; um tique corre os ganchos).
    pub fn cena(&mut self) {
        let h = self.host.as_mut().expect("vm");
        h.scene_sync(self.sim.world_mut());
        let _ = h.scene_tick(self.sim.world_mut(), 1.0 / 60.0);
    }

    /// O cartão pediu «Edit script» de `id`.
    pub fn pede(&mut self, id: ScriptId) {
        self.sim
            .world_mut()
            .get_resource_or_init::<ScriptEditRequests>()
            .0
            .push(id);
        self.quadro();
    }

    pub fn docs(&mut self) -> &mut CodeDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<CodeDocs>()
            .expect("família instalada")
    }

    pub fn texto_salvo(&self, id: ScriptId) -> String {
        self.sim
            .world()
            .resource::<ProjectScripts>()
            .get(id)
            .expect("script")
            .text
            .clone()
    }

    /// Uma tecla premida na aba `id`, direto à família.
    pub fn tecla(&mut self, id: ScriptId, key: Key, mods: Modifiers, text: Option<&str>) {
        let mut ts = TextSystem::without_system_fonts();
        let input = ph2d_editor_core::doc_surface::KeyInput {
            key,
            mods,
            pressed: true,
            text,
            paste: None,
        };
        let _ = self.docs().key(id.0, &mut ts, input);
    }

    pub fn escreve(&mut self, id: ScriptId, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.tecla(id, Key::Enter, Modifiers::default(), Some("\r"));
                continue;
            }
            let t = c.to_string();
            self.tecla(id, Key::Other, Modifiers::default(), Some(&t));
        }
    }

    /// Pinta a aba `id` (a vista ganha área e linhas).
    pub fn pinta(&mut self, id: ScriptId) {
        let mut scene = VectorScene::new();
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        self.docs()
            .paint(id.0, AREA, &mut scene, &mut ts, &Theme::default());
        self.ts = ts;
    }

    /// O rato sem botão em `(x, y)` da aba `id`.
    pub fn rato(&mut self, id: ScriptId, x: f32, y: f32) {
        use ph2d_editor_core::doc_surface::PointerInput;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let input = PointerInput {
            text: &mut ts,
            mods: Modifiers::default(),
            now_ns: 1,
        };
        let _ = self.docs().pointer_move(id.0, input, x, y);
        self.ts = ts;
    }

    /// Um clique real (carregar + largar) na aba `id`.
    pub fn clica(&mut self, id: ScriptId, x: f32, y: f32, mods: Modifiers) {
        use ph2d_editor_core::doc_surface::PointerInput;
        use ph2d_host::{PointerButton, PointerKind};
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        for kind in [PointerKind::Down, PointerKind::Up] {
            let input = PointerInput {
                text: &mut ts,
                mods,
                now_ns: 1,
            };
            assert!(
                self.docs()
                    .pointer(id.0, input, kind, PointerButton::Primary, x, y),
                "o clique em ({x}, {y}) não foi tomado"
            );
        }
        self.ts = ts;
    }
}

pub fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}

pub fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}
