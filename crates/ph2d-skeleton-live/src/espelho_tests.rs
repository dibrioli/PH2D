//! Os gates do ESPELHO — a geometria, a involução, os nomes e o que não viaja.

use super::*;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_skeleton_ecs::register_skeleton_components;

/// O registo com os componentes do esqueleto — a cópia profunda não copia o que ele não conhece.
fn registo() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    register_skeleton_components(&mut reg);
    reg
}

/// Uma cadeia de três ossos a partir de `(x0, 0)`, cada um com `1` de comprimento e a dobrar.
///
/// ⚠️ **Ela dobra de propósito:** uma cadeia recta é simétrica, e um espelho sobre ela devolveria a
/// mesma coisa — *uma fixtura que não contém o fenómeno não prova que ele aconteceu*.
fn cadeia_dobrada(sim: &mut SimWorld) -> Entity {
    let raiz = Entity::from_bits(
        crate::bone::create(sim, None, [2.0, 0.0], [3.0, 0.0]).expect("a raiz nasce"),
    );
    let meio = Entity::from_bits(
        crate::bone::create(sim, Some(raiz), [3.0, 0.0], [3.7, 0.7]).expect("o meio nasce"),
    );
    let _ponta = crate::bone::create(sim, Some(meio), [3.7, 0.7], [4.0, 1.4]).expect("a ponta");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    raiz
}

/// Os segmentos `(cabeça, ponta)` de todos os ossos, em MUNDO, ordenados — a régua da geometria.
fn segmentos(sim: &SimWorld) -> Vec<([f64; 2], [f64; 2])> {
    let mut v: Vec<_> = crate::skin_live::bone_segments(sim)
        .into_iter()
        .map(|(_, a, b)| (a, b))
        .collect();
    v.sort_by(|x, y| x.partial_cmp(y).expect("sem NaN"));
    v
}

/// ⭐⭐⭐ **A PROMESSA: cada osso da cópia vai de `M(cabeça)` a `M(ponta)`** — e é isso que o artista
/// vê.
///
/// ⚠️ **Medida em MUNDO e não nos campos locais:** a lei é uma conjugação (`W' = M ∘ W ∘ G`) e o que
/// interessa é o resultado dela, não os três sinais que a implementam. *Uma régua sobre os campos
/// mediria a implementação e ficaria verde sobre uma cópia que aponta ao contrário.*
#[test]
fn cada_osso_espelhado_vai_da_cabeca_a_ponta_reflectidas() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    let eixo = eixo_do_esqueleto(&sim, raiz).expect("a raiz tem eixo");
    let antes = segmentos(&sim);
    let copia = espelha(&mut sim, &registo(), raiz).expect("o espelho corre");
    assert_ne!(copia, raiz, "o espelho devolveu o proprio original");

    let esperado: Vec<_> = antes
        .iter()
        .map(|(a, b)| (reflecte(*a, eixo), reflecte(*b, eixo)))
        .collect();
    let todos = segmentos(&sim);
    assert_eq!(
        todos.len(),
        6,
        "a cena tem de ter os tres ossos e as tres copias"
    );
    for (a, b) in esperado {
        let achou = todos.iter().any(|(x, y)| {
            (x[0] - a[0]).abs() < 1e-5
                && (x[1] - a[1]).abs() < 1e-5
                && (y[0] - b[0]).abs() < 1e-5
                && (y[1] - b[1]).abs() < 1e-5
        });
        assert!(
            achou,
            "nenhum osso da cena vai de {a:?} a {b:?} — a copia nao e' o espelho do original"
        );
    }
}

/// ⭐⭐⭐ **E O ORIGINAL NÃO SE MEXE** — a metade sem a qual a de cima passaria com um verbo que
/// **move** o ramo em vez de o copiar.
#[test]
fn o_original_fica_exactamente_onde_estava() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    let antes = segmentos(&sim);
    espelha(&mut sim, &registo(), raiz).expect("o espelho corre");
    let depois = segmentos(&sim);
    for (a, b) in &antes {
        assert!(
            depois.iter().any(|(x, y)| x == a && y == b),
            "o osso que ia de {a:?} a {b:?} deixou de existir: o espelho MOVEU o original"
        );
    }
}

