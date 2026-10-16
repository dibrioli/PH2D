//! AS FRASES de rota — achadas nas fontes da crate e nos tutoriais, e a classe que cada uma escolhe.
//! Ver o cabeçalho de `motion_state_demo_rota_anunciada_tests.rs`.

use super::medida::Cena;
use std::path::Path;

/// está aqui reprova ([`toda_frase_de_rota_tem_classe_e_cena`]).
pub(super) const ANUNCIOS: &[(&str, &str, &str)] = &[
    ("motion_state_demo_announce.rs", "lazy_switch", "107"),
    ("motion_state_demo_announce.rs", "fio", "116"),
    ("motion_state_demo_announce.rs", "sim", "113"),
    ("motion_state_demo_announce.rs", "foco", "112"),
    ("motion_state_demo_announce.rs", "pivot", "111"),
    ("motion_state_demo_conferencia.rs", "space", "51"),
    (
        "motion_state_demo_conferencia_animadores.rs",
        "column",
        "56",
    ),
    (
        "motion_state_demo_conferencia_utilidade.rs",
        "color_family",
        "72",
    ),
    ("motion_state_colisor_nome_demo.rs", "announce", "137"),
    ("motion_state_acabamento_demo.rs", "announce", "135"),
    ("motion_state_imagens_grupo_demo.rs", "announce", "140"),
    ("motion_state_imagens_scene_demo.rs", "announce", "142"),
];

/// A cena que o próprio literal nomeia: `[cena N]` ou `[shape smoke =N]` — antes da tabela.
pub(super) fn cena_do_marcador(lit: &str) -> Option<Cena> {
    let numero = |de: &str| -> Option<String> {
        let n: String = de.chars().take_while(char::is_ascii_digit).collect();
        (!n.is_empty()).then_some(n)
    };
    if let Some((_, r)) = lit.split_once("[shape smoke =") {
        return numero(r).map(|n| Cena::Forma(n.parse().expect("n")));
    }
    lit.split_once("[cena ")
        .and_then(|(_, r)| numero(r))
        .map(Cena::Demo)
}

pub(super) fn cena_da_tabela(s: &str) -> Cena {
    s.strip_prefix("forma ").map_or_else(
        || Cena::Demo(s.to_string()),
        |n| Cena::Forma(n.parse().expect("n")),
    )
}

// ───────────────────────────── AS FRASES ─────────────────────────────

