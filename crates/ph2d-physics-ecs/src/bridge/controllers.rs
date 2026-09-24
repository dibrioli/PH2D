//! **UM TIQUE DOS CONTROLADORES — a porta única, e a razão de ela existir.**
//!
//! Report do dono, 2026-09-15: *«o Rewind não está funcionando com os projéteis. Eles têm um
//! comportamento diferente a cada rewind»*.
//!
//! # ⛔⛔ O defeito era um PAR de laços escritos à mão em dois sítios
//!
//! Esta ponte anda o relógio por **dois** caminhos — o laço da frente
//! ([`super::dispatch`]) e o laço de replay ([`super::rewind`]) — e cada controlador novo era
//! ligado ao primeiro. O `drive_players` chegou aos dois porque a W7 foi buscá-lo depois de um
//! report; o `drive_topdown` (TOP-20 #13) e o `drive_projectiles` (TOP-20 #14) nunca chegaram ao
//! segundo. ⇒ **um scrub para trás replayava um mundo onde aqueles corpos não se mexem**, e a
//! pergunta que o `bridge::tape` declara — *o mundo é função de `(tique, repouso, curvas, fita)`* —
//! passava a ter duas respostas conforme o botão que o artista tinha carregado.
//!
//! ⚠️ **É a MESMA forma do report anterior desta linha** (*«nada se move»*, 2026-09-15): uma
//! pergunta escrita em duas metades aceita a variante nova em só uma, **em silêncio**. Ali a cura
//! foi a porta [`crate::reads_the_keyboard`]; aqui é esta.
//!
//! ⇒ **Um quarto controlador não pode ser ensinado a meio par**: ele entra nesta função, e os dois
//! laços recebem-no por construção. O censo
//! `os_dois_lacos_dirigem_os_controladores_pela_mesma_porta` defende a propriedade.
//!
//! # ⚠️ A ORDEM é a lei, e é a do laço da frente
//!
//! `plataforma → vista de cima → projéctil`, e os três **antes** do `step`: é isso que faz o solver
//! tratar os corpos como movendo-se. ⛔ A ordem não é arbitrária — o mover de plataforma e o de
//! vista de cima podem viver na mesma entidade e ali o primeiro **ganha** (a lei transversal
//! *«um dono do transform por vez»*, decidida dentro do `drive_topdown`).

use ph2d_ecs::SimWorld;

use super::PhysicsBridge;

impl PhysicsBridge {
    /// **Um tique de TODOS os controladores.** Ver o cabeçalho do módulo.
    ///
    /// ⚠️ Chamada pelos **dois** laços que andam o relógio, e é esse o ponto: um deles sozinho não
    /// é o produto.
    pub(super) fn drive_controllers(&mut self, sim: &SimWorld) {
        // ⭐ O canal dos toques de mover é deste TIQUE: os três movers abaixo enchem-no, e a vida
        // lê-o depois do passo (plano 28 §8.1).
        self.toques_do_mover.clear();
        // Os PLAYERS (W2): o sensor pergunta ao BVH que o step ANTERIOR deixou e a mola escreve o
        // motor deste tique.
        self.drive_players(sim);
        // E os movers de VISTA DE CIMA (TOP-20 #13), no MESMO tique: os dois leem a mesma entrada
        // e os dois escrevem a pose antes do `step`.
        self.drive_topdown(sim);
        // E os PROJÉCTEIS (TOP-20 #14), pela mesma razão.
        self.drive_projectiles(sim);
    }

    /// ⭐⭐⭐ **O que responde AO passo** (plano 28 §8.2) — a porta irmã desta, chamada pelos DOIS
    /// laços **depois** do `step`, pela mesma razão que a de cima existe antes dele: um assunto
    /// ensinado a um laço só é um scrub que devolve outra corrida.
    ///
    /// Hoje é a VIDA. `publicar = false` no replay: o estado anda, os factos não saem.
    pub(super) fn depois_do_passo(&mut self, sim: &SimWorld, publicar: bool) {
        self.drive_health(sim, publicar);
    }
}
