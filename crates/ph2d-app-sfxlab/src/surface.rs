//! O `DocumentSurface` da família.

use std::any::Any;

use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::{
    Compose, DocCommand, DocMenu, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput,
    PointerLook,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

use crate::docs::SfxLabDocs;

pub const CMD_NEW_SOUND: NodeId = hash_node_id("sfxlab_cmd_new");

/// «Open Sound ▸ <nome>»: o id é do nome.
#[must_use]
pub fn cmd_abrir(nome: &str) -> NodeId {
    NodeId(ph2d_sfx_lab::novo::semente_do_nome(nome) ^ crate::A11Y_SALT)
}

impl DocumentSurface for SfxLabDocs {
    fn tabs(&self) -> Vec<DocTab> {
        self.abas
            .iter()
            .map(|a| DocTab {
                id: a.id,
                title: a.nome.clone(),
                dirty: false,
            })
            .collect()
    }

    fn paint(
        &mut self,
        id: u64,
        area: Rect,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: &Theme,
    ) {
        self.area = Some(area);
        let over = self.over;
        if let Some(a) = self.aba_mut(id) {
            crate::paint::aba(a, area, over, scene, text, *theme);
        }
    }

    fn chrome_over(&mut self, _id: u64, over: Option<Rect>) {
        self.over = over;
    }

    fn pointer(
        &mut self,
        id: u64,
        input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        if button != PointerButton::Primary {
            return false;
        }
        self.aba_mut(id).is_some_and(|a| {
            a.agora_ns = input.now_ns;
            crate::gestos::ponteiro(a, kind, x, y)
        })
    }

    fn pointer_move(&mut self, id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.aba_mut(id)
            .is_some_and(|a| crate::gestos::ponteiro(a, PointerKind::Move, x, y))
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, _shift: bool) -> bool {
        self.aba_mut(id)
            .is_some_and(|a| crate::gestos::roda(a, x, y, dy))
    }

    fn key(&mut self, id: u64, _ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        self.aba_mut(id)
            .map(|a| crate::gestos::tecla(a, &input))
            .unwrap_or_default()
    }

    fn compose(&mut self, id: u64, c: Compose<'_>) -> bool {
        self.aba_mut(id)
            .and_then(|a| a.janela.as_mut())
            .is_some_and(|j| j.compose(&c))
    }

    fn close(&mut self, id: u64) {
        self.fecha(id);
    }

    fn undo(&mut self, id: u64, redo: bool) -> bool {
        // Com a janela aberta, o som por baixo não muda escondido.
        self.aba_mut(id)
            .is_some_and(|a| a.janela.is_some() || a.desfaz(redo))
    }

    fn a11y(&self, id: u64) -> Vec<(NodeId, ph2d_a11y::Node)> {
        let Some(a) = self.area else {
            return Vec::new();
        };
        let node = ph2d_a11y::NodeBuilder::new(ph2d_a11y::Role::Document)
            .label(self.nome(id).unwrap_or_default().to_owned())
            .bounds(
                f64::from(a.x),
                f64::from(a.y),
                f64::from(a.w),
                f64::from(a.h),
            )
            .build();
        vec![(NodeId(crate::A11Y_SALT ^ id), node)]
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        self.focus.take()
    }

    fn family_commands(&self) -> Vec<DocCommand> {
        let mut out = vec![DocCommand {
            id: CMD_NEW_SOUND,
            label: tr("sfxlab.cmd.new").to_owned(),
            menu: DocMenu::File,
        }];
        out.extend(
            self.nomes
                .iter()
                .filter(|n| !self.abas.iter().any(|a| a.nome == **n))
                .map(|n| DocCommand {
                    id: cmd_abrir(n),
                    label: tr_with("sfxlab.cmd.open", &[("name", n)]),
                    menu: DocMenu::File,
                }),
        );
        out
    }

    fn family_command(&mut self, cmd: NodeId) -> bool {
        if cmd == CMD_NEW_SOUND {
            self.novo();
            return true;
        }
        let nome = self.nomes.iter().find(|n| cmd_abrir(n) == cmd).cloned();
        if let Some(n) = nome {
            self.pede_abrir(n);
            return true;
        }
        false
    }

    fn pointer_look(&self, id: u64, x: f32, y: f32) -> Option<PointerLook> {
        let a = self.abas.iter().find(|a| a.id == id)?;
        if a.conta_gotas() {
            return Some(PointerLook::Pick);
        }
        a.corpo
            .contains(x, y)
            .then(|| crate::gestos::aparencia(a, x, y))
    }

    fn wants_frame(&self) -> bool {
        self.abas.iter().any(|a| {
            a.em_voo()
                || a.quer_tocar
                || a.quer_usar
                || a.quer_escolher
                || a.conta_gotas()
                || a.ex.em_voo()
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
