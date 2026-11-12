//! ⭐ **O Platform Player em cartões** — a versão B da ficha
//! `docs/GameEngineFuncional/componentes/03_character.md` (dono 10/11): Walk e Jump abertos, cada
//! habilidade uma dobra com a CAIXA ao lado do nome, e o resto dobrado com o resumo à direita.
//! Nenhuma row some: todas vêm da tabela [`PLAYER_CARDS`] (a mesma do `populate` e da varredura).

#[path = "player_grupos.rs"]
mod grupos;
#[path = "player_pulo.rs"]
mod pulo;

use super::rows::{Campo, caixa_row, card_frame_with, num_row_unit, par_row, seg_row};
use super::table::{PLAYER_CARDS, PlayerRow};
use super::*;
use ph2d_editor_core::screens::hero::InspectorPlayerInfo;
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_i18n::{TextKey, tr};

use grupos::*;
use pulo::cartao;

fn row_of(id: NodeId) -> Option<&'static PlayerRow> {
    PLAYER_CARDS
        .iter()
        .flat_map(|(_, _, rows)| rows.iter())
        .find(|r| r.1 == id)
}

/// O pincel do painel: o que todas as partes partilham.
pub(super) struct Pintor<'a> {
    pub scene: &'a mut VectorScene,
    pub text: &'a mut TextSystem,
    pub theme: Theme,
    pub hit: &'a mut HitIndex,
    pub store: &'a WidgetStore,
    pub x: f32,
    pub w: f32,
    pub info: &'a InspectorPlayerInfo,
}

