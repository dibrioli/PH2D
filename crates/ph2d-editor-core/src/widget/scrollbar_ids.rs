//! ⭐ **A ESCADA DOS IDS de barra de rolagem** — irmão por ASSUNTO do [`super::scrollbar`], que
//! bateu no tecto de 500 LOC dos primitivos de widget.
//!
//! # ⚠️ Porque saem os IDS, e não a geometria
//!
//! O `scrollbar.rs` são duas coisas: a **lei** (a pista, o polegar, o arrasto, o visual) e o
//! **registo** de quem tem uma barra. A lei está estável há meses; o registo cresce **uma linha por
//! painel novo**, e é ele que este ficheiro isola — com os dois gates que o guardam ao lado, porque
//! um deles lê o próprio fonte e tem de ler o ficheiro onde as constantes de facto estão.
//!
//! # ⛔⛔ Um id aqui CONTA-SE, nunca se escolhe
//!
//! O `MOTION_PARAMS_SCROLLBAR_ID` traz a lição escrita: a linha escreveu `839` e o valor **contado
//! na integração** era `841`, porque duas outras linhas pousaram na mesma janela — e o git funde
//! duas `pub const` de **nomes diferentes** limpo, sem conflito nenhum. ⇒ o próximo id conta-se
//! contra o `main` **do dia da integração**, e os dois gates abaixo são a rede.

use ph2d_a11y::NodeId;

