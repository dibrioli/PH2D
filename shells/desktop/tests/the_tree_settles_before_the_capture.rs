//! ⭐⭐⭐ **Arch-gate da ORDEM DA FOTOGRAFIA** — a árvore reconcilia colada à captura que ela serve.
//!
//! # A lei
//!
//! A captura do undo tem de ser **ponto fixo dos sistemas**. Os verbos tardios da Hierarquia
//! (apagar · duplicar · *Remove from Sheet*) vivem no `render_loop::hierarchy::dispatch`, que corre
//! ~2 300 linhas **depois** da projecção de z — logo a fotografia guardaria um mundo e um documento
//! que discordam, e o quadro seguinte reconciliá-los-ia **sem entrada nenhuma**: o passo fantasma
//! do report de 2026-09-07, por outro verbo. Medido e confirmado para os dois em 2026-09-08
//! (`vec_zorder_late_writers_tests`), e curado por [`crate::vec_tree_settle`].
//!
//! # ⚠️ E ela corre EXACTAMENTE onde a fotografia é tirada, não em todo quadro
//!
//! A rede custa uma varredura `O(formas)` — medida, `0,3 ms` a 1 000, `1,6` a 5 000 e **`7,0 ms` a
//! 20 000, que são `42 %` de um quadro**, linear em toda a faixa. O `post_frame_undo` **já** suprime
//! a captura por cinco motivos nomeados (botão em baixo · arrasto do gizmo 3D · colorize a
//! recalcular · transição de estado de UI · sem entrada), e o comentário dele chama-lhe *«o passo
//! caro do quadro»*. ⇒ *a rede existe para a fotografia; ela corre exactamente quando a fotografia
//! corre.*
//!
//! ⛔ **Não é uma bandeira que um verbo novo tem de lembrar de levantar** — é a MESMA condição, a
//! mesma linha, sem lista a manter.
//!
//! ⚠️ **Porque um gate de TEXTO:** os gates de comportamento rodam um **espelho** da sequência do
//! quadro, escrito à mão. *Um espelho não vê o dia em que alguém reordena o quadro de verdade* — e
//! foi assim que esta classe de defeito entrou duas vezes.
//!
//! ⚠️ **O que este gate NÃO mede, e o limite é nomeado:** que a rede contenha *todos* os escritores
//! derivados do passe do desenho. Essa metade é medida pelos gates de ponto fixo com mutação
//! tardia — e foi um deles que descobriu que faltava o assentamento do pivô, **com os dois
//! controlos já verdes**.

const UNDO: &str = include_str!("../src/undo_app.rs");
const MAIN: &str = include_str!("../src/main.rs");

/// A posição (em bytes) da 1ª ocorrência de `needle` em `src`, ou pânico com a razão.
fn at(src: &str, needle: &str, onde: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "o passe `{needle}` sumiu de {onde} — se foi renomeado, actualize este gate (e confira \
             que a reconciliação continua colada à captura)"
        )
    })
}

/// **Mutação que deve sangrar:** mover a chamada para depois do `capture_project` que vira passo,
/// ou de volta para o `render_frame`.
#[test]
fn the_tree_settles_before_the_capture() {
    let supressao = at(
        UNDO,
        "let motivo = if self.held_button.is_some()",
        "undo_app.rs",
    );
    let settle = at(
        &UNDO[supressao..],
        "self.settle_tree_before_capture();",
        "undo_app.rs, depois do bloco da supressão",
    ) + supressao;
    let captura = at(
        &UNDO[settle..],
        "let Some(current) = self.capture_project()",
        "undo_app.rs, depois da rede",
    ) + settle;

    assert!(
        supressao < settle,
        "a rede tem de correr DEPOIS do bloco que decide a supressão: nos quadros suprimidos não \
         há fotografia para proteger, e ela custa uma varredura O(formas) — 42 % de um quadro a \
         20 000 formas"
    );
    assert!(
        settle < captura,
        "a rede tem de correr ANTES do `capture_project` que vira passo: senão a fotografia guarda \
         um mundo e um documento que discordam, e o quadro seguinte fecha-os SOZINHO — que é a \
         definição do passo fantasma (report de 2026-09-07)"
    );
}

/// ⛔ **O baseline do primeiro quadro também é uma fotografia.**
///
/// Ele vira o `undo_baseline` contra o qual todo passo seguinte é medido. Nascido por reconciliar,
/// o primeiro diff do artista carregaria a convergência dos sistemas dentro dele.
#[test]
fn the_first_baseline_is_settled_too() {
    let baseline = at(UNDO, "if self.undo_baseline.is_none() {", "undo_app.rs");
    let bloco = &UNDO[baseline..baseline + 240];
    assert!(
        bloco.contains("self.settle_tree_before_capture();"),
        "o baseline do primeiro quadro é tirado SEM a reconciliação — ele é a referência de todo \
         passo seguinte, e nascer por reconciliar põe a convergência dos sistemas dentro do \
         primeiro diff do artista"
    );
}

/// ⛔ **E ela NÃO pode voltar a correr em todo quadro.**
///
/// No `render_frame` ela corria sempre, inclusive nos ~60 quadros por segundo em que o app está
/// parado e nenhuma fotografia é tirada.
#[test]
fn the_net_does_not_run_on_every_frame() {
    assert!(
        !MAIN.contains("self.settle_tree_before_capture();"),
        "a rede voltou para o `render_frame` — ali ela corre em TODO quadro, incluindo os que a \
         captura suprime, e custa uma varredura O(formas) por nada"
    );
}

/// ⛔ **A rede não pode virar a única projecção.**
///
/// O passe do desenho (`render_loop/mod.rs`) continua a projectar a ordem de z **antes** dos ~40
/// consumidores da cena — `envelope_live`, `skeleton_live`, `pattern_live`, `align_live`, o
/// hit-test e o próprio desenho. A ordem das `paths` **é** a ordem de pintura: mover a leitura para
/// o fim do quadro daria a todos eles um quadro de atraso. *Esta é a razão medida pela qual a cura
/// foi uma SEGUNDA passagem e não mudar a posição da primeira.*
#[test]
fn the_drawing_pass_still_projects_the_z_order_before_the_consumers() {
    const RENDER_LOOP: &str = include_str!("../src/render_loop/mod.rs");
    let reorder = at(RENDER_LOOP, "vec_scene.reorder_to(", "render_loop/mod.rs");
    let consumidor = at(
        RENDER_LOOP,
        "crate::envelope_live::recook(",
        "render_loop/mod.rs",
    );
    assert!(
        reorder < consumidor,
        "a projecção do passe do desenho deixou de vir antes dos consumidores da cena: eles passam \
         a desenhar a pilha do quadro ANTERIOR, que é o preço que a segunda passagem existe para \
         não pagar"
    );
}
