//! **Arch-gate: o rail e a barra de topo LEEM o relógio da UI.**
//!
//! ## Porque este gate existe, e o número que o justifica
//!
//! O motor de movimento (`crate::motion`) foi construído, medido e gateado — e durante uma jornada
//! inteira **dois botões do app inteiro o liam**: o *Cancel* e o *Done* do card de Fill. O relógio
//! já tinha entrada para todos os chips do rail e todas as pills da barra (o `hover_targets` cobre
//! qualquer `Button`/`Radio`/`Toggle`/`Checkbox` registado), então o dado estava lá; **cegos eram
//! os sítios de PINTURA**, que nunca perguntavam.
//!
//! O sintoma foi um report do Enio de uma frase — *«não percebi nenhuma diferença com expressive»* —
//! e ele estava certo: o roteiro do smoke mandava passar o rato pela barra e pelo rail, e nenhum
//! dos dois estava ligado. **A feature existia e não alcançava nada que o artista toca.**
//!
//! ⚠️ **Um gate de unidade não vê isto.** As duas metades — *o widget sabe misturar* e *o pintor
//! passa-lhe o escalar* — são independentes: dá para ter os `paint_*_t` perfeitamente testados e
//! um `paint.rs` que nunca os chama, com a suíte toda verde. É a lição que a `line/anim` pagou no
//! motion path (*«um gate de unidade é CEGO à fiação do shell»*), aqui no chrome.

const HERO_PAINT: &str = include_str!("../src/screens/hero/paint.rs");
const LEFT_RAIL: &str = include_str!("../src/screens/hero/left_rail.rs");
const CLUSTER: &str = include_str!("../src/screens/hero/topbar/cluster_painter.rs");

/// ⚠️ **CONTROLE POSITIVO.** Um ficheiro renomeado deixaria os gates abaixo a varrer o vazio e a
/// passar por vácuo — a falha silenciosa que o `keyboard.rs` partido já produziu neste repo.
#[test]
fn the_scanned_files_are_the_real_ones() {
    assert!(
        HERO_PAINT.contains("paint_left_rail(") && HERO_PAINT.contains("paint_top_bar("),
        "o `screens/hero/paint.rs` deixou de chamar o rail ou a barra: os gates abaixo deixaram \
         de afirmar o que dizem"
    );
    assert!(
        LEFT_RAIL.contains("fn paint_rail("),
        "o `left_rail.rs` mudou de dono"
    );
    assert!(
        CLUSTER.contains("fn paint_topbar_rail_chip("),
        "o `cluster_painter.rs` mudou de dono"
    );
}

/// O pintor do hero entrega o relógio às DUAS superfícies.
///
/// **Mutação que deve sangrar:** trocar `&hero.motion` por `&Default::default()` em qualquer uma
/// das duas chamadas — o chrome volta a pintar a função escada e nenhum outro teste repara.
#[test]
fn the_hero_hands_the_clock_to_the_rail_and_to_the_top_bar() {
    for call in ["paint_left_rail(", "paint_top_bar("] {
        let at = HERO_PAINT
            .find(call)
            .unwrap_or_else(|| panic!("`{call}` não é chamado — ver o controle positivo"));
        let close = HERO_PAINT[at..]
            .find("\n    );")
            .unwrap_or_else(|| panic!("`{call}` não fecha como esperado"));
        let args = &HERO_PAINT[at..at + close];
        assert!(
            args.contains("&hero.motion"),
            "`{call}` não recebe `&hero.motion`. O relógio existe, o `hover_targets` já lhe dá \
             entrada para cada chip, e o pintor não pergunta — que é exactamente o estado em que \
             a UI viva ficou invisível ao artista durante uma jornada inteira."
        );
    }
}

/// O rail resolve o escalar **por id**, e não um número só para a coluna inteira.
///
/// ⚠️ Um `hover_t` constante compilaria e animaria **todos** os chips juntos ao passar o rato por
/// um — o defeito que só se vê na tela, e que um gate de presença não distingue de o certo.
#[test]
fn the_rail_asks_the_clock_for_each_chip_by_id() {
    assert!(
        LEFT_RAIL.contains("paint_tool_rail_t(") && LEFT_RAIL.contains("motion.get(id)"),
        "o rail não resolve o hover por id: ou não usa a porta com eixo, ou passa um escalar \
         único, e aí a coluna inteira acende junta"
    );
}

/// A pill da barra resolve pelo id **do chip que está a pintar**.
#[test]
fn the_top_bar_pill_asks_the_clock_for_its_own_chip() {
    assert!(
        CLUSTER.contains("(state, hover_t)") && CLUSTER.contains("motion.get(chip_id)"),
        "a pill da barra não lê o relógio pelo próprio `chip_id`"
    );
}

/// **As duas superfícies perguntam se podem MEXER-SE, e não só quanto do hover há.**
///
/// ⚠️ Este gate nasceu de um report do Enio no smoke — *«o reduce motion faz o quê? Pois não fez
/// nada»* — sobre um defeito que era meu: o crescimento do chip foi pendurado numa track `Fade`,
/// e `Fade` **sobrevive ao reduced motion por desenho** (o gatilho vestibular é a área a
/// deslocar-se, não a tinta a mudar). O resultado é que a definição que existe para parar o
/// movimento não parava o único movimento que o chrome tinha.
///
/// **Mutação que deve sangrar:** trocar `motion.travels()` por `true` em qualquer das duas — o
/// reduced motion volta a ser inerte no chrome, e nenhum gate de unidade repara, porque a
/// `hover_lift` continua perfeitamente testada com o argumento que ninguém lhe passa certo.
#[test]
fn both_surfaces_ask_whether_they_may_move() {
    for (name, src) in [("left_rail.rs", LEFT_RAIL), ("cluster_painter.rs", CLUSTER)] {
        assert!(
            src.contains("motion.travels()"),
            "o `{name}` cresce o chip sem perguntar `motion.travels()`: o reduced motion fica \
             inerte exactamente no canal que ele existe para parar"
        );
    }
}
