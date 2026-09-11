//! **SONDA (`--ignored`): ONDE os 335 µs de um quadro em repouso estão.**
//!
//! ⚠️ **Binário PRÓPRIO, e a separação é load-bearing — não é arrumação.** O
//! `register_all_panels()` é um registo **GLOBAL e irreversível**, e o censo irmão
//! (`ui_motion_population_census`) mede a linha `chrome` **antes** de ele correr: é a única janela
//! em que o piso da população (162 widgets) é observável. Com as duas sondas no MESMO binário, a
//! que corre primeiro regista tudo e o `chrome` da outra passa a medir **4491** — a linha de
//! CONTROLO morre e as duas passam a imprimir o mesmo número achando que comparam alguma coisa.
//! Foi exactamente o que aconteceu quando esta sonda nasceu lá dentro (medido: `chrome` saltou de
//! **3,09 µs** para **339,7**), e é a mesma armadilha de ordem que o doc do censo já nomeava
//! *dentro* de uma sonda, agora entre duas.
//!
//! Rode: `cargo test -p ph2d-panel-registry-init --release --test ui_motion_frame_halves -- --ignored --nocapture`

use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::HeroScreen;

const DT: f64 = 1.0 / 60.0;

/// **ONDE os 335 µs estão** — a decomposição que o censo acima não dá, e que refutou a minha
/// primeira hipótese.
///
/// A sonda irmã em `ph2d-editor-core` (`measure_what_a_resting_frame_is_made_of`) mede as três
/// peças do RELÓGIO sobre a mesma população e soma **53 µs**: o `advance` 7,6 · o `animate × POP`
/// 44,8 · o `Vec` 1,0. Contra os 335 do censo, **falta 84% do custo** — *números por peça que não
/// reconciliam com o total são a assinatura de uma atribuição incompleta*, e parar ali teria posto
/// a wave inteira no módulo errado.
///
/// O que sobra são as **duas travessias do STORE** que o `tick_hover` faz e que o relógio não vê:
///
/// | peça | o que é |
/// |---|---|
/// | `hover_targets()` | percorre `states` (`BTreeMap`) e deriva o alvo de cada widget |
/// | `set_hover_live × POP` | **uma inserção em `BTreeMap` por widget, por quadro** |
///
/// ⚠️ **A segunda é a suspeita, e a estrutura diz por quê:** `hover_live` é um `BTreeMap`, então
/// re-escrever o MESMO valor ainda paga a descida inteira — com 4491 chaves são ~12 níveis de
/// ponteiro por widget, ~54 mil derreferências por quadro, com o app parado.
///
/// ⚠️ **Sonda, não gate.** Ela imprime; o que decide a cura é qual das duas domina — *a travessia*
/// (cura: só processar o que mudou) ou *a escrita* (cura: não escrever o que não mudou) são waves
/// de tamanhos muito diferentes.
#[test]
#[ignore = "sonda: rode com --release -- --ignored --nocapture"]
fn measure_which_half_of_the_resting_frame_is_the_store() {
    // ⚠️ **O registo vem ANTES do `new`** — o `HeroScreen::new` já popula o store a partir do
    //    registo, então construir primeiro daria a fixture do *chrome* (162 widgets) com o rótulo
    //    do app cheio. É a mesma armadilha de ordem que o doc do censo acima nomeia.
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.tick_motion(DT);

    let ids: Vec<NodeId> = hero.store.hover_targets().map(|(id, _)| id).collect();
    println!(
        "[ui-half] decomposicao do quadro em REPOUSO, {} widgets macios",
        ids.len()
    );

    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };

    // (a) a TRAVESSIA: percorrer `states` e derivar o alvo de cada widget.
    let mut walk = Vec::with_capacity(200);
    for _ in 0..200 {
        let t0 = std::time::Instant::now();
        let n = hero.store.hover_targets().count();
        walk.push(t0.elapsed().as_secs_f64() * 1e6);
        std::hint::black_box(n);
    }

    // (b) a ESCRITA: uma inserção por widget, com o valor que ele JÁ TEM (o caso de repouso).
    let mut write = Vec::with_capacity(200);
    for _ in 0..200 {
        let t0 = std::time::Instant::now();
        for id in &ids {
            hero.store.set_hover_live(*id, 0.0);
        }
        write.push(t0.elapsed().as_secs_f64() * 1e6);
    }

    let (a, b) = (med(walk), med(write));
    println!(
        "[ui-half] {:>32} {:>11.3}us",
        "hover_targets (a travessia)", a
    );
    println!(
        "[ui-half] {:>32} {:>11.3}us",
        "set_hover_live x POP (a escrita)", b
    );
    println!("[ui-half] {:>32} {:>11.3}us", "as duas do STORE", a + b);
    println!(
        "[ui-half] ⚠️ o relogio custa 53us (sonda irma); 53 + {:.0} = {:.0} contra os 335 do censo",
        a + b,
        53.0 + a + b
    );

    // ⚠️ **O que sobra NÃO é uma sexta peça a encontrar — é o CACHE, e o censo prova-o sem
    //    instrumento nenhum.** Ele mede `chrome` (162 widgets) em 3,09 µs e `repouso` (4491) em
    //    335: **27,7× de população para 108× de custo**. Uma soma de peças `O(n)` seria LINEAR;
    //    super-linear a este ponto é a assinatura de um conjunto de trabalho que deixou de caber.
    //
    //    E é por isso que as sondas isoladas (esta inclusive) sub-estimam: cada uma percorre a
    //    MESMA estrutura 200 vezes, totalmente residente. O quadro real intercala cinco tiques
    //    sobre mapas diferentes, e eles despejam-se uns aos outros — *uma sonda que constrói os
    //    próprios dados logo antes de os medir mede o cache, não o produto* (a lição que o Painter
    //    pagou na §5.16).
    //
    // ⇒ **A cura NÃO é micro-otimizar uma das peças.** Nenhuma delas passa de 45 µs, e a soma das
    //    cinco não chega a um terço do total. O que custa é TOCAR em 4491 entradas; a única cura
    //    com a ordem de grandeza certa é **não as tocar** — processar o que MUDOU, que é wave
    //    própria e mexe no `tick_hover`, cujo publish atravessa nove consumidores e tem histórico
    //    de defeito subtil de flash.
    println!(
        "[ui-half] ⚠️ o que sobra e' CACHE: o censo mede 162 widgets em 3,09us e 4491 em 335us -- \
         27,7x de populacao para 108x de custo, super-linear"
    );
}
