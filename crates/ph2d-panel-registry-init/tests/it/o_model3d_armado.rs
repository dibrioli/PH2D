//! ⭐⭐⭐ **O PAINEL DO MODELADOR 3D, ARMADO — e os rótulos são os do CATÁLOGO, derivados.**
//!
//! # ⛔⛔ A ausência afirmada, medida
//!
//! A varredura declarava este painel em [`super::nenhum_rotulo_do_app_pinta_nada`] como
//! *«pinta a árvore de um documento de campo implícito (`FieldDoc`), que é COZIDO da hierarquia da
//! cena a cada quadro; sem mundo ECS ele não tem uma peça para listar»* — e essa frase é **uma
//! ausência afirmada sem olhar a API**, a mesma armadilha que o painel de params do Motion já tinha
//! pago no dia anterior. O painel não vê `FieldDoc` nenhum: ele lê um
//! [`ph2d_panel_model3d::ModelSnapshot`] publicado numa porta `thread_local`
//! (`ph2d_panel_model3d::publish`), e um snapshot é **dados**.
//!
//! ⇒ **`161` rótulos de dimensão** (`field.dim.*`) e **quinze fileiras de chips** nunca tinham
//! passado por uma régua de largura.
//!
//! # ⭐⭐ Cada fileira leva a população DELA, nunca as palavras mais largas da tabela
//!
//! Uma fileira de chips é dimensionada pela LISTA a que ela pertence, e juntar as mais largas de
//! listas diferentes construiria uma fileira que o produto nunca desenha — a fabricação que a
//! fixtura do Motion já pagou. ⇒ [`chips`] filtra por **família de chave**
//! (`panel.model3d.<família>.*`), que é exactamente o que o `scene_panel.rs` publica em cada campo.
//!
//! ⛔ **E as `75` formas do catálogo NÃO entram aqui, de propósito.** O campo `adds` leva **dois**
//! chips no produto (`add.open` e `add.light`); os outros `73` são os nomes das formas, e quem os
//! pinta é a **paleta** — uma janela flutuante do `ph2d-app-field3d`, que esta varredura não
//! alcança (ela pinta os painéis do registo). *Pô-los na fileira de chips mediria uma caixa que o
//! produto nunca desenha para eles.* A separação é **derivada**: `add.*` menos o que a
//! `shapes_table.rs` declara.

use std::path::PathBuf;
use std::sync::OnceLock;

use ph2d_field::{Bound, Param};
use ph2d_panel_model3d::{ModeChip, ModelSnapshot, ParamRow, publish};

/// A tabela dos rótulos deste painel.
const TABELA: &str = "../ph2d-i18n/src/model3d.rs";

/// A tabela das FORMAS — lida só para as **tirar** da fileira de criar. Ver o cabeçalho.
const TABELA_DE_FORMAS: &str = "../ph2d-app-field3d/src/shapes_table.rs";

/// ⛔ **Pisos de população.** Medidos em 2026-09-19: `161` chaves `field.dim.*`, `75` chaves
/// `panel.model3d.add.*` (das quais `73` são formas), `4` secções.
const PISO_DE_DIMENSOES: usize = 140;
const PISO_DE_FORMAS: usize = 60;
const PISO_DE_SECCOES: usize = 4;

fn ler(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", p.display()))
}

/// ⭐⭐ **Quem lê uma tabela de strings é a RÉGUA REGISTADA, nunca um leitor novo** — ver o doc de
/// [`ph2d_label_census::keys::declared_pairs_in`], que conta o preço de ter havido dois leitores.
fn pares(rel: &str) -> Vec<ph2d_label_census::keys::Par> {
    ph2d_label_census::keys::declared_pairs_in(&ler(rel))
}

fn fonte() -> f32 {
    ph2d_tokens::TypeToken::Sm.px()
}

/// Ordena `(chave, texto)` pela LARGURA PINTADA do texto, do mais largo para o mais estreito.
///
/// ⚠️ **Largura pintada e não contagem de letras** — `WWWW` pinta mais do que `iiii`, e a régua
/// desta varredura é a caixa.
fn por_largura(mut v: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let f = fonte();
    v.sort_by(|a, b| {
        ts.prefix_width(&b.1, f)
            .total_cmp(&ts.prefix_width(&a.1, f))
    });
    v
}

