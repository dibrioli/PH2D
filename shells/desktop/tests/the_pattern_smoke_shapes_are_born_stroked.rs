//! ⛔⛔ **TODA forma da cena do Texture Pattern NASCE COM CONTORNO** (Enio, 2026-08-27).
//!
//! > *"o contorno funciona com as shapes que eu desejo, mas não funcionam com os teus desenhos"*
//!
//! Este gate é o par executável do achado que fechou uma caça de **três mensagens** sobre um
//! *"pattern anula stroke"* que nunca existiu no produto:
//!
//! - a ferramenta de forma escreve `path.stroke = Some(..)` **incondicionalmente**
//!   ([`crates/ph2d-vec-edit/src/shape.rs`](../../../crates/ph2d-vec-edit/src/shape.rs)) ⇒ **toda**
//!   forma que o artista desenha tem contorno;
//! - a cena do smoke nascia de `..VecPath::default()`, que é `stroke: None`;
//! - e o `restyle_selected_strokes` **recusa por desenho** quem não tem um (*"ganhar um traço do
//!   nada seria a UI inventando geometria"*).
//!
//! ⇒ a secção *Stroke* ficava **pintada e inerte** — mas só nesta cena. E *um controlo nunca pintado
//! e um morto sob o dedo dão o mesmo report*, que nesta casa já custou duas waves.
//!
//! ⚠️ **A lição é da CENA, não do padrão:** uma cena montada por código não herda o que a
//! ferramenta de autoria garante. Ela tem de **nascer no estado em que o artista a encontraria**,
//! senão o smoke mede um objecto que o produto nunca produz — e o report que ele gera manda a
//! próxima janela caçar um defeito que não está lá.

use std::fs;
use std::path::Path;

fn smoke_src() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("texture_pattern_smoke.rs");
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// O fonte **sem comentários**.
///
/// ⚠️ **Não é higiene: o gate reprovou por causa disto na 1.ª corrida.** O doc-comment que explica a
/// lei cita `..VecPath::default()` para dizer o que estava errado, e a contagem leu-o como uma
/// 6.ª forma. *Um gate que lê a prosa sobre a lei em vez do código que a obedece mede o autor.*
fn smoke_code() -> String {
    smoke_src()
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **Tantos contornos quantas formas.** A conta é sobre o construtor (`..VecPath::default()`), que é
/// o que uma forma nova acrescenta — então a 8.ª forma desta cena só passa se nascer vestida.
#[test]
fn every_shape_in_the_pattern_smoke_is_born_with_a_stroke() {
    let src = smoke_code();
    let formas = src.matches("..VecPath::default()").count();
    let vestidas = src.matches("stroke: Some(contorno())").count();
    assert!(
        formas > 0,
        "a cena do smoke deixou de construir formas - este gate ficou sem sujeito"
    );
    assert_eq!(
        vestidas, formas,
        "{formas} formas na cena e so' {vestidas} com contorno: a que falta vai fazer a seccao \
         Stroke parecer MORTA, e o report volta como \"o padrao anula o contorno\""
    );
}

/// ⚠️ **E o contorno tem de ser VISÍVEL** — largura maior que zero e opaco.
///
/// Um `StrokeSpec` de largura `0` ou alfa `0` satisfaz o gate acima e **não desenha nada**: o
/// artista veria exactamente o mesmo que via antes, e a cura teria a forma certa e o efeito nenhum.
/// *Contar o que foi FEITO não é contar o que foi ENTREGUE.*
#[test]
fn the_smoke_stroke_is_actually_visible() {
    let src = smoke_src();
    let corte = src
        .find("fn contorno()")
        .expect("a porta unica do contorno da cena");
    let corpo = &src[corte..corte + 200];
    assert!(
        corpo.contains("STROKE_W"),
        "a largura deixou de sair da constante da cena"
    );
    let w = src
        .lines()
        .find(|l| l.trim_start().starts_with("const STROKE_W"))
        .expect("a constante existe");
    let n: f64 = w
        .split('=')
        .nth(1)
        .and_then(|s| s.split(';').next())
        .and_then(|s| s.trim().parse().ok())
        .expect("a largura e' um numero literal");
    assert!(n > 0.0, "largura {n} nao desenha traco nenhum");
    assert!(
        corpo.contains(", 255)"),
        "a cor do contorno nao e' opaca - um traco de alfa 0 passa o gate irmao e nao se ve^"
    );
}