impl Pintor<'_> {
    fn shown(&self, id: NodeId) -> bool {
        const SPRING_ONLY: [NodeId; 3] = [
            ids::INSP_PLAYER_FLOAT,
            ids::INSP_PLAYER_STIFFNESS,
            ids::INSP_PLAYER_DAMPING,
        ];
        (self.info.push_is_live || id != ids::INSP_PLAYER_REACT_PUSH)
            && (self.info.spring_is_live || !SPRING_ONLY.contains(&id))
    }

    /// As rows `lista`, com os pares lado a lado como antes (`PLAYER_PARES`).
    fn rows(&mut self, x: f32, w: f32, mut y: f32, lista: &[NodeId]) -> f32 {
        let no_cartao = ph2d_editor_core::widget::card_ink::card_body().is_some();
        let visiveis: Vec<&PlayerRow> = lista
            .iter()
            .filter(|id| self.shown(**id))
            .filter_map(|id| row_of(*id))
            .collect();
        let mut i = 0;
        while i < visiveis.len() {
            let &(label, id, _, unit) = visiveis[i];
            let par = visiveis
                .get(i + 1)
                .filter(|p| no_cartao && super::PLAYER_PARES.contains(&(id, p.1)));
            y = if let Some(&&(l2, id2, _, u2)) = par {
                i += 1;
                let campos = [Campo::new(label, id, unit), Campo::new(l2, id2, u2)];
                par_row(
                    self.scene, self.text, self.theme, self.hit, self.store, x, w, y, None, campos,
                )
            } else {
                num_row_unit(
                    self.scene, self.text, self.theme, self.hit, self.store, x, w, y, label, id,
                    unit, None,
                )
            };
            i += 1;
        }
        y
    }

    /// «12 m/s»: o número que a caixa mostra, na unidade da tela.
    fn valor(&self, id: NodeId) -> String {
        // A conversão m → px deixa resto de ponto flutuante (45.000 em vez de 45): o resumo mostra duas casas.
        let v = self.store.number_value(id).unwrap_or(0.0);
        let unit = row_of(id).and_then(|r| r.3);
        let (unit, _) = crate::unidade_mundo::no_campo(&[id], unit, 0.0);
        match unit {
            Some(u) => format!("{} {}", duas_casas(v), u.suffix()),
            None => duas_casas(v),
        }
    }

    fn resumo(&self, partes: &[(TextKey, NodeId)]) -> String {
        partes
            .iter()
            .map(|(k, id)| format!("{} {}", k.tr(), self.valor(*id)))
            .collect::<Vec<_>>()
            .join(" · ")
    }

    /// Uma dobra com nome e, fechada, o resumo à direita.
    fn dobra(
        &mut self,
        y: f32,
        id: NodeId,
        titulo: &str,
        resumo: &str,
        corpo: impl FnOnce(&mut Self, f32) -> f32,
    ) -> f32 {
        let g = super::rows::grupo_dobravel(
            self.scene, self.text, self.theme, self.hit, self.store, self.x, self.w, y, id, titulo,
        );
        let fim = if g.aberto() {
            corpo(self, g.y)
        } else {
            super::rows::resumo_do_grupo(
                self.scene, self.text, self.theme, self.x, self.w, g.y, titulo, resumo,
            );
            g.y
        };
        g.fecha(self.store, self.scene, self.hit, fim)
    }

    /// ⭐ **A dobra de uma HABILIDADE**: a caixa à esquerda do chevron liga e desliga; o chevron e o
    /// nome abrem. Fechada e ligada, o resumo diz o número principal; desligada, nada.
    fn habilidade(
        &mut self,
        y: f32,
        i: usize,
        fold: NodeId,
        titulo: &str,
        resumo: &[(TextKey, NodeId)],
        corpo: impl FnOnce(&mut Self, f32) -> f32,
    ) -> f32 {
        let ligada = self.info.abilities[i];
        let caixa_id = ids::INSP_PLAYER_ABILITY[i];
        let h = ph2d_editor_core::widget::group_header_h();
        let topo =
            y + ph2d_editor_core::widget::card_ink::card_body().map_or(0.0, |_| Spacing::Sm.px());
        let lado = h.min(TypeToken::Sm.px() + Spacing::Xs.px());
        let caixa = Rect::new(self.x, topo + (h - lado) * 0.5, lado, lado);
        super::rows::caixa_de_cabecalho(
            self.scene, self.text, self.theme, self.hit, self.store, caixa_id, caixa, ligada,
        );
        let dx = lado + Spacing::Sm.px();
        let texto = if ligada {
            self.resumo(resumo)
        } else {
            String::new()
        };
        // O título começa depois da caixa, mas a dobra (o recorte da animação) é da largura inteira:
        // com a dobra também deslocada, a faixa sob a caixa só aparecia no fim da abertura (smoke 10/11).
        use ph2d_editor_core::widget::{SectionHeader, paint_group_header};
        let header = SectionHeader::new(fold, titulo)
            .collapsible(!self.store.is_collapsed(fold))
            .open_t(self.store.section_open_live(fold));
        let rect = Rect::new(self.x + dx, topo, self.w - dx, h);
        paint_group_header(&header, rect, self.scene, self.text, self.theme);
        self.hit.register(fold, rect);
        let corpo_y = topo + h + ph2d_tokens::control_gap_px();
        match ph2d_editor_core::widget::SectionFold::begin(
            self.store, fold, self.x, self.w, corpo_y, self.scene, self.hit,
        ) {
            Some(dobra) => {
                let fim = corpo(self, corpo_y);
                dobra.finish(self.store, self.scene, self.hit, fim)
            }
            None => {
                super::rows::resumo_do_grupo(
                    self.scene, self.text, self.theme, rect.x, rect.w, corpo_y, titulo, &texto,
                );
                corpo_y
            }
        }
    }

    fn botao(&mut self, y: f32, id: NodeId, label: &str) -> f32 {
        let rect =
            ph2d_editor_core::property_row::caixa_do_botao(self.text, self.x, self.w, y, label);
        let btn = Button::new(id, label)
            .kind(ButtonKind::Default)
            .visual(self.store.button_visual(id));
        paint_button(&btn, rect, self.scene, self.text, self.theme);
        self.hit.register(id, rect);
        ph2d_editor_core::property_row::abaixo_do_botao(rect)
    }

    fn sim_ou_beira(
        &mut self,
        y: f32,
        campo: TextKey,
        id: NodeId,
        chips: &[NodeId; 2],
        v: u8,
    ) -> f32 {
        seg_row(
            self.scene,
            self.text,
            self.theme,
            self.hit,
            self.store,
            self.x,
            self.w,
            y,
            campo,
            id,
            chips,
            &[
                tr("panel.inspector.player.yes"),
                tr("panel.inspector.player.stop_at_edge"),
            ],
            v,
        )
    }
}

/// O corpo do Platform Player, de Walk até Advanced.
pub(super) fn paint(p: &mut Pintor<'_>, mut y: f32) -> f32 {
    y = cartao(p, y, "panel.inspector.player.walk", &WALK);
    let resumo = p.resumo(&[
        (
            TextKey::new("panel.inspector.player.sum_air"),
            ids::INSP_PLAYER_AIR_ACCEL,
        ),
        (
            TextKey::new("panel.inspector.player.sum_slope"),
            ids::INSP_PLAYER_MAX_SLOPE,
        ),
    ]);
    y = p.dobra(
        y,
        ids::INSP_PLAYER_FOLD_WALK_MORE,
        tr("panel.inspector.player.more_walk"),
        &resumo,
        |p, y| {
            let y = p.rows(p.x, p.w, y, &WALK_MORE);
            p.sim_ou_beira(
                y,
                TextKey::new("component.field.platform_player.53"),
                ids::INSP_PLAYER_WALK_OFF,
                &ids::INSP_PLAYER_WALK_OFF_IDS,
                p.info.walk_off_ledges,
            )
        },
    );
    y = cartao(p, y, "panel.inspector.player.jump", &JUMP);
    let resumo = p.resumo(&[
        (
            TextKey::new("panel.inspector.player.sum_fall"),
            ids::INSP_PLAYER_FALL_G,
        ),
        (
            TextKey::new("panel.inspector.player.sum_cut"),
            ids::INSP_PLAYER_CUT_G,
        ),
    ]);
    y = p.dobra(
        y,
        ids::INSP_PLAYER_FOLD_JUMP_MORE,
        tr("panel.inspector.player.more_jump"),
        &resumo,
        |p, y| p.rows(p.x, p.w, y, &JUMP_MORE),
    );
    y = habilidades(p, y);
    y = perdao_e_perna(p, y);
    avancado(p, y)
}

