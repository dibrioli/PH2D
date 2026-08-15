//! **A FITA — o traço PESA** (plano 38 W6, pesquisa no
//! [doc 37 §3.6](../../../../docs/Painter/37_pesquisa_tracos_procedurais.md)).
//!
//! Alchemy *Ribbon Shapes* (*"Leaves a trail of ribbon like shapes"*, controles `Size · Spacing ·
//! Friction · Gravity`) e Krita *Dyna* (*massa e arrasto*): a tinta é uma **massa presa ao cursor
//! por uma mola**, com atrito e peso. Ela atrasa na curva, chicoteia na saída e pende.
//!
//! ## Onde ela mora, e por que NÃO é o [`crate::stroke::Stroke::throw`]
//!
//! ⚠️ **O `Speed` move a TINTA e deixa o CAMINHO intacto; a fita move o CAMINHO.** A distinção não
//! é gosto — é o que cada lei precisa para não se realimentar:
//!
//! - O arremesso é `velocidade × antecipação`, e a velocidade é medida **do caminho**. Se ele
//!   movesse o caminho, a velocidade se somaria a si mesma e o traço fugiria da tela por composição.
//! - A fita é um **filtro passa-baixa** do caminho, exatamente como o estabilizador
//!   ([`mod@super::stabilize`]). Realimentar um passa-baixa é estável por construção: a saída
//!   persegue a entrada e nunca a ultrapassa em regime.
//!
//! É por isso que ela custa tão pouco: para o espaçamento, para os fios, para o preenchedor de vão,
//! para a Symmetry, para o Tiling e para o Spray, **a fita É o traço**. Nenhum deles sabe que ela
//! existe.
//!
//! ## Ela NÃO é um segundo estabilizador
//!
//! ⚠️ Os dois atrasam a linha, e a pergunta *"então não são duas portas para a mesma coisa?"* tem
//! resposta MEDÍVEL — **mas não a que eu escrevi primeiro.** A versão original desta nota dizia que
//! *"o atraso do estabilizador não depende da velocidade"*, e o CONTROLE do gate a derrubou: ele
//! depende (**50,4 → 386,4 px** para 8× a velocidade), porque um lag de primeira ordem em regime
//! também vale `v · τ`.
//!
//! **As duas distinções que sobrevivem à medição, cada uma com gate:**
//!
//! 1. **Ela ULTRAPASSA.** O estabilizador é uma média corrida e converge por baixo — nunca passa do
//!    alvo, com nenhuma intensidade. A fita tem massa: com `ζ < 1` ela passa e volta, e esse trecho
//!    é o que o artista lê como *chicote*
//!    (`the_ribbon_overshoots_the_stop_and_the_stabilizer_never_does`).
//! 2. **Ela é fato do RELÓGIO.** O estabilizador filtra por EVENTO, então a mão dele muda com a taxa
//!    do dispositivo; a fita é integrada em segundos e um mouse de 960 Hz desenha o que um de 125 Hz
//!    desenha (`the_ribbon_is_a_fact_of_the_clock_not_of_the_pointer_rate`, que carrega o
//!    estabilizador como controle **negativo**).
//!
//! Um filtra TREMOR, a outra dá PESO; compõem em série sem se contradizer.
//!
//! ## O relógio
//!
//! ⚠️ **A mola é integrada no TIQUE, e o caminho é percorrido no tique** — não no evento de
//! ponteiro. É a mesma lei que o [`mod@super::speed`] aplica à velocidade: *a grandeza é fato do
//! CAMINHO e do RELÓGIO, nunca de quão fino o dispositivo amostrou o caminho*. Um mouse de 960 Hz e
//! um de 125 Hz entregam a mesma fita, porque quem a move é o relógio de parede.

use super::*;
use crate::line_kind::{
    RIBBON_MAX_STEP_S, RIBBON_MAX_SUBSTEPS, RIBBON_SUBSTEP_FRACTION, RIBBON_TAIL_MAX_S,
    RIBBON_TAIL_REST_PX_S,
};

