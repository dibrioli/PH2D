//! ⭐ **Os gates do laço do preview** (W24).
//!
//! ⚠️ **O gate-mãe usa a TABELA MEDIDA como oráculo.** Uma lei de realimentação não se prova com um
//! caso: prova-se **fechando o laço** — alimentar a decisão com o custo real que aquela escolha teve
//! na máquina, e exigir que ela **assente** dentro do orçamento em poucos passos. É o que separa
//! *"a fórmula devolve um número plausível"* de *"o laço converge"*, e a diferença tem nome: um laço
//! mal posto oscila entre dois divisores para sempre, e o artista vê a imagem a piscar entre nítida
//! e grossa.

use super::*;
use ph2d_field_render::Orbit;

/// **O custo real de um traçado, medido nesta workstation** (release, máquina calma, 1920×1080).
///
/// Sonda: `probe_how_coarse_a_preview_can_be`, 22/08. Índice = divisor.
///
/// ⚠️ **Estes números não são um alvo, são o oráculo**: se o traçador ficar 3× mais rápido, o gate
/// continua a medir a mesma coisa — que o laço assenta dentro do orçamento que lhe derem.
const MEASURED_MS: [(u32, f32, f32); 4] = [
    // (divisor, cena 1 — três cilindros, cena 6 — escultura com furo)
    (1, 46.0, 121.0),
    (2, 17.8, 33.6),
    (3, 11.0, 16.6),
    (4, 9.4, 10.4),
];

const FULL: (u32, u32) = (1920, 1080);

/// O custo que a máquina cobraria por um traçado deste tamanho — interpolado da tabela pelo divisor
/// mais próximo.
fn measured_cost(size: (u32, u32), sculpture: bool) -> f32 {
    let d = (FULL.0 as f32 / size.0 as f32).round().max(1.0) as u32;
    let row = MEASURED_MS
        .iter()
        .min_by_key(|(rd, _, _)| rd.abs_diff(d))
        .expect("a tabela não é vazia");
    if sculpture { row.2 } else { row.1 }
}

/// ⭐ **O LAÇO ASSENTA, e assenta DENTRO do orçamento.**
///
/// ⚠️ **A previsão é otimista de propósito e o gate sabe disso**: o custo por pixel *sobe* quando a
/// imagem encolhe (o anti-serrilhado corre sobre as arestas, e há proporcionalmente mais aresta numa
/// imagem pequena), então prever o grosso a partir do cheio erra para baixo. O que se exige não é
/// que a primeira escolha acerte — é que a **segunda** corrija, e que a coisa **pare**.
#[test]
fn the_loop_settles_inside_the_budget_on_the_measured_scenes() {
    for (name, sculpture) in [("cena 1", false), ("cena 6", true)] {
        let mut measured: Option<Measured> = None;
        let mut history: Vec<(u32, u32)> = Vec::new();
        for _ in 0..6 {
            let size = preview_size(FULL, measured, PREVIEW_BUDGET_MS, 16);
            let cost = measured_cost(size, sculpture);
            measured = Some(Measured {
                pixels: u64::from(size.0) * u64::from(size.1),
                millis: cost,
            });
            history.push(size);
        }
        let last = *history.last().expect("seis passos");
        let before = history[history.len() - 2];
        assert_eq!(
            last, before,
            "{name}: o laço tem de ASSENTAR — ele andou {history:?}, e uma imagem a piscar entre \
             dois divisores é pior do que uma sempre grossa"
        );
        let settled = measured_cost(last, sculpture);
        assert!(
            settled <= PREVIEW_BUDGET_MS,
            "{name}: assentou em {last:?}, que custa {settled} ms — fora do orçamento de \
             {PREVIEW_BUDGET_MS} ms, e o preview continua a arrastar-se"
        );
        // ⚠️ E o controle: sem o laço isto NÃO acontecia. O traçado cheio é o ponto de partida, e
        // ele tem de estar fora do orçamento — senão o gate passaria com a lei apagada.
        let full_cost = measured_cost(FULL, sculpture);
        assert!(
            full_cost > PREVIEW_BUDGET_MS,
            "{name}: o traçado cheio custa {full_cost} ms e já cabia no orçamento — este gate não \
             está a medir nada"
        );
    }
}

/// **Sem medição, traça CHEIO** — o primeiro traçado é a medição.
///
/// ⚠️ É também o que faz a primeira coisa que o artista vê ser a peça **nítida**: a suavização só
/// aparece depois, em movimento, que é onde ela não se nota.
#[test]
fn the_first_trace_is_full_because_it_is_the_measurement() {
    assert_eq!(preview_size(FULL, None, PREVIEW_BUDGET_MS, 16), FULL);
    // …e uma medição sem sentido não vale como medição.
    for bad in [
        Measured {
            pixels: 0,
            millis: 50.0,
        },
        Measured {
            pixels: 1000,
            millis: f32::NAN,
        },
        Measured {
            pixels: 1000,
            millis: 0.0,
        },
    ] {
        assert_eq!(
            preview_size(FULL, Some(bad), PREVIEW_BUDGET_MS, 16),
            FULL,
            "uma medição impossível ({bad:?}) tem de cair no cheio, nunca numa divisão por zero"
        );
    }
}

