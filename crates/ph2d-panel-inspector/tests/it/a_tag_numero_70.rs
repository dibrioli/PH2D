//! ⭐ **A tag número 70 escolhe-se** (U5, auditoria GEF 07: «tags 65+ têm de ser escolhíveis») — nas
//! listas de tags do Ray Sensor e do Nav Agent, com a árvore de 70 tags: o chip abre a lista, a roda
//! leva-a ao fim, e o clique REAL na 70.ª chega ao barramento com o id dela.
//!
//! **Mutações que devem sangrar:** um tecto de 64 em `opcoes_da_arvore` · o `popovers::publica` não
//! registar as opções.

use crate::o_cartao_da_fisica::{achado, clica};
use crate::seam_ray::{assente_com, edicoes, nos_dois_modos, pinta, raio};
use ph2d_a11y::NodeId;
use ph2d_editor_core::nav_edits::{NavAlvoModo, NavFieldEdit};
use ph2d_editor_core::ray_edits::{RayFieldEdit, RaySee};
use ph2d_editor_core::screens::hero::InspectorTagRow;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{InspectorPanel, set_current_inspector_nav, set_current_inspector_ray};
use ph2d_ui_testkit::MockPanelHost;

const N: u64 = 70;

fn arvore() {
    ph2d_panel_inspector::set_current_tag_tree(
        (1..=N)
            .map(|t| InspectorTagRow {
                id: 1000 + t,
                path: format!("Tag{t}"),
                label: format!("Tag{t}"),
                depth: 0,
            })
            .collect(),
    );
}

/// Rola a lista aberta até a última opção aparecer e devolve o rect dela.
fn ate_a_ultima(
    h: &mut MockPanelHost,
    mut pinta: impl FnMut(&mut MockPanelHost) -> Vec<(NodeId, Rect)>,
    primeira: NodeId,
    ultima: NodeId,
) -> Rect {
    let mut r = pinta(h);
    for _ in 0..8 {
        if let Some(q) = achado(&r, ultima) {
            return q;
        }
        let p = h
            .store()
            .dropdown_popover()
            .map(|(_, p)| p)
            .or_else(|| achado(&r, primeira))
            .expect("a lista aberta");
        let _ = h.wheel_at(p.x + p.w * 0.5, p.y + p.h * 0.5, -20000.0);
        r = pinta(h);
    }
    panic!("a 70.ª opção nunca apareceu na lista");
}

#[test]
fn a_tag_numero_70_escolhe_se() {
    arvore();
    nos_dois_modos(|c| {
        let mut info = raio();
        info.only_see = RaySee::OnlyTag;
        let (mut h, mut st, r) = assente_com(&info, c, true);
        let chip = achado(&r, i::INSP_RAY_SEE_TAG).expect("o chip da tag");
        let _ = clica(&mut h, &mut st, chip);
        let alvo = ate_a_ultima(
            &mut h,
            |h| pinta(h, &mut st, c),
            i::INSP_RAY_SEE_TAG_OPT.id(0),
            i::INSP_RAY_SEE_TAG_OPT.id(N as usize - 1),
        );
        let saiu = edicoes(clica(&mut h, &mut st, alvo));
        assert_eq!(saiu, vec![RayFieldEdit::SeeTag(1000 + N)], "cartões {c}");
        set_current_inspector_ray(None);
    });
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

#[test]
fn a_tag_numero_70_escolhe_se_no_nav() {
    use crate::a_seccao_nav_esta_viva::{agente, edicoes, host, info};
    arvore();
    let (mut h, mut st) = host(info(None, Some(agente(NavAlvoModo::Tag))));
    let _ = h.paint::<InspectorPanel>(&mut st, crate::seam_ray::VIEWPORT);
    h.set_dropdown_open(i::INSP_NAV_TAG_PICK, true);
    let alvo = ate_a_ultima(
        &mut h,
        |h| h.paint::<InspectorPanel>(&mut st, crate::seam_ray::VIEWPORT),
        i::INSP_NAV_TAG_OPT.id(0),
        i::INSP_NAV_TAG_OPT.id(N as usize - 1),
    );
    for ev in h.click_at(alvo.x + alvo.w * 0.5, alvo.y + alvo.h * 0.5) {
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    assert_eq!(edicoes(&mut h), vec![NavFieldEdit::AlvoTag(1000 + N)]);
    set_current_inspector_nav(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}
