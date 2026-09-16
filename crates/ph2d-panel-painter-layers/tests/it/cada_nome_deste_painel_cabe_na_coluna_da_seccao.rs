//! ⭐⭐⭐ **O NOME DE UMA LINHA DESTE PAINEL CABE NA COLUNA DA SECÇÃO DELA.**
//!
//! ⛔⛔ Até 2026-09-16 este painel tinha **três** respostas para *«onde começa o controlo?»*, e as
//! três conviviam dentro do mesmo cartão:
//!
//! | linha | o que ela usava | onde o nome saía |
//! |---|---|---|
//! | caixa de marcar | o default (*«não sei que nomes vou pintar»*) | na **metade cega** da faixa |
//! | rótulo + chip · amostra de cor | o literal `LABEL_W = 60,0` | encostado à **esquerda** |
//! | número | um `fn seccao*()` por ficheiro | na coluna da secção ✅ |
//!
//! ⇒ *duas colunas de nome alternando linha sim linha não*, que é o defeito que o §6-quinquies da
//! spec existe para matar — um nível acima, entre famílias de linha em vez de entre linhas.
//!
//! Medido nesse dia com o sistema de texto REAL (`Sm`), sobre os rótulos de linha de propriedade
//! que o painel declara:
//!
//! | painel | metade cega | secção declarada |
//! |---|---|---|
//! | `220` (mínimo do dock) | `11` | `7` |
//! | `245` | `6` | **`1`** |
//! | `273,3` (a largura do dono) | `1` | **`0`** |
//! | `304` (omissão) | `0` | `0` |
//!
//! ⚠️ **E a coluna de `60 px` dos chips tinha o defeito dela própria, que a escada acima não vê:**
//! o rótulo **`Paint Mode`** mede `65,3 px` e saía cortado **em TODA largura de painel**, porque um
//! literal não cresce com o dock. Com a porta, a coluna mais estreita que ele encontra é `84,0`.
//!
//! ⚠️ **O que sobra no mínimo do dock é o TECTO, não uma falha:** ali a coluna bate em
//! `usable − vão − `[`NUMBER_INPUT_MIN_W_PX`](ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX), e
//! esse piso é uma **ordem do dono de 2026-05-24**. *Nessa ponta o nome corta, e é a troca que ele
//! escolheu.*
//!
//! # ⚠️ A régua é a PORTA, nunca uma segunda aritmética
//!
//! A coluna sai de [`ph2d_editor_core::widget::property_label_col_w_for`] com o mesmo `desired` e o
//! mesmo `control_need` que o painel lhe dá. ⛔ Comparar o resultado da porta com ele próprio seria
//! vácuo; o que se compara é a **largura do texto** com a coluna, que é o que o pintor elide.

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

/// Quantos nomes não cabem na coluna deles, a uma dada largura de painel.
///
/// `declarada = false` mede a **coluna cega** — o que uma linha recebia antes de a secção existir —,
/// e é o CONTROLO: sem ele, uma porta certa que ninguém chama daria exactamente os mesmos números.
fn elidem_com(ts: &mut TextSystem, painel: f32, declarada: bool) -> Vec<String> {
    let fonte = TypeToken::Sm.px();
    let mut out = Vec::new();
    for d in seccoes::TODAS {
        let linha = largura_da_linha(painel, d.nome);
        let rotulos: Vec<&str> = d.chaves.iter().map(|k| ph2d_i18n::tr(k)).collect();
        let mais_largo = declarada.then(|| {
            rotulos
                .iter()
                .map(|t| ts.prefix_width(t, fonte))
                .fold(0.0_f32, f32::max)
        });
        // ⚠️ **O `control_need` é o da SECÇÃO** — `campos` caixas ao piso, com os vãos —, que é
        //    exactamente o que a [`ph2d_editor_core::widget::colunas_da_linha`] deriva no produto.
        #[allow(clippy::cast_precision_loss)]
        let n = d.campos as f32;
        let precisa = n * ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX
            + (n - 1.0) * ph2d_tokens::control_gap_px();
        let col = ph2d_editor_core::widget::property_label_col_w_for(
            0.0,
            linha,
            mais_largo,
            Some(precisa),
        );
        for t in &rotulos {
            if ts.prefix_width(t, fonte) > col {
                out.push(format!("[{}] {t}", d.nome));
            }
        }
    }
    out
}

