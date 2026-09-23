//! ⭐⭐⭐ **UMA LINHA DE MARCAR TEM A ALTURA DE UMA LINHA DE PROPRIEDADE — em todo o app.**
//!
//! Report do dono, 2026-09-21, com foto da secção `PROJECTILE MOTION`: *«Apenas o checkbox tem sua
//! moldura e ele próprio menores que o padrão. isso acontece em vários painéis do APP.»*
//!
//! # ⛔⛔ A causa, medida
//!
//! São **duas grandezas com nomes parecidos**, e cinco secções copiaram uma para o lugar da outra:
//!
//! | grandeza | porta | valor | o que é |
//! |---|---|---:|---|
//! | altura de uma LINHA | [`ph2d_tokens::ROW_H_PX`] | `22` | a caixa do controlo, em toda fileira |
//! | aresta da MARCA | [`ph2d_tokens::CHECKBOX_BOX_PX`] | `18` | o quadrado que leva o visto |
//!
//! A marca vive **dentro** da caixa com um degrau de recuo de cada lado
//! (`lado = min(18, h − 2·Xs)`), logo escrever `18` onde se pedia a altura da linha encolhe as
//! **duas** coisas de que o report fala: a moldura de `22` para `18`, e a marca de `14` para `10`.
//!
//! ⚠️⚠️ **E a cura já tinha sido escrita — para UM dos treze sítios.** O
//! `sections/anchor_mount_row.rs` diz-o no próprio comentário desde 2026-09-15 (*«era `18.0`, o
//! MESMO literal em TREZE sítios»*), e os outros cinco ficaram com um comentário que afirma
//! `igual à das irmãs` — *uma frase que era verdade no dia em que foi escrita e que a cura da irmã
//! tornou falsa, sem nada deixar de compilar*. **Não havia censo**, e é por isso que só o olho do
//! dono a podia encontrar.
//!
//! # ⭐ O que esta régua mede
//!
//! O **rect que o painel REGISTA** para cada id cujo estado no store é uma caixa de marcar — ou
//! seja, a altura da linha pela porta do produto, e não um literal lido no fonte. É essa mesma
//! altura que o [`ph2d_editor_core::widget::checkbox`] usa como altura da caixa do controlo.
//!
//! # ⚠️⚠️ O QUE ESTA RÉGUA NÃO ALCANÇA — e como o buraco foi fechado À MÃO
//!
//! A varredura pinta cada painel **de fábrica** e, quando existe armação, com um documento na
//! mão — e mesmo assim só **três** painéis pintam uma marca booleana (`inspector`, `timeline`,
//! `grid_snap`). Um painel que só mostre caixas com um documento que a
//! [`super::paineis_armados::TABELA`] não sabe montar fica **invisível a este gate**.
//!
//! ⭐ O buraco foi fechado por ENUMERAÇÃO, em 2026-09-21: só quem chama
//! [`ph2d_editor_core::widget::paint_checkbox`] **fora da porta** pode escolher a altura, e são
//! `17` ficheiros — `ph2d-panel-inspector` (5) · `ph2d-panel-painter-layers` (4) ·
//! `ph2d-panel-vector` (1) · `ph2d-panel-wet-tuning` (1) · a própria porta e os gates do
//! `ph2d-editor-core`. **Todos os de fora do Inspector passam [`ph2d_tokens::ROW_H_PX`]**; a
//! doença era do Inspector e só dele. ⛔ *Isto é uma medição com data, não uma propriedade — quem
//! escrever o 18.º chamador não é avisado por nada.*
//!
//! ⏳ **ABERTO, com o mecanismo:** o `sections/script.rs` passa ao pintor **só a coluna do
//! controlo** em vez da linha, logo o [`colunas_da_linha`] volta a partir esse rectângulo em duas
//! e a caixa nasce a meio dele. A altura é `22`, logo este gate não a vê — e a
//! `TABELA` não sabe montar um `.luau`, logo a varredura também não.

//! ⛔ **O painel `authored` fica FORA, e a exclusão é verificada:** ali a moldura é o que o artista
//! desenhou, não uma linha de formulário — e uma exclusão por nome que não confirme que o painel
//! existe é a porta por onde um painel inteiro deixa de ser medido em silêncio.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_ui_testkit::MockPanelHost;

