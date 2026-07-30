//! **O RASCUNHO DO SOLVER** (filho de [`super`] — teto de LOC): os planos
//! DERIVADOS que os dois passes independentes de ordem materializam por passo
//! (doc 28 §5.45).
//!
//! Eles moram no `Grid` pelo mesmo motivo que `live_lo`/`live_hi`: o passo roda
//! a 40 Hz e não pode alocar. E **não entram no `GridSnapshot`** — o conteúdo é
//! reescrito inteiro dentro de um passe, então não há nada que um histórico
//! pudesse preservar.

/// O registro de destino do [`crate::solver::advect_jacobi`] — as três
/// grandezas que a advecção move, num só plano.
///
/// ⚠️ **Intercalado de propósito.** O *commit* lê os três de uma vez; três
/// planos separados fariam três passadas sobre a mesma janela usando um terço
/// de cada linha de cache.
#[derive(Clone, Copy, Default)]
pub struct AdvCell {
    pub film: f32,
    pub susp: f32,
    pub rgb: [f32; 3],
}

/// O rascunho do [`crate::solver::advect_jacobi`] — **derivado, não estado**.
///
/// Ele mora no `Grid` pelo MESMO motivo que `live_lo`/`live_hi`: o passe roda
/// a 40 Hz e não pode alocar. E **não entra no [`GridSnapshot`]** pela mesma
/// razão que aqueles não entram — o conteúdo dele é reescrito inteiro dentro
/// de um passe, então não há nada que um histórico pudesse preservar.
///
/// ⚠️ **Alocado PREGUIÇOSAMENTE** ([`SolverScratch::ensure`]): são 24 B por
/// célula, e um `Grid` que nunca dá um passo (o carimbo de um dab, uma
/// restauração de histórico) não os paga.
#[derive(Default)]
pub struct SolverScratch {
    /// O fluxo transiente **na grade FINA**, materializado uma vez por passo.
    ///
    /// ⚠️ **Ele é o que torna o gather viável, e o número está medido:** a
    /// vizinhança de cada célula pergunta o fluxo de 9 vizinhos, e re-amostrar
    /// (uma bilinear por pergunta na grade grossa) fazia a rota serial do
    /// gather custar **180,8 ms contra 36,0 do Gauss-Seidel**. Amostrado uma
    /// vez, o vizinho vira **carga**.
    pub uv: Vec<[f32; 2]>,
    /// A fração da massa que DEIXA cada célula neste passo — a soma dos pesos
    /// com que os destinos da vizinhança a puxam.
    pub outflow: Vec<f32>,
    /// O estado de destino, antes do *commit*.
    pub dst: Vec<AdvCell>,
    /// Quantos dos nove vizinhos carregam pigmento, **lidos ANTES** de a
    /// secagem escrever qualquer célula ([`crate::drying::drying_pass`]).
    ///
    /// ⚠️ **É este plano que torna a secagem independente de ordem.** O fator
    /// de borda é a única leitura cross-célula do passe, e ele lê o `susp` que
    /// o próprio passe reescreve — materializá-lo num pré-passe (gather puro)
    /// deixa o laço principal com leituras e escritas SÓ no próprio índice.
    pub edge: Vec<u8>,
}

impl SolverScratch {
    /// Garante os planos com `n` células. Idempotente e sem realocar depois da
    /// primeira vez.
    pub fn ensure(&mut self, n: usize) {
        if self.outflow.len() != n {
            self.uv = vec![[0.0; 2]; n];
            self.outflow = vec![0.0; n];
            self.dst = vec![AdvCell::default(); n];
            self.edge = vec![0; n];
        }
    }
}
