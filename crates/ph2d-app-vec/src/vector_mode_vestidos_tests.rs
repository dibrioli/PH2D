//! As cenas 27/28 de ponta a ponta, sem a shell: a 25 inteira (`end` = [`super::FIRST`]) e depois a
//! fila vestida até `9`, pelos MESMOS estágios do smoke.
//!
//! ⚠️ Os cliques do `step` vão por `HeroScreen::apply_event`, que neste binário não chega ao painel
//! (o registo é do processo — ver `vector_mode_interface_tests`). Por isso o harness faz o papel da
//! shell a cada estágio que avançou: sabe que clique aquele estágio deu e encaminha-o pelas MESMAS
//! portas do roteador dela (`widget_edit_for_id` → `widget_edit::apply`, `ui_state_edit_for_id` →
//! `ui_state_edit::apply`), e publica o que a shell publica ao painel. Duas coisas são da shell e
//! entram aqui como o seu RESULTADO: o *Create* (a cópia no lugar, a seleção nela, a instância
//! publicada) e o fim da transição do *Show* (a pose do papel instalada).

use super::{FIRST, LAST, fila, so_um, veste};
use crate::vector_mode_interface as interface;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily, ToolBars};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::WidgetKind;
use ph2d_panel_vector::{ids, state};
use ph2d_vec_scene::{Paint, Rgba8, VecScene};

struct Cena {
    vec: crate::state::VecState,
    scene: VecScene,
    hero: HeroScreen,
    sim: ph2d_ecs::SimWorld,
    states: ph2d_ui_state::StateSets,
}

impl Cena {
    fn new() -> Self {
        state::set_component_state(None);
        state::set_ui_states_state(None);
        state::set_widget_skin_state(None, 0);
        Self {
            vec: crate::state::VecState::default(),
            scene: VecScene::new(),
            hero: HeroScreen::new(NodeId(1)),
            sim: ph2d_ecs::SimWorld::default(),
            states: ph2d_ui_state::StateSets::default(),
        }
    }

    /// Um quadro: o estágio, a resposta da shell ao clique dele, a porta que torna as formas vivas
    /// e o que a shell publica ao painel.
    fn passo(&mut self, s: u8, hover: bool) -> Option<u8> {
        let next = if (interface::FIRST..=interface::LAST).contains(&s) {
            interface::step(
                s,
                &mut self.vec,
                &mut self.scene,
                &self.sim,
                &mut self.hero,
                false,
                FIRST,
            )
        } else {
            super::step(
                s,
                &mut self.vec,
                &mut self.scene,
                &self.sim,
                &mut self.hero,
                hover,
            )
        };
        if next.is_some() {
            self.responde(s, hover);
        }
        ph2d_vec_entities::entities::sync(&mut self.sim, &mut self.scene, &mut self.vec.entities);
        crate::shape_live::make_committed_shape_live(
            &mut self.sim,
            &mut self.scene,
            &self.vec.entities,
            &mut self.vec.shape,
            false,
        );
        let sel = self.vec.pen.selected_paths().to_vec();
        let skin = crate::widget_edit::publish(&self.sim, &self.vec.entities, &sel);
        let beyond = skin.as_ref().map_or(0, |(_, b)| *b);
        state::set_widget_skin_state(skin.map(|(s, _)| s), beyond);
        state::set_ui_states_state(crate::ui_state_edit::publish(
            &self.sim,
            &self.scene,
            &self.vec.entities,
            &sel,
            &self.states,
            None,
            false,
            false,
        ));
        next
    }

    /// O que a shell faz com o clique que o estágio `s` acabou de dar.
    fn responde(&mut self, s: u8, hover: bool) {
        match s {
            // A troca de ferramenta: a barra publicada com o *Select* aceso.
            interface::CORES => self.hero.store.set_tool_bars(Some(ToolBars {
                column: vec![vec![BarButton {
                    id: ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
                    face: BarFace::Icon(ph2d_editor_core::icons::IconId::Select),
                    label: String::new(),
                    on: true,
                    family: BarFamily::Neutral,
                }]],
                ..ToolBars::default()
            })),
            // O *Create*: uma cópia no lugar, a seleção segue para ela, e ela é instância.
            interface::CREATE => {
                let botao = so_um(&self.vec).expect("o Create anda com o botão na mão");
                let copia = self.scene.path(botao).expect("o botão existe").clone();
                let id = self.scene.push_path(copia);
                self.vec.pen.select_many(&[id]);
                state::set_component_state(Some(state::ComponentState {
                    is_instance: true,
                    ..state::ComponentState::default()
                }));
            }
            interface::WEAR => self.clica(ids::VECTOR_WIDGET_WEAR),
            interface::GRAVA_DEFAULT => self.clica(ids::vector_state_record_id(0)),
            interface::GRAVA_HOVER => self.clica(ids::vector_state_record_id(1)),
            interface::LAST => self.clica(ids::vector_state_apply_id(0)),
            s if (super::VESTE..LAST).contains(&s) => {
                let kind = super::FILA[usize::from(s - super::VESTE)].2;
                let chip = WidgetKind::ALL.iter().position(|&k| k == kind).unwrap();
                self.clica(ids::vector_widget_kind_id(chip));
            }
            LAST if hover => self.clica(ids::vector_state_apply_id(super::PAPEL_HOVER)),
            _ => {}
        }
    }