/// ⭐ As árvores onde vivem os ids dos painéis que este censo mede — só para a sonda os NOMEAR.
pub(crate) const FONTES_DOS_IDS: &[&str] = &[
    "../ph2d-panel-inspector/src/ids",
    "../ph2d-panel-grid-snap/src",
    "../ph2d-panel-timeline/src",
    "../ph2d-editor-core/src/ids",
    // ⚠️ O `GS_SNAP_ENABLED` NÃO vive em `src/ids` — sem esta árvore a excepção declarada abaixo
    //    lia-se `(sem nome)` e a régua não a podia prender a um id.
    "../ph2d-editor-core/src/grid_snap",
];
/// ⛔ Piso do extractor. ⚠️ **MEDIDO**, nunca escolhido — ver a corrida da sonda.
pub(crate) const PISO_DOS_SLUGS: usize = 400;

/// ⛔ A pele de canvas: ali a caixa é do tamanho que o **artista** desenhou.
const FORA_POR_DESENHO: &[&str] = &["authored"];

/// Uma marca medida: onde ela está e que altura a linha dela tem.
#[derive(Debug, Clone)]
pub(crate) struct Marca {
    pub painel: &'static str,
    pub armado: bool,
    pub altura: f32,
    pub especie: &'static str,
    pub id: ph2d_editor_core::NodeId,
}

/// ⚠️ **A altura não depende da largura da janela** — a escada da varredura de elisões muda `x` e
/// `w` das colunas e nunca o `h` de uma fileira. Uma janela chega, e varrer três seria medir a
/// mesma coisa três vezes.
pub(crate) fn viewport() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: 1366.0,
        h: 1024.0,
    }
}

pub(crate) fn censo() -> Vec<Marca> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut tudo = Vec::new();
    let mut visitados = 0usize;
    let mut vistos: Vec<&'static str> = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        visitados = reg.panels().len();
        vistos = reg.panels().iter().map(|p| p.manifest.id).collect();
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            if FORA_POR_DESENHO.contains(&id) {
                continue;
            }
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            abre_as_gavetas(host.store_mut());
            colhe(&mut host, painel, id, false, &mut tudo);

            // ⭐ A segunda passagem, com um DOCUMENTO na mão — a mesma tabela que a varredura de
            //   elisões usa, porque cinco painéis pintam ZERO fileiras de fábrica.
            if let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            {
                let mut host = MockPanelHost::new();
                (arm.arma)(host.store_mut());
                painel.populate(host.store_mut());
                abre_as_gavetas(host.store_mut());
                colhe(&mut host, painel, id, true, &mut tudo);
                // ⛔ O estado que uma fixtura deixa é o estado que a régua seguinte mede.
                (arm.desarma)();
            }
        }
    });
    // ⛔⛔ **A exclusão é VERIFICADA.** Uma lista de nomes que ninguém confirma é a porta por onde
    //    um painel inteiro deixa de ser medido em silêncio — basta alguém lhe mudar o `id`.
    for fora in FORA_POR_DESENHO {
        assert!(
            vistos.contains(fora),
            "{fora:?} está na lista de excluídos e NÃO existe no registo — a exclusão deixou de \
             descrever um painel, e a régua passou a saltar nada. Painéis vistos: {vistos:?}"
        );
    }
    assert!(
        visitados >= PISO_DE_PAINEIS && tudo.len() >= PISO_DE_MARCAS,
        "o censo viu {} marcas em {visitados} painéis (piso {PISO_DE_MARCAS} / \
         {PISO_DE_PAINEIS}) — uma varredura que lê pouco devolve ZERO acusações e lê-se como \
         aprovação.\n⚠️ SE CORREU COM `-p ph2d-panel-registry-init`: `flip`, `flip_frames`, \
         `painter_layers` e `wet_tuning` NÃO estão no `default` desta crate. Corra \
         `cargo nextest run --workspace -E 'test(a_marca_tem_a_altura_da_linha)'`.",
        tudo.len()
    );
    tudo
}

