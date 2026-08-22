//! **O modelo da §5 9-Slice do Inspector** — snapshot, flags de divergência e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** (700) — mesmo padrão de
//! `inspector_model_ordering.rs` / `_joint.rs` / `_physics.rs` / `_player.rs`, e o corte é por
//! família.
//!
//! A seção que a spec declarou em 2026-05 e que ninguém construiu: até 2026-08-21
//! `git grep -c SliceNine` dava **0** em todo o repositório.
//!
//! # Tags cruas, de propósito
//!
//! Como toda a família, este snapshot fala em `u8`/`f32` e **não** importa `ph2d_ecs`: o
//! `editor-core` fica solto do motor, e a conversão `tag ↔ enum` acontece nas duas pontas
//! (`from_tag` na shell). A **posição no array de ids É a tag** — a mesma lei da §9.

/// Snapshot da autoria de 9-slice da entidade selecionada.
///
/// ⚠️ **`present` é a pergunta que a seção faz primeiro.** Sem o componente, a seção pinta um
/// botão «+ Add 9-Slice» e mais nada — não há valores para mostrar, e mostrar zeros seria
/// afirmar que o sprite tem bordas a zero quando o que ele tem é *nenhuma autoria de 9-slice*.
/// *Ausência não é o valor por omissão.*
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorSliceInfo {
    pub entity_bits: u64,
    /// O componente `SliceNine` está anexado?
    pub present: bool,
    /// `SliceDrawMode` — tag `0..=2` (Simple / Sliced / Tiled).
    pub draw_mode_tag: u8,
    /// `[left, top, right, bottom]` em pixels da fonte.
    pub borders: [f32; 4],
    /// Tamanho alvo em metros; `0` = herda o do sprite.
    pub size: [f32; 2],
    /// `TileRegionMode` por região, na ordem de `SliceRegion::ALL` — tags `0..=3`.
    pub tile_modes: [u8; 8],
    /// `SliceTileMode` — tag `0..=1` (Continuous / Adaptive).
    pub tile_mode_tag: u8,
    /// `0..1`, lido só em Adaptive.
    pub stretch_value: f32,
    pub fill_center: bool,
    pub selected_count: usize,
    pub mixed: InspectorSliceMixed,
}

/// Que campos divergem na seleção múltipla — a afordância «Mixed».
///
/// ⚠️ Um campo divergente **não acende segmento nenhum** e não mostra número: acender o valor da
/// primária seria afirmar que toda a seleção concorda. Foi o defeito que a auditoria de
/// 2026-08-21 mediu em sete controlos desta família.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct InspectorSliceMixed {
    pub present: bool,
    pub draw_mode: bool,
    pub borders: bool,
    pub size: bool,
    pub tile_modes: bool,
    pub tile_mode: bool,
    pub stretch_value: bool,
    pub fill_center: bool,
}

/// Uma edição da §5, despachada como `EditorAction::InspectorSliceEdit`.
///
/// ⚠️ **`Border` e `RegionMode` carregam o ÍNDICE** em vez de existirem em quatro/oito variantes,
/// e isso não é economia: é a lei que o `PerCornerTintAt` e o `RegionX/Y/W/H` já pagaram — uma
/// edição que reescreve o array inteiro atropela, num fan-out de seleção múltipla, os vizinhos
/// divergentes de todas as outras entidades. *Edita-se o que se tocou, nunca o vetor todo.*
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SliceFieldEdit {
    /// Anexa `SliceNine::INERT`. Inerte de propósito: anexar não muda um pixel.
    Attach,
    /// Retira o componente.
    Detach,
    /// Tag de `SliceDrawMode`.
    DrawMode(u8),
    /// `(índice em [l, t, r, b], valor em pixels da fonte)`.
    Border(u8, f32),
    SizeX(f32),
    SizeY(f32),
    /// `(índice de `SliceRegion`, tag de `TileRegionMode`)`.
    RegionMode(u8, u8),
    /// Tag de `SliceTileMode`.
    TileMode(u8),
    StretchValue(f32),
    FillCenter(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ As duas edições indexadas têm de conseguir exprimir **cada** posição — senão a última
    /// borda (ou a última região) é inalcançável por gesto nenhum, que é como uma feature nasce
    /// morta sem que nada reprove.
    #[test]
    fn the_indexed_edits_can_address_every_slot() {
        let borders: Vec<SliceFieldEdit> = (0..4)
            .map(|i| SliceFieldEdit::Border(i, i as f32))
            .collect();
        assert_eq!(borders.len(), 4);
        let regions: Vec<SliceFieldEdit> =
            (0..8).map(|i| SliceFieldEdit::RegionMode(i, 1)).collect();
        assert_eq!(regions.len(), 8);
        // Distintas entre si — um `PartialEq` que colapsasse índices tornaria o gate de
        // despacho incapaz de distinguir a borda esquerda da direita.
        for (i, a) in borders.iter().enumerate() {
            for (j, b) in borders.iter().enumerate() {
                assert_eq!(i == j, a == b, "Border({i}) vs Border({j})");
            }
        }
    }

    /// Ausência não é o valor por omissão: `present: false` com bordas a zero é um estado
    /// **diferente** de um componente anexado com bordas a zero.
    #[test]
    fn absence_is_not_the_default_value() {
        let absent = InspectorSliceInfo {
            entity_bits: 1,
            present: false,
            draw_mode_tag: 0,
            borders: [0.0; 4],
            size: [0.0; 2],
            tile_modes: [0; 8],
            tile_mode_tag: 0,
            stretch_value: 0.5,
            fill_center: true,
            selected_count: 1,
            mixed: InspectorSliceMixed::default(),
        };
        let attached = InspectorSliceInfo {
            present: true,
            ..absent
        };
        assert_ne!(absent, attached);
    }
}
