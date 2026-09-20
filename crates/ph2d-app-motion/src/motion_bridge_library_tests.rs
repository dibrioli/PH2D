//! Gates for the "Add Node" palette wiring (the shell half). `super` is `render_loop::motion_bridge`.
//! The editor-core chrome half (scrim closes / item records the pick) is gated in
//! `ph2d-editor-core::screens::hero::chrome::command_palette`; here we prove the Motion-specific glue:
//! the model built from the live catalog, and the `OpenLibrary` intent stashing the spawn.

use crate::motion_state::MotionState;
use ph2d_panel_motion_graph::GraphIntent;

/// **The palette model groups the live catalog by category, sub-clustering the two overloaded ones.**
/// FALSIFIED by leaving Transform flat (its subs would be one `None`-titled cluster), or by dropping the
/// grouping (the model would be empty / mis-ordered).
#[test]
fn the_palette_model_groups_the_catalog_by_category_with_subclusters() {
    let m = MotionState::new();
    let model = super::library::build_palette_model(&m.registry, &[]);

    let titles: Vec<&str> = model.groups.iter().map(|g| g.title.as_str()).collect();
    assert_eq!(
        titles.first(),
        Some(&"Source"),
        "Source leads (display order)"
    );
    assert!(
        titles.contains(&"Transform") && titles.contains(&"Utility"),
        "the overloaded categories are present"
    );
    assert!(
        model.item_count() >= 100,
        "the full catalog is present (~111 nodes), got {}",
        model.item_count()
    );

    // Transform is split into NAMED sub-clusters; a plain category (Source) is one flat, un-titled list.
    let tx = model
        .groups
        .iter()
        .find(|g| g.title == "Transform")
        .expect("Transform group");
    assert!(
        tx.subs.iter().all(|s| s.title.is_some()),
        "Transform is split into named sub-clusters"
    );
    assert!(
        tx.subs
            .iter()
            .any(|s| s.title.as_deref() == Some("Forces & Physics")),
        "the Forces & Physics sub-cluster exists"
    );
    let src = model
        .groups
        .iter()
        .find(|g| g.title == "Source")
        .expect("Source group");
    assert_eq!(src.subs.len(), 1);
    assert!(
        src.subs[0].title.is_none(),
        "Source is a flat category (no sub-headers)"
    );

    // Every item id is `hash_node_id(type_name)` — the round-trip the pick closes in the bridge. Prove it
    // for one known node so a change to the id scheme is caught.
    let boids_id = ph2d_tool_registry::hash_node_id("motion.boids");
    assert!(
        model
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .any(|it| it.id == boids_id),
        "an item carries the hash of its type_name (motion.boids), so the pick maps back"
    );
}

/// **A non-empty `compatible` filters the palette to the smart-connect allow-list.** A loose end dropped
/// in space opens the palette showing ONLY the types that wire can feed. FALSIFIED by ignoring
/// `compatible` (the whole catalog would show, and the artist would pick something the shell then
/// refuses).
#[test]
fn the_palette_model_is_filtered_to_the_compatible_types() {
    let m = MotionState::new();
    let full = super::library::build_palette_model(&m.registry, &[]);
    let filtered = super::library::build_palette_model(&m.registry, &["motion.boids"]);
    assert!(
        full.item_count() > filtered.item_count(),
        "the allow-list narrows the catalog"
    );
    assert_eq!(
        filtered.item_count(),
        1,
        "only the one allowed type is shown"
    );
    let boids = ph2d_tool_registry::hash_node_id("motion.boids");
    assert!(
        filtered
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .any(|it| it.id == boids),
        "and it is the allowed type"
    );
}

