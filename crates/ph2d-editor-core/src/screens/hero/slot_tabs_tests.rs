//! A geometria de uma fila de abas e o salto de id.
//!
//! ⚠️ **As leis que precisam do REGISTRY de painéis não moram aqui** — nesta crate o
//! `test_support::ensure_panel_registry` é um `{}`, e uma varredura sobre zero painéis passa sobre
//! nada. Elas vivem em `ph2d-panel-registry-init/tests/`.

use super::*;

/// A coluna da direita **no mínimo** (`PANEL_MIN_W_PX`), que é a largura em que o gesto de fechar a
/// deixa — e a largura exacta do report de 2026-09-07.
fn bar() -> Rect {
    Rect::new(1146.0, 28.0, 220.0, TAB_BAR_H)
}

fn occupant(id: &'static str, node: u64, title: &'static str) -> Occupant {
    Occupant {
        id,
        node: NodeId(node),
        title,
        icon: crate::icons::IconId::Inspector,
    }
}

/// Os três ocupantes da foto do dono, na ordem do registo.
fn three() -> [Occupant; 3] {
    [
        occupant("audio_editor", 11, "Audio Editor"),
        occupant("audio_mixer", 12, "Audio Mixer"),
        occupant("inspector", 13, "Inspector"),
    ]
}

#[test]
fn the_tabs_touch_and_never_overlap() {
    let mut text = TextSystem::without_system_fonts();
    let occ = three();
    let laid = tab_layout(&occ, None, bar(), &mut text);
    assert!(!laid.is_empty());
    for w in laid.windows(2) {
        // ⭐ **Encostam**, sem vão: a lei do grupo — o que separa duas peças é a QUINA.
        assert!(
            (w[0].1.x + w[0].1.w - w[1].1.x).abs() < 0.001,
            "as abas deixaram de encostar: {:?} / {:?}",
            w[0].1,
            w[1].1
        );
    }
    let last = laid.last().unwrap().1;
    assert!(
        last.x + last.w <= bar().x + bar().w + 0.001,
        "a última aba sai da faixa ({last:?})"
    );
}

/// ⭐⭐⭐ **DOIS PAINÉIS DISTINTOS NUNCA MOSTRAM O MESMO RÓTULO** — o defeito do report.
///
/// Com a fila repartida em partes iguais, *«Audio Editor»* e *«Audio Mixer»* elidiam **as duas**
/// para `Audio …` nesta exacta largura de coluna. A cura é a aba medir o próprio nome, e a
/// consequência mede-se aqui: **o rótulo cabe inteiro**, logo não há elisão que o possa colapsar.
///
/// ⚠️ **A asserção do meio é o controlo de vacuidade**: ela prova que esta fixtura É a que falhava
/// — com largura igual, o orçamento fica ABAIXO do nome, que é a condição do defeito.
#[test]
fn a_tab_is_as_wide_as_its_own_name() {
    let mut text = TextSystem::without_system_fonts();
    let occ = three();
    let font = ph2d_tokens::TypeToken::Sm.px();
    // ⚠️ **O recuo de uma aba deixou de ser só o recuo:** o glifo e o vão dele saem do mesmo
    //    orçamento desde 2026-09-09. Medir aqui o recuo antigo daria a esta fixtura mais espaço do
    //    que o produto tem, e o controlo de vacuidade abaixo deixaria de descrever o defeito.
    let inset = tab_pad_x() * 2.0 + tab_icon_px() + ph2d_tokens::icon_label_gap_px();

    let equal_share = bar().w / occ.len() as f32;
    let widest = occ
        .iter()
        .map(|o| text.prefix_width(o.title, font))
        .fold(0.0_f32, f32::max);
    assert!(
        equal_share - inset < widest,
        "controlo partido: a partes iguais cada aba teria {equal_share} px e o maior nome mede \
         {widest} px — se ele coubesse, esta fixtura não reproduz o defeito"
    );

    let widths = tab_widths(&occ, &mut text);
    for (o, w) in occ.iter().zip(&widths) {
        let full = text.prefix_width(o.title, font);
        assert!(
            *w - inset >= full - 0.001,
            "«{}» não cabe na própria aba ({w} px, com {inset} de recuo, para {full} px de nome)",
            o.title
        );
    }
    assert!(
        (widths[0] - widths[1]).abs() > 0.001,
        "dois nomes diferentes deram a MESMA largura de aba"
    );
}

