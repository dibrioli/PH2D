//! ⭐ **A barra da tira em cartões** (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`, o método da Timeline):
//! os mesmos 25 controlos, ids e despacho, com as peças da lista: fila de ícones, campo com pega e rótulo ao
//! lado, caixa, botão, lista. A fileira começa ao lado do título; o que não couber desce, por grupos.
//! `PH2D_CARDS=0` pinta a barra de [`crate::paint_toolbar`].

use crate::ids;
use crate::paint_toolbar::{CYCLE_NAMES, PendingCycle, TWEEN_EASE_NAMES};
use crate::populate::{FPS_DEFAULT, GHOST_DEFAULT, HOLD_DEFAULT, STEP, TWEEN_DEFAULT};
use crate::state::FlipStripSnapshot;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::panel::cards::banda::{self, Icone, metrics as m};
use ph2d_editor_core::panel::cards::censo;
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::widget::DropdownState;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::card_ink::{CardInk, troca_popover};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{TextKey, tr};
use ph2d_tokens::{Spacing, Theme};

/// O que a dica diz de cada controlo cujo nome não chega; os ícones não têm outro nome.
const DICAS: [(NodeId, &str); 19] = [
    (ids::FLIP_PREV_DRAWING, "panel.flip_frames.tip.prev_drawing"),
    (ids::FLIP_PLAY, "panel.flip_frames.tip.play"),
    (ids::FLIP_NEXT_DRAWING, "panel.flip_frames.tip.next_drawing"),
    (ids::FLIP_GHOST, "panel.flip_frames.tip.ghost"),
    (ids::FLIP_KEY_PIN, "panel.flip_frames.tip.pin"),
    (ids::FLIP_AUTOKEY, "panel.flip_frames.tip.auto"),
    (ids::FLIP_FALLOFF, "panel.flip_frames.tip.falloff"),
    (ids::FLIP_ADDITIVE, "panel.flip_frames.tip.additive"),
    (ids::FLIP_KEY_ADD, "panel.flip_frames.tip.key_add"),
    (ids::FLIP_KEY_DUP, "panel.flip_frames.tip.key_dup"),
    (ids::FLIP_KEY_INSTANCE, "panel.flip_frames.tip.key_instance"),
    (ids::FLIP_KEY_UNLINK, "panel.flip_frames.tip.key_unlink"),
    (ids::FLIP_KEY_DELETE, "panel.flip_frames.tip.key_delete"),
    (ids::FLIP_KEY_LEFT, "panel.flip_frames.tip.key_left"),
    (ids::FLIP_KEY_RIGHT, "panel.flip_frames.tip.key_right"),
    (ids::FLIP_TWEEN_FADE, "panel.flip_frames.tip.fade"),
    (ids::FLIP_TWEEN_PAIRS, "panel.flip_frames.tip.pairs"),
    (ids::FLIP_TWEEN_ADD, "panel.flip_frames.tip.add"),
    (ids::FLIP_CYCLE_DD, "panel.flip_frames.tip.cycle"),
];

