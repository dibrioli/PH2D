//! ⭐ Os controlos que AGEM sobre o objeto fictício (DIRETIVA_IMPLEMENTACAO §2: zero no-op silencioso
//! — a galeria é o modelo que todo painel copia), pela tabela [`crate::controlos`]: interruptores,
//! escolhas, «+»/«−» das listas, a amostra que abre o seletor de cor do app, as ferramentas de um
//! clique (girar, espelhar, alinhar), o Apply que grava os valores de PARTIDA e o Cancel e o ↺ que
//! voltam a eles.

use crate::controlos::avancado::{META_ROWS, META_TYPES};
use crate::controlos::{self, Act, Kind, SWATCHES};
use crate::ids as g;
use crate::state::{BOOLEAN_NONE, Choices, Pick, SIDE_NONE, Values, WidgetGalleryState};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::par_travado::ParTravado;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_tokens::Theme;

/// A moldura fictícia onde o objeto se alinha e os dois vizinhos fictícios que o distribuir iguala.
const FRAME_W: f64 = 400.0; // LITERAL-PX-OK: dado do objeto fictício
const FRAME_H: f64 = 300.0; // LITERAL-PX-OK: dado do objeto fictício
const NEIGHBOURS: (f64, f64) = (40.0, 200.0); // LITERAL-PX-OK: dado do objeto fictício
const QUARTER_TURN: f64 = 90.0; // LITERAL-PX-OK: graus, dado do objeto fictício
const FULL_TURN: f64 = 360.0; // LITERAL-PX-OK: graus, dado do objeto fictício

/// O retrato dos valores vivos: as escolhas, os números e os textos no store.
pub(crate) fn current(choices: Choices, store: &WidgetStore) -> Values {
    Values {
        choices,
        numbers: g::FIELDS.map(|f| store.number_value(f.0).unwrap_or(f.1)),
        texts: g::TEXTS.map(|(id, _)| store.text(id).unwrap_or_default().to_owned()),
    }
}

/// A tinta do cartão da amostra `i` — a do tema PRÓPRIO dele, se o tem; senão a da lista.
fn swatch_ink(store: &WidgetStore, i: usize, list: Theme) -> CardInk {
    CardInk::of_card(store.section_theme(SWATCHES[i].0), list)
}

/// Repõe um texto (caret no fim, sem selecção); o `populate` registou-o no 1.º quadro.
fn set_text(store: &mut WidgetStore, id: NodeId, value: &str) {
    if let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        ..
    }) = store.get_mut(id)
    {
        value.clone_into(text);
        *caret = text.len();
        *selection_anchor = None;
    }
}

/// A cor que a amostra `i` mostra: a escolhida, ou o token de fábrica na tinta `ink`.
pub(crate) fn rgba(ch: &Choices, i: usize, ink: CardInk) -> [u8; 4] {
    ch.colors[i].unwrap_or_else(|| ink.color(SWATCHES[i].2).to_rgba8().to_u8_array())
}

/// ⭐ Os dois regimes da amostra (a lei das amostras do Inspector): com o seletor apontado para
/// ELA, a cor que ele escreve no `widget_color` é a dela; senão, nada. Corre por quadro, antes de
/// pintar.
pub(crate) fn sync_picked(state: &mut WidgetGalleryState, store: &WidgetStore, theme: Theme) {
    let Some(alvo) = store.picker_target() else {
        return;
    };
    let Some(i) = SWATCHES.iter().position(|s| s.1 == alvo) else {
        return;
    };
    if let Some(c) = store.widget_color(alvo)
        && c != rgba(&state.choices, i, swatch_ink(store, i, theme))
    {
        state.choices.colors[i] = Some(c);
    }
}

/// ⭐ A corrente do Scale: com ela fechada, mudar um eixo leva o outro na proporção guardada (a lei é
/// a do Inspector, [`ParTravado`]). Corre por quadro, antes de pintar: o arrasto escreve no store.
pub(crate) fn sync_link(state: &mut WidgetGalleryState, store: &mut WidgetStore) {
    use crate::state::Flag;
    if !state.choices.on(Flag::ScaleLink) {
        state.link = None;
        return;
    }
    let par = scale_vivo(store);
    let trava = state.link.get_or_insert_with(|| ParTravado::ao_ligar(par));
    let mudou = |a: f64, b: f64| (a - b).abs() > f64::EPSILON;
    let visto = trava.visto();
    if mudou(visto[0], par[0]) {
        store.set_number_value(g::SCALE_Y, trava.segue(0, par[0]));
    } else if mudou(visto[1], par[1]) {
        store.set_number_value(g::SCALE_X, trava.segue(1, par[1]));
    }
}

