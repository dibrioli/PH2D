//! **Dono 10/11: com o Input Map aberto, a Hierarchy e o Inspector alargam e estreitam pela borda.** A borda da coluna
//! vive FORA da área da aba; a shell larga o arrasto dela se a aba activa diz que o Move é seu. Estes gates pintam a
//! aba numa área central (as colunas dos lados ficam de fora) e passam o ponteiro pelas bordas como a shell o passa.

use super::{Mesa, View};
use ph2d_editor_core::doc_surface::PointerInput;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};

/// A área central entre as duas colunas: a Hierarchy em `x < 300`, o Inspector em `x ≥ 1300`.
const CENTRO: Rect = Rect {
    x: 300.0,  // LITERAL-PX-OK: a janela do teste
    y: 0.0,    // LITERAL-PX-OK: a janela do teste
    w: 1000.0, // LITERAL-PX-OK: a janela do teste
    h: 1000.0, // LITERAL-PX-OK: a janela do teste
};
/// Pontos sobre a borda interior de cada coluna (a costura vive dentro da coluna, colada à área).
const BORDA_DA_HIERARCHY: [(f32, f32); 3] = [(296.0, 500.0), (290.0, 520.0), (250.0, 540.0)];
const BORDA_DO_INSPECTOR: [(f32, f32); 3] = [(1302.0, 500.0), (1310.0, 520.0), (1360.0, 540.0)];

fn mesa(view: View) -> Mesa {
    let mut m = Mesa::aberta(view);
    m.paint_in(CENTRO);
    m
}

/// O Move pela porta da shell (`pointer_move` da aba activa), e a pintura na área central. `true` = a aba tomou-o.
fn mover(m: &mut Mesa, (x, y): (f32, f32)) -> bool {
    let ts = &mut m.ts;
    let (f, id) = m.hero.documents.active_surface_mut().expect("a aba activa");
    let input = PointerInput {
        text: ts,
        mods: Modifiers::default(),
        now_ns: 0,
    };
    let took = f.pointer_move(id, input, x, y);
    m.paint_in(CENTRO);
    took
}

fn botao(m: &mut Mesa, kind: PointerKind, (x, y): (f32, f32)) -> bool {
    let ts = &mut m.ts;
    let (f, id) = m.hero.documents.active_surface_mut().expect("a aba activa");
    let input = PointerInput {
        text: ts,
        mods: Modifiers::default(),
        now_ns: 0,
    };
    let took = f.pointer(id, input, kind, PointerButton::Primary, x, y);
    m.paint_in(CENTRO);
    took
}

#[test]
fn o_move_sobre_a_borda_das_colunas_nao_e_da_aba_em_nenhuma_vista() {
    for view in [View::Keyboard, View::List, View::TryIt] {
        let mut m = mesa(view);
        // Vindo de dentro da aba (o hover dela muda ao sair), e a seguir ao longo da borda, como num arrasto.
        let dentro = (CENTRO.x + CENTRO.w * 0.5, CENTRO.y + CENTRO.h * 0.5);
        for borda in [BORDA_DA_HIERARCHY, BORDA_DO_INSPECTOR] {
            mover(&mut m, dentro);
            for p in borda {
                assert!(
                    !mover(&mut m, p),
                    "{view:?}: o Move em {p:?} (borda de uma coluna, fora da aba) foi tomado pela aba — a shell \
                     largava o arrasto da largura da coluna"
                );
                assert!(
                    !botao(&mut m, PointerKind::Move, p),
                    "{view:?}: o Move pela porta `pointer` em {p:?} foi tomado"
                );
            }
        }
    }
}

#[test]
fn a_aba_continua_a_receber_o_clique_e_o_arrasto_que_nasceu_nela() {
    let mut m = mesa(View::Keyboard);
    let lista = m.segment(View::List);
    assert!(
        CENTRO.contains(lista.0, lista.1),
        "o segmento pinta-se na área"
    );
    assert!(
        botao(&mut m, PointerKind::Down, lista),
        "o Down dentro da aba é dela"
    );
    // Premido e arrastado para fora: o Move continua dela até o Up (o premido apaga-se ao sair).
    assert!(
        mover(&mut m, BORDA_DA_HIERARCHY[2]),
        "o arrasto que nasceu na aba é dela fora da área"
    );
    mover(&mut m, lista);
    botao(&mut m, PointerKind::Up, lista);
    assert_eq!(
        m.tab().doc.view(),
        View::List,
        "o clique no segmento troca a vista"
    );
    // Solto o botão, a borda volta a não ser da aba.
    assert!(!mover(&mut m, BORDA_DA_HIERARCHY[0]));
}
