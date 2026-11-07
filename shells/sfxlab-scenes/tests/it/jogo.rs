//! Use in Game e Create sound: evento real nos rects pintados, efeito lido nos bytes do projeto e no disparo do jogo.

use crate::Mesa;
use bevy_ecs::world::World;
use ph2d_app_audio::audio_2d::{preview_target_with, update_with};
use ph2d_app_audio::scene::SceneAudio;
use ph2d_app_sfxlab::Alvo;
use ph2d_app_sfxlab::aprovar::EstadoDoAprovado;
use ph2d_audio::{AudioEngine, AudioFormat};
use ph2d_component_desc::ComponentSet;
use ph2d_core::Vec2;
use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::scene::save::{WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{
    AudioListener2D, AudioSource2D, Entity, Name, ProjectSounds, SimWorld, SoundEntry, SoundSet,
    StableId, Transform, TransformPropagationState, WorklistBuf, assign_missing_stable_ids,
    stable_id_of,
};
use ph2d_editor_core::IconId;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::action_bus::HierRequest;
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::screens::hero::{AudioFieldEdit, InspectorAudioInfo, InspectorAudioSource};
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_object_picker::Alvo as Janela;
use ph2d_panel_hierarchy::{HierarchyPanel, HierarchyState};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_audio};
use ph2d_panel_sfxlab::tabela::CAMPOS;
use ph2d_sfx_dsp::{TETO_DA_ESCUTA, limitar_escuta, render};
use ph2d_sfx_lab::aprovar::em_wav;
use ph2d_sfx_recipe::{
    Body, ENGINE_V1, Envelope, ModeLaw, Recipe, Source, Take, Takes, Tone, Wave,
};
use ph2d_ui_testkit::MockPanelHost;
use std::collections::BTreeMap;

fn caixa(w: &mut World) -> Entity {
    let e = w
        .spawn((
            Transform::from_translation(Vec2::new(1.0, 0.0)),
            Name::new("Crate"),
        ))
        .id();
    w.spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Name::new("Ears"),
        AudioListener2D,
    ));
    assign_missing_stable_ids(w);
    e
}

fn estoura() -> Recipe {
    let mut r = Recipe::new(3, 0.5);
    r.add(
        8.0,
        Source::Body(Body {
            law: ModeLaw::Bar,
            base_hz: 220.0,
            modes: 16,
            t60: 4.0,
            damping: 0.0,
            rolloff: 0.0,
            contact_ms: 1.0,
            noise: 0.0,
        }),
        Envelope::default(),
    );
    r
}

fn duas_camadas() -> Recipe {
    let mut r = Recipe::new(5, 0.25);
    for hz in [220.0, 880.0] {
        r.add(
            0.4,
            Source::Tone(Tone {
                wave: Wave::Sine,
                hz,
                hz_end: hz,
                glide: 0.0,
            }),
            Envelope::default(),
        );
    }
    r
}

/// Dois cliques: o botão, e a linha do objeto na janela.
fn usa_pela_lista(m: &mut Mesa, aba: u64, objeto: Entity) {
    m.clica_na_aba(aba, Alvo::UseInGame);
    m.quadro();
    let id = stable_id_of(&m.mundo, objeto).expect("id").0;
    m.clica_na_aba(aba, Alvo::Janela(Janela::Objeto(id)));
    m.quadro();
}

fn bytes_do(m: &Mesa, som: &str) -> Vec<u8> {
    m.mundo
        .resource::<ProjectSounds>()
        .get(som)
        .unwrap_or_else(|| panic!("{som} não está no projeto"))
        .bytes
        .to_vec()
}

fn som_de(m: &Mesa, objeto: Entity) -> String {
    m.mundo
        .get::<AudioSource2D>(objeto)
        .expect("o objeto não tem fonte")
        .sound
        .clone()
}

fn arquivo(r: &Recipe) -> Vec<f32> {
    render(r, r.seed, ENGINE_V1, &Takes::default())
        .unwrap()
        .samples
}

