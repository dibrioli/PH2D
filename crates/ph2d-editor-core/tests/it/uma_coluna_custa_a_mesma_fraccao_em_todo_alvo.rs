//! ⭐⭐⭐ **A MESMA DECISÃO DE DESENHO CUSTA A MESMA FRACÇÃO EM TODO ALVO.**
//!
//! > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
//!
//! # ⛔⛔ O buraco, medido antes de existir cura
//!
//! As duas colunas são `308 + 304 = 612 px` **absolutos**, autorados contra a janela de referência
//! que o `tokens.json` declara ([`HERO_VIEWPORT_W`], `1366`). ⇒ elas valem:
//!
//! | alvo | largura | as duas colunas | |
//! |---|---:|---:|---|
//! | iPad 12,9" | `1366` | `44,8 %` | a decisão, tal como foi tomada |
//! | iPad 11" | `1194` | `51,3 %` | |
//! | iPad mini | `1133` | **`54,0 %`** | `+9,2` pontos sobre a decisão |
//!
//! *A mesma decisão de desenho custa `20 %` mais no aparelho mais pequeno, e nenhum documento
//! dizia isso* — [`medicoes/06 §1`](../../../../docs/UI_New_and_Simple/medicoes/06_o_orcamento_de_ecra_em_tablet.md).
//!
//! # ⭐ A lei, e porque ela é DERIVADA e não escolhida
//!
//! A fracção de referência **não é um número novo**: ela é `308 / 1366` e `304 / 1366`, os dois
//! tokens a dividir pela janela para que foram autorados. ⇒ a lei diz *«a coluna de fábrica nunca
//! ocupa mais fracção da janela do que ocupa na referência»*, e isso é a decisão que já foi tomada,
//! aplicada onde ela ainda não chegava.
//!
//! # ⛔⛔ E ela é um TECTO, nunca uma escala — a diferença tem um número
//!
//! Escalar nos dois sentidos poria a coluna a **crescer** num ecrã grande: na janela de `1 930 px`
//! em que o dono trabalha, `1930 × 308/1366 = 435 px` por coluna — **`870`** contra os `612` de
//! hoje, ou seja *a cura tornaria o app pior exactamente onde ele é usado todos os dias*.
//!
//! ⇒ acima da referência a lei é **inerte por construção**, e é isso que a 3.ª metade afirma.
//!
//! # ⚠️ Ela vale para a largura de FÁBRICA e não para a que o artista arrastou
//!
//! Uma escolha explícita é dele; apertá-la seria o *«aceita e mente»* que o `CLAUDE.md` §0.0
//! proíbe. ⛔ O que isto NÃO cura fica declarado: um `dock_width_choice` gravado num ecrã largo
//! continua a valer o que vale num estreito.

use ph2d_editor_core::screens::layout::{ChromeBands, DockSide, HERO_VIEWPORT_W};

/// Os três tablets, em pontos lógicos — a mesma lista do gate do orçamento.
const TABLETS: [(&str, f32); 3] = [
    ("iPad 12.9", 1366.0),
    ("iPad 11", 1194.0),
    ("iPad mini", 1133.0),
];

/// A folga: a aritmética é `f32` sobre tokens, e meio pixel não é uma regressão.
const FOLGA_PP: f32 = 0.1;

/// ⭐⭐⭐ **As duas colunas de fábrica custam a MESMA fracção nos três alvos.**
#[test]
fn uma_coluna_de_fabrica_custa_a_mesma_fraccao_nos_tres_alvos() {
    let fraccao = |w: f32| {
        let l = ChromeBands::default_dock_w(DockSide::Left, w);
        let r = ChromeBands::default_dock_w(DockSide::Right, w);
        100.0 * (l + r) / w
    };
    let referencia = fraccao(HERO_VIEWPORT_W);
    for (nome, w) in TABLETS {
        let f = fraccao(w);
        println!("  {nome:11} {w:6.0} px → as duas colunas custam {f:5.1} %");
        assert!(
            (f - referencia).abs() <= FOLGA_PP,
            "{nome}: as duas colunas custam {f:.1} % da janela e na referência custam \
             {referencia:.1} % — a mesma decisão de desenho passou a custar mais num alvo mais \
             pequeno, que é exactamente o que esta lei existe para impedir"
        );
    }
}

/// ⭐⭐ **O CONTROLO: sem a lei, o alvo mais pequeno paga mais** — senão a metade de cima seria
/// verde sobre um mundo em que as três fracções são iguais por acaso.
#[test]
fn e_sem_a_lei_o_alvo_mais_pequeno_pagaria_mais() {
    let bruto =
        |w: f32| 100.0 * (ChromeBands::DEFAULT.left_dock_w + ChromeBands::DEFAULT.right_dock_w) / w;
    let (_, mini) = TABLETS[2];
    assert!(
        bruto(mini) > bruto(HERO_VIEWPORT_W) + 5.0,
        "o controlo deixou de conter o fenómeno: sem a lei o iPad mini pagava {:.1} % contra \
         {:.1} % da referência, e a diferença deixou de ser visível — esta régua passou a medir \
         um mundo onde o defeito não existe",
        bruto(mini),
        bruto(HERO_VIEWPORT_W),
    );
}

