//! ⭐ **Casos reais: o mundo do jogo** (doc `CartoesUI/08` §1.3) — Image, Physics Body, Collider, Health,
//! Platform Player e Camera, com os nomes, unidades e faixas do código real. Quem constrói o cartão da
//! família copia este. As decisões do dono: o seletor «Use» logo abaixo de Image, nunca checkbox
//! (26/10); o alvo da câmera por arrastar, escrever ou conta-gotas, nunca lista (02/11).

use crate::controlos::{barra, card, dica, dropdown, field, marca, marca_vista, num, segs};
use crate::ids as g;
use crate::state::{Choices, Flag, Pick};
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;

pub(crate) fn image(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_IMAGE,
        tr("panel.widget_gallery.card.image"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let assets = segs(
                [g::IMG_ASSET_0, g::IMG_ASSET_1, g::IMG_ASSET_2],
                &ch,
                c.store,
            );
            let y = c.row(
                &[Cell::new(dropdown(g::IMG_ASSET, &assets))
                    .label(tr("panel.widget_gallery.card.image"))],
                y,
            );
            let usos = segs(
                [g::USE_STILL, g::USE_REPEATED, g::USE_ANIMATED],
                &ch,
                c.store,
            );
            let y = c.row(
                &[Cell::new(Widget::Choice(&usos)).label(tr("panel.widget_gallery.real.use"))],
                y,
            );
            match ch.pick(Pick::ImgUse) {
                0 => c.row(&[num!(g::IMG_FRAME, tr("panel.inspector.image.frame"))], y),
                1 => c.row(
                    &[
                        num!(g::IMG_W, tr("panel.widget_gallery.real.width")),
                        num!(g::IMG_H, tr("panel.widget_gallery.real.height")),
                    ],
                    y,
                ),
                _ => {
                    let y = c.row(
                        &[
                            num!(g::IMG_COLS, tr("panel.widget_gallery.real.columns")),
                            num!(g::IMG_ROWS, tr("panel.widget_gallery.real.rows")),
                        ],
                        y,
                    );
                    c.row(&[num!(g::IMG_FPS, tr("panel.inspector.anim_card.fps"))], y)
                }
            }
        }),
    )
}

pub(crate) fn rigid_body(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_RIGID_BODY,
        tr("component.rigid_body.name"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let corpos = segs([g::RB_DYNAMIC, g::RB_KINEMATIC, g::RB_STATIC], &ch, c.store);
            let mut y = c.row(
                &[Cell::new(Widget::Choice(&corpos))
                    .label(tr("component.field.rigid_body.0"))
                    .key(&[g::RB_DYNAMIC, g::RB_KINEMATIC, g::RB_STATIC])],
                y,
            );
            let tipo = ch.pick(Pick::RbBody);
            if tipo == 2 {
                return dica(c, tr("panel.widget_gallery.real.body.static_hint"), y);
            }
            if tipo == 0 {
                y = c.row(
                    &[
                        num!(g::RB_MASS, tr("component.field.mass_override.0")),
                        num!(g::RB_GRAVITY, tr("component.field.gravity_scale.0")),
                    ],
                    y,
                );
                y = c.row(&[num!(g::RB_LINEAR, tr("canal.fisica.amort_linear"))], y);
                y = c.row(&[num!(g::RB_ANGULAR, tr("canal.fisica.amort_angular"))], y);
            }
            marca(
                c,
                g::RB_LOCK,
                "canal.fisica.trava_rotacao",
                ch.on(Flag::RbLock),
                y,
            )
        }),
    )
}

pub(crate) fn collider(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_COLLIDER,
        tr("panel.widget_gallery.card.collider"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let formas = segs([g::COL_BOX, g::COL_CIRCLE, g::COL_CAPSULE], &ch, c.store);
            let y = c.row(
                &[Cell::new(Widget::Icons(&formas))
                    .label(tr("component.field.collider.0"))
                    .key(&[g::COL_BOX, g::COL_CIRCLE, g::COL_CAPSULE])],
                y,
            );
            let raio = num!(
                g::COL_RADIUS,
                IconId::CardRadius,
                tr("panel.widget_gallery.real.radius")
            );
            let y = match ch.pick(Pick::ColShape) {
                0 => c.row(
                    &[
                        num!(g::COL_W, tr("panel.widget_gallery.real.half_width")),
                        num!(g::COL_H, tr("panel.widget_gallery.real.half_height")),
                    ],
                    y,
                ),
                1 => c.row(&[raio], y),
                _ => c.row(
                    &[
                        raio,
                        num!(g::COL_HALF, tr("panel.widget_gallery.real.half_height")),
                    ],
                    y,
                ),
            };
            let y = c.row(
                &[
                    num!(g::COL_OX, tr("panel.widget_gallery.real.offset_x")),
                    num!(g::COL_OY, tr("panel.widget_gallery.real.offset_y")),
                ],
                y,
            );
            let y = c.row(
                &[
                    num!(g::COL_FRICTION, tr("component.field.collider.3")),
                    num!(g::COL_BOUNCE, tr("component.field.collider.2")),
                ],
                y,
            );
            marca(
                c,
                g::COL_TRIGGER,
                "component.field.collider.5",
                ch.on(Flag::ColTrigger),
                y,
            )
        }),
    )
}

