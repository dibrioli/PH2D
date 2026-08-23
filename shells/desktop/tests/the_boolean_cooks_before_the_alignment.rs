//! **Arch-gate: a ORDEM dos produtores de geometria derivada.**
//!
//! ⚠️ Ele existe porque nenhum teste de unidade alcança esta costura: a sequência mora dentro do
//! `render_frame`, que exige `gfx` (janela + GPU). Os gates de unidade provam o que cada produtor
//! computa; **nenhum deles prova em que ordem o produto os chama** — e trocar dois destes termos
//! dá arte diferente com a suíte inteira verde.
//!
//! # As três afirmações, e o que cada uma custa se cair
//!
//! 1. **A booleana roda DEPOIS dos cinco que estendem.** Ela consome *o que os filhos de facto
//!    desenham*: um operando com offset vivo tem de entrar deslocado. Antes deles, ela leria a
//!    fonte crua e a arte combinada ignoraria o offset — em silêncio.
//! 2. **A booleana roda ANTES do alinhamento.** O alinhamento é um campo do `StrokeSpec` do
//!    RESULTADO; alinhar os operandos e só então combiná-los responde outra pergunta.
//! 3. **O Apply roda DEPOIS do `recook`.** Ele materializa o plano que o produtor acabou de
//!    computar — *o que está na tela*. Chamá-lo antes seria consolidar a resposta do frame
//!    anterior, e a forma saltaria no clique.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// **A chamada do COZIMENTO da booleana** — a âncora dos dois gates de ordem abaixo.
///
/// ⚠️ Ela foi `"self.bool_live"` até 2026-08-22, e deixou de servir no dia em que um segundo
/// consumidor passou a LER o `bool_live` mais cedo no frame (o selo do papel booleano na
/// hierarquia, que consome o plano do quadro anterior de propósito). O `find` devolvia essa
/// leitura: um dos gates ficou **vermelho sobre uma ordem correcta**, e o outro ficou **VERDE POR
/// ACIDENTE** — ele só exigia `boolean < plan`, e a leitura precoce satisfazia isso sozinha.
///
/// ⛔ A lição não é *"o gate estava errado"*. Um arch-gate ancorado no NOME DO CAMPO afirma
/// *"ninguém mais toca neste campo"* — bem mais forte que a ordem que ele quer provar, e uma
/// afirmação que envelhece sozinha. A âncora certa é a chamada que ele mede.
const COOK: &str = ".recook(vec_scene, sim, &self.vec_entities, &vec_xf, &mut vec_live)";

/// Onde a chamada `needle` aparece — falha nomeando quem sumiu.
fn at(needle: &str) -> usize {
    SRC.find(needle)
        .unwrap_or_else(|| panic!("a chamada `{needle}` sumiu do render_loop"))
}

/// **A booleana viva cozinha depois dos cinco que estendem, e antes do alinhamento.**
///
/// ⚠️ As âncoras são o NOME das chamadas, nunca a indentação nem a distância em bytes: um
/// arch-gate ancorado numa métrica de formatação expira no primeiro `rustfmt` e pede para ser
/// silenciado em vez de acreditado (a lição que os dois arch-gates desta linha já pagaram em
/// 23/07).
#[test]
fn the_boolean_cooks_after_the_five_and_before_the_alignment() {
    // O ÚLTIMO `extend` é o que fecha o mapa fundido — qual dos cinco é ele não importa, e
    // depender disso amarraria o gate à ordem interna deles.
    let profile = SRC
        .rfind("vec_live.extend(")
        .expect("os `extend` que fundem o mapa sumiram do render_loop");
    let boolean = at(COOK);
    let align = at("self.align_live.recook(");
    let silhouette = at("self.fx_silhouette");

    assert!(
        profile < boolean,
        "a booleana cozinha ANTES do mapa estar fundido ({profile} vs {boolean}) — ela leria a \
         fonte crua de um operando que tem offset/pattern/contour vivo"
    );
    assert!(
        boolean < align,
        "o alinhamento corre ANTES da booleana ({align} vs {boolean}) — ele alinharia o traço dos \
         OPERANDOS em vez do traço do RESULTADO"
    );
    assert!(
        align < silhouette,
        "a silhueta corre antes do alinhamento ({silhouette} vs {align}) — ela é a união do que se \
         DESENHA, e o que se desenha já inclui as duas transformações"
    );
}

/// **O Apply da booleana corre depois do `recook`, e materializa o `plan`.**
///
/// ⚠️ A segunda metade é a que pina a porta única: `bake` recebe o plano; se algum dia ele voltar
/// a chamar o motor, o gate morde. Uma segunda resposta a *"o que este grupo desenha?"* faria a
/// arte saltar entre o último frame desenhado e o clique.
#[test]
fn the_apply_materialises_the_plan_the_producer_just_cooked() {
    let boolean = at(COOK);
    let plan = at("self.bool_live.plan(g)");
    let bake = at("crate::bool_gesture::bake(");
    assert!(
        boolean < plan && plan < bake,
        "o Apply consulta o plano fora de ordem (recook {boolean}, plan {plan}, bake {bake})"
    );
    assert!(
        !SRC.contains("bool_gesture::bake(sim, vec_scene, &mut self.vec_pen, &recompute"),
        "o bake voltou a re-computar em vez de materializar o plano"
    );
}

/// **A shell publica ao painel se há booleana viva selecionada.**
///
/// Sem esta linha o `Apply` nunca aparece (ou aparece sempre), e o painel — que não alcança o
/// mundo ECS — não tem como saber. É a mesma fronteira de toda a publicação deste painel.
#[test]
fn the_shell_publishes_whether_a_live_boolean_is_selected() {
    assert!(
        SRC.contains("ph2d_panel_vector::state::set_bool_group_selected("),
        "a shell parou de publicar o fato que decide se o Apply é oferecido"
    );
}

/// **A shell publica a FILEIRA do verbo por forma, e passa-lhe o PRIMÁRIO.**
///
/// ⚠️ O último elo da corrente, e o único que nenhum teste de unidade alcança: se esta chamada
/// sumisse, a fileira nunca seria oferecida no app real — e a suíte inteira continuaria verde,
/// porque os gates do painel publicam o estado eles próprios.
///
/// A segunda metade é a que prende a cura de 2026-08-22: o sujeito é `vec_pen.selected()` (o
/// primário), e **não** a contagem da seleção. Tocar um filho seleciona o GRUPO inteiro, então uma
/// regra de contagem torna a fileira inalcançável por clique.
#[test]
fn the_shell_publishes_the_per_shape_verb_row_with_the_primary() {
    assert!(
        SRC.contains("ph2d_panel_vector::state::set_bool_shape_row("),
        "a shell parou de publicar a fileira do verbo por forma — ela nunca apareceria no app"
    );
    assert!(
        SRC.contains("let primary = self.vec_pen.selected();"),
        "o sujeito da fileira deixou de ser o PRIMÁRIO — com a contagem, nenhum clique a alcança"
    );
}
