//! **A CORDA** — o pedido do Enio: um cordão que liga duas coisas do chrome e que **pendura**.
//!
//! Verlet de posição em **espaço de ECRÃ**, com as duas pontas pinadas: uma no controlo (o sítio
//! de onde a coisa nasceu) e outra no efeito (onde ela está agora). Entre elas, `n` nós que caem.
//!
//! ## Por que um motor PRÓPRIO, e não os dois que já temos
//!
//! - ⛔ **`rapier`** é o simulador de **MUNDO**, com contrato de determinismo (`physics_ecs_c9`,
//!   hash comparado em três SOs) e schema. Um enfeite de chrome passaria a poder **mover um hash
//!   de determinismo** — o preço mais alto que uma decoração já pediu neste repo.
//! - ⛔ **`verlet_rope`** dos nós é conteúdo **cozido do documento**, com fingerprint.
//! - ✅ Isto: ~100 linhas, zero deps, **descartável por construção** — que é exactamente o que uma
//!   decoração deve ser. Deitar fora este ficheiro não parte nada.
//!
//! ## A lei do relógio, que é a wave inteira
//!
//! Verlet clássico assume passo **FIXO**. Com passo variável — que é o que um app tem — um engasgo
//! de quadro faz a corda **saltar**, porque a velocidade implícita `p − q` é lida como se o passo
//! seguinte tivesse a mesma duração do anterior. A correcção é o factor `dt / dt_prev` (TCV, *time-
//! corrected Verlet*), e é literalmente a lei que este repositório já pagou **quatro vezes** no
//! relevo do Painter — *o desenho é facto do relógio, nunca de quão depressa a máquina amostrou*.
//!
//! ⚠️ **E o TCV NÃO CHEGA — construí-o, medi-o, e ele falha por 29,7 px.** O plano (§5.2) chamava
//! ao factor `dt / dt_prev` *«a wave inteira numa linha»*; dirigindo o MESMO gesto a 30 e a 120 fps
//! com ele, a corda pendura **29,7 px** mais baixo na máquina lenta. A medição nomeia três
//! dependências do relógio e o TCV só cura uma:
//!
//! 1. a **integração** — curada pelo `dt/dt_prev`;
//! 2. o **amortecimento**, `vel *= DAMP` por QUADRO: `DAMP^120` num segundo a 120 fps contra
//!    `DAMP^30` a 30, ou seja a mesma corda quatro vezes mais morta na máquina rápida;
//! 3. a **RIGIDEZ**, que é a maior e a que ninguém vê: a restrição de distância relaxa `ITERS`
//!    vezes por QUADRO, então a 120 fps ela puxa **quatro vezes mais por segundo**. Um solver
//!    iterativo é tão rígido quanto o número de passagens que o relógio lhe paga.
//!
//! ⇒ **O passo interno é FIXO** (`SIM_HZ`), e o quadro só decide *quantos* passos correm. As três
//! dependências morrem de uma vez e a igualdade deixa de ser aproximada: o mesmo gesto no mesmo
//! tempo dá **exactamente** os mesmos passos, logo exactamente a mesma forma. O `dt/dt_prev` sai
//! junto — com passo fixo ele é `1.0` por construção, e uma correcção que vale sempre 1 é uma
//! segunda resposta à espera de divergir.
//!
//! ⚠️ **E o acumulador traz consigo a lição que a água já pagou:** um quadro lento **não** compra
//! passos sem tecto (`MAX_STEPS`), senão ele fica ainda mais lento e a realimentação em `dt` que a
//! sim do Wet Paint documentou reaparece — aqui, num enfeite.

/// Nós da corda, pontas incluídas. Doze é o que dá a curva a leitura de *pendurado* sem que o passo
/// entre nós se veja; é número de APARÊNCIA, e o oráculo dele é o smoke.
pub const NODES: usize = 12;

/// Iterações da restrição de distância **por passo interno**. Verlet relaxa; três passagens tiram o
/// elástico visível sem prender a corda.
const ITERS: usize = 3;

/// O passo INTERNO, fixo. É ele que torna a forma um facto do relógio de parede e não da taxa de
/// quadros — ver o doc do módulo.
const SIM_HZ: f32 = 120.0;
const SIM_DT: f32 = 1.0 / SIM_HZ;

