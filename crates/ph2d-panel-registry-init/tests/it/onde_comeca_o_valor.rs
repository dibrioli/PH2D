//! ⭐⭐ **ONDE COMEÇA O VALOR — em quantas colunas os controlos de cada painel arrancam.**
//!
//! Ordem do dono (2026-09-21): *«quanto ao alinhamento precisamos melhorar em todos os lugares»*.
//!
//! A régua pergunta ao PRODUTO, como a irmã [`super::nenhum_nome_por_cima_do_controlo`]: pinta-se
//! cada painel pela porta do registo, de fábrica e armado, à largura do dono (`~297 px` medidos
//! em 2026-09-19), e colhe-se o que o índice de acerto registou. Numa linha que tem algo à
//! esquerda (o nome), o controlo começa numa COLUNA; um painel alinhado tem poucas colunas, e cada
//! uma com muitas linhas. *Uma coluna com uma linha só é, quase sempre, uma linha que escolheu o
//! próprio sítio.*

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_ui_testkit::MockPanelHost;

use super::a_marca_tem_a_altura_da_linha::abre_as_gavetas;

/// A largura do dono: a coluna docada que ele fotografou em 2026-09-19.
const LARGURA_DO_DONO: f32 = 300.0;

fn vista() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: LARGURA_DO_DONO,
        h: 4000.0,
    }
}

/// Um controlo que arranca DEPOIS da borda: `(x, y, largura, slug)`, e o GRUPO em que ele cai.
///
/// ⚠️ **O grupo é o troço entre duas peças de LARGURA INTEIRA** (um cabeçalho de secção, um botão de
/// acção): a coluna do valor é uma resposta da SECÇÃO (`Seccao`, §6-ter da spec), logo duas secções
/// com colunas diferentes estão certas e duas linhas do MESMO troço com colunas diferentes não.
#[derive(Debug, Clone)]
pub(crate) struct Arranque {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub grupo: usize,
    pub slug: String,
}

fn colhe(
    host: &mut MockPanelHost,
    painel: &mut ph2d_editor_core::panel::ErasedPanel,
    nomes: &std::collections::BTreeMap<ph2d_editor_core::NodeId, String>,
) -> (f32, Vec<Arranque>) {
    let _ = host.medindo_a_pintura_do_registo(painel, vista());
    let pintados = host.registos_da_ultima_pintura();
    let borda = pintados
        .iter()
        .map(|(_, r)| r.x)
        .fold(f32::INFINITY, f32::min);
    // Uma linha = os rectângulos cujo centro vertical cai na mesma faixa de 4 px.
    let mut linhas: std::collections::BTreeMap<i64, Vec<(ph2d_editor_core::NodeId, Rect)>> =
        std::collections::BTreeMap::new();
    for (n, r) in &pintados {
        if r.w <= 0.0 || r.h <= 0.0 || r.h > 60.0 {
            continue;
        }
        let k = ((r.y + r.h * 0.5) / 4.0).round() as i64;
        linhas.entry(k).or_default().push((*n, *r));
    }
    let direita = pintados
        .iter()
        .map(|(_, r)| r.x + r.w)
        .fold(f32::NEG_INFINITY, f32::max);
    let largura = (direita - borda).max(1.0);
    let mut out = Vec::new();
    let mut grupo = 0usize;
    for (_, rs) in linhas {
        let (n, r) = rs
            .iter()
            .min_by(|a, b| a.1.x.total_cmp(&b.1.x))
            .copied()
            .expect("linha nao vazia");
        // Uma peça que arranca JUNTO da borda e ocupa quase a linha inteira fecha o troço.
        // ⚠️ «junto» e não «na»: um cartão recua o conteúdo dele (o Painter), e com `< 1 px` os
        // cabeçalhos dos cartões não fechavam troço nenhum — o painel inteiro lia-se como UM troço,
        // e a prova de mutação que dava às fileiras do padrão a coluna de `148` SOBREVIVEU porque
        // `148` já era a coluna de outra secção desse troço gigante.
        if r.x - borda < 24.0 && r.w >= 0.8 * (largura - (r.x - borda)) {
            grupo += 1;
            continue;
        }
        // Só conta uma linha com ALGO à esquerda do controlo (o nome) — a que arranca na borda é
        // um botão, uma lista, uma escolha em paleta: outra forma, outra régua.
        if r.x - borda < 24.0 {
            continue;
        }
        // …e a que só tem um botão ENCOSTADO à direita (escolher, fechar, remover) também: o valor
        // dela não é um controlo registado, e o botão do fim não é onde a coluna começa.
        if r.x - borda > 0.66 * largura {
            continue;
        }
        // …e a que arranca numa ALÇA (um ponto de curva de `12 px`) também: uma alça é arrastada
        // dentro de um gráfico, não é onde a coluna de um valor começa.
        if r.w < 16.0 {
            continue;
        }
        out.push(Arranque {
            x: r.x - borda,
            y: r.y,
            w: r.w,
            grupo,
            slug: nomes
                .get(&n)
                .cloned()
                .unwrap_or_else(|| format!("#{:x}", n.0)),
        });
    }
    (borda, out)
}