/// ⭐⭐⭐ **ESPELHAR DUAS VEZES DEVOLVE A GEOMETRIA ORIGINAL** — a involução, que é a prova mais dura
/// de que a conjugação está certa.
///
/// ⚠️ **A barra é `f32`**: a pose vive num `Transform` cuja rotação e translação são `f32`, e a ida
/// e a volta passam por `atan2` — `1e-4` é folgado sobre o que se mede e apertado o bastante para
/// uma troca de sinal errada reprovar por três ordens de grandeza.
#[test]
fn espelhar_duas_vezes_devolve_a_geometria() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    let copia = espelha(&mut sim, &registo(), raiz).expect("o 1.º espelho");
    let volta = espelha(&mut sim, &registo(), copia).expect("o 2.º espelho");
    let ordem = ordem_da_arvore(&sim, volta);
    let originais = ordem_da_arvore(&sim, raiz);
    let segs = crate::skin_live::bone_segments(&sim);
    assert_eq!(
        ordem.len(),
        originais.len(),
        "a volta tem outra forma de arvore que o original"
    );
    for (k, e) in ordem.iter().enumerate() {
        let (_, a, b) = segs
            .iter()
            .copied()
            .find(|(x, _, _)| *x == e.to_bits())
            .expect("o osso da volta tem segmento");
        // A cadeia da volta compara-se com a do original, osso a osso na MESMA ordem de árvore.
        let original = originais[k];
        let (_, ra, rb) = segs
            .iter()
            .copied()
            .find(|(x, _, _)| *x == original.to_bits())
            .expect("o osso original tem segmento");
        for (t, o, nome) in [(a, ra, "cabeca"), (b, rb, "ponta")] {
            assert!(
                (t[0] - o[0]).abs() < 1e-4 && (t[1] - o[1]).abs() < 1e-4,
                "o {nome} do osso {k} voltou a {t:?} e o original esta' em {o:?} — a conjugacao nao \
                 e' uma involucao"
            );
        }
    }
}

/// ⭐⭐ **O LIMITE DE ÂNGULO TROCA E NEGA** — e só negar seria pior que não espelhar.
///
/// ⛔ Com `min > max` a lei trava a junta no **CENTRO** do que estiver escrito: o cotovelo espelhado
/// ficaria preso a meio caminho, e nada na tela o explicaria.
#[test]
fn o_limite_troca_de_ponta_e_nega() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    let meio = ordem_da_arvore(&sim, raiz)[1];
    sim.world_mut().entity_mut(meio).insert(BoneLimit {
        min: -0.25,
        max: 1.5,
    });
    let copia = espelha(&mut sim, &registo(), raiz).expect("o espelho");
    let meio_novo = ordem_da_arvore(&sim, copia)[1];
    let l = sim
        .world()
        .get::<BoneLimit>(meio_novo)
        .copied()
        .expect("a copia herdou o limite");
    assert!(
        (l.min + 1.5).abs() < 1e-9 && (l.max - 0.25).abs() < 1e-9,
        "o limite espelhado e' {l:?} e tinha de ser (-1,5 .. 0,25) — trocado E negado"
    );
    assert!(
        l.min < l.max,
        "min > max: a lei trava a junta no CENTRO da faixa"
    );
}

