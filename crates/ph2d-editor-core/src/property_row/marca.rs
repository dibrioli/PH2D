//! ⭐⭐⭐ **As linhas de MARCAR** (saíram de `property_row.rs`, tecto de 600 linhas). Dentro de um
//! cartão (T29/T64, dono 02/11: «Caixa à Figma») a linha é a do cartão — quadradinho com ✓ e o
//! nome à direita ([`crate::panel::cards::corpo::check`]); fora, a de sempre.

use super::{Seccao, property_row_fits};
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::key_mark::Rotulo;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **UMA LINHA DE MARCAR, pela porta** — o nome na coluna da secção, a caixa na do controlo.
///
/// ⛔⛔ Ela existe porque a linha de marcar é uma **linha de propriedade** (spec §6-quinquies) e
/// até 2026-09-15 cada painel montava a dele à mão: o rect, o `register`, o `checkbox_visual`, o
/// `bool → CheckboxValue`. São quatro passos, e **esquecer o terceiro deixa a caixa sem hover sem
/// nada deixar de compilar** — a família que o [`crate::widget::Checkbox::visual`] já nomeia.
///
/// ⚠️ **O valor é um `bool` do MODELO, nunca do store** — a lei que a §11 do Inspector escreveu
/// para a caixa *Playing*: ler o visual faria o primeiro clique depois de trocar de objecto mandar
/// o valor do objecto **anterior**.
#[allow(clippy::too_many_arguments)]
pub fn paint_check_row<'a>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    caixa: (NodeId, impl Into<Rotulo<'a>>, bool),
    seccao: Seccao,
) -> f32 {
    let (id, label, on) = caixa;
    let label = label.into();
    if let Some((ink, rows)) = super::cartao::card_rows() {
        let valor = if on {
            crate::widget::CheckboxValue::Checked
        } else {
            crate::widget::CheckboxValue::Unchecked
        };
        return (rows.check)(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            x,
            w,
            y,
            id,
            label,
            valor,
            false,
        );
    }
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let cb = crate::widget::Checkbox::new(id, label.texto)
        .coluna_de_fora()
        .visual(store.checkbox_visual(id))
        .value(if on {
            crate::widget::CheckboxValue::Checked
        } else {
            crate::widget::CheckboxValue::Unchecked
        })
        .seccao(seccao.com_o_nome(text_system, label.texto));
    // ⭐ O nome que QUEBRA cresce a linha; o alvo cobre as duas linhas dele.
    let linhas = cb.linhas_do_nome(rect, text_system);
    let passo = ph2d_tokens::row_pitch_px();
    hit_index.register(
        id,
        Rect::new(x, y, w, ROW_H_PX + passo * (linhas - 1) as f32),
    );
    crate::widget::paint_checkbox(&cb, rect, scene, text_system, theme);
    let dot = crate::widget::decorator_rect(rect);
    if let Some(alvo) =
        crate::widget::key_mark::paint_decorator_campo(scene, theme, dot, &[id], label)
    {
        hit_index.register(alvo, dot);
    }
    y + passo * linhas as f32
}

