//! Gates do **Colorize** no shell (`flip_colorize.rs`) — extraídos para o irmão pelo
//! teto de LOC do shell (HR-18, 600). A pilha de rabiscos, a posse do Ctrl+Z, o
//! re-Apply que SUBSTITUI em vez de empilhar, e a recusa que preserva o trabalho.

use super::*;
use ph2d_flip::Rgba;

/// Uma line-art de moldura FECHADA (um quadrado de tinta grossa) — o `boundaries` traça o
/// laço e o `colorize_with` preenche o interior onde a semente cai.
fn boxed_lineart() -> FlipDrawing {
    let mut s = FlipStroke::new();
    for &(x, y) in &[
        (0.0, 0.0),
        (100.0, 0.0),
        (100.0, 100.0),
        (0.0, 100.0),
        (0.0, 0.0),
    ] {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 4.0,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    let mut d = FlipDrawing::default();
    d.strokes.push(s);
    d
}

/// Uma semente no centro do quadrado (rótulo 0), da paleta de uma cor.
fn center_seed() -> (Vec<[u8; 4]>, Vec<Scribble>) {
    (
        vec![[220, 70, 70, 255]],
        vec![Scribble {
            label: 0,
            points: vec![Vec2::new(40.0, 50.0), Vec2::new(60.0, 50.0)],
            width: 2.0,
        }],
    )
}

/// **O re-Apply ao vivo SUBSTITUI, não empilha** — o coração do Trap/Bleed em tempo real
/// (6º smoke, "faça ficar em tempo real"). A operação viva restaura a base congelada e
/// reinsere; se ela reinserisse SEM restaurar, cada tique do slider empilharia outra pilha
/// de fills e a borda ficaria opaca de camadas mortas.
///
/// Mutação que o derruba: **pular a restauração da base** antes do 2º `insert_regions` —
/// o desenho cresceria para `base + n1 + n2` em vez de `base + n2`.
#[test]
fn the_live_reapply_replaces_the_regions_it_does_not_stack_them() {
    let (palette, seeds) = center_seed();
    let base = boxed_lineart();
    let base_len = base.strokes.len();

    // 1ª aplicação (Bleed no default).
    let mut d = base.clone();
    let n1 = insert_regions(
        &mut d,
        &palette,
        &seeds,
        1.0,
        2.0,
        ph2d_flip_colorize::DEFAULT_SQUEEZE,
    );
    assert!(n1 >= 1, "a moldura fechada tem de dar ao menos 1 região");
    assert_eq!(
        d.strokes.len(),
        base_len + n1,
        "1ª inserção: base + regiões"
    );

    // Re-Apply ao vivo: RESTAURA a base, reinsere com outro Bleed (mais colado).
    d.strokes.clone_from(&base.strokes);
    let n2 = insert_regions(
        &mut d,
        &palette,
        &seeds,
        1.0,
        2.0,
        ph2d_flip_colorize::squeeze_from_bleed(0.0),
    );
    assert_eq!(
        d.strokes.len(),
        base_len + n2,
        "o re-Apply SUBSTITUI (base + n2), nunca empilha (base + n1 + n2)"
    );
}

fn scr(n: u8) -> ([u8; 4], Vec<Vec2>) {
    (
        [n, 0, 0, 255],
        vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, f32::from(n))],
    )
}

/// **A disciplina da pilha de rabiscos** ("undo/redo ruim", 7º smoke): Ctrl+Z remove o
/// ÚLTIMO rabisco (LIFO), Ctrl+Shift+Z o devolve na ordem, e um rabisco NOVO descarta
/// os removidos — a lei de toda fila de redo (agir depois de desfazer mata o refazer).
#[test]
fn scribble_undo_pops_lifo_redo_restores_and_a_new_scribble_kills_the_redo() {
    let mut c = FlipColorize::default();
    for n in [1u8, 2, 3] {
        let (col, pts) = scr(n);
        c.push_scribble(col, pts);
    }
    assert!(c.can_undo_scribble() && !c.can_redo_scribble());

    c.undo_scribble(); // remove o 3
    c.undo_scribble(); // remove o 2
    assert_eq!(c.scribbles.len(), 1);
    assert_eq!(c.scribbles[0].0[0], 1, "o mais antigo fica");
    assert!(c.can_redo_scribble());

    c.redo_scribble(); // devolve o 2
    assert_eq!(c.scribbles.last().expect("redo").0[0], 2, "volta na ordem");

    // Um rabisco NOVO mata o redo pendente (o 3 nunca mais volta).
    let (col, pts) = scr(9);
    c.push_scribble(col, pts);
    assert!(
        !c.can_redo_scribble(),
        "agir depois de desfazer mata o refazer"
    );

    // Clear zera as DUAS filas.
    c.undo_scribble();
    c.clear();
    assert!(!c.can_undo_scribble() && !c.can_redo_scribble());
}

/// 🔴 **Um Apply RECUSADO não pode comer os rabiscos do artista** (auditoria 2026-07-20).
///
/// O `flip_colorize_apply` tem CINCO saídas que recusam o trabalho, e três delas mandam o
/// artista *corrigir e tentar de novo* ("a camada está travada", "desenhe a line-art
/// primeiro", "rabisque dentro das formas fechadas"). Enquanto o `mem::take` das sementes
/// morava no TOPO da função, obedecer era impossível: os rabiscos já tinham ido embora — e
/// o **Ctrl+Z não os devolvia**, porque a fila de removidos era limpa na linha seguinte e
/// o `undo_route` só é dono do atalho enquanto `can_undo_scribble()`. Uma recusa custava o
/// trabalho.
///
/// O fixture usa a recusa mais barata de alcançar headless (`gfx` ausente), mas a lei que
/// ele pina vale para as cinco: **só o SUCESSO consome**.
///
/// Mutação que sangra: devolver o `mem::take` (ou o `scribbles.clear()`) para antes das
/// validações ⇒ sobra 0 rabisco.
#[test]
fn a_refused_apply_keeps_the_scribbles_the_artist_drew() {
    let mut app = crate::App::new();
    app.flip_active = true;
    app.flip_style = Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Colorize,
        ..Default::default()
    });
    for n in [1u8, 2] {
        let (col, pts) = scr(n);
        app.flip_colorize.push_scribble(col, pts);
    }
    assert_eq!(app.flip_colorize.scribbles.len(), 2, "semeado");

    // `App::new()` é headless (`gfx: None`), então o Apply RECUSA na 1ª saída.
    app.flip_colorize_apply();

    assert_eq!(
        app.flip_colorize.scribbles.len(),
        2,
        "um Apply recusado devolveu {} rabiscos em vez de 2 — o artista perdeu o \
         trabalho e o Ctrl+Z não o traz de volta",
        app.flip_colorize.scribbles.len()
    );
    // E o Ctrl+Z continua alcançável (o Colorize ainda é dono do atalho).
    assert!(
        app.flip_colorize.can_undo_scribble(),
        "com rabisco pendente o Colorize tem de seguir dono do Ctrl+Z"
    );
}

/// **Vazio, o Colorize CEDE o atalho** — é o que deixa o Ctrl+Z pós-Apply cair no
/// Global (o dono do Apply). O contrato dos `can_*` é exatamente este gate de posse.
#[test]
fn an_empty_scribble_buffer_yields_the_chord() {
    let c = FlipColorize::default();
    assert!(!c.can_undo_scribble());
    assert!(!c.can_redo_scribble());
}
