//! **The lane's HOLD** — which strip fills the coverage its live strips leave, and the pose
//! a fade crosses TO or FROM.
//!
//! Split from [`crate::stack`] for the LOC cap, but it is one unit: the complement of
//! [`ClipLane::weight_at`], answering *"what is under the fade"* for a gap, a loop wrap, or a
//! lone fade at either edge (Enio's fixes, 2026-07-16 … 2026-07-19). It reads the lane's
//! geometry (`weight_at`, `blend_in`/`blend_out`, `gap_*`) and the strips' time map
//! (`fold`, `hold_source_time`), and writes nothing.

use crate::stack::{ClipLane, ClipStrip};
use ph2d_anim::Easing;

impl ClipLane {
    /// **Which strip is HOLDING at `t`, and how strongly** — the lane's answer for
    /// whatever coverage its live strips do not account for.
    ///
    /// `None` when the live strips already sum to a full 1 (a strip mid-span, or two
    /// crossfading through their overlap: the overlap sums to exactly 1, so nothing
    /// is held and the crossfade is untouched), or when nothing has ended yet.
    ///
    /// # A strip's pose does not evaporate at its edge
    ///
    /// Before this, a lane's answer where no strip covered was *silence* — nobody
    /// wrote, and the object simply kept the pose it had. But a strip covering `t`
    /// with weight 0 (the first instant of a fade-in) answered **rest**. The two
    /// disagreed across one pixel of ruler, so a fade-in against nothing began with a
    /// jump: the sprite sat where the previous strip left it, then snapped to the rest
    /// pose to start the ramp it was supposed to start from *where it was* (Enio,
    /// 2026-07-16: *"a sprite não faz a transição a partir de onde está mas pula para
    /// mais perto da posição inicial da outra strip"* — measured at 3 units in one
    /// frame).
    ///
    /// The fix is not to silence weight zero — that is what made the pose depend on
    /// which side the playhead arrived from. It is that **the gap was never silence**:
    /// the previous strip is still asserting its last frame, and the incoming fade
    /// crossfades against *that*. This is Blender's `Hold` extrapolation and Unity's
    /// clip extrapolation, and it is what makes the lone fade behave like the overlap
    /// the animator already trusts.
    ///
    /// **Forward only — UNLESS a loop makes the lane cyclic.** A strip does not reach
    /// back before it starts: fading in from the rest pose at the top of a timeline is
    /// a real thing to want, and there is nothing behind the first strip to hold.
    ///
    /// `loop_range` chega SEMPRE, e `wraps` diz se a costura é **VIAJADA** — duas
    /// perguntas, dois parâmetros. Um playhead de ping-pong REFLETE nas pontas e nunca
    /// cruza a costura, e a lei que nasceu disso (Enio, 2026-07-23) é sobre um **VÃO
    /// SECO**: ali a lane não escreve NADA e o objeto fica onde a strip o deixou. Um
    /// alcance filtrado a `None` respondia isso *e* mais três coisas que ninguém pediu —
    /// as duas BORDAS FADEADAS decaíam para o `rest`, que é invisível para quem parkou a
    /// sprite na pose da animação (medido: `-3,000` a faixa inteira, as duas pontas).
    /// Hoje as bordas com fade cruzam a costura sob os dois modos, cada uma sob o guard
    /// que a torna *a ponta da composição* (`reaches_the_end` · `opening_fade_owns_the_turn`),
    /// e o vão seco segue mudo.
    ///
    /// That last clause is true of a timeline you play once, and **false of one you
    /// loop**, where the ruler's ends are neighbours: what is "before the first strip"
    /// is "after the last", and what is "after the last" is "before the first". So a
    /// fade at EITHER edge of the loop crosses to the pose the loop shows at the seam
    /// (as duas pontas perguntam a [`Self::seam_split`], que é o que as impede de
    /// discordar), never to the rest pose or
    /// a strip that ended before it:
    ///
    /// - the **opening** fade-in (nothing has ended yet) put the object at the last
    ///   strip's pose one frame and at the rest pose the next — a jump (Enio,
    ///   2026-07-16);
    /// - the **closing** fade-out (the loop's last content fading out toward the wrap)
    ///   reached the previous strip's held frame instead of the first strip's start —
    ///   also a jump (Enio, 2026-07-19). It is the trailing ramp of the strip that ends
    ///   latest AT or before the loop end; a strip that straddles the end fades out past
    ///   the wrap, where the loop never reaches.
    ///
    /// Outside the loop range nothing wraps and the fade-from-rest above is untouched.
    ///
    /// Returns até DUAS fontes `(strip, o segundo de clipe que ela assere, peso)`. Duas
    /// só na costura de um loop cujas duas pontas fadeiam, onde a pose é uma MISTURA
    /// ([`Self::seam_split`]); em todo o resto é uma, e os pesos somam o mesmo complemento
    /// de sempre. O tempo vem daqui em vez de ser re-derivado pelo chamador porque os
    /// casos o respondem de formas diferentes, e um chamador que escolhesse seria uma
    /// segunda opinião sobre qual frame está sendo segurado.
    ///
    /// The weight is the complement of what is live, which is exactly what turns the
    /// normalized mix into a plain `lerp(held, incoming, w)` — see the tests.
    #[must_use]
    pub fn hold_at(
        &self,
        t: f64,
        loop_range: Option<(f64, f64)>,
        wraps: bool,
    ) -> [Option<(&ClipStrip, f64, f64)>; 2] {
        // A curva da costura, resolvida UMA vez: ela molda a entrada da cabeça, então o
        // somatório vivo daqui e o peso vivo do `stack_frames` têm de vê-la igual.
        //
        // ⚠️ Só sob um loop que ENVOLVE: sob ping-pong a volta não é uma travessia (o
        // playhead reflete), então a cabeça mantém a curva dela.
        let seam = if wraps {
            self.seam_curve(loop_range)
        } else {
            None
        };
        let live: f64 = (0..self.strips.len())
            .map(|i| self.weight_at_with(i, t, seam))
            .sum();
        let w = 1.0 - live;
        if w <= 0.0 {
            return [None, None];
        }
        // ⚠️ **E ela dispara TAMBÉM sob ping-pong** (Enio, 2026-07-31: *"para PingPong a FADE
        // final externa não provoca nenhum movimento, mas deveria provocar a transição mesmo
        // sendo inútil … para manter a coerência do sistema"*). Sem o alcance aqui, a cauda
        // decaía para o **REST**, e o `rest` é capturado por binding: quem ligou a sprite onde
        // a animação a DEIXA — o caso ordinário — via a influência cair sobre uma pose
        // idêntica, ou seja **nada** (medido: `5,00 5,00 5,00 5,00` com `rest = 5`, contra
        // `5,00 → 1,42` sob loop). Sob ping-pong a travessia é semanticamente inútil (o
        // playhead volta por onde veio) e o Enio a pediu assim mesmo: um fade autorado tem de
        // FAZER alguma coisa, e a que ele faz é a mesma que faz sob loop.
        //
        // **CLOSING edge of a loop.** The strip that ends latest (at or before the loop
        // end) in its fade-OUT ramp — INWARD (`ease_out`, from `t_end - bo`) or OUTWARD
        // (`lead_out`, in the gap from `t_end`) — crosses to the pose the loop shows at the
        // seam ([`Self::seam_split`]), not to the strip that ended before it (nor, for a
        // lead-out, to its own frozen last frame — that made the outward fade do nothing at
        // the wrap; Enio, 2026-07-19). A strip that STRADDLES the loop end fades out past
        // the wrap, where the loop never reaches, so `t_end <= b` gates it out. This runs
        // BEFORE the mid-timeline hold below, which is exactly the answer it overrides: at
        // the trailing edge the previous strip is not the pose to reveal.
        if let Some((a, b)) = loop_range
            && t >= a
            && t < b
        {
            let closing = self
                .strips
                .iter()
                .enumerate()
                .max_by(|(_, x), (_, y)| x.t_end.total_cmp(&y.t_end))
                .is_some_and(|(li, last)| {
                    let bo = self.blend_out(li);
                    // ⚠️ **Sob ping-pong, só o fade que ALCANÇA o fim do alcance** — o final da
                    // composição, que é o caso que o Enio pediu ("fim da animação"). Sem esta
                    // metade eu ressuscitei o bug de 2026-07-23 na hora, e o gate
                    // `a_pingpong_scrub_does_not_jump_at_a_faded_strips_exit` o pegou: um
                    // overlay que fadeia no MEIO do loop passava a cruzar para a costura em vez
                    // de revelar a lane de baixo, e o frame congelado dele espetava (x=20 onde
                    // o fundo lê 25).
                    //
                    // Sob um loop que ENVOLVE a largura é deliberada (o irmão *no overreach* a
                    // aprova): ali a costura É viajada, então segurar a pose de saída através
                    // dela é o desenho do loop sem emenda. Sob reflexão não há costura viajada,
                    // e a única travessia que faz sentido dar a um fade é a do fim.
                    let reaches_the_end = wraps || last.lead_end() >= b;
                    (bo > 0.0 || last.lead_out > 0.0)
                        && last.t_end <= b
                        && reaches_the_end
                        && t >= last.t_end - bo
                });
            if closing {
                let held = self.seam_held(a, b, w, seam);
                if held[0].is_some() {
                    return held;
                }
            }
        }
        // **FADE-OUT toward the NEXT strip (no loop needed).** A strip in its fade-out ramp
        // — and the gap AFTER it, up to where the next strip starts — crosses to the NEXT
        // strip's START, not to the rest pose (Enio, 2026-07-19: without this the object
        // sagged to rest during the fade, then JUMPED back to the strip's held pose in the
        // gap, then jumped again into the next strip). Now it travels to the next pose while
        // it fades, HOLDS it through the gap, and the next strip plays from it seamlessly.
        //
        // Runs BEFORE the mid-timeline hold below and overrides it: the hold reveals the
        // PREVIOUS strip (correct for a fade-IN, wrong for a fade-OUT, which reveals where
        // the object is GOING). It only fires when the strip actually faded out
        // (`blend_out > 0`, inside `fade_out_target`) — a hard cut with no fade keeps the
        // gap-holds-previous behaviour, which is the author's choice.
        if let Some(nxt) = self.fade_out_target(t) {
            return [Some((nxt, nxt.fold(0.0), w)), None];
        }
        // The most recently ENDED strip. A scan, not `strips.last()`: the lane is
        // sorted by START time, and a long strip can begin before a short one and
        // outlive it. This is the pose a fade-IN crosses FROM, and what a plain gap
        // (previous strip did not fade out) holds.
        //
        // **Only a BOUNDED gap holds — past its LAST strip the lane RELEASES**
        // (Enio, 2026-07-23: *"no momento em que coloco o clip na lane 2 o segundo
        // clip da lane 1 não toca mais"*). The hold exists so a fade crosses FROM
        // somewhere and a gap between strips stays deterministic; held forever, an
        // upper lane with one short strip becomes an eternal full-influence mask
        // over everything below it. So the hold asks "is anything still coming in
        // this lane?" — presence reaching `t`, BOUNDARY INCLUSIVE (`lead_end() >=
        // t`: a strip ahead, one still fading here, or the exact instant the last
        // one ends) holds; strictly beyond it the lane goes silent and the lanes
        // below show through. The inclusive edge is load-bearing: a CONTAINER's
        // held end frame is its interior evaluated exactly AT its length, and an
        // exclusive release erased every container's last pose (the
        // `nesting_leads` gate caught it). Under a WRAP loop the lane is cyclic
        // ("after the last" IS "before the first"), so inside the loop the
        // trailing hold stays — it is the pose the seam design rests on.
        if let Some(held) = self
            .strips
            .iter()
            .filter(|s| s.t_end <= t)
            .max_by(|a, b| a.t_end.total_cmp(&b.t_end))
        {
            // Strictly ahead (a future strip, or one still crossfading here) always
            // holds the previous pose to bridge the gap. A strip exactly AT its end
            // (`lead_end == t`) holds its frozen frame ONLY when it is a HARD cut
            // (`blend_out == 0`): a container's held end frame evaluated at its length
            // (the `nesting_leads` case — the inclusive edge that the comment above
            // calls load-bearing). A strip that FADED OUT has already taken its
            // influence to 0 at its `lead_end`, so its own just-ended boundary must
            // NOT snap its frozen frame back to full weight — that spike is invisible
            // during Play (a tick almost never lands exactly on the boundary) but a
            // paused, frame-snapped SCRUB parks on it, and under PingPong (the loop is
            // filtered to `None`) no seam smooths it, so the object jumped at the strip
            // exit (Enio, 2026-07-23).
            // ⚠️ **Um strip que JÁ ACABOU não está "à frente" — nem pela própria janela de
            // `lead_out`, que NÃO é um corte seco** (Enio, 2026-07-30: *"na lane 2 temos um
            // fade do lado direito para fora e ele não funcionou corretamente"*).
            //
            // Duas metades do predicado deixavam o strip que acabou de terminar responder
            // *"algo ainda vem"* sobre SI MESMO — o `lead_end()` de um `lead_out` o estende
            // além do `t_end`, e a isenção da borda inclusiva perguntava só `blend_out <= 0`
            // (o blend de SOBREPOSIÇÃO), que um fade para FORA nunca tem. O hold então
            // devolvia a pose congelada dele com peso `1 − w`, a cobertura da lane voltava a
            // exatamente **1**, e a fórmula do fade cruzava contra ela mesma: medido, peso
            // `1.000 → 0.000` ao longo da janela com a pose **PARADA** o trecho inteiro e um
            // degrau em `lead_end`. O fade só MOVIA o corte por `lead_out` segundos.
            //
            // Com um PRÓXIMO strip nada disto aparecia: o `fade_out_target` acima dispara
            // primeiro e a travessia funciona. O defeito vive só no ÚLTIMO strip de uma lane,
            // onde o que está do outro lado do fade são **as lanes de baixo** — e é isso que
            // um `influence < 1` já diz, desde que o hold não recomponha o peso.
            //
            // O `s.lead_out <= 0.0` é a mesma pergunta que o `fade_out_target` já faz
            // (`bo > 0.0 || s.lead_out > 0.0`): os dois têm de concordar sobre *"este strip
            // fadeou?"*. A borda inclusiva do corte seco fica intacta — ela é o frame final
            // segurado de um container, e o `nesting_leads` a pegou uma vez.
            let something_ahead = self.strips.iter().enumerate().any(|(i, s)| {
                let hard_cut = self.blend_out(i) <= 0.0 && s.lead_out <= 0.0;
                (s.t_end > t && s.lead_end() > t) || (s.lead_end() >= t && hard_cut)
            });
            // ⚠️ Cíclico só sob um loop que ENVOLVE: sob ping-pong "depois da última" não é
            // "antes da primeira", então o vão do fim NÃO segura — ele solta, como sem loop.
            let cyclic = wraps && loop_range.is_some_and(|(a, b)| t >= a && t < b);
            if something_ahead || cyclic {
                return [Some((held, held.hold_source_time(), w)), None];
            }
            return [None, None];
        }
        // Nothing has ended yet — the OPENING edge. Under a loop that brackets `t`,
        // wrap: the pose the object is coming FROM is the one the loop's end leaves
        // behind.
        //
        // ⚠️ **Pela MESMA porta que a borda de saída** ([`Self::seam_split`]) — antes esta
        // metade repetia a expressão *"a última strip lida em `b`"*, o que era a resposta
        // certa enquanto a costura tinha um dono só. Com as duas pontas fadeando ela é uma
        // MISTURA, e duas expressões para a mesma pose divergiriam exatamente onde o loop
        // pula: os dois lados da volta ficariam em poses diferentes.
        // ⚠️ **Sob ping-pong, só o FADE que abre a composição** — o espelho exato do
        // `reaches_the_end` da borda de saída (Enio, 2026-07-31: *"corrigiu a FADE externa
        // final mas matou a inicial"*).
        //
        // A lei de 2026-07-23 que mora aqui é sobre um **VÃO SECO**, e a cena dela diz isso:
        // *"se estamos num PingPong e no retorno ao início há um gap, ele deve parar
        // exatamente na posição inicial da strip … ao encontrar a strip o objeto ainda está
        // no lugar onde estava"* — uma strip **sem fade**, num vão onde a lane não escreve
        // NADA. Com um `lead_in` a lane já escreve (o peso rampa dentro do vão): a pergunta
        // deixa de ser *"escrevo?"* e passa a ser *"cruzo a partir de quê?"*, e a resposta
        // era o REST — invisível exatamente para quem ligou a sprite onde a animação a
        // deixa, que é o mesmo mecanismo que matava a cauda.
        //
        // Então o vão seco continua **mudo** (sem fade o predicado é falso) e o fade que
        // alcança o começo do alcance cruza da MESMA costura para onde a cauda vai — é isso
        // que faz as duas pontas *concordarem*, que é a coerência pedida. Sob um loop que
        // ENVOLVE nada disto se aplica: ali a abertura wrapa incondicionalmente, e o
        // `a_ping_pong_gap_never_shows_the_loops_end` pina as duas metades.
        let Some((a, b)) = loop_range else {
            return [None, None];
        };
        if t < a || t >= b || (!wraps && !self.opening_fade_owns_the_turn(t, a)) {
            return [None, None];
        }
        self.seam_held(a, b, w, seam)
    }

