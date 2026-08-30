//! ⭐⭐⭐ **O LEITOR DE TABELA — a porta ÚNICA da pergunta *"o que este ficheiro contém?"***
//!
//! Recebe TEXTO e devolve colunas numéricas com nome. **Não abre ficheiros e não conhece
//! caminhos**: quem lê o disco é o shell, uma vez por ficheiro, nunca dentro do cook — a lei do
//! [doc 63 §6](../../docs/Motion%20Nodes/63_pesquisa_industria_2026_e_plano_estado_da_arte.md),
//! e o `audio.bands` é o precedente vivo.
//!
//! ⚠️ **UMA porta, e não uma por consumidor.** O `source.table` (uma linha = um ELEMENTO) e o
//! `value.table` (uma linha = um MOMENTO) leem o mesmo ficheiro; dois leitores divergiriam no
//! dia em que o separador ou a deteção de cabeçalho mudasse, e o artista veria duas tabelas
//! diferentes do mesmo ficheiro.
//!
//! # As referências, e o que cada política deve a qual
//!
//! | política | de onde vem |
//! |---|---|
//! | uma linha vira um elemento | *Import CSV* do Blender (4.3+) e *Table Import* do Houdini |
//! | nomes do CABEÇALHO | os dois, e o Houdini auto-deteta nome+tipo da 1.ª linha |
//! | **só colunas numéricas** | Blender: *"numeric columns imported as attributes"* |
//! | célula vazia = `0` | Houdini: *"out of bound columns will be treated as 0"* |
//!
//! ⛔ **UMA divergência DELIBERADA do Blender, e ela evita um silêncio.** Ele infere o tipo da
//! coluna **do primeiro valor**; aqui a coluna é numérica quando **toda** célula não-vazia
//! converte. Com a regra dele, uma coluna `ano_nota` cujo primeiro valor calha ser `1990` e o
//! resto é texto entraria como uma coluna de **zeros** — dado falso, sem aviso. A nossa
//! salta-a e **nomeia-a** ([`Table::report`]).
//!
//! ⛔ **E o `P` do Houdini não é portável para cá, por TIPO.** Lá, o nome especial `P` numa
//! coluna faz a posição do ponto — mas lá um atributo abrange **3 colunas**. Aqui a coluna `P`
//! do stream é um `Vec2` e uma coluna de CSV é **uma escalar**: o nome não pode ser honrado, e
//! deixá-lo passar sequestraria a posição com metade do dado. ⇒ um cabeçalho que use um nome do
//! motor é **saltado e nomeado**, nunca renomeado em silêncio.

#![forbid(unsafe_code)]

/// Os separadores que a deteção considera, por ordem de desempate.
///
/// ⚠️ **`;` está aqui porque o Excel de meia Europa exporta com ele** quando a vírgula é o
/// separador decimal — um leitor que só saiba `,` devolve uma coluna só e o artista lê isso
/// como *"a app não abre o meu ficheiro"*.
const DELIMITERS: [char; 4] = [',', ';', '\t', '|'];

/// Uma coluna numérica, com o nome que o cabeçalho lhe deu.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedColumn {
    pub name: String,
    pub values: Vec<f32>,
}

/// Por que uma coluna não entrou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkipReason {
    /// Alguma célula não-vazia não é um número — com a primeira que falhou, para o artista
    /// poder ir lá ver.
    NotNumeric { first_bad: String },
    /// Um cabeçalho repetido: a segunda ocorrência não entra (a primeira fica).
    DuplicateName,
}