fn scale_vivo(store: &WidgetStore) -> [f64; 2] {
    [g::SCALE_X, g::SCALE_Y].map(|id| store.number_value(id).unwrap_or(1.0))
}

/// Sem «Overheal», o `Start` não passa do `Max` (por quadro, antes de pintar).
pub(crate) fn sync_health(state: &WidgetGalleryState, store: &mut WidgetStore) {
    use crate::state::Flag;
    let (max, start) = (number(store, g::HP_MAX), number(store, g::HP_START));
    if !state.choices.on(Flag::HpOverheal) && start > max {
        store.set_number_value(g::HP_START, max);
    }
}

/// Trata o clique `id` pela tabela; devolve se é um controlo da galeria.
pub(crate) fn on_click(
    state: &mut WidgetGalleryState,
    host: &mut dyn PanelHostInternal,
    id: NodeId,
) -> bool {
    let Some(c) = controlos::find(id) else {
        return false;
    };
    let ch = &mut state.choices;
    match c.kind {
        Kind::Toggle(f) | Kind::Eye(f) => ch.set(f, !ch.on(f)),
        Kind::Pick(p, i) => {
            ch.choose(p, i);
            if matches!(p, Pick::ExportScale | Pick::Format) {
                state.exported = false;
            }
            // O que sai muda: a corrida (ou o resultado dela) deixa de valer.
            if matches!(p, Pick::RenderFormat | Pick::RenderQuality) {
                state.render.cancel();
            }
        }
        // A caixa da lista e o ⠿ abrem pelo despacho dos cartões; a galeria só consome o clique.
        Kind::Box | Kind::Styles => {}
        Kind::AddOf(rows, picks, v) => {
            if let Some(i) = rows.iter().position(|f| !ch.on(*f)) {
                ch.set(rows[i], true);
                ch.choose(picks[i], v);
            }
        }
        Kind::Expand(p, i) => ch.choose(p, if ch.pick(p) == i { SIDE_NONE } else { i }),
        Kind::Palette(i) => {
            let alvo = if ch.pick(Pick::PixelTarget) == 0 {
                Pick::PixelFg
            } else {
                Pick::PixelBg
            };
            ch.choose(alvo, i);
        }
        Kind::Add(rows) => {
            if let Some(livre) = rows.iter().find(|f| !ch.on(**f)) {
                ch.set(*livre, true);
            }
        }
        Kind::Remove(f, swatch) => {
            ch.set(f, false);
            // As linhas sem cor (regras, rack) não têm amostra (`SEM_AMOSTRA`).
            if let Some(s) = SWATCHES.get(swatch) {
                ch.linked[swatch] = None;
                close_picker(host.store_mut(), s.1);
            }
        }
        Kind::Set(f, v) => ch.set(f, v),
        Kind::Layer(bit) => ch.layers ^= 1 << bit,
        Kind::AddMeta(v) => {
            if let Some(i) = META_ROWS.iter().position(|f| !ch.on(*f)) {
                ch.set(META_ROWS[i], true);
                ch.choose(META_TYPES[i], v);
            }
        }
        Kind::Swatch(i) => toggle_picker(*ch, host, i),
        Kind::Act(a) => act(state, host, a),
    }
    // Quem escreveu no Scale não arrasta o outro eixo: o que ele deixou passa a ser o visto.
    let par = scale_vivo(host.store());
    if let Some(trava) = &mut state.link {
        trava.rever(par);
    }
    true
}

fn number(store: &WidgetStore, id: NodeId) -> f64 {
    store.number_value(id).unwrap_or_default()
}

