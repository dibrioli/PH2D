//! Gates da **tabela viva** (plano UI/UX W8b.2).

use super::*;
use ph2d_editor_core::ids;

/// **A lista viva É a lista gerada, na ordem dela.**
///
/// ⚠️ O oráculo é `generated::ROWS` e não uma cópia à mão: uma segunda lista aqui divergiria no
/// dia em que o artista acrescentasse um filho, e o gate ficaria verde sobre o painel errado.
#[test]
fn the_rows_are_the_generated_table_in_order() {
    let live = baked();
    assert_eq!(live.len(), crate::generated::ROWS.len());
    for (r, c) in live.iter().zip(crate::generated::ROWS) {
        assert_eq!(r.kind, c.kind);
        assert_eq!(r.label, c.label);
        assert_eq!(r.key, c.key);
        assert_eq!(r.rgba, c.rgba);
        // ⚠️ Compara a CURVA reconstituída com o texto de origem, e não os dois textos: é a
        // travessia que pode perder, e é ela que decide se o painel desenha o glifo do artista.
        assert_eq!(
            r.icon.as_ref().map(ph2d_vector::BezPath::to_svg),
            c.icon.map(str::to_string),
            "a curva reconstituida nao e' a que o gerado carrega"
        );
        // ⚠️ O slug faz a viagem inversa: ele é resolvido para `IconId`, e o que se compara é o
        // slug DE VOLTA — um `from_slug` que devolvesse o ícone da posição errada sangraria aqui.
        assert_eq!(
            r.icon_id.map(ph2d_editor_core::icons::IconId::slug),
            c.icon_slug,
            "o icone escolhido nao voltou do proprio slug"
        );
    }
}

/// **O id de cada row sai da CHAVE, e nenhum colide com o outro nem com o chrome.**
///
/// ⚠️ O gerador **não cunha ids** (ver o doc de `ids::authored_row_id`), então a unicidade da
/// família contra o chrome estático não é verificável na foundational — as chaves REAIS só
/// existem aqui. É o mesmo arranjo do `wet_tuning_ids_dont_collide`.
#[test]
fn the_row_ids_do_not_collide_with_each_other_nor_with_the_chrome() {
    let mut seen: Vec<(ph2d_a11y::NodeId, String)> = vec![
        (ids::AUTHORED_PANEL, "panel".to_string()),
        (ids::AUTHORED_CLOSE, "close".to_string()),
        (ids::AUTHORED_DRAG_HANDLE, "drag".to_string()),
        (ids::AUTHORED_RESIZE_HANDLE, "resize".to_string()),
        (ids::AUTHORED_RESIZE_HANDLE_BL, "resize_bl".to_string()),
    ];
    for r in baked() {
        // ⚠️ Duas rows de mesmo RÓTULO têm a mesma chave de propósito (ver o doc de
        // `duplicate_keys`), então o que se afirma é: chaves distintas dão ids distintos.
        if seen.iter().any(|(_, k)| k == &r.key) {
            continue;
        }
        assert!(
            !seen.iter().any(|(id, _)| *id == r.id),
            "a row `{}` colide com um id ja' usado",
            r.key
        );
        seen.push((r.id, r.key.clone()));
    }
}