/// ⭐⭐⭐ **TUDO O QUE VALE A PENA DIZER SOBRE ESTA LEITURA** — e ela existe porque a promessa
/// anterior era FALSA.
///
/// ⛔⛔ O campo antigo chamava-se `skipped` e o doc dele dizia *"⚠️ Nunca em silêncio: o painel
/// mostra isto"*. A auditoria de 2026-08-30 foi ver quem o lia: **ninguém** — nem painel, nem
/// aviso, nem registo. ⇒ toda a justificação da divergência deliberada contra o Blender (recusar
/// a coluna inteira em vez de a converter pelo 1.º valor) **pagava o preço e não entregava o
/// benefício**. *Uma decisão que se justifica por um aviso tem de ter o aviso.*
///
/// Hoje quem o diz é a shell, uma vez por ficheiro, por [`Table::report`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Note {
    /// A coluna não entrou.
    Skipped { name: String, reason: SkipReason },
    /// A coluna entrou com OUTRO nome — o cabeçalho colidia com uma coluna do motor.
    /// ⚠️ Escrito pela shell: o leitor não conhece o motor (ver o `Cargo.toml`).
    Renamed { from: String, to: String },
    /// Quantas células daquela coluna estavam vazias e foram lidas como `0`.
    ///
    /// ⚠️ **Inofensivo numa coluna de dados e VENENO na coluna do tempo**: medido, uma célula
    /// vazia lá reordena a curva inteira e há linhas que nunca saem em instante nenhum.
    EmptyCells { name: String, count: usize },
    /// Linhas com um número de células diferente do cabeçalho — o RFC 4180 di-las obrigatórias.
    RaggedRows { count: usize, expected: usize },
    /// O ficheiro não era UTF-8 e foi lido noutra codificação. ⚠️ Escrito pela shell.
    Encoding { from: &'static str },
}

/// O que um ficheiro de tabela contém, já em colunas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub columns: Vec<NamedColumn>,
    /// Quantas linhas de DADOS (sem o cabeçalho).
    pub rows: usize,
    /// Tudo o que vale a pena dizer sobre esta leitura — ver [`Note`].
    pub notes: Vec<Note>,
    /// O separador que a deteção escolheu.
    pub delimiter: char,
    /// Se a primeira linha foi lida como cabeçalho.
    pub had_header: bool,
}

impl Table {
    /// A coluna com este nome, se existir.
    #[must_use]
    pub fn column(&self, name: &str) -> Option<&NamedColumn> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// ⭐ **A frase que a shell imprime** — `None` quando não há nada a dizer.
    ///
    /// ⚠️ **Uma porta só.** Se cada consumidor escrevesse a sua, o artista veria o mesmo
    /// ficheiro descrito de duas maneiras — e a que ele lê seria a que envelheceu.
    #[must_use]
    pub fn report(&self) -> Option<String> {
        if self.notes.is_empty() {
            return None;
        }
        let mut out = String::new();
        for n in &self.notes {
            if !out.is_empty() {
                out.push('\n');
            }
            match n {
                Note::Skipped {
                    name,
                    reason: SkipReason::NotNumeric { first_bad },
                } => out.push_str(&format!(
                    "  coluna «{name}» IGNORADA: «{first_bad}» nao e' um numero"
                )),
                Note::Skipped {
                    name,
                    reason: SkipReason::DuplicateName,
                } => out.push_str(&format!("  coluna «{name}» IGNORADA: nome repetido")),
                Note::Renamed { from, to } => out.push_str(&format!(
                    "  coluna «{from}» passou a chamar-se «{to}»: o nome e' do motor"
                )),
                Note::EmptyCells { name, count } => out.push_str(&format!(
                    "  coluna «{name}»: {count} celula(s) vazia(s), lidas como 0"
                )),
                Note::RaggedRows { count, expected } => out.push_str(&format!(
                    "  {count} linha(s) sem as {expected} celulas do cabecalho"
                )),
                Note::Encoding { from } => {
                    out.push_str(&format!("  o ficheiro nao era UTF-8; lido como {from}"))
                }
            }
        }
        Some(out)
    }
}

