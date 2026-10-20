//! ⭐⭐⭐ **TODO CAMPO EDITÁVEL TEM CANAL** (plano 35 §5.2) — o gate que torna «o Auto Key grava
//! absolutamente tudo» (dono, 19/10) verificável: um campo novo sem canal reprova.
//!
//! Lê o FONTE, como o `architecture_panel_wiring_parity`. A população é cada chamada de uma PORTA
//! com coluna de losango ([`PORTAS`] — as linhas de `property_row`, o slider com chip, a escolha, o
//! chip numérico do rail, o próprio `paint_decorator_for`) ou de um helper de linha de um painel
//! que só a repassa ([`LINHAS`]). Cada chamada tem de cair numa classe:
//!
//! - **canal**: um dos ids dela está numa tabela de canais ([`TABELAS`]: `ph2d_app_vec::marcas`,
//!   `ph2d_app_components::marcas_inspector`);
//! - **⛔ não-animável** ou **dívida**: um id (ou prefixo) dela está em [`POR_ID`], a função dela
//!   em [`POR_FUNCAO`] (o id vem numa variável), ou o ficheiro em [`DIVIDA_FICHEIRO`].
//!
//! Fora do [`ESCOPO`], cada crate que pinta campos está em [`DIVIDA_CRATE`] (o bloco 2 do plano 35
//! §10 zera-as) — uma crate de painel NOVA que pinte campos reprova até escolher o seu lado.
//!
//! ⛔ Toda tabela tem censo de obsolescência: uma entrada que já não casa nada reprova (uma isenção
//! que sobrevive ao código que a explicava é licença).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Classe {
    /// O canal existe por um caminho que o gate não lê como literal (o id derivado de um índice
    /// cuja inversa mora na tabela).
    Canal,
    /// ⛔ Não é propriedade da cena (plano 35 §8): estado da ferramenta ou da vista, estrutura, ou
    /// o parâmetro da própria animação.
    NaoAnimavel,
    /// Propriedade sem canal ainda — a zerar (plano 35 §10, bloco 2).
    Divida,
}
use Classe::{Canal, Divida, NaoAnimavel};

/// As portas com a coluna do losango (o nome da função chamada).
const PORTAS: &[&str] = &[
    "paint_fields_row",
    "paint_field_row",
    "paint_field_row_value",
    "paint_text_row",
    "paint_color_row",
    "paint_slider_with_chip",
    "paint_slider_with_chip_layout_adaptive",
    "paint_choice_row",
    "paint_decorator_for",
    // `ToolRailEntry::number` — o chip numérico do rail (o losango à direita dele).
    "number",
];

/// As crates cujos campos este gate classifica um a um; `None` = todos os ficheiros de `src/`,
/// `Some` = só estes (o resto da crate é dívida do bloco 2).
const ESCOPO: &[(&str, Option<&[&str]>)] = &[
    ("ph2d-panel-vector", None),
    (
        "ph2d-panel-inspector",
        Some(&[
            "sections/transform.rs",
            "sections/color_tint.rs",
            "sections/emissive_row.rs",
            "sections/joint_cards.rs",
            "sections/joint_kind_rows.rs",
        ]),
    ),
];

/// As tabelas de canais (caminhos a partir de `crates/`).
const TABELAS: &[&str] = &[
    "ph2d-app-vec/src/marcas.rs",
    "ph2d-app-components/src/marcas_inspector.rs",
];

/// Os helpers de LINHA de cada painel: recebem o id do chamador e repassam-no a uma porta — a
/// chamada deles com o id numa variável não é população, a dos chamadores deles é.
const LINHAS: &[(&str, &str)] = &[
    ("ph2d-panel-vector", "slider_row"),
    ("ph2d-panel-vector", "segmented"),
    ("ph2d-panel-vector", "segmented3"),
    ("ph2d-panel-vector", "number_row"),
    ("ph2d-panel-vector", "lone_number_row"),
    ("ph2d-panel-vector", "lone_number_row_full"),
    ("ph2d-panel-vector", "number_cell"),
    ("ph2d-panel-vector", "fields_row"),
    ("ph2d-panel-vector", "colour_swatch_row"),
    ("ph2d-panel-vector", "colour_swatch_row_rect"),
    ("ph2d-panel-inspector", "fields_row"),
    ("ph2d-panel-inspector", "num_row"),
    ("ph2d-panel-inspector", "num_row_unit"),
    ("ph2d-panel-inspector", "seg_row"),
];

