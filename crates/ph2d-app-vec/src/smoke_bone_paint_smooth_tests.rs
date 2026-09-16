//! ⭐⭐⭐⭐ **O BOTÃO `Smooth` ENTREGA — os gates do controlo que estava MORTO.**
//!
//! Irmão do [`super`] por RESPONSABILIDADE: *a arte obedece à lei da pele* é uma pergunta (a dele),
//! *o segundo segmento do par `Fast`/`Smooth` faz alguma coisa* é outra.
//!
//! # O defeito (auditoria de 2026-09-16)
//!
//! O painel *Bones* tem um par de segmentos `Fast`/`Smooth` ([`ph2d_tool_vector::SkinDeform::ALL`],
//! registado e pintado). Medido nesta cena, o `Smooth` entregava **a malha do `Fast`, ao bit**: a
//! lei de refinamento de então era o `k` GLOBAL, cujo tecto é `⌊√(orçamento / peças)⌋`, e toda malha
//! acima de `orçamento / 4` peças só admite `k = 1`.
//!
//! ⛔⛔ **É a terceira espécie de controlo morto do `CLAUDE.md` §5.0, e a sonda de registo é cega a
//! ela:** o id existe, é pintado, é registado, o clique chega à ferramenta e o valor chega ao
//! consumidor — *e o consumidor devolve a entrada*. Nenhum censo deste repo pergunta se a saída
//! MUDA.
//!
//! ⇒ é essa a pergunta que estes gates fazem, e o controlo da resposta é a lei antiga a reprovar.

use super::*;

/// O pior desvio da malha que a lei `refine` entrega — em pixels de ECRÃ.
///
/// ⚠️⚠️ **Ela refina com a MESMA porta da `ph2d-poly2d` que o produto chama**, e mede o desvio
/// **na malha refinada** — ⛔ medir a malha guardada depois de refinar mediria a outra.
fn faceta_da_malha(sim: &SimWorld, e: Entity, refine: Option<ph2d_poly2d::RefineOptions>) -> f64 {
    let (sm, p2l, pele) = campo_da_cena(sim, e);
    let mut w = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut w);
    let ossos = sm.ossos();
    let Some(o) = refine else {
        let posadas: Vec<[f64; 2]> = sm
            .mesh
            .rest
            .iter()
            .enumerate()
            .map(|(v, &q)| campo(q, sm.pesos_de(v)))
            .collect();
        return ph2d_poly2d::deviation_attrs(&sm.mesh, &posadas, &sm.pesos, ossos, &mut campo)
            * PX_POR_METRO;
    };
    let (m, p, a, _rel) =
        ph2d_poly2d::refine_posed_attrs(&sm.mesh, &sm.pesos, ossos, &mut campo, o);
    ph2d_poly2d::deviation_attrs(&m, &p, &a, ossos, &mut campo) * PX_POR_METRO
}

/// As opções do `Smooth` desta cena, com a lei e o ZOOM escolhidos.
///
/// ⚠️ **A tolerância chega em unidades LOCAIS**, como o quadro a entrega: o produto divide os
/// `0,5 px` de ecrã pela escala da câmara ([`PX_POR_METRO`] vezes o zoom) antes de entrar na folha.
///
/// ⭐⭐⭐ **E o ZOOM é a alavanca do produto, não um parâmetro de conveniência.** *A suavidade que o
/// olho vê é um facto de espaço de ECRÃ*: meia unidade local é meio pixel a zoom `1` e quatro a
/// zoom `8`. Uma arte que está lisa o suficiente enquadrada inteira deixa de estar quando o artista
/// se aproxima — e é exactamente aí que o botão tem de servir.
fn opcoes(adaptativo: bool, zoom: f64) -> ph2d_poly2d::RefineOptions {
    ph2d_poly2d::RefineOptions {
        tolerance_px: 0.5 / (PX_POR_METRO * zoom),
        max_pieces: ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES,
        adaptativo,
    }
}