#[test]
fn limitador_nunca_passa_do_teto_na_escuta_e_nao_entra_no_arquivo() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let r = estoura();
    let cru = arquivo(&r);
    assert!(
        cru.iter().any(|x| x.abs() > 1.0),
        "a fixture não estoura: não mede o limitador"
    );
    let id = m.abre("Boom", r);
    usa_pela_lista(&mut m, id, e);
    let bytes = bytes_do(&m, &som_de(&m, e));
    assert_eq!(bytes, em_wav(cru.clone()), "o arquivo não é o render cru");
    let limitado = limitar_escuta(&cru, ph2d_sfx_recipe::SAMPLE_RATE);
    assert_ne!(bytes, em_wav(limitado), "o limitador entrou no arquivo");
    let escuta = m.docs().escuta(id).expect("escuta").to_vec();
    let pico = escuta.iter().fold(0.0_f32, |a, x| a.max(x.abs()));
    assert!(pico <= TETO_DA_ESCUTA, "a escuta passou do teto: {pico}");
    assert!(
        m.docs().medida(id).expect("medida").clip(),
        "o estouro não acende o aviso"
    );
}

#[test]
fn aprovado_nao_muda_quando_o_motor_muda() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    assert_eq!(
        m.docs().estado_do_aprovado(id),
        Some(EstadoDoAprovado::Aprovado)
    );
    let som = som_de(&m, e);
    let antes = bytes_do(&m, &som);

    let velho = ENGINE_V1 - 1;
    m.lib_mut()
        .recipes
        .get_mut("Hit")
        .and_then(|r| r.approved.as_mut())
        .expect("aprovado")
        .engine = velho;
    m.espera(id);
    assert_eq!(
        m.docs().estado_do_aprovado(id),
        Some(EstadoDoAprovado::MotorVelho),
        "o aviso não apareceu"
    );
    assert_eq!(bytes_do(&m, &som), antes, "o som mudou sozinho");
    assert_eq!(som_de(&m, e), som, "o Play deixou de tocar o aprovado");
    assert_eq!(
        m.lib_mut().recipes["Hit"].approved.as_ref().unwrap().engine,
        velho,
        "o aviso sumiu sem o artista reaprovar"
    );
}

#[test]
fn mudar_a_receita_avisa_e_nao_toca_no_som_e_voltar_apaga_o_aviso() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    let som = som_de(&m, e);
    let antes = bytes_do(&m, &som);
    let camada = m.receita(id).layers[0].id;
    m.clica_na_aba(id, Alvo::Mudo(camada));
    m.espera(id);
    assert_eq!(
        m.docs().estado_do_aprovado(id),
        Some(EstadoDoAprovado::Mudou)
    );
    assert_eq!(bytes_do(&m, &som), antes);
    // Desfazer na aba devolve a receita, e o aprovado continua lá.
    use ph2d_editor_core::doc_surface::DocumentSurface;
    assert!(m.docs().undo(id, false));
    m.espera(id);
    assert_eq!(
        m.docs().estado_do_aprovado(id),
        Some(EstadoDoAprovado::Aprovado),
        "desfazer desaprovou"
    );
}

fn fonte_do_cartao() -> InspectorAudioSource {
    InspectorAudioSource {
        sound: String::new(),
        volume_db: 0.0,
        pitch: 1.0,
        looping: false,
        autoplay: true,
        max_distance: 10.0,
        attenuation: 1.0,
        non_spatialized_radius: 0.0,
        panning_strength: 1.0,
        max_polyphony: 1,
        bus_tag: 0,
        file_missing: false,
        reachable_by_signal: false,
        set: Default::default(),
    }
}

/// O clique real no `Create sound` do cartão, até o barramento que a shell drena.
fn clica_create_sound(bits: u64) {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_audio(Some(InspectorAudioInfo {
        entity_bits: bits,
        source: Some(fonte_do_cartao()),
        is_listener: false,
        listener_count: 1,
        is_active_listener: false,
        bus_labels: vec!["SFX".into(); 4],
        selected_count: 1,
    }));
    let janela = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let rects = h.paint::<InspectorPanel>(&mut st, janela);
    let id = ph2d_panel_inspector::ids::INSP_AUDIO_CREATE_SOUND;
    let (_, r) = rects
        .iter()
        .find(|(n, _)| *n == id)
        .copied()
        .expect("Create sound não foi pintado");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let ev = |kind, t| PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    };
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(ev(PointerKind::Down, 1_000_000_000));
    evs.extend(h.dispatch_pointer_event(ev(PointerKind::Up, 1_010_000_000)));
    for e in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, e);
    }
    let acoes = h.drained_actions();
    set_current_inspector_audio(None);
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorAudioEdit {
                entity_bits,
                edit: AudioFieldEdit::CreateSound,
            } if *entity_bits == bits
        )),
        "o clique não chegou ao barramento: {acoes:?}"
    );
}