    /// O roteador da shell para os dois assuntos destas cenas: a pele e os estados.
    fn clica(&mut self, id: NodeId) {
        let sel = self.vec.pen.selected_paths().to_vec();
        if let Some(verb) = crate::widget_edit::widget_edit_for_id(id) {
            crate::widget_edit::apply(&mut self.sim, &self.vec.entities, &sel, verb);
        } else if let Some(verb) = crate::ui_state_edit::ui_state_edit_for_id(id) {
            let pedido = crate::ui_state_edit::apply(
                &mut self.sim,
                &mut self.scene,
                &self.vec.entities,
                &sel,
                &mut self.states,
                verb,
            );
            // O fim da transição: a pose do papel instalada.
            if let Some((host, role)) = pedido {
                let poses = self
                    .states
                    .role(host, role)
                    .expect("o papel gravado")
                    .clone();
                for p in &poses.objects {
                    crate::ui_state_edit::install(
                        &mut self.sim,
                        &mut self.scene,
                        &self.vec.entities,
                        p,
                    );
                }
            }
        } else {
            panic!("clique sem rota no harness: {id:?}");
        }
    }
}

fn solido(c: [u8; 3]) -> Option<Paint> {
    Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)))
}

/// Corre as duas cenas até `9`; devolve-as no fim.
fn corre(hover: bool) -> Cena {
    let mut c = Cena::new();
    let mut s = interface::FIRST;
    for _ in 0..200 {
        if s == 9 {
            return c;
        }
        if let Some(n) = c.passo(s, hover) {
            s = n;
        }
    }
    panic!("parou no estágio {s} (hover = {hover})");
}

/// ⭐ **A fila vestida e o botão**: sete formas; as quatro da fila vestem Toggle, Slider, Checkbox e
/// TextInput, cada uma na cor do artista; o botão (a cópia vestida da 25) é a seleção, azul em
/// repouso na 27 e laranja na 28.
#[test]
fn a_fila_veste_e_o_botao_mostra_o_hover_so_na_28() {
    for hover in [false, true] {
        let c = corre(hover);
        assert_eq!(c.scene.paths().len(), 7, "hover = {hover}");
        assert_eq!(c.vec.entities.len(), 7, "hover = {hover}");
        let formas = fila(&c.vec);
        assert_eq!(formas.len(), 4);
        let esperado = [
            (WidgetKind::Toggle, [52, 168, 83]),
            (WidgetKind::Slider, [124, 77, 255]),
            (WidgetKind::Checkbox, [236, 236, 228]),
            (WidgetKind::TextInput, [32, 36, 44]),
        ];
        for (&id, (kind, cor)) in formas.iter().zip(esperado) {
            assert_eq!(veste(&c.vec, &c.sim, id), Some(kind), "hover = {hover}");
            let e = ph2d_ecs::Entity::from_bits(c.vec.entities[&id]);
            assert!(
                c.sim.world().get::<ph2d_ecs::VecShape>(e).is_some(),
                "{kind:?} ficou caminho morto, não forma viva"
            );
            assert_eq!(c.scene.path(id).unwrap().fill, solido(cor), "{kind:?}");
        }
        let botao = so_um(&c.vec).expect("uma seleção só");
        assert_eq!(veste(&c.vec, &c.sim, botao), Some(WidgetKind::Button));
        let cor = if hover {
            [240, 140, 40]
        } else {
            [40, 120, 240]
        };
        assert_eq!(
            c.scene.path(botao).unwrap().fill,
            solido(cor),
            "hover = {hover}"
        );
        state::set_component_state(None);
        state::set_ui_states_state(None);
    }
}
