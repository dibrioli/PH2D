use super::*;

fn duas() -> ScriptLibrary {
    let mut lib = ScriptLibrary::new();
    lib.insert("bob.luau", "ph2d.property(\"speed\", 1)\n");
    lib.insert("porta.luau", "-- acentuação: ção\n");
    lib
}

/// ⭐ Gravar → ler → gravar dá os MESMOS bytes (e a mesma biblioteca).
#[test]
fn library_blob_round_trips_byte_identical() {
    let lib = duas();
    let bytes = lib.to_bytes();
    let back = ScriptLibrary::from_bytes(&bytes).expect("lê");
    assert_eq!(back, lib);
    assert_eq!(back.to_bytes(), bytes);
    assert_eq!(
        ScriptLibrary::from_bytes(&[]).expect("vazio"),
        ScriptLibrary::new()
    );
}

/// Um campo de um MINOR futuro salta-se, no registo da biblioteca e no de um script.
#[test]
fn unknown_tags_are_skipped() {
    let lib = duas();
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&MAJOR.to_le_bytes());
    out.extend_from_slice(&(MINOR + 7).to_le_bytes());
    let mut r = Record::new();
    r.varint(1, 3).str(99, "futuro");
    for (id, d) in lib.iter() {
        let mut s = Record::new();
        s.varint(1, id.0)
            .str(2, &d.name)
            .bytes(42, &[1, 2, 3])
            .str(3, &d.text);
        r.bytes(2, &s.finish());
    }
    out.extend_from_slice(&r.finish());
    assert_eq!(ScriptLibrary::from_bytes(&out).expect("tolerante"), lib);
}

#[test]
fn another_major_is_refused_with_a_typed_error() {
    let mut bytes = duas().to_bytes();
    bytes[MAGIC.len()..MAGIC.len() + 2].copy_from_slice(&(MAJOR + 1).to_le_bytes());
    assert_eq!(
        ScriptLibrary::from_bytes(&bytes),
        Err(DocError::MajorMismatch {
            found: MAJOR + 1,
            supported: MAJOR
        })
    );
    assert_eq!(
        ScriptLibrary::from_bytes(b"PH2DMACRO\x01\x00\x00\x00\x00"),
        Err(DocError::NotAScriptLibrary)
    );
}

#[test]
fn names_stay_unique_on_insert_and_rename() {
    let mut lib = ScriptLibrary::new();
    let a = lib.insert("bob.luau", "");
    let b = lib.insert("bob.luau", "");
    let c = lib.insert("bob 2.luau", "");
    assert_eq!(lib.get(b).unwrap().name, "bob 2.luau");
    assert_eq!(lib.get(c).unwrap().name, "bob 3.luau");
    // Pedir um nome ocupado dá o menor livre — que pode ser o próprio.
    assert_eq!(lib.rename(b, "bob 3.luau").as_deref(), Some("bob 2.luau"));
    assert_eq!(lib.rename(b, "porta.luau").as_deref(), Some("porta.luau"));
    assert_eq!(
        lib.rename(c, "bob.luau").as_deref(),
        Some("bob 2.luau"),
        "o «2» ficou livre"
    );
    assert_eq!(
        lib.rename(a, "bob.luau").as_deref(),
        Some("bob.luau"),
        "o próprio nome não colide"
    );
    assert_eq!(lib.insert("", "").0, 4);
    assert_eq!(lib.by_name("script.luau"), Some(ScriptId(4)));
}

/// ⚠️ O contador nunca volta a entregar um id que já existiu — nem depois de apagar, nem a ler um
/// blob cujo `next` ficou para trás.
#[test]
fn ids_are_monotonic_and_never_reused() {
    let mut lib = duas();
    lib.remove(ScriptId(2));
    assert_eq!(lib.insert("x.luau", ""), ScriptId(3));
    let mut r = Record::new();
    r.varint(1, 1);
    let mut s = Record::new();
    s.varint(1, 9).str(2, "a.luau").str(3, "");
    r.bytes(2, &s.finish());
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&MAJOR.to_le_bytes());
    bytes.extend_from_slice(&MINOR.to_le_bytes());
    bytes.extend_from_slice(&r.finish());
    let mut back = ScriptLibrary::from_bytes(&bytes).expect("lê");
    assert_eq!(back.insert("b.luau", ""), ScriptId(10));
}

#[test]
fn set_text_reports_a_change_only_when_it_changes() {
    let mut lib = duas();
    assert!(lib.set_text(ScriptId(1), "x"));
    assert!(!lib.set_text(ScriptId(1), "x"));
    assert!(!lib.set_text(ScriptId(77), "x"));
}

/// ⛔ (auditoria do fecho, item 15) Um id no FIM da faixa (`u64::MAX`, ou o contador gravado lá) é um
/// ficheiro estragado: o próximo script novo receberia um id vivo. Recusado com uma frase legível —
/// nunca `max + 1` a dar a volta.
#[test]
fn um_id_no_fim_da_faixa_e_recusado_com_frase() {
    let mut lib = duas();
    lib.scripts.insert(
        ScriptId(u64::MAX),
        ScriptDoc {
            name: "fim.luau".into(),
            text: String::new(),
        },
    );
    let err = ScriptLibrary::from_bytes(&lib.to_bytes()).expect_err("recusado");
    assert_eq!(err, DocError::IdsExhausted { id: u64::MAX });
    let frase = err.to_string();
    assert!(
        !frase.contains('{') && frase.contains(&u64::MAX.to_string()),
        "a frase: {frase}"
    );
    let mut lib = duas();
    lib.next = u64::MAX;
    assert!(matches!(
        ScriptLibrary::from_bytes(&lib.to_bytes()),
        Err(DocError::IdsExhausted { .. })
    ));
    // Controlo: o último id que ainda deixa um a seguir lê-se.
    let mut lib = duas();
    lib.scripts.insert(
        ScriptId(u64::MAX - 2),
        ScriptDoc {
            name: "quase.luau".into(),
            text: String::new(),
        },
    );
    let back = ScriptLibrary::from_bytes(&lib.to_bytes()).expect("lê");
    assert_eq!(back.next, u64::MAX - 1);
}

/// A revisão muda a cada mudança de conteúdo (e só então), é única entre bibliotecas, e fica fora
/// da igualdade e do ficheiro.
#[test]
fn a_revisao_muda_com_o_conteudo_e_so_com_ele() {
    let mut lib = ScriptLibrary::new();
    let mut vistas = std::collections::BTreeSet::new();
    let muda = |lib: &ScriptLibrary, deve: bool, vistas: &mut std::collections::BTreeSet<u64>| {
        assert_eq!(vistas.insert(lib.revision()), deve, "{}", lib.revision());
    };
    muda(&lib, true, &mut vistas);
    let id = lib.insert("a.luau", "x = 1");
    muda(&lib, true, &mut vistas);
    assert!(!lib.set_text(id, "x = 1"));
    muda(&lib, false, &mut vistas);
    assert!(lib.set_text(id, "x = 2"));
    muda(&lib, true, &mut vistas);
    lib.rename(id, "a.luau");
    muda(&lib, false, &mut vistas);
    lib.rename(id, "b.luau");
    muda(&lib, true, &mut vistas);
    let lido = ScriptLibrary::from_bytes(&lib.to_bytes()).expect("lê");
    assert_eq!(lido, lib);
    muda(&lido, true, &mut vistas);
    assert_eq!(lib.clone().revision(), lib.revision());
    assert!(lib.remove(id).is_some());
    muda(&lib, true, &mut vistas);
    assert!(lib.remove(id).is_none());
    muda(&lib, false, &mut vistas);
}