/// As chaves do catálogo com um dado prefixo, do rótulo mais largo para o mais estreito.
fn familia(prefixo: &str) -> Vec<(String, String)> {
    por_largura(
        pares(TABELA)
            .into_iter()
            .filter(|p| p.chave.starts_with(prefixo))
            .map(|p| (p.chave, p.texto))
            .collect(),
    )
}

/// ⭐ **As chaves das FORMAS** — lidas para serem tiradas da fileira de criar.
fn formas() -> &'static std::collections::BTreeSet<String> {
    static CACHE: OnceLock<std::collections::BTreeSet<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let fonte = ler(TABELA_DE_FORMAS);
        let out: std::collections::BTreeSet<String> = fonte
            .lines()
            .filter_map(|l| l.trim().strip_prefix("key: \""))
            .filter_map(|r| r.split_once('"').map(|(k, _)| k.to_string()))
            .filter(|k| k.starts_with("panel.model3d.add."))
            .collect();
        assert!(
            out.len() >= PISO_DE_FORMAS,
            "o extractor leu {} formas (piso {PISO_DE_FORMAS}) — a tabela mudou de forma, e sem \
             ela a fileira de criar levaria as 75 formas numa caixa que o produto nunca desenha \
             para elas.",
            out.len()
        );
        out
    })
}

/// ⭐ **Uma fileira de chips com a população DELA.** O primeiro é o aceso.
fn chips(sufixo: &str) -> Vec<ModeChip> {
    let prefixo = format!("panel.model3d.{sufixo}.");
    let v = familia(&prefixo);
    assert!(
        !v.is_empty(),
        "a familia `{prefixo}*` leu ZERO chaves — uma fileira vazia nao e' pintada, e zero \
         le^-se como aprovacao"
    );
    v.into_iter()
        .enumerate()
        .map(|(i, (chave, _))| ModeChip {
            key: derramar(chave),
            active: i == 0,
        })
        .collect()
}

/// A fileira de CRIAR — a família `add.*` **menos** as formas. Ver o cabeçalho.
fn chips_de_criar() -> Vec<ModeChip> {
    let formas = formas();
    let v: Vec<ModeChip> = familia("panel.model3d.add.")
        .into_iter()
        .filter(|(k, _)| !formas.contains(k))
        .enumerate()
        .map(|(i, (chave, _))| ModeChip {
            key: derramar(chave),
            active: i == 0,
        })
        .collect();
    assert!(
        !v.is_empty(),
        "a fileira de criar ficou VAZIA — ou a tabela de formas passou a conter tudo, ou o \
         prefixo mudou"
    );
    v
}

/// As chaves das DIMENSÕES, do rótulo mais largo para o mais estreito.
fn dimensoes() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let v = familia("field.dim.");
        assert!(
            v.len() >= PISO_DE_DIMENSOES,
            "o extractor leu {} dimensoes (piso {PISO_DE_DIMENSOES}) — ou a tabela mudou de \
             forma, ou o ficheiro mudou de sitio. Uma fixtura que arma rotulos VAZIOS devolve \
             zero cortes, e zero le^-se como aprovacao.",
            v.len()
        );
        v.into_iter().map(|(k, _)| derramar(k)).collect()
    })
}

/// As chaves das SECÇÕES, do rótulo mais largo para o mais estreito.
fn seccoes() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let v = familia("panel.model3d.section.");
        assert!(
            v.len() >= PISO_DE_SECCOES,
            "o extractor leu {} seccoes (piso {PISO_DE_SECCOES})",
            v.len()
        );
        v.into_iter().map(|(k, _)| derramar(k)).collect()
    })
}

/// ⚠️ O `&'static str` vem de um derrame **único** (as listas vivem num `OnceLock`): o alvo da
/// medição é o texto, e o tempo de vida é um detalhe do tipo do campo.
fn derramar(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn dim(i: usize) -> &'static str {
    dimensoes()[i]
}

