//! Os gates da tabela de materiais. Ver [`super`].

use super::{Table, surface_of};
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::FieldMaterial;

/// Duas esferas bem separadas, em união — cada uma tem uma região da tela que é só dela.
fn two_balls() -> FieldDoc {
    let leaf = |x: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.25 }),
        mods: Vec::new(),
        verb: None,
    };
    FieldDoc::new(
        vec![
            leaf(-0.4),
            leaf(0.4),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("duas esferas")
}

fn a_world() -> (ph2d_ecs::SimWorld, bevy_ecs::entity::Entity) {
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &two_balls(), "peça");
    (sim, root)
}

/// ⭐⭐⭐ **A MESMA peça, com os materiais DISTINTOS** — que é a condição da lei do dono desde
/// 2026-09-22.
///
/// ⛔⛔ **Ela existe porque uma premissa MORREU:** até esse dia o `Table::build` dava lei do dono a
/// **toda** peça com mais de uma folha, e dois gates deste ficheiro liam `t.owners` sobre a
/// [`a_world`], cujos materiais são os de omissão. Com a lei do dono a passar a pedir materiais
/// **distintos** (ela emite uma fita por folha no TEXTO do shader, e acrescentar uma forma passava
/// a custar `2 310 ms` em vez de `7,85`), aqueles dois gates ficaram a afirmar sobre uma peça que
/// já não a tem. *A cura é a fixtura mudar-se para onde o sujeito vive, nunca o gate afrouxar.*
fn a_world_de_dois_materiais() -> (ph2d_ecs::SimWorld, bevy_ecs::entity::Entity) {
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);
    // A da esquerda ganha um azul; a da direita fica no material de omissão.
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(3),
        0.0,
    )
    .expect("o azul");
    (sim, root)
}

fn leaves_of(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
) -> Vec<bevy_ecs::entity::Entity> {
    world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("a raiz tem filhos")
        .iter()
        .copied()
        .collect()
}

/// ⭐⭐⭐ **O DEFAULT do componente É o da nodedef** — e este gate é a ponte entre as duas crates.
///
/// ⛔⛔ **A `ph2d-field-ecs` NÃO depende da `ph2d-material`**, de propósito: o material é um
/// componente de cena e a lei do OpenPBR é uma crate sem dependências nenhumas. O preço dessa
/// fronteira é o `Default` estar escrito duas vezes — e é **este** gate que impede as duas cópias de
/// divergirem. *Uma constante escrita em dois sítios ainda não é uma constante.*
///
/// **Mutação que deve sangrar:** mexer num dos dois `Default`.
#[test]
fn the_default_material_is_the_one_the_nodedef_declares() {
    let nosso = surface_of(FieldMaterial::default());
    let nodedef = ph2d_material::OpenPbr::default().prepare();
    // A superfície não é comparável campo a campo de fora, então compara-se o que ela FAZ: a luz
    // que devolve. ⚠️ Três normais, para um acerto por acaso numa delas não passar.
    for n in [[0.0, 0.0, 1.0], [0.6, 0.0, 0.8], [0.0, 0.8, 0.6]] {
        let v = [0.0, 0.0, 1.0];
        let l = [0.3, 0.6, 0.74];
        assert_eq!(
            nosso.direct(n, v, l, [3.0; 3]),
            nodedef.direct(n, v, l, [3.0; 3]),
            "o `FieldMaterial::default()` deixou de ser o material da nodedef (normal {n:?})"
        );
    }
}

