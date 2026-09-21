//! ⭐⭐⭐ **O PAINEL DA ESCULTURA, ARMADO — e a declaração que o dispensava era um palpite.**
//!
//! # ⛔⛔ A ausência afirmada, medida
//!
//! A varredura declarava este painel em [`super::nenhum_rotulo_do_app_pinta_nada`] como *«o painel
//! da escultura pinta o que a `AppGfx.sculpt3d` publica, e essa cena segura uma surface de wgpu —
//! o arnês corre sem dispositivo, logo sem peça»*. A primeira metade é verdade sobre **quem
//! publica** e falsa sobre **o que é publicado**: o que atravessa é um
//! [`ph2d_panel_sculpt3d::Sculpt3dSnapshot`] por uma porta `thread_local`
//! (`set_current_sculpt3d`), e um snapshot é **dados** — o painel nunca vê malha nem device (o
//! doc do próprio `matcap_keys` escreve essa aresta de dependência por extenso).
//!
//! ⇒ é a **terceira** vez neste repo que uma ausência é afirmada pelo ENDEREÇO de quem publica em
//! vez da API da porta.
//!
//! # ⭐⭐ Aqui os rótulos são ESTÁTICOS, e por isso a fixtura é magra
//!
//! Ao contrário dos params do Motion e do modelador, este painel **não recebe rótulos no
//! retrato**: as fileiras dele vivem numa tabela da própria crate
//! (`rows_sections::SECTIONS` + `rows::rows()`), com a chave i18n escrita na `Row`. ⇒ o que o
//! retrato compra é o painel **pintar de todo** — sem ele o `paint` sai cedo com uma frase de
//! vazio, que é o `0` que a varredura media.
//!
//! ⚠️ **O nível é `Pro` de propósito:** o `Row::shows` esconde as fileiras avançadas em `Basic`, e
//! o que esta passagem existe para fazer é pintar o máximo de rótulos. ⛔ Não é uma escolha de
//! gosto — é a mesma razão pela qual o `folded_by_default` da fixtura do Motion fica vazio.

use std::path::PathBuf;
use std::sync::OnceLock;

use ph2d_panel_sculpt3d::{Sculpt3dSnapshot, Sculpt3dUi, UiLevel, set_current_sculpt3d};

/// A tabela dos nomes dos MATERIAIS.
///
/// ⚠️ **Não é a `sculpt3d.rs`, e a diferença é do domínio:** os rótulos do PAINEL vivem ali, e
/// estes são nomes do MOTOR (`sculpt_engine.rs`) — a mesma partição que põe os manifestos dos nós
/// fora da tabela dos painéis do Motion. *Escrever o ficheiro errado aqui devolve zero, e o piso
/// de população é o que o disse em voz alta na primeira corrida.*
const TABELA: &str = "../ph2d-i18n/src/sculpt_engine.rs";

/// ⛔ **Piso de população.** Medido em 2026-09-19: **`10`** matcaps declarados.
const PISO_DE_MATCAPS: usize = 8;

fn ler(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", p.display()))
}

/// ⭐⭐ **A lista dos MATERIAIS, derivada da tabela.**
///
/// ⚠️ **Ela é o único texto que este painel recebe no RETRATO** — o resto das chaves dele vive na
/// tabela da crate. No produto ela vem da `ph2d-mesh-render` (que carrega o `wgpu` inteiro), e é
/// por isso que aqui se lê a tabela de i18n: *um arnês que importasse o renderizador para colher
/// dez palavras compilaria um backend gráfico para medir uma caixa.*
///
/// ⛔ Piso de população: um extractor que deixe de casar devolveria a lista vazia, o painel
/// pintaria só a opção do rig e a varredura leria menos rótulos — *e menos lê-se como aprovação*.
fn matcaps() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<(String, String)> = ph2d_label_census::keys::declared_pairs_in(&ler(TABELA))
            .into_iter()
            .filter(|p| p.chave.starts_with("sculpt3d.matcap."))
            .map(|p| (p.chave, p.texto))
            .collect();
        assert!(
            v.len() >= PISO_DE_MATCAPS,
            "o extractor leu {} matcaps (piso {PISO_DE_MATCAPS}) — a tabela mudou de forma",
            v.len()
        );
        // ⚠️ Do mais LARGO PINTADO para o mais estreito — o primeiro é o que a caixa tem de aguentar.
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let f = ph2d_tokens::TypeToken::Sm.px();
        v.sort_by(|a, b| {
            ts.prefix_width(&b.1, f)
                .total_cmp(&ts.prefix_width(&a.1, f))
        });
        v.into_iter()
            .map(|(k, _)| &*Box::leak(k.into_boxed_str()))
            .collect()
    })
}

