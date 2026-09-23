//! ⭐⭐⭐ **A ORDEM DAS SECÇÕES DO INSPECTOR É A DA PALETA QUE AS ACRESCENTA.**
//!
//! ⛔⛔⛔ **Report do dono, 2026-09-21:** *«várias seções muito confusas e desorganizadas»*.
//! Medido pelo `y` PINTADO em 2026-09-22: o Inspector tem **38** secções, e as **22** opcionais
//! estavam na ordem em que foram CONSTRUÍDAS — a sequência de índices de família lia
//! `11,11,12,13,11,11,9,9,11,9,11,11,14,3,11,11,11,11,11,13,13,0`, e a `Tags`, que todo objecto
//! pode ter, era a **última**, a `14 785 px` do topo.
//!
//! ⭐⭐⭐ **A ordem não é escolhida: é DERIVADA.** O catálogo
//! ([`ph2d_component_desc::ComponentCategory::ALL`]) declara **16 famílias por ordem**, e é por
//! elas que a paleta do *Add Component* agrupa — *uma tabela com dois consumidores e só um a
//! lê-la*.
//!
//! ⚠️⚠️ **A `ids::LIVE_SECTIONS` NÃO pode ser a fonte desta ordem:** ela é um ÍNDICE DE
//! ARMAZENAMENTO (as notas por secção indexam-na por POSIÇÃO, e o cabeçalho dela di-lo por
//! escrito). *A ordem de ARMAZENAMENTO e a de LEITURA são duas coisas.*

use ph2d_component_desc::{ComponentCategory, catalog};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_ui_testkit::MockPanelHost;

use super::quantas_entradas_tem_cada_painel::{VIEWPORT, abre_tudo};

/// ⭐ **Que COMPONENTE cada secção opcional edita** — e a família sai do catálogo, nunca daqui.
///
/// ⚠️ **Só as OPCIONAIS.** As fixas (Name · Transform · Render Source · …) são pintadas por três
/// orquestradores com portas próprias (*«só existe se houver sprite»*), e a ordem delas tem uma
/// inversão DECLARADA no gate de baixo.
///
/// ⛔ A segunda coluna é **verificada contra o catálogo**: um nome que ele não conheça reprova.
const SECCAO_E_COMPONENTE: &[(&str, &str)] = &[
    ("insp_live_tags_section", "ph2d::ecs::Tags"),
    ("insp_live_particles_section", "ph2d::ecs::ParticleEmitter"),
    ("insp_live_anim_section", "ph2d::ecs::SpriteAnimations"),
    ("insp_live_anchor_section", "ph2d::ecs::NamedAnchorList"),
    ("insp_live_topdown_section", "ph2d::physics::TopDownPlayer"),
    (
        "insp_live_projectile_section",
        "ph2d::physics::ProjectileMotion",
    ),
    ("insp_live_ray_section", "ph2d::physics::RaySensor"),
    ("insp_live_timer_section", "ph2d::ecs::Timers"),
    ("insp_live_action_section", "ph2d::ecs::SignalActions"),
    ("insp_live_factory_section", "ph2d::ecs::Factory"),
    ("insp_live_lifecycle_section", "ph2d::ecs::Lifetime"),
    ("insp_live_pathfollow_section", "ph2d::ecs::PathFollow"),
    ("insp_live_weapon_section", "ph2d::ecs::WeaponFire"),
    ("insp_live_sm_section", "ph2d::ecs::StateMachine"),
    ("insp_live_hud_section", "ph2d::ecs::UiCanvas"),
    ("insp_live_seq_section", "ph2d::ecs::SequencePlayer"),
    ("insp_live_watch_section", "ph2d::ecs::CounterWatch"),
    ("insp_live_trigger_section", "ph2d::ecs::SignalOnAction"),
    ("insp_live_tween_section", "ph2d::ecs::Tweens"),
    ("insp_live_audio_section", "ph2d::ecs::AudioSource2D"),
    ("insp_live_camera_section", "ph2d::ecs::GameCamera"),
    ("insp_live_shake_section", "ph2d::ecs::CameraShake"),
    ("insp_live_emitter_section", "ph2d::ecs::ShakeEmitter"),
    ("insp_live_script_section", "ph2d::script::LuauScript"),
];

/// O índice da família na ordem que a paleta usa.
fn familia(canonico: &str) -> usize {
    let desc = catalog::desc_for(canonico)
        .unwrap_or_else(|| panic!("o catálogo não conhece {canonico:?} — a tabela envelheceu"));
    ComponentCategory::ALL
        .iter()
        .position(|c| *c == desc.category)
        .expect("toda categoria está na ALL")
}

/// As secções na ordem em que são PINTADAS.
fn ordem_pintada() -> Vec<String> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mapa = super::o_que_o_artista_nao_alcanca::nomes_de(
        &[
            "../ph2d-panel-inspector/src/ids",
            "../ph2d-editor-core/src/ids",
        ],
        400,
    );
    let mut out = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("inspector");
        super::o_inspector_armado::arma_tudo();
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        abre_tudo(host.store_mut());
        let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        let mut v: Vec<(f32, String)> = Vec::new();
        for (nid, r) in host.registos_da_ultima_pintura() {
            if let Some(slug) = mapa.get(&nid)
                && slug.ends_with("_section")
            {
                v.push((r.y, slug.clone()));
            }
        }
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        v.dedup_by(|a, b| a.1 == b.1);
        out = v.into_iter().map(|(_, n)| n).collect();
        super::o_inspector_armado::desarma_tudo();
    });
    out
}

