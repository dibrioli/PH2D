//! **O painel do Grid não oferece uma cor que o canvas não pode entregar.**
//!
//! ## O defeito, medido em 2026-08-30
//!
//! A secção *Display* pintava uma fileira **Color** com um quadradinho clicável que abria o
//! selector de cor completo. O canvas nunca via aquele RGB: `grid_snap::render::grid_line_color`
//! deriva o R/G/B do fundo (`ColorToken::Bg0`, deslocado em luminância) e lê do utilizador **só o
//! alfa**. ⇒ o artista escolhia vermelho, o quadradinho ficava vermelho, e a grade continuava
//! cinzenta.
//!
//! ⚠️ **A lei de renderização é do Enio (2026-07-02) e FICA intocada** — está escrita no doc
//! daquela função: *«o grid sempre lê como um contraste relativo subtil, seja qual for o tema»*.
//! Uma cor escolhida à mão pode ser ilegível sobre o fundo, e ele escolheu a robustez. O que saiu
//! foi o **controlo**, não a lei.
//!
//! ⚠️ E o que sobrava dele — o alfa — já tinha porta: o slider **Opacity**, uma fileira acima, que
//! o renderer multiplica (`base_alpha × opacity`). *Duas portas para uma grandeza é o que este
//! repo evita*, e a segunda vinha embrulhada num selector que prometia outra coisa.
//!
//! ## As duas metades
//!
//! A primeira exige que o selector **não seja pintado nem registado**. A segunda é o controlo
//! positivo: a secção *Display* continua viva, com o Opacity e o Show Overlay no lugar — senão
//! este ficheiro ficaria verde sobre um painel que deixou de desenhar a secção inteira.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_grid_snap::state::GridSnapPanelState;
use ph2d_panel_grid_snap::{GridSnapPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

#[test]
fn the_grid_panel_offers_no_colour_it_cannot_deliver() {
    let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
    let mut st = GridSnapPanelState;
    let regs = host.paint::<GridSnapPanel>(&mut st, VIEWPORT);

    // ⚠️ Metade JUSTA primeiro: a secção Display tem de estar viva. Sem isto, um painel que
    // deixasse de a pintar por inteiro passaria — e o gate estaria a medir a sua própria fixtura.
    for (name, id) in [
        ("Show overlay", ids::GS_SHOW_OVERLAY),
        ("Opacity", ids::GS_OPACITY_SLIDER),
    ] {
        assert!(
            regs.iter().any(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0),
            "a fileira {name} da secção Display não foi pintada — a secção inteira sumiu, e a \
             asserção abaixo não provaria nada"
        );
    }

    assert!(
        !regs.iter().any(|(w, _)| *w == ids::GS_COLOR_PICKER),
        "o selector de cor do Grid voltou a ser registado. O canvas lê dele SÓ o alfa \
         (`grid_line_color` deriva o RGB do fundo — lei do Enio, 2026-07-02), e o alfa já tem o \
         slider Opacity. Um quadradinho que pinta um vermelho que a grade nunca vai ter é a \
         forma mais cara deste defeito: o artista confirma com os olhos e está errado."
    );
}