/// ⭐ **Lê a tabela.** Nunca falha e nunca entra em pânico: um ficheiro que não é uma tabela
/// devolve zero colunas, que é o que todo consumidor já sabe desenhar (a mesma política do
/// `audio.bands` para um ficheiro ausente — *o nó não adivinha e não falha*).
#[must_use]
pub fn parse(text: &str) -> Table {
    // ⚠️ O `str::lines` já come o `\r` do `\r\n`, então um ficheiro do Windows lê-se sem
    // tratamento nenhum — uma trait para isso seria código a resolver um problema resolvido.
    // ⚠️⚠️ **O BOM (`U+FEFF`) é descascado, e a auditoria mostrou os dois estragos que ele faz:**
    // com cabeçalho, o nome da 1.ª coluna fica com um caractere INVISÍVEL (então `Time Column:
    // tempo` digitado à mão nunca casa, e o artista lê isso como *"o nó não funciona"*); sem
    // cabeçalho, o `\u{feff}1` não converte, a deteção de cabeçalho dispara, e a **primeira
    // linha de dados evapora**.
    // ⛔ O RFC 4180 não menciona BOM nenhum — a razão de o descascar não é a norma, é que
    // ficheiros com ele existem e o leitor partia-se com eles.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let Some(first) = lines.first() else {
        return Table {
            delimiter: DELIMITERS[0],
            ..Table::default()
        };
    };
    let delimiter = detect_delimiter(first);
    let grid: Vec<Vec<String>> = lines.iter().map(|l| split_row(l, delimiter)).collect();
    // ⚠️ A 1.ª linha é CABEÇALHO quando alguma célula não-vazia dela não é um número — a
    // auto-deteção do Houdini. Uma tabela toda numérica não tem cabeçalho, e é o caso comum de
    // um dump de sensor.
    let had_header = grid[0]
        .iter()
        .any(|c| !c.trim().is_empty() && parse_number(c, delimiter).is_none());
    let width = grid.iter().map(Vec::len).max().unwrap_or(0);
    let names: Vec<String> = (0..width)
        .map(|i| {
            let raw = if had_header {
                grid[0].get(i).map(String::as_str).unwrap_or("")
            } else {
                ""
            };
            let clean = raw.trim();
            if clean.is_empty() {
                // ⚠️ `col1` e não `col0`: quem conta colunas numa folha de cálculo conta de 1.
                format!("col{}", i + 1)
            } else {
                clean.to_string()
            }
        })
        .collect();
    let body = if had_header { &grid[1..] } else { &grid[..] };

    let mut columns: Vec<NamedColumn> = Vec::new();
    let mut notes: Vec<Note> = Vec::new();
    // ⚠️ **Linhas curtas ou compridas são um FACTO sobre o ficheiro** (o RFC 4180 di-las
    // obrigatoriamente iguais), e sem esta contagem uma aspa solta transforma um valor real
    // num `0` — medido — com o `notes` vazio.
    let ragged = body.iter().filter(|r| r.len() != width).count();
    if ragged > 0 {
        notes.push(Note::RaggedRows {
            count: ragged,
            expected: width,
        });
    }
    for (i, name) in names.iter().enumerate() {
        if columns.iter().any(|c| &c.name == name) {
            notes.push(Note::Skipped {
                name: name.clone(),
                reason: SkipReason::DuplicateName,
            });
            continue;
        }
        let mut values = Vec::with_capacity(body.len());
        let mut bad: Option<String> = None;
        let mut empties = 0usize;
        for row in body {
            let cell = row.get(i).map(String::as_str).unwrap_or("");
            let t = cell.trim();
            if t.is_empty() {
                // Houdini: fora-de-limite é `0`. ⚠️ A célula VAZIA dentro dos limites é outro
                // caso — o Houdini di-lo do fora-de-limite —, e vai contada.
                values.push(0.0);
                empties += 1;
                continue;
            }
            match parse_number(t, delimiter) {
                Some(v) => values.push(v),
                None => {
                    bad = Some(t.to_string());
                    break;
                }
            }
        }
        match bad {
            Some(first_bad) => notes.push(Note::Skipped {
                name: name.clone(),
                reason: SkipReason::NotNumeric { first_bad },
            }),
            None => {
                if empties > 0 {
                    notes.push(Note::EmptyCells {
                        name: name.clone(),
                        count: empties,
                    });
                }
                columns.push(NamedColumn {
                    name: name.clone(),
                    values,
                });
            }
        }
    }
    Table {
        columns,
        rows: body.len(),
        notes,
        delimiter,
        had_header,
    }
}

