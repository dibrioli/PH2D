//! **O ESTADO DE PEDIDO da escultura** — o que ela pede ao laço do quadro.
//!
//! ⚠️ **Ele estava no `lib.rs` até 2026-09-11 (W2/L3-B)** e saiu quando a família inteira entrou
//! na crate: o `lib.rs` passou a ser o módulo que hospeda a [`Sculpt3dScene`] e os ~30 irmãos.

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

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⭐⭐ **A Fase B trouxe o roteador, e `"sculpt3d"` SAIU da catraca**
/// `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` (2026-09-11). As ~15 cenas `scenes_*.rs`
/// atravessaram com os outros 111 ficheiros, e com elas os 37 `env::var` que escolhem o nível:
/// a família lê hoje a env que declara.
///
/// ⚠️⚠️ **O `max_level` é [`crate::scenes::CENAS`], CONTADO** — ver o doc dele e o gate que o
/// mede. ⛔ Escrever aqui o número seria a terceira cópia de uma grandeza que já tem duas.
///
/// ⚠️⚠️ **UM roteador, e não trinta e dois.** Esta família lê ~32 variáveis `PH2D_*`, e a
/// esmagadora maioria é **DIAGNÓSTICO de retopologia** (`PH2D_RETOPO_*`, `PH2D_DUMP*`,
/// `PH2D_BENCH_*`, `PH2D_TIP_ALIGN`, `PH2D_ISO_*`, `PH2D_GRIDMAP_*`) — interruptores de
/// bissecção que ninguém do lado do dono alcança e que não roteiam cena nenhuma. Só a forma
/// `PH2D_*_SMOKE` entra aqui, e o gate do registo afirma-o pela forma do nome. *Declarar um
/// `PH2D_RETOPO_LEGACY` como roteador diria ao dono que ele tem uma cena para ver.*
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "sculpt3d",
    routers: &[ph2d_app_host::SmokeRouter {
        env: "PH2D_SCULPT3D_SMOKE",
        max_level: crate::scenes::CENAS,
    }],
};

#[cfg(test)]
mod tests {
    use super::*;

    /// **O estado nasce inerte** — e isto é o que mantém a promessa do módulo: sem ninguém
    /// pedir nada, o laço do quadro não tem trabalho de escultura para fazer.
    #[test]
    fn nothing_is_requested_before_anyone_asks() {
        let r = Sculpt3dRequests::default();
        assert!(!r.canvas_done && !r.bake_request && !r.alpha_request && !r.toggle_request);
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
}
