//! ⭐ **Os textos das chaves de texto** (v22, `doc_textos.rs`): a chave guarda um índice na tabela
//! da sessão (`TextosDosCanais`), o ficheiro leva a tabela COMPACTADA aos textos usados, e nem
//! gravar, nem reabrir, nem undo/redo trocam o texto de uma chave.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::canal::familia;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::{FieldKey, TextosDosCanais};
use ph2d_timeline::{
    DOC_VERSION, PropKind, TimelineDoc, TimelineIntent as I, TimelineState, apply_intent,
};

const FAMILIA: FieldKey = FieldKey::new(familia::VETOR, v::TEXTO_FAMILIA, 0, 0);
const CONTEUDO: FieldKey = FieldKey::new(familia::VETOR, v::TEXTO_CONTEUDO, 0, 0);
const VERMELHO: FieldKey = FieldKey::new(familia::VETOR, v::TINTA_PREENCH + v::OFF_SOLIDA, 0, 0);

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn f(x: f32) -> AnimValue {
    AnimValue::Float(x)
}

/// Os textos de cada chave do campo `k` (e o `default` da trilha), lidos pela `tabela`.
fn textos(doc: &TimelineDoc, k: FieldKey, tabela: &TextosDosCanais) -> Vec<Option<String>> {
    let b = doc.bindings().iter().find(|b| b.prop == PropKind::Field(k));
    let track = doc
        .active_clip()
        .track(b.expect("o campo tem binding").target);
    let track = track.expect("o campo tem trilha");
    let ler = |v: AnimValue| match v {
        AnimValue::Float(x) => tabela.texto(x).map(str::to_owned),
        _ => panic!("um texto é Float"),
    };
    let mut out: Vec<_> = track.keys().iter().map(|k| ler(k.value)).collect();
    out.push(ler(track.default_value()));
    out
}

fn rest(doc: &TimelineDoc, k: FieldKey) -> Option<f32> {
    doc.bindings()
        .iter()
        .find(|b| b.prop == PropKind::Field(k))
        .and_then(|b| b.rest)
}

/// ⭐ A v21 abre na v22 sem perda: tabela vazia, todo o resto campo a campo.
#[test]
fn o_doc_v21_migra_para_v22() {
    const V21_BIN: &[u8] = include_bytes!("../fixtures/timeline_doc_v21.bin");
    const V21_JSON: &str = include_str!("../fixtures/timeline_doc_v21.json");
    assert_eq!(DOC_VERSION, 22);
    let doc = TimelineDoc::from_bytes(V21_BIN).expect("o v21 tem de abrir");
    assert_eq!(doc.version, DOC_VERSION);
    assert!(doc.textos().is_empty(), "nenhuma build v21 publicava texto");
    let sem = |mut j: serde_json::Value| {
        j.as_object_mut().unwrap().remove("version");
        j.as_object_mut().unwrap().remove("textos");
        j
    };
    let antes: serde_json::Value = serde_json::from_str(V21_JSON).unwrap();
    assert_eq!(sem(serde_json::to_value(&doc).unwrap()), sem(antes));
    assert_eq!(
        doc.bindings()
            .iter()
            .filter(|b| matches!(b.prop, PropKind::Field(_)))
            .count(),
        2,
        "o que só a v21 tinha (os campos) atravessa a migração"
    );
    assert_eq!(
        TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap(),
        doc
    );
}