/// Por painel (de fábrica e armado): os arranques colhidos.
pub(crate) fn censo() -> Vec<(String, Vec<Arranque>)> {
    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES, 400);
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut out = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            abre_as_gavetas(host.store_mut());
            let (_, a) = colhe(&mut host, painel, &nomes);
            out.push((id.to_string(), a));
            if let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            {
                let mut host = MockPanelHost::new();
                (arm.arma)(host.store_mut());
                painel.populate(host.store_mut());
                abre_as_gavetas(host.store_mut());
                let (_, a) = colhe(&mut host, painel, &nomes);
                out.push((format!("{id} (armado)"), a));
                (arm.desarma)();
            }
        }
    });
    out
}

const FONTES: &[&str] = &[
    "../ph2d-panel-inspector/src",
    "../ph2d-panel-grid-snap/src",
    "../ph2d-panel-physics/src",
    "../ph2d-panel-sculpt3d/src",
    "../ph2d-panel-model3d/src",
    "../ph2d-panel-vector/src",
    "../ph2d-panel-flip/src",
    "../ph2d-panel-painter-layers/src",
    "../ph2d-panel-equalize-sizes/src",
    "../ph2d-panel-timeline/src",
    "../ph2d-panel-tokens/src",
    "../ph2d-panel-skeleton/src",
    "../ph2d-panel-tags/src",
    "../ph2d-panel-hierarchy/src",
    "../ph2d-editor-core/src/ids",
    "../ph2d-editor-core/src/grid_snap",
    "../ph2d-tool-painter/src",
    "../ph2d-tool-vector/src",
];

/// Colunas distintas (tolerância de 1 px) com a contagem e um exemplo.
pub(crate) fn colunas(a: &[Arranque]) -> Vec<Coluna> {
    let mut xs: Vec<Coluna> = Vec::new();
    for r in a {
        match xs.iter_mut().find(|c| (c.0 - r.x).abs() <= 1.0) {
            Some(c) => c.1 += 1,
            None => xs.push((r.x, 1, r.slug.clone())),
        }
    }
    xs.sort_by(|a, b| a.0.total_cmp(&b.0));
    xs
}

#[test]
#[ignore = "sonda de diagnóstico — corre à mão com --ignored --nocapture"]
fn diag_onde_comeca_o_valor() {
    for (painel, a) in censo() {
        let c = colunas(&a);
        if c.is_empty() {
            continue;
        }
        println!("\n{painel}: {} linha(s) em {} coluna(s)", a.len(), c.len());
        for (x, n, slug) in c {
            println!("  x={x:7.1}  {n:3}×  ex. {slug}");
        }
    }
}

#[test]
#[ignore = "sonda de diagnóstico — PH2D_PAINEL=<id> para ver linha a linha"]
fn diag_linha_a_linha() {
    let alvo = std::env::var("PH2D_PAINEL").unwrap_or_default();
    for (painel, a) in censo() {
        if !painel.starts_with(alvo.as_str()) {
            continue;
        }
        println!("\n{painel}");
        for r in &a {
            println!(
                "  g={:3}  y={:7.1}  x={:7.1}  w={:6.1}  {}",
                r.grupo, r.y, r.x, r.w, r.slug
            );
        }
    }
}

/// Uma coluna de valor: `(x, quantas linhas, um exemplo)`.
pub(crate) type Coluna = (f32, usize, String);

/// ⭐ **Os troços com MAIS DE UMA coluna** — `(painel, grupo, colunas)`.
pub(crate) fn tropecos() -> Vec<(String, usize, Vec<Coluna>)> {
    let mut out = Vec::new();
    for (painel, a) in censo() {
        let mut grupos: std::collections::BTreeMap<usize, Vec<Arranque>> =
            std::collections::BTreeMap::new();
        for r in a {
            grupos.entry(r.grupo).or_default().push(r);
        }
        for (g, rs) in grupos {
            let c = colunas(&rs);
            if c.len() > 1 {
                out.push((painel.clone(), g, c));
            }
        }
    }
    out
}