/// A classe de uma chamada por um dos seus ids — exacto, ou PREFIXO quando acaba em `_`.
const POR_ID: &[(&str, Classe, &str)] = &[
    (
        "VECTOR_PENCIL_",
        NaoAnimavel,
        "ajuste da ferramenta Lápis (§8)",
    ),
    ("VECTOR_SYM_", NaoAnimavel, "simetria da ferramenta (§8)"),
    (
        "VECTOR_SNAP_",
        NaoAnimavel,
        "o snap é da vista/ferramenta (§8)",
    ),
    (
        "VECTOR_BOOL_",
        NaoAnimavel,
        "a booleana é estrutura; o RESULTADO grava-se pelo canal Forma (§8)",
    ),
    (
        "VECTOR_STATE_",
        NaoAnimavel,
        "a transição de um estado da Interface É uma animação; não se anima a si própria",
    ),
    ("VECTOR_ANCHOR_", Divida, "Interface: as âncoras de layout"),
    (
        "VECTOR_FRAME_",
        Divida,
        "Interface: a moldura (clip, painel)",
    ),
    (
        "VECTOR_BRUSH_",
        Divida,
        "os parâmetros do Brush stroke (F24a §6.4)",
    ),
    (
        "VECTOR_BLEND_STEPS",
        Divida,
        "o Blend vivo: componente sem vocabulário",
    ),
    (
        "VECTOR_MORPH_T",
        Divida,
        "o Morph vivo: componente sem vocabulário",
    ),
    (
        "VECTOR_ENVELOPE_",
        Divida,
        "o Envelope vivo: componente sem vocabulário",
    ),
    (
        "VECTOR_TEXT_WRAP_AUTO",
        Divida,
        "o MODO da quebra (a largura existir ou não) não tem campo; a largura tem",
    ),
    ("INSP_TRANSFORM_SKEW_", Divida, "o skew não tem PropKind"),
    ("INSP_SPRITE_EMISSIVE", Divida, "a emissão do sprite (§A.2)"),
    (
        "INSP_JOINT_MOTOR_GROUP",
        Divida,
        "o eixo do motor (Enum, §A.5)",
    ),
    (
        "INSP_JOINT_MOTOR_MODE_GROUP",
        Divida,
        "o modo do motor (Enum, §A.5)",
    ),
    ("INSP_JOINT_MOTOR_FORCE", Divida, "a força do motor (§A.5)"),
    ("INSP_JOINT_BREAK_", Divida, "a quebra da junta (§A.5)"),
    ("INSP_JOINT_LIMITS_", Divida, "os limites da junta (§A.5)"),
    ("INSP_JOINT_SOFT_", Divida, "a mola da junta (§A.5)"),
];

