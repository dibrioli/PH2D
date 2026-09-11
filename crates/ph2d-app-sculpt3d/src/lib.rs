//! **A família `sculpt3d` a sair da shell** — a semente (W2/L3, 2026-09-11).
//!
//! Esta crate é o destino da família `sculpt3d`, que vivia inteira em
//! `shells/desktop/src/` (111 ficheiros, 31 847 LOC) e é hoje UMA pasta
//! (`shells/desktop/src/sculpt3d/`). A Fase A traz para cá **só o que já não precisa da
//! shell**; o corte do resto é a Fase B, pelo molde da `line/app-host`.
//!
//! ⚠️ **Porque é que a shell depende disto SEM `optional`** — e não é detalhe de
//! empacotamento: a `App` declarava, com a razão escrita ao lado de cada campo, que os
//! pedidos da escultura **não** levam `#[cfg(feature = "sculpt3d")]`. O motivo mais forte é
//! o [`Sculpt3dRequests::doc`]: um binário construído sem a escultura tem de ser um
//! **passa-adiante** dos bytes de um documento já gravado — carregá-los do load ao save sem
//! os ler — e não um triturador. Uma dependência opcional apagaria o campo nesse binário e
//! o artista perderia a escultura ao gravar. ⇒ esta crate tem **zero dependências** e é
//! barata o suficiente para toda a gente a pagar.
//!
//! O que **não** veio, e porquê (a lista que a Fase B fecha):
//! - `sculpt3d_pending`, `sculpt3d_rows`, `sculpt3d_dup`, `sculpt3d_sel` guardam tipos do
//!   módulo (`LoadedPiece`, `SculptRowsSeen`) que ainda vivem na shell. Eles estão
//!   agrupados do outro lado, num `Sculpt3dShellState` gateado que atravessa a fronteira
//!   inteiro quando aqueles tipos mudarem de casa.

/// **AS RÉGUAS DO GESTO** — quanto um pixel de arrasto vale, e onde uma grandeza deixa de
/// existir. Lei pura, zero dependências; era `shells/desktop/src/sculpt3d_rulers.rs`.
pub mod rulers;

pub use rulers::*;

/// **O QUE A ESCULTURA PEDE AO LAÇO DO QUADRO** — os cinco campos que a `App` guardava
/// soltos, num sítio só (ADR-0075).
///
/// ⚠️ **Todos são PEDIDO, nunca ação**, e a razão é a mesma nos quatro `bool`: quem os arma
/// (uma tecla, um pill, uma linha de painel) tem a cena emprestada, e quem os cumpre precisa
/// do `device`, do `sim`, do renderizador e do mapa de atlas — que só existem dentro do laço
/// de quadro. Armar aqui e drenar lá é o que os mantém honestos.
///
/// ⚠️ **Nenhum deles é um símbolo do módulo 3D** — são `bool` e bytes opacos —, e é por isso
/// que esta struct pode viver fora da feature. Um campo novo que traga um tipo do módulo
/// **não pertence aqui**: ele vai para o `Sculpt3dShellState` gateado do lado da shell, ou
/// espera a Fase B.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Sculpt3dRequests {
    /// A tela branca da cena `PH2D_SCULPT3D_SMOKE=2` (a DOAÇÃO) já foi montada.
    ///
    /// Latch de uma vez só: o laço lê-o com `mem::replace` e a cena nasce no primeiro quadro
    /// com GPU.
    pub canvas_done: bool,
    /// **O gesto de ASSAR pediu** (`docs/3D/02.2`) — armado pela tecla, drenado pelo laço.
    pub bake_request: bool,
    /// **O pedido de usar o sprite selecionado como PADRÃO do pincel.** Irmão do
    /// [`Self::bake_request`]: ler os pixels de um sprite precisa do mundo, do renderizador e
    /// do mapa de atlas, e os três só estão em escopo dentro do laço de quadro.
    pub alpha_request: bool,
    /// **O pill SCULPT pediu para ENTRAR ou SAIR do modo escultura** (ADR-0150).
    ///
    /// ⚠️ Aqui a razão de ser um pedido é a mais forte das quatro: entrar pode ter de **criar
    /// a cena**, o que exige o `device` e o tamanho da superfície — os dois só existem depois
    /// de a janela nascer.
    pub toggle_request: bool,
    /// **O documento de escultura como veio do arquivo**, em bytes opacos.
    ///
    /// ⛔⛔ **É ele que proíbe esta crate de ser uma dependência opcional.** Num binário
    /// construído SEM a feature `sculpt3d` ninguém o lê — e é isso que o torna um
    /// **passa-adiante**: os bytes de uma escultura gravada atravessam o load e voltam ao
    /// save intactos, em vez de serem descartados em silêncio. Com o módulo ligado ele é a
    /// fonte do save enquanto a cena não existir (projeto aberto antes de a GPU aparecer).
    pub doc: Vec<u8>,
}

