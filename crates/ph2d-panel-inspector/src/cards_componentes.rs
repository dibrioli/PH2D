//! ⭐ **Os COMPONENTES do objecto em cartões** (CartoesUI C4 — «um objeto com física, vida… perdia-os
//! no modo cartões»). Cada secção de componente que o Inspector de hoje pinta vira UM cartão: o
//! título é o do cabeçalho dela (absorvido — o cartão pinta-o), o corpo é o de hoje, sem mudar uma
//! linha dos ~45 pintores. Mesmos ids de secção ⇒ a dobra, a ordem, o tema e as notas são os de
//! sempre. Na mudança definitiva cada corpo passa ao vocabulário do cartão (campos Figma).

use crate::paint_frame::LiveSnapshots;
use crate::plano::{Plano, Tela};
use crate::reveal::SECTION_COMPONENT;
use crate::state::{InspectorState, MenuAdd};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, HeadAction};
use ph2d_editor_core::widget::{ButtonState, absorb_next_header, section_cards};
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;
use std::cell::Cell;

/// As linhas abertas das listas dos componentes (o estado do PAINEL que os pintores de hoje mexem).
#[derive(Clone, Copy, Default)]
pub(crate) struct Abertas {
    anchor: usize,
    anim: usize,
    anim_renomeando: Option<(u64, usize)>,
    timer: usize,
    action: usize,
    watch: usize,
    trigger: usize,
    emitter: usize,
    tween: usize,
    sm_state: usize,
    sm_trans: usize,
    resist: usize,
    constraint_add: MenuAdd,
}

impl Abertas {
    /// A animação aberta no cartão.
    pub(crate) fn anim(&self) -> usize {
        self.anim
    }

    /// O objecto e a linha cujo nome está a ser escrito.
    pub(crate) fn renomeando(&self) -> Option<(u64, usize)> {
        self.anim_renomeando
    }

    pub(crate) fn de(s: &InspectorState) -> Self {
        Self {
            anchor: s.anchor_selected,
            anim: s.anim_selected,
            anim_renomeando: s.anim_renomeando.as_ref().map(|r| (r.entity_bits, r.row)),
            timer: s.timer_selected,
            action: s.action_selected,
            watch: s.watch_selected,
            trigger: s.trigger_selected,
            emitter: s.emitter_selected,
            tween: s.tween_selected,
            sm_state: s.sm_state_selected,
            sm_trans: s.sm_trans_selected,
            resist: s.resist_selected,
            constraint_add: s.constraint_add,
        }
    }

    pub(crate) fn devolve(self, s: &mut InspectorState) {
        s.anchor_selected = self.anchor;
        s.anim_selected = self.anim;
        s.timer_selected = self.timer;
        s.action_selected = self.action;
        s.watch_selected = self.watch;
        s.trigger_selected = self.trigger;
        s.emitter_selected = self.emitter;
        s.tween_selected = self.tween;
        s.sm_state_selected = self.sm_state;
        s.sm_trans_selected = self.sm_trans;
        s.resist_selected = self.resist;
    }
}

/// As secções de corpo de hoje, pela ordem natural: as da sprite, as partilhadas sem cartão próprio
/// e os componentes (a paleta). O lugar na lista é o de `cards_objeto::tece`.
fn plano<'a>(
    store: &'a WidgetStore,
    x: f32,
    w: f32,
    a: &'a mut Abertas,
    snaps: &'a LiveSnapshots,
) -> Plano<'a> {
    let mut p = Plano::new();
    crate::paint_frame_shared::push_sprite_sections(
        &mut p,
        store,
        x,
        w,
        ROW_H_PX,
        snaps.sprite_info.as_ref(),
    );
    crate::paint_frame_shared::push_shared_sections(
        &mut p,
        store,
        x,
        w,
        ROW_H_PX,
        snaps.slice_info.as_ref(),
        None,
        snaps.sampling_info.as_ref(),
        None,
    );
    crate::paint_frame::push_physics_sections(
        &mut p,
        store,
        x,
        w,
        ROW_H_PX,
        snaps.physics_info.as_ref(),
        snaps.joint_info.as_ref(),
        snaps.wheel_info.as_ref(),
        snaps.player_info.as_ref(),
    );
    crate::paint_optional::push_optional_sections(
        &mut p,
        store,
        x,
        w,
        ROW_H_PX,
        &mut a.anim,
        &mut a.anchor,
        &mut a.timer,
        &mut a.action,
        &mut a.watch,
        &mut a.trigger,
        &mut a.emitter,
        &mut a.tween,
        &mut a.sm_state,
        &mut a.sm_trans,
        &mut a.resist,
        a.constraint_add,
        snaps,
    );
    p
}

/// Uma secção à vista: o id, o título do cabeçalho dela e a altura desse cabeçalho.
#[derive(Clone)]
pub(crate) struct Presente {
    pub id: NodeId,
    pub titulo: String,
    pub cab_h: f32,
    /// A largura do corpo e a que o título pede: o cartão Animation cede no cabeçalho estreito.
    pub largura: f32,
    pub titulo_w: f32,
    /// O resumo mais curto, quando o inteiro não cabe no título dobrado.
    pub resumo_curto: Option<String>,
}

