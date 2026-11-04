//! ⭐ **A barra do transporte em cartões** (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): a mesma lista
//! de `ITEMS`, os mesmos ids e o mesmo despacho, com os controlos da lista flutuante: segmentos, chips,
//! campo com pega, caixas T64. `PH2D_CARDS=0` pinta a barra de [`super::paint_bar`].

use super::*;
use ph2d_editor_core::panel::cards::banda::{self, Icone, metrics as m};
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, Seg};
use ph2d_editor_core::widget::card_field::{self, CardFieldSpec, Glyph};
use ph2d_editor_core::widget::card_ink::CardInk;

/// O passo da Dur (ver [`chips`]) e a largura do chip da animação e do clip.
const ANIM_W: f32 = 168.0; // LITERAL-PX-OK: «Animation: Scene» + seta, medido na foto; nomes longos cortam
const CLIP_W: f32 = 108.0; // LITERAL-PX-OK: a mesma largura do chip de antes

const DICAS: [(ph2d_a11y::NodeId, &str); 13] = [
    (TIMELINE_GO_START, "panel.timeline.tip.go_start"),
    (TIMELINE_PREV_FRAME, "panel.timeline.tip.prev_frame"),
    (TIMELINE_PLAY, "panel.timeline.tip.play"),
    (TIMELINE_NEXT_FRAME, "panel.timeline.tip.next_frame"),
    (TIMELINE_GO_END, "panel.timeline.tip.go_end"),
    (ids::TIMELINE_ADD_CLIP, "panel.timeline.tip.add_clip"),
    (ids::TIMELINE_DUP_CLIP, "panel.timeline.tip.dup_clip"),
    (ids::TIMELINE_RENAME_CLIP, "panel.timeline.tip.rename_clip"),
    (ids::TIMELINE_DELETE_CLIP, "panel.timeline.tip.delete_clip"),
    (ids::TIMELINE_REVERSE_CLIP, "panel.timeline.tip.reverse"),
    (TIMELINE_ADD_MARKER, "panel.timeline.tip.add_marker"),
    (
        ids::TIMELINE_ONION_SETTINGS,
        "panel.timeline.tip.onion_settings",
    ),
    (ids::TIMELINE_ONION_MODE, "panel.timeline.tip.onion_keys"),
];

/// O estado de cada caixa: id, nome (`Item`) e valor. Um valor de TAB não mora no documento.
fn caixas(snap: &TimelineViewSnapshot, view: BarView) -> [(Item, ph2d_a11y::NodeId, bool); 10] {
    let laco = snap.loop_range.is_some();
    [
        (Item::Loop, TIMELINE_LOOP, laco && !snap.loop_ping_pong),
        (
            Item::PingPong,
            TIMELINE_PINGPONG,
            laco && snap.loop_ping_pong,
        ),
        (Item::Physics, TIMELINE_PHYSICS, snap.simulate_physics),
        (Item::AutoKey, TIMELINE_AUTOKEY, snap.auto_key),
        (Item::Record, TIMELINE_RECORD, snap.performing),
        (
            Item::MotionPath,
            TIMELINE_MOTION_PATH,
            snap.position_is_path,
        ),
        (Item::Snap, TIMELINE_SNAP, snap.frame_snap),
        (Item::Speed, ids::TIMELINE_SPEED, view.speed_view),
        (Item::Onion, ids::TIMELINE_ONION, snap.onion.enabled),
        (
            Item::OnionMode,
            ids::TIMELINE_ONION_MODE,
            snap.onion.mode == ph2d_timeline::OnionMode::Keys,
        ),
    ]
}