/// **The `OpenLibrary` intent stashes the spawn AND the wire context for the bridge to open the palette.**
/// The bridge — which owns the editor `WidgetStore` — reads `open_library` after the drain, filters the
/// model to `compatible` and opens the full-screen palette. FALSIFIED by not handling `OpenLibrary` (the
/// stash stays `None`, the gesture opens nothing) or by dropping the wire context (smart-connect / splice
/// would fall back to a plain add).
#[test]
fn open_library_intent_stashes_the_spawn_and_wire_context() {
    use crate::motion_state::LibraryOpen;
    let mut m = MotionState::new();
    assert!(
        m.open_library.is_none(),
        "nothing stashed before the intent"
    );

    let _ = ph2d_panel_motion_graph::drain_intents(); // clear any intent the boot left behind
    ph2d_panel_motion_graph::push_intent(GraphIntent::OpenLibrary {
        x: 12.0,
        y: 34.0,
        connect_from: Some((7, 1)),
        connect_to: None,
        splice: None,
        compatible: vec!["motion.grid"],
    });
    super::apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor_core::ToastQueue::default(),
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
    );

    assert_eq!(
        m.open_library,
        Some(LibraryOpen {
            spawn: (12.0, 34.0),
            connect_from: Some((7, 1)),
            connect_to: None,
            splice: None,
            compatible: vec!["motion.grid"],
        }),
        "OpenLibrary stashes the spawn + the wire context; the bridge opens the palette with them"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// doc 115 W6 — O COLISOR SAI DA LISTA E O MOTOR FICA
// ─────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **O `motion.collide` não é oferecido, e a POPULAÇÃO dos retirados é gateada.**
///
/// Ordem do dono (2026-09-17): *«tirar o collide e deixar tudo pela Shape»*. O passe do
/// fim do cozimento faz o trabalho desde a W5, e a forma DECLARA a caixa dela desde a W4
/// — nenhum nó é preciso, e nenhum gesto do app pode acrescentar um.
///
/// ⚠️ **As DUAS metades, porque sozinha cada uma mente:** a primeira sem a segunda
/// passaria no dia em que alguém escondesse meio catálogo, e a segunda sem a primeira
/// diz que *um* nó está retirado sem dizer **qual**. ⛔ E o piso é `== 1` e não `>= 1`:
/// *uma catraca sem censo de obsolescência vira LICENÇA* (`CLAUDE.md` §5.0) — quem
/// esconder o segundo nó tem de vir aqui escrever a medição dele.
#[test]
fn o_colisor_saiu_do_catalogo_e_e_o_unico_retirado() {
    let m = MotionState::new();
    let catalogo = super::library::build_catalog(&m.registry);

    assert!(
        !catalogo.iter().any(|c| c.type_name == "motion.collide"),
        "o `motion.collide` nao pode ser oferecido — a ordem do dono e' que o artista \
         separa pelo `Collide` do cartao do Output, e a forma declara a caixa dela"
    );

    let retirados: Vec<&str> = m
        .registry
        .out_of_catalogue_ids()
        .filter_map(|id| m.registry.manifests().find(|man| man.id == id))
        .map(|man| man.name)
        .collect();
    assert_eq!(
        retirados,
        vec!["motion.collide"],
        "so' UM tipo esta' fora do catalogo, e e' este — quem retirar outro escreve a \
         MEDICAO ao lado do `register_out_of_catalogue` dele e corrige este gate"
    );
}

/// ⛔⛔ **E o MOTOR fica — esta é a metade que o dono decidiu, e sem ela a primeira
/// leria como «apagámos o nó».**
///
/// As duas cenas de banco do dispositivo (`=8` e `=9`, ADR-0140 Fase 5) montam-no e
/// **validam contra o registo**, e o nó mantém o kernel e a grelha. *Retirar a porta do
/// artista não pode apagar o kernel que faz `129 600` discos em `4,71 ms` contra
/// `416,29 ms` do passe na CPU* (§0.0: nunca deixar o caminho lento definir o produto).
///
/// ⚠️⚠️ **A 1.ª redacção deste gate COZINHAVA as duas cenas e demorou `1 553 s`** — ela
/// punha `129 600` peças pela CPU, **em debug**, que é precisamente o caminho que esta
/// wave existe para NÃO usar. Além de ser morta pelo tecto de `180 s` da suíte, ela
/// media a grandeza errada: *o que o dono decidiu preservar foi o KERNEL*, e quem
/// responde por ele é o registo, não um cozimento de referência. ⇒ pergunta-se ao
/// `KernelResolver`, que é mais barato **e** mais próximo da afirmação.
#[test]
fn as_duas_cenas_de_banco_montam_e_o_kernel_do_dispositivo_fica() {
    use ph2d_nodegraph::gpu::KernelResolver;
    let id = ph2d_nodegraph::node::NodeTypeId::of("motion.collide");

    for nivel in ["8", "9"] {
        let mut m = MotionState::new();
        let sinks =
            crate::motion_state::demo_router::build_level(Some(nivel), &mut m.doc, &m.registry);
        assert!(
            !sinks.is_empty(),
            "a cena =({nivel}) tem de montar — ela valida contra o registo, logo isto \
             reprova no dia em que o tipo deixar de ser registado"
        );
        assert!(
            m.doc
                .graph
                .nodes()
                .iter()
                .any(|n| n.type_name == "motion.collide"),
            "a cena =({nivel}) e' o banco do KERNEL — sem o no' ela deixa de ter sujeito"
        );
    }

    // ⭐ O que o dono mandou preservar, perguntado a quem o guarda.
    assert!(
        m_registry().gpu_kernel(id).is_some(),
        "o kernel de dispositivo do colisor desapareceu — era ele que fazia 129 600 \
         discos em 4,71 ms contra 416,29 ms da CPU"
    );
    assert!(
        m_registry().grid(id).is_some(),
        "a grelha espacial do colisor desapareceu — ele e' o unico cliente ITERADO dela"
    );
}

/// Um registo povoado, para as perguntas que não precisam de cena.
fn m_registry() -> ph2d_node_registry::NodeRegistry {
    MotionState::new().registry
}

/// ⚠️ **UM filtro, DUAS superfícies** — a paleta deriva do catálogo, e é isso que
/// impede o defeito que este repo já pagou: *uma lista escrita à mão ao lado de um
/// predicado é a segunda resposta à mesma pergunta, e a que o artista vê é a que
/// envelhece* (o `import_router`, 23/08).
///
/// ⛔ Sem este gate, alguém dá à paleta a própria varredura do registo e o nó volta a
/// aparecer **só ali** — com o menu do grafo, que lê o catálogo, a continuar limpo.
#[test]
fn a_paleta_nao_oferece_o_que_o_catalogo_nao_oferece() {
    let m = MotionState::new();
    let modelo = super::library::build_palette_model(&m.registry, &[]);
    let escondido = ph2d_tool_registry::hash_node_id("motion.collide");
    let aparece = modelo
        .groups
        .iter()
        .flat_map(|g| g.subs.iter())
        .flat_map(|s| s.items.iter())
        .any(|i| i.id == escondido);
    assert!(
        !aparece,
        "a paleta mostra o `motion.collide` — ela deixou de derivar do `build_catalog`"
    );
    // ⭐ O controlo POSITIVO: a extracção acima sabe achar um nó que EXISTE. Sem ele,
    // um `groups` vazio ou um hash trocado fariam este gate passar a medir nada.
    let vivo = ph2d_tool_registry::hash_node_id("motion.output");
    assert!(
        modelo
            .groups
            .iter()
            .flat_map(|g| g.subs.iter())
            .flat_map(|s| s.items.iter())
            .any(|i| i.id == vivo),
        "o controlo falhou: esta regua nao acha nem o `motion.output`, logo nao \
         afirma nada sobre o que ela diz estar ausente"
    );
}