#[test]
fn quatro_cliques_ate_ouvir_no_jogo() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    m.mundo.entity_mut(e).insert(AudioSource2D {
        autoplay: true,
        ..AudioSource2D::default()
    });
    let mut cliques = 0;

    clica_create_sound(e.to_bits());
    cliques += 1;
    // A linha da shell, em `fase_inspector_commits_audio.rs`.
    ph2d_app_sfxlab::pede_som_para(&mut m.mundo, e);
    m.quadro();
    m.quadro();
    let aba = m.activa();
    m.espera(aba);
    m.assenta();

    usa_pela_lista(&mut m, aba, e);
    cliques += 2;
    assert!(!som_de(&m, e).is_empty(), "o objeto não recebeu o som");

    cliques += 1;
    corre_e_ouve(&mut m);
    assert_eq!(cliques, 4);
}

/// O Play: o que a corrida faz com o mundo, e o que sai do mixer.
fn corre_e_ouve(m: &mut Mesa) {
    let mut sim = SimWorld::default();
    *sim.world_mut() = std::mem::take(&mut m.mundo);
    sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
    let mut livro = SceneAudio::default();
    let (mut motor, mut mixer) = AudioEngine::new(AudioFormat::stereo(48_000));
    let r = update_with(&mut sim, &mut livro, &mut motor);
    assert_eq!(r.started, 1, "o som não disparou no Play: {r:?}");
    assert_eq!(livro.live_voices(), 1);
    let mut saida = vec![0.0_f32; 8192];
    mixer.render(&mut saida, 4096);
    assert_eq!(mixer.active_voices(), 1);
    let pico = saida.iter().fold(0.0_f32, |a, x| a.max(x.abs()));
    assert!(pico > 0.01, "o disparo foi mudo: pico {pico}");
}

#[test]
fn objeto_sem_fonte_ganha_uma_que_toca_no_play() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    assert!(m.mundo.get::<AudioSource2D>(e).is_some(), "sem fonte");
    corre_e_ouve(&mut m);
}

/// Muito além dos dez metros do padrão da fonte.
fn caixa_longe(w: &mut World) -> Entity {
    let e = caixa(w);
    w.entity_mut(e)
        .insert(Transform::from_translation(Vec2::new(500.0, 0.0)));
    e
}

/// A linha da shell depois do clique em Create sound; a aba nova, já assentada.
fn cria_som_para(m: &mut Mesa, e: Entity) -> u64 {
    ph2d_app_sfxlab::pede_som_para(&mut m.mundo, e);
    m.quadro();
    m.quadro();
    let aba = m.activa();
    m.espera(aba);
    m.assenta();
    aba
}

fn confere_o_alcance_e_ouve(m: &mut Mesa, e: Entity) {
    let f = m.mundo.get::<AudioSource2D>(e).expect("sem fonte").clone();
    corre_e_ouve(m);
    assert_eq!(f.max_distance, ph2d_ecs::AUDIO_MAX_DISTANCE_M);
    assert_eq!(f.attenuation, 0.0);
}

#[test]
fn objeto_longe_do_ouvinte_ouve_o_som_criado() {
    let mut m = Mesa::nova();
    let e = caixa_longe(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    confere_o_alcance_e_ouve(&mut m, e);

    let mut m = Mesa::nova();
    let e = caixa_longe(&mut m.mundo);
    let aba = cria_som_para(&mut m, e);
    usa_pela_lista(&mut m, aba, e);
    confere_o_alcance_e_ouve(&mut m, e);
}

#[test]
fn fonte_que_ja_existe_fica_com_o_alcance_dela() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    m.mundo.entity_mut(e).insert(AudioSource2D {
        max_distance: 7.0,
        attenuation: 2.0,
        ..AudioSource2D::default()
    });
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    let f = m.mundo.get::<AudioSource2D>(e).expect("sem fonte");
    assert_eq!((f.max_distance, f.attenuation), (7.0, 2.0));
    assert!(!f.autoplay, "mudou o Autoplay do artista");
    assert!(!f.sound.is_empty(), "o som não chegou");
}

