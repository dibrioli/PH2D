//! A secção **PATTERN** do painel Vector (plano 33, W5) — módulo irmão do [`super`] (teto de LOC).
//!
//! ⚠️⚠️ **NÃO confundir com o [`super::paint_patternpath`].** Aquele é o *Pattern Along Path*
//! (plano 23): um MOTIVO copiado ao longo de uma guia. Este é a **TINTA** de uma forma — uma arte
//! repetida num reticulado, dentro dela.
//!
//! # Esta secção é a única porta do produto para afinar um padrão
//!
//! Sem ela o padrão nasce e **fica como nasceu**: o motor existiria, gateado e smokado, e o artista
//! teria uma imagem repetida que não consegue tocar. Foi exactamente esse o buraco entre a W4 e um
//! produto.
//!
//! # Ela só sobe quando há um padrão
//!
//! `current_texture_pattern()` devolve `None` para toda forma que não tem um — e então o cabeçalho
//! nem aparece. É a lei do `Join Selected Bodies` e da secção do Pattern on Path: *um controlo que
//! não se aplica é ruído, e um botão que recusa é pior que um botão que falta.*

use super::*;

/// Os quatro reticulados, na ordem em que o artista os pensa: o neutro, os dois tijolos, a colmeia.
const TILES: [(usize, &str); 4] = [(0, "Grid"), (1, "Brick"), (2, "Column"), (3, "Hex")];
/// As três leis de repetição.
const MODES: [(usize, &str); 3] = [(0, "Tile"), (1, "Mirror"), (2, "Clamp")];

