//! **As settings que vivem FORA do `ProjectState` atravessam o arquivo** — irmão do
//! [`super::tests`] (também `#[path]`-filho de `project`, então `super::super` é
//! `project` e o `ProjectFile` privado está ao alcance), cortado quando o pai bateu
//! o cap de 600 LOC do HR-18.
//!
//! O corte é por ASSUNTO, e o assunto é uma família: `physics` e `settings` são
//! campos do ARQUIVO, deliberadamente fora do `ProjectState` porque um Ctrl+Z do
//! canvas não deve rebobinar a gravidade da cena nem a escala do mundo. Os quatro
//! gates aqui respondem à mesma pergunta duas vezes — *o arquivo carrega o que foi
//! autorado?* e *quem o instala de volta, e na ordem certa?* — e a segunda metade é
//! sempre um arch-gate, porque sem janela o `gfx` é `None` e a chamada nem roda.

use super::*;
// As fixtures moram no irmão `tests` e são compartilhadas em vez de copiadas — duas
// cópias de `empty_state` divergiriam no dia em que o `ProjectState` ganhar um campo.
use super::tests::{empty_state, tmp_path};

/// **As settings de MUNDO da física sobrevivem ao arquivo — e chegam ao SOLVER.**
///
/// ⚠️ **O que este gate NÃO prova:** que as settings chegaram ao SOLVER. Sem
/// janela o `gfx` é `None`, então nem o `rebuild()` nem o `set_settings` do load
/// rodam aqui — um oráculo sobre o bridge seria verde por ausência. Ele prova a
/// metade que pode: o arquivo carrega os valores autorados e o schema os aceita.
///
/// As outras duas metades têm gates próprios, de propósito:
/// - que `set_settings` sobrevive a um `rebuild` → `ph2d-physics-ecs`
///   (`the_settings_survive_a_rebuild`);
/// - que o load chama os dois na ORDEM certa → o arch-gate abaixo.
#[test]
fn the_world_settings_survive_the_project_file() {
    let path = tmp_path("physics_world_settings");

    // Um mundo autorado: gravidade zero (top-down) e arrasto pesado — nada que
    // um default possa produzir por acidente.
    let authored = ph2d_physics_ecs::PhysicsSettings {
        gravity_x: 0.0,
        gravity_y: 0.0,
        linear_damping: 3.5,
        substeps: 7,
        ..Default::default()
    };
    let file = ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: Vec::new(),
        motion: String::new(),
        timeline: Vec::new(),
        physics: authored,
        tokens: Vec::new(),
        settings: crate::project_settings::collect(Default::default()),
        sculpt: Vec::new(),
        baked_forms: Vec::new(),
        player_tape: ph2d_physics_ecs::TapeWire::default(),
        sprite_pixels: Vec::new(),
    };
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");

    let mut app = crate::App::new();
    app.project_load_from(path.to_str().unwrap());

    // `gfx` é `None` sem janela, então o caminho que instala no BRIDGE não roda
    // aqui; o que este gate pode afirmar sem GPU é que o arquivo entregou os
    // valores autorados. (O lado do bridge é gateado em `ph2d-physics-ecs`.)
    let (ver, back): (u32, ProjectFile) =
        postcard::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(ver, PROJECT_SCHEMA);
    assert_eq!(
        back.physics, authored,
        "as settings de mundo nao sobreviveram ao arquivo"
    );
    let _ = app;
}

