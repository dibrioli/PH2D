//! ⭐⭐⭐ **Arch-gate da ORDEM DO FECHO DO QUADRO** — a árvore reconcilia ANTES da fotografia.
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
//! ⚠️ **Porque um gate de TEXTO, e não só os gates de comportamento:** aqueles rodam um **espelho**
//! da sequência do quadro, escrito à mão (a convenção desta shell). *Um espelho não vê o dia em que
//! alguém reordena o quadro de verdade* — e foi exactamente assim que esta classe de defeito entrou
//! duas vezes. Este lê o ficheiro do produto.
//!
//! ⚠️ **O que este gate NÃO mede, e o limite é nomeado:** que a rede contenha *todos* os escritores
//! derivados do passe do desenho. Essa metade não é derivável de texto — ela é medida pelos gates
//! de ponto fixo com mutação tardia, e foi um deles (o do *duplicar*) que descobriu que faltava o
//! assentamento do pivô, com os dois controlos já verdes.

const MAIN: &str = include_str!("../src/main.rs");

/// A posição (em bytes) da 1ª ocorrência de `needle`, ou pânico com a razão.
fn at(needle: &str) -> usize {
    MAIN.find(needle).unwrap_or_else(|| {
        panic!(
            "o passe `{needle}` sumiu do `render_frame` — se foi renomeado, actualize este gate \
             (e confira que a reconciliação continua ANTES da captura)"
        )
    })
}

/// **Mutação que deve sangrar:** trocar a ordem das duas linhas no `App::render_frame`.
#[test]
fn the_tree_settles_before_the_capture() {
    let quadro = at("self.run_render_frame();");
    let settle = at("self.settle_tree_before_capture();");
    let captura = at("self.post_frame_undo();");

    assert!(
        quadro < settle,
        "a reconciliação tem de correr DEPOIS do quadro: é lá dentro que o `hierarchy::dispatch` \
         apaga e duplica, e reconciliar antes deles não veria a escrita deles"
    );
    assert!(
        settle < captura,
        "a reconciliação tem de correr ANTES do `post_frame_undo`: senão a fotografia guarda um \
         mundo e um documento que discordam, e o quadro seguinte fecha-os SOZINHO — que é a \
         definição do passo fantasma (report de 2026-09-07)"
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
    let reorder = RENDER_LOOP
        .find("vec_scene.reorder_to(")
        .expect("a projecção do passe do desenho sumiu do render_loop");
    let consumidor = RENDER_LOOP
        .find("crate::envelope_live::recook(")
        .expect("o `envelope_live::recook` sumiu — escolha outro consumidor da cena e actualize");
    assert!(
        reorder < consumidor,
        "a projecção do passe do desenho deixou de vir antes dos consumidores da cena: eles passam \
         a desenhar a pilha do quadro ANTERIOR, que é o preço que a segunda passagem existe para \
         não pagar"
    );
}