/// ⭐⭐ **A CURVATURA espelha no `y` e NÃO no `x`** — o arco é um desvio lateral; o `x` mede-se ao
/// longo do eixo, que o `G` fixa.
#[test]
fn a_curvatura_espelha_so_no_eixo_que_inverte() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    if let Some(mut b) = sim.world_mut().get_mut::<Bone>(raiz) {
        b.curve.inn = [0.3, 0.4];
        b.curve.out = [-0.2, 0.5];
    }
    let copia = espelha(&mut sim, &registo(), raiz).expect("o espelho");
    let b = sim.world().get::<Bone>(copia).copied().expect("a copia");
    assert!(
        (b.curve.inn[0] - 0.3).abs() < 1e-9 && (b.curve.out[0] + 0.2).abs() < 1e-9,
        "o `x` da curvatura mudou ({:?}): ele mede-se AO LONGO do eixo, que o espelho fixa",
        b.curve
    );
    assert!(
        (b.curve.inn[1] + 0.4).abs() < 1e-9 && (b.curve.out[1] + 0.5).abs() < 1e-9,
        "o `y` da curvatura nao inverteu ({:?}): o arco espelhado sai para o mesmo lado",
        b.curve
    );
}

/// ⛔⛔ **A ÂNCORA E O OSSO INTELIGENTE NÃO VIAJAM** — os dois nomeiam OUTROS objectos da cena.
///
/// Copiá-los daria duas correntes a puxar o **mesmo losango**: o braço espelhado seguiria a mão do
/// original, e nada na tela o explicaria.
#[test]
fn a_ancora_e_o_osso_inteligente_ficam_para_tras() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    let ponta = *ordem_da_arvore(&sim, raiz).last().expect("a ponta");
    sim.world_mut()
        .entity_mut(ponta)
        .insert(ph2d_skeleton_ecs::IkGoal::default());
    sim.world_mut()
        .entity_mut(raiz)
        .insert(ph2d_skeleton_ecs::SmartBone::default());
    let copia = espelha(&mut sim, &registo(), raiz).expect("o espelho");
    let novos = ordem_da_arvore(&sim, copia);
    for e in &novos {
        assert!(
            sim.world().get::<ph2d_skeleton_ecs::IkGoal>(*e).is_none(),
            "a copia trouxe a ancora: duas correntes a puxar o mesmo losango"
        );
        assert!(
            sim.world()
                .get::<ph2d_skeleton_ecs::SmartBone>(*e)
                .is_none(),
            "a copia trouxe o osso inteligente: dois controlos a percorrer a mesma accao"
        );
    }
    // ⚠️ **O CONTROLO**: o original continua com os dois — o espelho tira-os da CÓPIA, não do rig.
    assert!(
        sim.world()
            .get::<ph2d_skeleton_ecs::IkGoal>(ponta)
            .is_some()
            && sim
                .world()
                .get::<ph2d_skeleton_ecs::SmartBone>(raiz)
                .is_some(),
        "o espelho tirou a ancora ou o controlo do ORIGINAL"
    );
}

/// ⭐⭐⭐ **O NOME TROCA DE LADO** — a tabela, e o que ela recusa.
///
/// ⚠️ **Um `replace` cego de `L` por `R` renomearia `"Leg"` para `"Reg"`.** O que se troca é um
/// MARCADOR: um sufixo, ou uma palavra inteira.
#[test]
fn o_nome_troca_de_lado_sem_estragar_a_palavra() {
    for (de, para) in [
        ("Arm.L", "Arm.R"),
        ("Arm.R", "Arm.L"),
        ("hand_l", "hand_r"),
        ("Foot_Left", "Foot_Right"),
        ("Left Arm", "Right Arm"),
        ("upper right leg", "upper left leg"),
    ] {
        assert_eq!(
            nome_espelhado(de).as_deref(),
            Some(para),
            "o nome `{de}` devia espelhar para `{para}`"
        );
    }
    for sem_lado in ["Leg", "Bone 7", "Relax", "Slide", ""] {
        assert_eq!(
            nome_espelhado(sem_lado),
            None,
            "`{sem_lado}` nao declara lado nenhum, e inventar-lhe um seria escrever uma decisao do \
             artista"
        );
    }
}

