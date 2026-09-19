//! Os gates das fileiras do estilo.

use super::*;

/// ⭐⭐⭐ **A IDA E A VOLTA FECHAM POR CADA LINHA** — o que o painel mostra é o que a escrita põe.
///
/// ⚠️ **Por VARREDURA sobre as dez**, e não por um caso: a arrumação é uma tabela de posições, e um
/// `slot` trocado entre duas linhas é exactamente o defeito que um caso só não vê.
#[test]
fn cada_linha_le_o_que_ela_propria_escreve() {
    for l in &LINHAS {
        let escrito = if l.teto.is_some() {
            with_number(Style::default(), l.slot, 0.375)
        } else {
            with_colour(Style::default(), l.slot, [200, 40, 90])
        };
        let linha = rows(escrito, true)
            .into_iter()
            .find(|r| r.param == Param::Style(l.slot))
            .expect("a linha que acabou de ser escrita");
        if l.teto.is_some() {
            assert!(
                (linha.value - 0.375).abs() < 1e-6,
                "{}: escrevi 0,375 e a linha lê {}",
                l.key,
                linha.value
            );
        } else {
            assert_eq!(
                linha.swatch,
                Some([200, 40, 90]),
                "{}: a amostra não devolve a cor escrita",
                l.key
            );
        }
    }
}

/// ⭐⭐⭐ **ESCREVER NUMA LINHA NÃO TOCA NAS OUTRAS.**
///
/// ⛔ É a metade que um gate de ida-e-volta sozinho não tem: um `slot` errado devolve o valor certo
/// **e** estraga o vizinho, e a primeira metade fica verde.
#[test]
fn escrever_numa_linha_nao_toca_nas_outras() {
    for l in &LINHAS {
        let escrito = if l.teto.is_some() {
            with_number(Style::default(), l.slot, 0.375)
        } else {
            with_colour(Style::default(), l.slot, [200, 40, 90])
        };
        let antes = wgsl::pack(&Style::default());
        let depois = wgsl::pack(&escrito);
        let largura = if l.teto.is_some() { 1 } else { 3 };
        for (i, (a, b)) in antes.iter().zip(&depois).enumerate() {
            let meu = (l.slot as usize..l.slot as usize + largura).contains(&i);
            assert!(
                meu || (a - b).abs() < f32::EPSILON,
                "{}: escrever no slot {} mexeu no slot {i}",
                l.key,
                l.slot
            );
        }
    }
}

/// ⭐⭐ **AS DEZ LINHAS COBREM OS VINTE NÚMEROS, e cada um UMA vez.**
///
/// ⛔ Uma posição que nenhuma linha alcance é um knob **vivo e inalcançável** — a espécie que o
/// `CLAUDE.md` §5.0 separa do knob morto, e cuja cura é OPOSTA.
#[test]
fn as_linhas_cobrem_a_arrumacao_inteira_e_sem_repetir() {
    let mut visto = [0u8; wgsl::PACKED];
    for l in &LINHAS {
        let largura = if l.teto.is_some() { 1 } else { 3 };
        for v in visto.iter_mut().skip(l.slot as usize).take(largura) {
            *v += 1;
        }
    }
    let orfas: Vec<usize> = (0..wgsl::PACKED).filter(|&i| visto[i] == 0).collect();
    assert!(orfas.is_empty(), "posições sem linha nenhuma: {orfas:?}");
    let repetidas: Vec<usize> = (0..wgsl::PACKED).filter(|&i| visto[i] > 1).collect();
    assert!(
        repetidas.is_empty(),
        "posições com DUAS linhas — duas superfícies sobre um valor: {repetidas:?}"
    );
}

/// ⭐⭐⭐ **FORA DO RENDER NÃO HÁ FILEIRA** — *uma affordance que não pode ser honrada é pior do que
/// nenhuma*, e no matcap o estilo não corre.
#[test]
fn no_matcap_a_seccao_nao_e_oferecida() {
    assert!(
        rows(Style::default(), false).is_empty(),
        "o matcap oferece estilo"
    );
    assert_eq!(rows(Style::default(), true).len(), LINHAS.len());
    // ⚠️ E a secção abre-se UMA vez — duas cabeçalhos seriam duas secções com o mesmo nome.
    let cabecalhos = rows(Style::default(), true)
        .iter()
        .filter(|r| r.section == Some(SECCAO))
        .count();
    assert_eq!(cabecalhos, 1, "a secção abre {cabecalhos} vezes");
}