/// ⛔ Pisos de população. ⚠️ Eles são do âmbito em que o app CORRE (a workspace), nunca o do `-p`.
/// ⭐ **MEDIDO**: o registo da workspace tem `27` painéis (2026-09-21).
const PISO_DE_PAINEIS: usize = 27;
/// ⭐ **MEDIDO** na 1.ª corrida (2026-09-21): `40` marcas em `27` painéis varridos. ⛔ Ele não é
/// escolhido — a 1.ª redacção deste ficheiro palpitou `60` e reprovou sobre um censo CORRECTO.
const PISO_DE_MARCAS: usize = 40;

fn colhe(
    host: &mut MockPanelHost,
    painel: &mut ph2d_editor_core::panel::ErasedPanel,
    id: &'static str,
    armado: bool,
    saida: &mut Vec<Marca>,
) {
    let _ = host.medindo_a_pintura_do_registo(painel, viewport());
    let registos = host.registos_da_ultima_pintura();
    for (nid, rect) in registos {
        // ⚠️ **As DUAS espécies de marca booleana**: o mesmo pintor serve `Checkbox` e `Toggle`
        //    ([`paint_boolean_mark`]), e medir só uma deixaria metade da população de fora.
        let especie = match host.store().get(nid) {
            Some(InteractiveState::Checkbox { .. }) => Some("checkbox"),
            Some(InteractiveState::Toggle { .. }) => Some("toggle"),
            _ => None,
        };
        if let Some(especie) = especie {
            saida.push(Marca {
                painel: id,
                armado,
                altura: rect.h,
                especie,
                id: nid,
            });
        }
    }
}

pub(crate) fn abre_as_gavetas(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas(store);
    for id in store.collapsible_ids() {
        store.set_collapsed(id, false);
    }
}

/// ⭐ A sonda que imprime o censo. `--run-ignored all` para a correr.
#[test]
#[ignore = "diagnóstico: imprime a tabela, não afirma nada"]
fn diag_que_altura_tem_cada_marca() {
    let tudo = censo();
    let mut por_painel: std::collections::BTreeMap<
        &str,
        std::collections::BTreeMap<String, usize>,
    > = std::collections::BTreeMap::new();
    for m in &tudo {
        *por_painel
            .entry(m.painel)
            .or_default()
            .entry(format!("{} {:.1}", m.especie, m.altura))
            .or_default() += 1;
    }
    println!("ROW_H_PX = {:.1}", ph2d_tokens::ROW_H_PX);
    println!("CHECKBOX_BOX_PX = {:.1}", ph2d_tokens::CHECKBOX_BOX_PX);
    println!("{} marcas em {} painéis", tudo.len(), por_painel.len());
    for (painel, alturas) in &por_painel {
        let linha: Vec<String> = alturas
            .iter()
            .map(|(h, n)| format!("{h} px × {n}"))
            .collect();
        println!("  {painel:24} {}", linha.join(" · "));
    }
    println!(
        "\n-- as que NÃO medem {:.1} px, pelo nome --",
        ph2d_tokens::ROW_H_PX
    );
    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES_DOS_IDS, PISO_DOS_SLUGS);
    let mut fora: Vec<String> = tudo
        .iter()
        .filter(|m| (m.altura - ph2d_tokens::ROW_H_PX).abs() > 0.01)
        .map(|m| {
            format!(
                "  {:10} {:8} {:5.1} px  {}",
                m.painel,
                m.especie,
                m.altura,
                nomes
                    .get(&m.id)
                    .map_or_else(|| "(sem nome)".to_string(), Clone::clone)
            )
        })
        .collect();
    fora.sort_unstable();
    fora.dedup();
    for l in &fora {
        println!("{l}");
    }
}