/// ⭐ **UMA fileira de cada FORMA de linha** — o que se mede aqui é a caixa de cada forma, e o
/// catálogo tem `161` rótulos para meia dúzia de formas.
fn fileiras() -> Vec<ParamRow> {
    let sec = seccoes();
    let base = |param: Param, key: &'static str| ParamRow {
        entity: 7,
        param,
        key,
        value: 1.5,
        lo: 0.0,
        live: true,
        integral: false,
        bound: Bound::Soft(10.0),
        section: None,
        choices: &[],
        swatch: None,
        subject: None,
    };
    vec![
        // A posição: um número com PISO negativo e um cabeçalho de secção por cima.
        ParamRow {
            lo: -10.0,
            value: -2.5,
            section: Some(sec[0]),
            ..base(Param::Pos(0), dim(0))
        },
        // Uma dimensão da forma — o rótulo mais largo do catálogo.
        ParamRow {
            section: Some(sec[1]),
            ..base(Param::Dim(0), dim(1))
        },
        // ⚠️ Uma CONTAGEM: passo inteiro, sem casas, e o piso é `1`.
        ParamRow {
            integral: true,
            value: 6.0,
            lo: 1.0,
            bound: Bound::Hard(64.0),
            ..base(Param::Dim(1), dim(2))
        },
        // ⚠️ Um ÂNGULO, cuja ponta é a própria representação.
        ParamRow {
            value: 45.0,
            lo: -180.0,
            bound: Bound::Wrap(180.0),
            ..base(Param::Rot(2), dim(3))
        },
        // ⛔ Uma linha que NÃO pode ser mexida — ela é pintada como um FACTO (rótulo e número),
        //    sem pista e sem campo, que é outra caixa.
        ParamRow {
            live: false,
            ..base(Param::Rot(1), dim(4))
        },
        // ⭐ Uma ESCOLHA — a lista REAL do produto (o único `Span::Choice` que ele declara).
        ParamRow {
            integral: true,
            value: 1.0,
            bound: Bound::Hard(2.0),
            choices: &ph2d_field::Axis::KEYS,
            ..base(Param::Dim(2), dim(5))
        },
        // ⭐ Uma AMOSTRA de cor — a linha deixa de ser um número.
        ParamRow {
            swatch: Some([200, 120, 60]),
            section: Some(sec[2]),
            ..base(Param::Dim(3), dim(6))
        },
        // ⭐ Uma linha com SUJEITO — o nome de quem ela descreve, que come largura.
        ParamRow {
            subject: Some("Background Parallax Layer".to_string()),
            ..base(Param::Scale, dim(7))
        },
    ]
}

/// Publica o modelo de fábrica impossível: cada fileira de chips cheia com a população dela.
pub fn arma() {
    publish(ModelSnapshot {
        modes: chips("mode"),
        frames: chips("frame"),
        selects: chips("select"),
        adds: chips_de_criar(),
        ops: chips("op"),
        verbs: chips("verb"),
        verb_subject: Some("Enemy Spawner · left wing".to_string()),
        characters: chips("character"),
        mods: chips("mod"),
        exports: chips("export"),
        acts: chips("act"),
        views: chips("view"),
        camera: chips("camera"),
        rows: fileiras(),
        isolated: Some("Collision Geometry".to_string()),
        node_count: 42,
        last_trace_ms: 26.7,
        view_label: "viewport.model3d.view.user",
        shadings: chips("shading"),
        looks: chips("look"),
        exposures: chips("exposure"),
        shading_label: "panel.model3d.shading.solid",
    });
}

/// ⛔ **Obrigatório:** a porta é `thread_local` e o binário de teste corre todos os módulos na
/// mesma thread. *O estado que uma fixtura deixa para trás é o estado que a régua seguinte mede.*
pub fn desarma() {
    publish(ModelSnapshot::default());
}