#[test]
#[ignore = "sonda de diagnóstico — os troços com mais de uma coluna"]
fn diag_tropecos() {
    for (painel, g, c) in tropecos() {
        println!("\n{painel} · grupo {g}");
        for (x, n, slug) in c {
            println!("  x={x:7.1}  {n:3}×  ex. {slug}");
        }
    }
}

/// ⛔ **As colunas A MAIS dentro dos troços que NÃO são defeito de alinhamento** — por painel, a
/// soma de `(colunas − 1)` sobre os troços dele, e cada linha diz porquê. A lista só ENCOLHE, e é
/// uma IGUALDADE: uma entrada que deixou de descrever o que mede reprova.
///
/// ⚠️ **A unidade é a COLUNA e não o troço** — a 1.ª redacção contava troços, e a prova de mutação
/// SOBREVIVEU: devolver os chips da timeline à coluna escrita à mão punha uma TERCEIRA coluna no
/// troço que já estava declarado por causa das abas, e o total de troços continuava `1`.
const COLUNAS_A_MAIS_DECLARADAS: &[(&str, usize, &str)] = &[(
    "timeline",
    1,
    "as abas (`tab_keys`) são um segmentado na faixa do título, não um par nome/valor",
)];

/// ⭐⭐⭐ **Os painéis que arrancam o valor em MAIS de uma coluna, e porquê** — todo painel que não
/// está aqui arranca numa coluna SÓ, de ponta a ponta (ordem do dono, 2026-09-23: *«quero tudo
/// alinhado e padronizado»*, com a troca dita: nomes de secção de uma componente perdem espaço e o
/// que não couber sai cortado com balão). A lei é a [`ph2d_editor_core::widget::ColunaDoPainel`].
///
/// ⚠️ **Ela era uma lista de quem JÁ estava numa coluna** (`UMA_COLUNA`, que só crescia) e virou a
/// lista das EXCEPÇÕES: um painel novo nasce obrigado à coluna única, sem ninguém se lembrar de o
/// inscrever. A lista é uma IGUALDADE — uma excepção que deixou de descrever o que mede reprova.
///
/// ⚠️ **E a espécie que motivou a lista antiga continua coberta:** os marcadores do vetor vivem num
/// troço deles, e a prova de mutação que lhes devolvia o vão escrito à mão SOBREVIVIA à régua de
/// troço — ela reprova aqui, porque a coluna é do PAINEL.
const COLUNAS_DECLARADAS_POR_PAINEL: &[(&str, usize, &str)] = &[
    (
        "timeline",
        2,
        "as abas (`tab_keys`) são um segmentado na faixa do título, não um par nome/valor",
    ),
    (
        "widget_gallery",
        3,
        "a galeria é um CATÁLOGO: cada amostra mostra o widget na largura dele, e alinhá-las \
         apagaria o que ela existe para mostrar",
    ),
];

/// O piso de painéis — o do âmbito de WORKSPACE (a régua das elisões mede o mesmo registo).
const PISO_DE_PAINEIS: usize = 27;
/// O piso de linhas colhidas — medido `455` em 2026-09-23 no âmbito de workspace (ver o `eprintln!`).
const PISO_DE_LINHAS: usize = 400;

