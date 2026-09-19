//! ⭐⭐⭐ **O PAINEL DE PARAMS DO MOTION, ARMADO — e os rótulos são os do CATÁLOGO, derivados.**
//!
//! # ⛔⛔ O buraco, medido
//!
//! A varredura de elisões declarava este painel em [`super::nenhum_rotulo_do_app_pinta_nada`]
//! como *«pinta os params do nó ESCOLHIDO no grafo, logo é vazio enquanto não houver grafo nem
//! escolha»* — e essa frase é **uma ausência afirmada sem olhar a API**, que é a armadilha que
//! este repo já pagou três vezes. O painel não precisa de grafo nenhum: ele lê um
//! [`ph2d_panel_motion_params::ParamsSnapshot`] publicado numa porta `thread_local`
//! (`set_current_params`), e um snapshot é **dados**, não um motor.
//!
//! ⇒ **`828` rótulos de parâmetro** do catálogo de nós nunca tinham passado por uma régua de
//! largura. Nenhum dos 30 censos do HR-15 os mede (eles perguntam *«vem da tabela?»*), e esta
//! varredura media este painel a **zero**.
//!
//! # ⭐⭐⭐ Os rótulos são DERIVADOS da tabela, nunca escritos aqui
//!
//! [`rotulos_do_catalogo`] lê as duas tabelas de `ph2d-i18n` e devolve os rótulos ordenados
//! **pelo que eles PINTAM** — não pelo número de letras, que é outra grandeza (`WWWW` é mais
//! largo que `iiii`). A fixtura leva os mais largos, um por espécie de fileira.
//!
//! ⚠️ **É isso que a impede de envelhecer:** um rótulo mais largo acrescentado amanhã entra na
//! medição sem ninguém editar este ficheiro. Uma lista escrita à mão mediria o catálogo do dia em
//! que foi escrita — que é como o `26 px` do `hand_right` chegou a estar na dívida com o número
//! errado.
//!
//! ⛔ **E há piso de população nas duas leituras**: um extractor que deixe de casar (a tabela
//! mudar de forma, o ficheiro mudar de sítio) devolveria uma lista vazia, a fixtura pintaria
//! rótulos vazios e a varredura leria **zero cortes** — *zero lê-se como aprovação*.
//!
//! # ⚠️ Nada é dobrado de propósito
//!
//! O `folded_by_default` fica **vazio**: uma secção fechada esconde as fileiras dela, e o que esta
//! passagem existe para fazer é PINTAR o máximo de rótulos. A dobra tem gates próprios no painel.

use std::path::PathBuf;
use std::sync::OnceLock;

use ph2d_panel_motion_params::{
    AngleRow, ChannelsRow, ColorRow, CurveRow, EnumRow, FileRow, GradientRow, PaletteRow, ParamRow,
    ParamsSnapshot, RowDisplay, ScalarRow, SeedRow, SourceRow, TextRow, ToggleRow,
    set_current_params,
};

/// As tabelas do catálogo, lidas em tempo de corrida.
///
/// ⚠️ **Por caminho relativo ao `CARGO_MANIFEST_DIR`, como a irmã [`super::o_inspector_armado`]** —
/// e é por isso que ela tem piso: mover a crate de sítio devolve ficheiro nenhum, e um extractor
/// que lê zero é indistinguível de um catálogo vazio.
const TABELAS: &[&str] = &[
    "../ph2d-i18n/src/node_params.rs",
    "../ph2d-i18n/src/node_params_motion.rs",
];

/// O ficheiro dos nomes de SECÇÃO — outra tabela, outra forma (`"chave" => "texto"`).
const TABELA_DE_GRUPOS: &str = "../ph2d-i18n/src/node_groups.rs";

/// O ficheiro das OPÇÕES de cada selector — as RESPOSTAS, onde as irmãs guardam a pergunta.
const TABELA_DE_OPCOES: &str = "../ph2d-i18n/src/node_options.rs";

