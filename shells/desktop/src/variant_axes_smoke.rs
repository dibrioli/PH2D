//! ⭐⭐⭐ **OS EIXOS DE PROPRIEDADE, DIRIGIDOS PELO PONTEIRO** — `PH2D_BUILD_SMOKE=79`.
//!
//! # O que ele mede
//!
//! Uma família de quatro versões nomeadas `Size=…, State=…` tem de virar **DUAS fileiras** no
//! cartão do Inspector — uma por pergunta —, e um chip tem de mudar **exactamente um eixo**.
//!
//! ⭐⭐⭐ **E desde 2026-08-31 ele mede também o caso do report** (*«quando mudo o conteúdo entre
//! `{}` o inspector não muda»*): um objecto **solto** — chaves no nome, família nenhuma — tem de
//! ter cartão, e **reescrever as chaves tem de reescrever o cartão**.
//!
//! ⚠️ **A lei tem gates puros** (`variant_axes_tests`), e eles não alcançam nada do que falha na
//! prática: o chip é pintado? está no hit-index? o `populate` registou-o? o clique nasce e chega ao
//! `swap`? É a mesma metade que o roteiro do navegador de assets existe para medir, e a mesma que
//! já mordeu esta linha duas vezes.
//!
//! ⛔ Uma linha com `NÃO` é um defeito, mesmo que o app não estoure.

thread_local! {
    /// A cópia que o artista escolhe — os passos a jusante voltam a ela.
    static COPY: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// ⭐ **O objecto SOLTO** — o caso do report de 2026-08-31: chaves no nome, família nenhuma.
    static LONE: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O Inspector precisa de um quadro pintado antes de os chips existirem no hit-index.
        20 => report_axes(app, "ao escolher a cópia"),
        24 => click_axis_chip(app, 0, 1, "Size -> o segundo valor"),
        28 => report_axes(app, "depois do chip"),
        32 => click_axis_chip(app, 1, 1, "State -> o segundo valor"),
        36 => report_axes(app, "depois do segundo chip"),
        // ⭐ E o caso que motivou o cartão: um objecto que não é cópia de nada.
        40 => select_lone(app),
        44 => report_axes(app, "no objecto SOLTO"),
        // ⭐⭐⭐ **O gesto do report**: reescrever o que está entre chaves.
        48 => rename_lone(app, "Casa {Size=Big, State=Run, Tag=City}"),
        52 => report_axes(app, "depois de REESCREVER as chaves"),
        _ => {}
    }
}

/// A cena: quatro versões nomeadas por eixos, e uma cópia escolhida.
///
/// ⚠️ **A família nasce como o produto a faz** — instanciar e promover a cópia a receita —, e não
/// por marcar quatro mestres soltos: o parentesco lê-se dos ELOS, e quatro mestres irmãos na
/// hierarquia **não** são uma família aqui (é a diferença medida contra o sistema vetorial).
fn build(app: &mut crate::App) {
    let vec_entities = &mut app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let registry = crate::init::build_component_registry();

    // A base, com uma peça — sem peça não há mapa determinístico entre as versões.
    let base = gfx
        .sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Casa {Size=Small, State=Idle}"),
            ph2d_ecs::MasterRoot,
        ))
        .id();
    gfx.sim.world_mut().spawn((
        ph2d_ecs::Transform::IDENTITY,
        ph2d_ecs::Name::new("Body"),
        ph2d_render::Sprite::atlas(0, [64.0, 64.0], [1.0; 4]),
        ph2d_ecs::ChildOf(base),
    ));
    ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
    ph2d_ecs::assign_missing_root_order(gfx.sim.world_mut());
    ph2d_ecs::assign_master_pieces(gfx.sim.world_mut());

    let base_id = gfx
        .sim
        .world()
        .get::<ph2d_ecs::StableId>(base)
        .map_or(0, |s| s.0);

    // As outras três versões: instanciar a base e promover a cópia a receita.
    for name in [
        "Casa {Size=Small, State=Run}",
        "Casa {Size=Big, State=Idle}",
        "Casa {Size=Big, State=Run}",
    ] {
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
        };
        let Ok(copy) = crate::instantiate::instantiate_master(
            &mut gfx.sim,
            &registry,
            base,
            None,
            &mut docs,
            crate::instantiate::ArtLink::Own,
        ) else {
            eprintln!("[axes] f=3 ⚠️ não consegui instanciar «{name}»");
            continue;
        };
        gfx.sim
            .world_mut()
            .entity_mut(copy)
            .insert((ph2d_ecs::MasterRoot, ph2d_ecs::Name::new(name)));
        ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
        ph2d_ecs::assign_master_pieces(gfx.sim.world_mut());
    }

    // E a cópia que o artista escolhe — uma instância normal da base.
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut gfx.vec_scene,
        vec_entities,
    };
    let copy = crate::instantiate::instantiate_master(
        &mut gfx.sim,
        &registry,
        base,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    );
    ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
    let Ok(copy) = copy else {
        eprintln!("[axes] f=3 ⚠️ a cópia do artista não nasceu");
        return;
    };
    COPY.with(|c| c.set(copy.to_bits()));

    // ⭐ E o objecto SOLTO — sem `MasterRoot`, sem `InstanceOf`, só o nome com chaves.
    let lone = gfx
        .sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::from_translation(ph2d_core::Vec2::new(3.0, 0.0)),
            ph2d_ecs::Name::new("Casa {Size=Small, State=Idle}"),
            ph2d_render::Sprite::atlas(0, [64.0, 64.0], [1.0; 4]),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
    ph2d_ecs::assign_missing_root_order(gfx.sim.world_mut());
    LONE.with(|c| c.set(lone.to_bits()));
    if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.clear_all_selection();
        hero.gizmo.add_to_selection(copy.to_bits());
    }
    eprintln!("[axes] f=3 cena — base StableId={base_id}, 4 versões «Casa {{…}}», cópia escolhida");
    // ⭐ O que a HIERARQUIA mostra — a metade que os dois reports do Enio nomeiam. ⚠️ Ela lê o
    // rótulo pela MESMA porta que o pintor da linha usa; uma conta paralela aqui mediria esta
    // função em vez do produto.
    for full in [
        "Casa {Size=Small, State=Idle}",
        "Casa {Size=Small, State=Idle} (1)",
        "Casa",
    ] {
        let shown = ph2d_editor::screens::hero::variant_axes::row_label(full);
        eprintln!("[axes] f=3 a hierarquia mostra «{shown}»   (nome: «{full}»)");
    }
}

