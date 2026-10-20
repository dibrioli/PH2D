//! Os ids e os relógios da equipa (W8).

use super::*;

/// Cada id derivado volta ao seu alvo, e os saltos não se leem uns pelos outros (a lição da W5).
#[test]
fn every_team_id_round_trips_and_no_salt_reads_another() {
    let mut all: Vec<Hit> = ALL.iter().map(|b| Hit::Button(*b)).collect();
    for n in [0usize, 1, 7, 4096] {
        all.extend([
            Hit::Edit(n),
            Hit::Delete(n),
            Hit::Restore(n),
            Hit::DropVersion(n),
            Hit::RenameVersion(n),
        ]);
    }
    for t in [1u64, 2, 99, u64::from(u32::MAX)] {
        all.extend([Hit::Bubble(ThreadId(t)), Hit::Row(ThreadId(t))]);
    }
    for h in &all {
        assert_eq!(hit_of(node(*h)), Some(*h), "{h:?}");
    }
    let mut ids: Vec<u64> = all.iter().map(|h| node(*h).0).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), all.len(), "dois alvos com o mesmo id");
    // Os ids da navegação e da facilitação não são da equipa.
    for b in [
        super::super::NavButton::Frames,
        super::super::NavButton::Present,
    ] {
        assert_eq!(hit_of(super::super::button_node(b)), None);
    }
    let fac = super::super::facilitate::fac_node(super::super::facilitate::FacButton::Surface);
    assert_eq!(hit_of(fac), None);
}

/// «Há quanto tempo» sem fuso: agora, minutos, horas, dias.
#[test]
fn ago_reads_in_the_largest_unit() {
    assert_eq!(ago(100, 100), "now");
    assert_eq!(ago(0, 59), "now");
    assert_eq!(ago(0, 60), "1 min ago");
    assert_eq!(ago(0, 3599), "59 min ago");
    assert_eq!(ago(0, 7200), "2 h ago");
    assert_eq!(ago(0, 86_400 * 3), "3 d ago");
    assert_eq!(
        ago(500, 100),
        "now",
        "um relógio que andou para trás não dá negativo"
    );
}

/// K4 — o cartão nunca passa a borda direita e, numa área mais estreita que ele, encosta à esquerda
/// (prova de mutação W8: só a borda direita tinha teste).
#[test]
fn the_card_x_keeps_both_edges() {
    let area = crate::zones::Rect::new(100.0, 0.0, 500.0, 400.0);
    assert_eq!(paint::card_x(200.0, 300.0, area, 10.0), 200.0);
    assert_eq!(
        paint::card_x(450.0, 300.0, area, 10.0),
        290.0,
        "a borda direita"
    );
    let narrow = crate::zones::Rect::new(100.0, 0.0, 200.0, 400.0);
    assert_eq!(
        paint::card_x(250.0, 300.0, narrow, 10.0),
        110.0,
        "estreita: à esquerda"
    );
}

/// W23 B14 — os trechos azuis do fio: cada ocorrência do `@nome` de quem a mensagem chama, por
/// ordem, os encaixados num só (`@Rui` dentro de `@Rui Costa`); quem o texto não mostra não pinta.
#[test]
fn mention_spans_cover_every_label_the_text_shows() {
    use ph2d_board_model::{Message, MessageId, Who};
    let text = "@Rui Costa, ping @Rui and @everyone; @Rui again";
    let m = Message {
        id: MessageId(1),
        author: "Ana".into(),
        text: text.into(),
        at: 0,
        edited: false,
        mentions: vec![
            Who::Person("Rui".into()),
            Who::Person("Rui Costa".into()),
            Who::Everyone,
            Who::Person("Bia".into()),
        ],
    };
    let got: Vec<&str> = paint::mention_spans(&m).into_iter().map(|r| &text[r]).collect();
    assert_eq!(got, ["@Rui Costa", "@Rui", "@everyone", "@Rui"]);
    let none = Message { mentions: Vec::new(), ..m };
    assert!(paint::mention_spans(&none).is_empty(), "o texto sozinho não é menção");
}