/// **Só quem RESPONDE é controle** — a segunda metade da lei do §4.
///
/// ⚠️ Afirmada por TIPO, e não pela lista da cena: a cena pode mudar de desenho, a lei não. Um
/// `Divider` registado acenderia sob o rato e não faria nada — o item-de-menu-morto com outro
/// nome; e um `Slider` NÃO registado ficaria desenhado e imóvel sob o dedo.
///
/// ⚠️ **E as duas listas TÊM de cobrir o catálogo inteiro** — a última asserção é o que impede
/// esta enumeração de apodrecer. Sem ela, um tipo novo não aparece em lista nenhuma, ninguém
/// decide de que lado ele fica, e o gate segue verde afirmando o que já sabia.
#[test]
fn only_the_kinds_that_respond_are_controls() {
    let is = |kind| {
        Row {
            kind,
            label: "x".to_string(),
            key: "x".to_string(),
            id: ph2d_a11y::NodeId(1),
            rgba: None,
            icon: None,
            icon_id: None,
            options: Vec::new(),
        }
        .is_control()
    };
    let responds = [
        WidgetKind::Button,
        WidgetKind::Toggle,
        WidgetKind::Checkbox,
        WidgetKind::Slider,
        WidgetKind::TextInput,
        WidgetKind::Tag,
        WidgetKind::ListItem,
        WidgetKind::NumberInput,
        // ⚠️ Um botão de ÍCONE é um botão: o que muda é a face, nunca o gesto.
        WidgetKind::IconButton,
        // ⚠️ **A família de LISTA responde, e a pergunta que ela levanta é a do ALVO:** o gesto
        // não muda o valor de UM controle, ele escolhe QUAL das N opções. O painel registra a row
        // e o clique escolhe — o mesmo arranjo que o `seg_row` dos painéis escritos à mão usa.
        WidgetKind::Tabs,
        WidgetKind::RadioGroup,
        WidgetKind::SegmentedAdaptive,
        // ⚠️ O dropdown responde por **duas** superfícies: o chip (que abre) e cada opção da
        // lista. Só o chip é uma row — as opções não estão na tabela, e por isso a `is_control`
        // não sabe nada sobre elas (ver `seam_authored_popover`).
        WidgetKind::Dropdown,
    ];
    let draws = [
        WidgetKind::ProgressBar,
        WidgetKind::SectionHeader,
        WidgetKind::Card,
        WidgetKind::Spinner,
        WidgetKind::Divider,
        WidgetKind::LevelMeter,
        WidgetKind::ColorSwatch,
    ];
    for k in responds {
        assert!(is(k), "{k:?} responde a um gesto e nao e' controle");
    }
    for k in draws {
        assert!(!is(k), "{k:?} so' desenha e foi tratado como controle");
    }
    assert_eq!(
        responds.len() + draws.len(),
        WidgetKind::ALL.len(),
        "um tipo novo entrou no catalogo e ninguem decidiu se ele RESPONDE"
    );
}

/// **Rótulos repetidos são CONTADOS, nunca renomeados.**
///
/// ⚠️ Inventar um sufixo (`opacity_2`) daria uma chave que o artista não escreveu, não vê e que
/// mudaria sozinha ao reordenar os filhos. O painel diz que há repetidos; quem desempata é ele, na
/// Hierarquia, com o nome.
#[test]
fn duplicate_labels_are_counted_not_renamed() {
    // A cena que shipa não tem repetidos — este é o CONTROLE.
    assert_eq!(duplicate_keys(), 0);
}

/// **A pintura entrega o ESTADO DO STORE, e não `None`.**
///
/// ⚠️ Arch-gate sobre o próprio `paint.rs`, e ele existe porque uma mutação sobreviveu a tudo o
/// mais: trocar `store.get(row.id)` por `None` deixa **todos** os gates de comportamento verdes —
/// as rows continuam vivas, o arrasto continua a mover o valor no store — e cada slider passa a
/// pintar a meio para sempre. O defeito só aparece numa screenshot, que é o modo de falha mais
/// caro que este repo conhece.
///
/// A metade complementar mora na porta (`the_live_state_reaches_the_paint` e
/// `the_colour_reaches_the_paint`, em `skin/tests.rs`): lá se prova que o estado e a cor MUDAM a
/// cena; aqui, que este painel os entrega.
///
/// ⚠️ **A janela é a CHAMADA, não uma contagem de bytes.** A versão anterior lia 400 bytes depois
/// do `paint_widget_skin_with(` — um número que envelhece no dia em que um argumento entra, e que
/// já custou a esta linha dois arch-gates reescritos (23/07). Aqui ela vai até o `);` que fecha,
/// que é o que a asserção de facto quer dizer.
#[test]
fn the_paint_hands_what_it_knows_to_the_skin() {
    let src = include_str!("paint.rs");
    // Controle positivo: uma varredura vazia tornaria este gate verde por vácuo.
    assert!(
        src.contains("paint_widget_skin_with("),
        "o paint nao chama a porta da pele"
    );
    let call = src
        .find("paint_widget_skin_with(\n")
        .expect("a chamada da pele sumiu do paint");
    let end = src[call..]
        .find("\n        );")
        .expect("a chamada da pele nao fecha");
    let window = &src[call..call + end];
    assert!(
        window.contains("store.get(row.id)"),
        "a pele recebe um estado que nao e' o do store — os controles pintariam o valor de \
         PREVIA para sempre, com toda a suite verde"
    );
    assert!(
        window.contains("rgba: row.rgba"),
        "a pele recebe um parametro que nao e' o da row — a swatch pintaria o xadrez para \
         sempre, com o pintor correto e toda a suite verde"
    );
    assert!(
        window.contains("icon: icon_glyph(row.icon_id, row.icon.as_ref())"),
        "a pele recebe um glifo que nao e' o da row, ou a precedencia foi re-escrita aqui — no \
         primeiro caso o botao pintaria a moldura vazia para sempre; no segundo, o canvas e o \
         painel poderiam escolher rotas diferentes, com toda a suite verde"
    );
}