/// Stable hit id for the inspector's scrollbar thumb. (Exported so
/// the dispatch can route Down/Move/Up events without a string
/// lookup. Hierarchy gets a separate id.)
pub const INSPECTOR_SCROLLBAR_ID: NodeId = NodeId(820);
pub const HIERARCHY_SCROLLBAR_ID: NodeId = NodeId(821);
/// Widget Gallery floating panel scrollbar — mirrors the Inspector's
/// behavior with an independent thumb id so dispatch can route
/// drag-scroll events without aliasing the Inspector's scroll state.
pub const GALLERY_SCROLLBAR_ID: NodeId = NodeId(822);
/// Grid Settings floating panel scrollbar — same shape as Inspector /
/// Gallery, independent thumb id keeps dispatch from aliasing.
pub const GRID_SETTINGS_SCROLLBAR_ID: NodeId = NodeId(823);
/// Color Equalization panel scrollbar — image-tools docked panel grew
/// past the dock height once Phase 2/3 (sharpen/LUT) landed.
/// Independent thumb id so dispatch can route drag-scroll cleanly.
pub const COLOR_EQUALIZATION_SCROLLBAR_ID: NodeId = NodeId(824);
/// Bg Removal panel scrollbar — same image-tools dock slot as
/// Color Equalization; the Protect-brush sub-section (Size slider +
/// 4-falloff segmented + Show-mask + Clear) pushes the panel past
/// the dock height. Independent thumb id keeps dispatch from
/// aliasing CEQ's scroll state when both tools are toggled.
pub const BG_REMOVAL_SCROLLBAR_ID: NodeId = NodeId(825);
/// Padding panel scrollbar — image-tools dock; Reset/Cancel/Apply
/// drift off the dock once the user shrinks the panel. Enio 2026-05-26:
/// "padrão central do app é painel com scroll. corrija todos".
pub const PADDING_SCROLLBAR_ID: NodeId = NodeId(826);
/// Upscale panel scrollbar — same image-tools dock; same overflow
/// pattern as Padding.
pub const UPSCALE_SCROLLBAR_ID: NodeId = NodeId(827);
/// Equalize Sizes panel scrollbar — image-tools dock; conditional
/// sub-sections (Fixed chips, Grid offset slider + arrange toggle,
/// upscale algorithm row) easily push the body past dock height.
pub const EQUALIZE_SIZES_SCROLLBAR_ID: NodeId = NodeId(828);
/// Painter Layers floating panel scrollbar — the layer stack grows past
/// the panel height once a few layers are added. Independent thumb id so
/// dispatch routes drag-scroll without aliasing the Inspector / other
/// panels. Wheel-scroll already works via the generic `dispatch_wheel`;
/// the panel registers this id on its `scrollbar_thumb_rect` to enable
/// thumb-DRAG (Painter W3 audit item 4).
pub const PAINTER_LAYERS_SCROLLBAR_ID: NodeId = NodeId(829);
/// Brush Studio floating panel scrollbar (W5) — the brush-parameter editor's
/// three sections (Stroke Path / Shape / Rendering) overflow the dock height.
/// Independent thumb id so dispatch routes drag-scroll without aliasing the
/// sidebar / layers panels that share the same dock slot.
pub const PAINTER_BRUSH_STUDIO_SCROLLBAR_ID: NodeId = NodeId(830);
/// Audio Mixer docked-panel scrollbar — the channel strips + the stacked
/// master-effect sections (EQ / Reverb / Delay / Comp / Ducking, each with
/// per-bus rows) overflow the Inspector-dock height. Independent thumb id so
/// dispatch routes drag-scroll without aliasing the other dock-slot panels.
/// (831 is `DROPDOWN_SCROLLBAR_ID` in `widget/dropdown.rs`.)
pub const AUDIO_MIXER_SCROLLBAR_ID: NodeId = NodeId(832);
/// Vector Style docked panel scrollbar (ADR-0108) — the panel body (Stroke/Fill +
/// Cap/Join + Dash/Gap + Draw modes + per-shape sliders + Vertex + Boolean +
/// Arrange) overflows the dock height. Independent thumb id so dispatch routes
/// drag-scroll without aliasing the Inspector that shares the dock slot.
/// NOTE: `831` is `DROPDOWN_SCROLLBAR_ID` (dropdown.rs) — dispatch special-cases
/// that id to the open dropdown, so a collision would make this thumb
/// un-draggable — and `832` is `AUDIO_MIXER_SCROLLBAR_ID` above. Use `833`.
pub const VECTOR_SCROLLBAR_ID: NodeId = NodeId(833);
/// Audio Editor docked-panel scrollbar (docs/Audio/, W3 block 3b) — the transport +
/// edit ops + the effects rack (selector, up to 4 parameter sliders, the chain list
/// and its commit row) overflow the dock height. Independent thumb id so dispatch
/// routes drag-scroll without aliasing the Audio Mixer / Inspector that share the
/// dock slot. Next free id is `835`; re-read the collision note above before taking it.
pub const AUDIO_EDITOR_SCROLLBAR_ID: NodeId = NodeId(834);
/// Flip Style docked-panel scrollbar (ADR-0114 W2) — the panel body (Mode +
/// Brush + Color + Erase + the Layers stack) overflows the dock height once a
/// few layers are added. Independent thumb id so dispatch routes drag-scroll
/// without aliasing the Inspector / Vector panels that share the dock slot.
/// Next free id is `836`; re-read the collision note above before taking it.
pub const FLIP_SCROLLBAR_ID: NodeId = NodeId(835);
/// Physics world docked-panel scrollbar (ADR-0131 D8 / W2b) — the panel body
/// (World + Solver + Damping + Sleep + Debug) overflows the dock height.
/// Independent thumb id so dispatch routes drag-scroll without aliasing the
/// Inspector / Vector / Flip panels that share the dock slot.
/// Next free id is `837`; re-read the collision note above before taking it.
pub const PHYSICS_SCROLLBAR_ID: NodeId = NodeId(836);
/// Wet Tuning docked-panel scrollbar (doc 22) — the full knob table (40 rows
/// in 6 sections) always overflows the dock height. Independent thumb id so
/// dispatch routes drag-scroll without aliasing the panels sharing the
/// column. Next free id is `838`; re-read the collision note above before
/// taking it.
pub const WET_TUNING_SCROLLBAR_ID: NodeId = NodeId(837);
/// Tokens docked-panel scrollbar (plano UI/UX W6) — a tabela de cor tem ~80
/// linhas e transborda a altura do dock em qualquer resolução. Thumb próprio
/// pelo mesmo motivo dos irmãos acima. Next free id is `839`; re-read the
/// collision note above before taking it.
pub const TOKENS_SCROLLBAR_ID: NodeId = NodeId(838);
/// Authored docked-panel scrollbar (plano UI/UX W8b.2) — the panel the
/// artist DREW, so its row count is whatever they drew: it can overflow
/// the dock at any height and there is no table to bound it. Own thumb id
/// for the same reason as the siblings above. Next free id is `840`;
/// re-read the collision note above before taking it.
pub const AUTHORED_SCROLLBAR_ID: NodeId = NodeId(839);
/// Painel da cena 3D (ADR-0150 W12) — seis seções, das quais a de FERRAMENTA
/// sozinha é uma faixa de dezesseis chips que reflui em várias linhas: ele
/// transborda o dock em qualquer resolução. Thumb próprio pelo mesmo motivo dos
/// irmãos acima. Next free id is `841`; re-read the collision note above before
/// taking it.
pub const SCULPT3D_SCROLLBAR_ID: NodeId = NodeId(840);
/// Motion **params** docked-panel scrollbar (doc 88 §B3). Medido: uma linha
/// escalar ocupa **34 px** e o dock comporta **24** delas, contra um teto de
/// `MAX_PARAM_ROWS = 16` e um pior nó (`motion.tint`) de **15 params** — ou seja
/// oito linhas de folga para uma varredura que dá a TODO nó o conjunto PRO. O
/// gate `a_full_panel_of_rows_fits_the_inspector` já previa o dia em texto:
/// *"o painel precisa ROLAR antes de o teto subir mais"*. Next free id is `842`;
/// re-read the collision note above before taking it.
///
/// ⚠️ **A linha escreveu 839; o valor CONTADO na integração é 841** — a
/// `line/Vector` (AUTHORED, 839) e a `line/sculpt3d` (SCULPT3D, 840) pousaram
/// antes na mesma janela, e os NOMES das constantes diferem, então o git funde
/// as três limpas e deixa a colisão para o `assert_ne!` da lista abaixo. O
/// número se CONTA a partir do `main` do dia, nunca se escolhe.
pub const MOTION_PARAMS_SCROLLBAR_ID: NodeId = NodeId(841);