#[test]
fn o_som_criado_leva_o_nome_do_objeto() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let aba = cria_som_para(&mut m, e);
    assert_eq!(m.docs().nome(aba), Some("Crate Sound"));
    usa_pela_lista(&mut m, aba, e);
    assert_eq!(som_de(&m, e), "Crate Sound.wav");

    let segunda = cria_som_para(&mut m, e);
    assert_eq!(
        m.docs().nome(segunda),
        Some("Crate Sound 2"),
        "nome repetido"
    );
    let solta = m.som_novo();
    assert_eq!(m.docs().nome(solta), Some("Sound 1"), "o New Sound mudou");
}

/// O passo de desfazer guarda a receita de antes, sem o aprovado: voltar não pode apagá-lo.
#[test]
fn desfazer_depois_de_aprovar_nao_desaprova() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    let camada = m.receita(id).layers[0].id;
    m.clica_na_aba(id, Alvo::Mudo(camada));
    m.espera(id);
    usa_pela_lista(&mut m, id, e);
    let aprovado = m.receita(id).approved.clone();
    assert!(aprovado.is_some());
    use ph2d_editor_core::doc_surface::DocumentSurface;
    assert!(m.docs().undo(id, false));
    m.espera(id);
    assert_eq!(m.receita(id).approved, aprovado, "desfazer desaprovou");
    assert_eq!(
        m.docs().estado_do_aprovado(id),
        Some(EstadoDoAprovado::Mudou)
    );
}

#[test]
fn com_conjunto_o_som_entra_na_lista() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    m.mundo.entity_mut(e).insert((
        AudioSource2D::default(),
        SoundSet {
            sounds: vec![SoundEntry::new("outro.wav")],
            ..SoundSet::default()
        },
    ));
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    let lista = m.mundo.get::<SoundSet>(e).expect("conjunto");
    let nomes: Vec<&str> = lista.sounds.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(nomes, ["outro.wav", "Hit.wav"]);
    assert!(som_de(&m, e).is_empty(), "escreveu no campo que ninguém lê");
}

#[test]
fn objeto_que_sumiu_avisa_na_janela() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    m.mundo.entity_mut(e).insert(AudioSource2D::default());
    ph2d_app_sfxlab::pede_som_para(&mut m.mundo, e);
    m.quadro();
    m.quadro();
    let aba = m.activa();
    m.espera(aba);
    m.mundo.despawn(e);
    m.clica_na_aba(aba, Alvo::UseInGame);
    m.quadro();
    assert!(
        m.docs().rect_de(aba, Alvo::Janela(Janela::Busca)).is_some(),
        "a janela não abriu"
    );
    assert!(m.docs().objeto_sumiu(aba), "a tela não diz por quê");
    assert!(
        !m.mundo
            .get_resource::<ProjectSounds>()
            .is_some_and(|s| !s.is_empty()),
        "aprovou sem objeto"
    );
}

#[test]
fn use_in_game_com_solo_grava_o_som_inteiro() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let r = duas_camadas();
    let inteiro = em_wav(arquivo(&r));
    let so_a_segunda = em_wav(arquivo(&r.solo(r.layers[1].id)));
    assert_ne!(inteiro, so_a_segunda, "a fixture não distingue o solo");
    let id = m.abre("Hit", r.clone());
    m.clica_na_aba(id, Alvo::Solo(r.layers[1].id));
    m.espera(id);
    assert_eq!(m.docs().solo(id), Some(r.layers[1].id));
    usa_pela_lista(&mut m, id, e);
    assert_eq!(bytes_do(&m, &som_de(&m, e)), inteiro, "o solo entrou");
}