/// ⭐⭐⭐ **CADA FOLHA LEVA O SEU MATERIAL** — e a tabela sabe qual é qual.
///
/// ⚠️ **O ponto é posto sobre cada esfera**, que é como ele chega do traçado: no meio de uma união
/// qualquer das duas é plausível, e um gate que apontasse ali passaria com a resposta errada.
#[test]
fn each_leaf_wears_its_own_material() {
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);
    assert_eq!(folhas.len(), 2);
    // A da esquerda fica VERMELHA; a da direita continua no material de omissão.
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(2),
        0.0,
    )
    .expect("o verde");
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(3),
        0.0,
    )
    .expect("o azul");

    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert_eq!(t.surfaces.len(), 2, "uma superfície por folha");
    let owners = t.owners.as_ref().expect("duas folhas pedem um dono");
    // O pólo de cada esfera: só ela existe ali.
    assert_eq!(owners.at([-0.4, 0.0, 0.25]), Some(0), "a esquerda");
    assert_eq!(owners.at([0.4, 0.0, 0.25]), Some(1), "a direita");
    // E as duas superfícies devolvem luz DIFERENTE — é isso que o artista vê.
    let (n, v, l) = ([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74]);
    assert_ne!(
        t.surfaces[0].direct(n, v, l, [3.0; 3]),
        t.surfaces[1].direct(n, v, l, [3.0; 3]),
        "as duas folhas têm materiais diferentes e devolvem a MESMA luz — a tabela não os separou"
    );
}

/// ⭐⭐ **Arrastar um número NÃO recompila a geometria** — é a razão de a tabela ter duas metades.
///
/// **Mutação que deve sangrar:** fazer o `refresh_authored` devolver `false` sempre (o slider de cor
/// deixa de ter efeito), ou `true` sempre (o quadro re-traça para sempre).
#[test]
fn changing_a_number_refreshes_the_surfaces_and_not_the_geometry() {
    // ⚠️ A fixtura leva materiais DISTINTOS porque a metade de baixo lê `t.owners` para observar
    // que a geometria não foi tocada — ver [`a_world_de_dois_materiais`].
    let (mut sim, root) = a_world_de_dois_materiais();
    let folhas = leaves_of(sim.world(), root);
    let mut t = Table::build(sim.world(), root, 0.8, 480.0);
    let antes = t.surfaces[0].direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74], [3.0; 3]);

    assert!(
        !t.refresh_authored(sim.world(), root),
        "nada mudou e a tabela disse que sim — o quadro re-traçaria para sempre"
    );
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(10),
        0.9,
    )
    .expect("a rugosidade");
    assert!(
        t.refresh_authored(sim.world(), root),
        "a rugosidade mudou e a tabela não deu por isso — o slider fica sem efeito"
    );
    assert_ne!(
        t.surfaces[0].direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74], [3.0; 3]),
        antes,
        "a superfície não seguiu o número"
    );
    // ⭐ **E a geometria não foi tocada:** o dono continua a responder o mesmo.
    assert_eq!(
        t.owners.as_ref().expect("dois donos").at([-0.4, 0.0, 0.25]),
        Some(0)
    );
}

/// Uma peça de UMA folha não constrói dono nenhum — não perguntar é o custo zero.
#[test]
fn a_single_leaf_asks_nobody_who_it_belongs_to() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.3 },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("uma esfera");
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert!(
        t.owners.is_none(),
        "uma peça de uma folha construiu um resolvedor de donos — é trabalho por uma pergunta que \
         não existe"
    );
    assert_eq!(t.surfaces.len(), 1);
}

