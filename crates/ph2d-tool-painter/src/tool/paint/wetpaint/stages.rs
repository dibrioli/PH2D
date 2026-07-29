//! **O laço de ESTÁGIOS do tick** (filho de [`super`] — teto de LOC): quantos estágios de simulação
//! este frame consegue pagar.

use super::*;

impl PainterTool {
    /// Roda estágios enquanto o orçamento permite; devolve quantos passos COMPLETARAM (é o que decide
    /// se há composite).
    pub(super) fn wet_run_stages(sess: &mut WetSession) -> usize {
        // ── O passo é INTERROMPÍVEL, e é isso que tira a travada ────────────
        //
        // Um passo custa **38,7 ms** na escala do produto (medido) contra 16,6 de um quadro de
        // 60 Hz, então enquanto ele era ATÔMICO o frame que o continha estourava **por
        // construção** e nenhum orçamento consertava isso — o smoke mediu `tool-tick pico
        // 73,70 ms` com o app a 55 fps. O maior ESTÁGIO sozinho custa 10,26 ms e **cabe** na folga,
        // então o tick roda estágios enquanto pode pagar e retoma no frame seguinte
        // ([`ph2d_wet_paint::sim::StepCursor`]).
        //
        // ⚠️ **O `acc` gateia COMEÇAR um passo, nunca CONTINUAR um:** um passo em voo é estado do
        // motor, e abandoná-lo deixaria o grid num meio-passo permanente.
        //
        // ⚠️ **O composite roda só quando um passo COMPLETA** — o artista vê estados inteiros; um
        // frame no meio de um passo simplesmente mostra o composite anterior.
        let mut steps = 0;
        loop {
            let in_flight = sess.engine.sim.step_pending();
            if !in_flight && (sess.acc < WET_STEP_S || steps >= WET_MAX_STEPS) {
                break;
            }
            if !sess.budget.can_step() {
                break;
            }
            if !in_flight && !sess.engine.sim_should_run() {
                // Nada a simular: consome a fatia como o laço antigo consumia.
                sess.acc -= WET_STEP_S;
                steps += 1;
                continue;
            }
            // O que o estágio custou é o que ele DEBITA: um custo estimado erraria, e o erro se
            // acumularia no bucket.
            let t0 = std::time::Instant::now();
            let done = sess.engine.step_stage();
            let spent = t0.elapsed().as_secs_f32() * 1e3;
            sess.budget.spend(spent);
            crate::wet_diag::note_step(spent);
            if done.is_some() {
                sess.acc -= WET_STEP_S;
                steps += 1;
            }
        }
        steps
    }
}
