//! ⭐⭐⭐ **Uma lista LONGA cabe na região e ROLA** — a metade que o clamp sozinho não cura.
//!
//! # Porque este ficheiro é separado do gate do verbo
//!
//! São duas leis distintas sobre o mesmo mecanismo, e populações diferentes as expõem:
//!
//! - **virar para cima** quando abaixo não cabe — mede-se com **5** entradas numa janela de tablet
//!   (`action_verb_is_a_dropdown.rs`);
//! - **rolar** quando não cabe de nenhum lado — precisa de uma lista que não caiba **em lado
//!   nenhum**, e a única do Inspector que chega lá é a do *«Rides Parent Anchor»*: uma entrada por
//!   âncora do pai, até **64**, mais o «—».
//!
//! ⚠️ **O clamp sem a rolagem é uma meia-cura NOMEADA:** o `popover_rect_clamped` **encolhe** o
//! painel quando nem abaixo nem acima cabe a lista inteira — e sem rolagem as entradas de baixo
//! ficam desenhadas fora dele, que é exactamente o defeito que o painel autorado já pagou (*«o
//! `popover_rect_clamped` fazia o trabalho dele e ninguém fazia o resto»*).
//!
//! ⚠️ **O sujeito é a PORTA, não a §12.** Os quatro seletores do Inspector passam pela mesma
//! função (`paint_frame_shared::paint_open_popover`); este ficheiro escolhe a lista que produz o
//! fenómeno, e o que ele defende vale para os quatro.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::InspectorAnchorInfo;
use ph2d_editor_core::widget::DROPDOWN_SCROLLBAR_ID;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_anchor};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// ⚠️ **64 âncoras é o CAP do modelo**, não um número escolhido: é o comprimento de
/// `ids::INSP_MOUNT_OPT`, e há gate na shell a ligá-lo ao cap do componente.
fn anchor_info() -> InspectorAnchorInfo {
    InspectorAnchorInfo {
        entity_bits: 0xABCD_1234,
        rows: Vec::new(),
        present: true,
        selected_count: 1,
        mixed: false,
        parent_anchors: (0..ids::INSP_MOUNT_OPT.len())
            .map(|i| format!("anchor_{i:02}"))
            .collect(),
        mount: None,
        mount_offset: [0.0, 0.0],
        vis_in_editor: false,
        vis_at_runtime: false,
    }
}

/// Pinta com o seletor da §12 aberto e devolve o que a pintura registou.
fn painted() -> (MockPanelHost, Vec<(ph2d_a11y::NodeId, Rect)>) {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_anchor(Some(anchor_info()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ids::INSP_MOUNT_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    set_current_inspector_anchor(None);
    (h, rects)
}

/// ⭐⭐⭐ **O painel fica DENTRO da região, e as duas alturas da rolagem são publicadas.**
///
/// ⚠️ **Três metades, e a primeira é sobre a FIXTURA:** ela mede que a lista de facto não cabe
/// (`content_h > visible_h`), senão as outras duas passariam por caber em vez de por rolar.
///
/// **Mutações que devem sangrar:** trocar o `popover_rect_clamped` pelo `popover_rect` na porta ·
/// apagar o `set_panel_content_h`/`set_panel_visible_h` do fim do `paint`.
#[test]
fn a_long_list_is_clamped_to_the_region_and_publishes_its_scroll_extents() {
    let regiao = HeroLayout::for_viewport(VIEWPORT).popover_region();
    let (h, _) = painted();

    let (dono, painel) = h
        .store()
        .dropdown_popover()
        .expect("o rect do popover nao foi publicado");
    assert_eq!(
        dono,
        ids::INSP_MOUNT_PICK,
        "o popover publicado e' de outro seletor"
    );

    let content_h = h
        .store()
        .panel_content_h(ids::INSP_MOUNT_PICK)
        .expect("a altura do CONTEUDO nao foi publicada — a roda nao teria ate' onde rolar");
    let visible_h = h
        .store()
        .panel_visible_h(ids::INSP_MOUNT_PICK)
        .expect("a altura VISIVEL nao foi publicada");

    assert!(
        content_h > visible_h,
        "a fixtura NAO produz o fenomeno: {} entradas cabem no painel (conteudo {content_h:.1}, \
         visivel {visible_h:.1}) — este gate estaria a passar por caber, nao por rolar",
        ids::INSP_MOUNT_OPT.len() + 1
    );
    assert!(
        painel.y >= regiao.y && painel.y + painel.h <= regiao.y + regiao.h,
        "o painel do popover ({:.1}..{:.1}) sai da regiao do chrome ({:.1}..{:.1})",
        painel.y,
        painel.y + painel.h,
        regiao.y,
        regiao.y + regiao.h
    );
    assert!(
        (visible_h - painel.h).abs() < 0.5,
        // LITERAL-PX-OK: meio pixel é a tolerância de igualdade entre duas medidas do MESMO rect
        "a altura visivel publicada ({visible_h:.1}) nao e' a do painel pintado ({:.1})",
        painel.h
    );
}

/// ⭐⭐ **Nada é registado FORA do painel, e a barra de rolagem existe.**
///
/// ⚠️ **A metade que o olho não vê:** uma linha rolada para fora continua a ter um rect, e
/// registá-lo inteiro faria o clique apanhar uma opção **por baixo** do popover — o artista carrega
/// numa secção e escolhe uma âncora.
///
/// **Mutação que deve sangrar:** registar `option_rect_in_scrolled` inteiro em vez da parte
/// recortada ao painel.
#[test]
fn nothing_is_registered_outside_the_panel_and_the_scrollbar_is_there() {
    let (_, rects) = painted();
    let (_, barra) = rects
        .iter()
        .find(|(n, _)| *n == DROPDOWN_SCROLLBAR_ID)
        .copied()
        .expect("a barra de rolagem nao foi hit-registada — a lista longa seria imovel ao arrasto");
    assert!(barra.h > 0.0, "a barra de rolagem tem area zero: {barra:?}");

    let mut vistas = 0usize;
    for &id in ids::INSP_MOUNT_OPT.iter() {
        let Some((_, r)) = rects.iter().find(|(n, _)| *n == id).copied() else {
            continue; // rolada para fora — e é exactamente isso que se espera de uma lista longa
        };
        vistas += 1;
        assert!(
            r.h > 0.0,
            "uma entrada visivel ficou com altura zero: {r:?}"
        );
    }
    assert!(
        vistas > 0 && vistas < ids::INSP_MOUNT_OPT.len(),
        "esperava-se uma FATIA da lista registada (viu {vistas} de {}) — todas registadas quer \
         dizer que se registou o que esta' fora do painel; nenhuma, que o popover nao pintou",
        ids::INSP_MOUNT_OPT.len()
    );
}
