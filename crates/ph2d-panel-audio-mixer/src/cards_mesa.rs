//! A mesa do cartão Channels: faixas verticais com fader, medidor, Mute e Solo.

use super::*;

/// Uma faixa da mesa: o instantâneo do quadro. As posições vêm do store, escritas pelo despacho.
#[derive(Clone)]
pub(super) struct Faixa {
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

pub(super) fn faixas(store: &WidgetStore) -> Vec<Faixa> {
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
pub(super) fn mesa(c: &mut CardCtx<'_>, faixas: &[Faixa], top: f32) -> f32 {
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
