//! ⭐ **A mesa de som em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): Output (tocar,
//! loudness, limitador), Channels (a mesa, que continua a ser faixas verticais com fader e medidor) e um
//! cartão por efeito do master, com os MESMOS ids de secção. Os parâmetros do rack viram campos ligados à
//! barra de sempre, que continua a avisar o painel; o `event.rs` não muda.

use crate::fader::{FADER_UNITY_POS, fader_db};
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_DELAY_FEEDBACK, AMIX_DELAY_MIX, AMIX_DELAY_TIME, AMIX_DUCK_DEPTH,
    AMIX_DUCK_KEY_BUS, AMIX_EQ_HIGH, AMIX_EQ_LOW, AMIX_EQ_MID, AMIX_FADER, AMIX_LOWCUT,
    AMIX_MASTER_METER, AMIX_MASTER_MUTE, AMIX_PAN, AMIX_PANEL, AMIX_PLAY, AMIX_REVERB_MIX,
    AMIX_REVERB_SIZE, AMIX_SEC_CHANNELS, AMIX_SEC_COMP, AMIX_SEC_DELAY, AMIX_SEC_DUCK, AMIX_SEC_EQ,
    AMIX_SEC_OUTPUT, AMIX_SEC_REVERB, SUB_BUS_COUNT, SUB_BUS_LABELS, SUB_COMP, SUB_DELAY_SEND,
    SUB_FADER, SUB_LOWCUT, SUB_METER, SUB_MUTE, SUB_PAN, SUB_SEND, SUB_SOLO, SUB_TONE, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::ferramenta::caixa_em_celula;
use ph2d_editor_core::panel::cards::linhas_extra::CardBar;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget};
use ph2d_editor_core::widget::LevelMeter;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

#[path = "cards_mesa.rs"]
mod mesa;
use mesa::{faixas, mesa};

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (os testes de um processo correm em paralelo).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// O campo do cartão de uma barra do rack: derivado da barra, para os dois nunca se desligarem.
#[must_use]
pub const fn chip_de(barra: NodeId) -> NodeId {
    const SAL: u64 = 0x37A1_90C4_5D2E_8B16;
    NodeId(barra.0 ^ SAL)
}

const PASSO_DA_SETA: f64 = 0.05; // LITERAL-PX-OK: a seta anda 5 % da fracção normalizada (domínio do rack)
const PASSO_DA_PEGA: f64 = 0.005; // LITERAL-PX-OK: a pega anda 0,5 % por px da mão (domínio do rack)

/// As barras do rack que viram campo: a fracção normalizada de sempre, de 0 a 1, com o seu rótulo.
fn barras_do_rack() -> impl Iterator<Item = NodeId> {
    [
        AMIX_EQ_LOW,
        AMIX_EQ_MID,
        AMIX_EQ_HIGH,
        AMIX_REVERB_SIZE,
        AMIX_REVERB_MIX,
        AMIX_DELAY_TIME,
        AMIX_DELAY_FEEDBACK,
        AMIX_DELAY_MIX,
        AMIX_DUCK_DEPTH,
    ]
    .into_iter()
    .chain(SUB_SEND)
    .chain(SUB_DELAY_SEND)
    .chain(SUB_COMP)
}

/// Regista o campo de cada barra do rack (no `populate`, uma vez).
pub(crate) fn populate(store: &mut WidgetStore) {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::widget::TextInputState;
    for barra in barras_do_rack() {
        let chip = chip_de(barra);
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(barra, chip, 1.0, 0.0);
        store.set_number_range(chip, 0.0, 1.0, PASSO_DA_PEGA);
    }
}

/// Antes de pintar: as barras seguem a mesa viva, e cada campo mostra a sua barra, salvo o que a mão leva.
pub(crate) fn prepare(store: &mut WidgetStore) {
    crate::event::espelhar_sliders(store);
    store.mark_card_list_close(AMIX_PANEL, AMIX_CLOSE);
    for id in [
        AMIX_SEC_REVERB,
        AMIX_SEC_DELAY,
        AMIX_SEC_COMP,
        AMIX_SEC_DUCK,
    ] {
        store.set_collapsed_if_unchosen(id, true);
    }
    let arrastado = store.number_input_drag().map(|d| d.id);
    for barra in barras_do_rack() {
        let chip = chip_de(barra);
        store.mark_card_field(
            chip,
            CardFieldSpec {
                step: PASSO_DA_SETA,
                scrub: PASSO_DA_PEGA,
                default: None,
            },
        );
        if arrastado != Some(chip)
            && let Some((_, v)) = store.slider(barra)
        {
            store.set_number_value(chip, f64::from(v));
        }
    }
}

/// O campo de uma barra do rack. O rótulo diz o que é; a marca de animação é a da barra (`marcas_mesa`).
fn campo(barra: &'static NodeId, rotulo: &'static str) -> Cell<'static> {
    Cell::new(Widget::Field {
        id: chip_de(*barra),
        glyph: Glyph::Text(letra_da_grandeza(rotulo)),
    })
    .label(rotulo)
    .key(std::slice::from_ref(barra))
}

fn aos_pares(c: &mut CardCtx<'_>, cells: &[Cell<'static>], y: f32) -> f32 {
    cells.chunks(2).fold(y, |y, par| c.row(par, y))
}

fn cartao(
    section: NodeId,
    title: &'static str,
    summary: Option<String>,
    body: impl Fn(&mut CardCtx<'_>, f32) -> f32 + 'static,
) -> CardDef<'static> {
    CardDef {
        section,
        title: tr(title),
        actions: Vec::new(),
        add: None,
        summary,
        body: Some(Box::new(body)),
    }
}