/// ⛔ **Pisos de população.** Medidos em 2026-09-19: as tabelas declaram **`828`** entradas
/// `node.*.param.*`, que depois de deduplicadas são **`444`** rótulos DISTINTOS — muitos nós
/// partilham a mesma palavra (`Seed`, `Amount`, `Mode`). ⚠️ *O piso é sobre a população que a
/// fixtura de facto escolhe, que é a deduplicada* — a 1.ª redacção pôs aqui o número das entradas
/// e reprovou sobre um extractor correcto.
const PISO_DE_ROTULOS: usize = 400;
const PISO_DE_GRUPOS: usize = 30;
/// Medido: **`146`** selectores declaram uma lista de opções com a forma `….<índice>`.
const PISO_DE_SELECTORES: usize = 120;

fn ler(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", p.display()))
}

/// ⭐⭐ **Quem lê uma tabela de strings é a RÉGUA REGISTADA, nunca um leitor novo.**
///
/// ⛔ O [`ph2d_label_census::declared_pairs_in`] já conhece as **duas** formas que este repo usa
/// (`"k" => "v"` e `("k", "v")`) e as continuações de linha — e o doc dele conta o preço de ter
/// havido dois leitores: **`1 432` entradas, 23 % da tabela**, nunca conferidas, porque só um
/// deles aprendeu o tuplo. *Escrever aqui um terceiro seria a mesma dívida uma terceira vez.*
fn pares(rel: &str) -> Vec<ph2d_label_census::keys::Par> {
    ph2d_label_census::keys::declared_pairs_in(&ler(rel))
}

/// ⭐⭐ **Os rótulos de parâmetro do catálogo, do mais LARGO para o mais estreito.**
///
/// ⚠️ **A ordem é pela LARGURA PINTADA e não pelo comprimento em letras**, medida no mesmo
/// tamanho e no mesmo peso em que o painel os escreve — *ordenar por caracteres escolheria a
/// palavra errada, e a régua desta varredura é a caixa, não a contagem*.
fn rotulos_do_catalogo() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut out: Vec<String> = TABELAS
            .iter()
            .flat_map(|rel| pares(rel))
            .filter(|p| p.chave.starts_with("node.") && p.chave.contains(".param."))
            .map(|p| p.texto)
            .collect();
        out.sort();
        out.dedup();
        assert!(
            out.len() >= PISO_DE_ROTULOS,
            "o extractor leu {} rotulos de param (piso {PISO_DE_ROTULOS}) — ou a tabela mudou de \
             forma, ou o ficheiro mudou de sitio. Uma fixtura que arma rotulos VAZIOS devolve zero \
             cortes, e zero le^-se como aprovacao.",
            out.len()
        );
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let fonte = ph2d_tokens::TypeToken::Sm.px();
        out.sort_by(|a, b| {
            ts.prefix_width(b, fonte)
                .total_cmp(&ts.prefix_width(a, fonte))
        });
        out
    })
}

/// Os nomes de secção do catálogo, do mais largo para o mais estreito.
fn titulos_de_seccao() -> &'static [String] {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut out: Vec<String> = pares(TABELA_DE_GRUPOS)
            .into_iter()
            .filter(|p| p.chave.starts_with("node.group."))
            .map(|p| p.texto)
            .collect();
        out.sort();
        out.dedup();
        assert!(
            out.len() >= PISO_DE_GRUPOS,
            "o extractor leu {} nomes de seccao (piso {PISO_DE_GRUPOS})",
            out.len()
        );
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let fonte = ph2d_tokens::TypeToken::Sm.px();
        out.sort_by(|a, b| {
            ts.prefix_width(b, fonte)
                .total_cmp(&ts.prefix_width(a, fonte))
        });
        out
    })
}

/// O `i`-ésimo rótulo mais largo do catálogo.
fn rotulo(i: usize) -> String {
    rotulos_do_catalogo()[i].clone()
}