/// Tecto de passos por quadro. ⚠️ **Um quadro lento não compra passos sem limite:** a sim do Wet
/// Paint mediu essa realimentação (quadro lento ⇒ mais passos ⇒ quadro mais lento) e o preço aqui
/// seria o mesmo num enfeite. O resto é DEITADO FORA, nunca guardado como dívida.
const MAX_STEPS: usize = 8;

/// Amortecimento por passo interno. Com passo fixo, «por passo» e «por segundo» são a mesma coisa.
const DAMP: f32 = 0.965;

/// Aceleração da gravidade em **píxeis de ecrã** por segundo ao quadrado. Não é `9.81`: o ecrã não
/// é o mundo, e o que decide é a queda parecer *peso de cordão* e não *pedra*.
const GRAVITY_PX_S2: f32 = 2600.0;

/// Quanto a corda é mais longa que a linha recta. `> 1` é o que a faz **pendurar** em vez de
/// esticar; em `1.0` ela seria a própria recta.
const SLACK: f32 = 1.22;

/// Deslocamento máximo de um nó por quadro, em píxeis.
///
/// ⚠️ **O degenerado nomeado:** as duas pontas a moverem-se mais depressa do que a restrição
/// apanha esticam a corda e devolvem-na com estalo. O tecto corta o estalo sem tocar no repouso —
/// num quadro normal nenhum nó chega perto dele.
const MAX_STEP_PX: f32 = 64.0;

/// Abaixo disto o controlo e o efeito são o MESMO ponto e não há corda para desenhar.
const MIN_SPAN_PX: f32 = 1.0;

/// Uma corda viva. `advance` por quadro, `points` para desenhar.
#[derive(Clone, Debug)]
pub struct Tether {
    p: Vec<[f32; 2]>,
    q: Vec<[f32; 2]>,
    /// Tempo por gastar em passos internos. ⚠️ **Guardado entre quadros**, senão a fracção de
    /// passo que sobra em cada um é perdida e a corda anda mais devagar do que o relógio.
    acc: f32,
    /// Ainda não foi colocada: o próximo `advance` põe todos os nós na recta em vez de os deixar
    /// cair de onde estavam. Sem isto, uma corda que reaparece noutro sítio **voa** até lá.
    fresh: bool,
}

impl Default for Tether {
    fn default() -> Self {
        Self::new(NODES)
    }
}

impl Tether {
    #[must_use]
    pub fn new(n: usize) -> Self {
        let n = n.max(2);
        Self {
            p: vec![[0.0, 0.0]; n],
            q: vec![[0.0, 0.0]; n],
            acc: 0.0,
            fresh: true,
        }
    }

    /// Esquece a pose: o próximo `advance` re-coloca a corda na recta.
    pub fn reset(&mut self) {
        self.fresh = true;
    }

    /// Os nós, do controlo (`[0]`) ao efeito (`[n-1]`).
    #[must_use]
    pub fn points(&self) -> &[[f32; 2]] {
        &self.p
    }

    /// Existe corda para desenhar? `false` quando controlo e efeito são o mesmo ponto.
    #[must_use]
    pub fn is_drawable(control: [f32; 2], effect: [f32; 2]) -> bool {
        let d = [effect[0] - control[0], effect[1] - control[1]];
        (d[0] * d[0] + d[1] * d[1]).sqrt() >= MIN_SPAN_PX
    }