#[test]
fn re_aprovar_substitui_o_mesmo_som() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    let som = som_de(&m, e);
    let antes = bytes_do(&m, &som);
    let n = m.mundo.resource::<ProjectSounds>().len();

    let camada = m.receita(id).layers[0].id;
    m.clica_na_aba(id, Alvo::Mudo(camada));
    m.espera(id);
    assert_eq!(
        m.docs().estado_do_aprovado(id),
        Some(EstadoDoAprovado::Mudou)
    );
    usa_pela_lista(&mut m, id, e);

    assert_eq!(som_de(&m, e), som, "o nome mudou");
    assert_eq!(m.mundo.resource::<ProjectSounds>().len(), n, "som a mais");
    let depois = bytes_do(&m, &som);
    assert_ne!(depois, antes, "não substituiu");
    assert_eq!(
        m.docs().estado_do_aprovado(id),
        Some(EstadoDoAprovado::Aprovado)
    );
    assert_eq!(
        m.lib_mut().recipes["Hit"].approved.as_ref().unwrap().blake3,
        ProjectSounds::id_of(&depois)
    );
}

/// Reaprovar uma receita não pode mudar a de outra, mesmo quando as duas soam igual.
#[test]
fn duas_receitas_iguais_nao_dividem_o_som() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let a = m.abre("A", duas_camadas());
    usa_pela_lista(&mut m, a, e);
    let do_a = som_de(&m, e);
    let b = m.abre("B", duas_camadas());
    usa_pela_lista(&mut m, b, e);
    let do_b = som_de(&m, e);
    assert_ne!(do_a, do_b, "as duas receitas dividem o mesmo som");
}

fn fotografa(w: &mut World, reg: &ComponentRegistry) -> WorldSnapshot {
    let mut snap = WorldSnapshot::new();
    let mut prop = TransformPropagationState::new(w);
    let mut lista = WorklistBuf::default();
    world_to_snapshot(w, &mut prop, &mut lista, reg, &mut snap).expect("captura");
    snap
}

fn por_id(w: &mut World, id: StableId) -> Option<AudioSource2D> {
    w.query::<(&StableId, &AudioSource2D)>()
        .iter(w)
        .find(|(s, _)| **s == id)
        .map(|(_, f)| f.clone())
}

#[test]
fn desfazer_use_in_game_devolve_a_ligacao() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let mut m = Mesa::nova();
    let com_fonte = caixa(&mut m.mundo);
    m.mundo
        .entity_mut(com_fonte)
        .insert(AudioSource2D::default());
    let sem_fonte = m.mundo.spawn((Transform::IDENTITY, Name::new("Bare"))).id();
    assign_missing_stable_ids(&mut m.mundo);
    let (id1, id2) = (
        stable_id_of(&m.mundo, com_fonte).unwrap(),
        stable_id_of(&m.mundo, sem_fonte).unwrap(),
    );
    let aba = m.abre("Hit", duas_camadas());
    let antes = fotografa(&mut m.mundo, &reg);

    usa_pela_lista(&mut m, aba, com_fonte);
    let aba2 = m.abre("Hit2", outra_receita());
    usa_pela_lista(&mut m, aba2, sem_fonte);
    let depois = fotografa(&mut m.mundo, &reg);
    assert_ne!(antes, depois, "a fila da cena não veria passo nenhum");

    // O que a fila de undo faz: mundo limpo, fotografia de volta. O som continua no projeto.
    let editaveis: Vec<Entity> = m
        .mundo
        .query_filtered::<Entity, bevy_ecs::query::With<Transform>>()
        .iter(&m.mundo)
        .collect();
    for e in editaveis {
        m.mundo.despawn(e);
    }
    snapshot_to_world(&mut m.mundo, &antes, &reg).expect("repõe");
    assert_eq!(
        por_id(&mut m.mundo, id1).expect("a fonte sumiu").sound,
        "",
        "a ligação não voltou"
    );
    assert!(por_id(&mut m.mundo, id2).is_none(), "a fonte nova ficou");
    assert!(m.mundo.resource::<ProjectSounds>().get("Hit.wav").is_some());
}

