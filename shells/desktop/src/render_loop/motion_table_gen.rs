//! ⭐⭐⭐ **A TABELA VIVA — o disco é lido AQUI, uma vez por ficheiro, nunca dentro do cook.**
//!
//! É a lei do [doc 63 §6](../../../../docs/Motion%20Nodes/63_pesquisa_industria_2026_e_plano_estado_da_arte.md)
//! (*"a FFT NUNCA entra no cook"*) aplicada a um segundo tipo de dado, e o desenho é o mesmo do
//! [`super::motion_audio_gen`]: a shell faz o trabalho pesado e **publica** o resultado no canal
//! externo, sob a chave que o nó lê. Os dois nós de tabela não dependem do leitor — o gate
//! `a_node_that_reads_a_file_cannot_even_depend_on_the_reader` lê os `Cargo.toml` deles para o provar.
//!
//! ⚠️ **Um leitor, dois nós.** O `source.table` (linha = elemento) e o `value.table`
//! (linha = momento) publicam sob a MESMA chave, que é o caminho: um ficheiro apontado pelos
//! dois é lido **uma** vez, e nunca podem discordar sobre o que ele contém.

use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_nodegraph::attr::{Column, Stream};

use crate::motion_state::MotionState;

/// O que o diálogo oferece quando o nó pede uma [`ph2d_node_registry::FileKind::Table`].
///
/// ⚠️ **A lista é da SHELL e o leitor é um só**: ele DETETA o separador, então `.csv`, `.tsv` e
/// `.txt` são o mesmo caminho de código. Isto diz o que o artista consegue escolher, nunca o que
/// o leitor sabe fazer.
pub(crate) const TABLE_EXTS: &[&str] = &["csv", "tsv", "txt", "tab"];

/// ⭐ **O bloco `0x80..=0x9F` do CP1252** — a única parte em que ele difere do Latin-1.
///
/// ⚠️ Existe porque o *Table DAT* do TouchDesigner — a referência que este ficheiro cita —
/// expõe `Default Read Encoding: Auto Detect / UTF8 / UTF16-LE / UTF16-BE / **CP1252**`, e
/// medido em 2026-08-30 um CSV cp1252 do Excel devolvia aqui uma tabela **vazia e memoizada**,
/// indistinguível de *«não há ficheiro»*.
const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{FFFD}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{FFFD}', '\u{017D}', '\u{FFFD}',
    '\u{FFFD}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{FFFD}', '\u{017E}', '\u{0178}',
];

fn cp1252_to_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0x80..=0x9F => CP1252_HIGH[(b - 0x80) as usize],
            _ => b as char,
        })
        .collect()
}

/// Os nomes que o motor usa e que a [`ph2d_nodegraph::attr::is_bookkeeping_column`] **não**
/// cobre, porque ela responde a outra pergunta (*"isto é estado de simulação?"*).
///
/// ⚠️ Cada um está aqui por um CONSUMIDOR: `P` é a posição (`Vec2`, e uma célula é escalar) ·
/// `size`/`rot`/`wrot`/`len`/`parent`/`depth`/`gen`/`sym` são o vocabulário que a tartaruga e o
/// rig escrevem · **`falloff` é lido pelo `motion.scale`** (ausente ⇒ `1.0`), então uma coluna
/// com esse nome multiplicaria a escala de tudo a jusante · `v` é a `VALUE_COLUMN` do domínio de
/// valor · `Index`/`Count` são reescritos pelo próprio nó.
const RESERVED_BY_SHAPE: &[&str] = &[
    "P", "parent", "len", "rot", "wrot", "size", "depth", "gen", "sym", "Index", "Count",
    "falloff", "v", "vel", "accel", "tint", "age", "life",
];

/// O que uma leitura deixa: o stream que os nós recebem, e a frase que o artista lê.
pub(crate) struct Loaded {
    pub(crate) stream: Stream,
    pub(crate) report: Option<String>,
}

/// As tabelas lidas, por CAMINHO.
///
/// ⚠️ **Nada é despejado, e o recurso está NOMEADO** — e desde 2026-08-30 ele é **contado** pela
/// soak anti-acumulação (`Census::tables`). Sem contagem, *"um despejo entraria no dia em que
/// isto medisse alguma coisa"* era uma frase sem instrumento.
#[derive(Default)]
pub(crate) struct TableCache {
    tables: BTreeMap<String, Arc<Loaded>>,
}

impl TableCache {
    /// A tabela daquele caminho, lida no máximo uma vez — e **DITA** uma vez.
    ///
    /// ⚠️ **Um caminho ilegível é memoizado como VAZIO**, senão todo quadro tentaria abrir o
    /// mesmo ficheiro que não existe — 60 `open()` falhados por segundo, por nó.
    pub(crate) fn load(&mut self, path: &str) -> Arc<Loaded> {
        if let Some(hit) = self.tables.get(path) {
            return Arc::clone(hit);
        }
        let loaded = Arc::new(read_and_shape(path));
        if let Some(r) = &loaded.report {
            // ⭐⭐ **A promessa cumprida.** O leitor recusa a coluna inteira quando UMA célula
            // não converte (a divergência deliberada contra o Blender, que infere pelo 1.º
            // valor), e essa troca só compensa se o artista souber. Até 2026-08-30 **ninguém
            // lia** a lista de recusas: o preço era pago e o benefício não existia.
            eprintln!("[tabela] {path}\n{r}");
        }
        self.tables.insert(path.to_string(), Arc::clone(&loaded));
        loaded
    }

