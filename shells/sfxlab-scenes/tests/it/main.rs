//! O hero montado com a aba do som e o painel: evento real nos rects pintados, efeito lido no documento e no PCM.

mod cartoes;
mod explorar;
mod faixas;
mod jogo;
mod medidores;
mod medir;
mod vaga;

use bevy_ecs::world::World;
use ph2d_app_sfxlab::{Alvo, CMD_NEW, Escuta, SfxLabDocs, cmd_abrir};
use ph2d_audio::SampleData;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::{
    InteractiveState, WidgetEvent, dispatch_key, dispatch_pointer, dispatch_text_input,
};
use ph2d_editor_core::panel::{ErasedPanel, PanelRegistry, install_panel_registry};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_panel_sfxlab::SfxLabPanel;
use ph2d_panel_sfxlab::tabela::Campo;
use ph2d_sfx_recipe::{Layer, Recipe, SfxLibrary};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::time::{Duration, Instant};

pub const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1400.0,
};
const TECLA_ESC: u32 = 0x1B;
const TECLA_ENTER: u32 = 0x0D;
const QUADROS_ATE_ASSENTAR: usize = 40;

#[derive(Default)]
pub struct EscutaFalsa {
    pub toques: usize,
}

impl Escuta for EscutaFalsa {
    fn toca(&mut self, _som: SampleData) {
        self.toques += 1;
    }

    fn para(&mut self) {}
}

pub struct Mesa {
    pub hero: HeroScreen,
    pub ts: TextSystem,
    pub mundo: World,
    pub escuta: EscutaFalsa,
    agora: u128,
}

