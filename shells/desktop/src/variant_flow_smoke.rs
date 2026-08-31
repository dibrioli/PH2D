//! ⭐⭐⭐ **O FLUXO INTEIRO DE CRIAR VARIAÇÕES, GESTO A GESTO** — `PH2D_BUILD_SMOKE=80`.
//!
//! # Porque ele existe
//!
//! Report do Enio (2026-08-31): *«parece que ainda não funciona. Me mostre o fluxo inteiro de
//! criar variações»*. ⚠️ **A segunda frase é o achado.** As chaves do nome DECLARAM propriedades;
//! elas **não criam** uma família. Uma família nasce dos ELOS — *Make Prefab* · *Instantiate* ·
//! *Make Prefab* outra vez sobre a cópia —, e dois objectos irmãos com chaves no nome não são
//! variantes um do outro, por mais parecidos que os nomes sejam.
//!
//! ⛔ **Isto imprime; não é um gate.** Ele corre os verbos pela MESMA porta que o menu drena
//! ([`crate::instance_verbs::drain`]) e, a cada passo, diz **o que o artista veria**: a voz do
//! toast, as linhas da Hierarquia (pelo rótulo derivado que o pintor usa) e as fileiras do cartão
//! de propriedades (pelo modelo que o pintor lê).
//!
//! *Se o fluxo tem um buraco, é aqui que ele aparece — com o nome do passo ao lado.*

thread_local! {
    /// A cópia viva com que o passo seguinte trabalha.
    static LIVE: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => step_1_build(app),
        8 => step_2_make_prefab(app),
        12 => report(app, "2. depois de *Make Prefab*"),
        16 => step_3_instantiate(app),
        20 => report(app, "3. depois de *Instantiate*"),
        24 => step_4_make_variant(app),
        28 => report(app, "4. depois de *Make Prefab* na cópia (= variante)"),
        32 => step_5_rename_variant(app),
        36 => report(app, "5. depois de renomear a variante"),
        _ => {}
    }
}

/// 1. Um objecto com o nome que DECLARA as propriedades.
fn step_1_build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else { return };
    let e = gfx
        .sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Casa {Size=Small}"),
            ph2d_render::Sprite::atlas(0, [64.0, 64.0], [1.0; 4]),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(gfx.sim.world_mut());
    ph2d_ecs::assign_missing_root_order(gfx.sim.world_mut());
    LIVE.with(|c| c.set(e.to_bits()));
    if let Some(hero) = gfx.hero_screen.as_mut() {
        hero.gizmo.clear_all_selection();
        hero.gizmo.add_to_selection(e.to_bits());
    }
    eprintln!("[fluxo] 1. nasceu «Casa {{Size=Small}}» — um objecto NORMAL, sem receita nenhuma");
}

fn step_2_make_prefab(app: &mut crate::App) {
    verb(app, crate::instance_verbs::Verb::Make, "Make Prefab");
}

fn step_3_instantiate(app: &mut crate::App) {
    verb(app, crate::instance_verbs::Verb::Place, "Instantiate");
}

fn step_4_make_variant(app: &mut crate::App) {
    verb(
        app,
        crate::instance_verbs::Verb::Make,
        "Make Prefab (na cópia)",
    );
}

/// 5. O gesto que FALTA a toda a gente: dar à variante o valor dela.
///
/// ⚠️ **Ele escreve no `Name` da RECEITA da variante**, que é o que o cartão lê — e é exactamente
/// o passo que a Hierarquia pode não oferecer, se a receita estiver escondida dela.
fn step_5_rename_variant(app: &mut crate::App) {
    let bits = LIVE.with(std::cell::Cell::get);
    let Some(gfx) = app.gfx.as_mut() else { return };
    let e = ph2d_ecs::Entity::from_bits(bits);
    let Some(root) = crate::instance_verbs::instance_root_of(&mut gfx.sim, e) else {
        eprintln!("[fluxo] 5. ⚠️ a cópia viva não tem raiz de instância — o passo 4 falhou");
        return;
    };
    let master_id = gfx
        .sim
        .world()
        .get::<ph2d_ecs::InstanceOf>(root)
        .map(|l| l.master);
    let Some(master_id) = master_id else {
        eprintln!("[fluxo] 5. ⚠️ a raiz não aponta receita nenhuma");
        return;
    };
    let target = {
        let mut q = gfx
            .sim
            .world_mut()
            .query::<(ph2d_ecs::Entity, &ph2d_ecs::StableId)>();
        q.iter(gfx.sim.world())
            .find(|(_, s)| s.0 == master_id)
            .map(|(e, _)| e)
    };
    match target {
        Some(m) => {
            gfx.sim
                .world_mut()
                .entity_mut(m)
                .insert(ph2d_ecs::Name::new("Casa {Size=Big}"));
            eprintln!("[fluxo] 5. a receita da variante passou a «Casa {{Size=Big}}»");
        }
        None => eprintln!("[fluxo] 5. ⚠️ não achei a entidade da receita"),
    }
}

/// Corre um verbo pela porta do menu e imprime a VOZ que o artista ouviria.
fn verb(app: &mut crate::App, v: crate::instance_verbs::Verb, name: &str) {
    let bits = LIVE.with(std::cell::Cell::get);
    let vec_entities = &mut app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else { return };
    let registry = crate::init::build_component_registry();
    let mut echo = crate::instance_sync::MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let mut select_out = None;
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut gfx.vec_scene,
        vec_entities,
    };
    let changed = crate::instance_verbs::drain(
        v,
        &mut gfx.sim,
        &registry,
        &mut echo,
        bits,
        &mut toasts,
        &mut docs,
        [1.5, 0.0],
        &mut select_out,
    );
    let voice: Vec<String> = toasts.iter().map(|t| t.message.clone()).collect();
    eprintln!(
        "[fluxo] *{name}* -> mudou={changed}  voz do app: {}",
        if voice.is_empty() {
            "(MUDO — isto e' um defeito)".to_string()
        } else {
            voice.join(" | ")
        }
    );
    if let Some(b) = select_out {
        LIVE.with(|c| c.set(b));
    }
    if let Some(hero) = gfx.hero_screen.as_mut()
        && let Some(b) = select_out
    {
        hero.gizmo.clear_all_selection();
        hero.gizmo.add_to_selection(b);
    }
}

/// O que o artista VÊ: as linhas da Hierarquia e as fileiras do cartão.
fn report(app: &mut crate::App, when: &str) {
    let Some(gfx) = app.gfx.as_mut() else { return };
    // ⚠️ Pelo MESMO rótulo derivado que o pintor da linha usa.
    let mut rows: Vec<String> = {
        let mut q = gfx.sim.world_mut().query::<&ph2d_ecs::Name>();
        q.iter(gfx.sim.world())
            .map(|n| ph2d_editor::screens::hero::variant_axes::row_label(n.as_str()))
            .collect()
    };
    rows.sort();
    eprintln!("[fluxo] {when}: objectos no mundo = {rows:?}");
    match ph2d_panel_inspector::probe_current_properties() {
        None => eprintln!("[fluxo] {when}: cartao PROPERTIES — ausente"),
        Some(info) => {
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
                let label = if ax.name.is_empty() {
                    "Variant"
                } else {
                    &ax.name
                };
                eprintln!(
                    "[fluxo] {when}: cartao PROPERTIES  {label}: {}",
                    opts.join(" ")
                );
            }
        }
    }
}
