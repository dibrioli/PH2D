//! ⭐⭐⭐ **O NAVEGADOR DE ASSETS, DIRIGIDO PELO PONTEIRO DE VERDADE** — `PH2D_BUILD_SMOKE=83`.
//!
//! # Por que este ficheiro existe
//!
//! A auditoria da etapa B (2026-08-30) fechou com um buraco NOMEADO: *«não há teste que aperte o
//! botão de verdade neste caminho»*. Os 21 gates de costura do navegador entregam `WidgetEvent`
//! sintéticos a um `MockPanelHost` — eles provam que o **braço** existe, e passam por cima de tudo
//! o que vem antes dele: o hit-index, o `is_focusable`, o `pointer_down`, a precedência do
//! botão direito, o nascimento do `Click`.
//!
//! ⚠️ **É exactamente aí que esta linha já foi mordida duas vezes:** a faixa de arrasto registada
//! como `Button` (o painel abria e não se movia) e o polegar da barra sem registo nenhum
//! (inagarrável) — os dois com a suíte inteira verde. E o precedente maior é o da `line/Vector`:
//! *«um controlo nunca pintado e um morto sob o dedo dão o MESMO report, e só o gesto REAL mede a
//! segunda costura»*.
//!
//! ⇒ este roteiro carrega no rato. Ele não substitui os gates; ele mede a metade que eles não
//! alcançam, e por isso **imprime** em vez de afirmar (o `AppGfx` segura uma superfície de janela
//! real, então nada disto é alcançável de um `#[test]`).
//!
//! # Como se lê a telemetria
//!
//! ```text
//! [asset-menu] f=NN <passo> — <o que se mediu>
//! ```
//!
//! Cada passo diz o que TINHA de acontecer. ⛔ Uma linha com `NÃO` é um defeito, mesmo que o app
//! não estoure — foi assim que os quatro achados da auditoria da etapa A se leram.

use ph2d_editor::NodeId;
// A visibilidade de um painel é uma capacidade do HOST, não do `HeroScreen` — o trait tem de
// estar em escopo para o roteiro a poder usar.
use ph2d_editor::panel::PanelHostInternal;

// A entidade da receita e o sítio do cartão, guardados entre quadros para os passos a jusante
// os poderem conferir.
thread_local! {
    static MASTER: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Onde o primeiro cartão foi encontrado — os passos seguintes voltam AQUI, e não à posição
    /// corrente do cursor (que o próprio menu pode ter deslocado).
    static CARD_AT: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O painel precisa de um quadro pintado antes de os cartões existirem no hit-index.
        20 => open_panel(app),
        25 => new_image(app),
        28 => report_library(app),
        // ⚠️ **O filtro é apertado ANTES de procurar o cartão**, e não é conforto: a grade ordena
        // por NOME, então assim que a cena passou a criar uma imagem o cartão 0 deixou de ser o
        // prefab e os dois passos seguintes mediram o asset errado. *Um roteiro que assume a
        // posição de um cartão mede a ordenação, não o verbo.* O chip é um gesto real, e apertá-lo
        // exercita a fileira de filtros de passagem.
        29 => click_kind_chip(app),
        32 => find_card(app),
        36 => right_click_card(app),
        // ⚠️ **Um quadro entre o menu abrir e o item ser apertado.** O overlay só regista os
        // rect das linhas quando as pinta, e apertar no mesmo quadro em que ele abre acerta em
        // nada — que é um falso NEGATIVO deste instrumento, não um defeito do produto.
        39 => click_menu_row(
            app,
            ph2d_editor::ids::CTX_MENU_ASSET_SELECT_USERS,
            "Select users",
        ),
        42 => report_selection(app),
        46 => right_click_card(app),
        49 => click_menu_row(
            app,
            ph2d_editor::ids::CTX_MENU_ASSET_REMOVE,
            "Remove from Library",
        ),
        52 => report_removed(app),
        // ⚠️ **Depois dos passos do menu, e não no meio deles.** Na 1.ª versão a reprodução corria
        // antes e apagava a cópia que o `Select users` ia contar — *um passo que perturba o que o
        // passo seguinte mede transforma o instrumento num falso acusador.*
        55 => repro(app),
        57 => repro_after(app),
        59 => remove_the_unused_image(app),
        61 => repro_after(app),
        _ => {}
    }
}