/// ⭐⭐⭐ **Quando não cabem todas, o que desliza é a JANELA — as abas não se mexem.**
///
/// > *«quando se clica na aba ela troca de lugar com a outra aba. não permita isso»* — Enio,
/// > 2026-09-07, no smoke da wave 34.
///
/// A fila está na ordem do REGISTO e o escolhido é o topo do z — duas perguntas, duas respostas.
/// Este teste mede a consequência no transbordo: seja qual for o escolhido, ele aparece, **e os
/// vizinhos dele aparecem na mesma ordem relativa**.
#[test]
fn the_window_slides_to_the_chosen_and_never_reorders_the_row() {
    let mut text = TextSystem::without_system_fonts();
    let many: Vec<Occupant> = (0..12)
        .map(|i| occupant("p", 100 + i, "Background Removal"))
        .collect();

    let all = tab_layout(&many, Some(many[0].node), bar(), &mut text);
    assert!(all.len() < many.len(), "doze abas couberam em 220 px");
    assert!(!all.is_empty(), "o transbordo comeu a fila inteira");

    for pick in [0usize, 5, 11] {
        let laid = tab_layout(&many, Some(many[pick].node), bar(), &mut text);
        let shown: Vec<u64> = laid.iter().map(|(o, _)| o.node.0).collect();
        assert!(
            shown.contains(&many[pick].node.0),
            "o escolhido ({pick}) não foi pintado: {shown:?}"
        );
        // ⭐ **Contígua e na ordem** — uma janela que reordenasse seria a troca de lugar do report.
        let first = many.iter().position(|o| o.node.0 == shown[0]).unwrap();
        let esperado: Vec<u64> = many[first..first + shown.len()]
            .iter()
            .map(|o| o.node.0)
            .collect();
        assert_eq!(
            shown, esperado,
            "a janela mexeu na ordem da fila ao escolher {pick}"
        );
    }
}

/// ⛔ **Um nome mais largo que a coluna inteira é APARADO, nunca deitado fora.**
#[test]
fn a_name_wider_than_the_whole_row_is_trimmed_not_dropped() {
    let mut text = TextSystem::without_system_fonts();
    let occ = [occupant(
        "long",
        7,
        "Um nome absurdamente comprido que nunca caberia numa coluna estreita",
    )];
    let laid = tab_layout(&occ, None, bar(), &mut text);
    assert_eq!(laid.len(), 1, "a única aba desapareceu");
    assert!(
        laid[0].1.w <= bar().w + 0.001,
        "a aba aparada continua mais larga que a faixa ({:?})",
        laid[0].1
    );
}

#[test]
fn an_empty_row_has_no_tabs() {
    let mut text = TextSystem::without_system_fonts();
    assert!(tab_layout(&[], None, bar(), &mut text).is_empty());
    assert!(
        tab_layout(&three(), None, Rect::new(0.0, 0.0, 0.0, 0.0), &mut text).is_empty(),
        "uma faixa de área zero pintou abas"
    );
}

/// ⛔ **O controlo de que o salto de id é uma BIJECÇÃO** — ele não pode criar colisões que o
/// espaço de ids de painel já não tivesse, e não pode devolver o próprio id do painel.
#[test]
fn the_tab_id_is_a_bijection_and_never_the_panels_own_id() {
    let ids = [NodeId(1), NodeId(2), NodeId(0xdead_beef), NodeId(u64::MAX)];
    for a in ids {
        assert_ne!(tab_node_id(a), a, "a aba herdou o id do painel ({a:?})");
        assert_eq!(tab_node_id(tab_node_id(a)), a, "o salto não é involutivo");
        for b in ids {
            if a != b {
                assert_ne!(
                    tab_node_id(a),
                    tab_node_id(b),
                    "dois painéis distintos deram a mesma aba"
                );
            }
        }
    }
}

/// ⭐⭐⭐ **NO PISO A ABA É O GLIFO — e nunca um `…` sozinho.**
///
/// > *«as abas … não reduzem de tamanho e colocam os `...` no nome»* — Enio, 2026-09-08.
///
/// O encolhimento chegou na wave 38; o nome ficou a evaporar até às reticências, **iguais em
/// todas**. Aqui mede-se o que sobra à largura mínima: o desenho do painel, dentro da aba e
/// centrado, e **nenhum** rótulo.
///
/// ⚠️ **A 1.ª asserção é o controlo de vacuidade** — ela prova que esta largura É a do defeito: se
/// o nome coubesse, a fixtura não produzia o fenómeno e as outras duas mediriam silêncio.
#[test]
fn a_squeezed_tab_is_its_glyph_and_never_a_lone_ellipsis() {
    let mut text = TextSystem::without_system_fonts();
    let title = "Inspector";
    let font = ph2d_tokens::TypeToken::Sm.px();
    let r = Rect::new(0.0, 0.0, tab_floor_w(), TAB_BAR_H);
    let budget = r.w - tab_pad_x() * 2.0 - tab_icon_px() - ph2d_tokens::icon_label_gap_px();
    assert!(
        text.prefix_width(title, font) > budget,
        "controlo partido: «{title}» cabe em {budget} px no piso — esta fixtura não reproduz o \
         nome espremido"
    );

    let f = tab_face(r, title, &mut text);
    assert!(
        f.label.is_none(),
        "a aba no piso ainda escreve {:?} — um rótulo aqui é o `…` do report, ou pior, texto a \
         transbordar por cima da vizinha",
        f.label
    );
    assert!(
        (f.icon.w - tab_icon_px()).abs() < 0.001 && (f.icon.h - tab_icon_px()).abs() < 0.001,
        "o glifo não tem o lado do ícone em linha desta casa: {:?}",
        f.icon
    );
    assert!(
        f.icon.x >= r.x - 0.001 && f.icon.x + f.icon.w <= r.x + r.w + 0.001,
        "o glifo saiu da própria aba: {:?} em {r:?}",
        f.icon
    );
    assert!(
        ((f.icon.y + f.icon.h * 0.5) - (r.y + r.h * 0.5)).abs() < 0.001,
        "o glifo não está centrado na fila: {:?}",
        f.icon
    );
}

