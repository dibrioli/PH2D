//! Report do dono 28/10: «Não consigo escrever em Description» (e nos comentários).

use crate::verso::{abre, alvo, toca};

fn texto(m: &mut crate::Mesa, id: ph2d_kanban_model::KanbanId, desc: bool) -> String {
    m.docs().probe_back_text(id.0, desc).unwrap_or_default()
}

#[test]
fn verso_um_clique_na_descricao_vazia_e_as_teclas_escrevem() {
    let (mut m, id, _) = abre("Pick the music");
    toca(&mut m, id, "DescArea");
    assert!(m.docs().probe_back_state(id.0).2, "o clique entra em edição");
    m.escreve("Hello world");
    m.quadro();
    assert_eq!(texto(&mut m, id, true), "Hello world");
}

#[test]
fn verso_um_clique_no_comentario_e_as_teclas_escrevem() {
    let (mut m, id, _) = abre("Pick the music");
    toca(&mut m, id, "CommentArea");
    m.escreve("Nice");
    m.quadro();
    assert_eq!(texto(&mut m, id, false), "Nice");
    let _ = alvo(&mut m, id, "CommentArea");
}

/// O acento de tecla morta (´ + e = é): o acento espera no caret e a letra composta entra.
#[test]
fn verso_a_tecla_morta_compoe_na_descricao() {
    use ph2d_editor_core::ime::ComposeKey;
    use ph2d_editor_core::screens::hero::board_keys::BoardKey;
    let (mut m, id, _) = abre("Pick the music");
    toca(&mut m, id, "DescArea");
    assert!(m.hero.handle_compose_key(ComposeKey::Dead('´')), "o acento é da composição");
    m.tecla(BoardKey::Char('e'), ph2d_host::Modifiers::default(), Some("é"));
    m.escreve("x");
    m.quadro();
    assert_eq!(texto(&mut m, id, true), "éx");
}

/// O menu «Aa» pinta-se por cima do verso, e o clique numa opção que cai sobre o «Save» muda o
/// estilo em vez de guardar.
#[test]
fn verso_o_menu_aa_fica_por_cima_e_o_clique_e_dele() {
    let (mut m, id, _) = abre("Pick the music");
    toca(&mut m, id, "DescArea");
    m.escreve("Title");
    let area = alvo(&mut m, id, "DescArea");
    // O «Aa» é o 1.º botão da barra, no canto de cima do editor.
    let (xs, row) = (ph2d_tokens::Spacing::Xs.px(), ph2d_tokens::ROW_H_PX);
    let aa = ph2d_editor_core::zones::Rect::new(area.x + xs, area.y + xs, row, row);
    m.clica((aa.x + aa.w / 2.0, aa.y + aa.h / 2.0));
    let menu = alvo(&mut m, id, "DescMenu");
    let hits = m.docs().probe_back_hits(id.0);
    let i_menu = hits.iter().rposition(|h| h.0 == "DescMenu").unwrap();
    let i_save = hits.iter().rposition(|h| h.0 == "DescSave").unwrap();
    assert!(i_menu > i_save, "o menu é registado depois (por cima) do Save");
    // A 2.ª linha (Heading 1).
    m.clica((menu.x + menu.w / 2.0, menu.y + menu.h * 0.3));
    assert!(m.docs().probe_back_state(id.0).2, "continua em edição (não guardou)");
    assert_eq!(texto(&mut m, id, true), "# Title");
}

/// 📷 A descrição com texto escrito e o menu «Aa» aberto, nos dois temas, em
/// `docs/TrelloClone/smoke/D10/`.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --test it d10_foto -- --ignored`
#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/D10/"]
fn d10_foto_descricao_escrita_e_menu_aa() {
    use ph2d_tokens::Theme;
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        let (mut m, id, _) = abre("Pick the music");
        m.hero.theme = theme;
        m.ts = ph2d_text::TextSystem::new();
        m.quadro();
        toca(&mut m, id, "DescArea");
        m.escreve("Hello world");
        m.quadro();
        let mut s = ph2d_vector::VectorScene::new();
        m.pinta(&mut s);
        crate::j2_fotos::grava_em("D10", &format!("descricao_escrita_{tag}"), &s);
        let area = alvo(&mut m, id, "DescArea");
        let (xs, row) = (ph2d_tokens::Spacing::Xs.px(), ph2d_tokens::ROW_H_PX);
        m.clica((area.x + xs + row / 2.0, area.y + xs + row / 2.0));
        m.quadro();
        let mut s = ph2d_vector::VectorScene::new();
        m.pinta(&mut s);
        crate::j2_fotos::grava_em("D10", &format!("descricao_menu_aa_{tag}"), &s);
    }
}