    /// **Este `t` está no fade que ABRE a composição?** — o espelho do `reaches_the_end`.
    ///
    /// Só pergunta quando a reflexão tirou a costura de cena (ping-pong): a primeira strip
    /// (a de menor `t_start`, e aqui ela é a primeira POR CONSTRUÇÃO — este ramo só roda
    /// quando nada terminou ainda) tem de ter um fade de entrada, começar dentro do alcance,
    /// e o fade tem de **chegar ao começo dele**.
    ///
    /// ⚠️ Esta última cláusula NÃO existe para evitar um salto, e a distinção importa: uma
    /// strip que fadeia entrando no MEIO do alcance, depois de um vão seco, salta de qualquer
    /// jeito no instante em que o fade começa (com a costura, salta PARA a costura; sem ela,
    /// para o REST — o peso segurado ali é 1). Ela existe porque tal strip **não é a abertura
    /// da composição**, e mudar o que ela cruza seria mexer no que ninguém reportou; o que um
    /// fade precedido de silêncio deveria cruzar é outra pergunta, e é anterior a isto.
    ///
    /// ⚠️ **Cliff nomeado, e é o MESMO das duas pontas:** `lead_start() <= a` é exato, então
    /// um fade que para 10 ms depois do começo do alcance não é a abertura — como um
    /// `lead_out` que para 10 ms antes do fim não é o fecho (`reaches_the_end`). Uma
    /// tolerância seria um número que eu não medi; a lei é uma só, nas duas bordas.
    fn opening_fade_owns_the_turn(&self, t: f64, a: f64) -> bool {
        self.strips
            .iter()
            .enumerate()
            .min_by(|(_, x), (_, y)| x.t_start.total_cmp(&y.t_start))
            .is_some_and(|(fi, first)| {
                let bi = self.blend_in(fi);
                // ⚠️ Não há cláusula *"tem de ter fade"*, e a MUTAÇÃO é que decidiu isso: uma
                // sobreviveu, e ela era **provavelmente redundante** — sem fade,
                // `lead_start() == t_start`, então as duas primeiras cláusulas exigem
                // `t_start == a`, e aí a janela pede `t < a` contra o `t >= a` que o chamador
                // já garantiu. Uma condição que não pode ser falsa lê como carga e não é.
                first.t_start >= a && first.lead_start() <= a && t < first.t_start + bi
            })
    }

