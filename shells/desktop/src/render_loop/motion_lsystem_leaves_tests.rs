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

/// ⛔⛔ **O nome posto numa letra que a gramática não emite tem de ser DITO.**
///
/// Report do Enio (2026-08-30): *"só apareceu em seu exemplo, ao trocar o tipo de árvore não
/// aparece mais"*. Os moldes de planta trazem `J`, mas **uma gramática escrita à mão pode não
/// trazer letra nenhuma** — e aí o campo fica cheio, nada nasce, e o artista não tem como saber
/// porquê. *Um controlo com valor lá dentro e efeito nenhum parece ligado: é a pior espécie de
/// morto.*
///
/// ⚠️ **A metade que se gateia é a DECISÃO, não o canal** — o aviso sai no `stderr`, que um teste
/// não lê; por isso a lei vive numa função pura ([`crate::render_loop::motion_lsystem_leaves::unanswered_slots`]) e é ela que se mede.
#[test]
fn a_letter_with_a_name_and_no_anchor_is_reported() {
    let anchor = |slot: usize| crate::render_loop::motion_lsystem_leaves::Anchor {
        p: [0.0, 0.0],
        rot: 0.0,
        grow: 1.0,
        slot,
    };
    let names = |a: &str, b: &str, c: &str| [a.to_string(), b.to_string(), c.to_string()];

    // Nome posto, letra ausente da gramática ⇒ acusa.
    assert_eq!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(&names("folha", "", ""), &[]),
        vec![0],
        "um nome sem ancora nenhuma tem de ser acusado"
    );
    // A letra existe ⇒ cala.
    assert!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(
            &names("folha", "", ""),
            &[anchor(0)]
        )
        .is_empty(),
        "com a ancora la' o aviso seria ruido"
    );
    // ⚠️ **Por SLOT, nunca «há âncoras?»** — uma gramática com `J` e um nome em `K` é exactamente
    // o caso do report, e uma régua que só perguntasse «esta planta tem âncoras?» ficaria muda.
    assert_eq!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(
            &names("folha", "flor", ""),
            &[anchor(0)]
        ),
        vec![1],
        "o slot que tem ancora cala e o que nao tem acusa"
    );
    // Campo vazio nunca acusa: não pedir objecto nenhum é o estado normal.
    assert!(
        crate::render_loop::motion_lsystem_leaves::unanswered_slots(&names("", "", ""), &[])
            .is_empty()
    );
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

/// ⛔⛔ **E DUAS FOLHAS NÃO SE EMPILHAM** — *"elas aparecem em cada segmento"*, a metade que era
/// do MOLDE e não da lei: `62` marcas em `30` sítios, folhas idênticas uma sobre a outra.
///
/// ⚠️ **Aqui a régua tem de ser a POSIÇÃO DA INSTÂNCIA**, não a do esqueleto: é o que o artista
/// vê, e é o que sobrevive a qualquer mudança de como a membrana escolhe as âncoras.
#[test]
fn no_two_leaves_are_drawn_on_top_of_each_other() {
    let (mut state, n) = factory_plant_with_leaf(5.0, false);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let inst = instances_of(&state, &key);
    let mut sitios: Vec<(i64, i64)> = inst
        .iter()
        .map(|i| ((i.world_pos[0] * 1e4) as i64, (i.world_pos[1] * 1e4) as i64))
        .collect();
    let total = sitios.len();
    assert!(total > 8, "so' {total} folhas");
    sitios.sort_unstable();
    sitios.dedup();
    assert_eq!(
        total,
        sitios.len(),
        "{total} folhas em {} sitios — elas empilham",
        sitios.len()
    );
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
