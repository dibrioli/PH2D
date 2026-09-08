//! ⭐⭐⭐ **OS VERBOS TARDIOS DA HIERARQUIA** — a promessa que a cura do reparent deixou por medir.
//!
//! # A pergunta, e a resposta que ela deu
//!
//! A F4.6c curou o *«reordenei objectos na hierarquia e não funcionou o undo»* movendo o dreno do
//! reparent para **antes** da projecção de z, e o handoff de 2026-09-07 nomeou o que ficava:
//!
//! > *«Os outros verbos tardios da Hierarquia — apagar · duplicar · Remove from Sheet — escrevem a
//! > árvore no MESMO sítio tardio e têm a mesma latência estrutural. NÃO foi medido se produzem o
//! > fantasma.»*
//!
//! ⚠️ **Medir era o passo, não curar** — se o fantasma não aparecesse, a resposta seria uma recusa
//! medida, e ela valeria tanto como uma cura. Os dois gates abaixo **nasceram vermelhos**: o
//! fantasma existe para os dois, e a cura é o [`crate::vec_tree_settle`].
//!
//! # O mecanismo, medido no produto (2026-09-08)
//!
//! O [`crate::vec_entities::sync`] é **bidireccional**, e é isso que fecha o ciclo:
//!
//! | verbo | o que o dreno escreve | o que o `sync` do quadro SEGUINTE faz sozinho |
//! |---|---|---|
//! | apagar | `despawn` da **entidade** (`hierarchy_delete.rs`) | tira o caminho do **documento** |
//! | duplicar | um caminho novo no **documento** (`hierarchy_duplicate.rs`) | cunha a **entidade** |
//!
//! Os dois drenos vivem no `render_loop::hierarchy::dispatch`, que corre **~2 300 linhas depois**
//! da projecção (`vec_scene.reorder_to`). ⇒ a fotografia do quadro do gesto guardava um mundo e um
//! documento que discordavam, e o quadro seguinte reconciliava-os **sem entrada nenhuma** — que é
//! a definição do passo fantasma.
//!
//! ⚠️ **Apagar tira do MUNDO e duplicar põe no DOCUMENTO**: as duas metades da mesma latência, e um
//! gate só deixaria metade da classe por medir.
//!
//! # ⚠️ Os CONTROLOS destes gates mudaram com a cura, e a mudança é o achado
//!
//! Na 1.ª redacção eles afirmavam *«o caminho ainda está no documento neste quadro»* e *«a entidade
//! ainda não existe»* — ou seja, descreviam o **defeito**, não a lei. Com a reconciliação a correr
//! antes da captura, os dois passaram a ser falsos: o gesto tardio deixa o estado **já fechado** na
//! fotografia. *Um controlo escrito contra o mundo doente reprova quando ele sara* — os de hoje
//! afirmam que a reconciliação FEZ o trabalho, que é o que impede o ponto fixo de ser vácuo.

use super::zorder_fixpoint_tests::{Frame, three_fresh_shapes, z};
use super::*;
use ph2d_vec_scene::{VecScene, rectangle};

/// ⭐⭐⭐ **APAGAR uma forma TARDE deixa a captura no ponto fixo.**
///
/// Vermelho antes da cura, com a mensagem inteira: a captura guardava o `world` sem a entidade e o
/// `vec` **ainda com o caminho**, e o quadro seguinte tirava-o sozinho.
///
/// ⚠️ **A régua é o PONTO FIXO, não a igualdade de um campo** — fotografar, deixar um quadro correr
/// sem entrada, e fotografar outra vez tem de dar a MESMA foto.
#[test]
fn a_late_delete_leaves_the_capture_a_fixed_point() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    let [_a, b, _c] = three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);
    let vitima = Entity::from_bits(*map.get(&b).expect("a forma tem entidade"));

    // O gesto TARDIO: o `hierarchy_delete::drain` despawna depois de a árvore já ter sido lida.
    frame.run_with_late(&mut sim, &mut scene, &mut map, |sim, _scene| {
        let _ = sim.world_mut().despawn(vitima);
    });
    let depois_do_gesto = frame.capture(&mut sim, &scene);

    // ⚠️ **CONTROLO — as DUAS metades**: sem elas o ponto fixo abaixo mediria um quadro parado.
    assert!(
        sim.world().get_entity(vitima).is_err(),
        "a fixtura nao apagou nada — o ponto fixo abaixo mediria um quadro parado"
    );
    assert!(
        !z(&scene).contains(&b),
        "a reconciliacao nao levou o caminho junto com a entidade — e' a metade do `sync` que faz \
         a fotografia ser coerente, e sem ela o ponto fixo passa por acidente noutro sitio"
    );

    // E agora o quadro seguinte, **sem entrada nenhuma**.
    frame.run(&mut sim, &mut scene, &mut map);
    let quadro_seguinte = frame.capture(&mut sim, &scene);

    assert!(
        depois_do_gesto == quadro_seguinte,
        "apagar TARDE deixa a captura fora do ponto fixo: o quadro seguinte muda o documento \
         sozinho, e o `post_frame_undo` le isso como uma accao do artista — e' o passo fantasma do \
         report de 2026-09-07, por outro verbo"
    );
}