/// **A ESCALA E A UNIDADE do projeto sobrevivem ao arquivo** (doc 88, D3).
///
/// `pixels_per_meter` não é conforto: desde a fronteira de display dos params de
/// Motion ele é a régua pela qual toda row de COMPRIMENTO é lida, e `display_unit`
/// escolhe a face. Perdê-los num reload muda os números na tela sem que ninguém
/// tenha tocado no documento — e o artista de pixel art que afinou o projeto em
/// `32 px/m` reabria em `100`.
///
/// ⚠️ **O que este gate NÃO prova**, e é a mesma fronteira do irmão da física logo
/// acima: que as settings chegaram ao `HeroScreen`. Sem janela o `gfx` é `None`,
/// então o `install` do load não roda aqui — um oráculo sobre ele seria verde por
/// ausência. Ele prova a metade que pode (o arquivo carrega o que foi autorado e o
/// schema o aceita); a outra metade é o arch-gate abaixo.
#[test]
fn the_project_scale_and_unit_survive_the_project_file() {
    let path = tmp_path("project_settings");

    // Um projeto de pixel art: nada aqui é alcançável por acidente a partir do
    // default (100 px/m, Pixels, Smooth).
    let authored = ph2d_editor::project::ProjectSettings {
        pixels_per_meter: 32.0,
        snap_move_meters: 0.16,
        snap_rotate_deg: 15.0,
        display_unit: ph2d_editor::project::DisplayUnit::Meters,
        image_filter: ph2d_editor::project::ImageFilterMode::PixelArt,
    };
    assert_ne!(
        authored,
        ph2d_editor::project::ProjectSettings::default(),
        "uma fixture igual ao default nao prova travessia nenhuma"
    );
    let file = ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: Vec::new(),
        motion: String::new(),
        timeline: Vec::new(),
        physics: Default::default(),
        tokens: Vec::new(),
        settings: crate::project_settings::collect(authored),
        sculpt: Vec::new(),
        baked_forms: Vec::new(),
        player_tape: ph2d_physics_ecs::TapeWire::default(),
        sprite_pixels: Vec::new(),
    };
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");

    let (ver, back): (u32, ProjectFile) =
        postcard::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(ver, PROJECT_SCHEMA);
    let mut loaded = ph2d_editor::project::ProjectSettings::default();
    crate::project_settings::install(&mut loaded, &back.settings);
    assert_eq!(
        loaded, authored,
        "as settings do projeto nao sobreviveram ao arquivo"
    );
}

/// **O load INSTALA as settings do arquivo — arch-gate sobre o fonte.**
///
/// Um campo que o save grava e o load nunca instala é a falha silenciosa exata
/// desta classe: o arquivo fica correto, o artista reabre, e a escala é a de
/// fábrica sem uma linha de erro. E nenhum teste de unidade alcança a chamada —
/// `hero_screen` vive dentro do `gfx`, que é `None` sem janela, então o `if let`
/// nem entra. Quando o fato é *o produto chama isto*, o gate lê o produto.
#[test]
fn the_load_installs_the_project_settings_from_the_file() {
    let src = include_str!("project_load.rs");
    assert!(
        src.contains("project_settings::install("),
        "o load precisa instalar as settings do ARQUIVO — sem isto elas sao \
         gravadas e nunca voltam, e nada na tela diz porque a escala mudou"
    );
    assert!(
        src.contains("&file.settings"),
        "o install tem de receber as settings do ARQUIVO, nao um default — \
         instalar um default e o mesmo que nao instalar, com mais linhas"
    );
}

/// **O load instala as settings DEPOIS do `rebuild()` — arch-gate sobre o fonte.**
///
/// `rebuild()` constrói um `PhysicsWorld` novo, que nasce nos defaults do motor.
/// Instalar antes dele é escrever num mundo que ele joga fora: a cena carregaria
/// com a gravidade do documento ANTERIOR, sem erro nenhum.
///
/// Isto é uma afirmação sobre a ORDEM de duas chamadas, e nenhum teste de
/// unidade a alcança — `gfx` é `None` sem janela, então as duas linhas nem
/// rodam. Mesmo padrão (e mesmo motivo) do
/// `the_z_projection_reads_the_tree_after_the_sync`: quando o fato é a ordem do
/// código do produto, o gate lê o código do produto.
#[test]
fn the_load_installs_the_world_settings_after_the_rebuild() {
    // ⚠️ O arquivo mudou quando o load saiu do `project.rs` (teto de LOC do
    // HR-18) — o gate segue o FATO, que é a ordem de duas chamadas dentro do
    // load, e não o endereço onde ele morava.
    let src = include_str!("project_load.rs");
    let rebuild = src
        .find("physics.rebuild()")
        .expect("o load precisa derrubar o mundo derivado do documento anterior");
    let install = src
        .find("physics.set_settings(")
        .expect("o load precisa instalar as settings de mundo do ARQUIVO");
    assert!(
        rebuild < install,
        "`set_settings` aparece ANTES de `rebuild()` em project.rs: o rebuild \
         constroi um PhysicsWorld novo nos defaults do motor e joga fora o que \
         acabou de ser instalado — a cena carrega com a gravidade do documento \
         anterior, em silencio"
    );
}
