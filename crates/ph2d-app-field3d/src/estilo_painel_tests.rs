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

/// ⭐⭐⭐ **CADA COR DO ESTILO TEM UMA AMOSTRA SÓ SUA** (report do Enio, 2026-09-19: *«se modifico
/// qualquer cor em style, todas mudam ao mesmo tempo»*).
///
/// # ⛔⛔ O defeito, e porque os cinco gates acima ficaram VERDES por cima dele
///
/// O selector de cor da casa é **um** e flutua; um painel entra nele registando o `NodeId` da
/// amostra. O id era cunhado `(entidade, campo)`, e as cinco cores do estilo têm `entity = 0` (o
/// estilo não é de entidade nenhuma) **e** caíam no braço final do `match`, que respondia `campo =
/// 0` ⇒ as cinco partilhavam `hash("model3d.color.swatch.0.0")`. Com o selector aberto numa, as
/// cinco liam *«aberto em mim»* e as cinco pediam a escrita.
///
/// ⚠️⚠️ **Os gates acima medem a LEI e o DRENO, e o defeito vive ENTRE os dois** — na identidade com
/// que a fileira é pintada. O gate da costura alimenta o dreno com a âncora já certa, logo ele entra
/// **abaixo** da rotura: é a lei que o `CLAUDE.md` §5.0 escreve como *«nenhum instrumento pergunta
/// se o VALOR chega a um consumidor»*, aqui na forma *«nenhum perguntava se duas fileiras são o
/// MESMO controlo»*.
#[test]
fn cada_cor_do_estilo_tem_uma_amostra_so_sua() {
    let fileiras = rows(Style::default(), true);
    let cores: Vec<_> = fileiras.iter().filter(|r| r.swatch.is_some()).collect();
    // ⚠️ **Piso de população**: sem ele, uma varredura que deixasse de achar cor nenhuma ficaria
    // trivialmente verde — e a secção tem cinco.
    assert_eq!(
        cores.len(),
        LINHAS.iter().filter(|l| l.teto.is_none()).count(),
        "a varredura não achou as cores do estilo"
    );
    let mut vistos: Vec<(u64, &'static str)> = Vec::new();
    for c in &cores {
        let id = ph2d_panel_model3d::swatch_id(c)
            .unwrap_or_else(|| panic!("{}: uma cor sem amostra é uma cor inalcançável", c.key))
            .0;
        if let Some((_, quem)) = vistos.iter().find(|(v, _)| *v == id) {
            panic!(
                "{} e {} são o MESMO controlo (id {id}) — mexer numa mexe na outra",
                quem, c.key
            );
        }
        vistos.push((id, c.key));
    }
}

/// ⭐⭐ **E a amostra do estilo NUNCA é a de um objecto** — a não-colisão entre as duas famílias.
///
/// ⛔ Sem este gate, a segurança do sentinela `entity = 0` dependeria do acidente de
/// `Entity::to_bits()` nunca valer zero. *Uma propriedade que vale por acidente é a que cai no dia
/// em que outra crate muda uma representação* — aqui ela é inexprimível: os dois nomes diferem.
#[test]
fn a_amostra_do_estilo_nao_colide_com_a_de_um_objecto() {
    let alheias: Vec<ph2d_panel_model3d::ParamRow> = [0u8, 4, 8, 12, 16]
        .into_iter()
        .flat_map(|k| [Param::Material(k), Param::Light(k)])
        .map(|param| ph2d_panel_model3d::ParamRow {
            entity: 0,
            param,
            key: "x",
            value: 0.0,
            lo: 0.0,
            bound: Bound::Soft(1.0),
            inert: None,
            integral: false,
            choices: &[],
            section: None,
            swatch: Some([0, 0, 0]),
            subject: None,
        })
        .collect();
    let deles: Vec<u64> = alheias
        .iter()
        .filter_map(|r| ph2d_panel_model3d::swatch_id(r).map(|i| i.0))
        .collect();
    assert_eq!(deles.len(), alheias.len(), "uma fileira de objecto sem id");
    for c in rows(Style::default(), true)
        .iter()
        .filter(|r| r.swatch.is_some())
    {
        let meu = ph2d_panel_model3d::swatch_id(c)
            .expect("a amostra do estilo")
            .0;
        assert!(
            !deles.contains(&meu),
            "{}: a amostra do estilo tem o id de uma cor de objecto",
            c.key
        );
    }
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