fn outra_receita() -> Recipe {
    let mut r = duas_camadas();
    r.seed = 9;
    r.seconds = 0.2;
    r
}

#[test]
fn preview_sem_transform_toca() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    m.mundo.entity_mut(e).remove::<Transform>();

    let mut sim = SimWorld::default();
    *sim.world_mut() = std::mem::take(&mut m.mundo);
    let mut livro = SceneAudio::default();
    let (mut motor, mut mixer) = AudioEngine::new(AudioFormat::stereo(48_000));
    assert!(
        preview_target_with(&mut sim, &mut livro, &mut motor, e),
        "o Preview ficou mudo"
    );
    let mut saida = vec![0.0_f32; 2048];
    mixer.render(&mut saida, 1024);
    assert!(saida.iter().any(|x| x.abs() > 0.01), "a voz foi muda");

    // O Play do jogo continua a exigir a posição: o disparo por sinal não mudou.
    let (mut motor2, _m2) = AudioEngine::new(AudioFormat::stereo(48_000));
    let mut livro2 = SceneAudio::default();
    assert!(
        !ph2d_app_audio::audio_2d::play_target_with(&mut sim, &mut livro2, &mut motor2, e),
        "o caminho do jogo mudou"
    );
}

#[test]
fn as_camadas_novas_nascem_com_a_curva_de_ataque_e_a_gravacao_com_volume_constante() {
    let mut m = Mesa::nova();
    let id = m.som_novo();
    m.espera(id);
    m.assenta();
    let esperado = Envelope::hit(0.001, 0.4, 0.01);
    assert_eq!(m.receita(id).layers[0].envelope, esperado, "Struck body");
    for f in [ph2d_app_sfxlab::Fonte::Noise, ph2d_app_sfxlab::Fonte::Tone] {
        m.clica_na_aba(id, Alvo::Add);
        m.quadro();
        m.clica_na_aba(id, Alvo::Item(f));
        m.quadro();
    }
    let r = m.receita(id);
    assert!(r.layers.iter().all(|l| l.envelope == esperado));
    assert_eq!(r.layers.len(), 3);
    // Com a curva de ataque, o cartão Shape abre com os três campos.
    let decay = CAMPOS
        .iter()
        .find(|c| c.chave == "sfxlab.card.decay_ms")
        .expect("Decay");
    assert!(decay.le(&r, r.layers.first()).is_some());

    m.lib_mut().takes.add(Take::new(48_000, 1, vec![0.1; 480]));
    m.quadro();
    m.clica_na_aba(id, Alvo::Add);
    m.quadro();
    m.clica_na_aba(id, Alvo::Item(ph2d_app_sfxlab::Fonte::Recording));
    m.quadro();
    let r = m.receita(id);
    assert_eq!(r.layers.len(), 4, "a gravação não entrou");
    assert_eq!(r.layers[3].envelope, Envelope::default());
}

/// Os blobs do arquivo: que o `ProjectFile` os atravessa byte a byte é `as_receitas_atravessam_o_arquivo_byte_a_byte`.
#[test]
fn projeto_salvo_e_aberto_mantem_o_aprovado() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    usa_pela_lista(&mut m, id, e);
    let som = som_de(&m, e);
    let bytes = bytes_do(&m, &som);
    let aprovado = m.receita(id).approved.clone().expect("aprovado");

    let blob_lab = ph2d_sfx_recipe::project::save(&m.mundo);
    let blob_sons = m.mundo.resource::<ProjectSounds>().to_bytes();
    assert_eq!(blob_lab[8], 3, "o blob não é o v3");

    let mut aberta = Mesa::nova();
    let lib = ph2d_sfx_recipe::project::open(&blob_lab).expect("blob");
    ph2d_sfx_recipe::project::install(&mut aberta.mundo, lib);
    aberta
        .mundo
        .insert_resource(ProjectSounds::from_bytes(&blob_sons).expect("sons"));
    aberta.quadro();
    assert!(
        aberta
            .hero
            .apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
                ph2d_app_sfxlab::cmd_abrir("Hit")
            ))
    );
    aberta.quadro();
    aberta.quadro();
    let aba = aberta.activa();
    aberta.espera(aba);
    assert_eq!(aberta.receita(aba).approved.as_ref(), Some(&aprovado));
    assert_eq!(bytes_do(&aberta, &som), bytes);
    assert_eq!(ProjectSounds::id_of(&bytes), aprovado.blake3);
    assert_eq!(
        aberta.docs().estado_do_aprovado(aba),
        Some(EstadoDoAprovado::Aprovado),
        "abrir o projeto mudou o estado"
    );
}