/// ⛔ Piso de população — uma varredura que leia poucas secções aprova qualquer ordem.
const PISO: usize = 20;

/// ⭐⭐⭐ **AS SECÇÕES OPCIONAIS SAEM POR FAMÍLIA, NA ORDEM DA PALETA.**
///
/// **Mutação que deve sangrar:** trocar duas chamadas de família no
/// [`crate::paint_familias`] — por exemplo pôr o ÁUDIO antes da LÓGICA.
///
/// ⚠️ **MUTAÇÃO NOMEADA, com a medição: ele NÃO afirma a ordem DENTRO de uma família.** Trocar o
/// `paint_projectile_section` com o `paint_topdown_section` — dois blocos completos, os dois da
/// família FÍSICA — deixa este gate **VERDE** (medido 2026-09-22, população `3`). E é o desenho:
/// a ordem intra-família é a de CONSTRUÇÃO, que esta wave não mexeu, e a única fonte de que ela
/// poderia ser derivada é a própria cadeia de chamadas — *afirmá-la aqui seria copiar o código
/// sob teste para dentro do teste*.
///
/// ⇒ o que se compra com isso está dito: um artista que queira o `Ray` antes do `Projectile`
/// mexe numa linha e nenhum gate o impede. *O que os gates defendem é o AGRUPAMENTO, que é o que
/// o report do dono nomeia.*
#[test]
fn as_seccoes_opcionais_saem_por_familia_na_ordem_da_paleta() {
    let pintadas = ordem_pintada();
    assert!(
        pintadas.len() >= PISO,
        "a varredura leu {} cabeçalhos (piso {PISO}) — ela deixou de alcançar o painel, e uma \
         régua que lê pouco aprova qualquer ordem",
        pintadas.len()
    );
    let por_nome: std::collections::BTreeMap<&str, &str> =
        SECCAO_E_COMPONENTE.iter().copied().collect();
    let mut seq: Vec<(usize, &str)> = Vec::new();
    for nome in &pintadas {
        if let Some(canonico) = por_nome.get(nome.as_str()) {
            seq.push((familia(canonico), nome.as_str()));
        }
    }
    assert_eq!(
        seq.len(),
        SECCAO_E_COMPONENTE.len(),
        "a tabela declara {} secções opcionais e a pintura mostrou {} — ou uma secção nova não \
         entrou na tabela, ou uma da tabela deixou de ser pintada",
        SECCAO_E_COMPONENTE.len(),
        seq.len()
    );
    let fora: Vec<String> = seq
        .windows(2)
        .filter(|w| w[0].0 > w[1].0)
        .map(|w| {
            format!(
                "{} (família {}) vem ANTES de {} (família {})",
                w[0].1, w[0].0, w[1].1, w[1].0
            )
        })
        .collect();
    assert!(
        fora.is_empty(),
        "a ordem das secções opcionais deixou de seguir a da paleta:\n  {}\n\n\
         ⇒ a ordem é DERIVADA do `ComponentCategory::ALL`, que é por onde o *Add Component* \
         agrupa. ⛔ Não a corrija pela `ids::LIVE_SECTIONS`: aquela é um índice de ARMAZENAMENTO.",
        fora.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO — a régua tem de conseguir ACUSAR uma ordem errada.**
///
/// ⚠️ Sem ele, um `familia()` que devolvesse sempre `0` deixaria o gate acima verde sobre o painel
/// em ordem de construção, que é exactamente o estado de que o dono se queixou.
#[test]
fn a_regua_distingue_as_familias() {
    let tags = familia("ph2d::ecs::Tags");
    let logica = familia("ph2d::ecs::Timers");
    let script = familia("ph2d::script::LuauScript");
    assert!(
        tags < logica && logica < script,
        "a régua não separa IDENTIDADE ({tags}) de LÓGICA ({logica}) de SCRIPT ({script})"
    );
}

/// ⏳ **A inversão que FICA, com a medição — e ela é do bloco FIXO, não do opcional.**
///
/// As secções que todo objecto (ou toda sprite) tem saem por três orquestradores com portas
/// próprias, e ali a `Image` (família `5`) vem antes da `Ordering` (`3`) e da `Rendering` (`4`):
/// *Render Source · Sprite Sheet · 9-Slice* antes de *Ordering · Sampling · Material & Blend*.
///
/// ⚠️ **Ela não é desleixo, é uma TROCA por decidir:** agrupar o que é da IMAGEM é defensável para
/// quem trabalha numa sprite, e desfazê-la obriga a reescrever três orquestradores cujas portas
/// não são a família (*«só existe se houver sprite»*). ⇒ fica NOMEADA, com o gate a **exigir que
/// ela exista** — no dia em que alguém a desfizer, este teste reprova e a nota sai com ela.
#[test]
fn a_inversao_do_bloco_fixo_esta_nomeada() {
    let pintadas = ordem_pintada();
    let pos = |n: &str| pintadas.iter().position(|s| s == n);
    let (Some(imagem), Some(ordem)) = (
        pos("insp_live_render_section"),
        pos("insp_live_ordering_section"),
    ) else {
        panic!("as duas secções fixas deixaram de ser pintadas — esta nota mede o nada");
    };
    assert!(
        imagem < ordem,
        "a inversão do bloco FIXO foi desfeita (a IMAGEM já não vem antes da ORDERING) — \
         ⇒ apague esta nota e este teste, que existiam só para a declarar"
    );
}