/// A cena: uma receita na biblioteca, com uma cópia na cena.
fn build(app: &mut crate::App) {
    // ⚠️ Os DOIS documentos que a instanciação toca vivem em campos diferentes do `App`, e é por
    // isso que eles se pegam separadamente: o `vec_scene` é do `AppGfx`, o mapa é do `App`.
    let vec_entities = &mut app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let master = crate::instance_smoke::spawn_master(&mut gfx.sim);
    ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
    ph2d_ecs::assign_missing_root_order(gfx.sim.world_mut());
    let id = gfx
        .sim
        .world()
        .get::<ph2d_ecs::StableId>(master)
        .map_or(0, |s| s.0);
    MASTER.with(|c| c.set(id));

    // Uma cópia, pela porta do produto — é o que o `Select users` tem de encontrar.
    let registry = crate::init::build_component_registry();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut gfx.vec_scene,
        vec_entities,
    };
    let placed = crate::instantiate::instantiate_master(
        &mut gfx.sim,
        &registry,
        master,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .is_ok();
    ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
    eprintln!(
        "[asset-menu] f=3 cena — receita StableId={id}, cópia instanciada: {}",
        if placed { "sim" } else { "NÃO" }
    );
}

/// Abre o painel **pelo pill**, com o ponteiro — não por `set_panel_visible`.
fn open_panel(app: &mut crate::App) {
    if let Some((x, y)) = app.smoke_find_widget(ph2d_editor::ids::TOPBAR_RIGHT_ASSETS) {
        app.smoke_pointer_down(x, y);
        app.smoke_pointer_up();
        eprintln!("[asset-menu] f=20 pill `Assets` apertado em ({x}, {y}) — pelo PONTEIRO");
        return;
    }
    // ⛔⛔ **ACHADO de 2026-08-30, e ele é PRÉ-EXISTENTE, não desta wave.**
    //
    // A janela nasce a **1024×768** (`init.rs`), e a essa largura a barra de cima **transborda**: o
    // grupo da direita é alinhado à direita e, quando não cabe, o `right_x` é preso ao fim do grupo
    // da esquerda ⇒ os últimos agrupadores **saem pela borda direita**. É o comportamento declarado
    // no `topbar/mod.rs` (*«the rightmost clusters clip off the right edge»*), e a consequência é
    // que os pills **`Layers`, `Assets` e `Script` ficam inalcançáveis no tamanho de arranque**.
    //
    // ⚠️ Medido aqui com a rede toda: o hero está pintado, o `Undo` ESTÁ no hit-index, e o `Assets`
    // não. E `request_inner_size(1600×900)` foi tentado e **negado pelo compositor** — alargar não
    // é uma saída que um roteiro possa tomar.
    //
    // ⇒ o roteiro abre o painel pela outra porta e **continua**, porque o que ele existe para medir
    // é o MENU. ⛔ A metade que fica por cobrir está nomeada: *o pill pelo ponteiro*.
    let win = app.gfx.as_ref().map(|g| g.surface.size());
    eprintln!(
        "[asset-menu] f=20 ⛔ o pill `Assets` está FORA do ecrã a {win:?} (a barra transborda e o \
         grupo da direita sai pela borda) — o roteiro abre o painel pela outra porta e segue"
    );
    if let Some(hero) = app.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
        hero.set_panel_visible(ph2d_panel_asset_browser::PANEL_ID, true);
    }
}

/// ⭐⭐⭐ **Cria uma imagem pela porta do PRODUTO e conta se ela chega à biblioteca.**
///
/// ⛔⛔ Report do Enio (2026-08-30): *«as imagens não aparecem no painel nem importando nem criando
/// as imagens no app»*. O roteiro anterior só punha um PREFAB na cena, então a metade das texturas
/// nunca era exercida — *um instrumento que não encena o caso não mede a ausência dele*.
///
/// ⚠️ **O `spawn_blank_canvas` é o caminho do *New Image…*, e é o caso NORMAL:** ele empacota no
/// **átlas** (não em textura própria) e regista a proveniência no `atlas_asset_map`. Era
/// exactamente esta forma que a lei da etapa A não via.
fn new_image(app: &mut crate::App) {
    // ⚠️ A escala vem do PROJECTO, que é onde a única conversão px→m deste app vive.
    let ppm = app
        .gfx
        .as_ref()
        .and_then(|g| g.hero_screen.as_ref())
        .map_or(100.0, |h| h.project.pixels_per_meter);
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let cell = gfx.next_import_cell;
    gfx.next_import_cell += 1;
    let out = crate::image_import::spawn_blank_canvas(
        &mut gfx.sim,
        &mut gfx.renderer,
        &gfx.asset_db,
        cell,
        64,
        2, // fundo branco
        ph2d_core::Vec2::new(2.0, 0.0),
        ppm,
        &mut gfx.atlas_asset_map,
    );
    eprintln!(
        "[asset-menu] f=25 imagem nova pela porta do produto (célula {cell} do átlas): {}",
        match &out {
            Ok((name, _)) => format!("«{name}»"),
            Err(e) => format!("FALHOU: {e}"),
        }
    );
}