/// ⭐⭐⭐ **A LISTA de opções de UM selector REAL — a mais larga que o catálogo declara.**
///
/// ⛔⛔ **A 1.ª redacção usava rótulos de PARAM como opções, e isso é uma fabricação:** uma
/// pergunta e uma resposta vivem em tabelas diferentes porque são populações diferentes, e a
/// caixa de um chip é dimensionada pela LISTA a que ele pertence. *Medir um chip com texto de
/// outra população mede uma caixa que o produto nunca desenha.*
///
/// ⚠️ **E a escolha é uma LISTA inteira, nunca as N palavras mais largas de tabela nenhuma:** um
/// selector oferece as opções DELE, e juntar as mais largas de selectores diferentes construiria
/// um selector que não existe.
///
/// ⛔ **A cerca é a FORMA DA CHAVE** (`….<índice>`, com o último segmento todo em dígitos): a
/// mesma tabela guarda o alfabeto e as queixas do parser do `source.lsystem`
/// (`node.lsystem.rule_problem.bad_condition`, `62` caracteres), que **não são opções de
/// selector nenhum** — sem a cerca, a fixtura pintaria uma frase inteira dentro de um chip.
///
/// ⚠️ O `&'static str` vem de um derrame **único** (o `OnceLock`): o alvo da medição é o texto, e
/// o tempo de vida é um detalhe do tipo da row.
fn opcoes() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let fonte = ph2d_tokens::TypeToken::Sm.px();
        let mut listas: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for p in pares(TABELA_DE_OPCOES) {
            let Some((prefixo, indice)) = p.chave.rsplit_once('.') else {
                continue;
            };
            if indice.is_empty() || !indice.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            listas.entry(prefixo.to_string()).or_default().push(p.texto);
        }
        assert!(
            listas.len() >= PISO_DE_SELECTORES,
            "o extractor leu {} selectores (piso {PISO_DE_SELECTORES}) — a tabela mudou de forma",
            listas.len()
        );
        // ⚠️ A largura mede-se ANTES do `max_by`: o `TextSystem` é `&mut` e um comparador que o
        //    empresta duas vezes não compila.
        let mut medidas: Vec<(f32, Vec<String>)> = listas
            .into_values()
            .filter(|l| l.len() >= 2 && l.len() <= ph2d_panel_motion_params::MAX_ENUM_OPTIONS)
            .map(|l| {
                let w = l
                    .iter()
                    .map(|t| ts.prefix_width(t, fonte))
                    .fold(0.0_f32, f32::max);
                (w, l)
            })
            .collect();
        medidas.sort_by(|a, b| b.0.total_cmp(&a.0));
        let melhor = medidas
            .into_iter()
            .next()
            .expect("nenhum selector com duas ou mais opcoes")
            .1;
        melhor
            .into_iter()
            .map(|t| &*Box::leak(t.into_boxed_str()))
            .collect()
    })
}