/// ⭐⭐ **Com espaço, a aba mostra as DUAS coisas — e o vão entre elas é o da porta.**
///
/// ⚠️ O vão vem de [`ph2d_tokens::icon_label_gap_px`] (`4`, veredito do dono de 2026-09-07 *«para o
/// app todo»*), e é medido aqui em GEOMETRIA: um censo textual vê a chamada, não a distância.
#[test]
fn a_roomy_tab_shows_the_glyph_and_the_whole_name() {
    let mut text = TextSystem::without_system_fonts();
    let title = "Inspector";
    let w = tab_natural_w(title, &mut text);
    let r = Rect::new(0.0, 0.0, w, TAB_BAR_H);

    let f = tab_face(r, title, &mut text);
    let (shown, x, _) = f.label.expect("na largura natural o nome tem de aparecer");
    assert_eq!(shown, title, "o nome foi cortado na largura que o mede");
    assert!(
        (x - (f.icon.x + f.icon.w + ph2d_tokens::icon_label_gap_px())).abs() < 0.001,
        "o nome não começa a um vão do glifo: glifo {:?}, nome em {x}",
        f.icon
    );
    assert!(
        f.icon.x >= r.x + tab_pad_x() - 0.001,
        "o glifo entrou no recuo da aba: {:?} em {r:?}",
        f.icon
    );
}

/// ⭐⭐⭐ **EM TODA LARGURA ENTRE O PISO E A NATURAL, o que se pinta ou é o nome ou não é nada.**
///
/// ⛔⛔ **Este teste nasceu de uma mutação SOBREVIVENTE.** A fixtura do piso não chega às duas
/// guardas que decidem se o rótulo sai: a `Inspector` num quadrado de 22 px deixa o orçamento
/// **negativo**, e o `budget > 0` sozinho já devolve `None`. Apagar as duas guardas deixava os
/// dois gates do piso VERDES. *Uma fixtura sem o fenómeno mede silêncio* — e o fenómeno vive nas
/// larguras INTERMÉDIAS, onde o orçamento é positivo e pequeno.
///
/// Ali o [`crate::text_elide::fit`] tem **duas** saídas que não dizem nada, e as duas shipariam:
/// o `…` sozinho (o report do dono) e o **texto cru**, que ele devolve de propósito quando nem as
/// reticências cabem — *«é melhor transbordar visivelmente do que desaparecer»*, lei certa numa
/// caixa isolada e errada numa fila, onde transbordar é escrever na aba do vizinho.
#[test]
fn between_the_floor_and_the_full_name_a_tab_never_shows_a_useless_label() {
    let mut text = TextSystem::without_system_fonts();
    let title = "Inspector";
    let font = ph2d_tokens::TypeToken::Sm.px();
    let overhead = tab_pad_x() * 2.0 + tab_icon_px() + ph2d_tokens::icon_label_gap_px();
    let natural = tab_natural_w(title, &mut text);

    let (mut with_label, mut without, mut useless_raw) = (0usize, 0usize, 0usize);
    let mut w = tab_floor_w();
    while w <= natural + 0.001 {
        let r = Rect::new(0.0, 0.0, w, TAB_BAR_H);
        let budget = w - overhead;
        // O que o elidor sozinho devolveria — a saída CRUA, sem as guardas desta casa.
        if budget > 0.0 {
            let raw = crate::text_elide::fit(&mut text, title, font, budget);
            if !raw.starts_with('I') || text.prefix_width(&raw, font) > budget {
                useless_raw += 1;
            }
        }

        let f = tab_face(r, title, &mut text);
        assert!(
            f.icon.x >= r.x - 0.001 && f.icon.x + f.icon.w <= r.x + r.w + 0.001,
            "o glifo saiu da aba a {w} px: {:?}",
            f.icon
        );
        match &f.label {
            None => without += 1,
            Some((s, x, _)) => {
                with_label += 1;
                assert!(
                    s.starts_with('I'),
                    "a {w} px a aba escreve {s:?}, que não é o princípio de «{title}»"
                );
                assert!(
                    text.prefix_width(s, font) <= budget + 0.001,
                    "a {w} px o rótulo {s:?} transborda o orçamento de {budget} px — ele vai \
                     escrever por cima da aba vizinha"
                );
                assert!(
                    *x >= f.icon.x + f.icon.w - 0.001,
                    "a {w} px o nome começa em cima do glifo"
                );
            }
        }
        w += 0.5;
    }

    // ⚠️ **Os TRÊS controlos de vacuidade.** Sem eles a varredura pode não cruzar a fronteira, ou
    //    nunca chegar às guardas — que foi exactamente como a mutação sobreviveu.
    assert!(
        without > 0,
        "nenhuma largura desta varredura esconde o nome"
    );
    assert!(
        with_label > 0,
        "nenhuma largura desta varredura mostra o nome"
    );
    assert!(
        useless_raw > 0,
        "a varredura nunca produziu uma saída inútil do elidor — ela não exercita as guardas que \
         este teste existe para medir"
    );
}

