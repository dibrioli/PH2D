//! **A seção SYMMETRY** do painel — irmã de [`super::paint_pencil`] pelo teto de 600 LOC dos
//! painéis, e o corte é o mesmo: aqui mora a seção de um MODO de desenho.
//!
//! ⚠️ **Isto nasceu como um efeito da pilha e foi REPROVADO** (Enio, 2026-08-01: *"melhor como uma
//! opção para as tools de desenho exatamente como o modo painter, morando em uma seção específica
//! para isso"*). A seção é essa.
//!
//! # `Enable` no TOPO, e ele gateia TUDO
//!
//! É a lei que o Enio estabeleceu no painel do impasto (*"é quem habilita esse modo de pintura …
//! esse card só aparece se enable estiver checado"*). Desarmado, os controles abaixo editariam o
//! estilo de um espelho que não existe — quatro chips e um slider que não mudam um pixel.
//!
//! # Cada controle aparece onde tem o que fazer
//!
//! **Segments** só no Radial (num espelho a contagem é dois por definição, e um slider preso em 2
//! é um controle morto) e **Fuse** só nos espelhos (no Radial não há costura a fechar, e o kernel
//! o ignora). **Apply** só quando há simetria VIVA na seleção — sem cópias não há o que
//! consolidar, e a pergunta é feita à shell, que é quem vê a cena.

use ph2d_i18n::tr;
use ph2d_symmetry::{MAX_SEGMENTS, MIN_SEGMENTS, SymmetryKind};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::symmetry_kind_id;

use crate::ids;
use crate::paint_sections::BodyCtx;

/// O `scale` do slider de Segments: o track anda `0..=1` e a contagem `MIN..=MAX`.
pub(crate) const SEGMENTS_SCALE: f32 = (MAX_SEGMENTS - MIN_SEGMENTS) as f32;
/// O `offset` do mesmo slider.
pub(crate) const SEGMENTS_OFFSET: f32 = MIN_SEGMENTS as f32;

/// O track `0..=1` que corresponde a `n` segmentos.
#[must_use]
pub(crate) fn segments_to_track(n: u32) -> f32 {
    // Os dois limites são consts `u32` do vocabulário: NaN é impossível por TIPO, e a ordem
    // deles é a MESMA que o `SymmetrySpec::segments()` honra — as duas leem a mesma faixa.
    let n = n.clamp(MIN_SEGMENTS, MAX_SEGMENTS); // CLAMP-OK: bounds são consts `u32`
    (n - MIN_SEGMENTS) as f32 / SEGMENTS_SCALE
}

/// **Quantas cópias o track `0..=1` pede** — a INVERSA de [`segments_to_track`], e a porta única
/// por onde um arrasto vira contagem.
///
/// ⚠️⚠️ **O arredondamento mora NESTA fronteira, e não no consumidor.** O track é contínuo e uma
/// rosácea de `7,4` cópias não existe: quem lê a resposta (o kernel radial, via
/// `SymmetrySpec::copy_count`) recebe um número de cópias. Duas conversões — uma aqui e outra na
/// tool — divergiriam no dia em que uma delas arredondasse para o outro lado, e o sintoma seria a
/// contagem escrita no chip a discordar das cópias desenhadas.
///
/// ⚠️ A faixa é a MESMA que o `populate_symmetry` dá ao par slider↔chip (`SEGMENTS_SCALE` /
/// `SEGMENTS_OFFSET`): é por isso que a ida-e-volta fecha.
#[must_use]
pub(crate) fn track_to_segments(t: f32) -> u32 {
    let n = t.mul_add(SEGMENTS_SCALE, SEGMENTS_OFFSET);
    if !n.is_finite() {
        // Um track degenerado não escolhe nada — o piso do vocabulário é a resposta honesta.
        return MIN_SEGMENTS;
    }
    let (lo, hi) = (SEGMENTS_OFFSET, SEGMENTS_OFFSET + SEGMENTS_SCALE);
    n.round().clamp(lo, hi) as u32 // CLAMP-OK: consts `u32`, e `n` é finito pela guarda acima
}

impl BodyCtx<'_> {
    /// **A seção SYMMETRY** — o modo de desenho simétrico.
    pub(crate) fn symmetry_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_SYMMETRY,
            tr("panel.vector.section.symmetry"),
            y,
        );
        if collapsed {
            return y;
        }
        let sym = snap.symmetry;
        y = self.segmented(
            tr("panel.vector.symmetry.enable"),
            &[
                (
                    ids::VECTOR_SYM_OFF,
                    tr("panel.vector.symmetry.off"),
                    !sym.on,
                ),
                (ids::VECTOR_SYM_ON, tr("panel.vector.symmetry.on"), sym.on),
            ],
            y,
        );
        if !sym.on {
            return y;
        }
        // ⚠️ A fileira é construída a partir de `SymmetryKind::ALL` e os rótulos vêm do `label()`
        // do próprio enum: um tipo novo entra no vocabulário e ganha o chip de graça. Uma tabela
        // paralela aqui nasceria incompleta no dia do quinto — e o chip que falta é invisível.
        let kinds: Vec<(ph2d_a11y::NodeId, &str, bool)> = SymmetryKind::ALL
            .iter()
            .map(|k| (symmetry_kind_id(*k), k.label(), *k == sym.kind))
            .collect();
        y = self.segmented(tr("panel.vector.symmetry.axis"), &kinds, y);

        if sym.kind == SymmetryKind::Radial {
            let track = self
                .store
                .slider(ids::VECTOR_SYM_SEGMENTS)
                .map_or_else(|| segments_to_track(sym.segments), |(_, v)| v);
            let n = self
                .store
                .number_value(ids::VECTOR_SYM_SEGMENTS_NUM)
                .unwrap_or(f64::from(sym.segments));
            y = self.slider_row(
                tr("panel.vector.symmetry.segments"),
                ids::VECTOR_SYM_SEGMENTS,
                ids::VECTOR_SYM_SEGMENTS_NUM,
                track,
                n,
                &format!("{n:.0}"),
                y,
            );
        } else {
            y = self.segmented(
                tr("panel.vector.symmetry.fuse"),
                &[
                    (
                        ids::VECTOR_SYM_FUSE_OFF,
                        tr("panel.vector.symmetry.off"),
                        !sym.fuse,
                    ),
                    (
                        ids::VECTOR_SYM_FUSE_ON,
                        tr("panel.vector.symmetry.on"),
                        sym.fuse,
                    ),
                ],
                y,
            );
        }

        // O **Apply** é a única coisa aqui que toca o documento, e por isso é a única que se
        // destaca (`Accent`, o idioma de commit desta shell — o mesmo do Apply da pilha de
        // efeitos). Sem simetria viva na seleção ele não é oferecido.
        if crate::state_symmetry::symmetry_live_count() > 0 {
            y = self.action_button_kind(
                ids::VECTOR_SYM_APPLY,
                tr("panel.vector.symmetry.apply"),
                ph2d_editor_core::widget::ButtonKind::Accent,
                y,
            );
        }
        y
    }
}
