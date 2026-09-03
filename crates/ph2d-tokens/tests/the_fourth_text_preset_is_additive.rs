//! ⭐⭐⭐ **O 4.º modo de texto é ADITIVO — os três que existem não se mexem.**
//!
//! Enio, 2026-09-01: *«Fizemos um excelente trabalho com as fonts do nosso app e isso custou muito
//! esforço e tempo. Talvez a versão nova do Vello consiga resultados melhores que o que temos, mas,
//! por precaução vamos manter o que temos e colocar o que o vello consegue fazer como mais uma
//! opção de aparência das fonts. CUidado para não destruir o que temos.»*
//!
//! ⇒ isto não é um gate de estilo: é **a instrução do dono, executável**.
//!
//! # O que o `CrispEmbolden` traz que os outros não conseguem exprimir
//!
//! Os três presets históricos pedem massa ao **eixo `wght` da fonte**, que engorda hastes verticais
//! e barras horizontais **juntas** — foi assim que o desenhador as desenhou. O `font_embolden` do
//! Vello 0.10 dilata a **outline** com `x` e `y` **independentes**, e é isso que permite engrossar
//! só as hastes. ⚠️ E funciona em fonte **não-variável**, onde o `weight_boost` é literalmente
//! inerte.
//!
//! # ⛔ O que este ficheiro NÃO consegue afirmar
//!
//! Que o resultado é mais bonito, ou que `0,2 px` é o número certo. O arnês desta casa não carrega
//! fontes — texto não produz tinta — e a aparência de um glifo não é mensurável aqui. **Quem mede é
//! o dono, a olhar.** O que se gateia é a parte que uma máquina pode afirmar: *os três antigos
//! ficaram exactamente como estavam, e o novo é de facto diferente*.

use ph2d_tokens::TextRendering;

/// **Os três presets históricos declaram dilatação ZERO.**
///
/// ⚠️ Não é um detalhe de gosto: com `(0.0, 0.0)` o Vello **nem entra** no caminho da expansão de
/// outline (`vello_encoding::glyph_cache` compara o `amount` contra `Diagonal2::new(0,0)` antes de
/// chamar o `expand_path`) ⇒ a rasterização deles é a mesma instrução por instrução.
///
/// **Mutação que deve sangrar:** pôr um valor não-nulo em qualquer um dos três — o trabalho de
/// fontes que o dono mandou preservar passaria a ser rasterizado por outro caminho.
#[test]
fn the_three_historic_presets_declare_zero_embolden() {
    // ⚠️ Derivado: *todos menos o novo*. Escrever os três à mão faria este gate ficar cego ao
    // quinto preset que alguém acrescente sem dilatação — e o ponto dele é justamente que a
    // dilatação seja a EXCEPÇÃO, não uma lista de três nomes.
    for p in TextRendering::ALL
        .iter()
        .copied()
        .filter(|p| *p != TextRendering::CrispEmbolden)
    {
        assert_eq!(
            p.params().embolden_px,
            (0.0, 0.0),
            "o preset {} ganhou dilatacao de outline: o caminho de rasterizacao dele MUDOU, \
             e o dono pediu explicitamente para nao destruir o que ja' existia",
            p.display_name()
        );
    }
}

/// **E o preset novo é de facto diferente** — senão seria uma opção que não oferece nada.
#[test]
fn the_new_preset_actually_emboldens_and_only_in_x() {
    let p = TextRendering::CrispEmbolden.params();
    assert!(
        p.embolden_px.0 > 0.0,
        "o CrispEmbolden nao engrossa nada: e' uma entrada de menu que nao faz diferenca"
    );
    assert_eq!(
        p.embolden_px.1, 0.0,
        "o CrispEmbolden engrossa o Y — e a razao de ele existir e' engrossar SO' o X: \
         a barra horizontal e' o que fecha os olhos das letras no corpo pequeno, e o eixo wght \
         ja' sabe engordar os dois juntos"
    );
}

/// **O ciclo do menu passa pelos QUATRO e volta ao princípio.**
///
/// ⚠️ Um `next()` que saltasse o preset novo deixá-lo-ia **inalcançável por teclado** enquanto o
/// menu o mostra — a espécie de controlo morto que este repo caça: pintado, registado, e sem via.
#[test]
fn the_cycle_visits_every_preset_exactly_once() {
    let mut seen = Vec::new();
    let mut cur = TextRendering::ALL[0];
    for _ in 0..TextRendering::ALL.len() * 2 {
        seen.push(cur.id());
        cur = cur.next();
        if cur == TextRendering::ALL[0] {
            break;
        }
    }
    let expected: Vec<&str> = TextRendering::ALL.iter().map(|p| p.id()).collect();
    assert_eq!(
        seen, expected,
        "o ciclo do menu nao visita a lista canonica uma vez cada, pela ordem dela"
    );
}

/// **Cada preset tem id e nome próprios** — dois presets com o mesmo id colidiriam no
/// `~/.ph2d/prefs.txt` e um deles seria inalcançável depois de reiniciar.
#[test]
fn no_two_presets_share_an_id_or_a_name() {
    let all = TextRendering::ALL;
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a.id(), b.id(), "dois presets partilham o id {}", a.id());
            assert_ne!(
                a.display_name(),
                b.display_name(),
                "dois presets partilham o nome {}",
                a.display_name()
            );
        }
    }
}