/// ⭐⭐⭐ **ARRASTAR UMA COR NÃO COMPILA FITA NENHUMA** — o gate sobre o TRABALHO, não sobre a
/// resposta.
///
/// # ⛔⛔ Porque o gate irmão não chega
///
/// O [`changing_a_number_refreshes_the_surfaces_and_not_the_geometry`] afirma que o dono **responde o
/// mesmo** depois de um número mudar. Isso é a **RESPOSTA**, e ela sai certa mesmo que alguém troque
/// o `refresh_authored` por um `Table::build` inteiro: a geometria é a mesma, logo o dono responde o
/// mesmo — *e o quadro paga um JIT por folha a cada pixel de arrasto do slider de cor*.
///
/// É a mesma lei que o [`ph2d_field_eval::owners::Owners::at_counting`] existe para servir, escrita
/// no doc dele: **um gate sobre a RESPOSTA é cego ao PREÇO**, e uma optimização cuja ausência não se
/// vê na saída precisa de um gate sobre o trabalho.
///
/// # A régua
///
/// [`ph2d_field_eval::POINT_TAPES`] — o gémeo, escrito nesta wave, do contador que a W70 construiu
/// para exactamente esta família. ⚠️ **A 1.ª redacção deste gate leu o `FLOAT_TAPES`** — a fita do
/// **traçado**, e não a do **PONTO**, que é a que o `Field::new` compila — e mediu **zero de zero**.
/// Quem a apanhou foi o **piso**: *uma régua que lê zero nos dois lados é verde e não afirma nada.* Construir a tabela compila **uma fita por folha**; re-traduzir os números não
/// pode compilar **nenhuma**.
///
/// ⚠️ **O piso do lado caro é afirmado também**: sem ele, um `Table::build` que deixasse de compilar
/// (porque alguém lhe tirou os donos) leria `0` nos dois lados e o gate ficaria verde a medir nada.
///
/// ⚠️ **Corre por `nextest`, que dá um processo por teste** — o §9 do `docs/Render3d/05` mede o que
/// acontece a este contador sob `cargo test`: as threads vêem-se umas às outras e oito gates caem.
///
/// **Mutação que deve sangrar:** `refresh_authored` a delegar num `Table::build`.
#[test]
fn dragging_a_colour_compiles_no_tape_at_all() {
    use std::sync::atomic::Ordering;
    // ⚠️⚠️ **A fixtura leva materiais DISTINTOS, e foi o PISO deste gate que o exigiu.** Ele
    // reprovou no dia em que a lei do dono passou a pedir materiais distintos, com a mensagem que
    // o próprio doc dele prevê por escrito (*«um `Table::build` que deixasse de compilar leria `0`
    // nos dois lados e o gate ficaria verde a medir nada»*). *Um piso que apanha a sua própria
    // premissa a morrer é a melhor prova de que ele tinha de existir.*
    let (mut sim, root) = a_world_de_dois_materiais();
    let folhas = leaves_of(sim.world(), root);

    // ── O lado CARO: construir a tabela compila uma fita por folha ──
    ph2d_field_eval::POINT_TAPES.store(0, Ordering::Relaxed);
    let mut t = Table::build(sim.world(), root, 0.8, 480.0);
    let compiladas = ph2d_field_eval::POINT_TAPES.load(Ordering::Relaxed);
    assert!(
        compiladas >= folhas.len(),
        "o piso: construir a tabela de {} folhas compilou {compiladas} fitas — se for zero, este \
         gate não tem lado caro e o barato não afirma nada",
        folhas.len()
    );

    // ── O lado BARATO: mexer num número do material não pode compilar nada ──
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(10),
        0.9,
    )
    .expect("a rugosidade");
    ph2d_field_eval::POINT_TAPES.store(0, Ordering::Relaxed);
    let mudou = t.refresh_authored(sim.world(), root);
    let depois = ph2d_field_eval::POINT_TAPES.load(Ordering::Relaxed);
    assert!(
        mudou,
        "o controlo: o número mudou e a tabela não deu por isso"
    );
    assert_eq!(
        depois, 0,
        "⛔ re-traduzir os números compilou {depois} fita(s) — é um JIT por folha a cada quadro de \
         um arrasto do selector de cor"
    );
}

