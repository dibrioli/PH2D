//! Gates das **TRÊS LETRAS** — onde um objecto nasce, virado para onde, de que tamanho, e com
//! que alfa.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte segue
//! o do produto: o irmão [`super::motion_lsystem_gen`] responde *como um ramo vira forma*.
//!
//! ⛔⛔ **Vários destes gates atravessam até à INSTÂNCIA** (`instances_of`), e não é zelo: a
//! folha publicava a rotação numa coluna chamada `rotation` e a convenção do Motion chama-lhe
//! `rot`. *Um nome de coluna errado não dá erro — a coluna é ignorada e o default desenha.*
//! Todo gate que lesse a coluna publicada passava com o defeito lá dentro, e passou.

use crate::render_loop::motion_lsystem_gen::publish;
use crate::render_loop::motion_lsystem_testkit::*;
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

/// ⭐⭐⭐ **A LETRA PLANTA O OBJECTO** — o report do Enio de 2026-08-29 (*"deveríamos ter um modo
/// de escolher o objeto que será exposto em cada fase"*).
///
/// ⚠️ A afirmação é a do PRODUTO: as linhas publicadas têm de trazer a **textura daquele
/// objecto**, e uma por âncora. Um gate que só contasse linhas passaria com folhas invisíveis.
#[test]
fn a_named_letter_plants_that_objects_appearance_at_every_anchor() {
    let (mut state, n) = plant_with_leaves(["folha", "", ""]);
    publish_object(&mut state, "folha", 7);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);

    let tex = column_v1(&state, &key, "texture_id");
    let geom = column_v1(&state, &key, "geometry_id");
    // A linha 0 é a PLANTA (geometria vectorial); as outras são as folhas.
    assert!(geom[0] > 0.0, "a linha 0 tem de ser a planta");
    let leaves = tex
        .iter()
        .skip(1)
        .filter(|t| (**t - 7.0).abs() < 0.5)
        .count();
    assert!(
        leaves > 0,
        "nenhuma folha plantada — texturas publicadas: {tex:?}"
    );
    assert!(
        geom.iter().skip(1).all(|g| *g == 0.0),
        "uma folha não pode levar geometria vectorial: {geom:?}"
    );
}

/// ⭐⭐ **AS TRÊS LETRAS SÃO TRÊS SLOTS, e cada uma planta o SEU objecto.**
///
/// ⚠️ **É o gate que apanha a ordem trocada.** `LEAF_PARAMS` e `LEAF_SYMBOLS` são duas listas
/// emparelhadas por índice; trocar a ordem numa só faria a flor nascer onde o artista pediu
/// folha — e a contagem total ficaria igual, então só medir "há folhas" não veria nada.
#[test]
fn each_of_the_three_letters_plants_its_own_object() {
    let (mut state, n) = plant_with_leaves(["j_obj", "k_obj", "m_obj"]);
    for (name, tid) in [("j_obj", 11u32), ("k_obj", 22), ("m_obj", 33)] {
        publish_object(&mut state, name, tid);
    }
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let tex = column_v1(&state, &key, "texture_id");
    // A gramática pousa `JKM` em cada sítio, nessa ordem, então as folhas publicadas têm de ser
    // a repetição de `[11, 22, 33]`.
    //
    // ⛔⛔ **A 1.ª redacção deste gate perguntava se as três texturas APARECEM, e a mutação que
    // troca `J` com `K` SOBREVIVEU** — trocadas, as três continuam a aparecer. *Um teste de
    // PERTENÇA não vê uma permutação; o que a vê é a SEQUÊNCIA.* E o doc dele dizia, em voz
    // alta, que apanhava a ordem trocada.
    let leaves: Vec<f32> = tex.iter().skip(1).copied().collect();
    assert!(leaves.len() >= 6, "poucas folhas: {leaves:?}");
    assert_eq!(
        leaves.len() % 3,
        0,
        "cada sítio pousa as três letras: {leaves:?}"
    );
    for (i, t) in leaves.iter().enumerate() {
        let want = [11.0, 22.0, 33.0][i % 3];
        assert!(
            (t - want).abs() < 0.5,
            "a folha {i} devia ser a textura {want} e é {t} — as letras e os params estão \
             emparelhados pelo ÍNDICE, e a ordem trocou: {leaves:?}"
        );
    }
}