/// Publica a escultura de fábrica impossível: tudo o que o painel sabe mostrar, mostrado.
///
/// ⭐⭐ **Sem `..Sculpt3dSnapshot::default()`, de propósito** — ela preenche os quinze campos pelo
/// nome, logo **um campo novo no retrato é erro de compilação aqui**. O clippy apanhou o
/// `..default()` morto da 1.ª redacção, e tirá-lo trocou um censo textual por uma prova do
/// compilador: *a irmã do modelador precisa de um gate para a mesma garantia porque lá as fileiras
/// são `Vec`s que podem ficar vazias sem deixar de compilar.*
pub fn arma() {
    set_current_sculpt3d(Some(retrato(UiLevel::Pro, true)));
}

/// ⭐⭐⭐ **O MESMO retrato, no estado do DIA A DIA** — nível `Basic` e o filtro DESARMADO.
///
/// ⛔⛔ **Ele existe porque a irmã arma o estado MÁXIMO, e as duas respondem a perguntas
/// diferentes.** A [`arma`] pinta tudo de propósito (é o que faz uma régua de LARGURA ver todos os
/// rótulos), e medir a ALTURA do painel nela lê o **pior caso**: medido em 2026-09-20, as catorze
/// fichas do filtro sozinhas valem `~221 px`, e elas **não são pintadas** enquanto o artista não
/// armar o filtro. *Uma coluna que colapsa dois estados num número descreve um app que ninguém
/// usa.*
///
/// ⚠️ **Ela partilha o construtor** ([`retrato`]) — duas cópias divergiriam no primeiro campo novo,
/// e a garantia que o cabeçalho deste ficheiro promete (*um campo novo é erro de compilação aqui*)
/// vale por haver **UM** sítio a nomear os quinze.
pub fn arma_o_dia_a_dia() {
    set_current_sculpt3d(Some(retrato(UiLevel::Basic, false)));
}

/// O retrato, com os dois botões que separam o **pior caso** do **dia a dia**.
fn retrato(nivel: UiLevel, filtro_armado: bool) -> Sculpt3dSnapshot {
    let ui = Sculpt3dUi {
        // ⚠️ Ver o cabeçalho: em `Basic` metade das fileiras não é pintada.
        ui_level: nivel,
        // Um material escolhido — sem isto o chip do rig ganha e os dez nomes não são medidos.
        // ⚠️⚠️ **O campo `matcap: Option<u8>` MORREU na `line/sculpt3d`** (o commit da luz PLANA):
        //    com que luz olhar passou a ser UM enum de três estados (`Flat` · `Rig` · `Matcap(i)`),
        //    porque os dois campos antigos podiam dizer coisas contraditórias ao mesmo tempo. Esta
        //    fixtura nasceu no `main` a escrever o campo velho ⇒ ela **não compilava** na árvore
        //    combinada, e é a colisão semântica que o §1.5.5 nomeia: *um campo que muda de forma
        //    funde limpo e não compila*. ⭐ A tradução é exacta e diz o MESMO: escolher o matcap `0`.
        lighting: ph2d_panel_sculpt3d::state::LightMode::Matcap(0),
        alpha_preview: true,
        wireframe: true,
        ..Sculpt3dUi::default()
    };
    Sculpt3dSnapshot {
        ui,
        // ⚠️ O filtro ARMADO pinta a fileira dele, que é outra caixa. ⛔ Ele e o `transform` são
        //    mutuamente exclusivos por construção na cena — armar os dois pintaria um estado que o
        //    produto não sabe representar.
        transform: None,
        filter_armed: filtro_armado,
        dyntopo: true,
        level: 1,
        level_count: 3,
        // ⭐ O AO velho pinta o aviso de obsolescência, que é uma FRASE — e uma frase é exactamente
        //   o que a lei da reticência corta pela metade.
        ao_stale: true,
        // ⚠️ O nome do sprite de onde o padrão veio: o rótulo de um chip, e um nome de ficheiro
        //   real é comprido.
        alpha_image_name: Some(std::sync::Arc::from("skin-pores-large-01 (imported).png")),
        pieces: 4,
        isolated: true,
        verts: 98_306,
        matcap_keys: matcaps(),
        alpha_seed: 0.35,
        model_span: 2.0,
        has_bake_target: true,
        // ⚠️ **ASSADO e na lei da FORMA** (o índice `1`): esta fixtura arma o que PINTA MAIS, e a
        // fileira da lei só existe quando o sprite escolhido já tem canais assados.
        lei_do_alvo: Some(1),
        lei_rotulos: &[
            "panel.sculpt3d.bake_law.paint",
            "panel.sculpt3d.bake_law.form",
        ],
    }
}

