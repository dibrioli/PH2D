//! ⭐⭐⭐ **A VARREDURA px/m de TODO número de mundo do Inspector** — report do dono, 27/10: com o
//! projecto em px os cartões mostravam `m`, `m/s` e `m/s2` fixos sobre o valor em metros.
//!
//! Por fixtura (uma por secção, as dos gates de cada secção), para cada id que a porta
//! (`unidade_mundo::guardada`) diz ser de mundo:
//!
//! 1. **a semente**: o número mostrado em px é o mostrado em metros × `ppm` (trocar o seletor com
//!    o mesmo objecto re-semeia);
//! 2. **a escrita**: `37` escrito em px chega ao barramento como `0,37 m` (a âncora, que guarda px,
//!    ao contrário: `0.37` em metros chega como `37`);
//! 3. **o censo**: a pintura em px passou pela porta e NADA lhe fugiu.
//!
//! E a régua inteira: todo candidato da porta é pintado em pelo menos uma fixtura.

use std::collections::BTreeSet;

use ph2d_editor_core::factory_edits::{InspectorFactory, InspectorFactoryInfo, InspectorLifecycle};
use ph2d_editor_core::nav_edits::NavAlvoModo;
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::unidade_mundo::{self, Grandeza};
use ph2d_panel_inspector::*;
use ph2d_ui_testkit::MockPanelHost;

const PPM: f32 = 100.0;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 30_000.0,
};

fn limpa() {
    set_current_inspector_particles(None);
    set_current_inspector_player(None);
    set_current_inspector_camera(None);
    set_current_inspector_physics(None);
    set_current_inspector_joint(None);
    set_current_inspector_wheel(None);
    set_current_inspector_ray(None);
    set_current_inspector_nav(None);
    set_current_inspector_audio(None);
    set_current_inspector_factory(None);
    set_current_inspector_projectile(None);
    set_current_inspector_luz(None);
    set_current_inspector_vida(None);
    set_current_inspector_parallax(None);
    set_current_inspector_path_follow(None);
    set_current_inspector_shake(None);
    set_current_inspector_emitter(None);
    set_current_inspector_topdown(None);
    set_current_inspector_slice(None);
    set_current_inspector_anchor(None);
    set_current_inspector_tween(None);
    ph2d_panel_inspector::set_current_inspector_anim(None);
}

fn fabrica() -> InspectorFactoryInfo {
    InspectorFactoryInfo {
        entity_bits: 0xFA_B1,
        factory: Some(InspectorFactory {
            spawn_where: ph2d_editor_core::factory_edits::InspectorSpawnWhere::Area,
            area: [3.5, 1.25],
            burst: 2,
            ..InspectorFactory::default()
        }),
        lifecycle: Some(InspectorLifecycle {
            lifetime_s: Some(2.0),
            on_death: String::new(),
            outside_margin: Some(0.75),
        }),
        is_spawned: false,
        has_game_camera: true,
        clock_playing: true,
        selected_count: 1,
    }
}

fn projectil() -> InspectorProjectileInfo {
    InspectorProjectileInfo {
        entity_bits: 0x9_1EC,
        initial_speed: 12.5,
        acceleration: 1.0,
        max_speed: 30.0,
        gravity: 2.0,
        bounciness: 0.5,
        max_bounces: 2,
        range: 7.25,
        face_velocity: true,
        homing_accel: 4.0,
        homing_target: String::new(),
        homing_target_missing: false,
        body_is_kinematic: true,
        has_body: true,
        clock_playing: true,
        flight_over: false,
        selected_count: 1,
    }
}

/// Um tween de POSIÇÃO: De/Para em metros.
fn tween_posicao() -> ph2d_editor_core::tween_edits::InspectorTweenInfo {
    ph2d_editor_core::tween_edits::InspectorTweenInfo {
        entity_bits: 0x7_3EE,
        rows: vec![ph2d_editor_core::tween_edits::InspectorTweenRow {
            canal: ph2d_tween::Canal::PositionX.tag(),
            de: [0.6, 0.0, 0.0, 0.0],
            para: [-1.25, 0.0, 0.0, 0.0],
            familia: 0,
            modo: 0,
            ao_acabar: ph2d_tween::AoAcabar::Hold.tag(),
            ciclo: ph2d_tween::Ciclo::Reinicia.tag(),
            duracao_us: Some(400_000),
            repeat: true,
            autostart: true,
            item: 0,
            relogio_item: 0,
            relogio_pos: 0,
        }],
        tem_sprite: true,
        selected_count: 1,
    }
}

