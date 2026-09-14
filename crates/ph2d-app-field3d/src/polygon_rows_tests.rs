//! ⭐⭐⭐ **O TETO DE VÉRTICES DE UM POLÍGONO É O DA FAMÍLIA DE LINHAS DO PAINEL** (W132) — e este
//! ficheiro é a medição, não uma nota ao lado do número.
//!
//! # ⚠️ De que recurso o teto é (`CLAUDE.md` §0)
//!
//! Não é o relógio. A sonda `measure_polygon_vertices` mediu o preço por ponto e ele é **linear e
//! caro** (`11,3×` um cilindro a 24 vértices), mas *o artista não tem rota mais barata*: a única
//! alternativa a um contorno irregular é **desenhá-lo**, e a W131 mediu que desenhar custa
//! `2,6×`–`3,1×` a fórmula. ⛔ *Um teto de preço aqui poria o caminho lento a mandar no rápido.*
//!
//! O recurso é o **registo de widgets**: o `populate` do painel corre antes de o documento existir e
//! cunha ids às cegas para uma família de tamanho fixo (`MAX_ROWS`). Uma linha além dela fica **sem
//! controlo** — o rodapé diz quantas não couberam, mas o artista fica com o *Fillet* inalcançável.
//!
//! ⭐ E por isso o teto **prova-se** com o painel na mão, e não com aritmética ao lado dele: quem
//! acrescentar uma linha a um nó (uma pose nova, um segundo raio) vê este gate ficar vermelho **com
//! a conta dentro**, que é o momento certo para decidir o que sai.

use ph2d_field::{MAX_POLYGON_VERTICES, MIN_POLYGON_VERTICES, Primitive};

/// Um polígono de `n` vértices, irregular — o mesmo molde da sonda de preço.
fn poligono(n: u32) -> Primitive {
    let pontos: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let t = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
            let r = 0.30 + 0.10 * (3.0 * t + 0.7).sin() + 0.05 * (5.0 * t + 1.9).cos();
            [r * t.cos(), r * t.sin()]
        })
        .collect();
    Primitive::Polygon {
        profile: ph2d_field::polygon_profile(pontos).expect("o contorno"),
        half_height: 0.20,
        round: 0.01,
        chamfer: 0.0,
    }
}

/// Quantas linhas o painel mostra para um nó com esta forma — **contadas no produtor das linhas**.
///
/// ⛔⛔ **Ela contava `params_of` e isso deixou de ser a mesma grandeza em 2026-09-14**: a amostra de
/// cor (§12 do `docs/Render3d/05`) **dobra** três params numa linha, e a de emissão dobra outros
/// três. Um censo sobre os params lê `2N + 19` onde o painel pinta `2N + 15`. ⚠️ Ele erra a **favor**
/// — o que o torna invisível, e é exactamente por isso que esta nota existe: *uma régua conservadora
/// não avisa no dia em que deixa de descrever o que mede.*
///
/// ⚠️ **`tudo_aceso` é o PIOR caso e é ele que a família tem de aguentar:** com o brilho acima de
/// zero a cor da emissão é publicada, e com o verniz acima de zero saem mais **quatro** linhas
/// (§21). Uma família dimensionada no estado apagado deixaria as últimas sem controlo exactamente no
/// gesto que as fez aparecer.
///
/// ⚠️⚠️ **E o PIOR CASO tem de ser RECONFERIDO a cada número novo do material** — ele não é uma
/// propriedade da forma, é o estado em que mais linhas coexistem. *Uma lista de «o que acender»
/// escrita à mão é a segunda resposta à pergunta que o `params_of` já responde*, e é por isso que
/// esta função acende **pelos pesos**, que são os únicos que decidem visibilidade.
fn linhas_do_painel(p: Primitive, tudo_aceso: bool) -> usize {
    use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Xform};
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    if tudo_aceso {
        // O brilho (`5`) e o verniz (`9`) — os dois pesos que abrem sub-linhas.
        for peso in [5u8, 9] {
            ph2d_field_ecs::set_param(
                sim.world_mut(),
                root,
                ph2d_field::Param::Material(peso),
                1.0,
            )
            .expect("o peso");
        }
    }
    crate::scene::panel::param_rows(sim.world(), &[root], 1.0).len()
}

