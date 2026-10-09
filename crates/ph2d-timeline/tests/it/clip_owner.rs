//! ⭐⭐ O DONO de cada clip (v19, doc ObjectsAndComponents 01 §6): a migração v18 sem perda e as
//! leis da Animação por objecto.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_timeline::{DOC_VERSION, MAX_CLIPS, PropKind, SCENE, StripSource, TimelineDoc};

const V18_BIN: &[u8] = include_bytes!("../fixtures/timeline_doc_v18.bin");
const V18_JSON: &str = include_str!("../fixtures/timeline_doc_v18.json");

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// Tira o campo `owner` de cada clip — o que a v19 acrescentou — e a versão.
fn sem_o_que_a_v19_acrescentou(mut v: serde_json::Value) -> serde_json::Value {
    v.as_object_mut().unwrap().remove("version");
    for c in v["clips"].as_array_mut().unwrap() {
        c.as_object_mut().unwrap().remove("owner");
    }
    v
}

#[test]
fn um_ficheiro_v18_abre_com_tudo_na_animacao_da_cena() {
    let doc = TimelineDoc::from_bytes(V18_BIN).expect("o v18 tem de abrir");
    assert_eq!(doc.version, DOC_VERSION);
    assert!(
        doc.clips().iter().all(|c| c.owner.is_none()),
        "todo clip antigo é da Cena"
    );
    assert_eq!(doc.clips_of(SCENE).collect::<Vec<_>>(), vec![0, 1]);

    // ZERO perda: o documento migrado, sem o que a v19 acrescentou, é o v18 campo a campo.
    let antes: serde_json::Value = serde_json::from_str(V18_JSON).unwrap();
    let depois = serde_json::to_value(&doc).unwrap();
    assert_eq!(
        sem_o_que_a_v19_acrescentou(depois),
        sem_o_que_a_v19_acrescentou(antes)
    );

    // E volta a gravar como v19 e a abrir igual.
    let de_novo = TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(de_novo, doc);
}

#[test]
fn os_clips_de_um_objecto_sao_a_animacao_dele() {
    let mut doc = TimelineDoc::new();
    let heroi = 7;
    assert!(!doc.has_animation(heroi));
    let andar = doc.add_clip_for("andar".into(), heroi).unwrap();
    let pular = doc.add_clip_for("pular".into(), heroi).unwrap();
    assert!(doc.has_animation(heroi));
    assert_eq!(doc.clips_of(heroi).collect::<Vec<_>>(), vec![andar, pular]);
    assert_eq!(
        doc.clips_of(SCENE).collect::<Vec<_>>(),
        vec![0],
        "a Cena não vê os do Herói"
    );
    assert!(!doc.has_animation(SCENE), "a Cena não é um objecto");

    // Duplicar uma variação fica no MESMO dono.
    let cansado = doc.duplicate_clip(andar).unwrap();
    assert!(doc.clip_belongs_to(cansado, heroi));

    // O tecto é POR animação (o seletor mostra uma): a Cena ainda cria.
    for n in 0..MAX_CLIPS {
        doc.add_clip_for(format!("x{n}"), heroi);
    }
    assert_eq!(doc.clips_of(heroi).count(), MAX_CLIPS);
    assert!(doc.add_clip_for("mais".into(), heroi).is_none());
    assert!(doc.duplicate_clip(andar).is_none());
    assert!(doc.add_clip_for("cena".into(), SCENE).is_some());
}

#[test]
fn tirar_a_animacao_leva_os_clips_e_os_strips_e_a_cena_fica() {
    let mut doc = TimelineDoc::new();
    let heroi = 7;
    let andar = doc.add_clip_for("andar".into(), heroi).unwrap();
    doc.add_clip_for("pular".into(), heroi).unwrap();
    let lane = doc.add_lane("Base".into()).unwrap();
    doc.add_strip(lane, andar, 0.0, 1.0).unwrap();
    doc.add_strip(lane, 0, 1.0, 2.0).unwrap();

    assert_eq!(doc.remove_animation(heroi), 2);
    assert!(!doc.has_animation(heroi));
    assert_eq!(doc.clips().len(), 1);
    let fontes: Vec<StripSource> = doc.stack()[0].strips.iter().map(|s| s.source).collect();
    assert_eq!(
        fontes,
        vec![StripSource::Clip(0)],
        "o strip do Herói saiu, o da Cena ficou"
    );

    // O ÚLTIMO clip da Cena nunca sai — nem pela porta de apagar clips.
    assert!(!doc.remove_clip(0));
    assert_eq!(doc.remove_animation(SCENE), 0);
}