/// As chamadas cujo id vem numa VARIÁVEL, pela função que as faz. Para [`Canal`], os nomes que a
/// variável pode ter estão no ficheiro da função E numa tabela.
const POR_FUNCAO: &[(&str, &str, Classe, &[&str], &str)] = &[
    (
        "ph2d-panel-vector",
        "dash_rows",
        Canal,
        &["VECTOR_DASH_IDS", "VECTOR_GAP_IDS"],
        "os pares do tracejado, das tabelas de ids",
    ),
    (
        "ph2d-panel-vector",
        "layer_row",
        Canal,
        &["vector_paint_swatch_id"],
        "a tinta da entrada `i` da pilha (`marcas::canal_do_campo`)",
    ),
    (
        "ph2d-panel-vector",
        "shape_field_chip",
        Divida,
        &[],
        "os campos da forma paramétrica na fila ②: o canal PARAM_VALOR existe, falta a inversa de \
         `vector_shape_field_id`",
    ),
    (
        "ph2d-panel-vector",
        "brush_section",
        Divida,
        &[],
        "os parâmetros do Brush stroke (F24a §6.4)",
    ),
    (
        "ph2d-panel-vector",
        "fx_param",
        Divida,
        &[],
        "os parâmetros dos efeitos: o canal EFEITO_PARAM existe, falta a inversa de \
         `vector_fx_param_id`",
    ),
    (
        "ph2d-panel-vector",
        "power_stroke_rows",
        Divida,
        &[],
        "o perfil de largura (Power Stroke): sem vocabulário",
    ),
    (
        "ph2d-panel-vector",
        "width_preset_row",
        Divida,
        &[],
        "o perfil de largura (Power Stroke): sem vocabulário",
    ),
    (
        "ph2d-panel-vector",
        "signal_rows",
        NaoAnimavel,
        &[],
        "a ligação de um sinal é fiação, não propriedade",
    ),
    (
        "ph2d-panel-vector",
        "easing_rows",
        NaoAnimavel,
        &[],
        "a curva da transição de um estado É a animação",
    ),
    (
        "ph2d-panel-inspector",
        "paint_motor_rows",
        Canal,
        &["INSP_JOINT_MOTOR_TARGET", "INSP_JOINT_MOTOR_SPEED"],
        "o alvo OU a velocidade do motor, pelo modo",
    ),
    (
        "ph2d-panel-inspector",
        "paint_kind_params",
        Divida,
        &[],
        "misto: comprimento de repouso e máximo têm canal; molas e limites não (§A.5)",
    ),
    (
        "ph2d-panel-inspector",
        "paint_tint_swatch_cell",
        Divida,
        &[],
        "as cores do tint e dos cantos (§A.2)",
    ),
];

/// Ficheiros inteiros em dívida — famílias sem vocabulário ainda.
const DIVIDA_FICHEIRO: &[(&str, &str, &str)] = &[
    (
        "ph2d-panel-vector",
        "paint_filters.rs",
        "os filtros da forma: sem vocabulário",
    ),
    (
        "ph2d-panel-vector",
        "paint_layout.rs",
        "Interface: o auto layout",
    ),
    (
        "ph2d-panel-vector",
        "paint_patternpath.rs",
        "o padrão ao longo do caminho: sem vocabulário",
    ),
    (
        "ph2d-panel-vector",
        "paint_textpath.rs",
        "o texto no caminho: sem vocabulário",
    ),
    (
        "ph2d-panel-vector",
        "paint_texture_pattern.rs",
        "o padrão de textura: o canal OFF_PADRAO existe, falta a inversa dos ids por `TexPatKnob`",
    ),
    (
        "ph2d-panel-vector",
        "paint_widget.rs",
        "Interface: a pele do widget",
    ),
];

/// As crates que pintam campos fora do [`ESCOPO`] (plano 35 §10, bloco 2).
const DIVIDA_CRATE: &[(&str, Classe, &str)] = &[
    ("ph2d-panel-audio-editor", Divida, "Áudio"),
    ("ph2d-panel-audio-mixer", Divida, "Áudio"),
    ("ph2d-panel-flip", Divida, "a camada do Flip (§A.8)"),
    (
        "ph2d-panel-painter-layers",
        Divida,
        "as camadas e ajustes do Painter (§A.7)",
    ),
    ("ph2d-panel-physics", Divida, "a Física (§A.5)"),
    (
        "ph2d-panel-bgremoval",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-color-equalization",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-equalize-sizes",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-padding",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-upscale",
        NaoAnimavel,
        "parâmetro de uma ferramenta de imagem",
    ),
    (
        "ph2d-panel-grid-snap",
        NaoAnimavel,
        "a grelha é da vista (§8)",
    ),
    (
        "ph2d-panel-wet-tuning",
        NaoAnimavel,
        "a afinação do motor de pintura é da ferramenta",
    ),
];

// ── leitura do fonte ────────────────────────────────────────────────────────────────────────

fn crates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

/// Os `.rs` de `src/` (sem os ficheiros de teste), como (caminho relativo a `src/`, fonte).
fn fontes(krate: &str) -> Vec<(String, String)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
                continue;
            }
            let nome = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !nome.ends_with(".rs") || nome.ends_with("_tests.rs") || nome == "tests.rs" {
                continue;
            }
            if let Ok(s) = std::fs::read_to_string(&p) {
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .into_owned();
                out.push((rel, sem_testes(&s).to_owned()));
            }
        }
    }
    let base = crates_root().join(krate).join("src");
    let mut out = Vec::new();
    walk(&base, &base, &mut out);
    out.sort();
    out
}