/// ⭐⭐⭐ **Dentro de um troço, o valor arranca numa coluna SÓ.**
///
/// Ordem do dono (2026-09-21): *«quanto ao alinhamento precisamos melhorar em todos os lugares»*.
/// A varredura achou CINCO degraus — os marcadores do vetor (`4 px`, um vão escrito à mão), o ALVO
/// da câmera (`25 px`) e o SOM do áudio (`6 px`, os dois com uma segunda declaração de coluna para
/// uma linha), os chips da barra da timeline (`17,5 px`, uma coluna de nome escrita à mão ao lado de
/// uma medida) e as fileiras de PADRÃO do Painter (`25 px`, uma coluna medida sobre os nomes delas
/// dentro de um cartão que já tinha a sua) — este último só visível no âmbito de workspace.
///
/// ⚠️ **Duas secções com colunas diferentes estão CERTAS** — a coluna é uma resposta da secção — e
/// a régua só as separa porque o troço acaba numa peça de largura inteira. ⚠️ E o piso de
/// população vem primeiro: uma pintura que não registasse nada daria zero degraus e ficaria verde.
#[test]
fn dentro_de_um_troco_o_valor_arranca_numa_coluna_so() {
    let censo = censo();
    let paineis = censo
        .iter()
        .filter(|(p, _)| !p.ends_with("(armado)"))
        .count();
    let linhas: usize = censo.iter().map(|(_, a)| a.len()).sum();
    eprintln!("[onde_comeca_o_valor] {paineis} painéis · {linhas} linhas com nome à esquerda");
    // ⛔⛔ **O ÂMBITO primeiro** — a lição que a régua das elisões pagou em 2026-09-20: `flip`,
    //    `flip_frames`, `painter_layers` e `wet_tuning` NÃO estão no `default` desta crate, logo uma
    //    corrida `-p` não os regista e as colunas deles não entram na conta. Esta régua foi
    //    calibrada no âmbito em que o app CORRE.
    assert!(
        paineis >= PISO_DE_PAINEIS && linhas >= PISO_DE_LINHAS,
        "a varredura viu {paineis} painéis e {linhas} linhas (piso {PISO_DE_PAINEIS} / \
         {PISO_DE_LINHAS}). ⚠️ SE VOCÊ CORREU ISTO COM `-p ph2d-panel-registry-init`, a causa é essa \
         e não o código — corra `cargo nextest run --workspace -E 'test(onde_comeca_o_valor)'`. \
         Senão, a pintura pelo registo deixou de registar, e zero degraus não provaria nada."
    );
    let mut a_mais: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut detalhe = Vec::new();
    for (painel, g, c) in tropecos() {
        *a_mais.entry(painel.clone()).or_default() += c.len() - 1;
        detalhe.push(format!(
            "{painel} · troço {g}: {}",
            c.iter()
                .map(|(x, n, s)| format!("x={x:.1} ({n}×, ex. {s})"))
                .collect::<Vec<_>>()
                .join(" · ")
        ));
    }
    let nomes: std::collections::BTreeSet<&str> = a_mais
        .keys()
        .map(String::as_str)
        .chain(COLUNAS_A_MAIS_DECLARADAS.iter().map(|(p, _, _)| *p))
        .collect();
    for painel in nomes {
        let medido = a_mais.get(painel).copied().unwrap_or(0);
        let declarado = COLUNAS_A_MAIS_DECLARADAS
            .iter()
            .find(|(p, _, _)| *p == painel);
        match declarado {
            None => assert_eq!(
                medido,
                0,
                "o painel `{painel}` tem {medido} coluna(s) de valor A MAIS dentro de um troço. \
                 Cure pela PORTA — `ph2d_editor_core::panel::value_col` ou UMA `Seccao` para o \
                 corpo inteiro —, nunca somando uma entrada:\n{}",
                detalhe.join("\n")
            ),
            Some((_, k, porque)) => assert_eq!(
                medido,
                *k,
                "`COLUNAS_A_MAIS_DECLARADAS` diz {k} em `{painel}` ({porque}) e a varredura mede \
                 {medido}. A MAIS é um degrau novo (cure-o pela porta); a MENOS é uma entrada que \
                 deixou de descrever o que mede (acerte-a):\n{}",
                detalhe.join("\n")
            ),
        }
    }
    let mut vistos = std::collections::BTreeSet::new();
    for (painel, a) in &censo {
        let c = colunas(a);
        let base = painel.trim_end_matches(" (armado)");
        match COLUNAS_DECLARADAS_POR_PAINEL
            .iter()
            .find(|(p, _, _)| *p == base)
        {
            None => assert!(
                c.len() <= 1,
                "`{painel}` arranca o valor em {} colunas — todo painel arranca numa SÓ \
                 (`ColunaDoPainel`). Cure pela PORTA: a linha tem de pedir a coluna \
                 (`property_row_columns_for` / `value_col`), nunca somar um vão à mão: {c:?}",
                c.len()
            ),
            Some((_, k, porque)) => {
                vistos.insert(base);
                assert_eq!(
                    c.len(),
                    *k,
                    "`COLUNAS_DECLARADAS_POR_PAINEL` diz {k} em `{painel}` ({porque}) e a \
                     varredura mede {}. A MAIS é um degrau novo; a MENOS é uma excepção que deixou \
                     de descrever o que mede: {c:?}",
                    c.len()
                );
            }
        }
    }
    for (p, _, _) in COLUNAS_DECLARADAS_POR_PAINEL {
        assert!(
            vistos.contains(p),
            "`{p}` está em `COLUNAS_DECLARADAS_POR_PAINEL` e a varredura não o viu — a excepção \
             já não descreve nada e tem de sair"
        );
    }
    // ⚠️ O piso da coluna ÚNICA — sem ele uma lei que devolvesse zero linhas a todos passava.
    let unicos = censo.iter().filter(|(_, a)| colunas(a).len() == 1).count();
    assert!(
        unicos >= COLUNA_UNICA_PISO,
        "só {unicos} painéis arrancam numa coluna (piso {COLUNA_UNICA_PISO}, medido 2026-09-23)"
    );
}

/// Quantos painéis (armados contados à parte) arrancam numa coluna só — medido `11` em 2026-09-23.
const COLUNA_UNICA_PISO: usize = 11;