/// ⭐ **Uma letra SEM nome não planta nada** — e um nome que ninguém publicou também não.
///
/// ⚠️ *Não adivinha e não falha*: um nome pode ser escrito antes de a forma existir, e o quadro
/// seguinte tenta de novo. O que não pode é nascer um quad branco no sítio da folha.
#[test]
fn an_unnamed_or_unpublished_letter_plants_nothing() {
    for names in [["", "", ""], ["nao_existe", "", ""]] {
        let (mut state, n) = plant_with_leaves(names);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let geom = column_v1(&state, &key, "geometry_id");
        assert_eq!(
            geom.len(),
            1,
            "só a planta devia estar publicada, e vieram {} linhas ({names:?})",
            geom.len()
        );
    }
}

/// ⭐⭐⭐ **A FOLHA APONTA PARA ONDE O RAMO APONTA** — report do Enio (2026-08-30): *"sem
/// rotação [relativa] ao galho"*.
///
/// ⚠️ **A afirmação é sobre a INSTÂNCIA**, não sobre a coluna: a `basis` que o desenho recebe
/// tem de ser uma rotação de verdade e tem de DIFERIR entre folhas de ramos que apontam para
/// lados diferentes. Um gate que lesse a coluna passaria com o nome errado, que foi o defeito.
#[test]
fn every_leaf_faces_the_way_its_branch_faces() {
    let (mut state, n) = factory_plant_with_leaf(5.0, false);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let inst = instances_of(&state, &key);
    assert!(inst.len() > 8, "so' {} instancias", inst.len());
    // A `basis` é `[cos, sin, -sin, cos]`; o ângulo é o `atan2` da 1.ª coluna.
    let mut angs: Vec<i64> = inst
        .iter()
        .map(|i| (i.basis[1].atan2(i.basis[0]).to_degrees().round()) as i64)
        .collect();
    let turned = angs.iter().filter(|a| **a != 0).count();
    assert!(
        turned * 4 >= inst.len(),
        "quase nenhuma folha esta' rodada ({turned} de {}) — a coluna voltou a ter o nome que \
         ninguem le'",
        inst.len()
    );
    angs.sort_unstable();
    angs.dedup();
    assert!(
        angs.len() >= 3,
        "as folhas todas com o mesmo angulo ({angs:?}) — elas nao seguem o ramo"
    );
    // ⚠️ **E a `basis` é uma ROTAÇÃO**, não lixo: `cos² + sin² = 1`. Um valor em radianos
    // metido numa conversão de graus continuaria a normalizar — o que o apanha é o gate de
    // ACORDO abaixo, contra o esqueleto.
    for i in &inst {
        let d = i.basis[0] * i.basis[0] + i.basis[1] * i.basis[1];
        assert!(
            (d - 1.0).abs() < 1e-4,
            "basis nao normalizada: {:?}",
            i.basis
        );
    }
}

/// ⚠️ **E o ângulo é O DO RAMO, em GRAUS** — o acordo entre as duas rotas.
///
/// O modo `Segments` publica o esqueleto cru, e a coluna `rot` dele é o que o desenho lê. O
/// modo `Branches` tem de entregar **o mesmo ângulo na mesma marca**: se uma rota usasse
/// `wrot` e a outra `rot`, o param `Orient` do artista valeria numa e não na outra.
///
/// ⚠️ **Ele compara só as marcas ABERTAS, e a assimetria é deliberada:** o `Segments` publica o
/// esqueleto **CRU** — todas as marcas acumuladas, sem aplicar o `mark_grow` —, porque aquele
/// modo é o contrato do `rig.*` e encolher a `size` de uma marca ali mudaria o que um
/// consumidor de rig lê. O peso viaja como COLUNA, então quem quiser filtrar tem o
/// `motion.cull`/`field.index_range` à mão. Este gate é sobre o **ÂNGULO**, não sobre quantas
/// se vêem.
#[test]
fn the_two_geometry_modes_turn_a_leaf_the_same_way() {
    // ⚠️ **A régua NORMALIZA, e a 1.ª redacção não o fazia:** a instância guarda a rotação
    // como `basis`, e recuperá-la com `atan2` devolve `(−180, 180]`, enquanto o esqueleto
    // acumula o `heading` livremente — a mesma folha lia-se `190` de um lado e `−170` do
    // outro. *Duas leituras iguais que a régua chama de diferentes são um defeito da régua.*
    let turn = |a: f64| -> i64 { (a.rem_euclid(360.0) + 0.5).floor() as i64 % 360 };
    let mut segs: Vec<i64> = Vec::new();
    let mut brs: Vec<i64> = Vec::new();
    for (mode, out) in [
        (ls::GEOMETRY_SEGMENTS, &mut segs),
        (ls::GEOMETRY_BRANCHES, &mut brs),
    ] {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state
            .doc
            .graph
            .set_param(n, ls::param::GEOMETRY, mode as f32);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        if mode == ls::GEOMETRY_BRANCHES {
            *out = instances_of(&state, &key)
                .iter()
                .map(|i| turn(f64::from(i.basis[1].atan2(i.basis[0]).to_degrees())))
                .collect();
        } else {
            // Em `Segments` o esqueleto é a corrente do nó — o MESMO `build` que o modo
            // `Branches` consome, pedido pela porta pública de sonda do nó.
            let p = &ls::PRESETS[0];
            let s = ls::probe_build(
                p.axiom,
                p.rules,
                5.0,
                &[(ls::param::ANGLE, p.angle), (ls::param::STEP, p.step)],
            );
            let (sym, rot) = (
                match s.get("sym") {
                    Some(Column::Scalar(v)) => v.clone(),
                    _ => vec![],
                },
                match s.get("rot") {
                    Some(Column::Scalar(v)) => v.clone(),
                    _ => vec![],
                },
            );
            let grow = match s.get("mark_grow") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => vec![],
            };
            for i in 0..sym.len() {
                if sym[i] as i32 as u8 == b'J' && grow[i] > 1.0 / 256.0 {
                    out.push(turn(f64::from(rot[i])));
                }
            }
        }
    }
    segs.sort_unstable();
    brs.sort_unstable();
    assert!(!segs.is_empty(), "o esqueleto nao trouxe marca nenhuma");
    assert_eq!(
        segs, brs,
        "as duas rotas viram a folha de maneiras diferentes"
    );
}

