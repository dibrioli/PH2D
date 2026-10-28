use super::*;
use crate::agenda::Exception;
use crate::{EventId, KanbanOp, KanbanSet, Moment, Record};

const DAY: i64 = 24 * 3_600_000;

fn hex_fixture(text: &str) -> Vec<u8> {
    let hex: String = text.lines().filter(|l| !l.starts_with('#')).collect();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// O conjunto v7 inteiro: o v6 cheio e um evento com fim e excepções.
pub(crate) fn full_v7() -> KanbanSet {
    let (mut set, _) = crate::links::tests::full_v6();
    let id = *set.agenda().events.keys().next().unwrap();
    let mut s = set.agenda().events[&id].clone();
    s.until = Some(Moment(99 * DAY));
    s.exceptions = vec![
        Exception {
            at: Moment(7 * DAY),
            becomes: None,
        },
        Exception {
            at: Moment(14 * DAY),
            becomes: Some(EventId(77)),
        },
    ];
    set.apply(
        crate::KanbanId::AGENDA,
        vec![KanbanOp::put(Record::Event(s))],
    );
    // Os três fundos que o v7 conhecia, um por Kanban (o índice de cada variante fica gravado).
    let photo = set.put_file(vec![1_u8, 2, 3], Some([4, 3]));
    let ids: Vec<_> = set.iter().map(|k| k.id).collect();
    let others = [Background::Gradient(4), Background::Image(photo)];
    let more = [set.create("Gradiente"), set.create("Imagem")];
    for (id, bg) in
        ids.iter()
            .take(1)
            .chain(&more)
            .zip([Background::Color(Hue::Pink), others[0], others[1]])
    {
        set.doc_mut(*id).unwrap().header.background = bg;
    }
    set
}

#[test]
fn v7_bytes_read_as_v8_equal() {
    let bytes = hex_fixture(include_str!("../tests/fixtures/kanban_set_v7_full.hex"));
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 7);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    let today = full_v7();
    assert!(
        back.iter().eq(today.iter()),
        "o degrau v7 → v8 dá os mesmos Kanbans"
    );
    assert_eq!(back.agenda(), today.agenda(), "e a mesma agenda");
    let bgs: Vec<_> = back.iter().map(|k| k.header.background).collect();
    assert!(bgs.contains(&Background::Color(Hue::Pink)));
    assert!(bgs.contains(&Background::Gradient(4)));
    assert!(bgs.iter().any(|b| matches!(b, Background::Image(_))));
    let again = KanbanSet::from_bytes(&back.to_bytes().unwrap()).unwrap();
    assert_eq!(again, back, "e, regravado em v8, continua igual");
}

/// O conjunto v8: o v7 e um Kanban por fundo novo (cor livre, cada padrão).
pub(crate) fn full_v8() -> KanbanSet {
    let mut set = full_v7();
    let mut put = |name: &str, bg: Background| {
        let id = set.create(name);
        set.doc_mut(id).unwrap().header.background = bg;
    };
    put("Livre", Background::Custom(Rgb([250, 240, 200])));
    for (i, kind) in PatternKind::ALL.into_iter().enumerate() {
        put(
            "Padrão",
            Background::Pattern(Pattern {
                kind,
                base: if i % 2 == 0 {
                    Tint::Hue(Hue::ALL[i])
                } else {
                    Tint::Rgb(Rgb([10, 20, u8::try_from(i).unwrap()]))
                },
                ink: Tint::Rgb(Rgb([255, 255, 255])),
            }),
        );
    }
    set
}

#[test]
fn the_v8_bytes_round_trip_custom_and_pattern_backgrounds() {
    let set = full_v8();
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        crate::FORMAT_VERSION
    );
    assert_eq!(crate::FORMAT_VERSION, 8);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert!(back.iter().eq(set.iter()));
    assert_eq!(back.agenda(), set.agenda());
    let kinds: Vec<_> = back
        .iter()
        .filter_map(|k| match k.header.background {
            Background::Pattern(p) => Some(p.kind),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, PatternKind::ALL);
}

#[test]
fn v8_fixture_reads_as_today() {
    let bytes = hex_fixture(include_str!("../tests/fixtures/kanban_set_v8_full.hex"));
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 8);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    let today = full_v8();
    assert!(back.iter().eq(today.iter()));
    assert_eq!(back.agenda(), today.agenda());
}

#[test]
#[ignore = "gera a fixture v8 (corre-se UMA vez; depois fica congelada)"]
fn dump_v8_fixture() {
    let bytes = full_v8().to_bytes().unwrap();
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    for chunk in hex.as_bytes().chunks(120) {
        println!("{}", std::str::from_utf8(chunk).unwrap());
    }
}