/// ⭐ **Assentar refina para o cheio — e UMA vez só.**
///
/// ⚠️ O «uma vez» é a metade que custa: sem ele, uma peça parada e já nítida re-traçaria a cada
/// quadro, queimando um núcleo para produzir a imagem que já está na tela.
#[test]
fn a_settled_view_refines_to_full_exactly_once() {
    let cam = Orbit::default();
    let doc = crate::field3d_smoke::scene(1);
    let coarse = (FULL.0 / 3, FULL.1 / 3);

    assert_eq!(
        next_trace(
            Some((&cam, coarse.0, coarse.1, &doc)),
            &cam,
            &doc,
            FULL,
            None,
            true,
            16
        ),
        Some(FULL),
        "nada mudou e o que está na tela é grosso: refina"
    );
    assert_eq!(
        next_trace(
            Some((&cam, FULL.0, FULL.1, &doc)),
            &cam,
            &doc,
            FULL,
            None,
            true,
            16
        ),
        None,
        "e depois de refinado, nada — senão é um núcleo a queimar por quadro"
    );
}

/// ⭐ **O movimento pede GROSSO, e nunca mais grosso que o piso.**
///
/// ⚠️ O piso é o que impede o laço de responder a uma peça pesadíssima com papa: ele fica preso em
/// [`MAX_PREVIEW_DIVISOR`] e a imagem fica **lenta**, que é a direção conservadora para um módulo
/// cuja razão de existir é a aresta.
#[test]
fn movement_asks_for_coarse_and_never_below_the_floor() {
    let cam = Orbit::default();
    let doc = crate::field3d_smoke::scene(1);
    let mut moved = cam;
    crate::field3d_input::law::orbit(&mut moved, 10.0, 0.0);

    // Uma peça absurdamente cara: 10 segundos por quadro cheio.
    let awful = Measured {
        pixels: u64::from(FULL.0) * u64::from(FULL.1),
        millis: 10_000.0,
    };
    let asked = next_trace(
        Some((&cam, FULL.0, FULL.1, &doc)),
        &moved,
        &doc,
        FULL,
        Some(awful),
        true,
        16,
    )
    .expect("mexeu: tem de traçar");
    assert_eq!(
        asked,
        (FULL.0 / MAX_PREVIEW_DIVISOR, FULL.1 / MAX_PREVIEW_DIVISOR),
        "nem com 10 s por quadro o preview desce abaixo do piso"
    );
    assert!(
        asked.0 < FULL.0,
        "e uma peça que não cabe no orçamento TEM de sair mais grossa — senão a wave não existe"
    );
}

/// ⭐ **O PISO é o divisor mais raso que cabe no orçamento** — nem mais fundo, nem mais raso.
///
/// ⚠️ **Uma prova de mutação passou VERDE e é por isso que este gate existe** (a segunda vez nesta
/// linha, depois do `SCULPT_SLOT` na W22). Pôr [`MAX_PREVIEW_DIVISOR`] a 8 não punha nada a
/// vermelho, porque o gate irmão **compara o resultado com a própria constante**: mutá-la move a
/// produção e a expectativa do teste ao mesmo tempo. *Um teste que lê a constante que testa não
/// testa a constante.*
///
/// O que se mede aqui é a **relação com a tabela medida**, que é de onde o número veio:
///
/// - a `MAX_PREVIEW_DIVISOR` a cena mais pesada **cabe** no orçamento (senão o piso é raso demais e
///   o preview arrasta-se);
/// - a `MAX_PREVIEW_DIVISOR − 1` ela **não cabe** (senão o piso é fundo demais, e o módulo está a
///   trocar nitidez por milissegundos que ninguém pediu).
#[test]
fn the_floor_is_the_shallowest_divisor_that_fits_the_budget() {
    let at = |d: u32| measured_cost((FULL.0 / d.max(1), FULL.1 / d.max(1)), true);
    let floor = MAX_PREVIEW_DIVISOR;
    assert!(
        at(floor) <= PREVIEW_BUDGET_MS,
        "a D={floor} a cena mais pesada custa {} ms e o orçamento é {PREVIEW_BUDGET_MS} — o piso é \
         raso demais, e o preview arrasta-se mesmo no fundo",
        at(floor)
    );
    assert!(
        floor > 1 && at(floor - 1) > PREVIEW_BUDGET_MS,
        "a D={} ela já custava {} ms, dentro do orçamento — o piso está mais fundo do que precisa, \
         e isso é nitidez trocada por milissegundos que ninguém pediu",
        floor - 1,
        at(floor - 1)
    );
}