    /// Avança um quadro de `dt` segundos.
    ///
    /// `simulate` vem do **carácter** (`UiMotion::decorates`) e nunca é re-derivado aqui: com
    /// `false` a corda é a **recta** entre os dois pontos e **nada é integrado** — a relação
    /// continua visível, o peso é que sai (plano §5.3). Uma corda que simulasse e desenhasse recta
    /// seria custo sem efeito, e há gate sobre isso.
    pub fn advance(&mut self, control: [f32; 2], effect: [f32; 2], dt: f32, simulate: bool) {
        if !simulate || self.fresh || dt <= 0.0 {
            self.lay_straight(control, effect);
            // ⚠️ `fresh` só se apaga a simular: uma corda que passou a vida em Discreto tem de cair
            // do sítio certo no quadro em que o artista escolhe Expressivo, e não de onde estava.
            self.fresh = !simulate;
            self.acc = 0.0;
            return;
        }

        self.acc += dt;
        let mut steps = 0;
        while self.acc >= SIM_DT && steps < MAX_STEPS {
            self.step(control, effect);
            self.acc -= SIM_DT;
            steps += 1;
        }
        // ⚠️ Estourar o tecto DEITA FORA o resto em vez de o guardar: dívida acumulada é a
        // realimentação em `dt` que a sim do Wet Paint mediu — o quadro seguinte pagaria os passos
        // deste e ficaria ainda mais lento.
        if steps == MAX_STEPS {
            self.acc = 0.0;
        }
        // As pontas são a verdade do QUADRO, não do último passo interno: sem isto a corda ficaria
        // até `SIM_DT` atrás do cursor, que é o único sítio onde o olho a compara com outra coisa.
        let n = self.p.len();
        self.p[0] = control;
        self.p[n - 1] = effect;
    }

    /// Um passo interno de duração `SIM_DT`.
    fn step(&mut self, control: [f32; 2], effect: [f32; 2]) {
        let n = self.p.len();
        let span = {
            let d = [effect[0] - control[0], effect[1] - control[1]];
            (d[0] * d[0] + d[1] * d[1]).sqrt()
        };
        let rest = (span * SLACK) / (n - 1) as f32;

        // Integração. As pontas não integram — são pinadas ao fim de cada passagem.
        let g = GRAVITY_PX_S2 * SIM_DT * SIM_DT;
        for i in 1..n - 1 {
            let vx = (self.p[i][0] - self.q[i][0]) * DAMP;
            let vy = (self.p[i][1] - self.q[i][1]) * DAMP;
            let (vx, vy) = clamp_step(vx, vy);
            self.q[i] = self.p[i];
            self.p[i][0] += vx;
            self.p[i][1] += vy + g;
        }

        // Restrição de distância.
        //
        // ⚠️ **As pontas não precisam de ser re-pinadas por passagem, e isso foi MEDIDO em vez de
        // assumido.** A primeira versão re-pinava dentro do laço com um comentário a afirmar que
        // pinar só no fim «deixaria a última correcção a arrastar as pontas» — falso: quem impede
        // isso são os dois guardas abaixo (`a != 0` / `b != n - 1`), que já nunca lhes tocam. Com e
        // sem o re-pin interno o resultado é IDÊNTICO ao último dígito (nó 1 em `120,943 / 126,724`
        // nos dois), e foi assim que a mutação que o removia sobreviveu aos gates: ela não era um
        // buraco, era um no-op. *Uma defesa que mede zero e diz no comentário que é load-bearing é
        // pior que defesa nenhuma — ela impede a próxima pessoa de procurar a verdadeira.*
        for _ in 0..ITERS {
            for a in 0..n - 1 {
                let b = a + 1;
                let d = [self.p[b][0] - self.p[a][0], self.p[b][1] - self.p[a][1]];
                let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
                if l <= f32::EPSILON {
                    continue;
                }
                let k = 0.5 * (l - rest) / l;
                let corr = [d[0] * k, d[1] * k];
                if a != 0 {
                    self.p[a][0] += corr[0];
                    self.p[a][1] += corr[1];
                }
                if b != n - 1 {
                    self.p[b][0] -= corr[0];
                    self.p[b][1] -= corr[1];
                }
            }
        }
    }

    fn lay_straight(&mut self, control: [f32; 2], effect: [f32; 2]) {
        let n = self.p.len();
        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            let pt = [
                control[0] + (effect[0] - control[0]) * t,
                control[1] + (effect[1] - control[1]) * t,
            ];
            self.p[i] = pt;
            self.q[i] = pt;
        }
    }
}

fn clamp_step(vx: f32, vy: f32) -> (f32, f32) {
    let l = (vx * vx + vy * vy).sqrt();
    if l > MAX_STEP_PX {
        let k = MAX_STEP_PX / l;
        (vx * k, vy * k)
    } else {
        (vx, vy)
    }
}

#[cfg(test)]
#[path = "tether_tests.rs"]
mod tests;
