//! ⛔⛔⛔ **DOIS ORÇAMENTOS DE TEXTO ERAM NÚMEROS ESCRITOS À MÃO, E OS DOIS ESTAVAM ERRADOS —
//! um por ser PEQUENO demais, o outro por ser GRANDE demais.**
//!
//! Achados em 2026-09-19 pela régua nova do `the_label_column_is_one_answer`
//! (`no_text_budget_is_a_bare_literal_at_the_painting_site`): no app inteiro havia **três**
//! chamadas de pintor de texto cujo orçamento era um literal cru, e as três eram defeito.
//!
//! | sítio | orçamento | o espaço REAL | o que se via |
//! |---|---:|---:|---|
//! | secção *Inspect* do Grid Snap | `80,0` | `Line / Neighbors` mede `94,48` | `Line / Neigh…` |
//! | etiqueta de selecção, o NOME | `80,0` | `60,0` até ao emblema | as letras **por cima** dele |
//! | etiqueta de selecção, a POSIÇÃO | `100,0` | `104,0` | apertado sem razão |
//!
//! # ⚠️ O do meio é o instrutivo, e é o oposto do que se procura
//!
//! Um orçamento **maior** que a coluna **não corta nada**: `Platform Player` mede `79,88` e cabia
//! nos `80` que lhe davam, logo era desenhado inteiro — `19,88 px` por baixo do emblema. *A
//! reticência que não aparece não é boa notícia; é a prova de que o número não descreve o espaço.*
//!
//! # ⛔ Porque estes gates pintam de verdade
//!
//! A régua daquele ficheiro é TEXTUAL e afirma que nenhum literal sobrou. **Ela não afirma que o
//! número que ficou no lugar é o certo** — quem o diz é o CENSO DE ELISÕES do `paint` real: o
//! orçamento com que cada texto foi de facto pintado.

use ph2d_editor_core::HeroSelection;
use ph2d_editor_core::grid_snap::inspect;
use ph2d_editor_core::grid_snap::state::GridSnapState;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::label_column_width;
use ph2d_editor_core::property_row::{Seccao, colunas_da_linha};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::paint_selection_overlay;
use ph2d_editor_core::text_elide::elisao::{self, Medido};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Os quatro rótulos da secção *Inspect*, em inglês. ⚠️ Escritos à mão de propósito: são o
/// CONTROLO da lista que o produto lê — *um censo derivado do sujeito mede-se a si próprio*.
const ROTULOS_DO_INSPECT: &[&str] = &["Probe A", "Probe B", "Distance", "Line / Neighbors"];

/// A fonte daquela secção — desde 2026-09-23 é a da PORTA do nome (`paint_label_row`), que é a de
/// toda linha de propriedade do app. Ela era um `12.0` privado da secção, com o nome pintado à mão.
const FONTE_DO_INSPECT: f32 = TypeToken::Sm.px();

/// ⚠️ **As larguras em que a coluna é medida — TRÊS, e cada uma é uma prova de mutação.** A `252`
/// de fábrica é um ponto NEUTRO: ali a secção medida com um campo, com dois, e a que só declara os
/// campos dão TODAS a mesma coluna (`118`), e pintar as sondas com a secção errada passava verde.
/// Medido pela porta (`colunas_da_linha`, coluna do nome):
///
/// | largura | medida, 1 campo | medida, 2 campos | só os campos (2) |
/// |---|---|---|---|
/// | `180` | `86` | `86` | **`82`** |
/// | `252` | `118` | `118` | `118` |
/// | `300` | **`142`** | `131` | **`142`** |
///
/// ⇒ a `180` separa a secção MEDIDA da que não sabe os nomes, e a `300` separa a de um campo da de
/// dois — que é o que dá às linhas de leitura e às das sondas a MESMA coluna.
const LARGURAS_DO_INSPECT: [f32; 3] = [180.0, 252.0, 300.0];

/// Pinta a secção *Inspect* do Grid Snap, com o censo armado, e devolve o que ela mediu.
fn inspect_pintado(largura: f32) -> Vec<Medido> {
    let mut ts = TextSystem::without_system_fonts();
    let mut cena = VectorScene::new();
    let mut hits = HitIndex::default();
    let store = WidgetStore::default();
    let estado = GridSnapState::default();
    elisao::medindo(|| {
        inspect::paint(
            Rect::new(0.0, 0.0, largura, inspect::height()),
            &mut cena,
            &mut ts,
            Theme::default(),
            &mut hits,
            &store,
            &estado,
        );
    })
    .1
}

/// ⭐⭐⭐ **Os quatro rótulos da secção chegam inteiros, e partilham UMA coluna.**
#[test]
fn nenhum_rotulo_da_seccao_de_inspeccao_e_cortado() {
    for largura in LARGURAS_DO_INSPECT {
        rotulos_da_seccao_de_inspeccao_a(largura);
    }
}

