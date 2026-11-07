//! A COSTURA hero → aba do SFX Lab, sem placa: o menu pelo `apply_event`, o rato pelo `board_view`, o teclado pelo
//! `board_keys`, o desfazer pelo `surface_input::undo`. Cada quadro corre como a shell: `service`, `follow_surfaces`, pintar.

mod explorar;
mod gestos;
mod render;

use bevy_ecs::world::World;
use ph2d_app_sfxlab::{Alvo, CMD_NEW_SOUND, Escuta, SfxLabDocs, cmd_abrir};
use ph2d_audio::SampleData;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_sfx_recipe::{LayerId, Recipe, SfxLibrary};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::time::{Duration, Instant};

pub const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1400.0,
    h: 800.0,
};

#[derive(Default)]
pub struct EscutaFalsa {
    pub toques: Vec<Vec<f32>>,
    pub paradas: u32,
}

impl Escuta for EscutaFalsa {
    fn toca(&mut self, som: SampleData) {
        self.toques.push(som.samples().to_vec());
    }

    fn para(&mut self) {
        self.paradas += 1;
    }
}

pub struct Mesa {
    pub hero: HeroScreen,
    pub ts: TextSystem,
    pub mundo: World,
    pub escuta: EscutaFalsa,
    pub agora: u128,
}

impl Default for Mesa {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesa {
    pub fn new() -> Self {
        let mut hero = HeroScreen::new(NodeId(1));
        ph2d_app_sfxlab::install(&mut hero);
        Self {
            hero,
            ts: TextSystem::without_system_fonts(),
            mundo: World::new(),
            escuta: EscutaFalsa::default(),
            agora: 1_000_000_000_000,
        }
    }

    pub fn quadro(&mut self) {
        self.agora += 16_666_667;
        self.hero.store.note_clock(self.agora);
        ph2d_app_sfxlab::service(&mut self.hero, &mut self.mundo, Some(&mut self.escuta));
        ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(&mut self.hero);
        let mut scene = VectorScene::new();
        let theme = self.hero.theme;
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            f.paint(id, AREA, &mut scene, &mut self.ts, &theme);
        }
    }

    pub fn docs(&mut self) -> &mut SfxLabDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<SfxLabDocs>()
            .expect("família instalada")
    }

    pub fn lib(&self) -> &SfxLibrary {
        self.mundo.resource::<SfxLibrary>()
    }

    pub fn poe(&mut self, nome: &str, r: Recipe) {
        if !self.mundo.contains_resource::<SfxLibrary>() {
            self.mundo.insert_resource(SfxLibrary::default());
        }
        self.mundo
            .resource_mut::<SfxLibrary>()
            .recipes
            .insert(nome.to_owned(), r);
    }

    /// «New Sound», pela porta do menu File. O id da aba activa.
    pub fn som_novo(&mut self) -> u64 {
        assert!(self.hero.apply_event(WidgetEvent::Click(CMD_NEW_SOUND)));
        self.quadro();
        self.quadro();
        self.activa()
    }

    /// «Open Sound ▸ <nome>», com o som já no mundo.
    pub fn abre(&mut self, nome: &str) -> u64 {
        self.quadro();
        assert!(self.hero.apply_event(WidgetEvent::Click(cmd_abrir(nome))));
        self.quadro();
        self.quadro();
        self.activa()
    }

    pub fn activa(&mut self) -> u64 {
        self.hero.documents.surface_active().expect("aba activa").id
    }

    /// Quadros até o render mais novo chegar.
    pub fn espera(&mut self, id: u64) {
        let fim = Instant::now() + Duration::from_secs(120);
        while self.docs().em_voo(id) {
            assert!(Instant::now() < fim, "o render não chegou em 120 s");
            std::thread::sleep(Duration::from_millis(2));
            self.quadro();
        }
        self.quadro();
    }

    pub fn centro(&mut self, id: u64, alvo: Alvo) -> (f32, f32) {
        let r = self
            .docs()
            .rect_de(id, alvo)
            .unwrap_or_else(|| panic!("{alvo:?} não foi pintado"));
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    fn rato(&mut self, kind: PointerKind, x: f32, y: f32) -> bool {
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
            true,
        )
    }

    pub fn carrega(&mut self, (x, y): (f32, f32)) -> bool {
        self.rato(PointerKind::Down, x, y)
    }

    pub fn larga(&mut self, (x, y): (f32, f32)) -> bool {
        self.rato(PointerKind::Up, x, y)
    }

    pub fn move_(&mut self, (x, y): (f32, f32)) -> bool {
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        board_view::pointer_move(&mut self.hero, input, x, y)
    }

    pub fn clica(&mut self, id: u64, alvo: Alvo) {
        let p = self.centro(id, alvo);
        assert!(self.carrega(p), "o carregar em {alvo:?} não foi tomado");
        assert!(self.larga(p), "o largar em {alvo:?} não foi tomado");
        self.quadro();
    }

    /// Carrega, anda em passos com um quadro entre cada um, e larga.
    pub fn arrasta(&mut self, de: (f32, f32), para: (f32, f32), passos: u16) {
        assert!(self.carrega(de), "o carregar não foi tomado");
        for i in 1..=passos {
            let t = f32::from(i) / f32::from(passos);
            self.move_((de.0 + (para.0 - de.0) * t, de.1 + (para.1 - de.1) * t));
            self.quadro();
        }
        assert!(self.larga(para), "o largar não foi tomado");
        self.quadro();
    }

    pub fn tecla(&mut self, k: BoardKey) -> bool {
        let m = Modifiers::default();
        let out = board_keys::key(&mut self.hero, &mut self.ts, k, m, true, None, None);
        let _ = board_keys::key(&mut self.hero, &mut self.ts, k, m, false, None, None);
        self.quadro();
        out.consumed
    }

    pub fn desfaz(&mut self, refazer: bool) -> Option<bool> {
        let r = ph2d_editor_core::screens::hero::surface_input::undo(&mut self.hero, refazer);
        self.quadro();
        r
    }

    pub fn receita(&mut self, id: u64) -> Recipe {
        self.docs().receita(id).expect("aba").clone()
    }

    pub fn camada(&mut self, id: u64, i: usize) -> LayerId {
        self.receita(id).layers[i].id
    }

    /// O som da aba é o que a receita `r` dá no motor.
    pub fn soa_como(&mut self, id: u64, r: &Recipe) {
        let certo = verdade(r, self.lib());
        assert_eq!(self.docs().pcm(id).expect("render"), &*certo);
    }

    pub fn hash(&mut self, id: u64) -> u64 {
        ph2d_sfx_dsp::pcm_hash(self.docs().pcm(id).expect("render"))
    }
}

/// O som de arquivo que a receita tem de dar.
pub fn verdade(r: &Recipe, lib: &SfxLibrary) -> Vec<f32> {
    ph2d_sfx_dsp::render(r, r.seed, r.engine, &lib.takes)
        .expect("render")
        .samples
}

pub fn pico(s: &[f32]) -> f32 {
    s.iter().fold(0.0, |a, b| a.max(b.abs()))
}