fn habilidades(p: &mut Pintor<'_>, mut y: f32) -> f32 {
    use TextKey as K;
    y = p.habilidade(
        y,
        0,
        ids::INSP_PLAYER_CARD_WALL,
        tr("panel.inspector.player.walls"),
        &[
            (
                K::new("panel.inspector.player.sum_slide"),
                ids::INSP_PLAYER_WALL_SLIDE,
            ),
            (
                K::new("panel.inspector.player.sum_jump"),
                ids::INSP_PLAYER_WALL_JUMP,
            ),
        ],
        |p, y| {
            let y = p.rows(p.x, p.w, y, &WALLS);
            let resumo = p.resumo(&[(
                K::new("panel.inspector.player.sum_reach"),
                ids::INSP_PLAYER_WALL_REACH,
            )]);
            p.dobra(
                y,
                ids::INSP_PLAYER_FOLD_WALL_SENSOR,
                tr("panel.inspector.player.wall_sensor"),
                &resumo,
                |p, y| p.rows(p.x, p.w, y, &WALL_SENSOR),
            )
        },
    );
    y = p.habilidade(
        y,
        1,
        ids::INSP_PLAYER_CARD_DASH,
        tr("panel.inspector.player.dash"),
        &[(
            K::new("panel.inspector.player.sum_speed"),
            ids::INSP_PLAYER_DASH_SPEED,
        )],
        |p, y| p.rows(p.x, p.w, y, &DASH),
    );
    y = p.habilidade(
        y,
        2,
        ids::INSP_PLAYER_CARD_CROUCH,
        tr("panel.inspector.player.crouch"),
        &[(
            K::new("panel.inspector.player.sum_height"),
            ids::INSP_PLAYER_CROUCH_HEIGHT,
        )],
        |p, y| {
            let mut y = p.rows(p.x, p.w, y, &CROUCH);
            if p.info.crouch_armed {
                y = p.sim_ou_beira(
                    y,
                    K::new("component.field.platform_player.54"),
                    ids::INSP_PLAYER_CROUCH_WALK_OFF,
                    &ids::INSP_PLAYER_CROUCH_WALK_OFF_IDS,
                    p.info.crouch_walk_off_ledges,
                );
            }
            match fit_crouch(p.info) {
                Some(label) => p.botao(y, ids::INSP_PLAYER_FIT_CROUCH, &label),
                None => y,
            }
        },
    );
    y = p.habilidade(
        y,
        3,
        ids::INSP_PLAYER_CARD_SWIM,
        tr("panel.inspector.player.swim"),
        &[(
            K::new("panel.inspector.player.sum_speed"),
            ids::INSP_PLAYER_SWIM_SPEED,
        )],
        |p, y| p.rows(p.x, p.w, y, &SWIM),
    );
    y = p.habilidade(
        y,
        4,
        ids::INSP_PLAYER_CARD_LEDGE,
        tr("panel.inspector.player.ledge"),
        &[(
            K::new("panel.inspector.player.sum_grab"),
            ids::INSP_PLAYER_LEDGE_GRAB,
        )],
        |p, y| p.rows(p.x, p.w, y, &LEDGE),
    );
    p.habilidade(
        y,
        5,
        ids::INSP_PLAYER_CARD_GLIDE,
        tr("panel.inspector.player.glide"),
        &[(
            K::new("panel.inspector.player.sum_fall"),
            ids::INSP_PLAYER_GLIDE_FALL,
        )],
        |p, y| p.rows(p.x, p.w, y, &GLIDE),
    )
}