impl BodyCtx<'_> {
    /// Secção **PATTERN** — a lei do padrão de textura da forma selecionada.
    pub(crate) fn texture_pattern_section(&mut self, y: f32) -> f32 {
        let Some(p) = state::current_texture_pattern() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_TEXPAT,
            tr("panel.vector.section.texpat"),
            y,
        );
        if collapsed {
            return y;
        }
        // A ARTE — trocar a imagem sem trocar a lei. ⚠️ O mesmo botão que o chip *Pattern* aciona
        // quando a forma ainda não tem padrão: uma porta, dois gatilhos.
        y = self.action_button(ids::VECTOR_TEXPAT_SOURCE, "Source...", y);
        // ⭐ **A ARTE pode ser uma FORMA do documento** (W7) — o modelo do Figma. O gesto é o de
        // duas mãos que a casa já tem: aperta, e o clique seguinte no canvas escolhe.
        y = self.action_button(ids::VECTOR_TEXPAT_PICK_SHAPE, "Use Shape...", y);

        // ⭐⭐ **UM PARÂMETRO QUE O MODO NÃO USA NÃO APARECE** (Enio, 2026-08-27).
        //
        // No `Clamp` há **uma** cópia, enquadrada na forma: o reticulado, o desfasamento, o tamanho
        // e o vão não têm quem os leia. Mostrá-los seria oferecer quatro knobs mortos — o defeito
        // que o [doc 90](../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md) catalogou
        // dezanove vezes, e que já custou a esta secção o Offset na colmeia.
        //
        // ⚠️ **Esconder NÃO é apagar**: a lei fica no documento e volta inteira ao sair do `Clamp`
        // — é o par da decisão de o enquadramento ser DERIVADO e nunca escrito.
        let repete = p.mode != 2;
        if repete {
            // O RETICULADO.
            let tiles: Vec<(ph2d_a11y::NodeId, &str, bool)> = TILES
                .iter()
                .map(|(i, l)| (tile_id(*i), *l, usize::from(p.kind) == *i))
                .collect();
            y = self.segmented("Tile", &tiles, y);
        }

        // O DESFASAMENTO — só com Brick/Column. ⚠️ Na grade ele não tem sentido, e na COLMEIA ele é
        // **fixo** em meio passo (é isso que a torna colmeia): oferecê-lo ali seria um knob que o
        // modelo ignora.
        if repete && matches!(p.kind, 1 | 2) {
            let denom = self
                .store
                .number_value(ids::VECTOR_TEXPAT_OFFSET_NUM)
                .unwrap_or(p.offset_denom);
            let track = self
                .store
                .slider(ids::VECTOR_TEXPAT_OFFSET)
                .map_or_else(|| denom_track(p.offset_denom), |(_, v)| v);
            y = self.slider_row(
                "Offset",
                ids::VECTOR_TEXPAT_OFFSET,
                ids::VECTOR_TEXPAT_OFFSET_NUM,
                track,
                denom,
                &format!("1/{}", denom.round() as i64),
                y,
            );
        }

        // O TAMANHO e o VÃO — os dois só existem enquanto o padrão REPETE.
        if repete {
            // ⭐⭐ O TAMANHO, **os DOIS eixos** (Enio, 2026-08-27: poder achatar a arte de
            // propósito). Era um número só — o lado maior, com o aspecto sempre preservado.
            //
            // ⚠️ **A protecção não desapareceu, mudou de lei imposta para gesto escolhido**: o
            // cadeado nasce LIGADO, e com ele mexer num eixo leva o outro pelo mesmo factor. Ele
            // preserva a razão **ACTUAL** e não a natural da arte — voltar ao aspecto da imagem
            // desfaria o achatamento que o artista acabou de autorar.
            for (axis, label, sid, nid) in [
                (
                    0usize,
                    "Width",
                    ids::VECTOR_TEXPAT_W,
                    ids::VECTOR_TEXPAT_W_NUM,
                ),
                (1, "Height", ids::VECTOR_TEXPAT_H, ids::VECTOR_TEXPAT_H_NUM),
            ] {
                let v = self.store.number_value(nid).unwrap_or(p.size[axis]);
                let track = self
                    .store
                    .slider(sid)
                    .map_or_else(|| size_track(p.size[axis]), |(_, t)| t);
                y = self.slider_row(label, sid, nid, track, v, &format!("{v:.2}"), y);
            }
            // ⚠️ O cadeado vem DEPOIS dos dois números que ele liga — ele descreve o que acontece
            // *àquelas duas linhas*, e um controlo que descreve o que está acima dele lê-se onde
            // está.
            y = self.checkbox_row(
                ids::VECTOR_TEXPAT_LOCK,
                tr("panel.vector.texpat.lock"),
                p.lock_aspect,
                y,
            );

            // O VÃO — bipolar; negativo é a sobreposição.
            let gap = self
                .store
                .number_value(ids::VECTOR_TEXPAT_GAP_NUM)
                .unwrap_or(p.gap);
            let gap_track = self
                .store
                .slider(ids::VECTOR_TEXPAT_GAP)
                .map_or_else(|| gap_track(p.gap), |(_, v)| v);
            y = self.slider_row(
                "Gap",
                ids::VECTOR_TEXPAT_GAP,
                ids::VECTOR_TEXPAT_GAP_NUM,
                gap_track,
                gap,
                &format!("{gap:.2}"),
                y,
            );
        }

        // ⭐ A POSIÇÃO — onde, dentro de uma repetição, a arte começa.
        //
        // ⚠️ Ela vive aqui porque as três alças de canvas do W6 foram RETIRADAS (Enio, 2026-08-27:
        // *"não ficou legal. vamos retirar e deixar os ajustes apenas no painel"*). O tamanho e a
        // rotação já tinham fileira; a posição não tinha nenhuma, e sem estas duas retirar as alças
        // teria tirado ao artista uma coisa que ele fazia.
        //
        // ⚠️ Dentro do `repete` pela mesma lei dos outros: no `Clamp` a colocação é DERIVADA (uma
        // cópia enquadrada na forma) e a fase não tem quem a leia.
        if repete {
            for (axis, label, sid, nid) in [
                (
                    0usize,
                    "Shift X",
                    ids::VECTOR_TEXPAT_SHIFT_X,
                    ids::VECTOR_TEXPAT_SHIFT_X_NUM,
                ),
                (
                    1,
                    "Shift Y",
                    ids::VECTOR_TEXPAT_SHIFT_Y,
                    ids::VECTOR_TEXPAT_SHIFT_Y_NUM,
                ),
            ] {
                let pct = self.store.number_value(nid).unwrap_or(p.shift_pct[axis]);
                let track = self
                    .store
                    .slider(sid)
                    .map_or_else(|| shift_track(p.shift_pct[axis]), |(_, v)| v);
                y = self.slider_row(label, sid, nid, track, pct, &format!("{pct:.0}%"), y);
            }
        }

        // O ÂNGULO do PADRÃO (não o da forma). ⚠️ Ele vale em TODOS os modos: no `Clamp` roda a
        // cópia enquadrada.

        let angle = self
            .store
            .number_value(ids::VECTOR_TEXPAT_ANGLE_NUM)
            .unwrap_or(p.angle_deg);
        let angle_track = self
            .store
            .slider(ids::VECTOR_TEXPAT_ANGLE)
            .map_or_else(|| angle_track(p.angle_deg), |(_, v)| v);
        y = self.slider_row(
            "Angle",
            ids::VECTOR_TEXPAT_ANGLE,
            ids::VECTOR_TEXPAT_ANGLE_NUM,
            angle_track,
            angle,
            &format!("{angle:.0}"),
            y,
        );

        // A REPETIÇÃO.
        let modes: Vec<(ph2d_a11y::NodeId, &str, bool)> = MODES
            .iter()
            .map(|(i, l)| (mode_id(*i), *l, usize::from(p.mode) == *i))
            .collect();
        self.segmented("Repeat", &modes, y)
    }
}