/// ⭐⭐ **A fixtura enche TODA fileira de chips do retrato — e a lista sai do STRUCT, não daqui.**
///
/// ⛔ Sem isto, um campo `Vec<ModeChip>` novo nasceria **vazio** e esta passagem continuaria verde:
/// *uma lista escrita à mão ao lado de um struct é a lista que apodrece*.
///
/// ⚠️ **Ela é TEXTUAL**, com o que isso implica: ela não guarda contra quem a quer contornar,
/// guarda contra o campo novo que ninguém se lembrou de encher. As duas metades são obrigatórias —
/// sem o piso de população, um extractor que deixasse de casar `pub struct ModelSnapshot {` leria
/// zero campos e `em_falta` ficaria trivialmente vazio.
#[test]
fn a_fixtura_enche_toda_fileira_de_chips() {
    const ESTE: &str = include_str!("o_model3d_armado.rs");
    let fonte = ler("../ph2d-panel-model3d/src/state.rs");
    let corpo = fonte
        .split_once("pub struct ModelSnapshot {")
        .expect("o struct ModelSnapshot mudou de forma")
        .1
        .split_once("\n}")
        .expect("o struct ModelSnapshot nao fecha")
        .0;
    let campos: Vec<&str> = corpo
        .lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix("pub "))
        .filter_map(|l| l.split_once(':'))
        .filter(|(_, t)| t.trim().starts_with("Vec<ModeChip>"))
        .map(|(n, _)| n)
        .collect();
    assert!(
        campos.len() >= 15,
        "o extractor leu {} fileiras de chips (piso 15) — o struct mudou de forma",
        campos.len()
    );
    let em_falta: Vec<&str> = campos
        .iter()
        .copied()
        .filter(|c| !ESTE.contains(&format!("{c}: chips")))
        .collect();
    assert!(
        em_falta.is_empty(),
        "estas fileiras de chips do painel do modelador NAO sao enchidas, logo nenhuma regua de \
         largura as ve^: {em_falta:?}"
    );
}

/// ⭐ **E os rótulos que ela arma são os MAIS LARGOS do catálogo, nesta ordem.**
///
/// ⛔ A metade que interessa é a segunda: sem ela, um extractor que devolvesse a lista por ordem
/// alfabética passaria — e a fixtura mediria a caixa com uma palavra qualquer.
#[test]
fn os_rotulos_da_fixtura_sao_os_mais_largos_do_catalogo() {
    let v = familia("field.dim.");
    assert!(v.len() >= PISO_DE_DIMENSOES);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let f = fonte();
    for par in v.windows(2) {
        assert!(
            ts.prefix_width(&par[0].1, f) >= ts.prefix_width(&par[1].1, f),
            "{:?} pinta menos do que {:?} e vem antes dele",
            par[0].1,
            par[1].1
        );
    }
    assert!(
        ts.prefix_width(&v[0].1, f) > ts.prefix_width(&v[v.len() - 1].1, f),
        "todas as dimensoes pintam o mesmo — a fixtura nao mede caixa nenhuma"
    );
}

/// ⛔⛔ **As `73` formas do catálogo NÃO entram na fileira de criar, e isto prova-o.**
///
/// ⚠️ A metade negativa é a que interessa: sem ela, alargar o filtro devolveria uma fileira de
/// `75` chips e a varredura mediria uma caixa que o produto nunca desenha para aquelas palavras.
#[test]
fn a_fileira_de_criar_nao_leva_as_formas() {
    let criar = chips_de_criar();
    let formas = formas();
    assert!(
        criar.len() <= 4,
        "a fileira de criar tem {} chips — o produto publica DOIS (`add.open` e `add.light`), e \
         os outros sao as formas, que a PALETA pinta",
        criar.len()
    );
    for c in &criar {
        assert!(
            !formas.contains(c.key),
            "{:?} e' uma FORMA e esta' na fileira de criar",
            c.key
        );
    }
    // ⭐ O controlo positivo: a família inteira é muito maior, senão o filtro podia estar a
    //   funcionar por a tabela estar vazia.
    assert!(
        familia("panel.model3d.add.").len() >= PISO_DE_FORMAS,
        "a familia `add.*` encolheu — o filtro deixou de ter o que filtrar"
    );
}
