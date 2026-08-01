//! **Quanto um strip CONTA** — a segunda das duas perguntas que uma lane responde.
//!
//! O módulo do [`crate::stack`] já as nomeia: *"Onde no clipe eu estou?"* (o mapa de tempo do
//! strip) e *"Quanto eu conto?"* (a curva de fade). A primeira é do strip; esta é da LANE,
//! porque a resposta depende dos VIZINHOS — sobrepor dois strips É o crossfade, e é a
//! sobreposição que define a janela.
//!
//! Irmão do [`crate::stack_hold`], que é o COMPLEMENTO disto (*"o que está por baixo do
//! fade"*): os dois foram separados do `stack.rs` pelo cap de 700 LOC, pela mesma linha.
//!
//! ⚠️ **A lei que nada aqui pode violar:** através de uma sobreposição os dois pesos somam
//! EXATAMENTE 1. Pesos complementares não precisam de valor-base, e é isso que torna o
//! crossfade imune ao afundar-para-a-pose-default que a Unity remenda com o
//! `AnimationOutputWeightProcessor`. Ela vale por CONSTRUÇÃO (o lado que sai é o complemento
//! explícito do que entra), não por acidente da curva de fábrica — e o gate que a pina varre
//! curvas ASSIMÉTRICAS de propósito.

use crate::stack::{ClipLane, FadeCtx, ramp_with};
use ph2d_anim::Easing;

/// Que fração da janela já passou (`1` numa janela de comprimento zero, como o
/// [`ramp_with`] — a borda degenerada tem de responder o mesmo nos dois caminhos).
fn frac(elapsed: f64, window: f64) -> f64 {
    if window <= 0.0 {
        return 1.0;
    }
    (elapsed / window).clamp(0.0, 1.0) // CLAMP-OK: uma fração de janela
}

impl ClipLane {
    /// The blend window at the START of strip `i`: how far into it another strip is
    /// still playing, or the authored `ease_in` when none is.
    ///
    /// **Every other strip live at that edge is asked, not just `strips[i-1]`.** The
    /// neighbour in sort order is the right strip to ask only when the lane is a
    /// staircase, and nothing makes it one: a body drag can drop a short strip
    /// *inside* a long one, or leave two overlapping strips with a third between
    /// them. Asking the wrong strip is how a lane's coverage silently collapsed —
    /// a strip would fade out against a neighbour that had already ended, `den`
    /// would fall toward zero, and the sprite would crawl back to its rest pose in
    /// the middle of a clip that never moves. (The bug the audit found; the tests
    /// only ever built staircases.)
    #[must_use]
    pub fn blend_in(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let reach = self.neighbour_reach_in(i);
        if reach > 0.0 {
            reach.min(s.span()) // the overlap IS the blend (Unity's rule)
        } else {
            s.ease_in.max(0.0).min(s.span())
        }
    }