#[test]
fn cada_nome_deste_painel_cabe_na_coluna_da_seccao() {
    let mut ts = TextSystem::new();
    for (painel, tecto) in ELIDEM_POR_LARGURA {
        let cortados = elidem_com(&mut ts, *painel, true);
        assert!(
            cortados.len() <= *tecto,
            "painel {painel}: {} nomes elidem e o tecto é {tecto} — subiu:\n  {}",
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

/// ⭐⭐⭐ **O CONTROLO: a coluna cega cortava-os, e é por isso que a declaração não é decoração.**
///
/// ⛔ *Uma porta certa que ninguém chama produz exactamente o app do report* (`§35.3`). Sem esta
/// metade, apagar o `.seccao(…)` dos sítios de pintura deixaria o gate de cima **verde**: ele mede
/// a porta, e a porta continuaria a responder bem a quem lhe perguntasse.
#[test]
fn e_a_coluna_cega_cortaria_mais() {
    let mut ts = TextSystem::new();
    for (painel, _) in ELIDEM_POR_LARGURA {
        let cega = elidem_com(&mut ts, *painel, false).len();
        let declarada = elidem_com(&mut ts, *painel, true).len();
        assert!(
            declarada <= cega,
            "painel {painel}: a secção declarada corta {declarada} e a cega cortava {cega} — \
             declarar a secção PIOROU o painel"
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

/// Lê todo o `src/` do painel, ficheiro a ficheiro.
fn fontes() -> Vec<String> {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("src existe").flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(std::fs::read_to_string(&p).expect("ficheiro legível"));
            }
        }
    }
    assert!(
        out.len() >= 40,
        "a varredura leu {} ficheiros — está a ler o sítio errado",
        out.len()
    );
    out
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
#[test]
fn a_declaracao_das_seccoes_e_o_painel_dizem_o_mesmo() {
    let ficheiros = fontes();
    let declaradas: Vec<&str> = seccoes::TODAS
        .iter()
        .flat_map(|d| d.chaves.iter().copied())
        .collect();
    assert!(
        declaradas.len() >= 55,
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
    // O outro lado: toda chave passada a uma PORTA de linha tem de estar declarada.
    let mut pintadas = 0usize;
    for src in &ficheiros {
        let linhas: Vec<&str> = src.lines().collect();
        for (i, l) in linhas.iter().enumerate() {
            let porta = l.contains("paint_checkbox_row(") || l.contains("paint_dropdown_row(");
            if !porta || l.contains("fn paint_") {
                continue;
            }
            for l2 in linhas.iter().take((i + 13).min(linhas.len())).skip(i + 1) {
                let t = l2.trim().trim_end_matches(',');
                let Some(k) = t
                    .trim_start_matches('"')
                    .split('"')
                    .next()
                    .filter(|k| t.starts_with('"') && k.starts_with("panel.painter_layers."))
                else {
                    continue;
                };
                pintadas += 1;
                assert!(
                    declaradas.contains(&k),
                    "o painel pinta uma linha com `{k}` e a secção dela não está declarada em \
                     `seccoes::TODAS` — o nome dela sairia sozinho, desalinhado das irmãs do cartão"
                );
                break;
            }
        }
    }
    assert!(
        pintadas >= 40,
        "o censo achou {pintadas} chaves passadas às portas de linha — o parse partiu-se"
    );
}

/// ⭐⭐⭐ **E O PINTOR PASSA A SECÇÃO — a metade sem a qual as outras são vácuas.**
///
/// ⛔⛔ *Uma porta certa que ninguém chama produz exactamente o app do report* (`§35.3`).
///
/// ⚠️ **A régua é textual e é o preço que ela cobra:** ela vê `Checkbox::new(` e procura um
/// `.seccao(` na construção. Um sítio que construa a caixa por outro caminho escapa-lhe — e é por
/// isso que ela traz **piso de população**.
#[test]
fn toda_caixa_deste_painel_declara_a_seccao_dela() {
    let mut sitios = 0usize;
    let mut mudos = Vec::new();
    for (n, src) in fontes().iter().enumerate() {
        let linhas: Vec<&str> = src.lines().collect();
        for (i, l) in linhas.iter().enumerate() {
            if !l.contains("Checkbox::new(") {
                continue;
            }
            sitios += 1;
            let janela = linhas[i..(i + 16).min(linhas.len())].join("\n");
            if !janela.contains(".seccao(") {
                mudos.push(format!("ficheiro #{n}, linha {}", i + 1));
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
        "estas caixas não declaram a secção delas, logo o nome vai para a coluna cega:\n  {}\n\n\
         A cura é `.seccao(crate::seccoes::seccao_da_chave(…))` — ou pintar pela porta \
         `paint_brush_top::paint_checkbox_row`, que a passa sozinha.",
        mudos.join("\n  ")
    );
}

/// ⭐⭐⭐ **E NENHUMA COLUNA DE RÓTULO É ESCOLHIDA NO SÍTIO DE PINTURA.**
///
/// ⛔⛔ O censo da casa (`the_label_column_is_one_answer`) procura `label_col` — e este painel
/// escrevia `LABEL_W`, `ADJ_LABEL_W`, `BLEND_LABEL_W`, `CARD_LABEL_W`. *A quinta grafia da mesma
/// pergunta*, e a régua passou por cima delas durante toda a conversão do resto do app.
///
/// ⏳ **Catraca: esta lista só ENCOLHE.** Cada entrada é uma coluna que ainda não passou pela porta,
/// com o que falta para ela passar.
/// ⭐ **O `paint_line.rs` SAIU desta lista em 2026-09-16** — a coluna dele passou pela porta, e as
/// duas que ele cortava (`Line Width` `66,4` e `Roughness` `68,8` numa coluna de `62`) deixaram de
/// cortar; no mesmo dia as barras dele viraram caixa única (spec §2), com os 12 chips da tabela
/// `line_barras`.
/// ⭐ **E o `paint_adjust.rs` saiu logo a seguir** — o `ADJ_LABEL_W = 44` cortava **17 de 44** nomes
/// em toda largura; a pilha passou pela porta e tem gate próprio, que pinta o painel
/// (`cada_nome_da_pilha_de_ajustes_cabe_na_coluna`).
const COLUNAS_A_MAO: &[(&str, &str)] = &[
    (
        "paint_composite.rs",
        "o `LABEL_W` é a coluna «N Ferramenta» de cada CAMADA do pincel composto — uma linha de \
         LISTA (índice, nome, força, reordenar), que a spec §1 governa por lei própria",
    ),
    (
        "paint_shape_layers.rs",
        "o `BLEND_LABEL_W` é o rótulo de uma fileira de CLUSTER (chip + caixa + opacidade), não uma \
         linha de propriedade — decidir se ela é uma é trabalho de produto",
    ),
    (
        "card.rs",
        "o `CARD_LABEL_W` governa as rows de um CARTÃO de técnica, que têm layout próprio",
    ),
];

#[test]
fn nenhuma_coluna_de_rotulo_e_escolhida_no_sitio_de_pintura() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pilha = vec![raiz];
    let mut acusados = Vec::new();
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("src existe").flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
                continue;
            }
            if p.extension().is_some_and(|x| x == "rs") {
                let nome = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                for (n, l) in std::fs::read_to_string(&p)
                    .expect("legível")
                    .lines()
                    .enumerate()
                {
                    let t = l.trim();
                    if t.starts_with("//") {
                        continue;
                    }
                    let baixo = t.to_ascii_lowercase();
                    if !(baixo.contains("label_col") || baixo.contains("label_w")) {
                        continue;
                    }
                    let Some((_, dir)) = t.split_once('=') else {
                        continue;
                    };
                    if dir.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
                        acusados.push((nome.clone(), n + 1, t.to_string()));
                    }
                }
            }
        }
    }
    let novos: Vec<String> = acusados
        .iter()
        .filter(|(f, _, _)| !COLUNAS_A_MAO.iter().any(|(t, _)| t == f))
        .map(|(f, n, t)| format!("{f}:{n}: {t}"))
        .collect();
    assert!(
        novos.is_empty(),
        "estas colunas de rótulo são escolhidas no sítio de pintura, o que a spec §3 proíbe (a \
         coluna docada é ARRASTÁVEL):\n  {}\n\nA cura é `paint_brush_rows::linha_da_chave(…)`.",
        novos.join("\n  ")
    );
    // ⚠️ **A metade de OBSOLESCÊNCIA** (`CLAUDE.md` §5.0): uma tolerância que já não descreve nada
    //    é uma licença aberta para o próximo literal que caia naquele ficheiro.
    for (f, porque) in COLUNAS_A_MAO {
        assert!(
            porque.len() > 40,
            "a tolerância de `{f}` não diz o mecanismo — uma lista sem mecanismo é uma licença"
        );
        assert!(
            acusados.iter().any(|(n, _, _)| n == f),
            "`{f}` está na lista de colunas à mão e já não escreve nenhuma — apague a linha"
        );
    }
}

/// ⭐⭐ **E O CONTROLO NUNCA FICA ABAIXO DO PISO QUE O DONO DECLAROU.**
///
/// ⛔ A conversão move o controlo para o MEIO da linha, logo ele ENCOLHE — e um chip de dropdown
/// espremido é a troca que esta lei existe para impedir. O piso é
/// [`ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX`] (ordem do dono, 2026-05-24).
#[test]
fn e_o_controlo_nunca_fica_abaixo_do_piso_do_dono() {
    let mut ts = TextSystem::new();
    let piso = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;
    let mut pior = f32::INFINITY;
    for d in seccoes::TODAS {
        for (painel, _) in ELIDEM_POR_LARGURA {
            let linha = largura_da_linha(*painel, d.nome);
            let sec = {
                let rotulos: Vec<&str> = d.chaves.iter().map(|k| ph2d_i18n::tr(k)).collect();
                ph2d_editor_core::widget::Seccao::medida(&mut ts, d.campos, &rotulos)
            };
            let row = ph2d_editor_core::widget::colunas_da_linha(0.0, linha, 0.0, 22.0, sec);
            pior = pior.min(row.control.w);
            assert!(
                row.control.w >= piso - 0.5,
                "[{}] a {painel}: o controlo fica com {:.1} e o piso do dono é {piso}",
                d.nome,
                row.control.w
            );
        }
    }
    assert!(
        pior.is_finite(),
        "nenhuma secção foi medida — a lista está vazia"
    );
}
