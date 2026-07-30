//! **Arch-gate: o smoke de expressões dirige o CATÁLOGO e o roteiro dele descreve UI que
//! EXISTE** (FASE 0.5 do plano 12).
//!
//! ## Os dois defeitos, medidos (auditoria de 2026-07-29)
//!
//! * **D-F** — a cena exercitava **ZERO das 50 receitas**: ela autorava três fórmulas
//!   escritas à mão. O catálogo que o artista de fato usa — galeria, linhas, knobs — nunca
//!   era tocado por um smoke, e é exatamente a metade que o Enio reprovou.
//! * **D-L** — o roteiro mandava usar um **widget DELETADO**: *"**Expression…** no menu
//!   abre um campo de texto"* e *"ESVAZIE o campo → volta aos keyframes"*. O campo inline
//!   morreu na W1 (grep por `EXPR_FIELD` dá zero). Pior: o passo 3 daquele roteiro é o
//!   gesto do **D-I** — o artista não acha o campo, usa o card, limpa a linha, Apply — e
//!   numa prop sem keys o objeto FICA. **O roteiro ensinava o bug.**
//!
//! ## Por que um gate de TEXTO
//!
//! O corpo do smoke é código de shell atrás de `PH2D_EXPR_SMOKE`, e o ROTEIRO é prosa: não
//! há teste de comportamento que possa lê-los. Um roteiro que descreve um widget morto é
//! pior que roteiro nenhum — ele faz o artista concluir que a feature está quebrada quando
//! ele não acha o controle, e faz a próxima LLM (fez esta) acreditar que o campo existe.
//!
//! ⚠️ E este gate NÃO é um espelho do smoke: ele afirma duas PROPRIEDADES — as fórmulas
//! nascem do catálogo, e nenhuma palavra do roteiro nomeia um id que não existe.

const EXPR: &str = include_str!("../src/expr_smoke.rs");

/// **As fórmulas do smoke saem do CATÁLOGO, não da mão de quem escreveu a cena.**
///
/// **Mutação que deve sangrar:** trocar `drive_recipe(doc, shaker, .., "shake")` por um
/// `doc.set_clip_expr(active, target, Some("time*1.2".into()))` — a forma da cena antiga.
#[test]
fn the_expression_smoke_authors_through_the_recipe_catalog() {
    assert!(
        EXPR.contains("RecipeStack::of("),
        "o smoke tem de construir as fórmulas pelo catálogo (`RecipeStack`): é o catálogo \
         que o artista clica, e uma fórmula escrita à mão exercita o avaliador e não a \
         galeria (auditoria §4 D-F)"
    );
    // As três receitas nomeadas, cada uma por um motivo declarado no doc da cena.
    for id in ["\"shake\"", "\"sway\"", "\"jitter\""] {
        assert!(
            EXPR.contains(id),
            "o smoke tem de dirigir a receita {id} — as três cobrem gerador cru, \
             modificador sobre keys, e a receita cujo desenho depende do `__seed`"
        );
    }
}

/// **O smoke ABRE o card**, em vez de mandar o artista procurá-lo.
///
/// Uma cena que autora por código e deixa o card fechado pula justamente a costura que ela
/// existe para provar — foi assim que o roteiro passou três waves apontando para um campo
/// que já não estava lá.
///
/// **Mutação que deve sangrar:** remover a chamada a `request_expr_card`.
#[test]
fn the_expression_smoke_lands_the_artist_inside_the_card() {
    assert!(
        EXPR.contains("request_expr_card("),
        "a cena tem de abrir o card (pela porta do painel), senão o roteiro é a única \
         coisa apontando para a UI — e prosa envelhece sem quebrar nada"
    );
}

/// **Nenhuma palavra do roteiro nomeia o campo de texto inline DELETADO.**
///
/// ⚠️ A busca é pelo VOCABULÁRIO do defeito, não pelo id: o campo morreu, então nenhum
/// identificador dele sobrou para procurar — o que sobrou foi a frase que ensinava a usá-lo.
///
/// **Mutação que deve sangrar:** escrever de volta *"abre um campo de texto"* ou
/// *"ESVAZIE o campo"* no doc do módulo.
#[test]
fn the_script_never_tells_the_artist_to_use_the_deleted_inline_field() {
    // Cada par é (frase proibida, por que ela é proibida). As frases vêm VERBATIM do
    // roteiro antigo, então o gate falha exatamente se ele voltar.
    for (phrase, why) in [
        (
            "abre um campo de texto",
            "o campo inline morreu na W1; quem abre é o CARD",
        ),
        (
            "ESVAZIE o campo",
            "este é o gesto do D-I e ele apontava para um widget inexistente",
        ),
    ] {
        assert!(
            !EXPR.contains(phrase),
            "o roteiro do smoke voltou a descrever o campo de texto inline DELETADO \
             (\"{phrase}\") — {why}. Um roteiro que nomeia UI morta faz o artista concluir \
             que a feature está quebrada."
        );
    }
    assert!(
        !EXPR.contains("EXPR_FIELD") && !EXPR.contains("expr_field"),
        "nem o id do campo morto pode reaparecer aqui"
    );
}

/// **O roteiro cobre as quatro correções da FASE 0**, cada uma com o gesto que a expõe.
///
/// ⚠️ Um smoke é a única coisa que julga a tela, e as quatro correções desta fase são
/// invisíveis quando funcionam (um clique que NÃO vaza, uma roda que NÃO zooma, um preview
/// que PARA, uma pose que VOLTA). Sem um passo por correção, o Enio não tem como reprovar o
/// que ainda estiver errado — e um smoke que não pode reprovar não é um smoke.
///
/// **Mutação que deve sangrar:** apagar qualquer um dos quatro passos do doc do módulo.
#[test]
fn the_script_asks_for_the_four_things_this_phase_fixed() {
    for (needle, what) in [
        (
            "Dur(s)",
            "D10 — o card engole o clique (a caixa que era editada por engano)",
        ),
        (
            "roda",
            "U3 — a roda sobre o card não pode dar zoom na timeline",
        ),
        ("seed", "D-J — a fita tem de desenhar o tremor DESTE objeto"),
        ("esconder", "D-K — esconder o painel para o preview"),
        (
            "Apply",
            "D-I — apagar a linha e aplicar devolve a propriedade",
        ),
    ] {
        assert!(
            EXPR.contains(needle),
            "o roteiro do smoke não pede o gesto de {what} (procurando por {needle:?})"
        );
    }
}