/// ⭐⭐⭐ **O QUE NÃO É UMA LINHA DE FORMULÁRIO — e a excepção diz o NOME e o PORQUÊ.**
///
/// ⚠️ **Esta régua lê o ESTADO no store e o defeito vive no PINTOR**, que ela não vê. Um
/// `InteractiveState::Toggle` pode ser pintado por [`paint_button`] como um chamamento à acção, e
/// aí a altura dele é uma decisão de produto — não a altura de uma fileira.
///
/// ⛔ Uma lista destas sem censo de obsolescência é uma LICENÇA: a 2.ª metade do gate exige que
/// cada entrada **continue a existir e a medir o que declara**.
/// ⭐⭐ **A entrada é o SÍMBOLO, nunca um nome nem um número.** O `GS_SNAP_ENABLED` é um
/// [`NodeId`] **cru** (`NodeId(1060)`), logo o mapa inverso dos *slugs* não o sabe nomear — e
/// escrever `1060` aqui seria um literal que ninguém reconcilia. ⇒ quem o renomear ou apagar
/// **deixa este ficheiro de compilar**, que é a prova mais forte que existe.
const FORA_COM_NOME: &[(&str, ph2d_editor_core::NodeId, f32)] = &[
    // O botão grande de *Snap* no topo do painel: `paint_snap_top_toggle` pinta-o com
    // `paint_button` e declara o número no sítio (`LITERAL-PX-OK: primary CTA height (Snap toggle
    // hero) — taller than ROW_H so it reads as the panel's main action`).
    (
        "grid_snap",
        ph2d_editor_core::grid_snap::ids::GS_SNAP_ENABLED,
        44.0,
    ),
];

/// ⭐⭐ **A PORTA da isenção** — `(painel, id)`, e é um par por uma razão medida.
///
/// ⛔⛔ Ela é uma função e não um fecho dentro do gate porque a mutação que a **afrouxa para
/// casar só pelo PAINEL sobreviveu** (2026-09-21): hoje o `grid_snap` tem uma só marca fora do
/// padrão, logo a população do produto **não discrimina** as duas leituras. *Um corpus que não
/// contém o fenómeno não o pode testar* ⇒ o discriminador vive numa fixtura construída
/// ([`a_isencao_e_do_PAR_e_nao_do_painel`]), que é a única forma honesta de o afirmar.
fn isenta(painel: &str, id: ph2d_editor_core::NodeId) -> bool {
    FORA_COM_NOME
        .iter()
        .any(|(p, exid, _)| *p == painel && *exid == id)
}

/// ⭐⭐⭐ **UMA ISENÇÃO É DO PAR `(painel, id)`, NUNCA DO PAINEL.**
///
/// ⚠️ Sem ela, isentar um painel inteiro passaria despercebido — e o dia em que uma segunda marca
/// daquele painel encolhesse, ninguém saberia.
#[test]
fn a_isencao_e_do_par_e_nao_do_painel() {
    let (painel, declarada, _) = FORA_COM_NOME[0];
    assert!(
        isenta(painel, declarada),
        "a marca DECLARADA tem de ser isenta — senão a lista não descreve nada"
    );
    // ⭐ O CONTROLO: outra marca do MESMO painel, que ninguém declarou.
    let estranha = ph2d_editor_core::grid_snap::ids::GS_SNAP_CENTER;
    assert_ne!(
        declarada, estranha,
        "o controlo tem de ser um id DIFERENTE, senão ele não contradiz nada"
    );
    assert!(
        !isenta(painel, estranha),
        "uma marca NÃO declarada do mesmo painel ficou isenta — a isenção está a casar pelo \
         PAINEL, e com isso um painel inteiro sai da régua em silêncio"
    );
}