// ── A janela do Use in Game ──

fn abre_a_janela(m: &mut Mesa, aba: u64) {
    m.clica_na_aba(aba, Alvo::UseInGame);
    m.quadro();
    assert!(
        m.docs().rect_de(aba, Alvo::Janela(Janela::Busca)).is_some(),
        "a janela não abriu"
    );
}

fn tecla_da_janela(m: &mut Mesa, key: BoardKey, text: Option<&str>) -> bool {
    let out = board_keys::key(
        &mut m.hero,
        &mut m.ts,
        key,
        Modifiers::default(),
        true,
        text,
        None,
    );
    m.quadro();
    out.consumed
}

/// O objeto ligado pelo Create sound continua a passar pela janela, já destacado: Enter confirma.
#[test]
fn use_in_game_sempre_abre_a_janela_e_enter_confirma_o_objeto_de_antes() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let aba = cria_som_para(&mut m, e);
    abre_a_janela(&mut m, aba);
    assert!(
        som_de_opcional(&m, e).is_empty(),
        "ligou antes de confirmar"
    );
    assert!(tecla_da_janela(&mut m, BoardKey::Enter, None));
    m.quadro();
    assert_eq!(som_de(&m, e), "Crate Sound.wav");
    assert!(
        m.docs().rect_de(aba, Alvo::Janela(Janela::Busca)).is_none(),
        "ficou aberta"
    );
}

fn som_de_opcional(m: &Mesa, e: Entity) -> String {
    m.mundo
        .get::<AudioSource2D>(e)
        .map(|f| f.sound.clone())
        .unwrap_or_default()
}

/// Com a janela aberta, Espaço escreve na busca: não toca o som, e o Delete não apaga a camada.
#[test]
fn escrever_na_busca_nao_chega_ao_som_por_baixo() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    abre_a_janela(&mut m, id);
    let antes = m.receita(id).clone();
    assert!(tecla_da_janela(&mut m, BoardKey::Space, None));
    assert!(tecla_da_janela(&mut m, BoardKey::Delete, None));
    assert!(tecla_da_janela(&mut m, BoardKey::Char('e'), Some("e")));
    assert_eq!(m.receita(id), antes, "a tecla mexeu no som");
    assert_eq!(m.escuta.toques, 0, "a tecla tocou o som");
    use ph2d_editor_core::doc_surface::DocumentSurface;
    assert!(m.docs().undo(id, false), "o desfazer desceu para o som");
    assert!(som_de_opcional(&m, e).is_empty());
}

/// Por baixo da janela nada é alvo: fora dela o clique só fecha, e não aprova nada.
#[test]
fn clicar_fora_fecha_sem_aprovar_e_nao_chega_ao_que_esta_por_baixo() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    let play = m.docs().rect_de(id, Alvo::Play).expect("Play");
    abre_a_janela(&mut m, id);
    assert!(
        m.docs().rect_de(id, Alvo::Play).is_none(),
        "o Play continua alvo"
    );
    let (x, y) = (play.x + play.w / 2.0, play.y + play.h / 2.0);
    assert!(m.ponteiro_da_shell(PointerKind::Down, x, y));
    m.ponteiro_da_shell(PointerKind::Up, x, y);
    m.quadro();
    assert!(m.docs().rect_de(id, Alvo::Janela(Janela::Busca)).is_none());
    assert_eq!(m.escuta.toques, 0, "o clique chegou ao Play");
    assert!(som_de_opcional(&m, e).is_empty(), "aprovou ao fechar");
    assert!(
        !m.mundo
            .get_resource::<ProjectSounds>()
            .is_some_and(|s| !s.is_empty())
    );
}

