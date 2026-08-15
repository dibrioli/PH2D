//! **AS PORTAS ENTRE O NÚMERO DO ARTISTA E O QUE O KERNEL CONSOME** — irmão do
//! [`super::brush`], cortado por ASSUNTO.
//!
//! O `brush.rs` diz **o que um pincel É** (os verbos, os campos, os defaults de
//! fábrica). Aqui vive a outra pergunta: **como cada número que o artista digita
//! vira o número que o laço do dab usa** —
//!
//! - [`Brush::weight`]: o slider de força → o peso do dab (a `StrengthCurve`);
//! - [`Brush::shaped_distance`]: a distância → a distância que a curva lê (o
//!   `hardness`);
//! - [`Brush::mask_weight`]: a distância → o peso do canal de máscara;
//! - [`Brush::reach`]: o raio → o deslocamento de um dab de peso cheio.
//!
//! ⚠️ **As quatro são PORTAS ÚNICAS, e é por isso que elas viajam juntas.** Cada
//! uma é o único lugar onde aquela conversão acontece; espalhá-las é como a
//! segunda cópia nasce, e este módulo existe para que a próxima conversão tenha
//! um endereço óbvio em vez de cair no arquivo que tiver espaço.

use super::*;

impl Brush {
    /// **O PESO que este pincel deposita** — a porta única entre o número do
    /// slider e o que um dab de fato faz.
    ///
    /// ⚠️ **Ela existe porque o slider e o peso deixaram de ser a mesma coisa.**
    /// O Blender eleva ao quadrado (`sculpt.cc:2339`, *"square it to make lower
    /// values more sensitive"*) e o SculptGL não; um `brush.strength` cru no
    /// sítio de uso seria a segunda resposta que ignora o modo **em silêncio**,
    /// e o `stroke.rs` tem UM consumidor — é ele que pergunta aqui.
    #[must_use]
    pub fn weight(&self) -> f32 {
        self.verb
            .profile(self.mode)
            .map_or(self.strength, |p| p.strength_curve.resolve(self.strength))
    }
}

/// A dureza do canal em `1.0` dá expoente **zero**, e `x^0 == 1` em toda a
/// pegada — o disco duro. É o topo da faixa da tool do original, e o nome existe
/// para o painel não repetir o literal.
pub const MAX_MASK_HARDNESS: f32 = 1.0;

impl Brush {
    /// **A DISTÂNCIA QUE A CURVA VAI LER** — a porta única do `hardness`.
    ///
    /// Porte literal do `apply_hardness_to_distances` (`sculpt.cc:7549-7575`),
    /// em distância NORMALIZADA (`t = d / raio`), que é a forma em que o resto
    /// deste motor fala:
    ///
    /// ```text
    /// t' = 0                        se t < hardness
    /// t' = (t − hardness)/(1 − h)   caso contrário
    /// ```
    ///
    /// Ou seja: um **platô de peso cheio** de raio `hardness · r`, e o falloff
    /// inteiro espremido na casca que sobra. Em `hardness = 1` o pincel vira um
    /// **disco duro** — e esse caso tem braço próprio no original porque a
    /// fórmula geral dividiria por zero.
    ///
    /// ⚠️ **Ela remapeia a distância de TODOS os consumidores da curva**, o
    /// canal de máscara incluído, porque é isso que o original faz: o
    /// `apply_hardness_to_distances` roda **antes** do
    /// `BKE_brush_calc_curve_factors`, e nenhuma curva sabe que ele existe.
    /// Aplicá-la só na geometria faria a máscara ler uma distância diferente da
    /// que o pincel usa no mesmo dab.
    #[must_use]
    pub fn shaped_distance(&self, t: f32) -> f32 {
        let h = self.hardness;
        if h <= 0.0 {
            // ⚠️ **O early-out É a identidade bit a bit**, e é ele que torna
            // esta wave invisível no produto: sem `hardness`, nem uma subtração
            // acontece.
            return t;
        }
        if h >= 1.0 {
            // O braço do disco duro: dentro do raio nada decai, fora dele o
            // peso é zero (que é o que a curva devolve em `t >= 1`).
            return if t < 1.0 { 0.0 } else { 1.0 };
        }
        if t < h { 0.0 } else { (t - h) / (1.0 - h) }
    }

