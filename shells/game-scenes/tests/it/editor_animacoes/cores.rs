//! **As cores das animações na folha** ([estudo](../../../../../docs/GameEngineFuncional/estudo_editor_de_animacoes_2026-11-07.md)
//! §4): cada gesto pelas portas reais da aba, conferido no que a pintura pôs em cada célula.

use super::*;
use ph2d_editor_core::SheetImage;
use std::sync::Arc;

/// A folha do herói, sem pixels de verdade: as cores só precisam da grelha.
fn com_folha(m: &mut Mesa) {
    let sheet = SheetImage::new(Arc::new(vec![255; 8 * 4 * 4]), 8, 4).expect("a folha");
    m.doc().snap.sheet = Some(sheet);
}

/// As animações `a0..an` a usar todas a célula 0, e a de índice `aberta` aberta.
fn cinco_na_celula_0(m: &mut Mesa, aberta: usize) {
    let e = Entity::from_bits(m.e);
    let mut lib = SpriteAnimations::new();
    for i in 0..5 {
        lib.insert(AnimationTag::new(format!("a{i}"), 0, 1))
            .unwrap();
    }
    *m.sim.world_mut().get_mut::<SpriteAnimations>(e).unwrap() = lib;
    m.frame();
    m.doc().selected = aberta;
    com_folha(m);
    m.paint();
}

fn celula(m: &mut Mesa, c: u32) -> ph2d_app_sprite_anim::editor::paint_sheet::PaintedCell {
    m.doc()
        .painted_cells
        .iter()
        .find(|p| p.cell == c)
        .cloned()
        .expect("a célula usada foi pintada")
}

/// Decisão do dono, 07/11: um quadro em várias animações empilha até 3 faixas e o resto vira «+N».
/// *Mutação: `BANDS_MAX = 5`, ou o `extra` sempre `0`.*
#[test]
fn a_celula_empilha_ate_tres_faixas_e_o_resto_vira_mais_n() {
    let mut m = Mesa::new();
    cinco_na_celula_0(&mut m, 0);
    let c = celula(&mut m, 0);
    assert_eq!((c.bands.as_slice(), c.extra), (&[0, 1, 2][..], 2));
    assert!(
        m.doc().painted_cells.iter().all(|p| p.cell != 7),
        "célula sem dono fica sem faixa"
    );
    // A foto de 07/11 mostrou o «+2» como «…» numa caixa curta.
    assert_eq!(m.paint_in(AREA), Vec::<String>::new(), "o «+N» cabe");
}

/// A animação aberta nunca fica escondida no «+N», mesmo sendo a última da lista.
/// *Mutação: tirar o passo que marca a aberta primeiro em `cell_users`.*
#[test]
fn a_aberta_vem_primeiro_mesmo_sendo_a_ultima_da_lista() {
    let mut m = Mesa::new();
    cinco_na_celula_0(&mut m, 4);
    let c = celula(&mut m, 0);
    assert_eq!((c.bands.as_slice(), c.extra), (&[4, 0, 1][..], 2));
}

/// O arrasto na folha aparece sob a mão, antes de soltar.
#[test]
fn a_faixa_nasce_durante_o_arrasto() {
    let mut m = Mesa::new();
    com_folha(&mut m);
    m.paint();
    let livre = 30;
    assert!(m.doc().cell_users()[livre].is_empty());
    let at = m.cell(livre as u32);
    m.pointer(PointerKind::Down, at);
    assert_eq!(m.doc().cell_users()[livre], [0]);
}

/// Passar o mouse lista TODAS, inclusive as que o «+N» esconde.
#[test]
fn passar_o_mouse_lista_quem_o_mais_n_esconde() {
    let mut m = Mesa::new();
    cinco_na_celula_0(&mut m, 0);
    let at = m.cell(0);
    m.drag_to(at);
    let users = m.doc().cell_users();
    assert_eq!(
        m.doc().hovered_names(&users),
        ["a0", "a1", "a2", "a3", "a4"]
    );
    let longe = m.cell(30);
    m.drag_to(longe);
    assert!(
        m.doc().hovered_names(&users).is_empty(),
        "célula sem dono não lista nada"
    );
}

/// A lista mostra a cor que a animação guarda, e This frame só abre com um quadro escolhido.
#[test]
fn this_frame_pede_um_quadro_escolhido() {
    let mut m = Mesa::new();
    assert_eq!(m.doc().this_frame_label(), "\u{2014}");
    let campo = centre(m.doc().layout.this_frame);
    m.click(campo);
    assert!(m.doc().editing.is_none(), "sem quadro, o campo não abre");
    let quadro = centre(m.doc().layout.frames[1].0);
    m.click(quadro);
    m.frame();
    assert_eq!(m.doc().this_frame_label(), "1");
    m.click(campo);
    assert!(matches!(m.doc().editing, Some((doc::Field::FrameMs(1), _))));
}

/// Os três modos escrevem o modelo pela porta de sempre, e o cartão de baixo soma o tempo do ciclo.
#[test]
fn o_segmentado_e_o_total_do_ciclo() {
    let mut m = Mesa::new();
    for (i, modo) in AnimCycleMode::ALL.into_iter().enumerate().rev() {
        let r = centre(m.doc().layout.modes[i]);
        m.click(r);
        m.frame();
        assert_eq!(m.doc().mode(), Some(modo));
    }
    assert_eq!(m.doc().total_ms(), 600, "seis quadros de 100 ms");
}

/// Dono, 07/11: o tempo da animação e o corte da folha têm cada um a sua moldura, com os seus dois campos dentro.
#[test]
fn o_tempo_e_o_corte_tem_molduras_separadas() {
    for w in [1200.0, 420.0] {
        // LITERAL-PX-OK: a janela do teste
        let mut m = Mesa::new();
        m.paint_in(Rect::new(0.0, 0.0, w, AREA.h));
        let l = m.doc().layout.clone();
        let dentro = |g: Rect, r: Rect| {
            r.x >= g.x && r.y >= g.y && r.x + r.w <= g.x + g.w && r.y + r.h <= g.y + g.h
        };
        assert!(
            dentro(l.time_group, l.fps) && dentro(l.time_group, l.this_frame),
            "{w}"
        );
        assert!(
            dentro(l.grid_group, l.cols) && dentro(l.grid_group, l.rows),
            "{w}"
        );
        let (a, b) = (l.time_group, l.grid_group);
        let apart = a.x + a.w <= b.x || a.y + a.h <= b.y;
        assert!(apart, "as molduras não se pisam a {w}: {a:?} {b:?}");
        assert!(l.modes[0].y >= b.y + b.h, "o segmentado fica abaixo a {w}");
    }
}

/// Dono, 07/11: os números dos quadros ficam à vista em repouso, na tira e nas células da animação aberta.
/// *Mutação: tirar o número de `paint_strip` ou de `paint_sheet`.*
#[test]
fn os_numeros_dos_quadros_ficam_na_tira_e_na_folha() {
    let mut m = Mesa::new();
    com_folha(&mut m);
    let e = m.e;
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    let ((), medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| {
        m.docs()
            .paint(e, AREA, &mut scene, &mut ts, &Theme::default());
    });
    for n in 1..=6 {
        let vezes = medidos.iter().filter(|t| t.texto == n.to_string()).count();
        assert!(vezes >= 2, "o {n} aparece na tira e na folha, leu {vezes}");
    }
}
