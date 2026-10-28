//! GATE: o CSV, lido por um leitor RFC 4180 INDEPENDENTE (escrito aqui, à parte do exportador),
//! devolve as mesmas células que o Kanban tem.

use ph2d_kanban_import::{CSV_COLUMNS, export_csv};
use ph2d_kanban_model::{Card, Kanban};

use crate::imported;

/// RFC 4180: registos separados por CRLF; campo entre aspas pode ter vírgula, CRLF e `""`.
fn parse_csv(s: &str) -> Vec<Vec<String>> {
    let (mut rows, mut row, mut field) = (Vec::new(), Vec::new(), String::new());
    let mut chars = s.chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    field.push('"');
                }
                '"' => quoted = false,
                _ => field.push(c),
            }
            continue;
        }
        match c {
            '"' => {
                assert!(field.is_empty(), "aspas a meio de um campo sem aspas");
                quoted = true;
            }
            ',' => row.push(std::mem::take(&mut field)),
            '\r' => {
                assert_eq!(chars.next(), Some('\n'), "CR sem LF fora de aspas");
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            '\n' => panic!("LF sem CR fora de aspas"),
            _ => field.push(c),
        }
    }
    assert!(
        !quoted && field.is_empty() && row.is_empty(),
        "o ficheiro acaba em CRLF"
    );
    rows
}

fn expected(k: &Kanban, c: &Card) -> Vec<(&'static str, String)> {
    let list = k
        .lists
        .get(&c.list)
        .map_or("", |l| l.title.as_str())
        .to_owned();
    let atts = k.attachments.values().filter(|a| a.card == c.id).count();
    vec![
        (
            "Card ID",
            format!("{:08x}{:016x}", c.created.0 / 1000, c.id.0),
        ),
        ("Card Name", c.title.clone()),
        ("Card URL", String::new()),
        ("Card Description", c.description.clone()),
        (
            "Members",
            c.members
                .iter()
                .map(|m| m.0.clone())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("Due Complete", c.due.is_some_and(|d| d.done).to_string()),
        ("Attachment Count", atts.to_string()),
        ("List Name", list),
        ("Archived", c.archived.to_string()),
    ]
}

#[test]
fn csv_cells_read_back_by_an_independent_parser() {
    let k = imported().kanban;
    let text = export_csv(&k);
    let rows = parse_csv(&text);
    assert_eq!(rows[0], CSV_COLUMNS);
    assert_eq!(
        rows.len() - 1,
        k.cards.len(),
        "uma linha por cartão, arquivados incluídos"
    );
    let col = |name: &str| CSV_COLUMNS.iter().position(|c| *c == name).expect(name);
    for row in &rows[1..] {
        assert_eq!(row.len(), CSV_COLUMNS.len());
        let c = k
            .cards
            .values()
            .find(|c| c.title == row[col("Card Name")])
            .expect("o cartão da linha");
        for (name, want) in expected(&k, c) {
            assert_eq!(row[col(name)], want, "{name} de {}", c.title);
        }
    }
    let roteiro = rows
        .iter()
        .find(|r| r[1] == "Roteiro, versão \"final\"")
        .expect("vírgula e aspas");
    assert_eq!(
        roteiro[col("Labels")],
        "Bug (red), Arte (purple), Urgente (yellow_dark), green_light, Sem cor, Cor estranha"
    );
    assert_eq!(
        (
            roteiro[col("Checklist Items Total")].as_str(),
            roteiro[col("Checklist Items Completed")].as_str()
        ),
        ("4", "1")
    );
    assert_eq!(roteiro[col("Due Date")], "2026-10-30T15:00:00.000Z");
    let trilha = rows
        .iter()
        .find(|r| r[1] == "Trilha sonora")
        .expect("CRLF dentro");
    assert!(trilha[col("Card Description")].contains("\"aspas\"\r\ne quebra"));
}