#[test]
fn o_ultimo_da_cena_fica_mesmo_com_objectos_a_ter_clips() {
    let mut doc = TimelineDoc::new();
    let i = doc.add_clip_for("andar".into(), 7).unwrap();
    assert!(!doc.remove_clip(0), "a Cena ficaria sem animação");
    assert!(
        doc.remove_clip(i),
        "o último do objecto sai: é tirar-lhe a Animação"
    );
}

#[test]
fn a_copia_de_um_objecto_leva_clips_proprios() {
    let (heroi, braco, camara) = (7_u64, 8_u64, 9_u64);
    let (copia, braco2) = (17_u64, 18_u64);
    let mut doc = TimelineDoc::new();
    let andar = doc.add_clip_for("andar".into(), heroi).unwrap();
    doc.set_active(andar);
    doc.insert_key(
        heroi,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        braco,
        PropKind::Rotation,
        s(1.0),
        AnimValue::Float(0.5),
        Interp::Linear,
    );
    doc.insert_key(
        camara,
        PropKind::Opacity,
        s(1.0),
        AnimValue::Float(0.5),
        Interp::Linear,
    );

    let remap = |e: u64| match e {
        7 => Some(copia),
        8 => Some(braco2),
        _ => None,
    };
    assert_eq!(doc.copy_animation(heroi, copia, remap), 1);
    let novo = doc.clips_of(copia).next().expect("a cópia tem animação");
    assert_eq!(
        doc.clips()[novo].name,
        "andar",
        "o nome é único POR animação — a cópia mantém o dela (onda 4, item 2)"
    );

    // Um conjunto: o clip guarda as tracks ordenadas pelo ALVO, não pela ordem de chegada.
    let alvos_da_copia: std::collections::BTreeSet<u64> = doc.clips()[novo]
        .clip
        .tracks()
        .iter()
        .map(|(t, _)| doc.binding(*t).unwrap().entity)
        .collect();
    assert_eq!(
        alvos_da_copia,
        [copia, braco2, camara].into(),
        "fora da cópia fica no original"
    );

    // Independente: mexer na cópia não chega ao original.
    doc.set_active(novo);
    doc.insert_key(
        copia,
        PropKind::TranslationX,
        s(2.0),
        AnimValue::Float(9.0),
        Interp::Linear,
    );
    let t_copia = doc
        .binding_for(copia, PropKind::TranslationX)
        .unwrap()
        .target;
    assert_eq!(doc.clips()[novo].clip.track(t_copia).unwrap().len(), 2);
    let t_orig = doc
        .binding_for(heroi, PropKind::TranslationX)
        .unwrap()
        .target;
    assert_eq!(doc.clips()[andar].clip.track(t_orig).unwrap().len(), 1);
}

#[test]
fn o_dope_sheet_lista_o_que_a_animacao_em_vista_anima() {
    use ph2d_timeline::{TimelineState, TimelineViewSnapshot, animation_view};
    let (heroi, espada, inimigo) = (7_u64, 8_u64, 9_u64);
    let mut st = TimelineState::new();
    st.doc.insert_key(
        inimigo,
        PropKind::TranslationY,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    let andar = st.doc.add_clip_for("andar".into(), heroi).unwrap();
    st.doc.set_active(andar);
    st.doc.insert_key(
        heroi,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    st.doc.insert_key(
        espada,
        PropKind::Rotation,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    st.doc.bind(espada, PropKind::Opacity); // nova, ainda sem chave: aparece em qualquer vista
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let linhas = |st: &mut TimelineState| {
        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(st, &ph2d_core::Playhead::new(1.0 / 60.0), false);
        snap.tracks
            .iter()
            .map(|t| (t.entity, t.prop))
            .collect::<Vec<_>>()
    };
    animation_view::follow(&mut st, &mut ph, Some(heroi), &[]);
    assert_eq!(
        linhas(&mut st),
        [
            (heroi, PropKind::TranslationX),
            (espada, PropKind::Rotation),
            (espada, PropKind::Opacity)
        ],
        "o Herói não lista o Inimigo, que só a Cena anima"
    );
    animation_view::follow(&mut st, &mut ph, None, &[]);
    assert_eq!(
        linhas(&mut st),
        [
            (inimigo, PropKind::TranslationY),
            (espada, PropKind::Opacity)
        ],
        "e a Cena não lista o que só o Herói anima"
    );
}