/// As ferramentas de um clique.
fn act(state: &mut WidgetGalleryState, host: &mut dyn PanelHostInternal, a: Act) {
    let store = host.store_mut();
    let (w, h) = (number(store, g::SIZE_W), number(store, g::SIZE_H));
    let between = (NEIGHBOURS.0 + NEIGHBOURS.1 - w) * 0.5;
    let between_v = (NEIGHBOURS.0 + NEIGHBOURS.1 - h) * 0.5;
    match a {
        Act::AlignLeft => store.set_number_value(g::POS_X, 0.0),
        Act::AlignHCenter => store.set_number_value(g::POS_X, (FRAME_W - w) * 0.5),
        Act::AlignRight => store.set_number_value(g::POS_X, FRAME_W - w),
        Act::AlignTop => store.set_number_value(g::POS_Y, 0.0),
        Act::AlignVCenter => store.set_number_value(g::POS_Y, (FRAME_H - h) * 0.5),
        Act::AlignBottom => store.set_number_value(g::POS_Y, FRAME_H - h),
        Act::DistributeH => store.set_number_value(g::POS_X, between),
        Act::DistributeV => store.set_number_value(g::POS_Y, between_v),
        Act::Flatten => state.choices.choose(Pick::Boolean, BOOLEAN_NONE),
        Act::TransformReset => {
            for (id, factory, ..) in g::FIELDS {
                if TRANSFORM.contains(&id) {
                    store.set_number_value(id, factory);
                }
            }
        }
        Act::Rotate90 => {
            let v = (number(store, g::ROTATION) + QUARTER_TURN) % FULL_TURN;
            store.set_number_value(g::ROTATION, v);
        }
        Act::FlipH | Act::FlipV => {
            let (campo, eixo) = if matches!(a, Act::FlipH) {
                (g::SCALE_X, 0)
            } else {
                (g::SCALE_Y, 1)
            };
            store.set_number_value(campo, -number(store, campo));
            if let Some(trava) = &mut state.link {
                trava.espelhou(eixo);
            }
        }
        Act::Export => state.exported = true,
        Act::RenderStart => state.render.start(),
        Act::RenderCancel => state.render.cancel(),
        Act::PickTarget(f, campo) => {
            let armado = state.choices.on(f);
            state.choices.set(f, !armado);
            if armado {
                set_text(
                    store,
                    campo,
                    ph2d_i18n::tr("panel.widget_gallery.real.object.coin"),
                );
            }
        }
        Act::Fire(i) => {
            state.fires[i] += 1;
            state.fired_at[i] = Some(std::time::Instant::now());
        }
        Act::PixelSwap => {
            let (fg, bg) = (
                state.choices.pick(Pick::PixelFg),
                state.choices.pick(Pick::PixelBg),
            );
            state.choices.choose(Pick::PixelFg, bg);
            state.choices.choose(Pick::PixelBg, fg);
        }
        Act::Apply => state.applied = current(state.choices, store),
        Act::Restore => {
            for (_, swatch, _) in SWATCHES {
                close_picker(store, swatch);
            }
            state.choices = state.applied.choices;
            for ((field, ..), n) in g::FIELDS.iter().zip(state.applied.numbers) {
                store.set_number_value(*field, n);
            }
            for ((id, _), text) in g::TEXTS.iter().zip(&state.applied.texts) {
                set_text(store, *id, text);
            }
        }
    }
}

/// O que o ↺ do cartão Transform repõe.
const TRANSFORM: [NodeId; 7] = [
    g::POS_X,
    g::POS_Y,
    g::ROTATION,
    g::SCALE_X,
    g::SCALE_Y,
    g::SKEW_X,
    g::SKEW_Y,
];

/// Abre o seletor de cor do app apontado para a amostra `i`, semeado com a cor que ela mostra — ou
/// fecha-o, se já era dela (o padrão das amostras do Painter).
fn toggle_picker(ch: Choices, host: &mut dyn PanelHostInternal, i: usize) {
    let id = SWATCHES[i].1;
    if host.store().picker_target() == Some(id) {
        host.store_mut().set_picker_target(None);
        return;
    }
    let ink = swatch_ink(host.store(), i, host.theme());
    let [r, gr, b, a] = rgba(&ch, i, ink);
    let store = host.store_mut();
    store.set_blender_value(
        core_ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(r, gr, b, a),
    );
    store.set_widget_color(id, [r, gr, b, a]);
    store.set_picker_target(Some(id));
}

/// O seletor aberto para uma amostra que some (ou cuja cor volta atrás) fecha-se: senão o quadro
/// seguinte reescreveria a cor escolhida por cima da reposta.
fn close_picker(store: &mut WidgetStore, swatch: NodeId) {
    if store.picker_target() == Some(swatch) {
        store.set_picker_target(None);
    }
}