/// ⭐⭐⭐ **TODA MARCA BOOLEANA DE UM PAINEL TEM A ALTURA DE UMA LINHA DE PROPRIEDADE.**
///
/// Report do dono, 2026-09-21 — ver o cabeçalho deste ficheiro.
#[test]
fn toda_marca_booleana_tem_a_altura_de_uma_linha() {
    let tudo = censo();
    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES_DOS_IDS, PISO_DOS_SLUGS);
    let nome_de = |m: &Marca| {
        nomes
            .get(&m.id)
            .map_or_else(|| "(sem nome)".to_string(), Clone::clone)
    };

    // ── 1.ª metade: ninguém foge do padrão sem estar nomeado ──────────────────
    let mut fora: Vec<String> = tudo
        .iter()
        .filter(|m| (m.altura - ph2d_tokens::ROW_H_PX).abs() > 0.01 && !isenta(m.painel, m.id))
        .map(|m| {
            format!(
                "{}{} · {} · {} mede {:.1} px",
                m.painel,
                // ⚠️ Dizer em QUE passagem ela apareceu: uma marca que só existe com um documento
                //    na mão não se encontra abrindo o painel vazio.
                if m.armado { " (armado)" } else { "" },
                m.especie,
                nome_de(m),
                m.altura
            )
        })
        .collect();
    fora.sort_unstable();
    fora.dedup();
    assert!(
        fora.is_empty(),
        "estas marcas booleanas não medem a altura de uma linha ({:.1} px):\n  {}\n\n         ⚠️ A altura de uma LINHA é a [`ph2d_tokens::ROW_H_PX`]; a aresta da MARCA é a          [`ph2d_tokens::CHECKBOX_BOX_PX`] ({:.1}), e escrever a segunda onde se pedia a primeira          encolhe a moldura E a marca (que é `min(aresta, altura − 2·Xs)`).\n         ⇒ a cura é passar pela porta `ph2d_editor_core::property_row::paint_check_row`, que não          aceita altura nenhuma — nunca um literal novo no sítio da pintura.",
        ph2d_tokens::ROW_H_PX,
        fora.join("\n  "),
        ph2d_tokens::CHECKBOX_BOX_PX,
    );

    // ── 2.ª metade: a excepção ainda descreve alguma coisa ────────────────────
    let obsoletas: Vec<String> = FORA_COM_NOME
        .iter()
        .filter(|(p, id, h)| {
            !tudo
                .iter()
                .any(|m| m.painel == *p && m.id == *id && (m.altura - h).abs() <= 0.01)
        })
        .map(|(p, id, h)| format!("{p} · {id:?} · declarada {h:.1} px"))
        .collect();
    assert!(
        obsoletas.is_empty(),
        "estas isenções já não descrevem nada — ou a marca saiu, ou mudou de altura, e uma \
         isenção que não descreve nada é uma licença:\n  {}",
        obsoletas.join("\n  ")
    );
}

/// ⭐ Sonda: que NOMES de linha de texto o Inspector armado de facto pinta.
///
/// ⚠️ Ela existe porque as catracas de elisão ficaram **verdes** depois de `34` nomes novos, e
/// *«nenhum corte novo»* lê-se igual a *«a varredura não viu os nomes»*. Um censo que passa sem
/// ver a população não afirma nada.
#[test]
#[ignore = "diagnóstico: imprime o que foi pintado"]
fn diag_que_nomes_de_texto_o_inspector_pinta() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let procurados = [
        "Recipe",
        "On Signal",
        "Spawn Tag",
        "On Spawned",
        "On Exhausted",
        "Target",
        "Sound",
        "Script",
        "On Enter",
        "Counter",
    ];
    let mut achados: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut total = 0usize;
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg
            .panels_mut()
            .iter_mut()
            .filter(|p| p.manifest.id == "inspector")
        {
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == "inspector")
                .expect("armação");
            let mut host = MockPanelHost::new();
            (arm.arma)(host.store_mut());
            painel.populate(host.store_mut());
            abre_as_gavetas(host.store_mut());
            for m in host.medindo_a_pintura_do_registo(painel, viewport()) {
                total += 1;
                if procurados.contains(&m.texto.as_str()) {
                    achados.insert(m.texto.clone());
                }
            }
            (arm.desarma)();
        }
    });
    println!("{total} rótulos medidos no inspector armado");
    println!("nomes de linha de texto encontrados: {achados:?}");
}

/// ⭐ Sonda: onde COMEÇA uma caixa de texto, contra onde começa uma caixa de número.
#[test]
#[ignore = "diagnóstico: imprime a tabela"]
fn diag_onde_comeca_uma_caixa_de_texto() {
    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES_DOS_IDS, PISO_DOS_SLUGS);
    for (painel, texto, numero) in censo_das_colunas() {
        println!("{painel:22} texto {texto:?}  número {numero:?}");
    }
    println!("\n-- as caixas de texto do inspector, pelo nome --");
    for (id, x) in caixas_de_texto_do_inspector() {
        println!(
            "  x={x:6.1}  {}",
            nomes
                .get(&id)
                .map_or_else(|| format!("{id:?}"), Clone::clone)
        );
    }
}