impl Stroke {
    /// Integra a ponta da fita um quadro de `dt` e devolve `true` se ela se moveu o bastante para
    /// valer um passo de caminho.
    ///
    /// ⚠️ **Ela persegue o `last_raw_pos`, que é a amostra já ESTABILIZADA?** Não: é a amostra
    /// média do `sampler`, crua. O estabilizador é uma etapa IRMÃ (o `stabilize` escreve o
    /// `stab_pos`, que a fita não lê) — encadeá-los faria a fita perseguir um alvo que já atrasa, e
    /// os dois atrasos se somariam sem ninguém ter pedido. Com a fita armada o caminho é o dela.
    pub(super) fn step_ribbon(&mut self, dt: f32, leashed: bool) {
        if dt <= 0.0 {
            return;
        }
        let tau = self.spec.ribbon_lag_s();
        if tau <= 0.0 {
            return;
        }
        // ω = 1/τ é a rigidez da mola; c = 2ζω o amortecimento. A forma normalizada é o que torna os
        // dois knobs ORTOGONAIS: `weight` diz QUANTO TEMPO, `friction` diz COMO ASSENTA.
        let w = 1.0 / tau;
        // ⚠️ **A COLEIRA é cortada no pen-up, e é o que separa uma fita de um gancho.** Enquanto a
        // mão desenha, a mola puxa a ponta para o cursor (`leashed`). No pen-up a mão SOLTOU: o
        // termo de mola sai e ficam a inércia, o atrito e a gravidade — a ponta segue na direção em
        // que ia, desacelera e assenta. É o *follow-through* da animação clássica, e é o que o
        // Alchemy (a forma termina no release) e o Dyna do Krita (a massa é solta) fazem.
        //
        // ⚠️ **Sem isto a cauda desenha uma ESPÍCULA, e não é opinião — foi medido:** uma mola
        // CONVERGE para o alvo, e com o alvo já parado no dedo essa convergência é uma **reta**.
        // Report do Enio (2026-08-15, com foto): a cauda levava a tinta de `x = 947,2` a
        // `x = 1316,8` — **369 px em 154 dabs**, em linha reta, largura cheia, atravessando o
        // desenho. Uma por traço. O doc abaixo argumentava contra *"um salto até o cursor"* e a
        // física chegava ao mesmo lugar pelo outro caminho.
        let k = if leashed { w * w } else { 0.0 };
        let c = 2.0 * self.spec.ribbon_damping() * w;
        let g = self.spec.ribbon_gravity_px_s2();
        let target = self.last_raw_pos;
        // ⚠️ **SUB-PASSOS, e é estabilidade e não capricho:** o Euler semi-implícito de uma mola só é
        // estável enquanto `ω·h` é pequeno, e `ω` cresce sem limite quando o peso encosta no zero.
        // Um quadro lento (GC, redimensionar, breakpoint) entrega um `dt` grande, e sobre uma mola
        // rígida a fita explode para fora da tela — com todo gate de unidade verde, porque a unidade
        // nunca vê um quadro de 200 ms.
        //
        // ⚠️ **O teto é do TRABALHO, NUNCA da resolução** — a lei do `ph2d_core::time::FixedStep`, e
        // a primeira versão disto fazia o contrário: ela capava `n` e deixava `h = dt/n` crescer, o
        // que **desfaz** a garantia da linha acima. Medido: com `n` capado em 32 onde 160 eram
        // precisos, `ω·h` ia de 0,25 a 1,25 e o maior autovalor a **1,7586 > 1** ⇒ `1e78` em dez
        // quadros, 90 GB de RAM e o editor no chão. Ver [`RIBBON_MAX_STEP_S`].
        //
        // Capar o `dt` custa TEMPO DE FÍSICA (a fita atrasa um pouco mais atravessando a engasgada),
        // que é exatamente o trade que o `FixedStep` nomeia — e nunca precisão.
        let dt = dt.min(RIBBON_MAX_STEP_S); // (NaN cai aqui para o teto: `f32::min` devolve o outro)
        let dt_max = RIBBON_SUBSTEP_FRACTION * tau;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((dt / dt_max).ceil() as usize).max(1);
        // ⚠️ **Um BATENTE, não um regulador.** Se ele morder, a `ω·h = 0,25` deixou de valer — o
        // gate `the_substep_ceiling_can_never_bind` prova que a aritmética das três consts o torna
        // inalcançável, e existe para que mover uma delas fique VERMELHO em vez de divergir.
        let n = n.min(RIBBON_MAX_SUBSTEPS);
        #[allow(clippy::cast_precision_loss)]
        let h = dt / n as f32;
        for _ in 0..n {
            let ax = (target[0] - self.ribbon_pos[0]) * k - self.ribbon_vel[0] * c;
            // ⚠️ `+g` porque o canvas é Y-PARA-BAIXO (coordenada de texel), como todo raster deste
            // módulo. Um sinal trocado aqui faz a fita subir, e nenhum gate de atraso o vê.
            let ay = (target[1] - self.ribbon_pos[1]) * k - self.ribbon_vel[1] * c + g;
            self.ribbon_vel[0] += ax * h;
            self.ribbon_vel[1] += ay * h;
            self.ribbon_pos[0] += self.ribbon_vel[0] * h;
            self.ribbon_pos[1] += self.ribbon_vel[1] * h;
        }
    }