/// O que o store precisa de saber antes de a fila se pintar com ele emprestado.
fn prepara(store: &mut WidgetStore, snap: &TimelineViewSnapshot, view: BarView) {
    let fps = if snap.fps > 0.0 {
        snap.fps
    } else {
        DEFAULT_FPS
    };
    let arrastado = store.number_input_drag().map(|d| d.id);
    for (id, valor, passo, casas) in [
        (TIMELINE_TIME_NUM, snap.time_seconds, 1.0 / fps, 2),
        (TIMELINE_FRAME_NUM, snap.frame as f64, 1.0, 0),
        (
            ids::TIMELINE_LENGTH_NUM,
            snap.view_length_seconds,
            chips::DUR_STEP_SECONDS,
            2,
        ),
    ] {
        store.mark_card_field(
            id,
            CardFieldSpec {
                step: passo,
                scrub: passo,
                default: Some(0.0),
            },
        );
        store.set_number_range(id, 0.0, f64::INFINITY, passo);
        store.set_number_drag_rate(id, passo);
        if arrastado != Some(id) {
            mirror_number(store, id, valor, casas);
        }
    }
    let mut espelha = |id, on| {
        if let Some(InteractiveState::Toggle { on: no_store, .. }) = store.get_mut(id) {
            *no_store = on;
        }
    };
    espelha(
        ids::TIMELINE_CLIP_AUTOPLAY,
        crate::transport_entry::ligado(snap),
    );
    for (_, id, on) in caixas(snap, view) {
        espelha(id, on);
    }
    for (id, chave) in DICAS {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, ph2d_i18n::tr(chave));
        }
    }
    store.mark_card_list_close(ph2d_editor_core::ids::TIMELINE_PANEL, ids::TIMELINE_CLOSE);
}

fn segs_das_abas<'a>(celulas: &'a [(ph2d_a11y::NodeId, String)], escolhida: usize) -> Vec<Seg<'a>> {
    celulas
        .iter()
        .enumerate()
        .map(|(i, (id, nome))| Seg {
            id: *id,
            icon: None,
            text: nome,
            on: i == escolhida,
        })
        .collect()
}

fn rotulo_do_campo(item: Item) -> &'static str {
    match item {
        Item::TimeChip => ph2d_i18n::tr("panel.timeline.card.time"),
        Item::LengthChip => ph2d_i18n::tr("panel.timeline.card.length"),
        _ => chip_rotulo(item),
    }
}

fn botoes_do_clip(snap: &TimelineViewSnapshot) -> Vec<Icone> {
    let mut v: Vec<Icone> = [
        (ids::TIMELINE_ADD_CLIP, IconId::Plus),
        (ids::TIMELINE_DUP_CLIP, IconId::Duplicate),
        (ids::TIMELINE_RENAME_CLIP, IconId::Text),
    ]
    .into_iter()
    .map(|(id, icon)| Icone {
        id: Some(id),
        icon,
        on: false,
    })
    .collect();
    // O lixo só existe acima de um clip: um documento tem sempre um para editar.
    if snap.clips.len() > 1 {
        v.push(Icone {
            id: Some(ids::TIMELINE_DELETE_CLIP),
            icon: IconId::Trash,
            on: false,
        });
    }
    v
}

fn fileira_do_transporte(snap: &TimelineViewSnapshot, view: BarView) -> [Icone; 5] {
    let vivo = |id, icon| Icone {
        id: Some(id),
        icon,
        on: false,
    };
    let play = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    // A lista de Containers não tem playback: o Play apaga-se e não regista alvo nenhum.
    let sem_tempo = crate::tab::rows(view.tab, snap) == crate::tab::Rows::Containers;
    [
        vivo(TIMELINE_GO_START, IconId::SkipBack),
        vivo(TIMELINE_PREV_FRAME, IconId::ChevronLeft),
        if sem_tempo {
            Icone {
                id: None,
                icon: play,
                on: false,
            }
        } else {
            vivo(TIMELINE_PLAY, play)
        },
        vivo(TIMELINE_NEXT_FRAME, IconId::ChevronRight),
        vivo(TIMELINE_GO_END, IconId::SkipForward),
    ]
}

fn clips_w(snap: &TimelineViewSnapshot) -> f32 {
    CLIP_W + m::FIELD_GAP + banda::icones_w(botoes_do_clip(snap).len())
}

