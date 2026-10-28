use super::*;

#[test]
fn short_film_tem_cinco_listas_e_a_review() {
    let mut set = KanbanSet::default();
    let now = Moment(1_800_000_000_000);
    let (id, review) = short_film_into(&mut set, now);
    let k = set.get(id).expect("o Kanban");
    assert_eq!(k.header.name, SHORT_FILM);
    assert_eq!(k.lists.len(), 5);
    assert_eq!(
        review
            .and_then(|r| k.lists.get(&r))
            .map(|l| l.title.as_str()),
        Some("Review")
    );
    let fox = k
        .cards
        .values()
        .find(|c| c.title == "Fox concept sheet")
        .expect("a capa");
    assert!(
        fox.due.as_ref().is_some_and(|d| d.at.0 < now.0 && !d.done),
        "atrasado por `now`"
    );
}

#[test]
fn sky_png_e_um_png_de_256_por_144() {
    let ([w, h], png) = sky_png();
    assert_eq!((w, h), (256, 144));
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(png.ends_with(&[0xAE, 0x42, 0x60, 0x82]), "o CRC do IEND");
}

/// O script do `PH2D_CODE_SMOKE=10` corre limpo sobre a cena 1 e faz o que diz: lê o Kanban, acha
/// o atrasado («Fox concept sheet»), comenta e move «Animate shot 12» para a «Review».
#[test]
fn o_script_da_cena_10_do_editor_corre_sobre_a_cena_1() {
    use ph2d_kanban_script::{Ctx, KanbanScripts};
    let now = Moment(1_793_188_800_000);
    let mut set = KanbanSet::default();
    let (id, review) = short_film_into(&mut set, now);
    let zone = ph2d_kanban_dates::Zone::utc();
    let cx = Ctx {
        now,
        zone: &zone,
        me: Member("Enio".into()),
        at: 1,
    };
    let mut histories = std::collections::BTreeMap::new();
    let r = KanbanScripts::new().run(
        script::KANBAN_NAME,
        script::KANBAN_LUAU,
        &mut set,
        &mut histories,
        &cx,
    );
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    let lines: Vec<&str> = r.printed.iter().map(|p| p.1.as_str()).collect();
    assert_eq!(lines[0], "Short film \u{2014} production\t5\tlistas");
    assert!(
        lines.iter().any(|l| l.contains("Fox concept sheet")),
        "{lines:?}"
    );
    let k = set.get(id).expect("o Kanban");
    let shot = k
        .cards
        .values()
        .find(|c| c.title == "Animate shot 12")
        .expect("o cartão");
    assert_eq!(Some(shot.list), review, "movido para a Review");
    assert!(
        k.comments
            .values()
            .any(|c| c.card == shot.id && c.text.contains("@Rui")),
        "comentado"
    );
}

/// As agulhas do condutor das fotos (`ph2d_app_code::smoke_drive::STEPS_10`) estão no texto.
#[test]
fn a_agulha_do_condutor_da_cena_10_esta_no_script() {
    assert_eq!(script::KANBAN_LUAU.matches(script::NEEDLE).count(), 1);
}
