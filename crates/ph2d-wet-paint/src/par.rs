//! **A caminhada por LINHAS, serial ou paralela — a mesma resposta** (ADR-0145,
//! a 2ª exceção sancionada ao "sem rayon" do repo; a 1ª é o
//! [ADR-0109](../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md)).
//!
//! O solver deste motor é **serial POR SEMÂNTICA** em três dos seus sete
//! passes, e isso não é negociável — mas não em TODOS, e a diferença é
//! mecânica, não de gosto. Um passe é row-paralelo byte-idêntico quando as
//! três condições do ADR-0109 valem, e aqui elas viram uma pergunta só:
//!
//! 1. **cada linha escreve SÓ a própria linha** (fatias disjuntas);
//! 2. **o que ela LÊ, ninguém escreve neste passe** (buffer diferente, ou o
//!    próprio índice da célula) — é isso que faz de um passe um *gather*
//!    (`smooth_velocity`) ou um passo de **Jacobi** (`project`, que lê `div` e
//!    escreve `prs`, quatro laços, cada um lendo um buffer e escrevendo OUTRO);
//! 3. **nenhuma redução cuja ORDEM importe** — `min`/`max` sobre inteiros e
//!    `||` são associativos E comutativos, então o `reduce` paralelo devolve o
//!    número exato do `fold` serial. Uma SOMA em float não seria (e é por isso
//!    que o `advect`, que soma quatro cantos, não entra nem se pudesse).
//!
//! ⚠️ **A garantia é ESTRUTURAL, não revisada:** o corpo de cada linha é UMA
//! função, e as duas rotas apenas a caminham. Não existe "versão paralela" do
//! kernel para divergir da serial — [`Rows`] escolhe o *walker*, nunca a
//! aritmética. O que só um relógio distingue, só um gate de relógio precisa
//! testar; a identidade sai de graça.
//!
//! ⛔ **Quem NÃO entra, e por qual mecanismo** (medido lendo o kernel, não a
//! nota do ADR-0134):
//!
//! | passe | por que é sequencial |
//! |---|---|
//! | `advect` | SUBTRAI nos 4 cantos-fonte (`susp[i01]`, `film[i01]` — linhas VIZINHAS), read-modify-write de célula compartilhada |
//! | `build_flow_field` | o freio LÊ `wet[probe]` alguns px adiante, que o carimbo de umidade deste mesmo passe pode ter escrito; e o backrun ESPALHA em `susp[nb]`/`sett[nb]`/`sett_rgb[nb]` |
//! | `drying_pass` | o fator de borda lê a vizinhança 3×3 de `susp`, que o passe ESCREVE — a linha `y` lê `y−1` já atualizada |
//! | a SAIA do `rebuild_active_region` | escreve `active[i±s]`, e o comentário do produto diz por quê: *"scanned top-to-down so earlier 2s shape later sums"* |
//!
//! Qualquer uso novo de `rayon` nesta crate — em especial num destes quatro —
//! **exige ADR novo**, exatamente como a cerca de contenção do ADR-0109 diz.

use rayon::prelude::*;

/// Como as linhas de um passe row-paralelo são caminhadas.
///
/// As duas variantes são a **MESMA resposta** — a paralela existe só para o
/// relógio. O produto pergunta [`Rows::pick`]; os gates de identidade forçam
/// as duas sobre a MESMA fixture e comparam byte a byte.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rows {
    Serial,
    Parallel,
}