/// ⛔ **Obrigatório:** a porta é `thread_local` e o binário de teste corre todos os módulos na
/// mesma thread. *O estado que uma fixtura deixa para trás é o estado que a régua seguinte mede.*
pub fn desarma() {
    set_current_sculpt3d(None);
}

/// ⭐⭐ **A fixtura arma o nível que pinta MAIS, e isto prova que os dois níveis diferem.**
///
/// ⛔ Sem a segunda metade, alguém podia trocar o `Pro` por `Basic` sem nada reprovar — e a
/// varredura passaria a medir metade das fileiras deste painel em silêncio. *Um nível escolhido
/// sem uma régua que o compare com o outro é uma preferência, não uma lei.*
#[test]
fn a_fixtura_arma_o_nivel_que_pinta_mais() {
    const ESTE: &str = include_str!("o_sculpt3d_armado.rs");
    // ⛔⛔ **A AGULHA MONTA-SE, e a razão é que este gate lê o PRÓPRIO ficheiro.** A redacção
    //    anterior procurava `ui_level: UiLevel::Pro`, e essa frase aparece **no corpo deste
    //    teste** (o `pro` que ele constrói para comparar) ⇒ ela era satisfeita por si mesma, e
    //    trocar o nível na fixtura tê-lo-ia deixado VERDE. *Um censo textual que se lê a si mesmo
    //    encontra sempre o que procura.*
    // ⭐ A forma montada nunca existe inteira no fonte, que é a mesma cura que a vassoura do HR-15
    //    desta casa usa.
    let agulha = concat!("retrato(UiLevel::", "Pro, true)");
    assert!(
        ESTE.contains(agulha),
        "a fixtura deixou de armar o pior caso — ela tem de pedir o nivel que pinta MAIS e o \
         filtro ARMADO, senao a varredura de largura deixa de ver metade dos rotulos deste painel"
    );
    let base = Sculpt3dUi::default();
    let pro = Sculpt3dUi {
        ui_level: UiLevel::Pro,
        ..base.clone()
    };
    let basico = Sculpt3dUi {
        ui_level: UiLevel::default(),
        ..base
    };
    // ⚠️ A régua é a PORTA do painel (`Row::visible`), nunca uma segunda cópia do `&&` — o doc
    //    dela diz que três cópias divergiriam no dia em que nascesse a terceira pergunta.
    let conta = |ui: &Sculpt3dUi| {
        ph2d_panel_sculpt3d::rows::rows()
            .filter(|r| r.visible(ui))
            .count()
    };
    assert!(
        conta(&pro) > conta(&basico),
        "o nivel `Pro` pinta {} fileiras e o de fabrica {} — se sao iguais, a escolha desta \
         fixtura nao compra nada e a regua que a defende nao afirma nada",
        conta(&pro),
        conta(&basico)
    );
}

/// ⭐ **E a lista de materiais que ela arma é a mais larga primeiro.**
#[test]
fn os_materiais_da_fixtura_vem_do_mais_largo() {
    let m = matcaps();
    assert!(m.len() >= PISO_DE_MATCAPS);
    let pares: Vec<(String, String)> = ph2d_label_census::keys::declared_pairs_in(&ler(TABELA))
        .into_iter()
        .filter(|p| p.chave.starts_with("sculpt3d.matcap."))
        .map(|p| (p.chave, p.texto))
        .collect();
    let texto = |k: &str| {
        pares
            .iter()
            .find(|(c, _)| c == k)
            .map(|(_, t)| t.clone())
            .unwrap_or_default()
    };
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let f = ph2d_tokens::TypeToken::Sm.px();
    for par in m.windows(2) {
        assert!(
            ts.prefix_width(&texto(par[0]), f) >= ts.prefix_width(&texto(par[1]), f),
            "{:?} pinta menos do que {:?} e vem antes dele",
            par[0],
            par[1]
        );
    }
    // ⭐ O controlo: se todos pintassem o mesmo, a ordenação não mediria caixa nenhuma.
    assert!(
        ts.prefix_width(&texto(m[0]), f) > ts.prefix_width(&texto(m[m.len() - 1]), f),
        "todos os materiais pintam o mesmo — a fixtura nao mede caixa nenhuma"
    );
}
