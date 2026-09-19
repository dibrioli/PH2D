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
/// ⭐⭐⭐ **E desde 14/09 `tudo_aceso` NÃO muda a contagem** — ele mudou de papel: era o *pior caso*,
/// e é hoje o **controlo** da lei que o dono mandou (*«não devem desaparecer, mas apenas serem
/// inativados, mas sempre visíveis»*). O que o estado da peça muda é o `live` de cada linha, e a
/// altura do painel deixou de depender dele.
///
/// ⚠️ **Ele acende pelos PESOS** (`12` e `19`), que são os únicos que decidem o `live` — *uma lista
/// de «o que acender» escrita à mão é a segunda resposta à pergunta que o `params_of` já responde*,
/// e ela já mordeu nesta wave.
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
        // O verniz (`12`) e o brilho (`19`) — os dois pesos que abrem sub-linhas.
        //
        // ⛔⛔ **Eram `5` e `9` até 14/09, e a re-numeração para a ordem da nodedef transformou-os
        // em `metalness` e num canal da cor do realce** — isto é, a sonda passaria a pôr o METAL a
        // `1`, que **esconde** duas linhas em vez de abrir quatro. *Uma lista de índices escrita à
        // mão sobrevive a uma re-numeração sem erro de compilação, e mede o contrário do que diz.*
        // O que a apanhou foi o gate do lado de lá (`extras`), que leu `18` onde espera `25`.
        for peso in [12u8, 19] {
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

/// ⭐⭐⭐ **QUANTAS FILEIRAS A CENA APENDE AO RETRATO** — contadas no PRODUTOR delas.
///
/// ⛔⛔⛔ **Era esta a metade que faltava, e ela custava a secção inteira** (auditoria de 2026-09-19,
/// `docs/Render3d/11` §10.9): o `publish_snapshot` apende as fileiras do ESTILO **no fim** da lista
/// que o [`param_rows`] devolve, e os gates deste ficheiro chamavam o `param_rows` **directamente**.
/// *Um gate que mede o produtor de metade da lista é cego à outra metade* — e a outra metade era a
/// que caía fora do teto.
///
/// ⚠️ **Derivada, NUNCA um `10` escrito aqui:** a camada de estilo cresce dentro da própria jornada
/// em que este gate nasceu (a suavidade da curvatura, a nitidez separada de aresta e de cova). Um
/// literal reprovaria sobre produto correcto na hora seguinte.
///
/// ⚠️ **No modo Render, que é o pior caso**: fora dele a `rows` devolve vazio por lei (*uma
/// affordance que não pode ser honrada é pior do que nenhuma*), e um teto medido no caso vazio não
/// mede nada.
fn fileiras_da_cena() -> usize {
    crate::estilo::rows(ph2d_style::Style::default(), true).len()
}

/// ⭐⭐⭐ **AS FILEIRAS DA CENA CABEM NA FOLGA QUE O PAINEL LHES GUARDA.**
///
/// ⚠️ **O piso de população é metade da lei:** se a `rows` passar a devolver zero (um `render` mal
/// lido, um corte), este gate ficaria verde a medir o nada — e o painel voltaria a ter uma secção
/// invisível sem ninguém saber.
#[test]
fn as_fileiras_da_cena_cabem_na_folga_delas() {
    let cena = fileiras_da_cena();
    let folga = ph2d_panel_model3d::MAX_SCENE_ROWS;
    println!("  fileiras da cena: {cena} · folga do painel: {folga}");
    assert!(
        cena >= 8,
        "a camada de estilo publica {cena} fileiras — o censo deixou de achar o que mede, e um zero \
         aqui lê-se exactamente como «cabe»"
    );
    assert!(
        cena <= folga,
        "a cena apende {cena} fileiras e o painel guarda folga para {folga} — as últimas ficam sem \
         controlo, e com o polígono no teto a secção INTEIRA desaparece (o rodapé diz `(+N)`, que o \
         artista lê como «faltam números do meu nó»). A cura é subir o `MAX_SCENE_ROWS`, com o \
         preço medido ao lado."
    );
    // ⛔ **E fora do Render ela é VAZIA, por lei** — sem este controlo o gate acima passaria com uma
    // `rows` que devolvesse sempre vazio.
    assert_eq!(
        crate::estilo::rows(ph2d_style::Style::default(), false).len(),
        0,
        "a camada de estilo passou a publicar fileiras fora do modo Render"
    );
}

/// ⭐⭐⭐ **O RETRATO INTEIRO — nó NO TETO **mais** a cena — CABE NO PAINEL.**
///
/// ⚠️ **É a soma que importa, e era ela que ninguém media.** Medido em 2026-09-19 com o teto a valer
/// exactamente o pior nó: `85 + 10 = 95` de `85` ⇒ **`0` de `10`** fileiras de estilo visíveis.
#[test]
fn o_retrato_inteiro_cabe_na_familia_registada() {
    let nó = linhas_do_painel(poligono(MAX_POLYGON_VERTICES), true);
    let cena = fileiras_da_cena();
    let teto = ph2d_panel_model3d::MAX_ROWS;
    println!(
        "  nó no teto: {nó} + cena: {cena} = {} de {teto}",
        nó + cena
    );
    assert!(
        nó + cena <= teto,
        "o retrato pede {} fileiras ({nó} do nó + {cena} da cena) e a família tem {teto} — o \
         `paint` corta em silêncio, e o que cai fora é a secção que vem POR ÚLTIMO",
        nó + cena
    );
}

/// ⭐⭐⭐ **O POLÍGONO NO TETO AINDA CABE NO PAINEL** — e a folga que sobra é impressa.
///
/// ⚠️ **A barra é o `MAX_ROWS_DE_UM_NO` da crate do painel, lido dela** — não um `64` escrito aqui.
/// *Um teto copiado é um teto que envelhece na wave em que o outro mudar.*
///
/// ⛔ **E é o teto do NÓ e não o `MAX_ROWS`**: desde 2026-09-19 a família registada tem uma folga
/// para as fileiras que a CENA apende (ver [`as_fileiras_da_cena_cabem_na_folga_delas`]), e medir o
/// nó contra o teto inteiro deixaria essa folga ser gasta por um polígono.
#[test]
fn every_row_of_the_biggest_polygon_fits_the_registered_family() {
    let teto = ph2d_panel_model3d::MAX_ROWS_DE_UM_NO;
    println!("  vértices | tudo apagado | tudo aceso | de {teto}");
    for n in [MIN_POLYGON_VERTICES, 8, 16, MAX_POLYGON_VERTICES] {
        println!(
            "{n:>10} | {:>12} | {:>10} |",
            linhas_do_painel(poligono(n), false),
            linhas_do_painel(poligono(n), true)
        );
    }
    // ⭐⭐⭐ **E AS DUAS COLUNAS SÃO IGUAIS DESDE 14/09** — é assim que se lê, aqui, a ordem do dono
    // (*«não devem desaparecer, mas apenas serem inativados, mas sempre visíveis»*): a **contagem de
    // linhas de um nó deixou de depender do estado da peça**, e o painel não muda de altura quando
    // o artista acende o verniz.
    //
    // ⚠️ **Sem esta asserção o parâmetro `tudo_aceso` desta sonda seria um knob morto** — ele
    // continuaria a existir e a não mudar nada, que é exactamente o defeito que este módulo caça nos
    // painéis. Aqui ele passa a ser o **controlo** de uma lei.
    for n in [MIN_POLYGON_VERTICES, MAX_POLYGON_VERTICES] {
        assert_eq!(
            linhas_do_painel(poligono(n), false),
            linhas_do_painel(poligono(n), true),
            "a contagem de linhas de um nó de {n} vértices MUDOU com o estado do material — o painel \
             volta a saltar de tamanho debaixo do dedo"
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
    let teto = ph2d_panel_model3d::MAX_ROWS_DE_UM_NO;
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
    // | `20` | o verniz: `+1` sempre (o peso) e `+4` com ele aceso (rugosidade, cor, IOR,
    //   escurecimento) |
    // | **`25`** | as últimas cinco entradas do OpenPBR: peso da base, rugosidade da difusa, peso e
    //   cor do realce, e o IOR — ⭐ e o material FECHOU, logo este número deixa de crescer por aí |
    //
    // ⛔ *Um número que não se mexe enquanto a grandeza muda é a forma mais silenciosa de um gate
    // deixar de descrever o que mede* — o que o prende é esta tabela, não o literal.
    // ⭐ **A barra é o `EXTRAS_DE_UM_NO` da crate do painel, lido dela** — desde 2026-09-19 o teto é
    // DERIVADO desse número, e um literal aqui seria a segunda cópia que diverge.
    assert_eq!(
        extras,
        ph2d_panel_model3d::EXTRAS_DE_UM_NO,
        "um nó deixou de ter {} linhas além dos `2N` dos vértices — a conta do teto muda com isto",
        ph2d_panel_model3d::EXTRAS_DE_UM_NO
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