/// O separador, escolhido pela linha de cabeçalho: o que mais aparece FORA de aspas.
///
/// ⚠️ Empate resolve-se pela ordem de [`DELIMITERS`], e zero ocorrências devolve a vírgula —
/// uma tabela de uma coluna só lê-se igual com qualquer separador.
fn detect_delimiter(line: &str) -> char {
    let mut best = (DELIMITERS[0], 0usize);
    for d in DELIMITERS {
        let n = count_outside_quotes(line, d);
        if n > best.1 {
            best = (d, n);
        }
    }
    best.0
}

fn count_outside_quotes(line: &str, d: char) -> usize {
    let mut quoted = false;
    let mut n = 0;
    for c in line.chars() {
        if c == '"' {
            quoted = !quoted;
        } else if c == d && !quoted {
            n += 1;
        }
    }
    n
}

/// Uma linha em células, com o essencial do RFC 4180: `"…"` protege o separador, e `""` dentro
/// de aspas é uma aspa literal.
fn split_row(line: &str, d: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cur.push('"');
                } else {
                    quoted = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            quoted = true;
        } else if c == d {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

/// Um número, com as duas formas que uma folha de cálculo emite.
///
/// ⚠️⚠️ **A VÍRGULA DECIMAL só é aceite quando o SEPARADOR não é a vírgula**, e é por isso que
/// esta função o recebe.
///
/// ⛔⛔ **A 1.ª redacção não o recebia, e o doc-comment dela afirmava que não precisava:**
/// *"quando o separador É a vírgula ela nunca chega aqui — o `split_row` já partiu a célula"*.
/// **Falso, e medido na auditoria de 2026-08-30:** o `split_row` devolve a célula com a vírgula
/// **intacta** quando ela veio entre ASPAS, que é exactamente como uma folha de cálculo escreve
/// um milhar. ⇒ `"1,200"` num ficheiro de vírgulas saía **`1,2`** — dividido por mil, em
/// silêncio e sem uma linha em [`Table::notes`]. E no mesmo ficheiro `"1.234,56"` matava a
/// coluna inteira, então metade do dado sumia e a outra metade estava errada.
///
/// ⚠️ E o `%` final é comido, **um só**: `12%` é `12`. Escalar por cento é decisão de quem
/// desenha, não do leitor — ele não pode adivinhar se o artista quer `0,12`.
///
/// ⚠️ **`hh:mm:ss` vira SEGUNDOS.** Um registador de dados escreve o tempo assim, e sem isto a
/// coluna que faz o `value.table` ser uma curva era recusada como «não numérica» — o nó ficava
/// com a caixa `Time Column` que nenhuma fonte real conseguia preencher.
fn parse_number(cell: &str, delimiter: char) -> Option<f32> {
    let t = cell.trim();
    let t = t.strip_suffix('%').unwrap_or(t).trim();
    if t.is_empty() {
        return None;
    }
    if let Some(secs) = parse_clock(t) {
        return Some(secs);
    }
    if let Ok(v) = t.parse::<f32>() {
        return v.is_finite().then_some(v);
    }
    // Vírgula decimal — só onde ela NÃO é o separador, e só se houver uma e nenhum ponto.
    if delimiter != ','
        && t.matches(',').count() == 1
        && !t.contains('.')
        && let Ok(v) = t.replace(',', ".").parse::<f32>()
    {
        return v.is_finite().then_some(v);
    }
    None
}

/// `hh:mm:ss[.f]` ou `mm:ss[.f]` em segundos. `None` para tudo o resto.
fn parse_clock(t: &str) -> Option<f32> {
    let parts: Vec<&str> = t.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }
    let mut secs = 0.0f64;
    for (i, p) in parts.iter().enumerate() {
        let v: f64 = p.trim().parse().ok()?;
        if !v.is_finite() || v < 0.0 {
            return None;
        }
        // O último campo são segundos; os anteriores multiplicam por 60 a cada nível.
        let scale = 60f64.powi((parts.len() - 1 - i) as i32);
        secs += v * scale;
    }
    let out = secs as f32;
    out.is_finite().then_some(out)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
