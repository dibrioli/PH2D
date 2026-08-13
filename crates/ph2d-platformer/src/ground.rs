//! **O que o CHÃO contribui** — as duas grandezas que toda a lei deriva da
//! amostra do sensor, antes de qualquer decisão.
//!
//! Módulo irmão do [`crate`] (o teto de LOC do arquivo), e o corte é por
//! ASSUNTO: o `lib.rs` fica com os TIPOS e a PORTA que soma as leis; aqui
//! moram as duas perguntas que se fazem ao chão — *quão depressa subo em
//! relação a ele?* e *quanto do movimento dele ainda me é devido?*.
//!
//! ⚠️ As duas são `pub` e re-exportadas na raiz: **nenhum caminho de chamador
//! muda** com esta mudança de casa.

use crate::{GroundSample, Vec2, perp_cw};

/// **A velocidade de SUBIDA relativa ao chão** — o número em que quase toda esta
/// lei ramifica (o pouso, as fases da gravidade, a porta do sensor de teto).
///
/// ⚠️ **Porta única, e a W10 é quem a exigiu:** a ponte precisa da MESMA grandeza
/// para decidir se casta os raios da quina ([`corner_probe_wanted`]), e a
/// tentação era ela ler `velocidade · up` direto — igual **enquanto** o probe só
/// existir no ar, onde a velocidade do chão é zero. Uma premissa verdadeira por
/// acidente de escopo é exatamente a que envelhece: bastaria um dia oferecer a
/// assistência de pé numa plataforma que sobe.
#[must_use]
pub fn relative_rise(footing: Option<&GroundSample>, body_velocity: Vec2, up: Vec2) -> f32 {
    let g = footing.map_or([0.0, 0.0], |s| s.ground_velocity);
    (body_velocity[0] - g[0]) * up[0] + (body_velocity[1] - g[1]) * up[1]
}

/// **O QUE O CHÃO AINDA DEVE** — a parte da velocidade do chão que a lei da
/// caminhada **não** põe na velocidade do personagem (K7).
///
/// # ⚠️ Porque isto existe: a plataforma era contada DUAS vezes
///
/// A [`walk`] mede tudo *relativo ao chão* (`rel = v − ground_v`) e empurra
/// `rel_along` até o alvo, então parado sobre um vagão ela leva `body_velocity`
/// **até a velocidade do vagão** — a tração é o modelo de transporte deste
/// motor. O integrador cinemático então somava `ground_velocity` outra vez, e a
/// medição é inequívoca (`tests/measure_kinematic_carry.rs`, vagão a 2 m/s por
/// 4,00 m):
///
/// | eixo | modo | tração | levado |
/// |---|---|---|---|
/// | horizontal | dinâmico | cheia | 3,95 m (**0,99×**) |
/// | horizontal | dinâmico | **zero** | 0,00 m (**0,00×**) |
/// | horizontal | CINEMÁTICO | cheia | 7,92 m (**1,98×**) |
/// | horizontal | CINEMÁTICO | **zero** | 3,97 m (0,99×) |
/// | vertical | qualquer | qualquer | ~4,00 m (1,00×) |
///
/// ⚠️ **A linha `dinâmico / zero` é a que fecha a atribuição:** desligada a
/// tração pela porta do ARTISTA (`PlatformPlayer::acceleration`), o modo
/// dinâmico não é levado **de todo** — logo quem o carrega é a caminhada, e só
/// ela. As duas somam 1,98×; o eixo vertical nunca teve o problema porque o
/// eixo da caminhada é a TANGENTE e ela não toca a normal.
///
/// # A lei
///
/// O que falta é `g` menos a projeção dele no **MESMO eixo** que a [`walk`]
/// usa. Perguntar pelo mesmo `perp_cw(normal)` — em vez de re-derivar de `up` —
/// é o que impede as duas metades de discordarem sobre *o que a tração cobre*.
///
/// ⚠️ **A representação apaga o caso degenerado:** com a normal `[0, 0]` (raio
/// nascido dentro da geometria) o eixo é `[0, 0]`, a caminhada não cobre nada, e
/// esta função devolve `g` INTEIRO — que é a resposta certa, sem um `if`.
///
/// ⚠️ **Sem tração o chão não leva de lado, e isso é físico:** um elevador
/// empurra pelo CONTATO (componente normal, sempre paga) e uma esteira leva por
/// ATRITO (componente tangente, que a caminhada modela). O modo dinâmico já se
/// comportava assim — esta lei é o que faz o cinemático concordar com ele.
#[must_use]
pub fn ground_carry(footing: Option<&GroundSample>) -> Vec2 {
    let Some(s) = footing else {
        return [0.0, 0.0];
    };
    let g = s.ground_velocity;
    // O eixo da caminhada, verbatim (`walk.rs`): a normal é unitária por
    // contrato do sensor, e é essa premissa que a projeção herda.
    let axis = perp_cw(s.normal);
    let along = g[0] * axis[0] + g[1] * axis[1];
    [g[0] - axis[0] * along, g[1] - axis[1] * along]
}