/// ⭐⭐⭐⭐ **O `Smooth` ENTREGA UMA DOBRA MAIS LISA QUE O `Fast`, NA CENA DO DONO** — e a lei
/// ANTIGA, no mesmo orçamento, entrega a do `Fast` ao bit.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate media a cena ENQUADRADA INTEIRA e reprovou com razão:** ali o
/// `Fast` já entrega `0,34 px` de faceta, **abaixo** da promessa de meio pixel, e o `Smooth` não
/// refinar é a resposta CERTA. *Um gate que exige refinamento onde ele não é preciso mede a
/// fixtura, não a lei.* A alavanca do produto é o ZOOM — ver [`opcoes`].
///
/// A tabela que esta fixtura imprime (corrente de três ossos a `25°`, arte `512 × 200`, malha de
/// bind de `2 268` peças):
///
/// | zoom | `Fast` | `Smooth` uniforme | `Smooth` adaptativo |
/// |---:|---:|---:|---:|
/// | `1×` | `0,34 px` | `0,34` (`2 268`) | `0,34` (`2 268`) — **nem precisa** |
/// | `2×` | `0,68 px` | `0,68` (`2 268`) | **`0,50`** (`2 364`) |
/// | `4×` | `1,37 px` | `1,37` (`2 268`) | **`0,50`** (`3 416`) |
/// | `8×` | `2,74 px` | `2,74` (`2 268`) | **`0,54`** (`4 720`) |
///
/// ⭐⭐⭐ **A leitura é toda a wave numa tabela:** a lei antiga entrega `2 268` peças em todo zoom
/// (a malha guardada, ao bit), e a nova segura a promessa de meio pixel até `8×`, onde ela para por
/// ter gasto `4 720` das `4 721` peças do orçamento — *o único sítio onde o tecto fala.*
///
/// ⛔ **A coluna do meio ser SEMPRE igual à primeira é o defeito**, e é por isso que ela fica no
/// gate: sem esse controlo, um `Smooth` que voltasse a ficar inerte passava outra vez.
///
/// (Mutação: o despachante ignorar `opts.adaptativo` ⇒ RED.)
#[test]
fn o_smooth_deixou_de_ser_um_controlo_morto_na_cena_do_produto() {
    const ALTURA: u32 = 200;
    let (sim, e) = cena(ALTURA, None);
    let guardada = campo_da_cena(&sim, e).0.mesh.tris.len();
    let orcamento = ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES;

    // A faceta é medida em pixels de ECRÃ **no zoom em que se olha** — logo ela cresce com ele.
    let mut mordeu = 0usize;
    for zoom in [1.0_f64, 2.0, 4.0, 8.0] {
        let fast = faceta_da_malha(&sim, e, None) * zoom;
        let uniforme = faceta_da_malha(&sim, e, Some(opcoes(false, zoom))) * zoom;
        let adaptativo = faceta_da_malha(&sim, e, Some(opcoes(true, zoom))) * zoom;
        let pecas_u = pecas_entregues(&sim, e, Some(opcoes(false, zoom)));
        let pecas_a = pecas_entregues(&sim, e, Some(opcoes(true, zoom)));
        println!(
            "zoom {zoom:>4.0}x | Fast {fast:>6.2} px | uniforme {pecas_u:>5} pecas {uniforme:>6.2}              px | adaptativo {pecas_a:>5} pecas {adaptativo:>6.2} px"
        );

        // ⛔ **O CONTROLO: a lei antiga é inerte nesta cena, em TODO zoom.** É a reprodução do
        // defeito, e se um dia ela deixar de valer, esta linha diz que a premissa mudou.
        assert_eq!(
            pecas_u, guardada,
            "zoom {zoom}x: a premissa do defeito caiu — a lei uniforme passou a refinar aqui"
        );
        assert!(
            (uniforme - fast).abs() < 1e-9,
            "zoom {zoom}x: a lei uniforme mexeu na faceta ({fast:.3} -> {uniforme:.3} px)"
        );
        assert!(
            pecas_a <= orcamento,
            "zoom {zoom}x: o Smooth furou o orcamento do quadro ({pecas_a} pecas)"
        );

        // ⭐⭐⭐ Onde a promessa de meio pixel é violada, a lei nova TEM de morder.
        if fast > 0.5 {
            mordeu += 1;
            assert!(
                pecas_a > guardada,
                "zoom {zoom}x: a faceta e' {fast:.2} px e o Smooth entregou as mesmas {guardada}                  pecas — o botao nao faz nada onde ele e' preciso"
            );
            assert!(
                adaptativo < fast,
                "zoom {zoom}x: o Smooth so' levou a faceta de {fast:.2} para {adaptativo:.2} px"
            );
        }
    }
    // ⚠️ E a escada tem de ALCANÇAR o fenómeno: sem isto o laço passa por vacuidade sobre uma
    // fixtura que nunca sai da tolerância. *Uma escada que começa no ponto neutro do knob não
    // testa o knob.*
    assert!(
        mordeu >= 2,
        "a escada de zoom nunca passou de meio pixel de faceta — ela nao alcanca a pergunta"
    );
}

/// Quantas peças a lei `refine` entrega nesta cena.
fn pecas_entregues(sim: &SimWorld, e: Entity, refine: Option<ph2d_poly2d::RefineOptions>) -> usize {
    let (sm, p2l, pele) = campo_da_cena(sim, e);
    let mut w = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut w);
    let ossos = sm.ossos();
    match refine {
        None => sm.mesh.tris.len(),
        Some(o) => {
            let (m, _, _, _) =
                ph2d_poly2d::refine_posed_attrs(&sm.mesh, &sm.pesos, ossos, &mut campo, o);
            m.tris.len()
        }
    }
}

/// ⛔⛔ **A MALHA GUARDADA CABE NO ORÇAMENTO DO QUADRO** — senão o `Smooth` nasce sem onde crescer.
///
/// ⚠️⚠️ **Esta é a outra metade do item 2 da auditoria:** até 2026-09-16 o orçamento era `1 543`
/// peças (derivado de um custo por peça de `1,08 µs` que o produto **nunca pagava**, porque a lei
/// de então não refinava), e a malha de bind desta cena mede mais que isso ⇒ o
/// `avisa_malhas_acima_do_orcamento` disparava em **toda** execução, sobre um aviso correcto e um
/// número errado.
///
/// ⭐ Com o custo remedido (`0,353 µs`) o orçamento é `4 721`, e a malha cabe com folga.
#[test]
fn a_malha_do_bind_cabe_no_orcamento_do_quadro() {
    const ALTURA: u32 = 200;
    let (sim, e) = cena(ALTURA, None);
    let guardada = campo_da_cena(&sim, e).0.mesh.tris.len();
    let orcamento = ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES;
    println!("malha guardada {guardada} pecas | orcamento do quadro {orcamento}");
    assert!(
        guardada < orcamento,
        "a malha que o bind guarda ({guardada}) nao cabe no orcamento do quadro ({orcamento}) — o \
         Smooth nasce sem onde crescer e o aviso dispara em toda execucao"
    );
    // ⭐ E tem de sobrar espaço a sério: refinar pede múltiplos da malha, não uma peça a mais.
    assert!(
        guardada * 2 <= orcamento,
        "so' sobram {} pecas de folga sobre {guardada} guardadas — nao da' para refinar",
        orcamento.saturating_sub(guardada)
    );
}