    /// **What the loop shows at the seam** (`b` ≡ `a`) — the pose the object rests on
    /// across the wrap, as a `(strip, clip-time)` the evaluator can sample.
    ///
    /// A seamless loop needs the fade on BOTH sides of the wrap to cross to the *same*
    /// pose, or they disagree and the loop jumps. That pose is whichever end OWNS the
    /// seam:
    ///
    /// - if a strip is fully live at the head `a` (no fade-in there), its own pose there
    ///   is the restart pose — the closing fade-out crosses to it and the loop lands on
    ///   it;
    /// - otherwise the head itself is fading in, and the object crosses from what the
    ///   loop's END leaves asserting: the last strip read at `b`. This is exactly the
    ///   opening wrap's own answer — the two share this door on purpose, so the fade-in
    ///   and the fade-out cannot disagree about the seam.
    ///
    /// ⚠️ **Quando as DUAS pontas fadeiam, a travessia se DIVIDE entre elas** (Enio,
    /// 2026-07-31: *"o fade da direita é descartado e o objeto simplesmente fica parado
    /// enquanto o playhead encontra-se ali"*). O desenho anterior escolhia *"a pose da
    /// última strip"* para os dois lados, o que mantinha os dois wraps de acordo — e fazia
    /// o fade de SAÍDA cruzar **para ele mesmo**: medido, `+5` a janela inteira, com um
    /// degrau só no fim. Um fade que não move é um fade descartado.
    ///
    /// A cura mantém o invariante (**os dois lados têm de ver a MESMA pose de costura**,
    /// senão o loop pula) e o expressa como uma MISTURA: a costura é
    /// `lerp(pose_do_fim, pose_do_começo, f)`, com
    ///
    /// ```text
    /// f = janela_de_saída / (janela_de_saída + janela_de_entrada)
    /// ```
    ///
    /// isto é, **a fração da travessia que acontece ANTES da volta**. O fade de saída leva
    /// o objeto de `pose_do_fim` até essa mistura; o de entrada continua dali até
    /// `pose_do_começo`. Os dois lados chamam esta função, então não podem discordar.
    ///
    /// ⚠️ **Os dois casos de UMA ponta só são BYTE-IDÊNTICOS ao que já shipava**, e é isso
    /// que protege o que o Enio disse que *"funciona muito bem"*: sem fade de entrada
    /// `f = 1` (a saída faz a travessia inteira, uma fonte só), sem fade de saída `f = 0`
    /// (a entrada faz tudo). A divisão só existe onde antes havia um fade inerte.
    ///
    /// A primeira pergunta continua sendo a de sempre: se uma strip está VIVA no topo (sem
    /// fade lá), ela é a dona da costura e não há o que dividir.
    ///
    /// Devolve até DUAS fontes `(strip, tempo-de-clipe, FRAÇÃO)`, com as frações somando 1.
    /// Array e não `Vec` porque isto roda por frame e por lane (`no_alloc_bridge`).
    fn seam_split(
        &self,
        a: f64,
        b: f64,
        seam: Option<Easing>,
    ) -> [Option<(&ClipStrip, f64, f64)>; 2] {
        // O que o FIM do loop deixa asserindo — a última strip lida na volta.
        let Some((ti, tail)) = self
            .strips
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| x.t_end.total_cmp(&y.t_end))
        else {
            return [None, None];
        };
        let tail_at = tail.fold((b - tail.t_start).clamp(0.0, tail.span())); // CLAMP-OK: span() >= 0