/// ⭐ **O tecto de cada número é o que a tabela declara** — e o da largura sai da LEI, não daqui.
///
/// ⚠️ *Um tecto escrito duas vezes é a segunda resposta que envelhece*: a largura do contorno tem
/// dono (`ph2d_style::Rim::MAX_WIDTH`), e este gate prende as duas pontas.
#[test]
fn o_tecto_da_largura_sai_da_lei() {
    let largura = LINHAS
        .iter()
        .find(|l| l.key == "panel.model3d.style.rim_width")
        .expect("a linha da largura");
    assert!(
        (largura.teto.expect("é um número") - ph2d_style::Rim::MAX_WIDTH).abs() < f32::EPSILON,
        "o tecto da largura no painel não é o da lei"
    );
}

/// ⭐⭐⭐ **O CLIQUE CHEGA AO ESTILO DA CENA — a costura, pelo DRENO do produto.**
///
/// # ⛔⛔ Porque este gate existe, e porque ele mede o dreno e não a função
///
/// *Um controlo nunca pintado e um MORTO SOB O DEDO dão o mesmo report*, e esta casa pagou essa
/// lição sete vezes só no módulo da escultura. Os gates acima medem a TABELA — eles ficariam verdes
/// com o braço do `match` a faltar, ou posto **depois** do genérico, que é o mesmo defeito escrito
/// de outra maneira.
///
/// ⚠️ **A ordem dos braços é load-bearing** e é ela que este gate prende: o braço genérico do
/// `SetColor` casa com QUALQUER âncora, e um braço de estilo a seguir a ele seria inalcançável.
#[test]
fn o_dreno_do_produto_escreve_no_estilo_da_cena() {
    // ⚠️ **O módulo tem de estar ARMADO** — o `with_smoke` devolve `None` a um módulo fechado, e
    // um gate que ignorasse esse `None` mediria o nada. (A 1.ª redacção deste gate estoirou aqui, o
    // que é o modo de falha bom.)
    crate::smoke::set_armed_by_panel(true);
    crate::smoke::with_smoke(|s| s.set_style(Style::default()));
    let mut world = bevy_ecs::world::World::new();

    // (1) um NÚMERO — a nitidez da curvatura.
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: 0,
        param: Param::Style(7),
        value: 3.5,
    });
    crate::scene::apply_intents_for_test(&mut world, &[]);
    let lido = crate::smoke::with_smoke(|s| s.style).expect("a cena");
    assert!(
        (lido.curvature.sharpness - 3.5).abs() < 1e-6,
        "o clique num número não chegou ao estilo: {}",
        lido.curvature.sharpness
    );

    // (2) e uma COR — a tinta da aresta, pela âncora.
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        entity: 0,
        anchor: Param::Style(4),
        srgb: [255, 0, 0],
    });
    crate::scene::apply_intents_for_test(&mut world, &[]);
    let lido = crate::smoke::with_smoke(|s| s.style).expect("a cena");
    assert!(
        lido.curvature.convex[0] > lido.curvature.convex[1],
        "o clique numa cor não chegou ao estilo: {:?}",
        lido.curvature.convex
    );
    // ⚠️ **E o número escrito antes SOBREVIVEU** — sem esta metade, um dreno que reescrevesse o
    // estilo inteiro a cada intent passaria.
    assert!(
        (lido.curvature.sharpness - 3.5).abs() < 1e-6,
        "a escrita da cor apagou o número que estava lá"
    );

    // ⚠️⚠️ **CONTROLO:** o mesmo dreno com um param de OUTRA família não pode tocar no estilo.
    //
    // ⛔ **E ele leva uma entidade DE VERDADE**, não o `0` das linhas de estilo: a 1.ª redacção
    // passou `0` e o `Entity::from_bits(0)` fez **pânico** dentro do braço genérico. *É o modo de
    // falha bom* — e diz uma coisa sobre o desenho: o `0` das linhas de estilo só é seguro porque
    // os braços delas vêm ANTES, e no dia em que alguém os mover o produto **estoura em vez de
    // escrever no sítio errado em silêncio**.
    let alheia = world.spawn_empty().id();
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: alheia.to_bits(),
        param: Param::Material(0),
        value: 0.123,
    });
    crate::scene::apply_intents_for_test(&mut world, &[]);
    let depois = crate::smoke::with_smoke(|s| s.style).expect("a cena");
    assert_eq!(depois, lido, "um param de outra família mexeu no estilo");
    crate::smoke::with_smoke(|s| s.set_style(Style::default()));
}
