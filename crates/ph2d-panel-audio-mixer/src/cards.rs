//! ⭐ **A mesa de som em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): Output (tocar,
//! loudness, limitador), Channels (a mesa, que continua a ser faixas verticais com fader e medidor) e um
//! cartão por efeito do master, com os MESMOS ids de secção. Os parâmetros do rack viram campos ligados à
//! barra de sempre, que continua a avisar o painel; o `event.rs` não muda.

use crate::fader::{FADER_UNITY_POS, fader_db};
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_DELAY, AMIX_DELAY_FEEDBACK, AMIX_DELAY_MIX, AMIX_DELAY_TIME,
    AMIX_DUCK, AMIX_DUCK_DEPTH, AMIX_DUCK_KEY_BUS, AMIX_EQ_HIGH, AMIX_EQ_LOW, AMIX_EQ_MID,
    AMIX_FADER, AMIX_LIMITER, AMIX_LOWCUT, AMIX_MASTER_METER, AMIX_MASTER_MUTE, AMIX_PAN,
    AMIX_PANEL, AMIX_PLAY, AMIX_REVERB, AMIX_REVERB_MIX, AMIX_REVERB_SIZE, AMIX_SEC_CHANNELS,
    AMIX_SEC_COMP, AMIX_SEC_DELAY, AMIX_SEC_DUCK, AMIX_SEC_EQ, AMIX_SEC_OUTPUT, AMIX_SEC_REVERB,
    SUB_BUS_COUNT, SUB_BUS_LABELS, SUB_COMP, SUB_DELAY_SEND, SUB_FADER, SUB_LOWCUT, SUB_METER,
    SUB_MUTE, SUB_PAN, SUB_SEND, SUB_SOLO, SUB_TONE, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::linhas_extra::CardBar;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget};
