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
///
/// ⚠️⚠️ **A agulha é `stroke: Some(`, e a 1.ª redacção contava `stroke: Some(contorno())`** — a
/// GRAFIA de uma porta, não a lei. A wave D do plano 35 acrescentou um segundo construtor
/// (`contorno_com_padrao`), e o gate reprovou **produto correcto**: as duas formas novas nascem
/// vestidas, só que por outra porta. *Um gate que fixa o nome de quem obedece à lei reprova a
/// segunda maneira de a obedecer* — e a lei aqui é *nascer com contorno*, seja qual for a tinta.
#[test]
fn every_shape_in_the_pattern_smoke_is_born_with_a_stroke() {
    let src = smoke_code();
    let formas = src.matches("..VecPath::default()").count();
    let vestidas = src.matches("stroke: Some(").count();
    // ⚠️ E nenhuma se declara SEM contorno: `stroke: None` satisfaria a contagem acima por
    // omissão em nenhum sítio, mas escrito explicitamente é a mesma regressão com outra cara.
    assert_eq!(
        src.matches("stroke: None").count(),
        0,
        "uma forma da cena declara-se SEM contorno - e' o report de 27/08 a voltar"
    );
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

/// ⭐⭐ **A CENA CONTÉM O FENÓMENO da wave D** — um contorno com padrão, e uma forma com padrão nos
/// DOIS.
///
/// ⚠️ Sem a segunda, a fileira `Fill | Stroke` **nunca é pintada** nesta cena e o smoke não alcança
/// metade da wave: com um alvo só não há escolha a oferecer. *Uma cena de smoke que não contém o
/// fenómeno mede outra coisa* — foi exactamente assim que o defeito do contorno sobreviveu a uma
/// wave inteira de gates verdes.
#[test]
fn the_scene_contains_a_patterned_stroke_and_a_shape_with_both() {
    let src = smoke_code();
    assert!(
        src.contains("fn contorno_com_padrao("),
        "a porta do contorno com padrao sumiu da cena - a wave D nao tem o que smokar"
    );
    // ⛔⛔ **E TODO padrão desta cena nasce ancorado NUMA FORMA** (report de 28/08): o construtor
    // ancora na origem do MUNDO por omissão, e numa faixa fina isso faz a fase sob o contorno não
    // ter relação nenhuma com a forma. ⚠️ Os dois construtores EXIGEM o canto (o compilador
    // garante-o); o que este gate impede é passá-lo à mão em vez de o derivar da mesma conta que
    // desenha a forma.
    let ancoras = src.matches("canto(").count();
    let tintas = src.matches("Some(pattern(").count() + src.matches("contorno_com_padrao(").count();
    assert!(
        ancoras >= tintas,
        "{tintas} tintas de padrao na cena e so' {ancoras} cantos derivados - alguma nasce na \
         origem do mundo, e a fase dela nao tem relacao com a forma"
    );
    assert_eq!(
        src.matches("stroke: Some(contorno_com_padrao(").count(),
        2,
        "a cena tem de ter DUAS formas de contorno com padrao: uma SO' contorno (o sujeito puro) e          uma com padrao nos dois (a unica que faz aparecer a fileira do alvo)"
    );
    assert_eq!(
        src.matches("fill: None").count(),
        1,
        "a forma SO' CONTORNO sumiu - e' ela que obriga o `Clamp` a enquadrar pela caixa do TRACO"
    );
}

/// ⚠️ **A largura do contorno com padrão é DERIVADA do ladrilho, nunca a `STROKE_W`.**
///
/// A `STROKE_W` de `0,03` é menos de um décimo de uma cópia: o motivo não se leria, e um smoke em
/// que a feature é invisível não é um smoke. *Uma largura escolhida à mão envelhece no dia em que o
/// tamanho da arte mudar.*
#[test]
fn the_patterned_stroke_is_wide_enough_to_show_its_art() {
    let src = smoke_src();
    let corte = src
        .find("fn contorno_com_padrao(")
        .expect("a porta do contorno com padrao");
    let corpo = &src[corte..corte + 400];
    assert!(
        corpo.contains("let lado = BOX / 6.0;") && corpo.contains("lado * 1.2"),
        "a largura do contorno com padrao deixou de sair do lado do ladrilho - com a `STROKE_W` da          cena (0,03) o motivo nao se le^"
    );
}