/// Quantos cartões a biblioteca publicou — a metade que o report do Enio mede.
fn report_library(_app: &mut crate::App) {
    let (n, kinds) = ph2d_panel_asset_browser::probe_index_summary();
    eprintln!("[asset-menu] f=28 a biblioteca tem {n} asset(s): {kinds}");
}

/// Aperta o chip **Prefab** da fileira de filtros — para o cartão 0 ser o prefab, quaisquer que
/// sejam os outros assets da biblioteca.
fn click_kind_chip(app: &mut crate::App) {
    // O chip `1` é o `AssetKind::ALL[0]` = Prefab (o `0` é o «All»).
    let id = ph2d_editor::ids::ASSET_KIND[1];
    match app.smoke_find_widget(id) {
        Some((x, y)) => {
            app.smoke_pointer_down(x, y);
            app.smoke_pointer_up();
            eprintln!("[asset-menu] f=29 chip de filtro `Prefab` apertado em ({x}, {y})");
        }
        None => eprintln!("[asset-menu] f=29 ⚠️ o chip `Prefab` NÃO está no hit-index"),
    }
}

/// Onde está o primeiro cartão. ⚠️ **Pelo hit-index**, que é a única prova de que ele é agarrável.
fn find_card(app: &mut crate::App) {
    let id = ph2d_editor::ids::asset_cell_id(0);
    match app.smoke_find_widget(id) {
        Some((x, y)) => {
            CARD_AT.with(|c| c.set((x, y)));
            eprintln!("[asset-menu] f=32 cartão 0 no hit-index em ({x}, {y})");
        }
        None => eprintln!(
            "[asset-menu] f=32 ⚠️ o cartão 0 NÃO está no hit-index — ou o painel não abriu, ou a \
             grade não pintou, ou as células não estão registadas"
        ),
    }
}

fn right_click_card(app: &mut crate::App) {
    let (x, y) = CARD_AT.with(std::cell::Cell::get);
    if x <= 0.0 && y <= 0.0 {
        return;
    }
    app.smoke_secondary_click(x, y);
    let open = app
        .gfx
        .as_ref()
        .and_then(|g| g.hero_screen.as_ref())
        .is_some_and(|h| h.store.context_menu().is_some());
    eprintln!(
        "[asset-menu] botão direito no cartão — menu aberto: {}",
        if open { "sim" } else { "NÃO" }
    );
}

fn click_menu_row(app: &mut crate::App, row: NodeId, label: &str) {
    match app.smoke_find_widget(row) {
        Some((x, y)) => {
            app.smoke_pointer_down(x, y);
            app.smoke_pointer_up();
            eprintln!("[asset-menu] `{label}` apertado em ({x}, {y})");
        }
        None => eprintln!(
            "[asset-menu] ⚠️ `{label}` NÃO está no hit-index — a linha do menu está pintada e \
             morta sob o dedo, que é o defeito que este roteiro existe para apanhar"
        ),
    }
}

fn report_selection(app: &mut crate::App) {
    let n = app
        .gfx
        .as_ref()
        .and_then(|g| g.hero_screen.as_ref())
        .map_or(0, |h| h.gizmo.iter_selected().count());
    eprintln!(
        "[asset-menu] f=42 `Select users` — {n} objecto(s) seleccionado(s) {}",
        if n >= 1 {
            "(esperado ≥ 1)"
        } else {
            "⚠️ NÃO"
        }
    );
}

