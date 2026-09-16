//! ⭐⭐⭐ **UMA LINHA DE MARCAR OCUPA A LINHA INTEIRA — a não ser que os dois nomes CAIBAM em meia.**
//!
//! # ⛔⛔ O defeito que este ficheiro existe para impedir
//!
//! Em 2026-09-15 a marca ganhou **CAIXA** por ordem do dono (*«na Godot coloca um box em todo o
//! lado direito da linha»*, spec §6-quinquies). A caixa ocupa a coluna do controlo e o piso dela é
//! o do campo — [`ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX`] = `72`.
//!
//! **Numa METADE de linha isso não cabe**, e o que sobra é o nome. Medido nesse dia sobre os **dez**
//! booleanos que o Inspector pintava emparelhados (*Animation* · *Timers* · *Audio* · *Camera* · o
//! *Flip H/V* da folha):
//!
//! | painel | METADE da linha | coluna que sobra ao nome | os nomes medem |
//! |---|---|---|---|
//! | `220` (mínimo do dock) | `83,0` | **`0,0`** ⛔ nome nenhum é pintado | `28,5`–`79,8` |
//! | `273,3` (a do dono) | `109,6` | **`15,6`** ⛔ | idem |
//! | `304` (omissão) | `125,0` | **`31,0`** ⛔ | idem |
//! | `420` | `183,0` | `83,5` ✅ | idem |
//!
//! ⇒ *a aparência que o dono mandou adoptar não cabe em meia linha*, e a lei que responde já estava
//! escrita: **spec §6-quater — emparelhar é uma escolha do painel; caber é uma medição da porta.**
//!
//! # ⚠️ A régua é o RECT REGISTADO, nunca uma segunda aritmética
//!
//! O alvo do clique de uma linha de marcar é a **linha inteira** (spec §6-quinquies), logo o rect
//! que o painel regista **é** a linha: ele diz se ela ocupou a largura toda ou metade dela. ⛔ A
//! coluna do nome não é observável daqui — e não precisa de ser: *o que o dono vê é a linha partir.*
//!
//! ⚠️ **O VEREDITO esperado sai da PORTA** ([`property_row_fits`]), que é a mesma que o painel
//! consulta. Isso não torna o gate circular: o que ele afirma é que o **painel PERGUNTA** — uma
//! mutação que emparelhe sempre (ou nunca) sangra, porque a escada cobre os dois regimes.
//!
//! # ⚠️ E a escada tem de conter os DOIS regimes
//!
//! Sem a metade de controlo, uma mutação que empilhe sempre passaria numa escada só de painéis
//! estreitos. É a mesma lição do `a_seccao_poe_todas_as_caixas_na_mesma_coluna` da Grelha.

use ph2d_editor_core::screens::hero::{
    HeroLayout, InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sprite};
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EE1;

/// O curso que o arrasto do dock permite (`DOCK_W_MIN`..`DOCK_W_MAX`), com a largura **datada** do
/// dono lá dentro. ⛔ Não é uma cerca: se ele a mudar, o número muda e a escada continua a valer.
const LARGURAS: &[f32] = &[220.0, 245.0, 273.3, 304.0, 400.0, 520.0, 720.0];

/// A sprite do fixture: grelha `4×2` **de propósito** — sem ela o *Show sheet on canvas* não é
/// pintado (`sheet_preview_row` devolve cedo), e o gate mediria uma secção sem a linha mais larga.
fn sprite() -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        entity_bits: ENTITY,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 3 },
        source_precision: Some(ph2d_editor_core::Precision::Rgba8),
        emissive: 0.0,
        sheet_label: None,
        source_pixels: Some((256, 256)),
        can_reimport: true,
        flip_x: false,
        flip_y: true,
        opacity: 1.0,
        tint_fill: false,
        hframes: 4,
        vframes: 2,
        frame: 3,
        tint: [1.0; 4],
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: false,
        region_rect: [0.0, 0.0, 64.0, 64.0],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }
}

fn rects_a(painel_w: f32) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        w: 2400.0,
        h: 8000.0,
    };
    let mut layout = HeroLayout::for_viewport(viewport);
    layout.inspector = Rect {
        x: viewport.w - painel_w,
        y: 0.0,
        w: painel_w,
        h: 8000.0,
    };
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_sprite(Some(sprite()));
    let r = host.paint_with_layout::<InspectorPanel>(&mut state, layout, viewport);
    set_current_inspector_sprite(None);
    r
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId, w: f32) -> Rect {
    rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("a linha {id:?} nao foi pintada ao painel {w}"))
}