/// ⭐⭐⭐ **A FOLHA NASCE NA PONTA, CRESCE, E FICA** — as duas queixas de 2026-08-30 sobre o
/// tamanho, e a correcção que o smoke seguinte impôs.
///
/// A 1.ª lei era um cruza-fade e Enio matou-a em duas palavras: *"a cada segmento a folha
/// cresce e diminui. bem bizarro"*. ⇒ a lei é a IDADE, **monótona**: a colheita nova abre com
/// a fracção da geração, e toda a mais velha fica cheia.
///
/// ⚠️ **As três afirmações são independentes e todas necessárias:**
/// 1. numa geração INTEIRA nenhuma folha está a meio (uma planta parada não tem folhas
///    encolhidas nem a crescer);
/// 2. a meio de uma geração **só a colheita nova** está a abrir;
/// 3. o total **nunca encolhe** com o crescimento — é isto que separa esta lei da anterior.
#[test]
fn a_leaf_is_born_at_the_tip_and_grows_into_it() {
    let leaf_sizes = |gens: f32| -> Vec<f32> {
        let (mut state, n) = factory_plant_with_leaf(gens, false);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let mut v: Vec<f32> = instances_of(&state, &key)
            .iter()
            .map(|i| i.size[0])
            .collect();
        v.sort_by(f32::total_cmp);
        v
    };
    // O objecto publicado mede `2.0` de largura (ver `publish_object`).
    const FULL: f32 = 2.0;
    // 1. Geração INTEIRA: toda folha está madura.
    for g in [4.0f32, 5.0] {
        let v = leaf_sizes(g);
        assert!(!v.is_empty(), "g={g}: nenhuma folha");
        for s in &v {
            assert!(
                (s - FULL).abs() < 1e-4,
                "g={g}: numa planta parada uma folha esta' a {s} e nao a {FULL}"
            );
        }
    }
    // 2. A MEIO: as velhas cheias, e a colheita nova a meio — e as duas populações existem.
    let v = leaf_sizes(4.5);
    let novas = v.iter().filter(|s| (**s - FULL * 0.5).abs() < 1e-3).count();
    let velhas = v.iter().filter(|s| (**s - FULL).abs() < 1e-4).count();
    assert_eq!(
        novas + velhas,
        v.len(),
        "a meio da geracao so' ha' duas alturas de folha: {v:?}"
    );
    assert!(
        novas > 0 && velhas > 0,
        "a meio tem de haver folhas novas A ABRIR e velhas JA' CHEIAS: {novas}/{velhas}"
    );
    // 3. ⛔⛔ **NUNCA ENCOLHE** — a afirmação que a lei anterior violava a cada geração.
    let mut anterior = 0.0f32;
    let mut g = 1.0f32;
    while g <= 5.0 {
        let total: f32 = leaf_sizes(g).iter().sum();
        assert!(
            total >= anterior - 1e-3,
            "a folhagem ENCOLHEU a g={g}: {anterior} -> {total}"
        );
        anterior = total;
        g += 0.25;
    }
    assert!(anterior > 0.0, "a varredura nunca chegou a ver uma folha");
}