fn rotulos_da_seccao_de_inspeccao_a(largura: f32) {
    let medidos: Vec<Medido> = inspect_pintado(largura)
        .into_iter()
        .filter(|m| ROTULOS_DO_INSPECT.contains(&m.texto.as_str()))
        .collect();
    for r in ROTULOS_DO_INSPECT {
        let Some(m) = medidos.iter().find(|m| m.texto == *r) else {
            panic!("⛔ o rótulo {r:?} não foi pintado — vistos: {medidos:?}");
        };
        // A `180` o nome mais largo cede ao controlo e ganha o balão — é a lei de toda linha.
        assert!(
            largura < 252.0 || m.coube(),
            "⛔ {:?} saiu {:?} num orçamento de {}",
            m.texto,
            m.pintado,
            m.largura
        );
    }
    // ⚠️ **A porta do nome faz a pergunta DUAS vezes** (2026-09-23, quando esta secção passou a
    //    pintar por ela): uma contra a COLUNA (o balão, que tem de saber a área do nome) e a
    //    segunda com a largura MEDIDA do que coube, porque o nome encosta à direita. ⇒ a coluna de
    //    um rótulo é a MAIOR largura em que ele foi perguntado — e é ela que tem de ser uma só em
    //    toda a secção, incluindo as duas linhas das SONDAS, que pintam pela porta das várias
    //    componentes e antes tinham uma coluna própria calculada à mão.
    let coluna_de = |texto: &str| {
        medidos
            .iter()
            .filter(|m| m.texto == texto)
            .map(|m| m.largura)
            .fold(0.0_f32, f32::max)
    };
    let primeira = coluna_de(ROTULOS_DO_INSPECT[0]);
    for r in ROTULOS_DO_INSPECT {
        assert_eq!(
            coluna_de(r),
            primeira,
            "⛔ {r:?} recebeu uma coluna ({}) diferente da dos irmãos ({primeira}) — os rótulos \
             desta secção partilham UMA coluna",
            coluna_de(r)
        );
    }
    // As sondas aparecem DUAS vezes (a linha de leitura e a linha dos campos) — as duas pela porta.
    for sonda in ["Probe A", "Probe B"] {
        let n = medidos
            .iter()
            .filter(|m| m.texto == sonda && (m.largura - primeira).abs() < 0.01)
            .count();
        assert_eq!(
            n, 2,
            "⛔ {sonda:?} foi perguntado {n}× contra a coluna da secção — a linha de leitura e a \
             linha dos campos têm de passar as duas pela porta do nome"
        );
    }
    // ⛔ **E o nome ENCOSTA À DIREITA** — a foto do dono (*«painel grid fora do padrão»*) era um
    //    `Probe B` pintado à esquerda por `paint_text`, que pergunta UMA vez, contra a coluna. A porta
    //    pergunta a segunda vez com a largura do que coube, que num nome curto é menor que a coluna.
    //    ⚠️ Um nome que NÃO coube (a `180`) é pintado na coluna inteira com o balão, e as duas
    //    perguntas coincidem — ali a afirmação não tem o que medir.
    for r in ROTULOS_DO_INSPECT {
        let deste = || medidos.iter().filter(|m| m.texto == *r);
        if !deste().all(Medido::coube) {
            continue;
        }
        assert!(
            deste().any(|m| m.largura < primeira - 0.01),
            "⛔ {r:?} só foi perguntado contra a coluna — ele foi pintado À ESQUERDA, fora da porta \
             do nome (`paint_label_row`)"
        );
    }
    for m in &medidos {
        assert_eq!(m.fonte, FONTE_DO_INSPECT, "a fonte da secção mudou");
    }
}

/// ⛔ **E a coluna é a da LEI, pedida com o rótulo mais largo — não um número com folga.**
///
/// Sem esta metade, repor um literal generoso passaria o gate acima: nada seria cortado e a coluna
/// do VALOR pagaria a folga em silêncio.
///
/// ⚠️ **Até 2026-09-23 a coluna ERA o rótulo mais largo** (`label_column_width` da lista, com o vão
/// somado à mão), e o valor desta secção arrancava a `+2,5 px` do resto da janela. Por ordem do dono
/// (*«quero tudo alinhado e padronizado»*, e depois *«painel grid fora do padrão»*, com foto) ela
/// pinta pelas PORTAS da casa: uma [`Seccao`] medida sobre os nomes dela, com DOIS campos (a linha
/// da sonda). ⇒ o gate afirma que a coluna é a da [`colunas_da_linha`] com essa secção — fora de um
/// painel a lei de secção, dentro do Grid Snap a coluna ÚNICA dele — e que o mais largo cabe nela.
#[test]
fn a_coluna_da_seccao_de_inspeccao_e_a_do_rotulo_mais_largo() {
    let medidos = inspect_pintado(252.0);
    let col = medidos
        .iter()
        .find(|m| m.texto == "Line / Neighbors")
        .expect("o rótulo mais largo tem de ser pintado")
        .largura;
    let mut ts = TextSystem::without_system_fonts();
    let mais_largo = label_column_width(
        &mut ts,
        FONTE_DO_INSPECT,
        ROTULOS_DO_INSPECT.iter().copied(),
    );
    let seccao = Seccao::medida(&mut ts, 2, ROTULOS_DO_INSPECT);
    let da_porta = colunas_da_linha(0.0, 252.0, 0.0, ph2d_tokens::ROW_H_PX, seccao)
        .label
        .w;
    assert!(
        (col - da_porta).abs() < 0.01,
        "a secção reservou {col} px e a porta, pedida com o rótulo mais largo ({mais_largo}), dá \
         {da_porta}"
    );
    assert!(
        col >= mais_largo,
        "o rótulo mais largo ({mais_largo}) não cabe em {col}"
    );
}