/// ⭐ Gravar → reabrir devolve o MESMO texto em cada chave (chaves, `default`, `rest`); o ficheiro
/// leva só os textos usados; gravar não toca no documento vivo nem na tabela da sessão; e reabrir
/// INTERNA na sessão — um índice que ela já deu continua a ser o mesmo texto.
#[test]
fn gravar_e_reabrir_devolve_o_texto_de_cada_chave_e_so_os_usados() {
    let mut sessao = TextosDosCanais::default();
    for tecla in ["O", "Ol"] {
        sessao.indice(Some(tecla));
    }
    let inter = sessao.indice(Some("Inter"));
    sessao.indice(Some("nunca usado"));
    let lato = sessao.indice(Some("Lato"));
    let ola = sessao.indice(Some("Olá"));
    let base = sessao.indice(Some("Base"));

    let (mut doc, e) = (TimelineDoc::new(), 7);
    let fam = PropKind::Field(FAMILIA);
    doc.insert_key(e, fam, s(0.0), f(inter), Interp::Linear);
    doc.insert_key(e, fam, s(1.0), f(lato), Interp::Linear);
    doc.insert_key(e, fam, s(2.0), f(0.0), Interp::Linear);
    doc.insert_key(e, PropKind::Field(CONTEUDO), s(0.0), f(ola), Interp::Linear);
    doc.insert_key(e, PropKind::Field(VERMELHO), s(0.0), f(7.0), Interp::Linear);
    for b in doc.bindings_mut() {
        if b.prop == PropKind::Field(CONTEUDO) {
            b.rest = Some(base);
        }
    }

    let (doc_antes, sessao_antes) = (doc.clone(), sessao.clone());
    let bytes = doc.to_bytes_com_textos(&sessao).expect("grava");
    assert_eq!(doc, doc_antes, "a compactação corre numa CÓPIA");
    assert_eq!(sessao, sessao_antes);
    let tem = |agulha: &str| bytes.windows(agulha.len()).any(|w| w == agulha.as_bytes());
    assert!(
        tem("Lato") && !tem("nunca usado"),
        "o ficheiro leva só os usados"
    );

    let mut aberto = TimelineDoc::from_bytes(&bytes).expect("abre");
    let mut no_ficheiro = aberto.textos().to_vec();
    no_ficheiro.sort();
    assert_eq!(
        no_ficheiro,
        ["Base", "Inter", "Lato", "Olá"],
        "só os usados"
    );

    // A sessão que abre já tem textos seus — e um deles é o mesmo.
    let mut outra = TextosDosCanais::default();
    outra.indice(Some("Lato"));
    outra.indice(Some("x"));
    assert!(aberto.adota_textos(&mut outra));
    assert!(
        aberto.textos().is_empty(),
        "o documento passa a indexar a sessão"
    );
    assert!(!aberto.adota_textos(&mut outra), "idempotente");
    assert_eq!(outra.texto(1.0), Some("Lato"), "interna, nunca substitui");
    assert_eq!(outra.texto(2.0), Some("x"));

    for k in [FAMILIA, CONTEUDO] {
        assert_eq!(
            textos(&aberto, k, &outra),
            textos(&doc, k, &sessao),
            "{k:?}"
        );
    }
    assert_eq!(
        textos(&aberto, FAMILIA, &outra),
        [Some("Inter"), Some("Lato"), None, Some("Inter")].map(|t| t.map(str::to_owned))
    );
    let r = rest(&aberto, CONTEUDO).expect("o rest atravessa");
    assert_eq!(outra.texto(r), Some("Base"));
    assert_eq!(
        textos(&aberto, VERMELHO, &TextosDosCanais::default()).len(),
        2,
        "o campo contínuo existe"
    );
    let b = aberto
        .bindings()
        .iter()
        .find(|b| b.prop == PropKind::Field(VERMELHO))
        .unwrap();
    assert_eq!(
        aberto.active_clip().track(b.target).unwrap().keys()[0].value,
        f(7.0),
        "um campo que não é texto não é remapeado"
    );

    // Regravar na sessão nova dá a mesma tabela (a ordem é a do 1.º uso, fixa).
    let de_novo = TimelineDoc::from_bytes(&aberto.to_bytes_com_textos(&outra).unwrap()).unwrap();
    assert_eq!(de_novo, TimelineDoc::from_bytes(&bytes).unwrap());
}