/// O corpo dos gates da física com a forma `shape` e todo número de mundo não nulo.
fn corpo(shape: u8) -> ph2d_editor_core::screens::hero::InspectorPhysicsInfo {
    let mut i = crate::seam_physics::with_body();
    i.shape_tag = shape;
    i.offset = [0.25, -0.5];
    i.linvel = [1.5, -2.0];
    i.walk_belt = 0.75;
    i.radius = 0.4;
    i.cap_half_height = 0.6;
    i
}

/// `(nome, arma)`.
type Fixtura = (&'static str, Box<dyn Fn()>);

/// `(nome, arma)` — uma por secção (ou estado) que pinta números de mundo.
fn fixturas() -> Vec<Fixtura> {
    use crate::{
        a_faixa_media_do_dock_nao_corta_nomes as cam, a_seccao_barra_de_vida_esta_viva as barra,
        a_seccao_nav_custo_esta_viva as custo, a_seccao_nav_esta_viva as nav,
        a_seccao_vida_esta_viva as vida, o_losango_dos_campos_novos as junta,
    };
    vec![
        (
            "particles",
            Box::new(|| {
                let mut i = crate::a_seccao_particles_esta_viva::info();
                i.gravity = [1.5, -9.75];
                set_current_inspector_particles(Some(i));
            }),
        ),
        (
            "player",
            Box::new(|| {
                let mut p = crate::seam_player::player();
                p.takeoff_speed = 1.5;
                p.dash_speed = 14.0;
                set_current_inspector_player(Some(p));
            }),
        ),
        (
            "camera",
            Box::new(|| {
                let mut c = cam::camera();
                c.camera.offset = [0.5, -1.25];
                if let Some(f) = c.follow.as_mut() {
                    f.offset = [0.75, 1.5];
                }
                set_current_inspector_camera(Some(c));
            }),
        ),
        (
            "physics",
            Box::new(|| set_current_inspector_physics(Some(corpo(1)))),
        ),
        (
            "physics_bola",
            Box::new(|| set_current_inspector_physics(Some(corpo(0)))),
        ),
        (
            "physics_capsula",
            Box::new(|| set_current_inspector_physics(Some(corpo(2)))),
        ),
        (
            "physics_part",
            Box::new(|| {
                let mut p = crate::seam_physics::as_part();
                p.offset = [0.25, -0.5];
                p.walk_belt = 0.75;
                set_current_inspector_physics(Some(p));
            }),
        ),
        (
            "joint_slider",
            Box::new(|| set_current_inspector_joint(Some(junta::junta(4)))),
        ),
        (
            "joint_spring",
            Box::new(|| set_current_inspector_joint(Some(junta::junta(1)))),
        ),
        (
            "joint_rope",
            Box::new(|| set_current_inspector_joint(Some(junta::junta(2)))),
        ),
        (
            "tween_posicao",
            Box::new(|| set_current_inspector_tween(Some(tween_posicao()))),
        ),
        (
            "joint_custom",
            Box::new(|| {
                let mut j = junta::junta(8);
                j.axis_mode_tag = [1, 1, 1];
                set_current_inspector_joint(Some(j));
            }),
        ),
        (
            "joint_custom_alvo",
            Box::new(|| {
                let mut j = junta::junta(8);
                j.motor_mode_tag = 1;
                j.motor_target_ui = 0.5;
                set_current_inspector_joint(Some(j));
            }),
        ),
        (
            "wheel",
            Box::new(|| set_current_inspector_wheel(Some(crate::seam_wheel::differential()))),
        ),
        (
            "ray",
            Box::new(|| {
                set_current_inspector_ray(Some(crate::a_seccao_ray_sensor_esta_viva::olho()))
            }),
        ),
        (
            "nav",
            Box::new(|| {
                set_current_inspector_nav(Some(nav::info(
                    Some(nav::regiao()),
                    Some(nav::agente(NavAlvoModo::Ponto)),
                )));
            }),
        ),
        (
            "nav_custo",
            Box::new(|| {
                set_current_inspector_nav(Some(custo::info(
                    Some(custo::agente(true, true)),
                    None,
                    Some(custo::atalho(false)),
                )));
            }),
        ),
        (
            "audio",
            Box::new(|| {
                let mut f = crate::the_audio_section_is_alive::fonte();
                f.non_spatialized_radius = 1.5;
                set_current_inspector_audio(Some(crate::the_audio_section_is_alive::info(
                    Some(f),
                    false,
                    0,
                )));
            }),
        ),
        (
            "factory",
            Box::new(|| set_current_inspector_factory(Some(fabrica()))),
        ),
        (
            "projectile",
            Box::new(|| set_current_inspector_projectile(Some(projectil()))),
        ),
        (
            "luz",
            Box::new(|| set_current_inspector_luz(Some(crate::seam_luz::ponto()))),
        ),
        (
            "vida_barra",
            Box::new(|| {
                let mut i = barra::info(barra::barra());
                i.health = Some(barra::vida_com_numeros());
                set_current_inspector_vida(Some(i));
            }),
        ),
        (
            "vida_dano",
            Box::new(|| {
                set_current_inspector_vida(Some(vida::info(Some(vida::vida()), Some(vida::dano()))))
            }),
        ),
        (
            "parallax",
            Box::new(|| {
                set_current_inspector_parallax(Some(crate::a_seccao_parallax_esta_viva::camada(
                    true,
                )))
            }),
        ),
        (
            "path_follow",
            Box::new(|| {
                let mut i = crate::seam_path_follow::info();
                i.lado = 0.5;
                set_current_inspector_path_follow(Some(i));
            }),
        ),
        (
            "shake",
            Box::new(|| {
                set_current_inspector_shake(Some(crate::seam_shake::info_camera()));
                set_current_inspector_emitter(Some(crate::seam_shake::info_emissor()));
            }),
        ),
        (
            "topdown",
            Box::new(|| {
                set_current_inspector_topdown(Some(
                    crate::a_seccao_topdown_mostra_o_objecto::mover(),
                ))
            }),
        ),
        (
            "slice",
            Box::new(|| {
                let mut s = crate::inspector_regression_slice::slice();
                s.size = [2.5, 1.5];
                set_current_inspector_slice(Some(s));
            }),
        ),
        (
            "anchors",
            Box::new(|| {
                set_current_inspector_anchor(Some(crate::inspector_regression_anchors::info()))
            }),
        ),
        (
            "anim_personagem",
            Box::new(|| {
                let info =
                    crate::seam_anim_card::with_character(crate::seam_anim::anim(false, false));
                ph2d_panel_inspector::set_current_inspector_anim(Some(info));
            }),
        ),
    ]
}

fn pinta(h: &mut MockPanelHost, st: &mut InspectorState, d: DisplayUnit) {
    h.project_mut().display_unit = d;
    h.project_mut().pixels_per_meter = PPM;
    let _ = h.paint::<InspectorPanel>(st, VIEWPORT);
    h.open_all_sections();
    let _ = h.paint::<InspectorPanel>(st, VIEWPORT);
}

/// Todo número que um `Debug` de uma acção carrega.
fn numeros_em(texto: &str) -> Vec<f64> {
    texto
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e'))
        .filter_map(|t| t.parse::<f64>().ok())
        .collect()
}

/// Escreve `txt` em `id` e diz se alguma acção levou `alvo` (a unidade GUARDADA).
fn escreve_e_procura(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    id: ph2d_a11y::NodeId,
    txt: &str,
    alvo: f64,
) -> bool {
    let _ = h.drained_actions();
    for ev in h.type_into_number(id, txt) {
        let _ = h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    let acoes = format!("{:?}", h.drained_actions());
    numeros_em(&acoes).iter().any(|v| (v - alvo).abs() < 1e-4)
}

/// ⭐⭐⭐ **Com o projecto em px, todo número de mundo mostra `valor × ppm`, a escrita volta à
/// unidade guardada, e nenhum foge da porta.**
#[test]
fn todo_numero_de_mundo_obedece_a_px_metros_do_projecto() {
    let mut falhas: Vec<String> = Vec::new();
    let mut pintados: BTreeSet<ph2d_a11y::NodeId> = BTreeSet::new();
    let mut verificados = 0_usize;
    for (nome, arma) in fixturas() {
        limpa();
        arma();
        let mut h = MockPanelHost::with_panel::<InspectorPanel>();
        let mut st = InspectorState::default();
        pinta(&mut h, &mut st, DisplayUnit::Meters);
        let mundo: Vec<(ph2d_a11y::NodeId, Grandeza)> = unidade_mundo::candidatos()
            .into_iter()
            .filter_map(|id| unidade_mundo::guardada(id).map(|g| (id, g)))
            .collect();
        let em_m: Vec<Option<f64>> = mundo
            .iter()
            .map(|(id, _)| h.store().number_value(*id))
            .collect();
        unidade_mundo::arma_censo();
        pinta(&mut h, &mut st, DisplayUnit::Pixels);
        let censo = unidade_mundo::colhe_censo();
        for id in &censo.fugas {
            falhas.push(format!(
                "{nome}: {id:?} pintado fora da porta (unidade de mundo FIXA, ou SEM unidade)"
            ));
        }
        let pela_porta: BTreeSet<_> = censo.pela_porta.iter().copied().collect();
        for ((id, g), m) in mundo.iter().zip(em_m) {
            if !pela_porta.contains(id) {
                continue; // não pintado nesta fixtura — a régua do fim confere a cobertura
            }
            pintados.insert(*id);
            let p = h.store().number_value(*id);
            match (m, p) {
                (Some(m), Some(p)) if m != 0.0 => {
                    if (p - m * f64::from(PPM)).abs() > 1e-3 * m.abs().max(1.0) {
                        falhas.push(format!("{nome}: {id:?} {g:?} mostrou {p} px sobre {m} m"));
                    }
                }
                _ => falhas.push(format!(
                    "{nome}: {id:?} sem valor não nulo na fixtura ({m:?}/{p:?})"
                )),
            }
            let chegou = if *g == Grandeza::PixelsDoMundo {
                pinta(&mut h, &mut st, DisplayUnit::Meters);
                let r = escreve_e_procura(&mut h, &mut st, *id, "0.37", 37.0);
                pinta(&mut h, &mut st, DisplayUnit::Pixels);
                r
            } else {
                escreve_e_procura(&mut h, &mut st, *id, "37", 0.37)
            };
            if !chegou {
                falhas.push(format!(
                    "{nome}: {id:?} {g:?} — o escrito não chegou na unidade guardada"
                ));
            }
            verificados += 1;
        }
    }
    limpa();
    let faltam: Vec<_> = unidade_mundo::candidatos()
        .into_iter()
        .filter(|id| !pintados.contains(id))
        .collect();
    assert!(
        falhas.is_empty() && faltam.is_empty(),
        "{verificados} verificados; {} falhas:\n{}\nnunca pintados por fixtura nenhuma: {faltam:?}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// ⚠️ **A faixa e a taxa de arrasto viajam com o valor** — o raio da roldana tem `0..100 m`, passo
/// `0,01` e arrasto `0,01 m/px` (o `populate`); com o projecto em px o arrasto e o stepper leem
/// `0..10000`, passo `1` e `1 px/px`. Sem a porta das faixas o arrasto andava 0,01 px por pixel e o
/// tecto era `100 px` — e voltar a metros devolve os números do `populate`.
#[test]
fn a_faixa_de_um_numero_de_mundo_segue_o_seletor() {
    limpa();
    set_current_inspector_wheel(Some(crate::seam_wheel::differential()));
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    pinta(&mut h, &mut st, DisplayUnit::Pixels);
    let px = (
        h.store().number_range(ids::INSP_WHEEL_RADIUS),
        h.store().number_drag_rate(ids::INSP_WHEEL_RADIUS),
    );
    pinta(&mut h, &mut st, DisplayUnit::Meters);
    let m = (
        h.store().number_range(ids::INSP_WHEEL_RADIUS),
        h.store().number_drag_rate(ids::INSP_WHEEL_RADIUS),
    );
    limpa();
    let perto = |a: Option<(f64, f64, f64)>, b: (f64, f64, f64)| {
        a.is_some_and(|a| {
            (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-6 && (a.2 - b.2).abs() < 1e-9
        })
    };
    assert!(perto(px.0, (0.0, 10_000.0, 1.0)), "faixa em px: {:?}", px.0);
    assert!(
        px.1.is_some_and(|r| (r - 1.0).abs() < 1e-9),
        "arrasto em px: {:?}",
        px.1
    );
    assert!(
        perto(m.0, (0.0, 100.0, 0.01)),
        "CONTROLO em metros: {:?}",
        m.0
    );
    assert!(
        m.1.is_some_and(|r| (r - 0.01).abs() < 1e-12),
        "CONTROLO arrasto em metros: {:?}",
        m.1
    );
}

/// As secções que declaram uma unidade de mundo (`m`, `m/s`, `m/s2`) e que a varredura acima
/// cobre com uma fixtura — mais as duas que passam pela porta do cartão Constraints
/// (`sync_constraint::unidade_e_passo`): o Transform e o HUD.
const SECCOES_DE_MUNDO: [&str; 32] = [
    "anim_rules.rs",
    "audio.rs",
    "camera.rs",
    "factory.rs",
    "hud.rs",
    "joint.rs",
    "joint_custom.rs",
    "joint_kind_rows.rs",
    "lifecycle.rs",
    "luz.rs",
    "nav.rs",
    "nav_custo.rs",
    "parallax.rs",
    "particles.rs",
    "path_follow.rs",
    "physics.rs",
    "physics_body.rs",
    "physics_part.rs",
    "physics_rows.rs",
    "player_rows.rs",
    "projectile.rs",
    "ray.rs",
    "shake.rs",
    "shake_emitter.rs",
    "slice_nine.rs",
    "topdown.rs",
    "transform.rs",
    "tween_editor.rs",
    "vida_barra.rs",
    "vida_dano.rs",
    "vida_impacto.rs",
    "wheel.rs",
];

/// ⭐⭐ **O CENSO: nenhuma secção declara uma unidade de mundo fora da porta px/m.**
///
/// Uma secção que escreve `Unit::Meters`/`MetersPerSecond*` tem de (a) estar na lista que a
/// varredura cobre — um cartão NOVO com metros reprova aqui até ganhar fixtura e registo na porta
/// (`unidade_mundo`) — e (b) pintar os números só pelas portas da casa (`rows::fields_row` /
/// `num_row_unit`), que são as que passam pela porta; um pintor do `editor-core` chamado direto
/// mostraria o sufixo fixo sem o censo da pintura o ver.
#[test]
fn censo_nenhum_numero_de_mundo_foge_da_porta_px_m() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sections");
    let mut fora: Vec<String> = Vec::new();
    let mut vistos = 0_usize;
    let mut entradas: Vec<_> = std::fs::read_dir(&dir)
        .expect("src/sections")
        .flatten()
        .collect();
    entradas.sort_by_key(std::fs::DirEntry::file_name);
    for e in entradas {
        let nome = e.file_name().to_string_lossy().into_owned();
        let texto = std::fs::read_to_string(e.path()).unwrap_or_default();
        let de_mundo = [
            "Unit::Meters",
            "Unit::MetersPerSecond",
            "Unit::MetersPerSecondSquared",
        ]
        .iter()
        .any(|t| texto.contains(t));
        if !de_mundo {
            continue;
        }
        vistos += 1;
        if !SECCOES_DE_MUNDO.contains(&nome.as_str()) {
            fora.push(format!(
                "{nome}: declara uma unidade de mundo e a varredura não a cobre"
            ));
        }
        for porta in [
            "paint_fields_row",
            "paint_field_row",
            "paint_number_input_with_buffer",
        ] {
            if texto.contains(porta) && nome != "rows.rs" {
                fora.push(format!(
                    "{nome}: pinta números por `{porta}` direto, fora da porta"
                ));
            }
        }
    }
    let obsoletas: Vec<_> = SECCOES_DE_MUNDO
        .iter()
        .filter(|n| !std::fs::read_to_string(dir.join(n)).is_ok_and(|t| t.contains("Unit::Meters")))
        .collect();
    assert!(
        vistos >= 25,
        "o censo leu só {vistos} secções de mundo — a régua mudou de forma"
    );
    assert!(fora.is_empty(), "{}", fora.join("\n"));
    assert!(
        obsoletas.is_empty(),
        "entradas sem metros (tire-as da lista): {obsoletas:?}"
    );
}

/// O id de `ids::NOME` (ou `ids::NOME[k]`), lido da sua declaração em `src/ids/` — a mesma
/// `hash_node_id` do produto.
fn id_por_nome(fontes: &str, nome: &str) -> Option<ph2d_a11y::NodeId> {
    let (base, k) = match nome.split_once('[') {
        Some((b, r)) => (b, r.trim_end_matches(']').parse::<usize>().ok()?),
        None => (nome, usize::MAX),
    };
    let decl = format!("pub const {base}:");
    let resto = &fontes[fontes.find(&decl)? + decl.len()..];
    let resto = &resto[resto.find('=')?..];
    let corpo = &resto[..resto.find(';')?];
    let literais: Vec<&str> = corpo.split("hash_node_id(\"").skip(1).collect();
    let alvo = if k == usize::MAX {
        literais.first()?
    } else {
        literais.get(k)?
    };
    Some(ph2d_tool_registry::hash_node_id_runtime(
        alvo.split('"').next()?,
    ))
}

/// ⭐⭐ **O CENSO pela grandeza: nenhum número de mundo nasce SEM unidade.**
///
/// A grandeza de cada campo está escrita na faixa do `populate_*` (`// LITERAL-PX-OK: m/s²`), que
/// é onde quem cria o campo diz o que ele guarda. Toda faixa em `m`, `m/s` ou `m/s²` tem de estar
/// na porta (`unidade_mundo::candidatos`); e a porta reprova na pintura (`Censo::fugas`) o campo
/// registado pintado sem unidade — a metade que a varredura acima apanha.
///
/// ⚠️ Fora, declarado: forças (N), binários (N·m) e densidades (kg/m²) também levam o metro, mas o
/// seletor não tem unidade em px para eles — o censo lê só o que começa por comprimento.
#[test]
fn censo_nenhum_numero_de_mundo_nasce_sem_unidade() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut ids_fonte = String::new();
    for e in std::fs::read_dir(src.join("ids"))
        .expect("src/ids")
        .flatten()
    {
        ids_fonte.push_str(&std::fs::read_to_string(e.path()).unwrap_or_default());
    }
    let candidatos: BTreeSet<_> = unidade_mundo::candidatos().into_iter().collect();
    let mut lidos = 0_usize;
    let mut fora: Vec<String> = Vec::new();
    let mut entradas: Vec<_> = std::fs::read_dir(&src)
        .expect("src")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("populate"))
        .collect();
    entradas.sort_by_key(std::fs::DirEntry::file_name);
    for e in entradas {
        let nome = e.file_name().to_string_lossy().into_owned();
        for linha in std::fs::read_to_string(e.path())
            .unwrap_or_default()
            .lines()
        {
            let Some((codigo, unidade)) = linha.split_once("LITERAL-PX-OK:") else {
                continue;
            };
            let u = unidade.trim();
            let de_mundo = u == "m"
                || ["m/s", "m ", "m,", "m;", "metros", "meters", "metres"]
                    .iter()
                    .any(|p| u.starts_with(p));
            let Some(i) = codigo.find("INSP_") else {
                continue;
            };
            if !de_mundo {
                continue;
            }
            let fim = codigo[i..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '[' || c == ']'))
                .map_or(codigo.len(), |f| i + f);
            let id_nome = &codigo[i..fim];
            lidos += 1;
            match id_por_nome(&ids_fonte, id_nome) {
                Some(id) if candidatos.contains(&id) => {}
                Some(_) => fora.push(format!("{nome}: {id_nome} ({u}) fora da porta px/m")),
                None => fora.push(format!("{nome}: {id_nome} sem declaração em src/ids")),
            }
        }
    }
    assert!(
        lidos >= 100,
        "o censo leu só {lidos} faixas de mundo — a régua mudou de forma"
    );
    assert!(
        fora.is_empty(),
        "{} números de mundo sem unidade:\n{}",
        fora.len(),
        fora.join("\n")
    );
}