        // Quem está presente no TOPO do loop — a strip cuja janela de presença contém `a`
        // (ela mesma, ou o fade dela alcançando de volta). `None` = vão no topo: não há
        // para onde cruzar, e a costura é o que o fim deixou (o desenho de sempre).
        let Some((hi, head)) = self
            .strips
            .iter()
            .enumerate()
            .filter(|(_, s)| s.lead_start() <= a && s.lead_end() > a)
            .min_by(|(_, x), (_, y)| x.t_start.total_cmp(&y.t_start))
        else {
            return [Some((tail, tail_at, 1.0)), None];
        };
        let head_at = head.fold((a - head.t_start).clamp(0.0, head.span())); // CLAMP-OK: span() >= 0

        // As duas janelas são a MEDIDA da travessia de cada lado — a de fora (`lead_*`) e a
        // de sobreposição (`blend_*`) juntas, porque as duas movem a pose e o defeito é o
        // mesmo nas duas (um `ease_in`/`ease_out` nas pontas congela igual).
        let l_out = tail.lead_out.max(0.0) + self.blend_out(ti);
        let l_in = head.lead_in.max(0.0) + self.blend_in(hi);
        let total = l_out + l_in;
        // `total == 0` é inalcançável pelos dois chamadores (o de saída exige uma janela de
        // saída; o de entrada só roda com peso a preencher, o que exige uma de entrada) —
        // o zero é o recuo conservador: a costura de sempre.
        let f = if total > 0.0 { l_out / total } else { 0.0 };
        if f >= 1.0 {
            // A cabeça não fadeia: ela é a dona da costura, e a saída faz a travessia
            // inteira até ela. É o `head_live >= 1` de sempre, respondido pela GEOMETRIA.
            return [Some((head, head_at, 1.0)), None];
        }
        if f <= 0.0 {
            return [Some((tail, tail_at, 1.0)), None];
        }
        // ⚠️ **A pergunta CARA é feita por último, e só aqui** — `weight_at` percorre a lane
        // por strip (ele consulta `blend_in`/`blend_out`), então somar a cobertura é
        // QUADRÁTICO, e a borda de ENTRADA do `hold_at` chama esta função em todo frame de
        // toda lane. Pô-la no topo custou uma REGRESSÃO REAL: o
        // `the_cost_of_depth_is_linear_not_explosive` mediu **3,36×** contra a barra de 2,9 e
        // a faixa sã de ~2,25 — perto do número do mutante dele. Aqui embaixo ela só roda na
        // geometria que de fato precisa dela.
        //
        // O que ela ainda decide: uma OUTRA strip totalmente viva em `a` enquanto a que
        // começa mais cedo ainda fadeia. Aí a costura tem dona (a que cobre), e dividir
        // mandaria o objeto para uma mistura que ninguém está mostrando.
        let head_live: f64 = (0..self.strips.len())
            .map(|i| self.weight_at_with(i, a, seam))
            .sum();
        if head_live >= 1.0
            && let Some(first) = self.strips.iter().find(|s| s.covers(a))
        {
            let elapsed = (a - first.t_start).clamp(0.0, first.span()); // CLAMP-OK: span() >= 0
            return [Some((first, first.fold(elapsed), 1.0)), None];
        }
        [Some((tail, tail_at, 1.0 - f)), Some((head, head_at, f))]
    }

    /// Reparte um peso `w` pelas frações de [`Self::seam_split`].
    fn seam_held(
        &self,
        a: f64,
        b: f64,
        w: f64,
        seam: Option<Easing>,
    ) -> [Option<(&ClipStrip, f64, f64)>; 2] {
        self.seam_split(a, b, seam)
            .map(|e| e.map(|(s, t, frac)| (s, t, w * frac)))
    }

    /// The strip that starts NEXT after time `end` — the smallest `t_start >= end`, with
    /// its index. `None` when nothing starts after (`end` is past the last strip).
    ///
    /// A strip that *overlaps* `end` (`t_start < end`) is not "next": that is a
    /// crossfade, and [`Self::weight_at`] already handles it with complementary weights.
    fn next_after(&self, end: f64) -> Option<(usize, &ClipStrip)> {
        self.strips
            .iter()
            .enumerate()
            .filter(|(_, o)| o.t_start >= end)
            .min_by(|(_, a), (_, b)| a.t_start.total_cmp(&b.t_start))
    }

    /// **What a fade-OUT at `t` crosses TO** — the next strip, or `None`.
    ///
    /// It fires while a strip is in its fade-out ramp AND through the gap after it, up to
    /// where the next strip's OWN fade-in ends:
    /// `t ∈ [s.t_end - blend_out(s), next.t_start + blend_in(next))`. Two conditions gate
    /// it, and both are the point:
    ///
    /// - `blend_out(s) > 0` — the strip actually has a fade-out. A hard cut (no fade) is
    ///   the author saying "hold and jump", and the gap keeps holding the PREVIOUS strip.
    /// - a `next` strip exists — there is somewhere to cross TO. The LAST strip's fade-out
    ///   with nothing after is the loop's job (`hold_at`'s closing branch) or a fade to
    ///   rest, not this.
    ///
    /// The crossed-to pose is the next strip's FROZEN first frame (`next.fold(0.0)`), the
    /// same pose the clip shows when it starts playing — so holding it through the gap and
    /// then playing it are the same value, and the entry is seamless.
    ///
    /// **The window reaches THROUGH the next strip's fade-in** (`+ blend_in(next)`), not
    /// just up to its start. When BOTH strips fade (this one out, the next one in), the
    /// object crosses to the next start and STAYS there while the next eases in — so the
    /// next strip eases from its own start instead of snapping back to the previous strip
    /// one frame after the gap. With no fade-in on the next strip, `blend_in` is 0 and the
    /// window is exactly the gap.
    fn fade_out_target(&self, t: f64) -> Option<&ClipStrip> {
        self.strips
            .iter()
            .enumerate()
            .filter(|(i, s)| {
                let bo = self.blend_out(*i);
                // Fades out either INWARD (`ease_out`, window starts at `t_end - bo`) or
                // OUTWARD (`lead_out`, in the gap from `t_end`). Both reach a next strip.
                (bo > 0.0 || s.lead_out > 0.0) && t >= s.t_end - bo
            })
            .filter_map(|(_, s)| {
                let (ni, nxt) = self.next_after(s.t_end)?;
                (t < nxt.t_start + self.blend_in(ni)).then_some(nxt)
            })
            .min_by(|a, b| a.t_start.total_cmp(&b.t_start))
    }
}