fn sem_testes(s: &str) -> &str {
    s.find("#[cfg(test)]\nmod tests {").map_or(s, |i| &s[..i])
}

fn ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// As funções de `s`: (nome, corpo) — do `fn` até ao `fn` seguinte.
fn funcoes(s: &str) -> Vec<(String, &str)> {
    let mut inicios = Vec::new();
    let mut i = 0;
    while let Some(p) = s[i..].find("fn ") {
        let at = i + p;
        let antes = s[..at].chars().next_back();
        if antes.is_none_or(|c| !ident(c)) {
            let nome: String = s[at + 3..].chars().take_while(|&c| ident(c)).collect();
            if !nome.is_empty() {
                inicios.push((at, nome));
            }
        }
        i = at + 3;
    }
    let mut out = Vec::new();
    for (k, (at, nome)) in inicios.iter().enumerate() {
        let fim = inicios.get(k + 1).map_or(s.len(), |(f, _)| *f);
        out.push((nome.clone(), &s[*at..fim]));
    }
    out
}

/// As chamadas a `nomes` em `corpo`: (nome chamado, texto dos argumentos).
fn chamadas<'a>(corpo: &'a str, nomes: &BTreeSet<&str>) -> Vec<(String, &'a str)> {
    let b = corpo.as_bytes();
    let mut out = Vec::new();
    for (i, _) in corpo.match_indices('(') {
        let ini = corpo[..i]
            .char_indices()
            .rev()
            .take_while(|&(_, c)| ident(c))
            .last()
            .map_or(i, |(j, _)| j);
        let nome = &corpo[ini..i];
        if !nomes.contains(nome) || corpo[..ini].ends_with("fn ") {
            continue;
        }
        if nome == "number" && !corpo[..ini].ends_with("ToolRailEntry::") {
            continue;
        }
        let (mut d, mut j) = (1, i + 1);
        while j < b.len() && d > 0 {
            match b[j] {
                b'(' => d += 1,
                b')' => d -= 1,
                _ => {}
            }
            j += 1;
        }
        out.push((nome.to_owned(), &corpo[i + 1..j.saturating_sub(1)]));
    }
    out
}

/// Os ids `ids::NOME` (maiúsculas) de um texto — `X[0]` conta como `X`.
fn ids_em(args: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in args.match_indices("ids::") {
        let nome: String = args[i + 5..].chars().take_while(|&c| ident(c)).collect();
        if nome.len() > 2 && nome.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            out.push(nome);
        }
    }
    out
}

/// Os nomes que as tabelas de canais citam (`painel::X`, `ferramenta::X`, `insp::X`, e as portas
/// de ids derivados `painel::f(`).
fn nomes_das_tabelas() -> (BTreeSet<String>, String) {
    let mut nomes = BTreeSet::new();
    let mut texto = String::new();
    for t in TABELAS {
        let s = std::fs::read_to_string(crates_root().join(t)).expect("tabela de canais");
        let s = sem_testes(&s).to_owned();
        for pref in ["painel::", "ferramenta::", "insp::"] {
            for (i, _) in s.match_indices(pref) {
                let n: String = s[i + pref.len()..]
                    .chars()
                    .take_while(|&c| ident(c))
                    .collect();
                if !n.is_empty() {
                    nomes.insert(n);
                }
            }
        }
        texto.push_str(&s);
    }
    (nomes, texto)
}

fn por_id(ids: &[String]) -> Option<&'static (&'static str, Classe, &'static str)> {
    POR_ID.iter().find(|(chave, _, _)| {
        ids.iter().any(|id| {
            if chave.ends_with('_') {
                id.starts_with(chave)
            } else {
                id == chave
            }
        })
    })
}

/// Uma chamada da população: onde está e o que leva.
struct Chamada {
    krate: &'static str,
    ficheiro: String,
    funcao: String,
    porta: String,
    ids: Vec<String>,
}

