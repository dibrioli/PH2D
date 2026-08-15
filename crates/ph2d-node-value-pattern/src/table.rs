//! A TABELA do `value.pattern` — a lista de valores AUTORADA como texto, e a
//! metade dela que o device consome.
//!
//! O que mora aqui é a LEI: *dada uma string, que lista de números ela é, e que
//! valor o elemento `i` recebe?* — e ela é a mesma nos dois caminhos (a CPU
//! chama [`parse`] + [`value_at`]; o WGSL do `lib.rs` lê o MESMO vetor pela
//! LUT que [`fill_lut`] escreve). Uma segunda cópia da gramática é como a face
//! do artista e o device passam a discordar sobre onde um valor acaba.

/// Quantos valores a tabela carrega, no máximo.
///
/// ⚠️ **É teto de RECURSO, e o recurso é o BUFFER DO DEVICE** — a LUT é um
/// `storage` de `LUT_LEN` floats por nó `value.pattern` no documento, alocado a
/// cada cook. Medido (`measure_table_cost`, `-- --ignored`):
///
/// | entradas | `fill` |
/// |----------|--------|
/// | 8        | 0,08 µs |
/// | 64       | 0,55 µs |
/// | 256      | 2,02 µs |
/// | 1024     | 7,94 µs |
///
/// O buffer é **4100 bytes, CONSTANTE** (a `resolution` de um `LutSpec` é uma
/// const, então ele não encolhe com a tabela curta), e é ele que decide: uma cena
/// com cem `value.pattern` paga **410 kB** de VRAM. O `fill` é linear no que foi
/// escrito, roda **uma vez por encode**, e a 1024 entradas cobra 8 µs — três
/// ordens abaixo de um quadro de 60 fps.
///
/// ⚠️ **O teto NÃO é o do painel** — o de antes era **8**, e ele vinha de *"params
/// são f32"*, não de recurso nenhum (folha 15). Este diz de que recurso é.
pub const MAX_ENTRIES: usize = 1024;

/// O comprimento do buffer da LUT: o cabeçalho (a CONTAGEM) mais as entradas.
///
/// ⚠️ **A contagem viaja no slot 0 porque o device não tem outra forma de sabê-la.**
/// Um `arrayLength` devolve a capacidade (`LUT_LEN`), não quantos valores o
/// artista escreveu, e um param f32 paralelo seria a segunda cópia de um número
/// que a própria string já determina — o par que diverge no dia em que alguém
/// edita um sem o outro.
pub const LUT_LEN: u32 = MAX_ENTRIES as u32 + 1;

/// **A gramática:** números separados por espaço e/ou vírgula.
///
/// Devolve a lista VAZIA para uma string ausente, vazia ou **malformada** — e a
/// lista vazia é o sinal de *"nada autorado"*, que faz o nó cair nos oito slots
/// legados. Recusar a string INTEIRA (em vez de pular o token ruim) é a política
/// do `ph2d_curve::parse`, e é a que se vê: um token ruim pulado apagaria um
/// slot em silêncio, e o padrão sairia com um passo a menos sem nada na tela
/// dizer por quê.
///
/// ⚠️ **Não-finito é malformado**, embora `"nan".parse::<f32>()` seja `Ok`: um
/// NaN aqui atravessa toda a cadeia a jusante e envenena o que ele tocar.
///
/// ⚠️ **Acima de [`MAX_ENTRIES`] a lista é TRUNCADA, não recusada** — o artista
/// mantém os primeiros mil valores funcionando, em vez de perder a tabela
/// inteira por causa da milésima primeira entrada.
pub fn parse(text: &str) -> Vec<f32> {
    let mut out = Vec::new();
    for tok in text.split([' ', '\t', '\n', '\r', ',', ';']) {
        if tok.is_empty() {
            continue;
        }
        match tok.parse::<f32>() {
            Ok(v) if v.is_finite() => out.push(v),
            // Um token ruim invalida a tabela INTEIRA (ver o doc acima).
            _ => return Vec::new(),
        }
        if out.len() == MAX_ENTRIES {
            break;
        }
    }
    out
}

/// O valor do elemento `i` sob a tabela `table` — `table[i mod len]`, o mesmo
/// ciclo por índice que os oito slots sempre fizeram.
///
/// Lista vazia devolve `None`: quem chama cai no caminho legado.
pub fn value_at(i: usize, table: &[f32]) -> Option<f32> {
    if table.is_empty() {
        None
    } else {
        Some(table[i % table.len()])
    }
}

/// Escreve a tabela no buffer da LUT — a metade do canal que mora no NÓ
/// (`ph2d-nodegraph` fica agnóstico da gramática, exatamente como fica agnóstico
/// de curva).
///
/// **O layout é `[len, v0, v1, …]`**: o slot 0 carrega quantos valores seguem, e
/// `len == 0` é o que diz ao WGSL para tomar o ramo legado — o mesmo sinal que a
/// lista vazia dá à CPU, de modo que os dois caminhos concordam sobre *"nada
/// autorado"* sem cada um ter a própria regra.
///
/// ⚠️ O resto do buffer é deixado como está (o chamador o entrega zerado); ler
/// além de `len` é inalcançável por construção, porque o índice é `i % len`.
pub fn fill_lut(text: &str, out: &mut [f32]) {
    let table = parse(text);
    if out.is_empty() {
        return;
    }
    // Cabe por construção (`LUT_LEN = MAX_ENTRIES + 1`), mas o `min` mantém o
    // preenchimento total mesmo que alguém encolha a `resolution`.
    let n = table.len().min(out.len().saturating_sub(1));
    #[expect(
        clippy::cast_precision_loss,
        reason = "n <= MAX_ENTRIES = 1024, exatamente representável em f32"
    )]
    {
        out[0] = n as f32;
    }
    out[1..=n].copy_from_slice(&table[..n]);
}

#[cfg(test)]
#[path = "table_tests.rs"]
mod tests;