/// **Quem DOBRA é exactamente o cabeçalho de seção — e ele NÃO é um controle.**
///
/// ⚠️ As duas metades, e a segunda é a que mantém o modelo: se `folds_a_section` e `is_control`
/// coincidissem em alguém, aquele tipo emitiria um intent ao ser dobrado (uma edição do documento
/// causada por um gesto de VISTA) ou ganharia um `InteractiveState` que o despacho de colapso
/// ignora — e o clique cairia no `switch` errado.
#[test]
fn only_the_section_header_folds_and_it_is_never_a_control() {
    let row = |kind| Row {
        kind,
        label: "x".to_string(),
        key: "x".to_string(),
        id: ph2d_a11y::NodeId(1),
        rgba: None,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    };
    for kind in WidgetKind::ALL {
        let r = row(kind);
        assert_eq!(
            r.folds_a_section(),
            kind == WidgetKind::SectionHeader,
            "{kind:?} discorda sobre dobrar uma secao"
        );
        assert_eq!(
            r.opens_a_picker(),
            kind == WidgetKind::ColorSwatch,
            "{kind:?} discorda sobre abrir o picker"
        );
        // ⚠️ **As três são MUTUAMENTE EXCLUSIVAS, e é isso que o gate afirma agora.** A versão
        // anterior repetia a soma (`wants_pointer == is_control || folds_a_section`) — uma cópia
        // da implementação, que só sabe dizer *"a soma mudou"* e foi exactamente o que ela disse
        // no dia em que a swatch entrou. O que importa é que **um clique tem UM significado**: um
        // tipo que fosse controle E cabeçalho emitiria um intent ao ser dobrado; um que fosse
        // controle E picker abriria o picker e nunca chegaria ao próprio switch.
        let claims = [r.is_control(), r.folds_a_section(), r.opens_a_picker()]
            .into_iter()
            .filter(|b| *b)
            .count();
        assert!(
            claims <= 1,
            "{kind:?} reclama {claims} significados para o mesmo clique"
        );
        assert_eq!(
            r.wants_pointer(),
            claims == 1,
            "{kind:?}: a porta do ponteiro discorda das perguntas que ela soma"
        );
    }
}

/// **O DOCUMENTO ABERTO VENCE O CÓDIGO COLADO** — a lei da tabela viva (report do Enio,
/// 2026-08-09: *"por que fechar o app e reabrir? isso não pode ser num app pro"*).
///
/// ⚠️ E a metade que importa tanto quanto é a VOLTA: publicar `None` devolve o painel à tabela
/// compilada. Sem ela, fechar o documento deixaria o painel a descrever uma árvore que já não
/// existe — e um build sem documento autorado nenhum não teria painel para mostrar.
#[test]
fn the_open_document_wins_over_the_pasted_code_and_giving_it_back_restores_it() {
    let baked_len = with_rows(<[Row]>::len);
    assert!(
        baked_len > 0,
        "a tabela compilada esta' vazia — o controle nao vale"
    );

    let mine = vec![Row {
        kind: WidgetKind::Slider,
        label: "Ao vivo".to_string(),
        key: "ao_vivo".to_string(),
        id: ph2d_editor_core::ids::authored_row_id("ao_vivo"),
        rgba: None,
        icon: None,
        icon_id: None,
        options: Vec::new(),
    }];
    set_live_rows(Some(mine));
    with_rows(|t| {
        assert_eq!(t.len(), 1, "a tabela viva nao venceu a compilada");
        assert_eq!(t[0].label, "Ao vivo");
    });

    set_live_rows(None);
    assert_eq!(
        with_rows(<[Row]>::len),
        baked_len,
        "tirar o documento nao devolveu o painel a' tabela compilada"
    );
}

/// **Uma tabela viva VAZIA não é a ausência de uma** — o painel de um documento sem controles
/// mostra-se vazio, e não o painel compilado de outra pessoa.
#[test]
fn an_empty_live_table_is_not_the_absence_of_one() {
    set_live_rows(Some(Vec::new()));
    assert_eq!(
        with_rows(<[Row]>::len),
        0,
        "um documento com zero rows caiu na tabela compilada — o `Option` colapsou"
    );
    set_live_rows(None);
}