/// ⏱️ **SONDA — o que CONSTRUIR a tabela custa por quadro** (a medição que o `docs/Render3d/05`
/// §11.6 encomendou por escrito: *«a primeira medição da wave seguinte»*).
///
/// # ⚠️ A pergunta, e porque a outra sonda não lhe responde
///
/// A [`pick_tests::measure_what_a_material_per_object_would_cost`] mediu a **RESOLUÇÃO** — *«de quem
/// é este pixel?»*, `1,6 ms` a 16 folhas — com as fitas **já compiladas**. Ela não mediu a
/// compilação, e o doc dela diz-o: *«medir sem ela mediria o JIT, não a pergunta»*.
///
/// ⛔ **Mas o produto paga o JIT.** O [`super::sync`] reconstrói a tabela sempre que o documento
/// muda, e o documento muda em **todo quadro de um arrasto do gizmo** — mover uma forma reescreve a
/// pose, o cozimento devolve outro `FieldDoc`, e o `mudou_o_doc` do `scene.rs` fica `true`. ⇒ o
/// número desta sonda é um **custo por quadro**, não um custo de arranque.
///
/// ⚠️ Corra-a com a máquina calma — ela imprime o `loadavg` ao lado, e acima de `~5` a leitura não
/// vale nada (`CLAUDE.md` §5.0).
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_building_the_table_costs_per_frame() {
    use std::time::Instant;

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("load: {}", carga.split_whitespace().next().unwrap_or("?"));
    println!("folhas ·   build ·  leaves ·  Owners ·  % de um quadro de 16,7 ms");
    for k in [1usize, 2, 4, 8, 16, 32, 64] {
        let lado = (k as f32).sqrt().ceil() as usize;
        let passo = 0.9 / lado as f32;
        let mut nodes: Vec<Node> = (0..k)
            .map(|i| Node {
                xform: Xform::at(
                    ((i % lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    ((i / lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    0.0,
                ),
                kind: NodeKind::Leaf(Primitive::Sphere {
                    radius: passo * 0.45,
                }),
                mods: Vec::new(),
                verb: None,
            })
            .collect();
        nodes.push(Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: (0..k).map(|i| NodeId(i as u32)).collect(),
            },
            mods: Vec::new(),
            verb: None,
        });
        let doc = FieldDoc::new(nodes, NodeId(k as u32)).expect("a grelha de esferas");
        let mut sim = ph2d_ecs::SimWorld::new();
        let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
        let world = sim.world();

        // ⚠️ **A mediana de N, nunca uma leitura** — e um aquecimento antes, senão a 1.ª corrida
        // mede o `malloc` a crescer.
        let med = |f: &mut dyn FnMut()| -> f64 {
            f();
            let mut v: Vec<f64> = (0..9)
                .map(|_| {
                    let t = Instant::now();
                    f();
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let build = med(&mut || {
            let _ = Table::build(world, root, 0.8, 480.0);
        });
        let folhas = med(&mut || {
            let _ = super::leaves(world, root);
        });
        let (_, placed) = super::leaves(world, root);
        let reg = crate::smoke::sampled_registry();
        let margem = ph2d_field_render::hit_tolerance(0.8, 480.0);
        let owners = med(&mut || {
            let _ = ph2d_field_eval::owners::Owners::new(&placed, &reg, margem);
        });
        println!(
            "{k:6} · {build:7.3} · {folhas:7.3} · {owners:7.3} · {:5.1} %",
            build / 16.7 * 100.0
        );
    }
}

/// ⭐⭐⭐ **TODO NÚMERO QUE UM MATERIAL TEM CHEGA À LEI** — o censo, derivado de
/// [`ph2d_field::MATERIAL_FIELDS`] e não de uma lista escrita à mão.
///
/// # ⛔⛔ Porque ele existe depois de três waves que já o provavam por pedaços
///
/// A cor (§12), o brilho (§20) e o verniz (§21) trouxeram cada um o seu gate, e cada um cobria **os
/// números daquela wave**. ⚠️ *Um número novo escrito no [`ph2d_field_ecs::FieldMaterial`] e
/// esquecido no [`crate::materials::surface_of`] não acordaria nenhum deles* — ele viajaria até ao
/// documento, seria gravado no arquivo, apareceria no painel, e **não faria nada**. É o *consumidor
/// que projecta o valor fora* do `CLAUDE.md` §5.0, na forma mais cara: o fio inteiro, e o fim mudo.
///
/// ⇒ este percorre **todas** as posições que a escrita aceita, uma a uma.
///
/// # ⚠️ As duas coisas que o tornam honesto
///
/// 1. **A base tem os dois PESOS ligados.** A cor da emissão e os quatro números do verniz são
///    inertes por construção com os pesos a zero — medi-los ali provaria o contrário do que se quer.
/// 2. **A régua é a RADIÂNCIA, e não a [`ph2d_material::Surface`]:** ela guarda o `OpenPbr` inteiro
///    lá dentro, logo duas superfícies com params diferentes são **sempre** diferentes por
///    `PartialEq`. *Compara-se o que a lei RESPONDE, não o que ela guarda.*
///
/// ⚠️ **E o valor alternativo de cada posição não é uma constante:** ele é *«longe do que lá está»*,
/// porque um número que por acaso já valesse `0,5` não se moveria ao ser posto em `0,5`.
///
/// **Mutação que deve sangrar:** apagar qualquer linha do `surface_of`.
#[test]
fn every_number_a_material_has_reaches_the_law() {
    use ph2d_field_ecs::FieldMaterial;

    // ⚠️ **Os dois pesos acesos** — ver a nota acima. E a base é fosca, para o verniz ter onde
    // aparecer.
    // ⚠️⚠️ **E a SUBSUPERFÍCIE também tem de estar acesa** (17/09): com `subsurface_weight = 0` as
    // nove entradas dela são inertes POR LEI, e este censo acusaria nove knobs vivos de uma vez —
    // *um corpus no ponto NEUTRO de um knob não testa esse knob*. ⛔ E o peso fica em `0,5` e não
    // em `1`: a `1` a difusa da base desaparece do `mix`, e as entradas dela passariam a ser as
    // inertes. **Este é um dos DOIS pesos que a mesma frase acima já pedia, agora com um terceiro.**
    let base = FieldMaterial {
        roughness: 0.6,
        emission: 0.5,
        coat: 0.7,
        subsurface_weight: 0.5,
        ..FieldMaterial::default()
    };
    let devolve = |m: FieldMaterial| {
        // ⚠️ **A curvatura é uma ENTRADA da lei**, como a direcção da luz: sem ela o caminho maciço
        // resolve-se no raio de `100` do piso e fica indistinguível de uma difusa — as entradas do
        // raio e da escala dele passariam a ler-se mortas. Ver `ph2d_field_render::curvatura`.
        let s = crate::materials::surface_of(m).at_curvature(1.0);
        let n = [0.0_f32, 0.3, 0.953_939_2];
        let v = [0.0_f32, 0.0, 1.0];
        let para_a_luz = [0.4_f32, 0.6, 0.692_820_3];
        let d = s.direct(n, v, para_a_luz, [3.0; 3]);
        let e = s.emission(n, v);
        [d[0] + e[0], d[1] + e[1], d[2] + e[2]]
    };
    let referencia = devolve(base);

    // ⭐⭐⭐ **DOIS caminhos, e cada número tem de mover PELO MENOS UM** — a subsuperfície escolhe
    // entre a parede fina e a maciça, e as duas leem entradas DIFERENTES: o raio e a escala dele
    // são da maciça (a fina não sabe nada sobre a forma da peça) e a FASE é da fina (o
    // `mx_subsurface_bsdf` da referência recebe a anisotropia e **nunca a usa** — está no corpo
    // dele). ⇒ exigir que todas movam o MESMO caminho acusaria quatro entradas vivas.
    let fina = FieldMaterial {
        thin_walled: 1.0,
        ..base
    };
    let ref_fina = devolve(fina);
    for k in 0..ph2d_field::MATERIAL_FIELDS {
        let mut outro = base;
        let antes = outro.get(k).expect("a posição existe");
        // Longe do que lá está, e dentro da faixa de todas elas (os DOIS IOR vivem em `1..=2,5`).
        let novo = if k == 11 || k == 17 {
            if antes > 1.75 { 1.1 } else { 2.4 }
        } else if antes > 0.5 {
            0.1
        } else {
            0.9
        };
        assert!(outro.set(k, novo), "a posição {k} recusou a escrita");
        let agora = devolve(outro);
        let d_macico = (0..3)
            .map(|c| (agora[c] - referencia[c]).abs())
            .fold(0.0_f32, f32::max);
        let mut outra_fina = fina;
        assert!(outra_fina.set(k, novo), "a posição {k} recusou a escrita");
        let agora_fina = devolve(outra_fina);
        let d_fina = (0..3)
            .map(|c| (agora_fina[c] - ref_fina[c]).abs())
            .fold(0.0_f32, f32::max);
        let d = d_macico.max(d_fina);
        assert!(
            d > 1.0e-3,
            "o número {k} do material ({antes} → {novo}) NÃO move a radiância ({referencia:?} → \
             {agora:?}) — ele chega ao documento, ao arquivo e ao painel, e morre no `surface_of`"
        );
    }
}

/// ⭐⭐⭐ **NENHUM MATERIAL QUE UM GESTO PRODUZ DEVOLVE LUZ NEGATIVA** — a cerca que o `docs/Render3d/05`
/// §8 nomeou e nunca gateou.
///
/// # ⛔⛔ O defeito que ela vigia, e porque ele é do MODELO e não nosso
///
/// O multi-scatter da difusa do OpenPBR tem um **polo**: com `base_diffuse_roughness = 1` o
/// denominador zera em `base_color ≈ 5,981`, e a `6,0` a indirecta devolve `[−676, −716, −813]`. É
/// uma propriedade do **GLSL de referência**, que esta crate porta fielmente — ⛔ *«melhorar» a
/// fórmula seria deixar de ser a referência.*
///
/// A §8 escreveu a cerca: *«a porta que deixar autorar `base_color` coage a `0..1`»*. A porta existe
/// (o selector de cor, §12) e a `base_diffuse_roughness` passou a ser autorável em 14/09 (§22) —
/// **este é o gate que faltava**, e ele mede o espaço inteiro que um gesto alcança, não o par que a
/// nota nomeia.
///
/// # ⚠️ A varredura é DETERMINÍSTICA, e cobre as 23 posições
///
/// Um LCG de semente fixa dá `20 000` materiais, cada número uniforme **na faixa que o slider
/// oferece** (`0..1`, ou `1..2,5` nos dois IOR). ⭐ *Um gate aleatório com semente fixa é
/// reproduzível como um literal e cobre o que uma tabela escrita à mão nunca cobriria* — e a mesma
/// varredura acorda sozinha quando um número novo entrar, porque ela é derivada do
/// `MATERIAL_FIELDS`.
///
/// ⚠️ **E ela mede `direct`, `indirect` E `emission`** — o polo mora na indirecta, e um gate que só
/// olhasse a luz directa estaria a olhar para o lado.
///
/// ⭐ **Medido ao escrevê-la: o polo continua INALCANÇÁVEL pelo produto**, e por uma razão que a §8
/// não tinha. O suspeito era o `base_weight`, que multiplica a cor base e **tem campo numérico
/// aberto** — mas ele escala a indirecta **linearmente** (`1 → 8` dá `0,36 → 2,80`, sempre
/// positiva). O polo exige a **cor** acima de `~6`, e a única porta que a escreve é o selector, que
/// fala `sRGB8`. ⇒ *a cerca da §8 estava certa e o mecanismo dela era outro.*
///
/// ⛔ **E a prova de que esta varredura não é fraca é uma mutação na própria CERCA:** levantar a
/// coerção a `0..1` (`u × 8`) põe-na vermelha com `[15,2, −1,0, 3,6]`. *Um gate de ausência tem de
/// mostrar que alcança a presença.*
#[test]
fn no_material_a_gesture_can_produce_returns_negative_light() {
    use ph2d_field_ecs::FieldMaterial;

    let mut estado = 0x2026_0914_u64;
    let mut proximo = || {
        estado = estado
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((estado >> 33) as f32) / ((1u64 << 31) as f32)
    };
    let n = [0.0_f32, 0.3, 0.953_939_2];
    let v = [0.0_f32, 0.0, 1.0];
    for _ in 0..20_000 {
        let mut m = FieldMaterial::default();
        for k in 0..ph2d_field::MATERIAL_FIELDS {
            let u = proximo();
            // As faixas são as do `material_span`: fracções, e os dois IOR físicos.
            let valor = if k == 11 || k == 17 { 1.0 + u * 1.5 } else { u };
            assert!(m.set(k, valor), "a posição {k} recusou a escrita");
        }
        let s = crate::materials::surface_of(m);
        let luz = {
            let d = s.direct(n, v, [0.4, 0.6, 0.692_820_3], [3.0; 3]);
            let i = s.indirect(n, v, &crate::render_light::StudioSky);
            let e = s.emission(n, v);
            [0, 1, 2].map(|c| d[c] + i[c] + e[c])
        };
        assert!(
            luz.iter().all(|c| c.is_finite() && *c >= 0.0),
            "um material que os sliders produzem devolveu luz NEGATIVA ou não-finita: {luz:?} de \
             {m:?}"
        );
    }
}

/// ⭐⭐⭐⭐ **N FOLHAS COM O MESMO MATERIAL NÃO PEDEM LEI DO DONO — e isso vale `300×`.**
///
/// A lei do dono emite **uma fita inteira por folha** ([`ph2d_field_eval::owners_wgsl`]:
/// `dono_folha_0`, `dono_folha_1`, …) e essas fitas entram no **TEXTO** do shader do pintor, cujo
/// cache tem por chave o texto ⇒ *toda forma acrescentada é um texto novo e uma compilação inteira
/// do driver*. Medido 2026-09-22 (`docs/Render3d/03` §W9): com lei do dono, acrescentar uma forma
/// custa **`2 310 ms`**; sem ela, **`7,85 ms`**.
///
/// ⚠️⚠️ **A saída é byte-idêntica por CONSTRUÇÃO:** com todos os materiais iguais o `dono_mix`
/// devolve `(a, b, t)` cujos `ler_mat(a)` e `ler_mat(b)` dão o MESMO `Mat`, logo a lei calcula
/// `ca + (ca − ca) · t` — que é `ca` **exactamente**, porque o termo é `0,0 · t`. O gate de PIXEL
/// que o afirma vive no [`crate::preview::device_tests`]; este afirma a DECISÃO.
///
/// ⛔ **E isto não é o caso raro, é o caso NORMAL de quem modela:** uma peça a ser construída tem o
/// material de omissão em toda folha. O caso com materiais distintos é o do gate irmão
/// [`each_leaf_wears_its_own_material`], que continua a pedir a lei.
///
/// ⭐ **O CONTROLO está dentro**, e é ele que impede a cura de virar *«nunca há lei do dono»*:
/// autorar um material diferente numa folha traz a lei de volta na mesma fixtura.
#[test]
fn n_folhas_com_o_mesmo_material_nao_pedem_lei_do_dono() {
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);
    assert_eq!(folhas.len(), 2, "a fixtura tem de ter DUAS folhas");

    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert_eq!(t.surfaces.len(), 2, "uma superfície por folha");
    assert!(
        t.owners.is_none(),
        "duas folhas com o MESMO material pediram lei do dono — ela é inerte aqui e põe uma fita \
         por folha no texto do shader, o que faz toda forma acrescentada recompilar o pintor"
    );

    // ⭐ **O CONTROLO**: um material autorado diferente traz a lei de volta.
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(3),
        0.0,
    )
    .expect("o azul");
    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert!(
        t.owners.is_some(),
        "CONTROLO: com materiais DISTINTOS a lei do dono tem de existir — sem esta metade a cura \
         lê-se como «nunca há lei do dono» e cada folha passaria a usar o material da primeira"
    );
}