/// ⭐⭐⭐ **O POLÍGONO NO TETO AINDA CABE NO PAINEL** — e a folga que sobra é impressa.
///
/// ⚠️ **A barra é o `MAX_ROWS` da crate do painel, lido dela** — não um `64` escrito aqui. *Um teto
/// copiado é um teto que envelhece na wave em que o outro mudar.*
#[test]
fn every_row_of_the_biggest_polygon_fits_the_registered_family() {
    let teto = ph2d_panel_model3d::MAX_ROWS;
    println!("  vértices | tudo apagado | tudo aceso | de {teto}");
    for n in [MIN_POLYGON_VERTICES, 8, 16, MAX_POLYGON_VERTICES] {
        println!(
            "{n:>10} | {:>12} | {:>10} |",
            linhas_do_painel(poligono(n), false),
            linhas_do_painel(poligono(n), true)
        );
    }
    let no_teto = linhas_do_painel(poligono(MAX_POLYGON_VERTICES), true);
    assert!(
        no_teto <= teto,
        "um polígono de {MAX_POLYGON_VERTICES} vértices pede {no_teto} linhas e a família do painel \
         tem {teto} — as últimas ficam SEM controlo, e a última de todas é o *Fillet*. A cura é \
         baixar o `MAX_POLYGON_VERTICES`, não alargar a família em silêncio."
    );
}

/// ⭐⭐ **E O SEGUINTE JÁ NÃO CABERIA** — a outra ponta da cerca, que é o que a torna uma **medição**
/// e não uma folga escolhida.
///
/// ⚠️ **Sem esta metade o teto podia estar em qualquer número abaixo do real** e o gate de cima
/// ficaria verde na mesma — é a lição da catraca sem censo de obsolescência: *uma tolerância que só
/// se defende de um lado não descreve nada*.
///
/// ⛔ **A primeira redacção construía o polígono de `MAX + 1` e explodiu** — o documento recusa-o, e
/// com razão: é a mesma cerca que este gate existe para medir. ⇒ a lei mede-se em **dois pontos
/// válidos**, confirma-se que ela é uma recta de declive `2` (um vértice são duas linhas), e é essa
/// recta que responde pelo lado de fora. *Uma sonda que só alcança o lado de dentro do limite não
/// pode dizer que o limite está no sítio certo.*
#[test]
fn one_more_vertex_would_not_fit() {
    let teto = ph2d_panel_model3d::MAX_ROWS;
    let (a, b) = (MIN_POLYGON_VERTICES, MAX_POLYGON_VERTICES);
    let (la, lb) = (
        linhas_do_painel(poligono(a), true),
        linhas_do_painel(poligono(b), true),
    );
    let declive = (lb - la) / (b - a) as usize;
    assert_eq!(
        declive, 2,
        "as linhas de um polígono deixaram de crescer duas por vértice ({la} a {a} e {lb} a {b}) — \
         a recta abaixo passa a estar errada, e com ela o teto"
    );
    let extras = lb - 2 * b as usize;
    // ⚠️⚠️ **A história deste número em 2026-09-14, num dia só:**
    //
    // | valor | o que mudou |
    // |---|---|
    // | `10` | antes do material por objecto |
    // | `15` | os cinco números do material (13/09) |
    // | `15` | a régua passou a contar LINHAS e não params (`−4` canais dobrados) **e** o brilho
    //   próprio acrescentou `+1` — ⛔ *duas correcções de sinal oposto no mesmo literal* |
    // | **`20`** | o verniz: `+1` sempre (o peso) e `+4` com ele aceso (rugosidade, cor, IOR,
    //   escurecimento) |
    //
    // ⛔ *Um número que não se mexe enquanto a grandeza muda é a forma mais silenciosa de um gate
    // deixar de descrever o que mede* — o que o prende é esta tabela, não o literal.
    assert_eq!(
        extras, 20,
        "um nó deixou de ter 20 linhas além dos `2N` dos vértices — a conta do teto muda com isto"
    );
    let seguinte = 2 * (b as usize + 1) + extras;
    assert!(
        seguinte > teto,
        "um polígono de {} vértices ainda caberia ({seguinte} de {teto}) — o `MAX_POLYGON_VERTICES` \
         está ABAIXO do que o painel aguenta, e a forma oferece menos do que pode",
        b + 1
    );
}

/// ⭐ **Todas as linhas de um polígono têm rótulo** — a tabela de chaves cobre o teto inteiro.
///
/// ⚠️ **Uma chave sem tradução pinta o identificador CRU e vaza uma string por quadro**
/// (`ph2d_i18n::tr` faz `leak_key`), então este gate percorre a faixa toda e não uma amostra.
#[test]
fn every_vertex_row_has_a_label() {
    for n in MIN_POLYGON_VERTICES..=MAX_POLYGON_VERTICES {
        for d in ph2d_field::dims(&poligono(n)) {
            assert_ne!(
                ph2d_i18n::tr(d.key),
                d.key,
                "a linha `{}` de um polígono de {n} vértices não tem rótulo — o painel pintaria o \
                 identificador cru, e o `tr` vaza uma string por quadro a fazê-lo",
                d.key
            );
        }
    }
}
