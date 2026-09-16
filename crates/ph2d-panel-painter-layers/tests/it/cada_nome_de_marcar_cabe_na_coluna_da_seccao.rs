//! ⭐⭐⭐ **O NOME DE UMA CAIXA DE MARCAR CABE NA COLUNA DA SECÇÃO DELA.**
//!
//! ⛔⛔ Até 2026-09-16 nenhuma das **34** linhas de marcar deste painel declarava secção, logo
//! todas caíam no default (*«sou uma linha de formulário e não sei que nomes vou pintar»*), cuja
//! coluna é a **metade cega** da linha. Medido nesse dia com o sistema de texto REAL:
//!
//! | painel | cortados ANTES | cortados DEPOIS |
//! |---|---|---|
//! | `220` (mínimo do dock) | `11` | `7` |
//! | `245` | `6` | **`1`** |
//! | `273,3` (a largura do dono) | `1` | **`0`** |
//! | `304` (omissão) | `0` | `0` |
//!
//! ⚠️ **Os que sobram a `220` são o TECTO, não uma falha:** ali a coluna bate em
//! `usable − vão − 72`, e o `72` é o piso do campo que o dono declarou em 2026-05-24. *Nessa ponta
//! o nome corta, e é a troca que ele escolheu.*
//!
//! # ⚠️ A régua é a PORTA, nunca uma segunda aritmética
//!
//! A coluna sai de [`ph2d_editor_core::widget::property_label_col_w_for`] com o mesmo `desired` que
//! o painel lhe dá — o rótulo mais largo da secção. ⛔ Comparar o resultado da porta com ele próprio
//! seria vácuo; o que se compara é a LARGURA DO TEXTO com a coluna, que é o que o pintor elide.

use ph2d_panel_painter_layers::seccoes;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// ⏳ **QUANTOS NOMES ELIDEM, POR LARGURA DE PAINEL — e só ENCOLHE.**
///
/// ⚠️ A escada cobre o curso que o dock permite, com a largura do dono como **amostra datada** e
/// nunca como cerca (a lição que o gate irmão do Inspector pagou duas vezes).
const ELIDEM_POR_LARGURA: &[(f32, usize)] = &[
    // O MÍNIMO do dock: aqui a coluna e o tecto do campo colidem.
    (220.0, 7),
    (245.0, 1),
    // Amostra DATADA da largura do dono (lida em 2026-09-14).
    (273.3, 0),
    // A largura de OMISSÃO de um dock.
    (304.0, 0),
    // O MÁXIMO do dock.
    (720.0, 0),
];

/// ⚠️ **As secções que vivem num CARTÃO com recuo próprio** — o *Composite Brush* e o *Line*
/// desenham o fundo delas e recuam [`Spacing::Sm`] de cada lado antes de pintar as linhas. Uma
/// régua que lhes desse a linha inteira mediria uma coluna que elas não têm.
const EM_CARTAO: &[&str] = &["composite", "line"];

/// A largura de uma linha deste painel, por secção.
fn largura_da_linha(painel: f32, seccao: &str) -> f32 {
    let conteudo = painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX;
    if EM_CARTAO.contains(&seccao) {
        conteudo - 2.0 * Spacing::Sm.px()
    } else {
        conteudo
    }
}

/// Quantos nomes de marcar não cabem na coluna deles, a uma dada largura de painel.
///
/// `declarada = false` mede a **metade cega** — o que uma linha recebia antes de a secção existir —,
/// e é o CONTROLO: sem ele, uma porta certa que ninguém chama daria exactamente os mesmos números.
fn elidem_com(ts: &mut TextSystem, painel: f32, declarada: bool) -> Vec<String> {
    let fonte = TypeToken::Sm.px();
    let mut out = Vec::new();
    for (nome, chaves) in seccoes::TODAS {
        let linha = largura_da_linha(painel, nome);
        let rotulos: Vec<&str> = chaves.iter().map(|k| ph2d_i18n::tr(k)).collect();
        let mais_largo = declarada.then(|| {
            rotulos
                .iter()
                .map(|t| ts.prefix_width(t, fonte))
                .fold(0.0_f32, f32::max)
        });
        // ⚠️ **`campos = 1`** — uma linha de marcar tem um controlo só; é o que o painel declara.
        let col = ph2d_editor_core::widget::property_label_col_w_for(
            0.0,
            linha,
            mais_largo,
            Some(ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX),
        );
        for t in &rotulos {
            if ts.prefix_width(t, fonte) > col {
                out.push(format!("[{nome}] {t}"));
            }
        }
    }
    out
}