/// Que secções de componente este objecto tem: pinta-as numa cena à parte (sem cliques) e guarda
/// as que desenharam alguma coisa, com o título do cabeçalho.
pub(crate) fn presentes(
    store: &WidgetStore,
    text: &mut TextSystem,
    theme: Theme,
    w: f32,
    abertas: Abertas,
    snaps: &LiveSnapshots,
    so: Option<NodeId>,
) -> Vec<Presente> {
    let mut a = abertas;
    let tarefas = plano(store, 0.0, w, &mut a, snaps).into_tarefas();
    let mut out = Vec::new();
    for (id, t) in tarefas {
        if so.is_some_and(|s| s != id) {
            continue; // T61: o cartão preso só precisa da secção dele
        }
        // O Animated Sprite vazio é só o convite do cartão Image (ficha 02): sem Animation, Look nem Advanced.
        use ph2d_editor_core::ids as c;
        let da_imagem = [
            c::INSP_LIVE_ANIM_SECTION,
            c::INSP_LIVE_COLOR_SECTION,
            c::INSP_LIVE_RENDER_SECTION,
        ];
        if da_imagem.contains(&id) && crate::cards_objeto::animada_vazia() {
            continue;
        }
        let mut cena = VectorScene::new();
        let mut hit = ph2d_editor_core::interaction::HitIndex::default();
        let (fim, cab) = absorb_next_header(|| {
            section_cards::with_section_cards(&mut cena, theme, 0.0, |s| {
                t(
                    &mut Tela {
                        scene: s,
                        text: &mut *text,
                        hit: &mut hit,
                    },
                    theme,
                    0.0,
                )
            })
        });
        if fim > 0.0 {
            let (titulo, cab_h) = cab.unwrap_or_default();
            let titulo_w = ph2d_editor_core::text_elide::title_elided_width(
                text,
                &titulo,
                ph2d_card_stack::metrics::CARD_TITLE_PX,
            );
            let resumo_curto =
                crate::sections::physics_colide::curto_se_a_fila_nao_cabe(id, text, w, titulo_w);
            out.push(Presente {
                id,
                titulo,
                cab_h,
                largura: w,
                titulo_w,
                resumo_curto,
            });
        }
    }
    out
}

/// T60: o id do ↺ no cabeçalho do cartão de componente `section`.
pub fn reset_of(section: NodeId) -> NodeId {
    const SAL: u64 = 0xC4E1_7B39_06AD_52F8;
    NodeId(section.0 ^ SAL)
}

/// O componente que o ↺ do cartão `section` repõe: o principal da secção (o Player, que nasce
/// pelo menu Add e não pelo «+», também — o registo constrói-o igual).
fn repoe(section: NodeId) -> Option<&'static str> {
    let (_, nome) = SECTION_COMPONENT.iter().find(|(s, _)| *s == section)?;
    ph2d_component_desc::desc_for(nome)?
        .may_have_section()
        .then_some(*nome)
}

/// Os ↺ dos cartões de componente: botões com a dica (uma vez por quadro, antes da pintura).
pub(crate) fn prepara(store: &mut WidgetStore) {
    for (s, _) in SECTION_COMPONENT {
        if repoe(s).is_none() {
            continue;
        }
        let id = reset_of(s);
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, ph2d_i18n::tr("card.insp.reset_component"));
        }
    }
}

/// ⭐ **T60 — o ↺ de TODO cartão de componente, uma lei**: pede à app que reponha o componente
/// como o «+» o anexa (o objecto preso sob a pressão, senão a seleção). A app atende-o
/// (`ph2d_app_components::component_attach::reset_asked`); o Desfazer é o diff do quadro.
pub(crate) fn reset_click(store: &mut WidgetStore, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(nome) = SECTION_COMPONENT
        .iter()
        .find(|(s, _)| reset_of(*s) == id)
        .and_then(|(s, _)| repoe(*s))
    else {
        return false;
    };
    // O objecto do retrato instalado: o preso sob a pressão (T61, `event::apply_event`) ou a seleção.
    if let Some(bits) = crate::state::current_inspector_transform().map(|t| t.entity_bits) {
        store.request_component_reset(bits, nome);
    }
    true
}

/// ⭐ O cartão de UMA secção de componente: o corpo de hoje, com o cabeçalho absorvido (o cabeçalho
/// fica por baixo do título do cartão).
pub(crate) fn cartao<'a>(
    p: &'a Presente,
    abertas: &'a Cell<Abertas>,
    snaps: &'a LiveSnapshots,
) -> CardDef<'a> {
    let id = p.id;
    let cab_h = p.cab_h;
    let actions: Vec<HeadAction> = repoe(id)
        .map(|_| HeadAction {
            id: reset_of(id),
            icon: IconId::Reset,
            on: false,
        })
        .into_iter()
        .collect();
    if let Some(proprio) = crate::cards_sprite::proprio(p, actions.clone()) {
        return proprio;
    }
    if let Some(proprio) = crate::cards_anim::proprio(p, actions.clone(), abertas) {
        return proprio;
    }
    CardDef {
        section: id,
        title: &p.titulo,
        actions,
        add: None,
        summary: p
            .resumo_curto
            .clone()
            .or_else(|| crate::sections::physics_cartao::resumo_do_cartao(id)),
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            ph2d_editor_core::panel::cards::compacta::fluir(c, |c| {
                let store = c.store;
                let theme = c.ink.theme;
                let mut a = abertas.get();
                let y0 = y - cab_h;
                let fim = {
                    let Some(t) = plano(store, c.x, c.w, &mut a, snaps)
                        .into_tarefas()
                        .into_iter()
                        .find_map(|(i, t)| (i == id).then_some(t))
                    else {
                        return y;
                    };
                    let (scene, text, hit) =
                        (&mut *c.scene, &mut *c.text_system, &mut *c.hit_index);
                    absorb_next_header(|| {
                        section_cards::with_section_cards(scene, theme, y0, |s| {
                            t(
                                &mut Tela {
                                    scene: s,
                                    text,
                                    hit,
                                },
                                theme,
                                y0,
                            )
                        })
                    })
                    .0
                };
                abertas.set(a);
                fim.max(y)
            })
        })),
    }
}