/// ⭐ **UMA fileira de cada espécie, cada uma com um dos rótulos mais largos do catálogo.**
///
/// ⚠️ **Uma por espécie e não uma por nó:** o que esta passagem mede é a CAIXA de cada forma de
/// fileira, e o catálogo tem `828` rótulos para `13` formas. Pintar o catálogo inteiro mediria a
/// mesma caixa 64 vezes.
fn fileiras() -> Vec<ParamRow> {
    vec![
        ParamRow::Scalar(ScalarRow {
            name: "speed",
            label: rotulo(0),
            value: 12.5,
            min: 0.0,
            hard_min: 0.0,
            max: 100.0,
            hard_max: 1000.0,
            step: 0.1,
            integer: false,
            driven_by: None,
            display: RowDisplay {
                scale: 1.0,
                suffix: "px",
            },
        }),
        // ⚠️ Uma fileira **CONDUZIDA** pinta o nome de quem a conduz — outro texto, outra caixa.
        ParamRow::Scalar(ScalarRow {
            name: "amount",
            label: rotulo(1),
            value: 0.75,
            min: 0.0,
            hard_min: 0.0,
            max: 1.0,
            hard_max: 1.0,
            step: 0.01,
            integer: false,
            driven_by: Some(rotulo(2)),
            display: RowDisplay {
                scale: 1.0,
                suffix: "",
            },
        }),
        ParamRow::Color(ColorRow {
            label: rotulo(3),
            channels: ["tint_r", "tint_g", "tint_b", "tint_a"],
            srgb: [200, 120, 60, 255],
        }),
        ParamRow::Toggle(ToggleRow {
            name: "enabled",
            label: rotulo(4),
            on: true,
        }),
        ParamRow::Enum(EnumRow {
            name: "mode",
            label: rotulo(5),
            selected: 1,
            labels: opcoes(),
        }),
        ParamRow::Angle(AngleRow {
            name: "rotation",
            label: rotulo(6),
            deg: 45.0,
            min_deg: -180.0,
            max_deg: 180.0,
            step_deg: 1.0,
        }),
        ParamRow::Seed(SeedRow {
            name: "seed",
            label: rotulo(7),
            value: 1337.0,
            min: 0.0,
            max: 65535.0,
        }),
        // ⚠️ A `problem` entra porque ela é a **FRASE** que a fileira pinta quando o que o artista
        //    escreveu não compila — e uma frase é exactamente o que a lei da reticência corta pela
        //    metade. A `help` é a irmã dela no caminho feliz.
        ParamRow::Text(TextRow {
            name: "formula",
            label: rotulo(8),
            value: "sin(t * 2) + noise(p)".to_string(),
            problem: Some("Unknown function `noize` — did you mean `noise`?".to_string()),
            help: Some("Any expression over `t`, `p` and the element index.".to_string()),
        }),
        ParamRow::Curve(CurveRow {
            name: "falloff",
            label: rotulo(9),
            value: String::new(),
        }),
        ParamRow::Gradient(GradientRow {
            name: "ramp",
            label: rotulo(10),
            value: String::new(),
        }),
        ParamRow::Palette(PaletteRow {
            name: "palette",
            label: rotulo(11),
            value: String::new(),
        }),
        ParamRow::Channels(ChannelsRow {
            label: rotulo(12),
            text_param: "column",
            mode_param: "column_mode",
            channels: vec![
                ("Position", "P", 0),
                ("Velocity", "v", 1),
                ("Lifetime", "age", 2),
            ],
            selected: 1,
            custom: "my_column".to_string(),
            extra: Vec::new(),
        }),
        ParamRow::Source(SourceRow {
            label: rotulo(13),
            param: "shape",
            options: vec![
                "Background Parallax Layer".to_string(),
                "Hero (tall variant)".to_string(),
            ],
            current: "Hero (tall variant)".to_string(),
        }),
        ParamRow::File(FileRow {
            name: "clip",
            label: rotulo(14),
            value: "/home/artist/projects/footsteps-loop.ogg".to_string(),
            missing: true,
        }),
    ]
}

/// Publica o nó de fábrica impossível: uma fileira de cada espécie, em duas secções.
pub fn arma() {
    let rows = fileiras();
    let titulos = titulos_de_seccao();
    // ⚠️ **A segunda secção começa a meio** — duas secções pintam dois cabeçalhos, e um cabeçalho
    //    de secção tem uma caixa própria que nenhuma fileira mede.
    let sections = vec![
        (titulos[0].clone(), 0usize),
        (titulos[1].clone(), rows.len() / 2),
    ];
    // ⚠️ Um param **modificado** pinta a seta de repor ao lado da fileira, que come largura.
    let modified = ["speed", "mode", "formula"]
        .into_iter()
        .map(str::to_string)
        .collect();
    set_current_params(Some(ParamsSnapshot {
        node: 1,
        title: rotulo(15),
        rows,
        modified,
        sections,
        folded_by_default: std::collections::BTreeSet::new(),
    }));
}

