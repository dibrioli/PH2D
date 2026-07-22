//! **Todo componente de física registrado tem de ser autorável pela UI.**
//!
//! Enio, depois do smoke das oito waves: *"tudo isso está exposto na UI e é possível
//! criar essas cenas todas usando apenas os parâmetros expostos?"*. A resposta era sim —
//! conferida componente a componente — mas **prosa envelhece**, e a nona wave vai ser
//! escrita por alguém que não estava nesta conversa.
//!
//! Um componente que chega ao motor sem chegar à §11 é o **órfão** que a DIRETIVA §2
//! proíbe: ele funciona em toda cena de smoke (que constrói com código) e é inalcançável
//! no produto. É o modo de falha exato que o painel de MUNDO teve no W2b — tudo a
//! jusante funcionando sobre um painel que não existia no build.
//!
//! O gate é **estrutural, sobre o fonte**: para cada nome registrado em
//! `register_physics_components`, alguém no caminho de ESCRITA da UI tem de nomeá-lo.
//! Não prova que o controle está pintado (isso é o `architecture_panel_wiring_parity` e
//! os seams que CLICAM), prova que ele **existe** — e é a metade que faltava.

use std::fs;

/// Os arquivos por onde uma edição do Inspector vira componente. Se um dia houver um
/// quarto, ele entra aqui — e o gate falha até entrar, que é o ponto.
const WRITERS: [&str; 3] = [
    "src/render_loop/inspector_physics_apply.rs",
    "src/render_loop/inspector_physics_markers.rs",
    "src/render_loop/inspector_joint.rs",
];

#[test]
fn every_registered_physics_component_has_a_ui_writer() {
    let registry = fs::read_to_string("../../crates/ph2d-physics-ecs/src/lib.rs")
        .expect("o registry de física");
    let written: String = WRITERS
        .iter()
        .map(|f| fs::read_to_string(f).unwrap_or_else(|e| panic!("{f}: {e}")))
        .collect();

    // Os nomes canônicos exatamente como o registry os cunha — a mesma string que o
    // `queue_set` precisa para achar o `type_id`, então comparar por ela é comparar a
    // coisa que de fato liga os dois lados.
    let mut registered: Vec<&str> = registry
        .lines()
        .filter_map(|l| l.trim().strip_prefix("reg.register::<"))
        .filter_map(|l| l.split('"').nth(1))
        .collect();
    registered.sort_unstable();
    assert!(
        registered.len() >= 18,
        "o registry encolheu ({} nomes) — ou o parse quebrou, e um gate que não lê nada \
         passa sempre",
        registered.len()
    );

    let orphans: Vec<&str> = registered
        .iter()
        .copied()
        .filter(|name| !written.contains(name))
        .collect();
    assert!(
        orphans.is_empty(),
        "componentes de física que NENHUM caminho da UI escreve: {orphans:?}\n\n\
         Um componente sem UI funciona em toda cena de smoke (que constrói com código) e \
         é inalcançável no produto — o órfão que a DIRETIVA §2 proíbe. Ou dê a ele uma \
         row na §11 (e um arm em `apply_physics_edit` / `apply_marker_edit`), ou não o \
         registre ainda."
    );
}
