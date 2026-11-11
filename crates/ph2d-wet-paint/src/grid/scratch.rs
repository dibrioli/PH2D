//! **O RASCUNHO DO SOLVER** (filho de [`super`] — teto de LOC): os planos
//! DERIVADOS que os dois passes independentes de ordem materializam por passo
//! (doc 28 §5.45).
//!
//! Eles moram no `Grid` pelo mesmo motivo que `live_lo`/`live_hi`: o passo roda
//! a 40 Hz e não pode alocar. E **não entram no `GridSnapshot`** — o conteúdo é
//! reescrito inteiro dentro de um passe, então não há nada que um histórico
//! pudesse preservar.

/// As três grandezas que a advecção move, numa célula — a vista por célula do destino. O rascunho
/// guarda-as em planos ([`SolverScratch::dst_film`] & irmãos, 31/10): o *commit* é então uma cópia
/// contígua por plano para os planos do `Grid`, que têm a mesma forma.
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
/// ⚠️ **Alocado PREGUIÇOSAMENTE** ([`SolverScratch::ensure`]): são 25 B por
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
    /// A linha-base do retro-traço de cada destino, como `y − floor(y − uy)` (cabe em `i8`: `|uy| ≤ 3`);
    /// [`Self::BT_NADA`] para o inativo ou o que sai da folha. Deixa o `outflow_rows` filtrar as `2·alcance+1`
    /// linhas candidatas por um byte em vez de um retro-traço inteiro.
    pub bt: Vec<i8>,
    /// O estado de destino, antes do *commit*.
    pub dst_film: Vec<f32>,
    /// O `susp` de destino (o mesmo rascunho serve a [`crate::solver::sedimenta`]).
    pub dst_susp: Vec<f32>,
    /// A cor de destino, na forma do `Grid::susp_rgb`.
    pub dst_rgb: Vec<[f32; 3]>,
    /// Quantos dos nove vizinhos carregam pigmento, **lidos ANTES** de a
    /// secagem escrever qualquer célula ([`crate::drying::drying_pass`]).
    ///
    /// ⚠️ **É este plano que torna a secagem independente de ordem.** O fator
    /// de borda é a única leitura cross-célula do passe, e ele lê o `susp` que
    /// o próprio passe reescreve — materializá-lo num pré-passe (gather puro)
    /// deixa o laço principal com leituras e escritas SÓ no próprio índice.
    pub edge: Vec<u8>,
    /// Os dois predicados que o **backrun** pergunta aos VIZINHOS — `sett > 0`
    /// e `bloom < 6` —, lidos ANTES de o passe escrever qualquer célula.
    ///
    /// ⚠️ **Alocado só com o knob LIGADO** ([`SolverScratch::ensure_backrun`]),
    /// e é por isso que ele tem porta própria: `extBackrun` é `Hidden` e nasce
    /// em `0.0`, então a sessão default **não paga um byte** por um plano que
    /// nenhum passe leria. O irmão [`SolverScratch::ensure`] cobre os planos
    /// que todo passo materializa.
    pub bflags: Vec<u8>,
    /// A altura e o peso da ladeira do óleo ([`crate::solver::ladeira_do_oleo`]), nas linhas da bbox × `s`; só as
    /// janelas lidas são escritas a cada passe.
    pub ladeira_h: Vec<f32>,
    pub ladeira_assente: Vec<f32>,
    /// A janela de colunas de cada linha fina que o passe de fluxo lê da ladeira.
    pub ladeira_jan: Vec<(i32, i32)>,
    /// Os tiles de 64 de cada linha fina que a janela cobre (B13.2: duas manchas nas mesmas linhas).
    pub ladeira_jan_tiles: Vec<u64>,
}

/// A cópia de um rascunho é um rascunho VAZIO: o conteúdo é reescrito inteiro dentro de um passe, e o
/// primeiro passo da cópia volta a alocá-lo ([`Self::ensure`]). É um terço do instantâneo de uma macro
/// (medido, `docs/Macros/02_plano_W5.md` §3).
impl Clone for SolverScratch {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl SolverScratch {
    /// O [`Self::bt`] de quem não puxa nada.
    pub const BT_NADA: i8 = i8::MIN;

    /// Garante os planos com `n` células. Idempotente e sem realocar depois da
    /// primeira vez. Todo plano nasce a ZEROS (`alloc_zeroed`, nenhum byte escrito): cada passe escreve a faixa
    /// viva que lê — o `bt` incluído, que o `prepare_rows` escreve antes de o `outflow_rows` o ler (B12.4).
    pub fn ensure(&mut self, n: usize) {
        if self.outflow.len() != n {
            self.uv = vec![[0.0; 2]; n];
            self.outflow = vec![0.0; n];
            self.bt = vec![0; n];
            self.dst_film = vec![0.0; n];
            self.dst_susp = vec![0.0; n];
            self.dst_rgb = vec![[0.0; 3]; n];
            self.edge = vec![0; n];
            #[cfg(test)]
            veneno::aplica(self, false);
        }
    }

    /// Garante o plano do backrun. Chamado só quando o knob está ligado.
    pub fn ensure_backrun(&mut self, n: usize) {
        if self.bflags.len() != n {
            self.bflags = vec![0; n];
            #[cfg(test)]
            veneno::aplica(self, true);
        }
    }
}

/// B12.4: o rascunho ENVENENADO — com [`veneno::LIGADO`] no fio que dá o passo, cada plano que o `ensure` aloca
/// nasce a lixo (NaN, bytes ≠ 0) em vez de zeros: um passe que lê o que não escreveu muda a grade.
#[cfg(test)]
pub(crate) mod veneno {
    use super::SolverScratch;
    use std::cell::Cell;

    thread_local! {
        pub(crate) static LIGADO: Cell<bool> = const { Cell::new(false) };
        /// Quantos `ensure` envenenaram neste fio (o gate prova que o veneno chegou ao passo).
        pub(crate) static APLICADOS: Cell<usize> = const { Cell::new(0) };
    }

    const NAN: f32 = f32::from_bits(0x7fa5_a5a5);

    pub(super) fn aplica(s: &mut SolverScratch, backrun: bool) {
        if !LIGADO.get() {
            return;
        }
        APLICADOS.set(APLICADOS.get() + 1);
        if backrun {
            s.bflags.fill(0xa5);
            return;
        }
        s.uv.fill([NAN; 2]);
        s.outflow.fill(NAN);
        s.bt.fill(3);
        s.dst_film.fill(NAN);
        s.dst_susp.fill(NAN);
        s.dst_rgb.fill([NAN; 3]);
        s.edge.fill(0xa5);
    }
}

#[cfg(test)]
#[path = "scratch_tests.rs"]
mod tests;
