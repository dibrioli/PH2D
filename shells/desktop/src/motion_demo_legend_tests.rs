//! Gates da legenda no canvas — o salto global, e a lei de que uma ficha é uma FICHA.

use super::*;

/// **O QUE SE PUBLICA É O QUE SE LÊ, e publicar de novo SUBSTITUI.**
///
/// ⚠️ A segunda metade é a que interessa: se ele acumulasse, abrir uma cena depois de outra
/// deixaria as fichas da anterior a pairar sobre figuras que já não existem — e o defeito leria
/// como *"a cena nova desenhou coisas a mais"*.
#[test]
fn publishing_replaces_and_never_accumulates() {
    publish(vec![Caption::new([1.0, 2.0], "primeira")]);
    assert_eq!(captions().len(), 1);
    publish(vec![
        Caption::new([0.0, 0.0], "a"),
        Caption::new([1.0, 1.0], "b"),
    ]);
    let now = captions();
    assert_eq!(now.len(), 2, "a segunda publicação substitui a primeira");
    assert_eq!(now[0].text, "a");
    publish(Vec::new());
    assert!(captions().is_empty(), "e uma cena sem legenda limpa o slot");
}

/// **AS CENAS QUE PUBLICAM LEGENDA** — a lista, e uma linha por cena nova.
///
/// ⚠️ **Ela é uma ENUMERAÇÃO, e uma enumeração apodrece.** O que a mantém honesta é o controle
/// [`the_legend_list_is_not_empty_and_names_real_scenes`]: uma entrada que deixe de existir não
/// compila, e uma cena nova sem entrada não é apanhada aqui — é apanhada pelo Enio, e é por isso
/// que o cabeçalho do [`super`] manda pôr a `captions()` ao lado do `publish`.
fn scenes() -> Vec<(&'static str, Vec<Caption>)> {
    vec![
        (
            "=82 o painel que encolhe",
            crate::motion_state::conferencia_demos_gates::captions(),
        ),
        (
            "=83 o campo que era um número",
            crate::motion_state::conferencia_demos_campo::captions(),
        ),
        (
            "=84 o que o efeito não sabia fazer",
            crate::motion_state::conferencia_demos_fx_modes::captions(),
        ),
        (
            "=85 a forma que o artista desenha",
            crate::motion_state::conferencia_demos_drawn::captions(),
        ),
        (
            "=86 a fronteira curva",
            crate::motion_state::conferencia_demos_bezier::captions(),
        ),
    ]
}

/// O controle da lista acima: ela existe e cada cena diz alguma coisa.
#[test]
fn the_legend_list_is_not_empty_and_names_real_scenes() {
    let all = scenes();
    assert!(all.len() >= 3, "tres cenas ja' publicam legenda");
    for (name, caps) in &all {
        assert!(!caps.is_empty(), "{name} publica pelo menos uma ficha");
    }
}

/// **UMA FICHA É CURTA** — e o número é do formato, não do gosto.
///
/// A ficha da casa cresce com o texto e **tapa** o que está por baixo (é a lei escrita no
/// `paint_chip`). Numa cena de conferência as figuras vivem a `COL_X = 2,7` do centro, ou seja
/// há ~5,4 unidades de mundo entre os centros das duas metades: uma legenda que se esticasse
/// tanto quanto a frase do terminal cobriria a figura vizinha, e o rótulo passaria a esconder
/// justamente o que ele manda comparar.
///
/// ⚠️ **O gate mede a CENA, não a lista aqui.** Ele lê a `captions()` real da `=83`, então uma
/// linha nova com uma frase longa reprova onde nasce.
#[test]
fn every_caption_is_chip_sized() {
    /// Quantos caracteres cabem sem a ficha invadir a metade vizinha, medido no corpo da ficha
    /// da casa (11 px) contra a distância entre as duas colunas.
    const MAX_CHARS: usize = 40;
    for (name, all) in scenes() {
        for c in &all {
            assert!(
                c.text.chars().count() <= MAX_CHARS,
                "{name}: legenda longa demais para uma ficha ({} chars): {:?}",
                c.text.chars().count(),
                c.text
            );
            assert!(
                !c.text.trim().is_empty(),
                "{name}: uma ficha vazia é um rectângulo"
            );
        }
        // E o controle: a barra não é vacuamente alta.
        assert!(
            all.iter().any(|c| c.text.chars().count() > MAX_CHARS / 2),
            "{name}: as legendas dizem alguma coisa (a barra não é folgada de graça)"
        );
    }
}

/// **CADA METADE TEM A SUA FICHA, E ELAS POUSAM EM LADOS OPOSTOS.**
///
/// ⚠️ Sem isto, uma cena podia publicar as duas no mesmo sítio e o rótulo diria a coisa certa
/// sobre a figura errada — o modo de falha que uma legenda tem de propósito e um `eprintln!`
/// não tem.
#[test]
fn every_scene_labels_both_halves_on_opposite_sides() {
    for (name, all) in scenes() {
        assert_eq!(all.len() % 2, 0, "{name}: uma ficha por metade");
        for pair in all.chunks_exact(2) {
            let (l, r) = (&pair[0], &pair[1]);
            assert!(
                l.world[0] < 0.0,
                "{name}: a ficha da esquerda pousa à esquerda"
            );
            assert!(r.world[0] > 0.0, "{name}: e a da direita, à direita");
            assert_eq!(l.world[1], r.world[1], "{name}: as duas à mesma altura");
            assert_ne!(l.text, r.text, "{name}: e elas dizem coisas diferentes");
        }
        // ⚠️ **E nenhuma ficha pousa sobre a linha SEGUINTE.** Numa cena apertada (`ROW_GAP` de
        // 1,55) uma folga escolhida a olho poria o rótulo da linha 2 em cima da figura da 1 — o
        // defeito que uma legenda tem e um `eprintln!` não tinha.
        let mut ys: Vec<f32> = all.iter().map(|c| c.world[1]).collect();
        ys.dedup();
        for w in ys.windows(2) {
            assert!(
                w[0] > w[1],
                "{name}: as fichas descem em ordem ({} depois de {})",
                w[1],
                w[0]
            );
        }
    }
}