/// `(painel, xs das caixas de TEXTO, xs das caixas de NÚMERO)` — pela porta do produto.
pub(crate) fn censo_das_colunas() -> Vec<(&'static str, Vec<i32>, Vec<i32>)> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut saida = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            if FORA_POR_DESENHO.contains(&id) {
                continue;
            }
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id);
            let mut host = MockPanelHost::new();
            if let Some(a) = arm {
                (a.arma)(host.store_mut());
            }
            painel.populate(host.store_mut());
            abre_as_gavetas(host.store_mut());
            let _ = host.medindo_a_pintura_do_registo(painel, viewport());
            let (mut t, mut n) = (Vec::new(), Vec::new());
            for (nid, r) in host.registos_da_ultima_pintura() {
                match host.store().get(nid) {
                    Some(InteractiveState::TextInput { .. }) => t.push(r.x.round() as i32),
                    Some(InteractiveState::NumberInput { .. }) => n.push(r.x.round() as i32),
                    _ => {}
                }
            }
            if let Some(a) = arm {
                (a.desarma)();
            }
            t.sort_unstable();
            t.dedup();
            n.sort_unstable();
            n.dedup();
            if !t.is_empty() {
                saida.push((id, t, n));
            }
        }
    });
    saida
}

/// `(id, x)` de cada caixa de TEXTO do Inspector armado.
pub(crate) fn caixas_de_texto_do_inspector() -> Vec<(ph2d_editor_core::NodeId, f32)> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut saida = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg
            .panels_mut()
            .iter_mut()
            .filter(|p| p.manifest.id == "inspector")
        {
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == "inspector")
                .expect("armação");
            let mut host = MockPanelHost::new();
            (arm.arma)(host.store_mut());
            painel.populate(host.store_mut());
            abre_as_gavetas(host.store_mut());
            let _ = host.medindo_a_pintura_do_registo(painel, viewport());
            for (nid, r) in host.registos_da_ultima_pintura() {
                if matches!(
                    host.store().get(nid),
                    Some(InteractiveState::TextInput { .. })
                ) {
                    saida.push((nid, r.x));
                }
            }
            (arm.desarma)();
        }
    });
    saida.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    saida
}

/// ⛔ **SONDA EXPLORATÓRIA (2026-09-22)** — que FORMA tem cada selector de cor do app.
///
/// Report do dono: *«os seletores de cor de todo o app precisam ser padronizados»*.
///
/// ⚠️ **A lista de ids vem do FONTE e o RECT vem do PRODUTO** — é a mesma troca que o
/// [`super::o_que_o_artista_nao_alcanca`] já faz, e a metade que interessa (a geometria) é medida.
#[test]
#[ignore]
fn diag_que_forma_tem_cada_seletor_de_cor() {
    let nomes = super::o_que_o_artista_nao_alcanca::nomes_de(FONTES_DAS_CORES, 400);
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut linhas: Vec<String> = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            abre_as_gavetas(host.store_mut());
            colhe_cores(&mut host, painel, id, &nomes, &mut linhas);

            // ⭐ A passagem ARMADA — cinco painéis pintam ZERO fileiras de fábrica, e o Inspector
            //   (o painel de que o report fala) é um deles.
            if let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            {
                let mut host = MockPanelHost::new();
                (arm.arma)(host.store_mut());
                painel.populate(host.store_mut());
                abre_as_gavetas(host.store_mut());
                colhe_cores(&mut host, painel, id, &nomes, &mut linhas);
                (arm.desarma)();
            }
        }
    });
    linhas.sort();
    linhas.dedup();
    eprintln!("=== {} selectores de cor pintados ===", linhas.len());
    for l in &linhas {
        eprintln!("{l}");
    }
}

