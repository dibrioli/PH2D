//! **A ESPONJA da borda absorvente** — o perfil que faz uma onda morrer antes da parede em vez
//! de ricochetear (doc 89, folha 06, célula 36 · o *Reflect Edges* do AE Wave World).
//!
//! Cortada do `lib.rs` no teto de LOC (HR-18), e o corte é por RESPONSABILIDADE: ali mora a lei
//! do passo e o que o nó É, aqui *quanto de amplitude cada célula guarda por tique*.

/// **O PERFIL da camada absorvente** — a largura da esponja e a mordida dela.
///
/// ⚠️ **Ele é um valor e não duas constantes soltas porque os dois números têm de
/// ser VARRIDOS JUNTOS:** uma esponja estreita e forte e uma larga e fraca tiram a
/// mesma energia total e reflectem quantidades muito diferentes, então medir um com
/// o outro fixo mede a combinação e não a lei. A sonda `sweeps_the_sponge_profile`
/// varre a grade dos dois.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Sponge {
    /// A largura da camada, em CÉLULAS.
    pub(crate) cells: f32,
    /// Quanto da amplitude a parede tira por tique, no limite dela.
    ///
    /// ⚠️ **NÃO é `1,0`, e a razão é física e não gosto:** uma máscara que cai a
    /// zero de repente é ela própria um reflector — uma mudança abrupta de
    /// impedância reflecte, que é o fenómeno que esta borda existe para negar. A
    /// esponja é GRADUADA por isso, e o piso fica abaixo de 1 para a última fila
    /// ainda propagar alguma coisa em vez de virar uma segunda parede.
    pub(crate) strength: f32,
}

impl Sponge {
    /// O que SHIPA. ⚠️ **MEDIDO, não escolhido** (`sweeps_the_sponge_profile`, o ECO
    /// que volta ao miolo em % do que uma parede reflectora devolve):
    ///
    /// ```text
    ///   cells\str    0.02    0.05    0.10    0.15    0.25    0.50    1.00
    ///          2   55.81%  45.36%  37.95%  30.10%  31.54%  37.49%  46.49%
    ///          4   54.45%  40.98%  32.64%  30.37%  29.69%  36.67%  40.89%
    ///          6   50.72%  44.00%  27.77%  25.99%  32.30%  33.97%  40.16%
    ///          8   45.87%  41.63%  25.45%  28.83%  31.32%  36.44%  31.93%
    ///         12   39.09%  25.23%  27.63%  25.06%  30.05%  28.65%  29.26%
    /// ```
    ///
    /// ⭐ **A `strength` tem um U, e as duas pontas são ruins por razões DIFERENTES:**
    /// fraca demais mal absorve (`0,02` devolve metade), forte demais reflecte na
    /// própria escada de impedância (`1,00` é a pior coluna em toda a metade estreita).
    /// *A física escrita em [`Sponge::strength`] estava certa e agora está MEDIDA.*
    ///
    /// ⭐ **A largura paga até `6` e depois é ruído** (`6`→`12` vale ~1 ponto), e cada
    /// célula de esponja é miolo que o artista deixa de poder usar ⇒ o joelho é `6`.
    ///
    /// ⚠️ **LIMITE NOMEADO: isto reduz o eco a ~26 %, não a zero.** Uma fronteira
    /// verdadeiramente não-reflectora é a condição de radiação (Mur), que é a outra
    /// família nomeada em [`step`] — ela troca a VIZINHANÇA no bordo, não a amplitude.
    pub(crate) const SHIPPED: Self = Self {
        cells: 6.0,
        strength: 0.15,
    };

    /// **Quanto da amplitude a célula `(r, c)` guarda por tique** — `1` no miolo,
    /// caindo para dentro da parede.
    ///
    /// A queda é QUADRÁTICA na distância à parede mais próxima: é o perfil clássico
    /// de camada absorvente, e é o que faz a transição do miolo para a esponja ser
    /// suave o bastante para não reflectir de volta.
    pub(crate) fn at(self, r: usize, c: usize, rows: usize, cols: usize) -> f32 {
        // A distância à parede mais PRÓXIMA, em células: o mínimo das quatro.
        let dr = r.min(rows.saturating_sub(1).saturating_sub(r));
        let dc = c.min(cols.saturating_sub(1).saturating_sub(c));
        let d = dr.min(dc) as f32;
        if d >= self.cells || self.cells <= 0.0 {
            return 1.0;
        }
        let into = 1.0 - d / self.cells;
        1.0 - self.strength * into * into
    }
}