    /// ⭐ **Esquece o que foi lido** — o recarregar, que aqui é um GESTO.
    ///
    /// ⛔ **Um observador de disco foi recusado, e o argumento é NOSSO:** um ficheiro que se relê
    /// sozinho faz o mesmo projeto, no mesmo instante, desenhar coisas diferentes, e o scrub
    /// deixa de ser exacto. ⚠️ **A referência está DIVIDIDA** — o *File In DAT* do TouchDesigner
    /// tem um `Refresh Pulse` (gesto) e o *Table DAT*, que é o nó equivalente ao nosso, tem
    /// `Sync to File`, que **vigia o disco**. A 1.ª redacção chamou ao gesto *"o padrão-ouro"*
    /// citando o DAT errado: a decisão fica, a palavra não.
    pub(crate) fn forget(&mut self, path: &str) {
        self.tables.remove(path);
    }

    /// Quantas tabelas estão vivas — lido pela soak anti-acumulação e pelos gates (o mesmo
    /// `#[cfg(test)]` do `BandCache::len`: é uma sonda, não produto).
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.tables.len()
    }
}

/// Lê o ficheiro, converte a codificação se preciso, e dá forma ao stream.
///
/// ⚠️ **O renomear de colunas do MOTOR vive aqui, e não no leitor**: quem sabe que `id`, `sim_t`
/// ou `v` são do motor é o `ph2d_nodegraph`, e o leitor **não pode depender dele** — é essa
/// cerca que o mantém incapaz de opinar sobre o cook. A shell depende dos dois.
///
/// ⛔⛔ **E RENOMEIA em vez de SALTAR.** A 1.ª redacção tinha uma lista à mão dentro do leitor
/// que (a) omitia `v`, `falloff` e `id` — os três que o plano 93 nomeia como o perigo — e (b)
/// bloqueava nove escalares inofensivos, entre eles `size`, o nome mais plausível para a coluna
/// de um gráfico de barras. Medido: um CSV com uma coluna `id` entrava no motor **como
/// identidade**, e um com `falloff` multiplicava a escala de tudo a jusante.
/// *Renomear não perde dado nenhum e não sequestra nada.*
fn read_and_shape(path: &str) -> Loaded {
    let Ok(bytes) = std::fs::read(path) else {
        return Loaded {
            stream: Stream::new(0),
            report: None,
        };
    };
    let (text, encoding) = match String::from_utf8(bytes) {
        Ok(t) => (t, None),
        Err(e) => (cp1252_to_string(e.as_bytes()), Some("CP1252")),
    };
    let table = ph2d_table::parse(&text);
    let mut notes = table.notes.clone();
    if let Some(from) = encoding {
        notes.insert(0, ph2d_table::Note::Encoding { from });
    }
    let mut stream = Stream::new(table.rows);
    for c in &table.columns {
        // ⚠️ A pergunta *"este nome é do motor?"* tem UMA porta, e é a do motor.
        let name = if ph2d_nodegraph::attr::is_bookkeeping_column(&c.name)
            || RESERVED_BY_SHAPE.contains(&c.name.as_str())
        {
            let renamed = format!("{}_csv", c.name);
            notes.push(ph2d_table::Note::Renamed {
                from: c.name.clone(),
                to: renamed.clone(),
            });
            renamed
        } else {
            c.name.clone()
        };
        stream.set(name, Column::Scalar(c.values.clone()));
    }
    let shaped = ph2d_table::Table {
        notes,
        ..ph2d_table::Table::default()
    };
    Loaded {
        stream,
        report: shaped.report(),
    }
}

/// Publica, para cada nó de tabela do grafo, o conteúdo do ficheiro dele.
///
/// ⚠️ **A chave NÃO carrega o instante** — a tabela não é função do playhead. É por isso que o
/// `source.table` é `Effect::Pure`, e o `value.table` é `Temporal` por ler o playhead ELE, e não
/// por o dado mudar.
pub(crate) fn publish(motion: &mut MotionState) {
    let wanted: Vec<String> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| {
            n.type_name == ph2d_node_source_table::MANIFEST.name
                || n.type_name == ph2d_node_value_table::MANIFEST.name
        })
        .filter_map(|n| {
            motion
                .doc
                .graph
                .node_text_params()
                .get(&n.id)
                // ⚠️⚠️ **A chave de CADA nó, e não a do `source.table` para os dois.** Até
                // 2026-08-30 esta linha usava a constante do `source.table` para ambos, e as
                // duas valiam `"file"` **por coincidência**: uma auditoria renomeou a do
                // `value.table` para `"path"` e a suíte inteira do shell ficou VERDE, com o nó
                // a receber dado nenhum. O gate `the_two_table_nodes_name_the_file_the_same_way`
                // fecha o outro lado.
                .and_then(|m| {
                    let key = if n.type_name == ph2d_node_value_table::MANIFEST.name {
                        ph2d_node_value_table::FILE_KEY
                    } else {
                        ph2d_node_source_table::FILE_KEY
                    };
                    m.get(key)
                })
                .cloned()
        })
        .filter(|p| !p.is_empty())
        .collect();
    for path in wanted {
        let loaded = motion.table_cache.load(&path);
        let key = ph2d_node_registry::table_external_key(&path);
        motion.pump.cook.set_external(key, loaded.stream.clone());
    }
}

#[cfg(test)]
#[path = "motion_table_gen_tests.rs"]
mod tests;
