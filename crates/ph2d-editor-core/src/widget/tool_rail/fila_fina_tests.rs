//! Os gates da fila fina: contraste em todo tema, a geometria da maquete, e nada de legenda.

use super::*;
use ph2d_tokens::{Color as Tok, TOOL_ROW_H_PX};

/// `fg` (com alpha) por cima de `bg` (opaco), canal a canal em sRGB — o que o compositor pinta.
fn sobre(fg: Tok, bg: Tok) -> Tok {
    let a = f64::from(fg.a) / 255.0;
    let mix = |f: u8, b: u8| (f64::from(f) * a + f64::from(b) * (1.0 - a)).round() as u8;
    Tok {
        r: mix(fg.r, bg.r),
        g: mix(fg.g, bg.g),
        b: mix(fg.b, bg.b),
        a: 0xFF,
    }
}

/// O chão da fila (o de `Barra::chao`, o tom da lista) sobre o fundo da janela.
fn chao(theme: Theme) -> Tok {
    sobre(
        ColorToken::KanbanList.resolve(theme),
        ColorToken::Bg0.resolve(theme),
    )
}

fn contraste(token: ColorToken, theme: Theme) -> f64 {
    let bg = chao(theme);
    sobre(token.resolve(theme), bg).contrast_ratio(&bg)
}

/// ⛔ Dono 12/11: todo controlo da fila ≥ 3:1 contra o chão (glifo, borda do escolhido), texto
/// ≥ 4,5:1 — nos 12 temas, claros e escuros. Imprime o mínimo de cada papel (`--nocapture`).
#[test]
fn a_fila_fina_tem_contraste_em_todo_tema() {
    let papeis: [(&str, ColorToken, f64); 5] = [
        ("glifo", TINTA.glifo, 3.0),
        ("aceso", TINTA.aceso, 4.5),
        ("escolhido", TINTA.escolhido, 3.0),
        ("borda", TINTA.borda, 3.0),
        ("texto", TINTA.texto, 4.5),
    ];
    let mut falhas = Vec::new();
    for (nome, token, piso) in papeis {
        let mut min = f64::MAX;
        let mut pior = Theme::ALL[0];
        for theme in Theme::ALL {
            let c = contraste(token, theme);
            if c < min {
                min = c;
                pior = theme;
            }
            if c < piso {
                falhas.push(format!("{nome} {token:?} em {theme:?}: {c:.2} < {piso}"));
            }
        }
        println!("fila fina · {nome:<9} {token:?}: mínimo {min:.2} ({pior:?})");
    }
    assert!(
        falhas.is_empty(),
        "contraste abaixo do piso:\n{}",
        falhas.join("\n")
    );
}

fn fila_de_repouso() -> ToolRail {
    let icone = |id, i| ToolRailEntry::icon(NodeId(id), "x", i).with_sub("CAPTION");
    let opcao = |id, f: &str| ToolRailEntry::compound(NodeId(id), "x", f, "SUB");
    let mut rail = ToolRail::new(
        NodeId(1),
        "",
        vec![
            icone(10, IconId::Move).active(),
            icone(11, IconId::Rotate),
            icone(12, IconId::Scale),
            icone(13, IconId::Move),
            ToolRailEntry::Divider,
            opcao(20, "Global"),
            opcao(21, "Selected"),
            ToolRailEntry::Divider,
            icone(30, IconId::Undo),
            icone(31, IconId::Redo),
            ToolRailEntry::Divider,
            opcao(40, "Object Mode ▾"),
        ],
    );
    let mut ts = TextSystem::without_system_fonts();
    medir_fila_fina(&mut rail, &mut ts);
    rail
}

/// ⭐ A geometria da maquete `.c15tl`: cabe na faixa de 26 px, glifos a 6 px dentro do grupo e 12 px
/// entre itens (medido de tinta a tinta), e a primeira tinta a `TOOL_ROW_PAD_PX` da borda.
#[test]
fn a_fila_fina_tem_as_medidas_da_maquete() {
    let rail = fila_de_repouso();
    assert!(cabe_na_fila_fina(&rail));
    let faixa = Rect::new(0.0, 100.0, 1600.0, TOOL_ROW_H_PX);
    let slots: Vec<_> = fila_fina_slots(&rail, faixa)
        .into_iter()
        .filter(|s| s.id.is_some())
        .collect();
    for s in &slots {
        assert!(
            s.rect.y >= faixa.y && s.rect.y + s.rect.h <= faixa.y + faixa.h,
            "{s:?}"
        );
    }
    let tinta = |i: usize| {
        let s = slots[i];
        let r = respiro(&rail.entries[s.index]);
        (s.rect.x + r, s.rect.x + s.rect.w - r)
    };
    assert!((tinta(0).0 - TOOL_ROW_PAD_PX).abs() < 0.01);
    let vao = |a: usize, b: usize| tinta(b).0 - tinta(a).1;
    assert!(
        (vao(0, 1) - TOOL_ROW_GROUP_GAP_PX).abs() < 0.01,
        "dentro do grupo"
    );
    assert!(
        (vao(3, 4) - TOOL_ROW_GAP_PX).abs() < 0.01,
        "glifo -> Global"
    );
    assert!(
        (vao(4, 5) - TOOL_ROW_GAP_PX).abs() < 0.01,
        "Global -> Selected"
    );
    assert!(
        (vao(6, 7) - TOOL_ROW_GROUP_GAP_PX).abs() < 0.01,
        "undo -> redo"
    );
    assert!(fila_fina_cabe(&rail, 1600.0));
    assert!(!fila_fina_cabe(&rail, 100.0));
}

/// Um campo numérico não entra na fila fina: sem a legenda dele não se sabe o que é.
#[test]
fn um_campo_numerico_mantem_a_fila_alta() {
    let mut rail = fila_de_repouso();
    rail.entries.push(ToolRailEntry::number(
        NodeId(50),
        "Opacity",
        "OPACITY",
        None,
    ));
    assert!(!cabe_na_fila_fina(&rail));
}