    /// **A CURVA DO CANAL DE MÁSCARA** — `(1 − t)^{2(1 − hardness)}`, o
    /// `Masking.paint` do original (`Masking.js:66-69`).
    ///
    /// ⚠️ **A aritmética é `f64` e a arredondada é UMA**, como em todo o porte
    /// (`ref_kernels`): o `Math.pow` do JS trabalha em duplo e o
    /// `Float32Array` guarda uma vez. Computá-la em `f32` acumularia uma
    /// segunda arredondada e a paridade sairia do piso do formato.
    ///
    /// ⚠️ **`t` chega JÁ normalizado pelo raio** e não é clampado aqui: o
    /// original clampa (`if dist > 1 dist = 1`) e a nota do
    /// [`crate::ref_kernels`] mede que esse ramo é **inalcançável** — quem monta
    /// a pegada só admite `d² < r²`. A guarda contra `t > 1` mora onde o
    /// consumo mora, e duplicá-la aqui seria a segunda resposta à mesma
    /// pergunta.
    #[must_use]
    pub fn mask_weight(&self, t: f32) -> f32 {
        let softness = 2.0 * (1.0 - f64::from(self.mask_hardness));
        (1.0 - f64::from(t)).powf(softness) as f32
    }
}

impl Brush {
    /// O deslocamento, com sinal, que um dab deste pincel alcança — em unidades
    /// de mundo. Porta única: Draw, Inflate, Clay e Crease perguntam aqui.
    ///
    /// ⚠️ **O termo `honours_invert()` aqui é INERTE hoje, e fica assim mesmo.**
    /// Depois que o predicado passou a dizer a verdade, *todo* verbo que lê
    /// `reach` está na whitelist ⇒ trocá-lo por um `if self.invert` puro daria o
    /// mesmo número em todos os doze, e uma mutação que o remova **não sangra**.
    /// Ele fica porque é aqui que a pergunta pertence — *o sinal é assunto do
    /// VERBO, não do checkbox* —, e porque o dia em que entrar um verbo que
    /// consome `reach` sem ter oposto é o dia em que ele deixa de ser inerte, sem
    /// ninguém precisar lembrar. Defesa em camadas documentada em vez de gateada,
    /// pelo precedente do ADR-0145.
    ///
    /// ⚠️ **Este bloco estava ÓRFÃO** — colado acima do `alpha_weight`, descrevendo
    /// uma função que não é esta, com o `reach` sem doc nenhum. É a classe que
    /// este módulo já registrou duas vezes (*"minhas linhas `mod` orfanaram
    /// doc-comments"*), e ela não levanta erro: só uma leitura pega.
    #[must_use]
    pub fn reach(&self, radius: f32) -> f32 {
        let s = if self.invert && self.verb.honours_invert() {
            -1.0
        } else {
            1.0
        };
        radius * REACH_FRACTION * s
    }

    /// **O RAIO QUE A CONSULTA DA PEGADA USA** — a quinta porta, e a única que
    /// pode devolver MAIS que o raio do pincel.
    ///
    /// ⚠️ **Ela existe porque um campo elástico decide o próprio suporte, e o
    /// inverso não é verdade.** Um verbo de carimbo é uma curva que já vale zero
    /// em `t = 1`, então a pegada e a influência são o mesmo círculo. Um
    /// Kelvinlet não acaba em lado nenhum: o raio do pincel é o `ε` dele — a
    /// ESCALA da resposta —, e quem decide onde cortar é o resíduo que ainda
    /// sobra (ver [`crate::KELVINLET_REACH`]).
    ///
    /// ⚠️ **A leitura do anel do cursor MUDA, e é o preço nomeado:** com um
    /// campo, ele deixa de significar *o que eu toco* e passa a significar *a
    /// escala do que eu deformo* — a mesma leitura do Elastic Deform do Blender,
    /// cujo pincel deforma bem além do círculo desenhado. Um artista que quer o
    /// círculo literal tem o `s-mode` ao lado, no mesmo verbo.
    #[must_use]
    pub fn query_radius(&self, radius: f32) -> f32 {
        if self.mode.field(self.verb).is_some() {
            radius * crate::KELVINLET_REACH
        } else if self.verb == crate::Verb::ClayStrips {
            // ⚠️ **Uma CAIXA não cabe no círculo que a inscreve.** O canto de
            // uma faixa `1 × L` está a `√(1 + L²)` raios do centro, e uma
            // consulta de raio `r` devolveria a faixa com as QUINAS comidas —
            // um defeito mudo, porque a silhueta continuaria plausível. O fator
            // é perguntado à própria forma, nunca recomputado aqui.
            radius * crate::Footprint::strip_query_factor(self.strip_length)
        } else {
            radius
        }
    }
}