fn liga(on: bool) -> Option<String> {
    Some(
        tr(if on {
            "card.common.on"
        } else {
            "card.common.off"
        })
        .to_string(),
    )
}

/// ⭐ Os cartões da mesa, pela ordem natural.
pub(crate) fn cards(store: &WidgetStore) -> Vec<CardDef<'static>> {
    let faixas = faixas(store);
    let somados = (
        snapshot::reverb_on(),
        snapshot::delay_on(),
        snapshot::ducking(),
    );
    vec![
        cartao(
            AMIX_SEC_OUTPUT,
            "card.audio_mixer.output",
            None,
            cartao_de_saida,
        ),
        cartao(
            AMIX_SEC_CHANNELS,
            "card.audio_mixer.channels",
            None,
            move |c, y| mesa(c, &faixas, y),
        ),
        cartao(
            AMIX_SEC_EQ,
            "panel.audio_mixer.master.eq",
            None,
            cartao_de_eq,
        ),
        cartao(
            AMIX_SEC_REVERB,
            "panel.audio_mixer.master.reverb",
            liga(somados.0),
            cartao_de_reverb,
        ),
        cartao(
            AMIX_SEC_DELAY,
            "panel.audio_mixer.master.delay",
            liga(somados.1),
            cartao_de_delay,
        ),
        cartao(
            AMIX_SEC_COMP,
            "panel.audio_mixer.master.comp",
            None,
            cartao_de_comp,
        ),
        cartao(
            AMIX_SEC_DUCK,
            "panel.audio_mixer.master.ducking",
            liga(somados.2),
            cartao_de_ducking,
        ),
    ]
}

const SILENCIO_LUFS: f32 = -70.0; // LITERAL-PX-OK: abaixo disto a loudness lê-se «silêncio» (domínio do áudio)

fn cartao_de_saida(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let tocando = snapshot::play_test();
    let y = c.full(
        None,
        Widget::Button {
            id: AMIX_PLAY,
            text: tr(if tocando {
                "panel.audio_mixer.master.stop"
            } else {
                "panel.audio_mixer.master.play_test"
            }),
            primary: tocando,
        },
        y,
    );
    let lufs = snapshot::loudness();
    let valor = if lufs <= SILENCIO_LUFS {
        tr("card.audio_mixer.silent").to_string()
    } else {
        format!("{lufs:.1}")
    };
    c.pair(
        caixa_em_celula(
            crate::AMIX_LIMITER,
            tr("panel.audio_mixer.master.limiter"),
            snapshot::limiter(),
        ),
        Cell::new(Widget::Readout { text: &valor }).label(tr("card.audio_mixer.loudness")),
        y,
    )
}

fn cartao_de_eq(c: &mut CardCtx<'_>, y: f32) -> f32 {
    c.row(
        &[
            campo(&AMIX_EQ_LOW, tr("panel.audio_mixer.master.low")),
            campo(&AMIX_EQ_MID, tr("panel.audio_mixer.master.mid")),
            campo(&AMIX_EQ_HIGH, tr("panel.audio_mixer.master.high")),
        ],
        y,
    )
}

/// As quatro linhas de envio de um efeito: o nome do barramento diz para quem é o número.
fn por_barramento(ids: &'static [NodeId; SUB_BUS_COUNT]) -> Vec<Cell<'static>> {
    (0..SUB_BUS_COUNT)
        .map(|i| campo(&ids[i], SUB_BUS_LABELS[i].tr()))
        .collect()
}

fn cartao_de_reverb(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let y = c.check(
        crate::AMIX_REVERB,
        tr("card.common.enabled"),
        snapshot::reverb_on(),
        y,
    );
    let y = c.row(
        &[
            campo(&AMIX_REVERB_SIZE, tr("panel.audio_mixer.master.size")),
            campo(&AMIX_REVERB_MIX, tr("panel.audio_mixer.master.return")),
        ],
        y,
    );
    let y = c.hint(tr("card.audio_mixer.sends_hint"), y);
    aos_pares(c, &por_barramento(&SUB_SEND), y)
}

fn cartao_de_delay(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let y = c.check(
        crate::AMIX_DELAY,
        tr("card.common.enabled"),
        snapshot::delay_on(),
        y,
    );
    let y = c.row(
        &[
            campo(&AMIX_DELAY_TIME, tr("panel.audio_mixer.master.time")),
            campo(&AMIX_DELAY_FEEDBACK, tr("panel.audio_mixer.master.fbk")),
            campo(&AMIX_DELAY_MIX, tr("panel.audio_mixer.master.return")),
        ],
        y,
    );
    let y = c.hint(tr("card.audio_mixer.sends_hint"), y);
    aos_pares(c, &por_barramento(&SUB_DELAY_SEND), y)
}

fn cartao_de_comp(c: &mut CardCtx<'_>, y: f32) -> f32 {
    aos_pares(c, &por_barramento(&SUB_COMP), y)
}

fn cartao_de_ducking(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let y = c.pair(
        caixa_em_celula(
            crate::AMIX_DUCK,
            tr("card.common.enabled"),
            snapshot::ducking(),
        ),
        campo(&AMIX_DUCK_DEPTH, tr("panel.audio_mixer.master.depth")),
        y,
    );
    let chave = snapshot::ducking_key();
    let segs: Vec<Seg<'static>> = (0..SUB_BUS_COUNT)
        .map(|i| Seg {
            id: AMIX_DUCK_KEY_BUS[i],
            icon: None,
            text: SUB_BUS_LABELS[i].tr(),
            on: i == chave,
        })
        .collect();
    c.full(
        Some(tr("panel.audio_mixer.master.key")),
        Widget::Choice(&segs),
        y,
    )
}