/// ⭐⭐⭐ **DUPLICAR uma forma TARDE deixa a captura no ponto fixo.**
///
/// O ESPELHO do de cima: o `hierarchy_duplicate::drain` empurra um **caminho** para o documento (o
/// dono da geometria vetorial é ele) e quem cunha a entidade é o `sync`. Sem a reconciliação, a
/// captura guardava um `vec` com o caminho e um `world` sem a entidade dele.
#[test]
fn a_late_duplicate_leaves_the_capture_a_fixed_point() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut frame = Frame::new(&mut sim);

    three_fresh_shapes(&mut scene);
    frame.run(&mut sim, &mut scene, &mut map);
    let antes = scene.paths().len();

    // O gesto TARDIO: o caminho novo entra no documento depois de o `sync` já ter corrido.
    frame.run_with_late(&mut sim, &mut scene, &mut map, |_sim, scene| {
        scene.push_path(rectangle([9.0, 0.0], [10.0, 1.0]));
    });
    let depois_do_gesto = frame.capture(&mut sim, &scene);

    // ⚠️ **CONTROLO — as DUAS metades**: o documento cresceu, e a entidade dele já foi cunhada.
    assert_eq!(
        scene.paths().len(),
        antes + 1,
        "a fixtura nao duplicou nada"
    );
    let novo = scene.paths().last().expect("o caminho novo").id;
    let bits = *map.get(&novo).expect(
        "a reconciliacao nao cunhou a entidade do caminho novo — sem ela a fotografia \
                 guarda um documento e um mundo que discordam",
    );
    assert!(
        sim.world()
            .get::<ph2d_ecs::StableId>(Entity::from_bits(bits))
            .is_some(),
        "a entidade nasceu SEM identidade duravel — ela entraria no snapshot sem `StableId` e o \
         primeiro Ctrl+Z nao teria o que repor"
    );

    frame.run(&mut sim, &mut scene, &mut map);
    let quadro_seguinte = frame.capture(&mut sim, &scene);

    assert!(
        depois_do_gesto == quadro_seguinte,
        "duplicar TARDE deixa a captura fora do ponto fixo: o mundo ganha a entidade sozinho no \
         quadro seguinte, e o `post_frame_undo` le isso como uma accao do artista"
    );
}

// ── ⭐⭐ O PREÇO DA SEGUNDA PASSAGEM (§0.0: medir antes de a pôr no caminho de todo quadro) ───────

/// ⭐⭐ **Quanto custa reconciliar a árvore uma segunda vez, por quadro.**
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- the_second_pass_costs --ignored --nocapture
/// ```
///
/// ⚠️ **`#[ignore]`**: mede um relógio (a família de flakes do `CLAUDE.md` §5.0) e imprime o `load`
/// ao lado — *sem ele, a régua que desmente uma flake é a própria flake*.
///
/// # Porque a pergunta é esta, e não «é barato?»
///
/// A cura põe uma passagem inteira — `sync` + assentamento do pivô + os três `assign_missing_*` +
/// a projecção — no caminho de **todo** quadro, não só no do gesto. O §0.0 manda medir antes de
/// escrever o limite, e o limite aqui é o orçamento de um quadro: **16,7 ms**.
///
/// ⚠️ **A dimensão varrida é o número de FORMAS**, porque é dela que os dois passes caros dependem:
/// o `sync` compara o mapa com as `paths` do documento, e a projecção percorre a árvore.
#[test]
#[ignore = "mede um relogio; rode com a maquina calma e --release"]
fn the_second_pass_costs_this_much_of_a_frame() {
    const ITERS: usize = 25;
    let load: f64 = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN);
    println!(
        "\n  a SEGUNDA passagem (a rede antes da captura) — mediana de {ITERS}, load = {load:.2}"
    );
    println!(
        "  ┌─────────┬──────────┬──────────┬──────────┬──────────┬──────────┬────────────────┐"
    );
    println!(
        "  │  formas │   sync   │   pivo   │ assigns  │ snapshot │ projecao │ TOTAL (% quadro)│"
    );
    println!(
        "  ├─────────┼──────────┼──────────┼──────────┼──────────┼──────────┼────────────────┤"
    );
    for n in [100usize, 1_000, 5_000] {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();
        let mut frame = Frame::new(&mut sim);
        for k in 0..n {
            let x = k as f64;
            scene.push_path(rectangle([x, 0.0], [x + 1.0, 1.0]));
        }
        // Um quadro normal primeiro: as entidades nascem, o pivô assenta, a pilha projecta-se.
        frame.run(&mut sim, &mut scene, &mut map);
        let mut col = [(); 5].map(|()| Vec::with_capacity(ITERS));
        for _ in 0..ITERS {
            let t = std::time::Instant::now();
            crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
            col[0].push(t.elapsed().as_secs_f64() * 1000.0);
            let t = std::time::Instant::now();
            crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
            col[1].push(t.elapsed().as_secs_f64() * 1000.0);
            let t = std::time::Instant::now();
            ph2d_ecs::assign_missing_root_order(sim.world_mut());
            ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
            ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
            col[2].push(t.elapsed().as_secs_f64() * 1000.0);
            let t = std::time::Instant::now();
            let snap = frame.snapshot_now(&mut sim);
            col[3].push(t.elapsed().as_secs_f64() * 1000.0);
            let t = std::time::Instant::now();
            scene.reorder_to(&snap);
            col[4].push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let med = col.map(|mut v| {
            v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
            v[v.len() / 2]
        });
        let total: f64 = med.iter().sum();
        println!(
            "  │ {n:>7} │ {:>6.3}ms │ {:>6.3}ms │ {:>6.3}ms │ {:>6.3}ms │ {:>6.3}ms │ {total:>7.3}ms {:>5.1}% │",
            med[0],
            med[1],
            med[2],
            med[3],
            med[4],
            total / 16.7 * 100.0
        );
    }
    println!(
        "  └─────────┴──────────┴──────────┴──────────┴──────────┴──────────┴────────────────┘\n"
    );
}