#[test]
fn cada_nome_de_marcar_cabe_na_coluna_da_seccao() {
    let mut ts = TextSystem::new();
    for (painel, tecto) in ELIDEM_POR_LARGURA {
        let cortados = elidem_com(&mut ts, *painel, true);
        assert!(
            cortados.len() <= *tecto,
            "painel {painel}: {} nomes de marcar elidem e o tecto é {tecto} — subiu:\n  {}",
            cortados.len(),
            cortados.join("\n  ")
        );
        // ⚠️ **A metade de OBSOLESCÊNCIA** (`CLAUDE.md` §5.0): se melhorou, o número desce AQUI.
        assert!(
            cortados.len() == *tecto,
            "painel {painel}: elidem {} e a tabela ainda diz {tecto} — aperte o número",
            cortados.len()
        );
    }
}

/// ⭐⭐⭐ **A declaração e o painel dizem a MESMA coisa — os dois lados.**
///
/// ⛔ Uma chave declarada que o painel não pinta engorda a coluna de graça; um rótulo pintado que
/// não está declarado sai da coluna das irmãs. ⚠️ **Piso de população** porque uma varredura partida
/// devolve zero dos dois lados e lê-se como aprovada (`CLAUDE.md` §5.0).
///
/// ⚠️ **A extracção é por JANELA — as 12 linhas a seguir a cada chamada** —, e não «toda chave numa
/// linha só»: a 1.ª redacção acusou a `wetpaint.resolution`, que é um `tr_with` de READOUT a viver
/// numa linha própria. *Um censo que reconhece a FORMA do literal conta tudo o que tem essa forma.*
///
/// ⚠️ **O lado «pintado e não declarado» só vê o que está ESCRITO junto da chamada** — as duas
/// caixas do cartão *Line* chegam por uma tabela (`checkbox_of`), logo entram pelo outro lado (a
/// chave existe no fonte). *Um censo textual vê texto; a metade que ele não vê é nomeada aqui.*
#[test]
fn a_declaracao_das_seccoes_e_o_painel_dizem_o_mesmo() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut ficheiros = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("src existe").flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                ficheiros.push(std::fs::read_to_string(&p).expect("ficheiro legível"));
            }
        }
    }
    assert!(
        ficheiros.len() >= 40,
        "a varredura leu {} ficheiros — está a ler o sítio errado",
        ficheiros.len()
    );
    let declaradas: Vec<&str> = seccoes::TODAS
        .iter()
        .flat_map(|(_, c)| c.iter().copied())
        .collect();
    assert!(
        declaradas.len() >= 30,
        "só {} chaves declaradas — a lista encolheu de mais para o painel que ela descreve",
        declaradas.len()
    );
    let tudo: String = ficheiros.concat();
    for k in &declaradas {
        assert!(
            tudo.contains(&format!("\"{k}\"")),
            "a chave `{k}` está declarada em `seccoes` e o painel não a escreve em lado nenhum — \
             ela engorda a coluna da secção sem nome nenhum precisar disso"
        );
    }
    // O outro lado: toda chave passada a `paint_checkbox_row` tem de estar declarada.
    let mut pintadas = 0usize;
    for src in &ficheiros {
        let linhas: Vec<&str> = src.lines().collect();
        for (i, l) in linhas.iter().enumerate() {
            if !l.contains("paint_checkbox_row(") || l.contains("fn paint_checkbox_row") {
                continue;
            }
            for l in linhas.iter().take((i + 13).min(linhas.len())).skip(i + 1) {
                let t = l.trim().trim_end_matches(',');
                let Some(k) = t
                    .strip_prefix('"')
                    .and_then(|r| r.strip_suffix('"'))
                    .filter(|k| k.starts_with("panel.painter_layers."))
                else {
                    continue;
                };
                pintadas += 1;
                assert!(
                    declaradas.contains(&k),
                    "o painel pinta uma linha de marcar com `{k}` e a secção dela não está \
                     declarada em `seccoes::TODAS` — o nome dela sairia sozinho, desalinhado das \
                     irmãs do cartão"
                );
                break;
            }
        }
    }
    assert!(
        pintadas >= 25,
        "o censo achou {pintadas} chaves passadas a `paint_checkbox_row` — o parse partiu-se"
    );
}