/// ⭐⭐ **Escolhe o objecto SOLTO** — ele declara `Size`/`State` no nome e não pertence a família
/// nenhuma, que é exactamente o estado em que o Inspector não lia as chaves de lado nenhum.
fn select_lone(app: &mut crate::App) {
    let bits = LONE.with(std::cell::Cell::get);
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.clear_all_selection();
        hero.gizmo.add_to_selection(bits);
    }
    eprintln!("[axes] f=40 escolhido o objecto SOLTO «Casa {{Size=Small, State=Idle}}»");
}

/// ⭐⭐⭐ **Reescreve o nome do objecto solto** — o gesto literal do report do Enio de 2026-08-31.
///
/// ⚠️ **Ele escreve no `Name` do ECS**, que é onde a Hierarquia e o Inspector gravam: medir isto
/// noutro sítio mediria esta função em vez do produto.
fn rename_lone(app: &mut crate::App, to: &str) {
    let bits = LONE.with(std::cell::Cell::get);
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    gfx.sim
        .world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(bits))
        .insert(ph2d_ecs::Name::new(to));
    eprintln!("[axes] f=48 o objecto solto passou a chamar-se «{to}»");
}

/// O que o cartão OFERECE agora — lido do modelo que ele pinta.
fn report_axes(_app: &mut crate::App, when: &str) {
    let Some(info) = ph2d_panel_inspector::probe_current_properties() else {
        eprintln!("[axes] {when}: ⚠️ o cartão de PROPRIEDADES não está publicado");
        return;
    };
    if info.rows.is_empty() {
        eprintln!("[axes] {when}: ⚠️ NENHUMA fileira — a família não foi derivada");
        return;
    }
    for ax in &info.rows {
        let opts: Vec<String> = ax
            .options
            .iter()
            .map(|o| {
                if o.current {
                    format!("[{}]", o.label)
                } else {
                    o.label.clone()
                }
            })
            .collect();
        eprintln!("[axes] {when}: {} -> {}", ax.name, opts.join(" "));
    }
    eprintln!("[axes] {when}: {} fileira(s)", info.rows.len());
}

/// Carrega no chip `(eixo, valor)` — **pelo ponteiro**, no rect que o cartão registou.
fn click_axis_chip(app: &mut crate::App, axis: usize, value: usize, what: &str) {
    let Some(&id) = ph2d_editor::ids::INSP_INSTANCE_AXIS_OPTION
        .get(axis)
        .and_then(|row| row.get(value))
    else {
        return;
    };
    match app.smoke_find_widget(id) {
        Some((x, y)) => {
            app.smoke_pointer_down(x, y);
            app.smoke_pointer_up();
            eprintln!("[axes] chip apertado ({what}) em ({x}, {y})");
        }
        None => eprintln!(
            "[axes] ⚠️ o chip ({what}) NÃO está no hit-index — ele foi pintado? foi registado?"
        ),
    }
}