/// ⛔⛔ **ACIMA da referência a lei é INERTE, ao pixel** — a metade que impede a cura de piorar o
/// ecrã em que o dono de facto trabalha.
#[test]
fn acima_da_referencia_a_lei_nao_toca_em_nada() {
    // ⚠️ `1 930` é a largura da janela MEDIDA na bancada do dono (as fotos das cenas desta jornada).
    for w in [HERO_VIEWPORT_W, 1930.0, 2560.0, 3840.0] {
        for (side, esperado) in [
            (DockSide::Left, ChromeBands::DEFAULT.left_dock_w),
            (DockSide::Right, ChromeBands::DEFAULT.right_dock_w),
        ] {
            let got = ChromeBands::default_dock_w(side, w);
            assert!(
                (got - esperado).abs() < f32::EPSILON,
                "numa janela de {w:.0} px a coluna {side:?} mediu {got} e o token diz {esperado} \
                 — a lei tem de ser inerte acima da referência, senão ela FAZ CRESCER o chrome \
                 no ecrã grande em que o app é usado"
            );
        }
    }
}

/// ⚠️ **E ela nunca desce abaixo do mínimo do PAINEL** — abaixo dele o cabeçalho e uma linha
/// deixam de caber juntos, e uma coluna que não sabe desenhar-se é pior do que uma coluna larga.
#[test]
fn ela_para_no_minimo_do_painel() {
    let min = ph2d_tokens::PANEL_MIN_W_PX;
    for w in [600.0, 800.0, 975.0] {
        for side in [DockSide::Left, DockSide::Right] {
            let got = ChromeBands::default_dock_w(side, w);
            assert!(
                got >= min,
                "numa janela de {w:.0} px a coluna {side:?} encolheu para {got} e o mínimo do \
                 painel é {min}"
            );
        }
    }
    // ⭐ O CONTROLO da própria cerca: nos três tablets o mínimo **não** morde, logo a metade de
    //   cima deste ficheiro mede a lei e não o clamp.
    for (nome, w) in TABLETS {
        for side in [DockSide::Left, DockSide::Right] {
            assert!(
                ChromeBands::default_dock_w(side, w) > min,
                "{nome}: o mínimo do painel passou a morder, logo a fracção medida acima é a do \
                 CLAMP e não a da lei — os números do doc têm de ser refeitos"
            );
        }
    }
}

/// ⭐⭐⭐ **A PORTA DO PRODUTO LÊ A LEI** — sem esta metade, desligar o fio deixava os gates de cima
/// **verdes**, porque nenhum deles percorre a rota que o app percorre.
///
/// ⚠️ É a armadilha que esta jornada já pagou DUAS vezes (o gate da paleta entrou pelo chrome, e o
/// gate do orçamento reconstruía as bandas): *um gate que chama a lei em vez de percorrer a rota
/// afirma que a lei existe, nunca que o produto a usa.*
#[test]
fn a_porta_do_store_le_a_lei() {
    use ph2d_editor_core::interaction::WidgetStore;
    let store = WidgetStore::default();
    for (nome, w) in TABLETS {
        for side in [DockSide::Left, DockSide::Right] {
            let porta = store.dock_width(side, w);
            let lei = ChromeBands::default_dock_w(side, w);
            assert!(
                (porta - lei).abs() < f32::EPSILON,
                "{nome}: a porta do produto devolve {porta} para a coluna {side:?} e a lei diz \
                 {lei} — o fio entre as duas está cortado, e os gates de cima não o veem"
            );
        }
    }
}

/// ⚠️ **E a ESCOLHA do artista atravessa a porta intacta** — o CONTROLO negativo da metade acima.
///
/// ⛔ Sem ele, uma implementação que escalasse TUDO (a escolha incluída) passaria o gate anterior
/// e apertaria em silêncio um número que o artista arrastou com a mão.
#[test]
fn e_a_escolha_do_artista_atravessa_intacta() {
    use ph2d_editor_core::interaction::WidgetStore;
    use ph2d_editor_core::screens::layout::DockSide as DS;
    let mut store = WidgetStore::default();
    // ⚠️ Um número que a lei mudaria se ela lhe tocasse: no mini a fábrica encolhe para `~255`.
    let escolhido = 300.0;
    store.set_dock_width(DS::Left, escolhido);
    let (_, mini) = TABLETS[2];
    let got = store.dock_width(DS::Left, mini);
    assert!(
        (got - escolhido).abs() < f32::EPSILON,
        "a escolha do artista era {escolhido} e a porta devolveu {got} numa janela de {mini:.0} px \
         — a lei apertou uma decisão explícita dele, que é o «aceita e mente» que o §0.0 proíbe"
    );
    // ⭐ O controlo: sem escolha, a MESMA janela dá outro número.
    let sem_escolha = WidgetStore::default().dock_width(DS::Left, mini);
    assert!(
        (sem_escolha - escolhido).abs() > 1.0,
        "a fixtura não contém o fenómeno: sem escolha a porta devolve {sem_escolha}, que é o \
         mesmo número que o artista escolheu — este gate não distingue as duas coisas"
    );
}

/// ⛔⛔ **E O QUADRO ENTREGA A LARGURA DA JANELA** — o último elo, e o único que um teste não
/// alcança por chamada: o `frame_layout` é `pub(super)`.
///
/// ⚠️ **A agulha é o BRAÇO INTEIRO e não o nome da função:** procurar `dock_width` sozinho ficaria
/// verde com `dock_width(side, 1366.0)` escrito à mão — *que é exactamente a regressão que este
/// gate existe para apanhar*, porque ela devolve o produto ao estado de antes desta wave sem
/// mexer numa linha de lei.
#[test]
fn e_o_quadro_entrega_a_largura_da_janela() {
    const QUADRO: &str = include_str!("../../src/screens/hero/frame_layout.rs");
    for side in ["Left", "Right"] {
        let agulha = format!(".dock_width(crate::screens::layout::DockSide::{side}, viewport.w)");
        assert!(
            QUADRO.contains(&agulha),
            "o `frame_layout` deixou de entregar a largura da janela à coluna {side} — a lei fica \
             viva e o produto volta ao absoluto, com todos os outros gates deste ficheiro verdes"
        );
    }
}