/// Onde uma frase põe uma cena ou um cartão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Lugar {
    /// `corre(m) no processador` — com cartões, eles; sem, a cena inteira.
    Processador,
    /// `corre(m) / continua(m) na placa`.
    Placa,
    /// `corre(m) no dispositivo` — a palavra antiga; sem cartões quer dizer TUDO.
    Dispositivo,
    /// `corre toda no processador` — a cena inteira, por causa do cartão.
    Toda,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Afirma {
    /// Cita a linha do terminal.
    Terminal(Citacao),
    /// Cita o `[formas] … pela PLACA`.
    FormasNaPlaca,
    Cartoes(Lugar, Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Citacao {
    Inteira,
    Hibrida,
    /// `[motion-route] device: …` sem dizer qual.
    Placa,
    Processador,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Classe {
    pub(super) afirma: Vec<Afirma>,
    /// A frase é um *«DEU ERRADO se o terminal disser…»* — afirma o CONTRÁRIO.
    pub(super) nega: bool,
    /// A forma que já mentiu e que nenhuma classe exprime.
    pub(super) crua: Option<&'static str>,
    /// A variável de ambiente que a frase manda pôr à frente do comando.
    pub(super) variante: Option<String>,
}

pub(super) const LUGARES: &[(&str, Lugar)] = &[
    ("correm no processador", Lugar::Processador),
    ("corre no processador", Lugar::Processador),
    ("correm na placa", Lugar::Placa),
    ("corre na placa", Lugar::Placa),
    ("continuam na placa", Lugar::Placa),
    ("continua na placa", Lugar::Placa),
    ("correm no dispositivo", Lugar::Dispositivo),
    ("corre no dispositivo", Lugar::Dispositivo),
    ("corre toda no processador", Lugar::Toda),
];

/// As formas que afirmaram uma rota sem a medir e que nenhuma classe exprime.
pub(super) const CRUAS: &[&str] = &[
    "tudo isto corre",
    "tudo volta para o processador",
    "ou nenhum vai",
    "derruba a cadeia",
    "sai da placa",
    "sai do processador",
    "recusa a gpu",
    "recusa o device",
    "coza na cpu",
    "coze na cpu",
    "so' corre na cpu",
    "só corre na cpu",
];

/// Os nomes entre crases, com a posição — só os que podem ser cartões (não uma linha do terminal
/// nem uma variável de ambiente).
pub(super) fn cartoes_em(frase: &str) -> Vec<(usize, String)> {
    crases(frase)
        .into_iter()
        .filter_map(|(a, b)| {
            let nome = frase[a + 1..b].trim();
            let terminal = nome.starts_with('[')
                || nome.contains('=')
                || nome.starts_with("PH2D_")
                || nome.starts_with("CPU:")
                || nome.starts_with("device:")
                || nome.contains("pela PLACA");
            (!terminal && !nome.is_empty()).then(|| (a, nome.to_string()))
        })
        .collect()
}

/// Os pares de crases `(abre, fecha)` em bytes. ⚠️ Uma crase colada a uma LETRA não abre nada: é o
/// acento grave do terminal sem acentos (*«a\` margem»*).
pub(super) fn crases(texto: &str) -> Vec<(usize, usize)> {
    let mut fora = Vec::new();
    let mut aberta: Option<usize> = None;
    let mut antes: Option<char> = None;
    for (i, c) in texto.char_indices() {
        if c == '`' {
            match aberta {
                Some(a) => {
                    fora.push((a, i));
                    aberta = None;
                }
                None if !antes.is_some_and(char::is_alphanumeric) => aberta = Some(i),
                None => {}
            }
        }
        antes = Some(c);
    }
    fora
}

/// ⭐ **A classe que a frase escolhe.**
pub(super) fn classifica(frase: &str) -> Classe {
    let baixa = frase.to_lowercase();
    let mut afirma = Vec::new();
    if frase.contains("[motion-route] device: HIBRIDO") {
        afirma.push(Afirma::Terminal(Citacao::Hibrida));
    } else if frase.contains("[motion-route] device: o plano inteiro") {
        afirma.push(Afirma::Terminal(Citacao::Inteira));
    } else if frase.contains("[motion-route] device:") {
        afirma.push(Afirma::Terminal(Citacao::Placa));
    }
    if frase.contains("[motion-route] CPU") || frase.contains("`CPU:") {
        afirma.push(Afirma::Terminal(Citacao::Processador));
    }
    if frase.contains("`pela PLACA")
        || frase.contains("«pela PLACA»")
        || (frase.contains("`[formas]") && frase.contains("pela PLACA"))
    {
        afirma.push(Afirma::FormasNaPlaca);
    }
    // Os lugares pela ordem; cada um leva os cartões escritos desde o anterior (o sujeito vem antes
    // do verbo: *«o `Shape` e o `Grid` correm no processador; o `Twist` corre na placa»*).
    let mut achados: Vec<(usize, Lugar)> = Vec::new();
    for (forma, lugar) in LUGARES {
        let mut de = 0;
        while let Some(i) = baixa[de..].find(forma) {
            achados.push((de + i, *lugar));
            de += i + forma.len();
        }
    }
    achados.sort_by_key(|a| a.0);
    let cartoes = cartoes_em(frase);
    let mut desde = 0;
    for (onde, lugar) in achados {
        let deles: Vec<String> = cartoes
            .iter()
            .filter(|(p, _)| *p >= desde && *p < onde)
            .map(|(_, n)| n.clone())
            .collect();
        afirma.push(Afirma::Cartoes(lugar, deles));
        desde = onde;
    }
    let variante = frase.find("`PH2D_").and_then(|i| {
        frase[i + 1..]
            .split('`')
            .next()
            .filter(|v| v.contains('=') && !v.starts_with("PH2D_MOTION_ROUTE_LOG"))
            .map(str::to_string)
    });
    Classe {
        afirma,
        nega: frase.contains("DEU ERRADO") || frase.contains(" disser"),
        crua: CRUAS.iter().copied().find(|c| baixa.contains(c)),
        variante,
    }
}

/// O texto partido em frases — num ponto final, de exclamação ou de pergunta seguido de espaço, e
/// numa linha em branco; nunca dentro de crases.
pub(super) fn frases(texto: &str) -> Vec<String> {
    let mut fora = Vec::new();
    let mut atual = String::new();
    let pares = crases(texto);
    let mut it = texto.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        atual.push(c);
        let crase = pares.iter().any(|(a, b)| i > *a && i < *b);
        let proximo = it.peek().map(|p| p.1);
        let fim = !crase
            && ((matches!(c, '.' | '!' | '?') && proximo.is_none_or(char::is_whitespace))
                || (c == '\n' && proximo == Some('\n')));
        if fim {
            fora.push(std::mem::take(&mut atual));
        }
    }
    fora.push(atual);
    fora.into_iter()
        .map(|f| f.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|f| !f.is_empty())
        .collect()
}

/// Uma frase de rota achada, e de onde.
#[derive(Debug, Clone)]
pub(super) struct Achada {
    pub(super) onde: String,
    pub(super) frase: String,
    pub(super) classe: Classe,
    /// `None` = a LEI (o cartão em todas as cenas).
    pub(super) cena: Option<Cena>,
}

pub(super) fn tem_rota(c: &Classe) -> bool {
    !c.afirma.is_empty() || c.crua.is_some()
}

/// Os literais de texto de um ficheiro Rust, com a função que os contém (comentários fora).
pub(super) fn literais(fonte: &str) -> Vec<(String, String)> {
    let b = fonte.as_bytes();
    let mut fora = Vec::new();
    let mut i = 0;
    let mut funcao = String::new();
    while i < b.len() {
        if fonte[i..].starts_with("//") {
            i += fonte[i..].find('\n').unwrap_or(b.len() - i);
            continue;
        }
        if fonte[i..].starts_with("fn ") && (i == 0 || !b[i - 1].is_ascii_alphanumeric()) {
            let nome: String = fonte[i + 3..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            funcao = nome;
            i += 3;
            continue;
        }
        if b[i] == b'r'
            && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_'))
            && matches!(b.get(i + 1), Some(b'"' | b'#'))
        {
            let hashes = fonte[i + 1..].chars().take_while(|c| *c == '#').count();
            if b.get(i + 1 + hashes) == Some(&b'"') {
                let corpo = i + 2 + hashes;
                let fecha = format!("\"{}", "#".repeat(hashes));
                let fim = fonte[corpo..].find(&fecha).map_or(b.len(), |k| corpo + k);
                fora.push((funcao.clone(), fonte[corpo..fim].to_string()));
                i = fim + fecha.len();
                continue;
            }
        }
        if b[i] == b'\'' {
            // Um char literal (`'"'`) ou um tempo de vida.
            if fonte[i..].starts_with("'\\") {
                i += fonte[i + 2..].find('\'').map_or(2, |k| k + 3);
            } else if b.get(i + 2) == Some(&b'\'') {
                i += 3;
            } else {
                i += 1;
            }
            continue;
        }
        if b[i] == b'"' {
            let mut j = i + 1;
            let mut s = String::new();
            while j < b.len() && b[j] != b'"' {
                if b[j] == b'\\' {
                    match b.get(j + 1) {
                        Some(b'\n') => {
                            j += 2;
                            while j < b.len() && b[j].is_ascii_whitespace() {
                                j += 1;
                            }
                            continue;
                        }
                        Some(b'n') => s.push('\n'),
                        Some(b't') => s.push('\t'),
                        Some(&o) => s.push(o as char),
                        None => {}
                    }
                    j += 2;
                    continue;
                }
                let c = fonte[j..].chars().next().expect("char");
                s.push(c);
                j += c.len_utf8();
            }
            fora.push((funcao.clone(), s));
            i = j + 1;
            continue;
        }
        i += fonte[i..].chars().next().map_or(1, char::len_utf8);
    }
    fora
}

pub(super) fn src() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

pub(super) fn tutoriais() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/Motion Nodes/tutoriais/src")
}

/// ⭐ **As frases de rota dos ANÚNCIOS** — toda fonte não-teste da crate, todo literal; e as que não
/// têm cena na [`ANUNCIOS`].
pub(super) fn frases_dos_anuncios() -> (Vec<Achada>, Vec<String>) {
    let mut achadas = Vec::new();
    let mut sem_cena = Vec::new();
    let mut nomes: Vec<_> = std::fs::read_dir(src())
        .expect("src")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| {
            n.ends_with(".rs")
                && !n.ends_with("_tests.rs")
                && !n.ends_with("_probe.rs")
                && !n.contains("census")
        })
        .collect();
    nomes.sort();
    for nome in nomes {
        let fonte = std::fs::read_to_string(src().join(&nome)).expect("fonte");
        for (funcao, lit) in literais(&fonte) {
            for frase in frases(&lit) {
                let classe = classifica(&frase);
                if !tem_rota(&classe) {
                    continue;
                }
                let cena = cena_do_marcador(&lit).or_else(|| {
                    ANUNCIOS
                        .iter()
                        .find(|(f, g, _)| *f == nome && *g == funcao)
                        .map(|(_, _, c)| cena_da_tabela(c))
                });
                if cena.is_none() {
                    sem_cena.push(format!("{nome}::{funcao}: «{frase}»"));
                    continue;
                }
                achadas.push(Achada {
                    onde: format!("{nome}::{funcao}"),
                    frase,
                    classe,
                    cena,
                });
            }
        }
    }
    (achadas, sem_cena)
}

/// O texto que o PDF mostra: sem `<style>`, com `<code>X</code>` como `` `X` ``, sem etiquetas.
pub(super) fn texto_do_html(html: &str) -> String {
    let mut s = html.to_string();
    for (abre, fecha) in [("<style", "</style>"), ("<script", "</script>")] {
        while let Some(i) = s.find(abre) {
            let j = s[i..].find(fecha).map_or(s.len(), |k| i + k + fecha.len());
            s.replace_range(i..j, " ");
        }
    }
    let s = s.replace("<code>", "`").replace("</code>", "`");
    // Uma etiqueta de BLOCO (célula, linha, parágrafo, item) acaba a frase: uma tabela não tem
    // pontos finais, e sem isto ela seria uma frase só.
    const BLOCOS: &[&str] = &[
        "</td", "</th", "</tr", "</p", "</li", "</div", "</h1", "</h2", "</h3", "<br", "</table",
    ];
    let mut fora = String::new();
    let mut etiqueta: Option<String> = None;
    for c in s.chars() {
        match (&mut etiqueta, c) {
            (None, '<') => etiqueta = Some(String::from("<")),
            (Some(e), '>') => {
                let bloco = BLOCOS.iter().any(|b| e.starts_with(b));
                fora.push_str(if bloco { "\n\n" } else { " " });
                etiqueta = None;
            }
            (Some(e), _) => e.push(c),
            (None, _) => fora.push(c),
        }
    }
    fora.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// ⭐ **As frases de rota dos TUTORIAIS** — a cena de cada um é a do 1.º `PH2D_GPU_COOK_DEMO=<n>`
/// dele (o comando de abrir); uma frase sem *«nesta cena»* fala da LEI.
pub(super) fn frases_dos_tutoriais() -> Vec<Achada> {
    let mut nomes: Vec<_> = std::fs::read_dir(tutoriais())
        .expect("tutoriais")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".html"))
        .collect();
    nomes.sort();
    let mut fora = Vec::new();
    for nome in nomes {
        let html = std::fs::read_to_string(tutoriais().join(&nome)).expect("html");
        let cena = html.split("PH2D_GPU_COOK_DEMO=").nth(1).map(|r| {
            Cena::Demo(
                r.chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>(),
            )
        });
        for frase in frases(&texto_do_html(&html)) {
            let classe = classifica(&frase);
            if !tem_rota(&classe) {
                continue;
            }
            let desta = frase.to_lowercase().contains("nesta cena")
                || classe
                    .afirma
                    .iter()
                    .any(|a| !matches!(a, Afirma::Cartoes(..)));
            fora.push(Achada {
                onde: nome.clone(),
                frase,
                classe,
                cena: if desta { cena.clone() } else { None },
            });
        }
    }
    fora
}