/// ⭐⭐⭐ **UMA FILA QUE TRANSBORDA RESERVA AS DUAS SETAS** — e as abas não entram nelas.
///
/// ⚠️ **A ordem da decisão é load-bearing:** primeiro pergunta-se se TODAS cabem na faixa
/// **inteira**; só depois de a resposta ser não é que as setas nascem. Reservar-lhes espaço antes
/// faria uma fila que cabia deixar de caber por causa de uma saída que ela não usa — e o gate
/// irmão abaixo é o que prende essa ordem.
#[test]
fn a_row_that_overflows_reserves_the_two_arrows() {
    use crate::screens::hero::slot_tabs_overflow as ovf;
    let mut text = TextSystem::without_system_fonts();
    let many: Vec<Occupant> = (0..14)
        .map(|i| occupant("p", 300 + i, "Background Removal"))
        .collect();
    let bar = bar();

    let plan = tab_plan(&many, Some(many[0].node), bar, &mut text).expect("há fila");
    assert!(
        plan.hidden_after > 0,
        "controlo partido: catorze abas couberam em {} px — esta fixtura não transborda",
        bar.w
    );
    assert!(
        (plan.bar.w - (bar.w - ovf::arrow_w() * 2.0)).abs() < 0.001,
        "a faixa das abas não recuou o espaço das duas setas: {:?}",
        plan.bar
    );

    // ⛔ Nenhuma aba pintada invade o território das setas.
    let (prev, _next) = ovf::arrow_rects(bar);
    for (_, r) in tab_layout(&many, Some(many[0].node), bar, &mut text) {
        assert!(
            r.x + r.w <= prev.x + 0.001,
            "uma aba entrou por baixo das setas: {r:?} contra {prev:?}"
        );
    }
}

/// ⭐ **E uma fila que CABE não tem setas** — dois controlos mudos seriam chrome morto.
#[test]
fn a_row_that_fits_keeps_the_whole_band() {
    let mut text = TextSystem::without_system_fonts();
    let occ = three();
    let bar = bar();
    let plan = tab_plan(&occ, None, bar, &mut text).expect("há fila");
    assert_eq!((plan.hidden_before, plan.hidden_after), (0, 0));
    assert!(
        (plan.bar.w - bar.w).abs() < 0.001,
        "a faixa encolheu sem haver transbordo: {:?}",
        plan.bar
    );
}

/// ⛔ **O id de uma seta não é o de nenhuma outra coisa** — e as doze são distintas.
#[test]
fn the_arrow_ids_are_twelve_distinct_ids() {
    use crate::screens::hero::slot_tabs_overflow as ovf;
    use crate::screens::slot::Slot;
    let mut all = Vec::new();
    for s in Slot::ALL {
        let (p, n) = ovf::arrow_ids(s);
        assert_eq!(ovf::arrow_of(p), Some((s, -1)));
        assert_eq!(ovf::arrow_of(n), Some((s, 1)));
        all.push(p);
        all.push(n);
    }
    let mut sorted = all.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), all.len(), "duas setas partilham um id");
    for o in three() {
        assert_eq!(ovf::arrow_of(o.node), None);
        assert_eq!(ovf::arrow_of(tab_node_id(o.node)), None);
    }
}