fn populacao() -> Vec<Chamada> {
    let mut out = Vec::new();
    for &(krate, so) in ESCOPO {
        let mut nomes: BTreeSet<&str> = PORTAS.iter().copied().collect();
        nomes.extend(LINHAS.iter().filter(|(k, _)| *k == krate).map(|(_, f)| *f));
        for (ficheiro, s) in fontes(krate) {
            if so.is_some_and(|fs| !fs.contains(&ficheiro.as_str())) {
                continue;
            }
            for (funcao, corpo) in funcoes(&s) {
                for (porta, args) in chamadas(corpo, &nomes) {
                    out.push(Chamada {
                        krate,
                        ficheiro: ficheiro.clone(),
                        funcao: funcao.clone(),
                        porta,
                        ids: ids_em(args),
                    });
                }
            }
        }
    }
    out
}

fn e_linha(krate: &str, funcao: &str) -> bool {
    LINHAS.iter().any(|&(k, f)| k == krate && f == funcao)
}

fn na_divida(krate: &str, ficheiro: &str) -> bool {
    DIVIDA_FICHEIRO
        .iter()
        .any(|&(k, f, _)| k == krate && f == ficheiro)
}

// ── os gates ─────────────────────────────────────────────────────────────────────────────────

/// ⭐ Toda chamada da população tem classe: canal, ⛔ não-animável, ou dívida nomeada.
#[test]
fn todo_campo_pintado_com_losango_tem_canal_ou_porque() {
    let (canais, _) = nomes_das_tabelas();
    let mut sem = Vec::new();
    for c in populacao() {
        if na_divida(c.krate, &c.ficheiro) {
            continue;
        }
        let ok = if c.ids.is_empty() {
            e_linha(c.krate, &c.funcao)
                || POR_FUNCAO
                    .iter()
                    .any(|&(k, f, ..)| k == c.krate && f == c.funcao)
        } else {
            c.ids.iter().any(|id| canais.contains(id)) || por_id(&c.ids).is_some()
        };
        if !ok {
            sem.push(format!(
                "{}/src/{} · fn {} → {}({})",
                c.krate,
                c.ficheiro,
                c.funcao,
                c.porta,
                if c.ids.is_empty() {
                    "<id numa variável>".to_owned()
                } else {
                    c.ids.join(", ")
                }
            ));
        }
    }
    assert!(
        sem.is_empty(),
        "campos com coluna de losango SEM canal e sem porquê ({}). Ponha o id numa tabela de \
         canais ({TABELAS:?}) ou classifique-o em POR_ID / POR_FUNCAO (plano 35 §5.2, §8):\n  {}",
        sem.len(),
        sem.join("\n  ")
    );
}