/// ⭐ **Uma peça BARATA não é suavizada** — a outra metade da lei, e a que se esquece.
///
/// ⚠️ Um laço que respondesse *sempre* com o divisor máximo passaria em todos os gates de custo
/// acima: o preview caberia no orçamento, o laço assentaria, e a imagem ficaria grossa **para
/// sempre**, inclusive numa peça de duas primitivas que a máquina traça em 8 ms. *Medir a presença
/// da degradação sem medir a ausência dela é meio gate.*
#[test]
fn a_cheap_piece_is_never_softened() {
    let cheap = Measured {
        pixels: u64::from(FULL.0) * u64::from(FULL.1),
        millis: 8.0,
    };
    assert_eq!(
        preview_size(FULL, Some(cheap), PREVIEW_BUDGET_MS, 16),
        FULL,
        "o traçado cheio já cabe no orçamento: baixar a resolução seria perder nitidez de graça"
    );

    // E o mesmo pela porta que a produção usa.
    let cam = Orbit::default();
    let doc = crate::field3d_smoke::scene(1);
    let mut moved = cam;
    crate::field3d_input::law::orbit(&mut moved, 10.0, 0.0);
    assert_eq!(
        next_trace(
            Some((&cam, FULL.0, FULL.1, &doc)),
            &moved,
            &doc,
            FULL,
            Some(cheap),
            true,
            16
        ),
        Some(FULL),
        "mexer numa peça barata continua a traçar CHEIO"
    );
}

/// **Uma área degenerada nunca pede zero pixels.**
#[test]
fn a_sliver_of_an_area_never_asks_for_zero_pixels() {
    let tiny = (20u32, 300u32);
    let heavy = Measured {
        pixels: 6000,
        millis: 900.0,
    };
    let (w, h) = preview_size(tiny, Some(heavy), PREVIEW_BUDGET_MS, 16);
    assert!(
        w >= 16 && h >= 16,
        "o piso de {w}x{h} tem de respeitar o MIN_TRACE — um traçado de zero pixels é um pânico \
         à espera"
    );
}

/// ⭐ **Um REFINAMENTO cede à mão; um traçado de MOVIMENTO nunca** (W32).
///
/// ⚠️ **A segunda metade é a que mata a regra óbvia.** *"Mudou? abandona o que está a correr"* tem um
/// modo de falha fatal: numa órbita contínua a câmera muda **a cada quadro**, e um traçado grosso que
/// leve mais do que um quadro seria cancelado antes de acabar, **sempre** — o artista arrastaria o
/// rato contra uma imagem **congelada**. Um refinamento, esse, só começa quando nada está a mudar:
/// ele nunca está no caminho de si mesmo.
#[test]
fn a_refinement_yields_to_the_hand_and_a_motion_trace_never_does() {
    let full = FULL;
    let coarse = (full.0 / 3, full.1 / 3);

    assert!(
        cancels_the_inflight(full, coarse, full),
        "um refinamento cheio com a mão a pedir grosso TEM de ser abandonado — era a espera de 121 ms"
    );
    assert!(
        !cancels_the_inflight(coarse, coarse, full),
        "⛔ um traçado de MOVIMENTO nunca é cancelado — senão a imagem congela numa órbita contínua"
    );
    assert!(
        !cancels_the_inflight(coarse, full, full),
        "…nem quando o que se pede a seguir é o refinamento: a imagem grossa é a que está a chegar"
    );
    assert!(
        !cancels_the_inflight(full, full, full),
        "…e um refinamento não se cancela a si próprio"
    );
}

/// ⭐ **A projeção do gizmo NÃO é construída no desenho** — ela tem um dono, e é o da área.
///
/// ⚠️ **É a costura que esta wave podia partir e que nenhum gate de aritmética alcança.** Até aqui o
/// tamanho do traçado e o da área eram o mesmo número, e o desenho projetava as alças a partir do do
/// traçado — correto por coincidência. Com o preview grosso os dois divergem, e uma `Screen::new` no
/// desenho poria o gizmo a um terço do tamanho: as setas agarrariam longe da superfície **só durante
/// o movimento**, que é o defeito mais difícil de reproduzir que este módulo poderia ter.
#[test]
fn the_draw_does_not_build_its_own_projection() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/field3d_smoke_draw.rs"),
    )
    .expect("o arquivo do desenho existe");
    let offenders: Vec<&str> = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .filter(|l| l.contains("Screen::new"))
        .collect();
    assert!(
        offenders.is_empty(),
        "o desenho tem de pedir a projeção a `field3d_input::area_screen`, não construir a dele: \
         {offenders:?}"
    );
}