/// ⭐⭐⭐ **O CONTROLO: a metade cega cortava-os, e é por isso que a declaração não é decoração.**
///
/// ⛔ *Uma porta certa que ninguém chama produz exactamente o app do report* (`§35.3`). Sem esta
/// metade, apagar o `.seccao(…)` dos sítios de pintura deixaria o gate de cima **verde**: ele mede
/// a porta, e a porta continuaria a responder bem a quem lhe perguntasse.
///
/// Medido em 2026-09-16 — a mesma população, com a coluna que a linha recebia ANTES:
///
/// | painel | metade cega | secção declarada |
/// |---|---|---|
/// | `220` | `11` | `7` |
/// | `245` | `6` | **`1`** |
/// | `273,3` | `1` | **`0`** |
#[test]
fn e_a_metade_cega_cortaria_mais() {
    let mut ts = TextSystem::new();
    for (painel, _) in ELIDEM_POR_LARGURA {
        let cega = elidem_com(&mut ts, *painel, false).len();
        let declarada = elidem_com(&mut ts, *painel, true).len();
        assert!(
            declarada <= cega,
            "painel {painel}: a secção declarada corta {declarada} e a metade cega cortava {cega} \
             — declarar a secção PIOROU o painel"
        );
    }
    // ⚠️ E em pelo menos uma largura do curso ela tem de ser **estritamente** melhor, senão a
    //    declaração não está a comprar nada e o gate de cima seria vácuo.
    let ganho = ELIDEM_POR_LARGURA.iter().any(|(painel, _)| {
        elidem_com(&mut ts, *painel, true).len() < elidem_com(&mut ts, *painel, false).len()
    });
    assert!(
        ganho,
        "a secção declarada não cura nenhum nome em largura nenhuma — ou a lista está errada, ou \
         os sítios de pintura deixaram de a passar"
    );
}

/// ⭐⭐⭐ **E O PINTOR PASSA A SECÇÃO — a metade sem a qual as outras duas são vácuas.**
///
/// ⛔⛔ *Uma porta certa que ninguém chama produz exactamente o app do report* (`§35.3`). As duas
/// leis acima medem a PORTA da coluna: elas continuariam verdes com o `.seccao(…)` apagado de todos
/// os sítios de pintura, porque a porta responderia bem a quem lhe perguntasse — e ninguém
/// perguntaria.
///
/// ⚠️ **A régua é textual e é o preço que ela cobra:** ela vê `Checkbox::new(` e procura um
/// `.seccao(` na construção. Um sítio que construa a caixa por outro caminho escapa-lhe — e é por
/// isso que ela traz **piso de população**: se a varredura partir, ela mede zero e reprova em vez de
/// se ler como aprovada.
#[test]
fn toda_caixa_deste_painel_declara_a_seccao_dela() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pilha = vec![raiz];
    let mut sitios = 0usize;
    let mut mudos = Vec::new();
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("src existe").flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
                continue;
            }
            if p.extension().is_some_and(|x| x == "rs") {
                let src = std::fs::read_to_string(&p).expect("ficheiro legível");
                let linhas: Vec<&str> = src.lines().collect();
                for (i, l) in linhas.iter().enumerate() {
                    if !l.contains("Checkbox::new(") {
                        continue;
                    }
                    sitios += 1;
                    let janela = linhas[i..(i + 16).min(linhas.len())].join("\n");
                    if !janela.contains(".seccao(") {
                        let nome = p.file_name().unwrap_or_default().to_string_lossy();
                        mudos.push(format!("{nome}:{}", i + 1));
                    }
                }
            }
        }
    }
    assert!(
        sitios >= 5,
        "a varredura achou {sitios} construções de caixa — o parse partiu-se ou o painel mudou de \
         idioma; conte-as antes de baixar este piso"
    );
    assert!(
        mudos.is_empty(),
        "estas caixas não declaram a secção delas, logo o nome vai para a metade cega da linha:\n  \
         {}\n\nA cura é `.seccao(crate::seccoes::seccao(…))` — ou pintar pela porta \
         `paint_brush_top::paint_checkbox_row`, que a passa sozinha.",
        mudos.join("\n  ")
    );
}