impl Rows {
    /// A rota para uma janela de `rows × span` células, dado o **piso medido do
    /// passe que pergunta**.
    ///
    /// ⚠️ **O piso é POR-PASSE, e isso é medição, não gosto** — e a medição
    /// derrubou DUAS versões minhas antes desta. A tabela é a razão
    /// serial÷paralelo por tamanho de janela (`tests/measure_parallel_rows.rs`,
    /// release, **estado restaurado antes de cada amostra**):
    ///
    /// ```text
    ///     celulas    project  smooth  rebuild
    ///       60_000     0,51x   0,44x    0,54x
    ///      122_952     0,73x   0,59x    0,77x
    ///      194_788     1,72x   0,95x    0,93x
    ///      411_166     1,92x   1,42x    1,11x
    ///      679_140     2,73x   1,82x    1,60x
    ///    2_546_830     4,35x   5,26x    2,30x
    ///    9_800_850     5,12x   5,26x    2,14x
    /// ```
    ///
    /// **Razão < 1,00 é o thread-pool custando mais que o trabalho**, e é por
    /// isso que os pisos são altos: abaixo deles o passe é sub-milissegundo, então
    /// ficar serial não custa nada e protege contra uma regressão real.
    ///
    /// ⚠️ **O `rebuild_active_region` tem o piso mais alto dos três** por duas
    /// razões que os outros não têm: o scan da extensão viva percorre **TODA linha
    /// da tela**, não a bbox — então o número de tarefas é `altura` mesmo numa
    /// poça minúscula —, e a SAIA fica serial, o que limita o teto dele a ~2,1×
    /// por Amdahl.
    ///
    /// ⚠️ **A metodologia é parte do número, e a primeira estava errada:** sem o
    /// restore por amostra a mesma varredura dava `smooth` a **3,01×** onde a
    /// honesta dá **0,95×** em 195k células — repetir um passe sobre o mesmo grid
    /// deixa a janela quente (e, no caso do `rebuild`, APERTA a bbox que ele
    /// próprio varre). Eu quase fixei os pisos 4× baixos demais por causa disso.
    ///
    /// A unidade é CÉLULA e não linha de propósito: uma janela de 4000 linhas
    /// com 3 células cada não é trabalho, é 4000 tarefas.
    ///
    /// Os três pisos que essa tabela produz são [`MIN_CELLS_JACOBI`],
    /// [`MIN_CELLS_GATHER`] e [`MIN_CELLS_REBUILD`] — eles moram **aqui**, ao
    /// lado da medição que os justifica, e não em cada passe: quem re-medir a
    /// tabela precisa ver os três de uma vez.
    pub fn pick(rows: usize, span: usize, min_cells: usize) -> Rows {
        if rows.saturating_mul(span) >= min_cells {
            Rows::Parallel
        } else {
            Rows::Serial
        }
    }
}

/// Piso do `project`: 0,73× em 123k células (prejuízo), 1,72× em 195k, 1,92× em
/// 411k. Tabela e metodologia no [`Rows::pick`].
pub const MIN_CELLS_JACOBI: usize = 256 << 10;

/// Piso do `smooth_velocity`: 0,95× em 195k (empate), 1,42× em 411k.
///
/// ⚠️ **Vale o MESMO número que o [`MIN_CELLS_JACOBI`] hoje, e continua sendo uma
/// const separada de propósito** — são duas medições de dois passes, não um fato
/// com dois nomes. Colapsá-las é exactamente a falha que os defaults do Wet Paint
/// pagaram (a baseline do reconcile e o perfil de produto eram iguais até o dia
/// em que não foram, e o early-return escondeu a divergência).
pub const MIN_CELLS_GATHER: usize = 256 << 10;

/// Piso do `rebuild_active_region` — **o DOBRO dos outros dois**: 1,11× em 411k
/// (marginal), 1,60× em 679k. Ele varre TODA linha da tela e tem a saia serial.
pub const MIN_CELLS_REBUILD: usize = 512 << 10;