fn perdao_e_perna(p: &mut Pintor<'_>, y: f32) -> f32 {
    use TextKey as K;
    let resumo = p.resumo(&[
        (
            K::new("panel.inspector.player.sum_coyote"),
            ids::INSP_PLAYER_COYOTE,
        ),
        (
            K::new("panel.inspector.player.sum_buffer"),
            ids::INSP_PLAYER_BUFFER,
        ),
    ]);
    let y = p.dobra(
        y,
        ids::INSP_PLAYER_CARD_FORGIVE,
        tr("panel.inspector.player.forgiveness"),
        &resumo,
        |p, y| p.rows(p.x, p.w, y, &FORGIVE),
    );
    let resumo = p.resumo(&[(
        K::new("panel.inspector.player.sum_float"),
        ids::INSP_PLAYER_FLOAT,
    )]);
    p.dobra(
        y,
        ids::INSP_PLAYER_CARD_LEG,
        tr("panel.inspector.player.leg"),
        &resumo,
        |p, y| {
            let y = p.rows(p.x, p.w, y, &LEG);
            match fit(p.info) {
                Some(label) => p.botao(y, ids::INSP_PLAYER_FIT, &label),
                None => y,
            }
        },
    )
}

fn avancado(p: &mut Pintor<'_>, y: f32) -> f32 {
    let modo = [
        tr("panel.inspector.player.dynamic"),
        tr("panel.inspector.player.kinematic"),
        tr("panel.inspector.player.pure"),
    ];
    let modo_agora = modo
        .get(usize::from(p.info.mode_tag))
        .copied()
        .unwrap_or_default();
    let resumo = if p.info.emits_signals {
        format!(
            "{modo_agora} · {}",
            tr("panel.inspector.player.sum_signals_on")
        )
    } else {
        modo_agora.to_owned()
    };
    p.dobra(
        y,
        ids::INSP_PLAYER_FOLD_ADVANCED,
        tr("panel.inspector.player.advanced"),
        &resumo,
        |p, y| {
            let mut y = seg_row(
                p.scene,
                p.text,
                p.theme,
                p.hit,
                p.store,
                p.x,
                p.w,
                y,
                TextKey::new("component.field.player_mode.0"),
                ids::INSP_PLAYER_MODE,
                &ids::INSP_PLAYER_MODE_IDS,
                &modo,
                p.info.mode_tag,
            );
            y = caixa(
                p,
                y,
                ids::INSP_PLAYER_EMIT_BOX,
                tr("panel.inspector.player.emit_signals"),
                p.info.emits_signals,
            );
            y = seg_row(
                p.scene,
                p.text,
                p.theme,
                p.hit,
                p.store,
                p.x,
                p.w,
                y,
                TextKey::new("component.field.platform_player.52"),
                ids::INSP_PLAYER_LIFT_POLICY,
                &ids::INSP_PLAYER_LIFT_POLICY_IDS,
                &[
                    tr("panel.inspector.player.full"),
                    tr("panel.inspector.player.up_only"),
                    tr("panel.inspector.player.none"),
                ],
                p.info.platform_lift,
            );
            let i = ph2d_editor_core::screens::hero::PlayerAbility::ALL.len() - 1;
            let ligada = p.info.abilities[i];
            y = caixa(
                p,
                y,
                ids::INSP_PLAYER_ABILITY[i],
                TextKey::new("component.field.platform_player.61").tr(),
                ligada,
            );
            if ligada {
                y = p.rows(p.x, p.w, y, &FALL);
            }
            if p.info.reaction_is_live {
                y = p.rows(p.x, p.w, y, &REACT);
            }
            y
        },
    )
}

fn caixa(p: &mut Pintor<'_>, y: f32, id: NodeId, label: &str, on: bool) -> f32 {
    let valor = if on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let sec = ph2d_editor_core::property_row::Seccao::medida(p.text, 1, &[label]);
    caixa_row(
        p.scene,
        p.text,
        p.theme,
        p.hit,
        p.store,
        p.x,
        p.w,
        y,
        (id, label, valor),
        sec,
    )
}

fn fit(info: &InspectorPlayerInfo) -> Option<String> {
    (info.min_float_known && info.spring_is_live).then(|| {
        if info.float_height <= info.min_float_height {
            ph2d_i18n::tr_with(
                "panel.inspector.player.fit_needs",
                &[("min", &format!("{:.2}", info.min_float_height))],
            )
        } else {
            tr("panel.inspector.player.fit_to_collider").to_string()
        }
    })
}

fn fit_crouch(info: &InspectorPlayerInfo) -> Option<String> {
    (info.min_float_known && info.crouch_height > 0.0).then(|| {
        if info.crouch_height <= info.min_float_height {
            ph2d_i18n::tr_with(
                "panel.inspector.player.fit_crouch_needs",
                &[("min", &format!("{:.2}", info.min_float_height))],
            )
        } else {
            tr("panel.inspector.player.fit_crouch_to_collider").to_string()
        }
    })
}

/// O número do resumo com até duas casas: a conversão m → px deixa resto de ponto flutuante (45.000 em vez de 45).
fn duas_casas(v: f64) -> String {
    let t = format!("{v:.2}");
    let t = t.trim_end_matches('0').trim_end_matches('.');
    ph2d_editor_core::widget::zero_sem_sinal(t).into_owned()
}