pub(crate) fn health(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_HEALTH,
        tr("component.health.name"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let y = c.row(
                &[
                    num!(g::HP_MAX, tr("component.field.health_fields.1")),
                    num!(g::HP_START, tr("component.field.health_fields.2")),
                ],
                y,
            );
            let valor = |id| c.store.number_value(id).unwrap_or_default();
            let fracao = (valor(g::HP_START) / valor(g::HP_MAX).max(1.0)) as f32;
            let y = barra(c, 0.0, fracao, y);
            let y = c.row(
                &[
                    num!(g::HP_REGEN, tr("component.field.health_fields.5")),
                    num!(g::HP_DELAY, tr("panel.widget_gallery.real.regen_delay")),
                ],
                y,
            );
            marca(
                c,
                g::HP_OVERHEAL,
                "component.field.health_fields.4",
                ch.on(Flag::HpOverheal),
                y,
            )
        }),
    )
}

pub(crate) fn player() -> CardDef<'static> {
    card(
        g::CARD_PLAYER,
        tr("component.platform_player.name"),
        Vec::new(),
        Box::new(|c: &mut CardCtx<'_>, y| {
            let y = c.row(
                &[num!(g::PL_SPEED, tr("panel.widget_gallery.real.speed"))],
                y,
            );
            let y = c.row(
                &[
                    num!(g::PL_ACCEL, tr("panel.widget_gallery.real.accel")),
                    num!(g::PL_AIR, tr("panel.widget_gallery.real.air_accel")),
                ],
                y,
            );
            let y = c.row(
                &[
                    num!(g::PL_JUMP, tr("panel.widget_gallery.real.jump_height")),
                    num!(g::PL_AIR_JUMPS, tr("component.field.platform_player.9")),
                ],
                y,
            );
            c.row(
                &[
                    num!(g::PL_COYOTE, tr("panel.widget_gallery.real.coyote")),
                    num!(g::PL_BUFFER, tr("panel.widget_gallery.real.buffer")),
                ],
                y,
            )
        }),
    )
}

/// `true` se a cena fictícia tem um objeto com este nome (a câmera a seguir um nome que não existe
/// diz-o, em vez de ficar parada sem explicação).
fn objeto_existe(nome: &str) -> bool {
    [
        "panel.widget_gallery.name_default",
        "panel.widget_gallery.real.object.enemy",
        "panel.widget_gallery.real.object.coin",
    ]
    .iter()
    .any(|k| tr(k).eq_ignore_ascii_case(nome.trim()))
}

/// O texto de estado do alvo da câmera (público para o teste de costura o comparar).
#[must_use]
pub fn estado_do_alvo(a_apontar: bool, alvo: &str) -> String {
    if a_apontar {
        tr("panel.widget_gallery.real.target_picking").to_owned()
    } else if objeto_existe(alvo) {
        tr("panel.widget_gallery.real.target_follows").replace("{name}", alvo.trim())
    } else {
        tr("panel.widget_gallery.real.target_missing").replace("{name}", alvo.trim())
    }
}

/// ⭐ **O modelo do cartão Camera** (doc 05 §8: o dono achou o do Inspector «todo bugado»): título,
/// tudo em células da tinta do cartão, a marca na linha do rótulo, o alvo sem lista.
pub(crate) fn camera(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_CAMERA,
        tr("component.game_camera.name"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let conta_gotas = segs([g::CAM_PICK], &ch, c.store);
            let y = c.row(
                &[
                    Cell::new(Widget::Text {
                        id: g::CAM_TARGET,
                        placeholder: tr("panel.widget_gallery.real.target_hint"),
                    })
                    .label(tr("component.field.follow_fields.0")),
                    Cell::new(Widget::Bare(&conta_gotas)),
                ],
                y,
            );
            let alvo = c.store.text(g::CAM_TARGET).unwrap_or_default().to_owned();
            let estado = estado_do_alvo(ch.on(Flag::CamPicking), &alvo);
            let y = dica(c, &estado, y);
            let formatos = segs(
                [g::CAM_FORMAT_0, g::CAM_FORMAT_1, g::CAM_FORMAT_2],
                &ch,
                c.store,
            );
            let y = c.row(
                &[Cell::new(dropdown(g::CAM_FORMAT, &formatos))
                    .label(tr("panel.inspector.camera.format"))],
                y,
            );
            let quadro = segs([g::CAM_FRAME_ALWAYS, g::CAM_FRAME_SELECTED], &ch, c.store);
            let y = c.row(
                &[Cell::new(Widget::Choice(&quadro)).label(tr("panel.inspector.camera.frame"))],
                y,
            );
            let y = c.row(
                &[
                    Cell::new(field(
                        g::CAM_DEAD_W,
                        Glyph::Text(tr("panel.widget_gallery.glyph.w")),
                    ))
                    .label(tr("panel.widget_gallery.real.dead_zone")),
                    Cell::new(field(
                        g::CAM_DEAD_H,
                        Glyph::Text(tr("panel.widget_gallery.glyph.h")),
                    ))
                    .key(&[g::CAM_DEAD_W, g::CAM_DEAD_H]),
                ],
                y,
            );
            let y = c.row(
                &[
                    num!(g::CAM_DAMPING, tr("panel.widget_gallery.real.damping")),
                    num!(g::CAM_LOOKAHEAD, tr("panel.widget_gallery.real.lookahead")),
                ],
                y,
            );
            let y = marca(
                c,
                g::CAM_ACTIVE,
                "component.field.camera_fields.3",
                ch.on(Flag::CamActive),
                y,
            );
            marca_vista(
                c,
                g::CAM_LOOK,
                "panel.widget_gallery.real.look_through",
                ch.on(Flag::CamLook),
                y,
            )
        }),
    )
}
