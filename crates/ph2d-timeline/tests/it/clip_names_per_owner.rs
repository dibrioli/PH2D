//! ⭐⭐ **Os nomes de clip são únicos POR ANIMAÇÃO** (doc ObjectsAndComponents 01 §14, onda 4 item 2).
//!
//! Um osso inteligente guarda o NOME do clip e resolve-o dentro da animação onde vive — então a
//! unicidade que importa é a do DONO, não a do documento. Global, ela punha «andar 2» na cópia de
//! um Prefab, e o osso da cópia procurava «andar» no documento inteiro e tocava o do ORIGINAL.

use ph2d_timeline::{SCENE, TimelineDoc};

const HEROI: u64 = 7;
const COPIA: u64 = 17;

fn nome(doc: &TimelineDoc, i: usize) -> &str {
    &doc.clips()[i].name
}

/// Dois donos com «andar» coexistem — e cada um acha o SEU pelo nome.
#[test]
fn dois_donos_com_o_mesmo_nome_coexistem() {
    let mut doc = TimelineDoc::new();
    let a = doc.add_clip_for("andar".into(), SCENE).unwrap();
    let b = doc.add_clip_for("andar".into(), HEROI).unwrap();
    assert_eq!(nome(&doc, a), "andar");
    assert_eq!(
        nome(&doc, b),
        "andar",
        "o nome da Cena coagiu o do Herói — a unicidade é por animação"
    );
    assert_eq!(doc.clip_named(SCENE, "andar"), Some(a));
    assert_eq!(doc.clip_named(HEROI, "andar"), Some(b));
    assert_eq!(
        doc.clip_named(COPIA, "andar"),
        None,
        "um dono sem esse clip NÃO cai para o de outro dono"
    );
    // No MESMO dono continua a escada.
    let c = doc.add_clip_for("andar".into(), HEROI).unwrap();
    assert_eq!(nome(&doc, c), "andar 2");
}

/// Renomear: para um nome do MESMO dono numera; para o de outro dono aceita.
#[test]
fn renomear_so_colide_dentro_do_dono() {
    let mut doc = TimelineDoc::new();
    let andar = doc.add_clip_for("andar".into(), HEROI).unwrap();
    let pular = doc.add_clip_for("pular".into(), HEROI).unwrap();
    doc.rename_clip(pular, "andar".into());
    assert_eq!(
        nome(&doc, pular),
        "andar 2",
        "renomear para um nome do mesmo dono duplicou-o"
    );
    assert_eq!(nome(&doc, andar), "andar");
    let cena = doc.add_clip_for("correr".into(), SCENE).unwrap();
    doc.rename_clip(cena, "andar".into());
    assert_eq!(
        nome(&doc, cena),
        "andar",
        "o nome de OUTRO dono recusou a renomeação — ele não colide"
    );
}

/// A cópia de um objecto mantém os nomes («andar», não «andar 2») — e o «Clip N» de partida e o
/// «X copy» do Duplicate contam só o dono.
#[test]
fn a_copia_mantem_os_nomes_e_as_portas_que_inventam_contam_o_dono() {
    let mut doc = TimelineDoc::new();
    doc.add_clip_for("andar".into(), HEROI).unwrap();
    assert_eq!(doc.copy_animation(HEROI, COPIA, |_| None), 1);
    let novo = doc.clips_of(COPIA).next().unwrap();
    assert_eq!(nome(&doc, novo), "andar", "a cópia ganhou um sufixo");
    assert_eq!(doc.clip_named(COPIA, "andar"), Some(novo));

    let dup = doc.duplicate_clip(novo).unwrap();
    assert_eq!(nome(&doc, dup), "andar copy");
    let dup_heroi = doc
        .duplicate_clip(doc.clip_named(HEROI, "andar").unwrap())
        .unwrap();
    assert_eq!(
        nome(&doc, dup_heroi),
        "andar copy",
        "a cópia do Herói contou o «andar copy» da CÓPIA"
    );
    // O «Clip N» da Cena já está tomado («Clip 2» é o seed dela); o do Herói começa do princípio.
    doc.add_clip_for("Clip 2".into(), SCENE).unwrap();
    assert_eq!(doc.fresh_clip_name_in(SCENE), "Clip 3");
    assert_eq!(doc.fresh_clip_name_in(HEROI), "Clip 2");
}