/// **A janela flutuante do INPUT MAP** (plano 30) — a lista de acções transborda o cartão assim que
/// o projecto passa de meia dúzia delas.
///
/// ⛔ Auditoria de 2026-08-24: a barra era pintada e **nunca registada** — não arrastava, não fazia
/// hover, e não tinha id nenhum. *Uma barra que não se pode agarrar é um enfeite que promete um
/// gesto.* Thumb próprio pelo mesmo motivo dos irmãos acima. Next free id is `843`; re-read the
/// collision note above before taking it.
pub const INPUT_MAP_SCROLLBAR_ID: NodeId = NodeId(842);

/// **O painel do módulo de MODELAGEM 3D** (ADR-0161) — o report do Enio de 2026-08-27:
/// *«o painel 3d Model precisa de scroll e barra de scroll»*.
///
/// ⛔ **Ele já RECORTAVA e nunca rolava**, que é a pior das três formas: um painel sem recorte
/// desenha por cima do título e vê-se; um que recorta e rola funciona; **um que recorta e não rola
/// esconde os controles e não diz nada.** As linhas do fim — o rodapé, e as fileiras de parâmetros
/// de um documento com vários nós — ficavam inalcançáveis, sem sinal nenhum de que existiam.
///
/// ⚠️ Thumb próprio pelo mesmo motivo dos irmãos acima.
pub const MODEL3D_SCROLLBAR_ID: NodeId = NodeId(843);

/// Barra do **navegador de assets** (plano `docs/Components/07`, wave A4) — a grade de cartões
/// passa da altura do painel assim que o projecto tem mais de uma dúzia de assets.
///
/// ⚠️ Thumb próprio pelo mesmo motivo dos irmãos acima.
pub const ASSET_BROWSER_SCROLLBAR_ID: NodeId = NodeId(844);

/// Barra da **coluna de catálogos** do navegador (plano 07, wave A3).
///
/// ⚠️ **Thumb próprio, e a região que ela rola NÃO é um painel:** a chave de rolagem é
/// `ids::ASSET_CATALOG_COL`, e as tabelas `panel_scroll`/`panel_content_h`/`panel_visible_h`
/// aceitam qualquer `NodeId` — é o que o popover do dropdown já faz. Aliasá-la ao thumb da grade
/// faria as duas regiões rolarem juntas.
///
/// ⚠️⚠️ **O `845` CONTA-SE contra o `main` do dia da integração, nunca se escolhe** (§5.0): o
/// `MOTION_PARAMS_SCROLLBAR_ID` traz a lição — a linha escreveu `839` e o valor contado na
/// integração era `841`, porque o git funde duas `pub const` de NOMES diferentes **limpo**. Next
/// free id is `846`; re-read the collision note above before taking it.
pub const ASSET_CATALOG_SCROLLBAR_ID: NodeId = NodeId(845);

/// ⭐ **A bancada de widgets** (`ph2d-panel-widget-lab`) — o estudo dos quatro desenhos de slider.
///
/// ⚠️⚠️ **`846` foi CONTADO na integração de 2026-09-04; a `line/UIUX` escreveu `844`.** Entre o
/// fecho dela e a fusão, a `line/components` instalou este ficheiro e gastou o `844` e o `845` no
/// navegador de assets e nos catálogos. ⛔ *Um id que soma entre linhas conta-se, nunca se
/// escolhe* — e as duas terem escrito o mesmo literal é a colisão que funde **muda**: o
/// `assert_ne!` da lista de unicidade é que a acusaria, e só se alguém a mantivesse em dia.
pub const LAB_SCROLLBAR_ID: NodeId = NodeId(846);

#[cfg(test)]
mod tests {
    use super::*;

