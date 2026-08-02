//! **A SEQUÊNCIA leva a algum lugar** (plano UI/UX §4/W4) — a 4ª condição de UI da política deste
//! repo, e a que nenhuma das outras três implica.
//!
//! Todo edit pode ter gate, todo widget pode estar registado e clicável, e o gesto ainda não
//! levar a lugar nenhum. Aqui a corrente inteira é dirigida: **o clique escolhe → a shell escreve
//! no ECS → o resolvedor produz a tinta → o desenho a usa** — e trocar de modo re-veste.
//!
//! ⚠️ **Mais o arch-gate da FIAÇÃO.** O `render_loop` exige janela e GPU, então nenhum teste de
//! unidade o alcança: sem a asserção sobre o fonte, o resolvedor ficaria correto, gateado e
//! **nunca chamado** — a metade silenciosa de toda feature de chrome.

const RENDER_LOOP: &str = include_str!("../src/render_loop/mod.rs");

/// **A fiação existe: o passe de desenho publica a tinta resolvida.**
///
/// Mutação que este gate mata: apagar a linha do `vec_view.bound = …`. Sem ela o `VecViewState`
/// sai sempre vazio, `painted()` devolve `Borrowed` em toda forma, e **todo binding fica inerte
/// com os oito gates de unidade verdes**.
#[test]
fn the_draw_pass_publishes_the_resolved_paint() {
    assert!(
        RENDER_LOOP.contains("vec_view.bound = crate::vec_bindings::resolve("),
        "o passe de desenho deixou de publicar a tinta dos tokens — todo binding fica inerte, e \
         os gates de unidade do resolvedor continuam verdes"
    );
    assert!(
        RENDER_LOOP.contains("ph2d_panel_vector::state::set_token_bindings("),
        "a shell deixou de publicar os bindings da seleção — os chips do painel mostrariam '—' \
         para sempre, mesmo sobre uma forma bindada"
    );
}

/// ⚠️ **O resolvedor NÃO entra no `view_state`.**
///
/// Aquela porta é chamada por todo caminho de hit-test e gesto (o pick, o marquee, a linha de
/// corte), e nenhum deles pergunta de que cor a forma é. Resolver token ali seria trabalho de
/// desenho pago por quem só quer geometria — e o custo é silencioso, porque nada quebra.
#[test]
fn the_hit_test_path_does_not_pay_for_token_resolution() {
    const VIEW_STATE: &str = include_str!("../src/vec_entities.rs");
    assert!(
        !VIEW_STATE.contains("vec_bindings::resolve"),
        "o `view_state` passou a resolver tokens — todo hit-test e todo gesto pagam agora por uma \
         pergunta de DESENHO que não fazem"
    );
}