fn hierarquia_com(row: NodeId, bits: u64) -> (MockPanelHost, HierarchyState) {
    ph2d_panel_hierarchy::cards::force_cards_on_this_thread(Some(false));
    let mut h = MockPanelHost::with_panel::<HierarchyPanel>();
    let mut linhas = BTreeMap::new();
    linhas.insert(
        row,
        HierarchyEntity {
            name: "Crate".into(),
            icon: IconId::Eyedropper,
            indent: 0,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            hovered: false,
            muted: false,
            locked: false,
            group_locked: false,
            role: RowRole::Object,
            components: Vec::new(),
            content: Vec::new(),
            has: ComponentSet::default(),
            entity: Some(bits),
        },
    );
    ph2d_panel_hierarchy::sync_from_hierarchy(h.store_mut(), &[row], linhas);
    let st = HierarchyState::default();
    let _ = h.drained_actions();
    (h, st)
}

/// O rato sobre a linha pintada, até o painel responder.
fn clica_na_linha(h: &mut MockPanelHost, st: &mut HierarchyState, row: NodeId) {
    let janela = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let rects = h.paint::<HierarchyPanel>(st, janela);
    let (_, r) = rects
        .iter()
        .find(|(n, _)| *n == row)
        .copied()
        .expect("a linha não foi pintada");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let ev = |kind, t| PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    };
    let mut evs = h.dispatch_pointer_event(ev(PointerKind::Down, 1_000_000_000));
    evs.extend(h.dispatch_pointer_event(ev(PointerKind::Up, 1_010_000_000)));
    assert!(
        evs.contains(&WidgetEvent::Click(row)),
        "o rato não virou clique na linha: {evs:?}"
    );
    for ev in evs {
        h.apply_panel_event::<HierarchyPanel>(st, ev);
    }
}

/// O clique real na linha da Hierarchy, com o conta-gotas armado: o objeto recebe o som e a seleção da cena não muda.
#[test]
fn o_conta_gotas_escolhe_na_hierarchy_sem_selecionar() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let id = m.abre("Hit", duas_camadas());
    abre_a_janela(&mut m, id);
    m.clica_na_aba(id, Alvo::Janela(Janela::ContaGotas));
    m.quadro();

    let row = NodeId(0xC0DE);
    let (mut h, mut st) = hierarquia_com(row, e.to_bits());
    clica_na_linha(&mut h, &mut st, row);
    let acoes = h.drained_actions();
    assert!(
        !acoes
            .iter()
            .any(|a| matches!(a, EditorAction::Hierarchy(_))),
        "a linha selecionou: {acoes:?}"
    );
    m.quadro();
    m.quadro();
    assert_eq!(som_de(&m, e), "Hit.wav", "a escolha não chegou ao objeto");
    assert!(
        m.docs().rect_de(id, Alvo::Janela(Janela::Busca)).is_none(),
        "ficou aberta"
    );

    // Desarmado de novo: a mesma linha volta a selecionar.
    let (mut h, mut st) = hierarquia_com(row, e.to_bits());
    clica_na_linha(&mut h, &mut st, row);
    assert!(
        h.drained_actions()
            .iter()
            .any(|a| matches!(a, EditorAction::Hierarchy(HierRequest::SelectRow { .. }))),
        "o conta-gotas ficou armado"
    );
}

/// Com a aba do som fora de vista, a Hierarchy volta a ser da cena.
#[test]
fn trocar_de_aba_desarma_o_conta_gotas() {
    let mut m = Mesa::nova();
    let e = caixa(&mut m.mundo);
    let a = m.abre("A", duas_camadas());
    abre_a_janela(&mut m, a);
    m.clica_na_aba(a, Alvo::Janela(Janela::ContaGotas));
    m.quadro();
    let _b = m.abre("B", outra_receita());
    m.quadro();
    m.quadro();
    let row = NodeId(0xC0DE);
    let (mut h, mut st) = hierarquia_com(row, e.to_bits());
    clica_na_linha(&mut h, &mut st, row);
    assert!(
        h.drained_actions()
            .iter()
            .any(|a| matches!(a, EditorAction::Hierarchy(HierRequest::SelectRow { .. }))),
        "a Hierarchy continuou presa ao conta-gotas"
    );
}