    /// The scrollbar thumb ids + `DROPDOWN_SCROLLBAR_ID` are hand-assigned raw
    /// `NodeId`s (820..) — NOT hashed, so `node_id_collisions` (which scans the
    /// chrome hash-ids) does NOT cover them. A collision silently breaks drag
    /// routing: dispatch special-cases `DROPDOWN_SCROLLBAR_ID`, so any thumb
    /// aliased onto it becomes un-draggable (the Vector panel bug, 2026-07-07).
    /// Assert pairwise uniqueness here so a new panel's id can't re-collide.
    ///
    /// ⚠️ **This list is an ENUMERATION, and it ROTTED** (found 2026-08-05, plano UI/UX W8b.2):
    /// the last two panels to take an id — Wet Tuning (837) and Tokens (838) — were never added,
    /// so for two waves the gate that exists to stop a collision was blind to the very ids most
    /// likely to collide. A hand-written list guards the entries somebody remembered to list.
    /// [`every_scrollbar_id_is_in_the_uniqueness_list`] is the fix: it reads THIS FILE and makes
    /// the omission itself a failure, so the next id cannot be born unguarded.
    #[test]
    fn scrollbar_and_dropdown_thumb_ids_are_unique() {
        let ids = [
            ("INSPECTOR", INSPECTOR_SCROLLBAR_ID),
            ("HIERARCHY", HIERARCHY_SCROLLBAR_ID),
            ("GALLERY", GALLERY_SCROLLBAR_ID),
            ("GRID_SETTINGS", GRID_SETTINGS_SCROLLBAR_ID),
            ("COLOR_EQUALIZATION", COLOR_EQUALIZATION_SCROLLBAR_ID),
            ("BG_REMOVAL", BG_REMOVAL_SCROLLBAR_ID),
            ("PADDING", PADDING_SCROLLBAR_ID),
            ("UPSCALE", UPSCALE_SCROLLBAR_ID),
            ("EQUALIZE_SIZES", EQUALIZE_SIZES_SCROLLBAR_ID),
            ("PAINTER_LAYERS", PAINTER_LAYERS_SCROLLBAR_ID),
            ("PAINTER_BRUSH_STUDIO", PAINTER_BRUSH_STUDIO_SCROLLBAR_ID),
            ("AUDIO_MIXER", AUDIO_MIXER_SCROLLBAR_ID),
            ("VECTOR", VECTOR_SCROLLBAR_ID),
            ("AUDIO_EDITOR", AUDIO_EDITOR_SCROLLBAR_ID),
            ("FLIP", FLIP_SCROLLBAR_ID),
            ("PHYSICS", PHYSICS_SCROLLBAR_ID),
            ("WET_TUNING", WET_TUNING_SCROLLBAR_ID),
            ("TOKENS", TOKENS_SCROLLBAR_ID),
            ("AUTHORED", AUTHORED_SCROLLBAR_ID),
            ("SCULPT3D", SCULPT3D_SCROLLBAR_ID),
            ("MOTION_PARAMS", MOTION_PARAMS_SCROLLBAR_ID),
            ("INPUT_MAP", INPUT_MAP_SCROLLBAR_ID),
            ("MODEL3D", MODEL3D_SCROLLBAR_ID),
            ("ASSET_CATALOG", ASSET_CATALOG_SCROLLBAR_ID),
            ("LAB", LAB_SCROLLBAR_ID),
            ("ASSET_BROWSER", ASSET_BROWSER_SCROLLBAR_ID),
            ("DROPDOWN", crate::widget::DROPDOWN_SCROLLBAR_ID),
        ];
        for (i, (na, a)) in ids.iter().enumerate() {
            for (nb, b) in &ids[i + 1..] {
                assert_ne!(a, b, "scrollbar id collision: {na} == {nb} ({a:?})");
            }
        }
    }

    /// **Toda const `*_SCROLLBAR_ID` deste arquivo está na lista de unicidade acima.**
    ///
    /// ⚠️ Ele lê o PRÓPRIO fonte porque a pergunta não é sobre valores — é sobre a lista estar
    /// COMPLETA, e nenhum teste que só compara os ids que a lista contém consegue fazê-la: um id
    /// omitido é invisível para ele. É o mesmo padrão dos arch-gates da shell, aqui aplicado ao
    /// arquivo que declara as consts.
    #[test]
    fn every_scrollbar_id_is_in_the_uniqueness_list() {
        let src = include_str!("scrollbar_ids.rs");
        let declared: Vec<&str> = src
            .lines()
            .filter_map(|l| l.trim().strip_prefix("pub const "))
            .filter_map(|l| l.split(':').next())
            .filter(|n| n.ends_with("_SCROLLBAR_ID"))
            .collect();
        // Controle positivo: uma varredura vazia tornaria este gate verde por vácuo.
        assert!(
            declared.len() >= 10,
            "a varredura nao achou as consts — o gate estaria verde por vacuo"
        );
        let list = src
            .split("fn scrollbar_and_dropdown_thumb_ids_are_unique()")
            .nth(1)
            .expect("a lista de unicidade existe");
        let list = list.split("];").next().expect("a lista fecha");
        for name in declared {
            assert!(
                list.contains(name),
                "`{name}` nao esta na lista de unicidade — um id novo nasceu sem guarda"
            );
        }
    }
}