/// O id do chip de reticulado `i`. ⚠️ Uma porta só: o paint OFERECE por aqui e a shell HONRA pela
/// gémea ([`tile_index_of`]) — duas listas escritas à mão divergiriam no dia em que uma crescesse.
#[must_use]
pub fn tile_id(i: usize) -> ph2d_a11y::NodeId {
    match i {
        1 => ids::VECTOR_TEXPAT_TILE_BRICK,
        2 => ids::VECTOR_TEXPAT_TILE_COLUMN,
        3 => ids::VECTOR_TEXPAT_TILE_HEX,
        _ => ids::VECTOR_TEXPAT_TILE_GRID,
    }
}

/// A gémea de [`tile_id`]: o índice de reticulado que este id nomeia.
#[must_use]
pub fn tile_index_of(id: ph2d_a11y::NodeId) -> Option<u8> {
    (0..TILES.len())
        .find(|i| tile_id(*i) == id)
        .map(|i| i as u8)
}

/// O id do chip de repetição `i`.
#[must_use]
pub fn mode_id(i: usize) -> ph2d_a11y::NodeId {
    match i {
        1 => ids::VECTOR_TEXPAT_MODE_MIRROR,
        2 => ids::VECTOR_TEXPAT_MODE_CLAMP,
        _ => ids::VECTOR_TEXPAT_MODE_TILE,
    }
}

/// A gémea de [`mode_id`].
#[must_use]
pub fn mode_index_of(id: ph2d_a11y::NodeId) -> Option<u8> {
    (0..MODES.len())
        .find(|i| mode_id(*i) == id)
        .map(|i| i as u8)
}

/// O track `0..1` de um tamanho de mundo. ⚠️ O MESMO mapa que o `event` e o `populate` usam.
#[must_use]
pub(crate) fn size_track(size: f64) -> f32 {
    (((size - crate::TEXPAT_SIZE_MIN) / (crate::TEXPAT_SIZE_MAX - crate::TEXPAT_SIZE_MIN))
        .clamp(0.0, 1.0)) as f32
}

/// O track `0..1` de um vão bipolar (`0.5` = encostado).
#[must_use]
pub(crate) fn gap_track(gap: f64) -> f32 {
    (((gap + crate::TEXPAT_GAP_MAX) / (2.0 * crate::TEXPAT_GAP_MAX)).clamp(0.0, 1.0)) as f32
}

/// O track `0..1` de uma fase em percentagem (unipolar `0..100`). ⚠️ O MESMO mapa que o `event` e o
/// `populate` usam — a fronteira única.
#[must_use]
pub(crate) fn shift_track(pct: f64) -> f32 {
    ((pct / crate::TEXPAT_SHIFT_MAX).clamp(0.0, 1.0)) as f32
}

/// O track `0..1` de um ângulo em graus (unipolar `0..360`).
#[must_use]
pub(crate) fn angle_track(deg: f64) -> f32 {
    ((deg / crate::TEXPAT_ANGLE_MAX).clamp(0.0, 1.0)) as f32
}

/// O track `0..1` do denominador do desfasamento.
#[must_use]
pub(crate) fn denom_track(n: f64) -> f32 {
    (((n - crate::TEXPAT_DENOM_MIN) / (crate::TEXPAT_DENOM_MAX - crate::TEXPAT_DENOM_MIN))
        .clamp(0.0, 1.0)) as f32
}