/// ⛔ As árvores de ids que a sonda das cores lê.
///
/// ⚠️⚠️ **A 1.ª redacção lia OITO árvores e a sonda achava OITO selectores.** *A dívida não era o
/// que a régua dizia; a régua é que era estreita* — a mesma lição que o censo do HR-15 pagou em
/// 2026-09-20 (`44` de `325` crates). ⇒ ela lê hoje **todo** painel que declara ids
/// (`grep -l hash_node_id` sobre `crates/ph2d-panel-*/src`), mais as duas folhas que pintam
/// fileiras de cor por conta própria.
const FONTES_DAS_CORES: &[&str] = &[
    "../ph2d-panel-asset-browser/src",
    "../ph2d-panel-audio-editor/src",
    "../ph2d-panel-audio-mixer/src",
    "../ph2d-panel-authored/src",
    "../ph2d-panel-bgremoval/src",
    "../ph2d-panel-color-equalization/src",
    "../ph2d-panel-equalize-sizes/src",
    "../ph2d-panel-flip/src",
    "../ph2d-panel-flip-frames/src",
    "../ph2d-panel-grid-snap/src",
    "../ph2d-panel-hierarchy/src",
    "../ph2d-panel-inspector/src",
    "../ph2d-panel-model3d/src",
    "../ph2d-panel-motion-graph/src",
    "../ph2d-panel-padding/src",
    "../ph2d-panel-painter-layers/src",
    "../ph2d-panel-physics/src",
    "../ph2d-panel-sculpt3d/src",
    "../ph2d-panel-skeleton/src",
    "../ph2d-panel-tags/src",
    "../ph2d-panel-timeline/src",
    "../ph2d-panel-tokens/src",
    "../ph2d-panel-upscale/src",
    "../ph2d-panel-vector/src",
    "../ph2d-panel-wet-tuning/src",
    "../ph2d-panel-widget-lab/src",
    "../ph2d-editor-core/src/ids",
    "../ph2d-tool-vector/src",
    "../ph2d-param-editors/src",
];

fn colhe_cores(
    host: &mut MockPanelHost,
    painel: &mut ph2d_editor_core::panel::ErasedPanel,
    id: &'static str,
    nomes: &std::collections::BTreeMap<ph2d_editor_core::NodeId, String>,
    saida: &mut Vec<String>,
) {
    let _ = host.medindo_a_pintura_do_registo(painel, viewport());
    // ⛔⛔ **O QUE A SONDA NÃO SABE NOMEAR CONTA-SE, e a 1.ª redacção deitava-o fora em SILÊNCIO.**
    //
    // O mapa inverso é feito de LITERAIS (`hash_node_id("…")`), logo um id **COMPUTADO** —
    // `tokens_swatch_id(row)`, `vector_paint_swatch_id(i)`, `painter_shape_layer_color_swatch_id`
    // — não tem slug nenhum e caía neste `continue`. *Uma varredura que ignora o que não sabe
    // nomear mede o alcance do NOME, não o do produto*, e é por isso que ela lia `8` selectores
    // num app que pinta mais.
    let mut anonimos = 0usize;
    for (nid, r) in host.registos_da_ultima_pintura() {
        let Some(slug) = nomes.get(&nid) else {
            anonimos += 1;
            continue;
        };
        let s = slug.to_ascii_lowercase();
        if !(s.contains("swatch") || s.contains("color") || s.contains("colour")) {
            continue;
        }
        // ⛔⛔ **DUAS populações que o nome NÃO separa, e confundi-las era o erro:**
        //   - `*_section` é a BANDA DE DOBRA de uma secção (largura do painel inteiro);
        //   - `insp_live_*_color` / `insp_section_*_color` é o PONTO de cor do CABEÇALHO
        //     (um disco de 18 px na borda direita do título — `color_circle_hit_rect`).
        // Nenhum dos dois é uma linha de propriedade, e padronizá-los ao controlo seria
        // trocar um cabeçalho por um campo.
        // ⭐⭐ **O que se EXCLUI imprime-se, e isso não é enfeite.** A 1.ª redacção desta sonda
        //    deitava as duas populações fora em silêncio e lia `8` selectores; um `swatch`
        //    QUADRADO numa linha de propriedade cai na mesma regra que um ponto de cabeçalho, e
        //    *uma exclusão muda faz a régua parecer uma medição do produto*.
        let balde = if s.ends_with("_section") {
            "BANDA "
        } else if s.ends_with("_color") && (r.w - r.h).abs() < 0.5 {
            "PONTO "
        } else {
            "FILEIRA"
        };
        saida.push(format!(
            "{balde} {id:<16} w={:6.1} h={:5.1} x={:6.1}  {slug}",
            r.w, r.h, r.x
        ));
    }
    if anonimos > 0 {
        saida.push(format!(
            "ANONIMO {id:<16} {anonimos} rect(s) SEM SLUG (id computado ou arvore de ids fora \
             das FONTES). ⚠️ E' o total do painel, NAO so' as cores: a sonda filtra por NOME, logo \
             um selector aqui dentro e' invisivel para ela"
        ));
    }
}