/// Caminha `buf` em linhas de `stride`, chamando `f(indice_da_linha, linha)`.
///
/// `buf` TEM de medir um número inteiro de linhas — todo chamador fatia faixas
/// exatas, e uma linha curta no fim seria uma linha com aritmética diferente.
pub(crate) fn walk_rows<T, F>(mode: Rows, buf: &mut [T], stride: usize, f: F)
where
    T: Send,
    F: Fn(usize, &mut [T]) + Send + Sync,
{
    debug_assert_eq!(buf.len() % stride, 0, "a faixa nao mede linhas inteiras");
    match mode {
        Rows::Serial => buf
            .chunks_mut(stride)
            .enumerate()
            .for_each(|(r, row)| f(r, row)),
        Rows::Parallel => buf
            .par_chunks_mut(stride)
            .enumerate()
            .for_each(|(r, row)| f(r, row)),
    }
}

/// [`walk_rows`] para um passe que escreve **DOIS** planos na mesma passada
/// (as duas componentes de um campo vetorial — `flow_x`/`flow_y`,
/// `vel_x`/`vel_y`, `div`/`prs`).
pub(crate) fn walk_rows2<T, U, F>(mode: Rows, a: &mut [T], b: &mut [U], stride: usize, f: F)
where
    T: Send,
    U: Send,
    F: Fn(usize, &mut [T], &mut [U]) + Send + Sync,
{
    debug_assert_eq!(a.len(), b.len(), "os dois planos tem faixas diferentes");
    debug_assert_eq!(a.len() % stride, 0, "a faixa nao mede linhas inteiras");
    match mode {
        Rows::Serial => a
            .chunks_mut(stride)
            .zip(b.chunks_mut(stride))
            .enumerate()
            .for_each(|(r, (ra, rb))| f(r, ra, rb)),
        Rows::Parallel => a
            .par_chunks_mut(stride)
            .zip(b.par_chunks_mut(stride))
            .enumerate()
            .for_each(|(r, (ra, rb))| f(r, ra, rb)),
    }
}

/// [`walk_rows`] com uma **REDUÇÃO** por linha.
///
/// ⚠️ `red` TEM de ser associativa **e comutativa** — o `reduce` do rayon
/// combina em ordem de agendamento e pode chamar `ident` várias vezes. `min`,
/// `max` e `||` sobre inteiros/bools qualificam; uma soma em float **não**.
pub(crate) fn walk_rows_reduce<T, R, F, Red>(
    mode: Rows,
    buf: &mut [T],
    stride: usize,
    ident: R,
    f: F,
    red: Red,
) -> R
where
    T: Send,
    R: Send + Sync + Copy,
    F: Fn(usize, &mut [T]) -> R + Send + Sync,
    Red: Fn(R, R) -> R + Send + Sync,
{
    debug_assert_eq!(buf.len() % stride, 0, "a faixa nao mede linhas inteiras");
    match mode {
        Rows::Serial => buf
            .chunks_mut(stride)
            .enumerate()
            .map(|(r, row)| f(r, row))
            .fold(ident, &red),
        Rows::Parallel => buf
            .par_chunks_mut(stride)
            .enumerate()
            .map(|(r, row)| f(r, row))
            .reduce(|| ident, &red),
    }
}

/// [`walk_rows`] para um par de escalares POR LINHA (não uma fatia de canvas):
/// o rascunho `live_lo`/`live_hi` do rebuild, um `i32` por linha.
pub(crate) fn walk_row_scalars2<T, F>(mode: Rows, a: &mut [T], b: &mut [T], f: F)
where
    T: Send,
    F: Fn(usize, &mut T, &mut T) + Send + Sync,
{
    debug_assert_eq!(
        a.len(),
        b.len(),
        "os dois rascunhos tem tamanhos diferentes"
    );
    match mode {
        Rows::Serial => a
            .iter_mut()
            .zip(b.iter_mut())
            .enumerate()
            .for_each(|(r, (ra, rb))| f(r, ra, rb)),
        Rows::Parallel => a
            .par_iter_mut()
            .zip(b.par_iter_mut())
            .enumerate()
            .for_each(|(r, (ra, rb))| f(r, ra, rb)),
    }
}