/// ⭐⭐ **A BANDEIRA DE ALFA DA FONTE CHEGA À INSTÂNCIA** — report do Enio (2026-08-30): *"o
/// Alpha usado escurece as bordas da pintura (diferente da sprite)"*.
///
/// O lowering do Motion cravava `premultiplied: 0.0`, ou seja *«esta textura é alfa direta»*,
/// para TODA instância que ele emite — e um documento pintado sobe **já premultiplicado**. O
/// fragmento pré-multiplicava outra vez ⇒ `RGB·α²` ⇒ borda escura.
///
/// ⚠️ **E o controlo é a outra metade:** uma fonte de alfa directa tem de continuar a dar
/// `0`, senão a cura seria só o defeito ao contrário.
#[test]
fn a_premultiplied_source_says_so_all_the_way_to_the_instance() {
    for (premultiplied, want) in [(true, 1.0f32), (false, 0.0)] {
        let (mut state, n) = factory_plant_with_leaf(5.0, premultiplied);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let inst = instances_of(&state, &key);
        assert!(!inst.is_empty(), "sem folhas nao ha' o que medir");
        for i in &inst {
            assert_eq!(
                i.premultiplied, want,
                "fonte premultiplicada={premultiplied} chegou como {}",
                i.premultiplied
            );
        }
    }
}

/// ⚠️ **O param `Orient` do artista ALCANÇA a folha** — e é ele que torna observável a escolha
/// entre as duas colunas de ângulo do esqueleto.
///
/// ⛔ **Sem este gate a escolha é invisível:** no default (`Growth` = mundo) a marca leva
/// `rot == wrot`, então ler a coluna errada dá exactamente o mesmo número e a mutação
/// SOBREVIVE. É em `Local` que elas divergem — ali uma marca não virou nada em relação ao pai,
/// logo `rot == 0` — e é essa a resposta que o artista pediu ao escolher `Local`.
#[test]
fn the_orient_param_reaches_the_leaf() {
    let angles = |orient: f32| -> Vec<i64> {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state.doc.graph.set_param(n, ls::param::ORIENT, orient);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let mut v: Vec<i64> = instances_of(&state, &key)
            .iter()
            .map(|i| (f64::from(i.basis[1].atan2(i.basis[0]).to_degrees())).round() as i64)
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    // `Growth` (o default) = o ângulo do ramo no mundo ⇒ várias direcções.
    let world = angles(0.0);
    assert!(
        world.len() >= 3,
        "em Growth as folhas tem de seguir o ramo: {world:?}"
    );
    // `Local` = quanto a marca virou em relação ao pai ⇒ nada.
    let local = angles(1.0);
    assert_eq!(
        local,
        vec![0],
        "em Local a folha nao vira nada em relacao ao ramo"
    );
}

/// ⭐⭐⭐ **A FOLHA FORA DO TINT — E SÓ DO TINT.**
///
/// Report do Enio (2026-08-30): *"uma opção para livrar as folhas, os frutos do tint que pinta
/// tudo na árvore"*. ⛔⛔ **A 1.ª cura escreveu `falloff` e partiu a planta** — o report seguinte
/// dele foi *"Keep own color não funciona, as folhas não aparecem"*: o `falloff` é a máscara de
/// TODOS os modificadores, e o `motion.move` faz `P' = P + (dx, dy) · falloff`, então as folhas
/// ficavam paradas enquanto a árvore andava (a cena `=108` move cada coluna).
///
/// ⚠️⚠️ **E o gate que eu tinha NÃO O VIU, porque media a COLUNA e não a CONSEQUÊNCIA.** Este
/// mede as duas metades, e a segunda é a que faltava:
/// 1. a máscara nasce, e é a estreita (`tint_mask`), nunca o `falloff`;
/// 2. **um `motion.move` a jusante continua a levar as folhas com a planta.**
#[test]
fn the_leaves_keep_their_own_colour_and_still_travel_with_the_plant() {
    let masks = |effects: f32| -> (Vec<f32>, Vec<f32>) {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_EFFECTS, effects);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        (
            column_v1(&state, &key, ph2d_nodegraph::attr::TINT_MASK_COLUMN),
            column_v1(&state, &key, "falloff"),
        )
    };
    // 1. `0` = Keep Own Colour (o default): a planta a `1`, cada folha a `0`.
    let (mask, falloff) = masks(0.0);
    assert!(
        !mask.is_empty(),
        "a mascara tem de nascer no modo que a usa"
    );
    assert_eq!(
        mask.iter().filter(|f| **f == 1.0).count(),
        1,
        "so' a planta e' alcancada pela cor: {mask:?}"
    );
    assert!(
        mask.iter().filter(|f| **f == 0.0).count() > 8,
        "as folhas tem de sair fora do alcance da cor: {mask:?}"
    );
    // ⛔ **E NUNCA o `falloff`** — foi ele que parou as folhas.
    assert!(
        falloff.is_empty(),
        "a mascara larga voltou: o `motion.move` deixaria as folhas para tras"
    );
    // 2. `1` = Reached: a coluna NÃO nasce ⇒ ausente ⇒ `1` em toda a casa.
    assert!(
        masks(1.0).0.is_empty(),
        "com a cor a alcancar, a mascara nao pode nascer — ela apagaria uma de montante"
    );
    // ⭐⭐⭐ **A CONSEQUÊNCIA**: com a máscara posta, um `motion.move` a jusante move TUDO.
    let (mut state, n) = factory_plant_with_leaf(5.0, false);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let antes = match state
        .pump
        .cook
        .externals()
        .get(&key)
        .expect("publicada")
        .value
        .get("P")
    {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    };
    // ⚠️ **O nó REAL, cozido pelo grafo** — não uma cópia da lei dele nesta fixtura: uma
    // fixtura que reimplementa o que testa pode estar errada de maneiras que o produto não
    // está (aconteceu duas vezes nesta jornada).
    let mv = state.doc.graph.add_node("motion.move");
    state.doc.graph.set_param(mv, "dx", 5.0);
    state
        .doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (n, 0),
            to: (mv, 0),
            delayed: false,
        })
        .expect("liga a planta ao move");
    let cooked = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, mv, 0.0)
        .expect("o move cozinha");
    let depois = match cooked[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P do move"),
    };
    let (a, b) = (antes, depois);
    assert_eq!(a.len(), b.len());
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        assert!(
            (q[0] - p[0] - 5.0).abs() < 1e-4,
            "a linha {i} nao andou com a planta: {p:?} -> {q:?} — a mascara larga voltou"
        );
    }
    let _ = mv;
}