/// ⭐⭐ **E o nome da cópia é ÚNICO, sempre** — a referência durável entre objectos nesta casa é o
/// NOME, e dois ossos com o mesmo seriam o mesmo sujeito para a timeline.
#[test]
fn nenhum_osso_da_cena_partilha_o_nome_com_outro() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    // Nomes que JÁ declaram lado, e com o outro lado JÁ ocupado: o pior caso para a unicidade.
    for (k, e) in ordem_da_arvore(&sim, raiz).into_iter().enumerate() {
        sim.world_mut()
            .entity_mut(e)
            .insert(Name::new(format!("Arm{k}.L")));
    }
    espelha(&mut sim, &registo(), raiz).expect("o 1.º espelho");
    espelha(&mut sim, &registo(), raiz).expect("o 2.º espelho, sobre nomes ja' ocupados");
    let mut nomes: Vec<String> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<Bone>().is_some())
        .filter_map(|er| er.get::<Name>().map(|n| n.0.clone()))
        .collect();
    let total = nomes.len();
    nomes.sort();
    nomes.dedup();
    assert_eq!(
        nomes.len(),
        total,
        "dois ossos ficaram com o mesmo nome: a timeline passa a anima'-los como se fossem um"
    );
}

/// ⛔⛔⛔ **A PONTA DA CURVA APONTA PARA O FILHO DA CÓPIA, e não para o do original.**
///
/// O `curve_tip` nomeia um filho por **identidade**, e a cópia profunda **não remapeia nada** (o doc
/// dela di-lo por escrito). Sem a cura, o ramo espelhado arquearia a seguir a um osso do outro lado
/// do corpo — em silêncio, porque a referência resolve e aponta para algo que existe.
///
/// ⚠️ **O CONTROLO é um id de FORA do ramo**, que tem de ficar como está: ali a referência do
/// original continua a ser a resposta certa, e apagá-la seria perder a escolha do artista.
#[test]
fn a_ponta_da_curva_segue_o_filho_da_copia() {
    let mut sim = SimWorld::default();
    let raiz = cadeia_dobrada(&mut sim);
    let [r, meio, _ponta] = [0, 1, 2].map(|i| ordem_da_arvore(&sim, raiz)[i]);
    let id_do_meio = ph2d_ecs::stable_id_of(sim.world(), meio).expect("o meio tem identidade");
    if let Some(mut b) = sim.world_mut().get_mut::<Bone>(r) {
        b.curve_tip = ph2d_skeleton_ecs::CurveTip::Bone(id_do_meio);
    }
    let copia = espelha(&mut sim, &registo(), raiz).expect("o espelho");
    let meio_novo = ordem_da_arvore(&sim, copia)[1];
    let id_novo = ph2d_ecs::stable_id_of(sim.world(), meio_novo).expect("a copia tem identidade");
    let b = sim.world().get::<Bone>(copia).copied().expect("a copia");
    assert_eq!(
        b.curve_tip,
        ph2d_skeleton_ecs::CurveTip::Bone(id_novo),
        "a ponta da curva da copia aponta para o filho do ORIGINAL: o ramo espelhado arqueia a \
         seguir a um osso do outro lado do corpo"
    );

    // ⚠️ **O CONTROLO**: um id de FORA do ramo fica intocado.
    let mut sim2 = SimWorld::default();
    let outro = cadeia_dobrada(&mut sim2);
    let raiz2 = Entity::from_bits(
        crate::bone::create(&mut sim2, None, [-5.0, 0.0], [-4.0, 0.0]).expect("outra raiz"),
    );
    ph2d_ecs::assign_missing_stable_ids(sim2.world_mut());
    let alheio = ph2d_ecs::stable_id_of(sim2.world(), outro).expect("id alheio");
    if let Some(mut b) = sim2.world_mut().get_mut::<Bone>(raiz2) {
        b.curve_tip = ph2d_skeleton_ecs::CurveTip::Bone(alheio);
    }
    let copia2 = espelha(&mut sim2, &registo(), raiz2).expect("o espelho");
    assert_eq!(
        sim2.world()
            .get::<Bone>(copia2)
            .map(|b| b.curve_tip)
            .expect("a copia"),
        ph2d_skeleton_ecs::CurveTip::Bone(alheio),
        "um id de FORA do ramo foi reescrito: a escolha do artista perdeu-se"
    );
}