use ph2d_editor_core::widget::LevelMeter;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

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
    let mut y = c.full(
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
    y = c.check(
        crate::AMIX_LIMITER,
        tr("panel.audio_mixer.master.limiter"),
        snapshot::limiter(),
        y,
    );
    let lufs = snapshot::loudness();
    let valor = if lufs <= SILENCIO_LUFS {
        tr("card.audio_mixer.silent").to_string()
    } else {
        format!("{lufs:.1}")
    };
    c.row(
        &[Cell::new(Widget::Readout { text: &valor }).label(tr("card.audio_mixer.loudness"))],
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
        tr("panel.audio_mixer.master.reverb"),
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
        tr("panel.audio_mixer.master.delay"),
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
    let y = c.check(
        crate::AMIX_DUCK,
        tr("panel.audio_mixer.master.ducking"),
        snapshot::ducking(),
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
    let y = c.full(
        Some(tr("panel.audio_mixer.master.key")),
        Widget::Choice(&segs),
        y,
    );
    c.row(
        &[campo(
            &AMIX_DUCK_DEPTH,
            tr("panel.audio_mixer.master.depth"),
        )],
        y,
    )
}

/// Uma faixa da mesa: o instantâneo do quadro. As posições vêm do store, escritas pelo despacho.
#[derive(Clone)]
struct Faixa {
    nome: &'static str,
    fader: NodeId,
    pan: NodeId,
    tone: NodeId,
    lowcut: NodeId,
    mute: NodeId,
    solo: Option<NodeId>,
    meter: NodeId,
    valor: [f32; 4],
    mudo: bool,
    solado: bool,
    rms: [f32; 2],
    pico: [f32; 2],
    clip: [bool; 2],
}

fn faixas(store: &WidgetStore) -> Vec<Faixa> {
    let le = |id, omissao| store.slider(id).map_or(omissao, |(_, v)| v);
    let mut out = Vec::with_capacity(1 + SUB_BUS_COUNT);
    out.push(Faixa {
        nome: tr("panel.audio_mixer.strip.master"),
        fader: AMIX_FADER,
        pan: AMIX_PAN,
        tone: AMIX_CUTOFF,
        lowcut: AMIX_LOWCUT,
        mute: AMIX_MASTER_MUTE,
        solo: None,
        meter: AMIX_MASTER_METER,
        valor: [
            le(AMIX_PAN, 0.5),
            le(AMIX_CUTOFF, 1.0),
            le(AMIX_LOWCUT, 0.0),
            le(AMIX_FADER, FADER_UNITY_POS),
        ],
        mudo: snapshot::muted(),
        solado: false,
        rms: snapshot::master_rms(),
        pico: snapshot::tick_master_hold(),
        clip: snapshot::master_clipped(),
    });
    let (rms, mudos, solos, clips) = (
        snapshot::sub_rms(),
        snapshot::sub_muted(),
        snapshot::sub_soloed(),
        snapshot::sub_clipped(),
    );
    for i in 0..SUB_BUS_COUNT {
        out.push(Faixa {
            nome: SUB_BUS_LABELS[i].tr(),
            fader: SUB_FADER[i],
            pan: SUB_PAN[i],
            tone: SUB_TONE[i],
            lowcut: SUB_LOWCUT[i],
            mute: SUB_MUTE[i],
            solo: Some(SUB_SOLO[i]),
            meter: SUB_METER[i],
            valor: [
                le(SUB_PAN[i], 0.5),
                le(SUB_TONE[i], 1.0),
                le(SUB_LOWCUT[i], 0.0),
                le(SUB_FADER[i], FADER_UNITY_POS),
            ],
            mudo: mudos[i],
            solado: solos[i],
            rms: rms[i],
            pico: snapshot::tick_sub_hold(i),
            clip: clips[i],
        });
    }
    out
}

/// O rótulo de uma fila da mesa, em cima dela, a toda a largura (rótulo EM CIMA, 05 §4). Devolve o `y`.
fn rotulo_da_fila(c: &mut CardCtx<'_>, texto: &str, y: f32) -> f32 {
    ph2d_editor_core::paint::paint_text_elided(
        c.text_system,
        c.scene,
        texto,
        c.x,
        y,
        m::LABEL_PX,
        c.w,
        c.ink.color(ColorToken::CardLabel),
    );
    y + m::LABEL_PX + m::LABEL_BOTTOM
}

/// Centrado sem o respiro das bordas: a coluna da faixa é estreita e «Master» tem de caber nela inteiro.
fn centrado(c: &mut CardCtx<'_>, texto: &str, r: Rect, px: f32, cor: ColorToken) {
    let tinta = c.ink.color(cor);
    let tw = c.text_system.prefix_width(texto, px).min(r.w);
    ph2d_editor_core::paint::paint_text_elided(
        c.text_system,
        c.scene,
        texto,
        r.x + (r.w - tw) * 0.5,
        r.y,
        px,
        r.w,
        tinta,
    );
}

/// ⭐ **A mesa**: faixas verticais lado a lado à largura inteira do cartão, para o nome caber. Em baixo,
/// Pan, Tone e Low Cut: eram barras mudas (o nome estava no id, não na tela); aqui cada fila diz o que é.
fn mesa(c: &mut CardCtx<'_>, faixas: &[Faixa], top: f32) -> f32 {
    let n = faixas.len() as f32;
    let vao = m::FIELD_GAP;
    let coluna = (c.w - vao * (n - 1.0)) / n;
    let x0 = c.x;
    let x_de = |i: usize| x0 + i as f32 * (coluna + vao);
    let mut y = top + m::LABEL_TOP;
    for (i, f) in faixas.iter().enumerate() {
        let r = Rect::new(x_de(i), y, coluna, m::LABEL_PX);
        centrado(c, f.nome, r, m::LABEL_PX, ColorToken::Text1);
    }
    y += m::LABEL_PX + m::FIELD_ROW_GAP;
    // Na coluna estreita fader e medidor encolhem na mesma proporção: a faixa nunca invade a vizinha.
    let inteiro = m::MIXER_FADER_W + m::GRID_GAP + m::MIXER_METER_W;
    let escala = (coluna / inteiro).min(1.0);
    let (fader_w, meter_w) = (m::MIXER_FADER_W * escala, m::MIXER_METER_W * escala);
    let conjunto = inteiro * escala;
    for (i, f) in faixas.iter().enumerate() {
        let cx = x_de(i) + (coluna - conjunto) * 0.5;
        c.bar(
            &CardBar {
                id: f.fader,
                value: f.valor[3],
                vertical: true,
                ticks: &[FADER_UNITY_POS],
            },
            Rect::new(cx, y, fader_w, m::MIXER_STRIP_H),
        );
        let medidor = LevelMeter::new(f.fader, f.nome)
            .rms(f.rms[0], f.rms[1])
            .peak_hold(f.pico[0], f.pico[1])
            .clipped(f.clip[0], f.clip[1]);
        c.meter(
            f.meter,
            &medidor,
            Rect::new(
                cx + fader_w + m::GRID_GAP * escala,
                y,
                meter_w,
                m::MIXER_STRIP_H,
            ),
        );
    }
    y += m::MIXER_STRIP_H + m::FIELD_ROW_GAP;
    y = rotulo_da_fila(c, tr("card.audio_mixer.gain_db"), y);
    for (i, f) in faixas.iter().enumerate() {
        let db = if f.valor[3] <= 0.0 {
            tr("card.audio_mixer.silent").to_string()
        } else {
            format!("{:.0}", fader_db(f.valor[3]))
        };
        let r = Rect::new(x_de(i), y, coluna, m::LABEL_PX);
        centrado(c, &db, r, m::LABEL_PX, ColorToken::Text2);
    }
    y += m::LABEL_PX + m::FIELD_ROW_GAP;
    y = botoes(c, faixas, y, x0, coluna, vao);
    let filas = [
        tr("card.audio_mixer.pan"),
        tr("card.audio_mixer.tone"),
        tr("card.audio_mixer.low_cut"),
    ];
    for (fila, rotulo) in filas.iter().enumerate() {
        y = rotulo_da_fila(c, rotulo, y + m::LABEL_TOP);
        for (i, f) in faixas.iter().enumerate() {
            c.bar(
                &CardBar {
                    id: [f.pan, f.tone, f.lowcut][fila],
                    value: f.valor[fila],
                    vertical: false,
                    ticks: &[],
                },
                Rect::new(x_de(i), y, coluna, m::MIXER_BAR_H),
            );
        }
        y += m::MIXER_BAR_H;
    }
    y
}

/// `Mute` no Master; `M | S` nas outras. O par fica lado a lado enquanto a letra cabe; numa coluna que
/// não o aguenta empilha, para o botão nunca sair vazio (a lei da mesa de antes, `StripRow::ms_linhas`).
fn botoes(c: &mut CardCtx<'_>, faixas: &[Faixa], y: f32, x0: f32, coluna: f32, vao: f32) -> f32 {
    let curto = tr("panel.audio_mixer.strip.mute_short");
    let longo = tr("panel.audio_mixer.strip.mute");
    let solo = tr("panel.audio_mixer.strip.solo_short");
    let cabe = c.text_system.prefix_width(longo, m::SEG_TEXT_PX) + m::SEG_INSET * 2.0 <= coluna;
    let letra = c
        .text_system
        .prefix_width(curto, m::SEG_TEXT_PX)
        .max(c.text_system.prefix_width(solo, m::SEG_TEXT_PX));
    let meia = (coluna - m::SEG_PAD) * 0.5;
    let empilha = meia < letra + m::SEG_PAD * 2.0;
    for (i, f) in faixas.iter().enumerate() {
        let x = x0 + i as f32 * (coluna + vao);
        match f.solo {
            None => c.tone_button(
                Rect::new(x, y, coluna, m::FIELD_H),
                f.mute,
                if cabe { longo } else { curto },
                f.mudo,
                ColorToken::Danger,
            ),
            Some(id_solo) => {
                let (r_m, r_s) = if empilha {
                    (
                        Rect::new(x, y, coluna, m::FIELD_H),
                        Rect::new(x, y + m::FIELD_H + m::SEG_PAD, coluna, m::FIELD_H),
                    )
                } else {
                    (
                        Rect::new(x, y, meia, m::FIELD_H),
                        Rect::new(x + meia + m::SEG_PAD, y, meia, m::FIELD_H),
                    )
                };
                c.tone_button(r_m, f.mute, curto, f.mudo, ColorToken::Danger);
                c.tone_button(r_s, id_solo, solo, f.solado, ColorToken::Warn);
            }
        }
    }
    y + m::FIELD_H
        + if empilha {
            m::FIELD_H + m::SEG_PAD
        } else {
            0.0
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
    use ph2d_editor_core::widget::card_ink::CardInk;

    /// Numa coluna estreita a mesa encolhe e empilha `M`/`S`: nada sai do cartão nem invade a vizinha;
    /// na larga o par fica lado a lado. *Mutação: tirar o `escala` ou o `empilha`.*
    #[test]
    fn the_desk_fits_any_card_width_and_stacks_the_pair_when_it_must() {
        let medir = |w: f32| {
            let mut scene = ph2d_vector::VectorScene::new();
            let mut ts = ph2d_text::TextSystem::without_system_fonts();
            let store = WidgetStore::default();
            let mut hits = HitIndex::default();
            let mut c = CardCtx::flutuante(
                &mut scene,
                &mut ts,
                &store,
                &mut hits,
                CardInk::of_card(None, ph2d_tokens::Theme::Dark),
                0.0,
                w,
            );
            let lista = faixas(&store);
            let _ = mesa(&mut c, &lista, 0.0);
            hits.iter_registrations().collect::<Vec<_>>()
        };
        for w in [150.0, 174.0, 270.0, 420.0] {
            let rects = medir(w);
            for (id, r) in &rects {
                assert!(
                    r.x >= -0.01 && r.x + r.w <= w + 0.01,
                    "{id:?} sai em {w}: {r:?}"
                );
            }
            let de = |id| {
                rects
                    .iter()
                    .find(|(n, _)| *n == id)
                    .map(|(_, r)| *r)
                    .unwrap()
            };
            let (fader, medidor) = (de(SUB_FADER[0]), de(SUB_METER[0]));
            assert!(
                fader.x + fader.w <= medidor.x + 0.01,
                "fader sobre o medidor em {w}"
            );
            let vizinho = de(SUB_FADER[1]);
            assert!(
                medidor.x + medidor.w <= vizinho.x + 0.01,
                "invadiu a vizinha em {w}"
            );
        }
        let solo_y = |w| {
            let rects = medir(w);
            let de = |id| {
                rects
                    .iter()
                    .find(|(n, _)| *n == id)
                    .map(|(_, r)| *r)
                    .unwrap()
            };
            (de(SUB_MUTE[0]).y, de(SUB_SOLO[0]).y)
        };
        let (m_largo, s_largo) = solo_y(420.0);
        assert!(
            (m_largo - s_largo).abs() < 0.01,
            "na larga fica lado a lado"
        );
        let (m_estreito, s_estreito) = solo_y(150.0);
        assert!(s_estreito > m_estreito, "na estreita empilha");
    }
}