impl Sculpt3dRequests {
    /// Lê e **desarma** o pedido de assar — a forma que o laço do quadro usa.
    ///
    /// ⚠️ Um `take_*` e não um `bool` público lido à mão: um pedido que se lê sem se desarmar
    /// cumpre-se a cada quadro enquanto ninguém o limpar, e isso lê-se como *«o botão assou
    /// sozinho»*.
    pub fn take_bake(&mut self) -> bool {
        std::mem::take(&mut self.bake_request)
    }

    /// Lê e desarma o pedido do alpha. Irmão do [`Self::take_bake`], mesma razão.
    pub fn take_alpha(&mut self) -> bool {
        std::mem::take(&mut self.alpha_request)
    }

    /// Lê e desarma o pedido do pill. Irmão do [`Self::take_bake`], mesma razão.
    pub fn take_toggle(&mut self) -> bool {
        std::mem::take(&mut self.toggle_request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O estado nasce inerte** — e isto é o que mantém a promessa do módulo: sem ninguém
    /// pedir nada, o laço do quadro não tem trabalho de escultura para fazer.
    #[test]
    fn nothing_is_requested_before_anyone_asks() {
        let r = Sculpt3dRequests::default();
        assert!(!r.canvas_done && !r.bake_request && !r.alpha_request && !r.toggle_request);
        assert!(r.doc.is_empty());
    }

    /// **Um pedido cumpre-se UMA vez.** ⚠️ O controlo é a segunda leitura: um `take` que
    /// devolvesse `true` duas vezes faria o laço assar a cada quadro, que é o defeito que
    /// estes três métodos existem para tornar inexprimível.
    #[test]
    fn a_request_is_honoured_once_and_then_it_is_gone() {
        let mut r = Sculpt3dRequests {
            bake_request: true,
            alpha_request: true,
            toggle_request: true,
            ..Default::default()
        };
        assert!(r.take_bake() && r.take_alpha() && r.take_toggle());
        assert!(
            !r.take_bake() && !r.take_alpha() && !r.take_toggle(),
            "um pedido lido sem ser desarmado cumpre-se a cada quadro"
        );
    }

    /// ⛔⛔ **O passa-adiante**: os bytes de um documento atravessam quem não os lê.
    ///
    /// Este teste corre numa crate que **não conhece** o módulo 3D — é essa a prova. Ele não
    /// mede uma função nossa; mede que o *lugar* onde o documento espera não depende da
    /// feature. Se alguém tornar esta crate `optional` na shell, o campo desaparece do
    /// binário sem escultura e uma escultura gravada é descartada no save seguinte.
    #[test]
    fn the_document_bytes_survive_a_build_that_never_reads_them() {
        let bytes = vec![7u8, 3, 9, 1];
        let mut r = Sculpt3dRequests {
            doc: bytes.clone(),
            ..Default::default()
        };
        // O ciclo de um binário sem escultura: nada lê o documento, tudo o resto é drenado.
        let _ = (r.take_bake(), r.take_alpha(), r.take_toggle());
        assert_eq!(
            r.doc, bytes,
            "os bytes da escultura não sobreviveram ao quadro"
        );
    }
}