/// ⭐⭐⭐ **AS LINHAS DE MARCAR DE UMA SECÇÃO — emparelhadas quando CABEM, empilhadas quando não.**
///
/// ⛔⛔⛔ **Medido em 2026-09-15, depois de a marca ganhar CAIXA** (§6-quinquies): as **cinco**
/// fileiras emparelhadas de booleanos do Inspector (*Animation* · *Timers* · *Audio* · *Camera* ·
/// o *Flip H/V* da folha) ficaram com os **dez** nomes CORTADOS em todo o curso útil do dock:
///
/// | painel | METADE da linha | coluna do nome que sobra | os nomes medem |
/// |---|---|---|---|
/// | `220` (mínimo do dock) | `83,0` | **`0,0`** ⛔ (nome nenhum é pintado) | `28,5`–`79,8` |
/// | `273,3` (a do dono) | `109,6` | **`15,6`** ⛔ | idem |
/// | `304` (omissão) | `125,0` | **`31,0`** ⛔ | idem |
/// | `420` | `183,0` | `83,5` ✅ | idem |
///
/// ⚠️⚠️ **A causa não é o emparelhamento: é que uma METADE não tem onde pôr a caixa.** O piso do
/// campo é [`crate::widget::NUMBER_INPUT_MIN_W_PX`] (`72`, ordem do dono de 2026-05-24) e numa
/// metade de `109,6` ele come tudo menos `15,6` — *o tecto do §6 e a metade colidem, e quem perde é
/// o nome*. Antes da caixa a marca media `18` e sobrava linha para o nome; **a aparência que o dono
/// mandou adoptar não cabe em meia linha.**
///
/// ⇒ a lei do §6-quater aplica-se aqui **tal como está escrita**: *emparelhar é uma escolha do
/// painel; caber é uma medição da porta.* ⛔ E a pergunta vai à [`property_row_fits`], nunca a uma
/// segunda aritmética.
///
/// ⚠️ **Quando emparelham, a «secção» das duas é o PAR** — as duas metades partilham uma coluna
/// medida sobre os dois nomes, senão a da esquerda e a da direita caem em `x` diferentes dentro da
/// mesma fileira. Quando desemparelham, cada uma entra na coluna da **secção**, que é o que as
/// alinha com os números ao lado.
#[allow(clippy::too_many_arguments)]
pub fn paint_check_rows<'a, L: Into<Rotulo<'a>> + Copy>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    caixas: &[(NodeId, L, bool)],
    seccao: Seccao,
) -> f32 {
    let caixas: Vec<(NodeId, Rotulo<'a>, bool)> = caixas
        .iter()
        .map(|&(id, l, on)| (id, l.into(), on))
        .collect();
    let gap = ph2d_tokens::Spacing::Sm.px();
    let meia = ((w - gap) * 0.5).max(1.0);
    let cartao = crate::widget::card_ink::card_body().is_some();
    if cartao && crate::widget::card_ink::corpo_a_fluir() {
        // O fluxo emparelha, em colunas iguais, e junta ao vizinho o que sobrar (05 §4).
        return caixas.iter().fold(y, |yy, c| {
            paint_check_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                yy,
                *c,
                seccao,
            )
        });
    }
    let fonte = if cartao {
        ph2d_card_stack::metrics::VALUE_PX
    } else {
        TypeToken::Sm.px()
    };
    // No cartão cada metade é uma MOLDURA com o nome em cima (dono 13/11): o par cabe quando a
    // medida da linha de marcar — o nome com o losango, ou ✓ e «On» — cabe na metade.
    let cabe = |ts: &mut TextSystem, meia: f32, quer: f32, a: Rotulo<'_>, b: Rotulo<'_>| {
        if cartao {
            let pede = crate::widget::card_check_w;
            pede(ts, a).max(pede(ts, b)) <= meia
        } else {
            property_row_fits(meia, quer)
        }
    };
    let mut cur_y = y;
    let mut i = 0;
    while i < caixas.len() {
        // ⚠️ **A medida é a do nome MAIS LARGO do par**, não a de cada um: as duas metades vão
        //    partilhar uma coluna só, logo é ela que tem de caber. *Perguntar por nome aprovaria um
        //    par cuja coluna comum não cabe em nenhuma das duas.*
        let par = (i + 1 < caixas.len()).then(|| {
            text_system
                .prefix_width(caixas[i].1.texto, fonte)
                .max(text_system.prefix_width(caixas[i + 1].1.texto, fonte))
        });
        match par {
            Some(quer) if cabe(text_system, meia, quer, caixas[i].1, caixas[i + 1].1) => {
                let sec =
                    Seccao::medida(text_system, 1, &[caixas[i].1.texto, caixas[i + 1].1.texto]);
                // No cartão o losango mora na linha do nome, acima do canto da SUA moldura: as
                // duas metades têm a largura inteira e alinham com os campos vizinhos.
                paint_check_row(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x,
                    meia,
                    cur_y,
                    caixas[i],
                    sec,
                );
                cur_y = paint_check_row(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x + meia + gap,
                    meia,
                    cur_y,
                    caixas[i + 1],
                    sec,
                );
                i += 2;
            }
            _ => {
                cur_y = paint_check_row(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x,
                    w,
                    cur_y,
                    caixas[i],
                    seccao,
                );
                i += 1;
            }
        }
    }
    cur_y
}