    /// How far another strip still plays INTO the start of strip `i` — `0.0` when
    /// nothing does. The overlap, before it is capped by the span.
    ///
    /// This is the ONE place that answers *"whose window is this?"*, and both callers
    /// need the same answer: [`Self::blend_in`] uses it to pick between the overlap and
    /// the authored `ease_in`, and the panel uses it to decide whether the ease handle is
    /// draggable or **read-only** (Unity greys the field out when an overlap defines it).
    /// Two copies of this test would be two ways to disagree about who owns an edge, and
    /// the artist would find the disagreement by dragging a handle that does nothing.
    #[must_use]
    pub fn neighbour_reach_in(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        self.strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.covers(s.t_start))
            .map(|(_, o)| o.t_end - s.t_start)
            .fold(0.0_f64, f64::max)
    }

    /// Mirror of [`Self::neighbour_reach_in`] at the end — and it asks the *other*
    /// question: only a strip that is still there when this one ENDS (`o.t_end >=
    /// s.t_end`) shortens its tail. One that ends earlier makes a hump in the middle,
    /// and the middle is not an edge.
    #[must_use]
    pub fn neighbour_reach_out(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        self.strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.t_start < s.t_end && o.t_end >= s.t_end)
            .map(|(_, o)| s.t_end - o.t_start)
            .fold(0.0_f64, f64::max)
    }

    /// The blend window at the END of strip `i`. Mirror of [`Self::blend_in`].
    ///
    /// A strip only fades OUT against something that is still there when it ends —
    /// `o.t_end >= s.t_end`. A strip that ends *before* this one does not shorten
    /// this one's tail; it makes a hump in its middle, and the middle is not an
    /// edge. That distinction is the whole of the containment fix.
    #[must_use]
    pub fn blend_out(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let reach = self.neighbour_reach_out(i);
        if reach > 0.0 {
            reach.min(s.span())
        } else {
            s.ease_out.max(0.0).min(s.span())
        }
    }

    /// Strip `i`'s weight at `t`: the product of its fade-in and fade-out ramps,
    /// zero where it does not cover `t`.
    ///
    /// The product is Unity's shape (`mixIn(t) * mixOut(t)`), and the ramp is
    /// smoothstep — an S, not a line, because a linear crossfade has a visible
    /// corner in velocity at both ends. Transcendental-free (HR-5).
    ///
    /// **The fade-in reaches OUTWARD by `lead_in`.** The ramp runs `0 -> 1` over
    /// `[lead_start, t_start + blend_in]`, so the strip is "present" in its lead-in
    /// window (in the gap), and [`Self::hold_at`] gives the complement to the previous
    /// strip's held pose: the object travels from that pose to this clip's frozen first
    /// frame ([`ClipStrip::source_time_with_lead`]) across the gap, and reaches full
    /// weight exactly at `t_start` — where the clip starts playing clean. With no lead
    /// (`lead_in == 0`) the window is `[t_start, ...]` and this is byte-for-byte the old
    /// behaviour.
    ///
    /// **The fade-out reaches OUTWARD by `lead_out`**, symmetrically: the ramp runs `1 -> 0`
    /// over `[lead_end - (lead_out + blend_out), lead_end]`, so the strip stays "present" in
    /// its lead-out window (in the gap AFTER it), holding its frozen LAST frame
    /// ([`ClipStrip::source_time_with_lead`]) while [`Self::hold_at`] gives the complement to
    /// the next strip's start — the object travels there across the gap. With no lead-out the
    /// window is `[…, t_end]` and this is byte-for-byte the old behaviour.
    #[must_use]
    pub fn weight_at(&self, i: usize, t: f64) -> f64 {
        self.weight_at_with(i, t, FadeCtx::default())
    }

    /// [`Self::weight_at`] com a curva da COSTURA de um loop, quando há uma.
    ///
    /// ⚠️ **`seam` substitui a curva de ENTRADA da strip da cabeça, e só dela** (decisão do
    /// Enio: *"se houver dois fades (um em cada ponta) o fade da última prevalece sobre o
    /// fade da primeira"*): as duas metades da volta são UMA travessia, e duas curvas a
    /// moldariam com um joelho no meio. Quem resolve *qual* é ela — e paga o custo uma vez
    /// por lane, não uma por strip — é [`Self::seam_curve`].
    ///
    /// ⚠️ **Os três chamadores de peso têm de passar o MESMO `seam`** (o somatório vivo do
    /// `hold_at`, a cobertura da cabeça do `seam_split` e o peso vivo do `stack_frames`):
    /// o peso segurado é `1 − vivo`, então uma metade que resolvesse a curva diferente da
    /// outra faria a lane somar ≠ 1 e a pose afundar para as lanes de baixo.
    #[must_use]
    pub fn weight_at_with(&self, i: usize, t: f64, ctx: FadeCtx) -> f64 {
        let s = &self.strips[i];
        let lead = s.lead_in.max(0.0);
        if t < s.lead_start() || t >= s.lead_end() {
            return 0.0;
        }
        // ⚠️ **A costura é UMA travessia, então as duas pontas dela são FATIAS de uma curva
        // só** ([`Seam`]) — cada uma corria um S inteiro na própria janela, e o objeto
        // PARAVA na volta (velocidade medida: 0,000 exatamente ali). Só com as duas pontas
        // fadeando (`split`); com uma só, o caminho abaixo é o que já shipava, ao bit.
        let sm = ctx.split_seam();
        // ⚠️ **Numa SOBREPOSIÇÃO o crossfade tem UMA curva, e o lado que SAI é o COMPLEMENTO
        // dela** — é isso que deixa a curva autorada alcançar um crossfade sem quebrar nada.
        //
        // A lei que a sobreposição não pode violar é `w_entra + w_sai == 1` (pesos
        // complementares não precisam de valor-base, e é o que torna o crossfade imune ao
        // afundar-para-a-pose-default que a Unity remenda com o
        // `AnimationOutputWeightProcessor`). Antes disto ela valia por ACIDENTE da curva de
        // fábrica — `smoothstep(1−u) == 1−smoothstep(u)` —, então uma curva assimétrica de um
        // lado só a quebrava, e a saída era recusar a curva ali. Tomando o complemento
        // EXPLICITAMENTE ela passa a valer para QUALQUER curva, por construção.
        //
        // ⚠️ Quem governa é a curva de ENTRADA de quem CHEGA — é o clipe que se vê chegar.
        //
        // ⚠️ **Com `None` isto NÃO é byte-idêntico ao que shipava, e a diferença foi MEDIDA:**
        // `1 − s(u)` e `s(1−u)` são iguais na ÁLGEBRA e não em `f64` — sobre um milhão de
        // amostras, **626 962 diferem, pior delta 2,2e-16** (um ulp). É invisível numa pose e
        // não é nada que se veja, mas a frase "reduz literalmente" seria falsa, e uma
        // afirmação de identidade que ninguém conferiu é como um gate de identidade nasce
        // verde sobre uma diferença real.
        let win_in = lead + self.blend_in(i);
        let fade_in = match sm {
            Some(sm) if sm.head == i => sm.head_weight(frac(t - s.lead_start(), win_in)),
            _ => ramp_with(t - s.lead_start(), win_in, ctx.shape(s.curve_in)),
        };
        let win_out = s.lead_out.max(0.0) + self.blend_out(i);
        let fade_out = match (sm, self.overlap_partner_out(i)) {
            // A costura vem primeiro: uma sobreposição no fim do alcance é o crossfade da
            // volta, e quem manda ali é a travessia inteira.
            (Some(sm), _) if sm.tail == i => {
                sm.tail_weight(frac(win_out - (s.lead_end() - t), win_out))
            }
            // `win_out > 0` é garantido por haver parceiro (a sobreposição É a janela), mas o
            // guard fica: sem ele um `win_out` zero viraria `1 − 1 = 0` e apagaria o strip.
            (_, Some(j)) if win_out > 0.0 => {
                1.0 - ramp_with(
                    win_out - (s.lead_end() - t),
                    win_out,
                    ctx.shape(self.strips[j].curve_in),
                )
            }
            _ => ramp_with(s.lead_end() - t, win_out, ctx.shape(s.curve_out)),
        };
        fade_in * fade_out
    }

    /// **Quem CHEGA na sobreposição que termina o strip `i`** — o parceiro cuja `curve_in`
    /// governa aquele crossfade, e de cujo complemento sai o peso de quem parte.
    ///
    /// O mesmo predicado do [`Self::neighbour_reach_out`] (é ele que define o que "sobrepor a
    /// saída" quer dizer), resolvido para o ÍNDICE em vez da largura: entre vários candidatos
    /// vence o que alcança MAIS para trás, que é o que decide a janela.
    #[must_use]
    pub fn overlap_partner_out(&self, i: usize) -> Option<usize> {
        let s = &self.strips[i];
        self.strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.t_start < s.t_end && o.t_end >= s.t_end)
            .max_by(|(_, a), (_, b)| b.t_start.total_cmp(&a.t_start))
            .map(|(j, _)| j)
    }

    /// **Onde MORA a curva que molda esta borda** — `(strip, borda)`, a porta única que o
    /// desenho LÊ e o menu ESCREVE.
    ///
    /// Numa sobreposição a travessia é uma só e a curva é a de quem CHEGA, então as duas
    /// cunhas do crossfade são duas VISTAS do mesmo campo; fora dela, cada borda é dona da
    /// sua. Duas respostas para *"onde mora esta curva?"* é um menu que aceita o clique e
    /// não muda a tela (foi o que aconteceu — ver [`crate::intent_apply_fade`]).
    #[must_use]
    pub fn curve_owner(&self, i: usize, edge: u8) -> (usize, u8) {
        match self.overlap_partner_out(i) {
            Some(j) if edge == 1 => (j, 0),
            _ => (i, edge),
        }
    }

    /// **A curva que de fato molda o fade de uma borda** — o que o PAINEL desenha dentro da
    /// cunha. Lê pelo [`Self::curve_owner`].
    #[must_use]
    pub fn effective_curve(&self, i: usize, edge: u8) -> Option<Easing> {
        let (j, e) = self.curve_owner(i, edge);
        if e == 0 {
            self.strips[j].curve_in
        } else {
            self.strips[j].curve_out
        }
    }

    /// Este strip fadeia CONTRA um vizinho na entrada? (Então a sobreposição é o blend.)
    #[must_use]
    pub fn overlapped_in(&self, i: usize) -> bool {
        self.neighbour_reach_in(i) > 0.0
    }

    /// O espelho de [`Self::overlapped_in`], na saída.
    #[must_use]
    pub fn overlapped_out(&self, i: usize) -> bool {
        self.neighbour_reach_out(i) > 0.0
    }
}