/// Uma peça da barra.
enum Peca {
    Icones(Vec<Icone>),
    Campo(NodeId, &'static str),
    Marca(NodeId, &'static str, bool),
    Verbo(NodeId, &'static str),
    Lista(NodeId, u8, &'static [TextKey]),
}

fn vivos(pares: &[(NodeId, IconId)]) -> Peca {
    Peca::Icones(
        pares
            .iter()
            .map(|&(id, icon)| Icone {
                id: Some(id),
                icon,
                on: false,
            })
            .collect(),
    )
}

/// Os grupos, na ordem da barra de antes: um grupo só se parte numa coluna mais estreita do que ele.
fn grupos(snap: &FlipStripSnapshot) -> [Vec<Peca>; 6] {
    let t = |k: &str| tr(&format!("panel.flip_frames.toolbar.{k}"));
    let play = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    [
        vec![
            vivos(&[
                (ids::FLIP_PREV_DRAWING, IconId::SkipBack),
                (ids::FLIP_PLAY, play),
                (ids::FLIP_NEXT_DRAWING, IconId::SkipForward),
            ]),
            Peca::Campo(ids::FLIP_FPS_NUM, t("fps")),
        ],
        vec![
            Peca::Marca(ids::FLIP_GHOST, t("ghost"), snap.ghost),
            Peca::Campo(
                ids::FLIP_GHOST_BEFORE_NUM,
                tr("panel.flip_frames.toolbar.ghost_before"),
            ),
            Peca::Campo(
                ids::FLIP_GHOST_AFTER_NUM,
                tr("panel.flip_frames.toolbar.ghost_after"),
            ),
            Peca::Marca(ids::FLIP_KEY_PIN, t("pin"), snap.current_pinned),
        ],
        vec![
            Peca::Marca(ids::FLIP_AUTOKEY, t("auto"), snap.autokey),
            Peca::Marca(ids::FLIP_FALLOFF, t("falloff"), snap.falloff),
            Peca::Marca(ids::FLIP_ADDITIVE, t("additive"), snap.additive),
        ],
        vec![
            vivos(&[
                (ids::FLIP_KEY_ADD, IconId::Plus),
                (ids::FLIP_KEY_DUP, IconId::Copy),
                (ids::FLIP_KEY_INSTANCE, IconId::Link),
                (ids::FLIP_KEY_UNLINK, IconId::Unlink),
                (ids::FLIP_KEY_DELETE, IconId::Trash),
            ]),
            Peca::Campo(ids::FLIP_HOLD_NUM, t("hold")),
            vivos(&[
                (ids::FLIP_KEY_LEFT, IconId::ChevronLeft),
                (ids::FLIP_KEY_RIGHT, IconId::ChevronRight),
            ]),
        ],
        vec![
            Peca::Campo(ids::FLIP_TWEEN_NUM, t("tween")),
            Peca::Lista(ids::FLIP_TWEEN_EASE_DD, snap.tween_ease, &TWEEN_EASE_NAMES),
            Peca::Marca(ids::FLIP_TWEEN_FADE, t("fade"), snap.tween_fade),
            Peca::Marca(ids::FLIP_TWEEN_PAIRS, t("pairs"), snap.tween_pairs),
            Peca::Verbo(ids::FLIP_TWEEN_ADD, t("add")),
        ],
        vec![Peca::Lista(ids::FLIP_CYCLE_DD, snap.cycle, &CYCLE_NAMES)],
    ]
}

/// O store antes de a fila se pintar com ele emprestado; corre uma vez por quadro.
pub(crate) fn prepara(store: &mut WidgetStore, snap: &FlipStripSnapshot) {
    let hold = crate::toolbar_plan::hold_atual(snap);
    let arrastado = store.number_input_drag().map(|d| d.id);
    for (id, valor, default) in [
        (ids::FLIP_FPS_NUM, f64::from(snap.fps), FPS_DEFAULT),
        (
            ids::FLIP_GHOST_BEFORE_NUM,
            f64::from(snap.ghost_before),
            GHOST_DEFAULT,
        ),
        (
            ids::FLIP_GHOST_AFTER_NUM,
            f64::from(snap.ghost_after),
            GHOST_DEFAULT,
        ),
        (ids::FLIP_HOLD_NUM, f64::from(hold), HOLD_DEFAULT),
        (
            ids::FLIP_TWEEN_NUM,
            f64::from(snap.tween_count),
            TWEEN_DEFAULT,
        ),
    ] {
        store.mark_card_field(
            id,
            CardFieldSpec {
                step: STEP,
                scrub: STEP,
                default: Some(default),
            },
        );
        store.set_number_drag_rate(id, STEP);
        if arrastado != Some(id) && store.focus_id() != Some(id) {
            store.set_number_value(id, valor);
        }
    }
    for (id, cur) in [
        (ids::FLIP_CYCLE_DD, snap.cycle),
        (ids::FLIP_TWEEN_EASE_DD, snap.tween_ease),
    ] {
        store.register_if_absent(
            id,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: Some(usize::from(cur)),
            },
        );
    }
    for (id, chave) in DICAS {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(chave));
        }
    }
}

/// A lista mede-se pela maior opção: a escolhida muda, a caixa não salta.
fn lista_w(c: &mut CardCtx<'_>, nomes: &[TextKey]) -> f32 {
    let maior = nomes
        .iter()
        .map(|k| c.text_system.prefix_width(k.tr(), m::VALUE_PX).ceil())
        .fold(0.0_f32, f32::max);
    maior + m::FIELD_PAD_R * 2.0 + m::ICON_GLYPH
}

/// A coluna partilhada dos cinco rótulos: quando a barra desce, os valores arrancam no mesmo sítio.
fn coluna_dos_numeros(c: &mut CardCtx<'_>) -> f32 {
    let nomes = ["fps", "ghost_before", "ghost_after", "hold", "tween"]
        .map(|k| tr(&format!("panel.flip_frames.toolbar.{k}")));
    c.coluna_de_rotulos(&nomes)
}

fn largura(c: &mut CardCtx<'_>, p: &Peca) -> f32 {
    match p {
        Peca::Icones(v) => banda::icones_w(v.len()),
        Peca::Campo(..) => CardCtx::campo_em_fila_w(coluna_dos_numeros(c)),
        Peca::Marca(_, nome, _) => c.marca_em_fila_w(nome),
        Peca::Verbo(_, nome) => c.botao_w(nome),
        Peca::Lista(_, _, nomes) => lista_w(c, nomes),
    }
}

fn pinta(c: &mut CardCtx<'_>, p: &Peca, w: f32, (x, y): (f32, f32)) -> Option<PendingCycle> {
    let r = Rect::new(x, y, w, m::FIELD_H);
    if censo::ligado() {
        let nome = match p {
            Peca::Campo(_, n) | Peca::Marca(_, n, _) | Peca::Verbo(_, n) => n,
            Peca::Icones(_) | Peca::Lista(..) => "",
        };
        censo::frase((r.x, r.w), r.y, r.y + r.h, nome);
    }
    match p {
        Peca::Icones(v) => c.icones_em_fila(v, x, y),
        Peca::Campo(id, nome) => {
            let col = coluna_dos_numeros(c);
            c.campo_em_fila(*id, nome, Glyph::Text(letra_da_grandeza(nome)), col, (x, y));
        }
        Peca::Marca(id, nome, on) => c.marca_em_fila(*id, nome, *on, x, y),
        Peca::Verbo(id, nome) => c.botao_em(*id, nome, r),
        Peca::Lista(id, cur, nomes) => {
            let aberta = matches!(
                c.store.get(*id),
                Some(InteractiveState::Dropdown { open: true, .. })
            );
            let texto = nomes.get(usize::from(*cur)).map_or("", |k| k.tr());
            c.lista_em(*id, texto, aberta, r);
            return aberta.then_some(PendingCycle { id: *id, chip: r });
        }
    }
    None
}

/// A barra em `cab` (ao lado do título) e, o que não couber, em fileiras a partir de `corpo`. Devolve onde
/// as células começam e a lista aberta, que se pinta por último.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    cab: Rect,
    corpo: Rect,
    snap: &FlipStripSnapshot,
) -> (f32, Option<PendingCycle>) {
    let ink = CardInk::of_list(theme);
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    let mut c = CardCtx::flutuante(
        ctx.scene,
        ctx.text_system,
        store,
        hit,
        ink,
        corpo.x,
        corpo.w,
    );
    let passo = m::FIELD_H + m::FIELD_ROW_GAP;
    let (mut x, mut y, mut fim) = (cab.x, cab.y, cab.x + cab.w);
    let mut inicio = cab.x;
    let mut fileiras = 0_u16;
    let mut aberta = None;
    for grupo in grupos(snap) {
        let larguras: Vec<f32> = grupo.iter().map(|p| largura(&mut c, p)).collect();
        let total =
            larguras.iter().sum::<f32>() + m::FIELD_GAP * larguras.len().saturating_sub(1) as f32;
        let mut desce = |x: &mut f32, y: &mut f32, fim: &mut f32, inicio: &mut f32| {
            fileiras += 1;
            *inicio = corpo.x;
            *fim = corpo.x + corpo.w;
            *x = corpo.x;
            *y = corpo.y + f32::from(fileiras - 1) * passo - banda::SOBRA_DO_CABECALHO;
        };
        if x > inicio && x + total > fim {
            desce(&mut x, &mut y, &mut fim, &mut inicio);
        }
        for (p, w) in grupo.iter().zip(larguras) {
            if x > inicio && x + w > fim {
                desce(&mut x, &mut y, &mut fim, &mut inicio);
            }
            aberta = pinta(&mut c, p, w, (x, y)).or(aberta);
            x += w + m::FIELD_GAP;
        }
        x += Spacing::Sm.px();
    }
    let depois = if fileiras > 0 {
        y + m::FIELD_H + m::FIELD_ROW_GAP
    } else {
        corpo.y
    };
    if let Some(p) = &aberta {
        let _ = troca_popover(Some((p.id, ink)));
    }
    (depois, aberta)
}