/// **Os dois nomes do par cabem cada um na sua metade?** — a mesma pergunta que o painel faz, à
/// mesma porta.
fn o_par_cabe(ts: &mut TextSystem, linha_w: f32, a: &str, b: &str) -> bool {
    let fonte = TypeToken::Sm.px();
    let meia = ((linha_w - Spacing::Sm.px()) * 0.5).max(1.0);
    let quer = ts.prefix_width(a, fonte).max(ts.prefix_width(b, fonte));
    ph2d_editor_core::property_row::property_row_fits(meia, quer)
}

#[test]
fn uma_linha_de_marcar_ocupa_a_linha_inteira() {
    let mut ts = TextSystem::new();
    let flip_h = ph2d_i18n::tr("panel.inspector.sprite_sheet.flip_h");
    let flip_v = ph2d_i18n::tr("panel.inspector.sprite_sheet.flip_v");
    let mut viu_empilhado = false;
    let mut viu_emparelhado = false;

    for &w in LARGURAS {
        let rects = rects_a(w);
        // A linha inteira é a que uma caixa de linha inteira ocupa — lida do PRODUTO, e não de uma
        // conta do recuo do cartão feita aqui.
        let inteira = rect_de(&rects, ph2d_panel_inspector::ids::INSP_SPRITE_CENTERED, w);
        let fx = rect_de(&rects, ph2d_panel_inspector::ids::INSP_SPRITE_FLIP_X, w);
        let fy = rect_de(&rects, ph2d_panel_inspector::ids::INSP_SPRITE_FLIP_Y, w);

        if o_par_cabe(&mut ts, inteira.w, flip_h, flip_v) {
            viu_emparelhado = true;
            assert!(
                (fx.y - fy.y).abs() < 0.01 && fy.x > fx.x + 1.0,
                "painel {w}: os dois nomes CABEM em meia linha e o painel partiu-a na mesma \
                 (fx={fx:?} fy={fy:?})"
            );
        } else {
            viu_empilhado = true;
            assert!(
                (fx.w - inteira.w).abs() < 0.01 && (fy.w - inteira.w).abs() < 0.01,
                "painel {w}: os nomes NAO cabem em meia linha e o painel emparelhou na mesma — \
                 e' o defeito de 2026-09-15, com o nome a sair cortado (fx={fx:?} fy={fy:?}, a \
                 linha inteira mede {:.2})",
                inteira.w
            );
            assert!(
                (fx.x - inteira.x).abs() < 0.01 && (fy.x - inteira.x).abs() < 0.01,
                "painel {w}: as duas caixas empilharam mas nao na coluna da seccao"
            );
            assert!(
                fy.y > fx.y + 1.0,
                "painel {w}: empilhadas e no MESMO y — uma tapa a outra"
            );
        }
    }

    // ⚠️ **O CONTROLO**: sem os dois regimes na escada, metade das asserções acima nunca corre e a
    // mutação sobrevive.
    assert!(
        viu_empilhado,
        "a escada nunca chegou ao regime ESTREITO — o gate nao mede o defeito do report"
    );
    assert!(
        viu_emparelhado,
        "a escada nunca chegou ao regime LARGO — o gate aprovaria um painel que empilha sempre"
    );
}

/// ⭐⭐ **E a ALTURA de uma linha de marcar é a de toda linha de propriedade.**
///
/// ⛔⛔ Ela era o literal `18` escrito em **treze** secções, e `18` é exactamente o lado da marca —
/// por isso o sinal enchia a caixa (report do dono de 2026-09-15, com foto). Curado, quatro dessas
/// cópias morreram na wave seguinte porque a porta passou a ser a dona da altura.
#[test]
fn a_altura_de_uma_linha_de_marcar_e_a_do_app() {
    let rects = rects_a(304.0);
    for id in [
        ph2d_panel_inspector::ids::INSP_SPRITE_CENTERED,
        ph2d_panel_inspector::ids::INSP_SPRITE_FLIP_X,
        ph2d_panel_inspector::ids::INSP_SPRITE_FLIP_Y,
        ph2d_panel_inspector::ids::INSP_SHEET_PREVIEW,
    ] {
        let r = rect_de(&rects, id, 304.0);
        assert!(
            (r.h - ph2d_tokens::ROW_H_PX).abs() < 0.01,
            "a linha {id:?} mede {:.2} de altura e a linha do app mede {:.2}",
            r.h,
            ph2d_tokens::ROW_H_PX
        );
    }
}