/// ⛔ Censo: cada entrada de cada tabela de isenção ainda casa alguma coisa, e um id que ganhou
/// canal sai da isenção.
#[test]
fn nenhuma_isencao_sobrevive_ao_codigo_que_a_explicava() {
    let pop = populacao();
    let (canais, texto_tabelas) = nomes_das_tabelas();
    let mut velhas = Vec::new();
    for &(chave, _, porque) in POR_ID {
        let casa = pop.iter().any(|c| {
            !na_divida(c.krate, &c.ficheiro)
                && !c.ids.iter().any(|id| canais.contains(id))
                && por_id(&c.ids).is_some_and(|e| e.0 == chave)
        });
        if !casa {
            velhas.push(format!("POR_ID {chave} ({porque})"));
        }
    }
    for &(krate, funcao, classe, nomes, porque) in POR_FUNCAO {
        let casa = pop
            .iter()
            .any(|c| c.krate == krate && c.funcao == funcao && c.ids.is_empty());
        if !casa {
            velhas.push(format!("POR_FUNCAO {krate}::{funcao} ({porque})"));
        }
        if classe == Canal {
            let ficheiro = pop
                .iter()
                .find(|c| c.krate == krate && c.funcao == funcao)
                .map(|c| c.ficheiro.clone())
                .unwrap_or_default();
            let fonte = fontes(krate)
                .into_iter()
                .find(|(f, _)| *f == ficheiro)
                .map(|(_, s)| s)
                .unwrap_or_default();
            assert!(!nomes.is_empty(), "{krate}::{funcao} é Canal sem nomes");
            for n in nomes {
                if !fonte.contains(n) || !texto_tabelas.contains(n) {
                    velhas.push(format!(
                        "POR_FUNCAO {krate}::{funcao}: `{n}` não está no ficheiro E numa tabela"
                    ));
                }
            }
        }
    }
    for &(krate, ficheiro, porque) in DIVIDA_FICHEIRO {
        if !pop
            .iter()
            .any(|c| c.krate == krate && c.ficheiro == ficheiro)
        {
            velhas.push(format!("DIVIDA_FICHEIRO {krate}/{ficheiro} ({porque})"));
        }
    }
    for &(krate, linha) in LINHAS {
        let fonte: String = fontes(krate).into_iter().map(|(_, s)| s).collect();
        let portas: BTreeSet<&str> = PORTAS
            .iter()
            .copied()
            .chain(LINHAS.iter().filter(|(k, _)| *k == krate).map(|(_, f)| *f))
            .collect();
        // Uma linha é uma função que chama uma porta, ou o nome curto de uma (`use … as linha`).
        let chama = funcoes(&fonte)
            .iter()
            .any(|(n, corpo)| n == linha && !chamadas(corpo, &portas).is_empty())
            || PORTAS
                .iter()
                .any(|p| fonte.contains(&format!("{p} as {linha};")));
        if !chama {
            velhas.push(format!("LINHAS {krate}::{linha} não chama uma porta"));
        }
    }
    for c in &pop {
        for id in &c.ids {
            if canais.contains(id) && por_id(std::slice::from_ref(id)).is_some() {
                velhas.push(format!("POR_ID isenta `{id}`, que já tem canal"));
            }
        }
    }
    velhas.sort();
    velhas.dedup();
    assert!(
        velhas.is_empty(),
        "isenções sem objecto (apague-as ou corrija-as):\n  {}",
        velhas.join("\n  ")
    );
}

/// Toda crate que pinta campos com losango está no [`ESCOPO`] ou em [`DIVIDA_CRATE`] — e cada
/// dívida de crate ainda pinta algum.
#[test]
fn toda_crate_que_pinta_campos_escolheu_o_seu_lado() {
    let portas: BTreeSet<&str> = PORTAS.iter().copied().collect();
    let pinta = |krate: &str| {
        fontes(krate)
            .iter()
            .any(|(_, s)| !chamadas(s, &portas).is_empty())
    };
    let mut erros = Vec::new();
    let Ok(dir) = std::fs::read_dir(crates_root()) else {
        panic!("crates/ ilegível");
    };
    let mut todas: Vec<String> = dir
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|n| n.starts_with("ph2d-panel-"))
        .collect();
    todas.sort();
    for k in &todas {
        let dentro =
            ESCOPO.iter().any(|(e, _)| e == k) || DIVIDA_CRATE.iter().any(|(d, ..)| d == k);
        if pinta(k) && !dentro {
            erros.push(format!(
                "{k} pinta campos com losango e não está no ESCOPO nem em DIVIDA_CRATE"
            ));
        }
    }
    for &(k, _, porque) in DIVIDA_CRATE {
        if !pinta(k) {
            erros.push(format!("DIVIDA_CRATE {k} ({porque}) já não pinta campos"));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join("\n"));
}

/// Controlo do próprio leitor: a varredura acha a população conhecida (sem isto, um leitor que
/// casasse ZERO aprovaria tudo).
#[test]
fn a_varredura_acha_os_campos_que_se_conhecem() {
    let pop = populacao();
    let tem = |id: &str| pop.iter().any(|c| c.ids.iter().any(|x| x == id));
    for id in [
        "VECTOR_WIDTH",
        "VECTOR_FILL_SWATCH",
        "VECTOR_CONTOUR_STEPS",
        "VECTOR_PAINT_OPACITY",
        "VECTOR_OBJ_OPACITY_NUM",
        "INSP_TRANSFORM_POS_X",
        "INSP_SPRITE_OPACITY",
    ] {
        assert!(
            tem(id),
            "a varredura não achou `{id}` — o leitor do fonte partiu"
        );
    }
    assert!(pop.len() > 100, "população suspeita: {}", pop.len());
}