/// A porta SEM tabela recusa perder o texto de uma chave — nunca grava um índice que ninguém lê.
#[test]
fn to_bytes_sem_tabela_recusa_perder_um_texto() {
    let mut doc = TimelineDoc::new();
    doc.insert_key(1, PropKind::Field(CONTEUDO), s(0.0), f(0.0), Interp::Hold);
    assert!(doc.to_bytes().is_ok(), "«nenhum» não precisa de tabela");
    doc.insert_key(1, PropKind::Field(CONTEUDO), s(1.0), f(3.0), Interp::Hold);
    assert!(doc.to_bytes().is_err());
}

/// ⭐ Undo/redo de uma chave de texto mantém o texto — também depois de a sessão ter crescido, e
/// através de gravar e reabrir.
#[test]
fn undo_e_redo_de_uma_chave_de_texto_mantem_o_texto() {
    let mut sessao = TextosDosCanais::default();
    let (mut st, mut ph) = (TimelineState::new(), Playhead::new(1.0 / 60.0));
    let prop = PropKind::Field(CONTEUDO);
    let mut chave = |st: &mut TimelineState, t: f64, x: f32| {
        let i = I::AddKey {
            entity: 9,
            prop,
            t: s(t),
            value: f(x),
            interp: Interp::Linear,
        };
        apply_intent(st, &mut ph, i);
    };
    chave(&mut st, 0.0, sessao.indice(Some("Olá")));
    chave(&mut st, 1.0, sessao.indice(Some("Olá mundo")));
    let depois = textos(&st.doc, CONTEUDO, &sessao);
    // A sessão continua a crescer (cada tecla) entre o gesto e o undo.
    for tecla in ["a", "ab", "abc"] {
        sessao.indice(Some(tecla));
    }
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, I::Undo);
    let ola = Some("Olá".to_owned());
    assert_eq!(textos(&st.doc, CONTEUDO, &sessao), [ola.clone(), ola]);
    apply_intent(&mut st, &mut ph, I::Redo);
    assert_eq!(textos(&st.doc, CONTEUDO, &sessao), depois);

    let mut fresca = TextosDosCanais::default();
    let mut aberto =
        TimelineDoc::from_bytes(&st.doc.to_bytes_com_textos(&sessao).unwrap()).unwrap();
    aberto.adota_textos(&mut fresca);
    assert_eq!(textos(&aberto, CONTEUDO, &fresca), depois);
}

/// ⭐ Um texto NUNCA interpola: a chave nasce `Hold` e nenhuma edição de interp a tira de lá (a do
/// ponto, a da selecção, «Convert to Bezier»); a meio caminho vale o texto da chave de antes.
#[test]
fn um_texto_nunca_interpola_entre_dois_indices() {
    let mut sessao = TextosDosCanais::default();
    let (a, b) = (sessao.indice(Some("Inter")), sessao.indice(Some("Lato")));
    let (mut st, mut ph) = (TimelineState::new(), Playhead::new(1.0 / 60.0));
    let prop = PropKind::Field(FAMILIA);
    for (t, x) in [(0.0, a), (1.0, b)] {
        let i = I::AddKey {
            entity: 3,
            prop,
            t: s(t),
            value: f(x),
            interp: Interp::Linear,
        };
        apply_intent(&mut st, &mut ph, i);
    }
    let target = st.doc.binding_for(3, prop).unwrap().target;
    let key = st.doc.active_clip().track(target).unwrap().ids()[0];
    apply_intent(
        &mut st,
        &mut ph,
        I::SetInterp {
            target,
            key,
            interp: Interp::Linear,
        },
    );
    apply_intent(
        &mut st,
        &mut ph,
        I::SetSelectedInterp {
            interp: Interp::Linear,
        },
    );
    apply_intent(&mut st, &mut ph, I::ConvertSelectionToBezier);
    let track = st.doc.active_clip().track(target).unwrap();
    assert!(
        track.keys().iter().all(|k| k.interp == Interp::Hold),
        "{:?}",
        track.keys()
    );
    assert_eq!(st.doc.active_clip().sample(target, 0.5), Some(f(a)));
    assert_eq!(st.doc.active_clip().sample(target, 0.999), Some(f(a)));
}