fn report_removed(app: &mut crate::App) {
    let want = MASTER.with(std::cell::Cell::get);
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let still = {
        let mut q = gfx
            .sim
            .world_mut()
            .query_filtered::<&ph2d_ecs::StableId, bevy_ecs::prelude::With<ph2d_ecs::MasterRoot>>();
        q.iter(gfx.sim.world()).any(|s| s.0 == want)
    };
    eprintln!(
        "[asset-menu] f=52 `Remove from Library` — a receita {want} ainda está na biblioteca: {} \
         (esperado: nao)",
        if still { "⚠️ SIM" } else { "nao" }
    );
}

// ── ⛔ REPRODUÇÃO dos dois reports de 2026-08-30 (segunda ronda) ────────────────────────────────

/// **Report 1:** *«uma sprite que foi deletada do canvas não consegui deletar do painel»*.
/// **Report 2:** *«um prefab com cópia no canvas foi deletado do painel»*.
///
/// Os dois são o MESMO gesto — apagar da CENA — com resultados opostos na biblioteca. Este passo
/// mede os dois lado a lado, porque uma explicação que não os cobre aos dois é meia explicação.
pub(crate) fn repro(app: &mut crate::App) {
    let want_master = MASTER.with(std::cell::Cell::get);
    let (n0, k0) = ph2d_panel_asset_browser::probe_index_summary();
    eprintln!("[repro] antes de apagar: {n0} asset(s) — {k0}");

    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    // (a) apagar a CÓPIA do prefab — como o `HierDelete` faz (despawn com cascata).
    let copy = {
        let mut q = gfx
            .sim
            .world_mut()
            .query::<(ph2d_ecs::Entity, &ph2d_ecs::InstanceOf)>();
        q.iter(gfx.sim.world())
            .find(|(_, l)| l.master == want_master)
            .map(|(e, _)| e)
    };
    if let Some(c) = copy {
        gfx.sim.world_mut().despawn(c);
        eprintln!("[repro] cópia do prefab APAGADA da cena");
    } else {
        eprintln!("[repro] ⚠️ não achei a cópia do prefab");
    }
    // (b) apagar a SPRITE da imagem nova.
    let canvas = {
        let mut q = gfx
            .sim
            .world_mut()
            .query::<(ph2d_ecs::Entity, &ph2d_ecs::Name)>();
        q.iter(gfx.sim.world())
            .find(|(_, n)| n.0.starts_with("Canvas"))
            .map(|(e, _)| e)
    };
    if let Some(c) = canvas {
        gfx.sim.world_mut().despawn(c);
        eprintln!("[repro] sprite da imagem APAGADA da cena");
    }
    // A receita ainda existe no mundo?
    let alive = {
        let mut q = gfx
            .sim
            .world_mut()
            .query_filtered::<&ph2d_ecs::StableId, bevy_ecs::prelude::With<ph2d_ecs::MasterRoot>>();
        q.iter(gfx.sim.world()).any(|s| s.0 == want_master)
    };
    eprintln!("[repro] a receita {want_master} continua no MUNDO: {alive}");
}

/// ⭐⭐⭐ **Tira do painel a imagem cuja sprite foi apagada** — a cura do report, pela porta do
/// produto (o mesmo `AssetCardVerb` que o item do menu levanta).
///
/// ⚠️ Ele encena o caso EXACTO do report: a sprite já não existe, então o `Select users` conta `0`
/// e a 1.ª versão respondia *«mude esses 0 objectos para a tirar»*.
fn remove_the_unused_image(app: &mut crate::App) {
    let Some(id) = ph2d_panel_asset_browser::probe_first_texture() else {
        eprintln!("[repro] ⚠️ não há textura na biblioteca para tirar");
        return;
    };
    if let Some(hero) = app.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
        hero.bus
            .push(ph2d_editor::action_bus::EditorAction::AssetCardVerb {
                asset: ph2d_editor::interaction::drag_payload::DragPayload::Image { asset: id },
                verb: ph2d_editor::action_bus::AssetCardAction::RemoveFromLibrary,
            });
        eprintln!("[repro] `Remove from Library` pedido para a imagem sem utilizadores");
    }
}

/// O que a biblioteca passou a ter, um quadro depois.
pub(crate) fn repro_after(_app: &mut crate::App) {
    let (n, k) = ph2d_panel_asset_browser::probe_index_summary();
    eprintln!("[repro] DEPOIS de apagar: {n} asset(s) — {k}");
}