    /// O passo de caminho de um quadro: integra e percorre até onde a ponta chegou.
    ///
    /// ⚠️ **Chamado do [`Stroke::tick`], e é ele o ÚNICO que percorre** num traço de fita — o
    /// `extend` só grava onde o dedo está. Isto vale para a mão PARADA tanto quanto para a mão em
    /// movimento, e é o que dá a segunda metade da feature de graça: solte o gesto no ar e a fita
    /// continua a chegar, porque o tique não parou.
    pub(super) fn tick_ribbon(&mut self, dt: f32, out: &mut Vec<Dab>) {
        self.step_ribbon(dt, true);
        self.walk_smoothed(
            StrokePoint {
                pos: self.ribbon_pos,
                pressure: self.last_raw_pressure,
            },
            out,
        );
        // Um quadro parado ainda é onde o heading pode assentar (fita pesada), então as aberturas
        // seguram pelo MESMO portão do `settle` — sem isto a abertura nunca é solta.
        self.warmup_gate(out);
    }

    /// **A CAUDA** — no pen-up a mão soltou e a fita ainda tem inércia; a física corre até ela
    /// assentar, e o traço termina onde a fita de facto parou.
    ///
    /// ⚠️ **Ela NÃO é encerrada por um salto até o cursor**, ao contrário do estabilizador (cujo
    /// `finish` percorre em linha reta até `last_raw_pos`, *"para o traço acabar exactamente onde a
    /// caneta levantou"*). Numa fita esse salto seria um artefacto: a tinta **deve** ficar atrás.
    ///
    /// ⚠️ **E a 1ª versão desta cauda chegava ao mesmo gancho pelo outro caminho.** Ela rodava a
    /// física com a MOLA ainda presa ao cursor, e uma mola **CONVERGE** para o alvo — com o alvo
    /// parado no dedo, convergir é andar em **linha reta** até ele. Medido no report do Enio
    /// (2026-08-15, com foto): **369 px em 154 dabs**, largura cheia, uma espícula por traço
    /// atravessando o desenho. *Argumentar contra o salto não impede a física de o desenhar.*
    ///
    /// **Hoje o pen-up CORTA A COLEIRA** ([`Stroke::step_ribbon`] com `leashed = false`): ficam a
    /// inércia, o atrito e a gravidade. A ponta segue na direção em que ia, desacelera e assenta —
    /// o *follow-through* da animação clássica, e o que o Alchemy (a forma termina no release) e o
    /// Dyna do Krita (a massa é solta) fazem. Medido, o mesmo gesto: **156 px**, e a tinta **para
    /// antes do dedo** em vez de o ultrapassar.
    ///
    /// O teto de tempo existe porque um `ζ` pequeno balança por muito tempo, e um traço não pode
    /// continuar a crescer depois de o artista o ter terminado.
    pub(super) fn finish_ribbon(&mut self, out: &mut Vec<Dab>) {
        let dt = 1.0 / 60.0; // o passo do relógio da cauda: um quadro nominal
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let steps = (RIBBON_TAIL_MAX_S / dt) as usize;
        for _ in 0..steps {
            self.step_ribbon(dt, false);
            let speed =
                (self.ribbon_vel[0] * self.ribbon_vel[0] + self.ribbon_vel[1] * self.ribbon_vel[1])
                    .sqrt();
            // `walk_smoothed` ACRESCENTA (é o `walk_space` que empurra), então a cauda inteira cai
            // no mesmo buffer que o chamador já limpou.
            self.walk_smoothed(
                StrokePoint {
                    pos: self.ribbon_pos,
                    pressure: self.last_raw_pressure,
                },
                out,
            );
            if speed <= RIBBON_TAIL_REST_PX_S {
                break;
            }
        }
    }
}
