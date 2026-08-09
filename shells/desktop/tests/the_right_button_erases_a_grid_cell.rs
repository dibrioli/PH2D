//! **Arch-gate: o botão direito do Grid Stamp chega ao carimbo, e desarma DEPOIS de carimbar.**
//!
//! ## Por que um gate de TEXTO
//!
//! O `CanvasPaintTool` é **contrato congelado** (§6) e o `CanvasPointer` dele **não carrega botão** —
//! de propósito. Então "o direito apaga" não é uma propriedade do tool: é uma costura de SHELL, que
//! amostra o botão e o empurra fora de banda antes da entrega, exatamente como `painter_canvas_mods`
//! empurra Shift/Ctrl/Alt. O lado do tool é gateado por comportamento (`grid_stamp_settings::tests`,
//! que pinta uma célula e a apaga); o que NENHUM teste de unidade alcança é esta metade — o `match`
//! de botão vive dentro do `input_dispatch`, que exige janela.
//!
//! ## As duas propriedades
//!
//! **(1) O secundário chega.** Sem o braço no `match`, o direito cai no menu de contexto e o método
//! fica com metade do gesto que o define, sem erro e sem aviso.
//!
//! **(2) A ORDEM do fecho é load-bearing.** O `painter_canvas_up` entrega a fase `Up`, que ainda
//! carimba a cauda do traço — desarmar o flag ANTES dele faria o ÚLTIMO carimbo PINTAR em vez de
//! apagar, e o defeito seria uma célula fantasma no fim de cada arrasto: pequeno, intermitente, e
//! invisível para qualquer gate que só pergunte *"o flag foi desarmado?"*.
//!
//! ⚠️ Os dois asserts começam por um **controle positivo** (a âncora TEM de existir). Um gate de texto
//! cuja âncora se mudou de arquivo passa por vácuo — a cicatriz que o `keyboard.rs` acabou de deixar,
//! e que é pior que o defeito, porque ele volta a passar no dia em que a propriedade quebrar.

const DISPATCH: &str = include_str!("../src/input_dispatch.rs");
const GRID_ERASE: &str = include_str!("../src/input_dispatch/painter_grid_erase.rs");

/// O Down secundário é reivindicado pelo Grid Stamp.
///
/// **Mutação que deve sangrar:** remover o braço `(Secondary, Down) if … painter_grid_erase_down(..)`
/// do `match` de botão.
#[test]
fn the_secondary_down_reaches_the_grid_stamp() {
    assert!(
        DISPATCH.contains("PointerButton::Secondary, PointerKind::Down"),
        "controle positivo: o `match` de botão secundário mudou-se do `input_dispatch.rs` — este gate \
         estaria varrendo o arquivo errado e passaria por vácuo"
    );
    assert!(
        DISPATCH.contains("self.painter_grid_erase_down(evt.x, evt.y)"),
        "nenhum braço entrega o Down secundário ao Grid Stamp — o botão direito cai no menu de \
         contexto e o método fica sem a metade que apaga"
    );
}

/// O Up secundário fecha o gesto pela porta do Grid Stamp.
#[test]
fn the_secondary_up_closes_the_grid_stamp_gesture() {
    assert!(
        DISPATCH.contains("PointerButton::Secondary, PointerKind::Up"),
        "controle positivo: o braço de Up secundário sumiu do `input_dispatch.rs`"
    );
    assert!(
        DISPATCH.contains("self.painter_grid_erase_up()"),
        "o Up secundário não fecha o gesto de apagar — o flag ficaria armado e o PRÓXIMO traço, de \
         botão esquerdo, apagaria"
    );
}

/// **A ordem dentro do fecho:** carimbar a cauda primeiro, desarmar depois.
///
/// **Mutação que deve sangrar:** trocar as duas linhas de `painter_grid_erase_up`.
#[test]
fn the_gesture_is_disarmed_after_the_tail_is_stamped_not_before() {
    let body = fn_body(GRID_ERASE, "fn painter_grid_erase_up(&mut self)").expect(
        "controle positivo: `painter_grid_erase_up` não existe mais em `painter_grid_erase` — \
         o dono mudou-se de arquivo e este gate estaria varrendo o errado",
    );
    let stamp = body
        .find("self.painter_canvas_up()")
        .expect("o fecho não entrega mais a fase Up — a cauda do traço não seria carimbada");
    let disarm = body
        .find("self.set_painter_grid_erase(false)")
        .expect("o fecho não desarma mais o gesto");
    assert!(
        stamp < disarm,
        "o gesto é desarmado ANTES da fase Up: o último carimbo do arrasto pintaria em vez de apagar, \
         deixando uma célula fantasma no fim de cada gesto"
    );
}

/// O corpo de `fn <sig> {` até a chave que o fecha, contando profundidade. Um `find` de duas âncoras
/// no arquivo inteiro casaria com chamadas de OUTRA função — e é justamente a ordem que se afirma.
fn fn_body<'a>(src: &'a str, sig: &str) -> Option<&'a str> {
    let at = src.find(sig)?;
    let open = at + src[at..].find('{')?;
    let mut depth = 0usize;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[open..open + i]);
                }
            }
            _ => {}
        }
    }
    None
}