/// Pinta a etiqueta de selecção com `nome` e devolve o censo.
fn etiqueta_pintada(nome: &str) -> Vec<Medido> {
    let mut ts = TextSystem::without_system_fonts();
    let mut cena = VectorScene::new();
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 1600.0, 900.0));
    let sel = HeroSelection {
        label: nome.to_string(),
        kind: "ENT".to_string(),
        world_pos: (0.0, 0.0),
    };
    elisao::medindo(|| {
        paint_selection_overlay(&layout, &sel, &mut cena, &mut ts, Theme::default());
    })
    .1
}

/// ⭐⭐⭐ **O NOME PÁRA ANTES DO EMBLEMA — e um nome que lá chegaria é CORTADO.**
///
/// ⚠️ O sujeito é `Platform Player` por medição e não por gosto: ele mede **`79,88`**, que cabia
/// nos `80` do literal e **não** cabe na coluna de `60`. *Era exactamente esta palavra que se
/// desenhava inteira por cima do emblema.*
#[test]
fn o_nome_da_etiqueta_de_seleccao_para_antes_do_emblema() {
    const NOME_LONGO: &str = "Platform Player";
    const NOME_CURTO: &str = "Hero";
    let longo = etiqueta_pintada(NOME_LONGO);
    let m = longo
        .iter()
        .find(|m| m.texto == NOME_LONGO)
        .expect("o nome tem de ser pintado");
    assert!(
        !m.coube(),
        "⛔ {NOME_LONGO:?} ({:?}) coube num orçamento de {} — com o literal de `80` ele era \
         desenhado INTEIRO por cima do emblema, que começa a `60` do início dele",
        m.pintado,
        m.largura
    );

    // ⭐ O CONTROLO: um nome que cabe na coluna **não** é tocado. Sem ele, um orçamento de zero
    //   passaria a metade de cima e a etiqueta ficaria sem nome nenhum.
    let curto = etiqueta_pintada(NOME_CURTO);
    let m = curto
        .iter()
        .find(|m| m.texto == NOME_CURTO)
        .expect("o nome curto tem de ser pintado");
    assert!(
        m.coube(),
        "⛔ {NOME_CURTO:?} saiu {:?} num orçamento de {} — a coluna encolheu demais",
        m.pintado,
        m.largura
    );
}

/// ⚠️⚠️ **A GEOMETRIA da etiqueta não se moveu nem um pixel, e é isso que torna a cura segura.**
///
/// O emblema sempre começou a `60` do início do nome; o que mudou foi o nome parar **um vão antes**
/// dele em vez de o atravessar. ⇒ `coluna_do_nome + Spacing::Sm == 60`, e a da posição é o que
/// sobra até ao recuo do lado direito da etiqueta (`220 − 8 − (8 + 60 + 32 + 8) = 104`).
#[test]
fn as_colunas_da_etiqueta_saem_da_caixa_dela() {
    let medidos = etiqueta_pintada("Hero");
    let nome = medidos
        .iter()
        .find(|m| m.texto == "Hero")
        .expect("o nome é pintado");
    assert!(
        (nome.largura + Spacing::Sm.px() - 60.0).abs() < 0.01,
        "a coluna do nome é {} e devia parar um vão ({}) antes dos 60 px em que o emblema começa",
        nome.largura,
        Spacing::Sm.px()
    );
    assert_eq!(nome.fonte, TypeToken::Xs.px(), "a fonte da etiqueta mudou");

    let pos = medidos
        .iter()
        .find(|m| m.texto.starts_with('\u{b7}'))
        .expect("o texto de posição é pintado");
    let esperado =
        220.0 - Spacing::Md.px() - (Spacing::Md.px() + 60.0 + Spacing::Xl3.px() + Spacing::Md.px());
    assert!(
        (pos.largura - esperado).abs() < 0.01,
        "a coluna da posição é {} e o espaço até ao recuo direito é {esperado}",
        pos.largura
    );
}