fn largura(c: &mut CardCtx<'_>, item: Item, snap: &TimelineViewSnapshot) -> f32 {
    match item {
        Item::Tabs => {
            let celulas = crate::transport_tabs::items(snap);
            c.segmentos_em_fila_w(&segs_das_abas(&celulas, 0))
        }
        Item::Crumbs => crate::breadcrumb::width(snap),
        Item::Animation => ANIM_W,
        Item::Clips => clips_w(snap),
        Item::Transport => banda::icones_w(TRANSPORT_BTNS),
        Item::ReverseKeys | Item::OnionSettings => banda::icones_w(1),
        Item::AddMarker => c.botao_w(ph2d_i18n::tr("panel.timeline.add_marker")),
        Item::TimeChip | Item::FrameChip | Item::LengthChip => {
            c.campo_em_fila_w(rotulo_do_campo(item))
        }
        _ => c.marca_em_fila_w(rotulo(item)),
    }
}

fn numero(c: &mut CardCtx<'_>, item: Item, id: ph2d_a11y::NodeId, x: f32, y: f32) {
    let nome = rotulo_do_campo(item);
    c.campo_em_fila(id, nome, Glyph::Text(letra_da_grandeza(nome)), x, y);
}

fn pinta_clips(
    c: &mut CardCtx<'_>,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> ClipChip {
    let chip = Rect::new(x, y, CLIP_W, m::FIELD_H);
    let open = matches!(
        c.store.get(ids::TIMELINE_CLIP_DD),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let texto = crate::transport_clips::source_options(snap, view.tab)
        .into_iter()
        .nth(crate::transport_clips::selected_source(snap, view))
        .map(|o| o.label)
        .unwrap_or_default();
    c.lista_em(ids::TIMELINE_CLIP_DD, &texto, open, chip);
    let botoes = botoes_do_clip(snap);
    let bx = x + CLIP_W + m::FIELD_GAP;
    c.icones_em_fila(&botoes, bx, y);
    ClipChip {
        id: ids::TIMELINE_CLIP_DD,
        rect: chip,
        open,
    }
}

fn pinta(
    c: &mut CardCtx<'_>,
    item: Item,
    (x, y): (f32, f32),
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> Option<ClipChip> {
    match item {
        Item::Tabs => {
            let celulas = crate::transport_tabs::items(snap);
            let segs = segs_das_abas(&celulas, crate::transport_tabs::selected(view.tab, snap));
            c.segmentos_em_fila(&segs, x, y);
        }
        Item::Crumbs => {}
        Item::Animation => {
            let rect = Rect::new(x, y, ANIM_W, m::FIELD_H);
            let open = matches!(
                c.store.get(ids::TIMELINE_ANIM_DD),
                Some(InteractiveState::Dropdown { open: true, .. })
            );
            let texto = crate::transport_animation::chip_text(snap);
            c.lista_em(ids::TIMELINE_ANIM_DD, &texto, open, rect);
            return Some(ClipChip {
                id: ids::TIMELINE_ANIM_DD,
                rect,
                open,
            });
        }
        Item::Clips => return Some(pinta_clips(c, x, y, snap, view)),
        Item::Transport => c.icones_em_fila(&fileira_do_transporte(snap, view), x, y),
        Item::ReverseKeys => c.icones_em_fila(
            &[Icone {
                id: Some(ids::TIMELINE_REVERSE_CLIP),
                icon: IconId::LetterI,
                on: false,
            }],
            x,
            y,
        ),
        Item::OnionSettings => c.icones_em_fila(
            &[Icone {
                id: Some(ids::TIMELINE_ONION_SETTINGS),
                icon: IconId::Settings,
                on: false,
            }],
            x,
            y,
        ),
        Item::AddMarker => {
            let nome = ph2d_i18n::tr("panel.timeline.add_marker");
            let w = c.botao_w(nome);
            c.botao_em(TIMELINE_ADD_MARKER, nome, Rect::new(x, y, w, m::FIELD_H));
        }
        Item::TimeChip => numero(c, item, TIMELINE_TIME_NUM, x, y),
        Item::FrameChip => numero(c, item, TIMELINE_FRAME_NUM, x, y),
        Item::LengthChip => {
            let id = ids::TIMELINE_LENGTH_NUM;
            if snap.view_length_explicit {
                numero(c, item, id, x, y);
            } else {
                // Sem duração autorada a caixa lê ∞; o store fica com o finito, que é o que se edita.
                card_field::com_valores(&[(id, f64::INFINITY)], || numero(c, item, id, x, y));
            }
        }
        _ => {
            let par = caixas(snap, view).into_iter().find(|(i, _, _)| *i == item);
            if let Some((_, id, on)) = par {
                c.marca_em_fila(id, rotulo(item), on, x, y);
            }
        }
    }
    None
}

/// A barra na tira do cabeçalho `strip` e, o que não couber, em fileiras a partir de `body`. Devolve o
/// mesmo que [`super::paint_bar`]: onde a folha começa e os dois chips.
pub(crate) fn paint_bar(
    ctx: &mut PaintCtx,
    theme: Theme,
    strip: Rect,
    body: Rect,
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> (f32, Option<ClipChip>, Option<ClipChip>) {
    prepara(ctx.host.store_mut(), snap, view);
    let ink = CardInk::of_list(theme);
    let passo = m::FIELD_H + m::FIELD_ROW_GAP;
    let (mut clip_chip, mut anim_chip) = (None, None);
    let (mut lim_x, mut lim_w, mut y) = (strip.x, strip.w, strip.y);
    let mut fileiras = 0usize;
    let mut x = lim_x;
    // O Autoplay vem logo a seguir aos clips, mas pode descer sozinho numa barra estreita.
    let autoplay = ph2d_i18n::tr(crate::transport_entry::LABEL_KEY);
    let pecas = ITEMS.into_iter().flat_map(|i| {
        let extra = (i == Item::Clips).then_some(Peca::Autoplay);
        std::iter::once(Peca::Item(i)).chain(extra)
    });
    for peca in pecas {
        let (store, hit) = ctx.host.store_and_hit_index_mut();
        let mut c = CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit, ink, body.x, body.w);
        let w = match peca {
            Peca::Item(item) => largura(&mut c, item, snap),
            Peca::Autoplay => c.marca_em_fila_w(autoplay),
        };
        // Os três números são um grupo: não se partem entre fileiras.
        let precisa = if matches!(peca, Peca::Item(Item::TimeChip)) {
            [Item::TimeChip, Item::FrameChip, Item::LengthChip]
                .into_iter()
                .map(|i| largura(&mut c, i, snap) + m::FIELD_GAP)
                .sum::<f32>()
                - m::FIELD_GAP
        } else {
            w
        };
        if x > lim_x && x + precisa > lim_x + lim_w {
            fileiras += 1;
            (lim_x, lim_w) = (body.x, body.w);
            y = body.y + (fileiras - 1) as f32 * passo - banda::SOBRA_DO_CABECALHO;
            x = lim_x;
        }
        match peca {
            Peca::Autoplay => {
                let on = crate::transport_entry::ligado(snap);
                c.marca_em_fila(ids::TIMELINE_CLIP_AUTOPLAY, autoplay, on, x, y);
            }
            Peca::Item(item) => {
                match pinta(&mut c, item, (x, y), snap, view) {
                    Some(chip) if chip.id == ids::TIMELINE_ANIM_DD => anim_chip = Some(chip),
                    Some(chip) => clip_chip = Some(chip),
                    None => {}
                }
                if item == Item::Crumbs && w > 0.0 {
                    let meio = y + (m::FIELD_H - ROW_H_PX) * 0.5;
                    crate::breadcrumb::paint(ctx, theme, x, meio, snap);
                }
            }
        }
        x += w + m::FIELD_GAP;
    }
    let depois = if fileiras > 0 {
        y + m::FIELD_H + banda::VAO_ATE_A_FOLHA
    } else {
        body.y
    };
    (depois, clip_chip, anim_chip)
}

/// O que anda na fila: os itens da barra e o Autoplay, que no painel de antes ia colado aos clips.
#[derive(Copy, Clone, PartialEq)]
enum Peca {
    Item(Item),
    Autoplay,
}