impl Mesa {
    pub fn nova() -> Self {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<ph2d_panel_inspector::InspectorPanel>());
        reg.push(ErasedPanel::new::<SfxLabPanel>());
        let _ = install_panel_registry(reg);
        let mut hero = HeroScreen::new(NodeId(1));
        ph2d_app_sfxlab::install(&mut hero);
        ph2d_panel_sfxlab::publica(None);
        let _ = ph2d_panel_sfxlab::tomar();
        ph2d_panel_hierarchy::arma_escolha(false);
        Self {
            hero,
            ts: TextSystem::without_system_fonts(),
            mundo: World::new(),
            escuta: EscutaFalsa::default(),
            agora: 1_000_000_000_000,
        }
    }

    /// Um quadro como o da shell: o serviço, e o hero inteiro pintado.
    pub fn quadro(&mut self) {
        self.agora += 16_666_667;
        self.hero.store.note_clock(self.agora);
        ph2d_app_sfxlab::service(&mut self.hero, &mut self.mundo, Some(&mut self.escuta));
        ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(&mut self.hero);
        let mut scene = VectorScene::new();
        let (hero, ts) = (&mut self.hero, &mut self.ts);
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || ph2d_editor_core::screens::hero::paint_hero_screen(hero, JANELA, &mut scene, ts),
        );
    }

    pub fn assenta(&mut self) {
        for _ in 0..QUADROS_ATE_ASSENTAR {
            self.quadro();
        }
    }

    pub fn docs(&mut self) -> &mut SfxLabDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<SfxLabDocs>()
            .expect("família instalada")
    }

    pub fn lib_mut(&mut self) -> bevy_ecs::world::Mut<'_, SfxLibrary> {
        if !self.mundo.contains_resource::<SfxLibrary>() {
            self.mundo.insert_resource(SfxLibrary::default());
        }
        self.mundo.resource_mut::<SfxLibrary>()
    }

    /// Abre `r` por «Open Sound ▸ <nome>». O id da aba, com o render pronto e os cartões assentados.
    pub fn abre(&mut self, nome: &str, r: Recipe) -> u64 {
        self.lib_mut().recipes.insert(nome.to_owned(), r);
        self.quadro();
        assert!(self.hero.apply_event(WidgetEvent::Click(cmd_abrir(nome))));
        self.quadro();
        self.quadro();
        let id = self.activa();
        self.espera(id);
        self.assenta();
        id
    }

    pub fn som_novo(&mut self) -> u64 {
        assert!(self.hero.apply_event(WidgetEvent::Click(CMD_NEW)));
        self.quadro();
        self.quadro();
        self.activa()
    }

    pub fn activa(&mut self) -> u64 {
        self.hero.documents.surface_active().expect("aba activa").id
    }

    pub fn espera(&mut self, id: u64) {
        let fim = Instant::now() + Duration::from_secs(120);
        loop {
            self.quadro();
            if !self.docs().em_voo(id) {
                break;
            }
            assert!(Instant::now() < fim, "o render não chegou em 120 s");
            std::thread::sleep(Duration::from_millis(1));
        }
        self.quadro();
    }

    pub fn receita(&mut self, id: u64) -> Recipe {
        self.docs().receita(id).expect("aba").clone()
    }

    pub fn camada(&mut self, id: u64) -> Option<Layer> {
        let r = self.receita(id);
        let sel = self.docs().selecionada(id)?;
        r.layer(sel).cloned()
    }

    pub fn hash(&mut self, id: u64) -> u64 {
        ph2d_sfx_dsp::pcm_hash(self.docs().pcm(id).expect("render"))
    }

    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        self.hero.hit_index.rect_for(id)
    }

    pub fn mao(&mut self, kind: PointerKind, x: f32, y: f32) {
        let arena = bumpalo::Bump::new();
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs: Vec<_> =
            dispatch_pointer(&mut self.hero.store, &self.hero.hit_index, ev, &arena).to_vec();
        for e in evs {
            self.hero.apply_event(e);
        }
    }

    /// Premir na pega e arrastar `dx`, sem largar. O limiar do scrub pede dois `Move`.
    pub fn agarra(&mut self, chip: NodeId, dx: f32) {
        let r = self
            .rect(chip)
            .unwrap_or_else(|| panic!("{chip:?} não foi pintado"));
        let p = handle_rect(r);
        let (x, y) = (p.x + p.w * 0.5, p.y + p.h * 0.5);
        self.mao(PointerKind::Down, x, y);
        self.quadro();
        self.mao(PointerKind::Move, x + dx * 0.5, y);
        self.quadro();
        self.mao(PointerKind::Move, x + dx, y);
        self.quadro();
    }

    pub fn solta(&mut self, chip: NodeId, dx: f32) {
        let r = self
            .rect(chip)
            .unwrap_or_else(|| panic!("{chip:?} não foi pintado"));
        let p = handle_rect(r);
        self.mao(PointerKind::Up, p.x + p.w * 0.5 + dx, p.y + p.h * 0.5);
        self.quadro();
    }

    pub fn clica(&mut self, id: NodeId) {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não foi pintado"));
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.mao(PointerKind::Down, x, y);
        self.mao(PointerKind::Up, x, y);
        self.quadro();
    }

    /// O clique na aba, pela porta da shell: é da aba só se nenhum painel está por cima.
    pub fn clica_na_aba(&mut self, id: u64, alvo: Alvo) {
        let r = self
            .docs()
            .rect_de(id, alvo)
            .unwrap_or_else(|| panic!("{alvo:?} não foi pintado"));
        self.ponteiro_da_shell(PointerKind::Down, r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.ponteiro_da_shell(PointerKind::Up, r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.quadro();
    }

    /// `true` = a aba tomou o evento.
    pub fn ponteiro_da_shell(&mut self, kind: PointerKind, x: f32, y: f32) -> bool {
        let no_canvas =
            self.hero.chrome_panel_at(x, y).is_none() && self.hero.chrome_hit(x, y).is_none();
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        board_view::pointer(
            &mut self.hero,
            input,
            kind,
            PointerButton::Primary,
            x,
            y,
            no_canvas,
        )
    }

    fn tecla(&mut self, keycode: u32) {
        let arena = bumpalo::Bump::new();
        let k = KeyEvent {
            keycode,
            modifiers: Modifiers::default(),
            kind: KeyKind::Down,
            timestamp_ns: 0,
        };
        let evs: Vec<_> = dispatch_key(&mut self.hero.store, k, &arena).to_vec();
        for e in evs {
            self.hero.apply_event(e);
        }
    }

    pub fn esc(&mut self) {
        self.tecla(TECLA_ESC);
        self.quadro();
    }

    /// Escrever o número e Enter, como quem digita.
    pub fn digita(&mut self, chip: NodeId, texto: &str) {
        match self.hero.store.get_mut(chip) {
            Some(InteractiveState::NumberInput { buffer, caret, .. }) => {
                buffer.clear();
                *caret = 0;
            }
            _ => panic!("{chip:?} não é um número registrado"),
        }
        self.hero.store.set_focus(Some(chip));
        let arena = bumpalo::Bump::new();
        for ch in texto.chars() {
            let _ = dispatch_text_input(&mut self.hero.store, ch, &arena);
        }
        self.tecla(TECLA_ENTER);
        self.quadro();
    }

    pub fn desfaz(&mut self, refazer: bool) -> Option<bool> {
        let r = ph2d_editor_core::screens::hero::surface_input::undo(&mut self.hero, refazer);
        self.quadro();
        r
    }
}

/// Todos os valores que a tabela lê neste estado, para ver quem mudou.
pub fn leitura(r: &Recipe, l: Option<&Layer>, campos: &[Campo]) -> Vec<Option<f64>> {
    campos.iter().map(|c| c.le(r, l)).collect()
}