/// ⭐⭐⭐ **DUAS PLANTAS IGUAIS COM FOLHAS DIFERENTES NÃO PARTILHAM A CORRENTE** — achado §3.3 da
/// auditoria de seis lentes.
///
/// ⚠️ **A `ribbon_key` é endereçada pelo CONTEÚDO**, e a lista dele saía do `MANIFEST.params`,
/// que é **`f32`-only por contrato congelado** (§6). Os três nomes de objecto de folha são
/// canais de TEXTO: a shell lê-os para construir a corrente e publicava-a sob uma chave que os
/// ignorava ⇒ duas plantas com os mesmos números e a mesma gramática, uma com *Leaf (J) = folha*
/// e outra *= flor*, cunhavam a **mesma** chave e a segunda **sobrescrevia** a primeira.
///
/// ⚠️⚠️ **O doc-comment da própria função declarava a invariante que ela quebrava** — *«um param
/// novo entra na chave sozinho»*, verdade para o `f32` e falsa para o texto. ⇒ os dois lados
/// saem agora de listas (`MANIFEST.params` e `ls::TEXT_PARAMS`), e esta é a metade que o mede.
#[test]
fn two_identical_plants_with_different_leaves_do_not_share_the_stream() {
    use crate::render_loop::motion_lsystem_testkit::{key_of, plant_with_leaves};
    let (mut a, na) = plant_with_leaves(["folha", "", ""]);
    let (mut b, nb) = plant_with_leaves(["flor", "", ""]);
    assert_ne!(
        key_of(&mut a, na),
        key_of(&mut b, nb),
        "duas plantas so' diferentes no OBJECTO da folha partilharam a chave da corrente"
    );
}

/// ⚠️ **O CONTROLE do gate acima: a chave não pode passar a distinguir tudo.**
///
/// Uma cura que metesse, por exemplo, o id do nó na chave separaria estas duas — e mataria a
/// razão de a chave ser de CONTEÚDO, que é duas plantas iguais partilharem o trabalho em vez de
/// o fazerem duas vezes. Aqui a barra é que duas plantas **realmente iguais** continuem a
/// colidir.
#[test]
fn two_truly_identical_plants_still_share_it() {
    use crate::render_loop::motion_lsystem_testkit::{key_of, plant_with_leaves};
    let (mut a, na) = plant_with_leaves(["folha", "", ""]);
    let (mut b, nb) = plant_with_leaves(["folha", "", ""]);
    assert_eq!(
        key_of(&mut a, na),
        key_of(&mut b, nb),
        "duas plantas IGUAIS deixaram de partilhar a corrente — a chave deixou de ser de conteudo"
    );
}