/// ⛔ **Obrigatório:** a porta é `thread_local` e o binário de teste corre todos os módulos na
/// mesma thread. *O estado que uma fixtura deixa para trás é o estado que a régua seguinte mede.*
pub fn desarma() {
    set_current_params(None);
}

/// ⭐⭐ **A fixtura pinta UMA fileira de cada espécie — e a contagem sai do ENUM, não daqui.**
///
/// ⛔ Sem isto, uma variante nova de [`ParamRow`] nasceria **sem nunca ser medida** e esta passagem
/// continuaria verde: *uma lista escrita à mão ao lado de um enum é a lista que apodrece*.
/// ⚠️ A régua é o `include_str!` deste ficheiro contra o do painel, porque um `match` exaustivo
/// sobre `ParamRow` precisaria de um valor de cada variante — que é exactamente o que se está a
/// tentar provar que existe.
///
/// ⛔ **E ela é TEXTUAL, com o que isso implica escrito aqui:** uma mutação mostrou que escrever a
/// variante por um `use … as` a esconde do censo. *Ele não guarda contra quem o quer contornar;
/// guarda contra a variante nova que ninguém se lembrou de armar* — que é o modo de falha real, e
/// para esse ele é exacto. ⚠️ **As duas metades são obrigatórias:** sem o piso de população, um
/// extractor que deixasse de casar o `pub enum ParamRow {` leria zero espécies e `em_falta`
/// ficaria trivialmente vazio (as duas mutações sangram).
#[test]
fn a_fixtura_arma_uma_fileira_de_cada_especie() {
    const ESTE: &str = include_str!("o_motion_armado.rs");
    let fonte = ler("../ph2d-panel-motion-params/src/snapshot.rs");
    let corpo = fonte
        .split_once("pub enum ParamRow {")
        .expect("o enum ParamRow mudou de forma")
        .1
        .split_once('}')
        .expect("o enum ParamRow nao fecha")
        .0;
    let especies: Vec<&str> = corpo
        .lines()
        .map(str::trim)
        .filter_map(|l| l.split_once('(').map(|(n, _)| n))
        .filter(|n| !n.is_empty() && n.chars().next().is_some_and(char::is_uppercase))
        .collect();
    assert!(
        especies.len() >= 13,
        "o extractor leu {} especies de fileira (piso 13) — o enum mudou de forma",
        especies.len()
    );
    let em_falta: Vec<&str> = especies
        .iter()
        .copied()
        .filter(|e| !ESTE.contains(&format!("ParamRow::{e}(")))
        .collect();
    assert!(
        em_falta.is_empty(),
        "estas especies de fileira do painel de params NAO sao armadas, logo nenhuma regua de \
         largura as ve^: {em_falta:?}"
    );
}

/// ⭐ **E os rótulos que ela arma são os MAIS LARGOS do catálogo, nesta ordem.**
///
/// ⛔ A metade que interessa é a segunda: sem ela, um extractor que devolvesse a lista por ordem
/// alfabética passaria — e a fixtura mediria a caixa com uma palavra qualquer.
#[test]
fn os_rotulos_da_fixtura_sao_os_mais_largos_do_catalogo() {
    let r = rotulos_do_catalogo();
    assert!(r.len() >= PISO_DE_ROTULOS);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let fonte = ph2d_tokens::TypeToken::Sm.px();
    for par in r.windows(2) {
        assert!(
            ts.prefix_width(&par[0], fonte) >= ts.prefix_width(&par[1], fonte),
            "{:?} pinta menos do que {:?} e vem antes dele",
            par[0],
            par[1]
        );
    }
    // ⚠️ O controlo: o mais largo tem de pintar MAIS do que o mais estreito, senão a lista pode
    //    estar ordenada e vazia de variação (todos iguais) e a fixtura não mede nada.
    assert!(
        ts.prefix_width(&r[0], fonte) > ts.prefix_width(&r[r.len() - 1], fonte),
        "todos os rotulos do catalogo pintam o mesmo — a fixtura nao mede caixa nenhuma"
    );
}
