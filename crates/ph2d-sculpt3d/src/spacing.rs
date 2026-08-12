//! **O ESPAÇAMENTO** — quantos dabs um gesto deposita, e onde.
//!
//! A [`crate::SculptStroke`] garante que o efeito de um traço não depende de
//! quão fino o motor amostrou o caminho. Isto aqui responde a outra metade da
//! mesma frase: **o caminho tem de ser amostrado**. Um evento de ponteiro que
//! salta 40 px deixaria um vão de 40 px entre dois dabs, e nenhuma lei de
//! envelope preenche o que ninguém carimbou.
//!
//! A lei era a do SculptGL (`SculptBase.js:126-151`) e **deixou de ser numa
//! metade**, por medição:
//!
//! ```text
//! dist = |mouse − âncora|
//! if dist < min_spacing { return }           // ⇐ o CARRY: não carimba, e a âncora FICA
//! n = floor(dist / min_spacing)
//! …carimba n dabs, a EXATAMENTE `min_spacing` um do outro…
//! âncora = ÚLTIMO DAB                        // ⇐ e não o ponteiro
//! ```
//!
//! ⚠️ **A última linha é a mudança, e ela vale 6,485 % → 0,000 %.** O original
//! espaça os dabs por `dist/n ∈ [ms, 2·ms)` e depois move a âncora para o
//! **PONTEIRO** — o que descarta o resíduo acima de um passo. Medido
//! (`tests/measure_path_invariance.rs`), o MESMO caminho entregue em 1..100
//! eventos irregulares produzia **26..33 dabs**, e o resultado do produto
//! divergia **6,485 %** da excursão do próprio traço. Com a âncora no último
//! dab, o resíduo é **geometria preservada** em vez de um número descartado:
//! **33..33 dabs** e **0,000 %**.
//!
//! ⚠️ **Isto corrige uma promessa que o [`crate::SculptStroke`] faz e não
//! cumpria.** O cabeçalho dele atribui a independência de amostragem ao
//! ENVELOPE; a medição diz que o envelope a **amortece ~2,7×** e que quem a
//! entrega é este arquivo. As duas metades continuam necessárias — o envelope
//! pelo que faz com dabs SOBREPOSTOS, este pelo que faz com a LISTA.
//!
//! ⚠️ **A paridade por-TRAÇO com o driver do original cai aqui, e é deliberado**
//! (a por-DAB fica: o kernel não é tocado). Um driver cuja lista de dabs varia
//! 1,27× pelo mesmo caminho não é alvo de paridade — é um defeito que a
//! referência tem, e o `docs/3D/19` §3.2.1 registra a decisão.
//!
//! ⚠️ **E o carry está no TIPO, não numa convenção:** [`walk`] devolve `None`
//! para o caso do carry, então "esqueci de não mover a âncora" não é um erro que
//! se comete distraído — é um `match` que não compila pela metade. A metade nova
//! está no tipo pela mesma razão: quem chama pergunta [`Walk::anchor`] em vez de
//! escrever `= [x, y]`, e o terceiro sítio de chamada nasce certo.
//!
//! ⚠️ **A unidade é a do CHAMADOR.** Este módulo não sabe o que é um pixel; ele
//! recebe dois pontos e uma distância mínima na mesma régua. No nosso shell essa
//! régua é a TELA, porque o raio do pincel é em pixels de tela (item 6b) — e é
//! exatamente por isso que o espaçamento e o raio de tela são **uma fatia só**.

/// Fração do raio que separa dois dabs. É o `0.15` do original
/// (`SculptBase.js:127`), e a razão de ele ser uma FRAÇÃO e não um número de
/// pixels é a mesma do falloff: um pincel grande espalha mais, e o que o artista
/// vê como "traço contínuo" é a sobreposição, não a distância absoluta.
pub const MIN_SPACING_FRACTION: f32 = 0.15;

/// A distância mínima entre dois dabs de um pincel de raio `radius`.
#[must_use]
pub fn min_spacing(radius: f32) -> f32 {
    (radius * MIN_SPACING_FRACTION).max(f32::MIN_POSITIVE)
}

/// Os pontos em que um gesto de `from` a `to` deposita — ver [`walk`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Walk {
    from: [f32; 2],
    to: [f32; 2],
    /// Quantos dabs, ≥ 1 por construção.
    steps: u32,
    next: u32,
    /// A fração do segmento que UM passo vale — `min_spacing / dist`.
    ///
    /// ⚠️ **É ela que separa esta lei da do original.** Lá o passo era `1/n`,
    /// que estica o espaçamento até `2·ms` para o último dab pousar no ponteiro;
    /// aqui ele é fixo, e o que sobra vira a [`Walk::anchor`].
    step: f64,
}

impl Walk {
    /// Quantos dabs este gesto deposita.
    #[must_use]
    pub fn len(self) -> u32 {
        self.steps
    }

    /// Nunca — um `Walk` só existe com pelo menos um dab. Existe para o clippy
    /// e para dizer isso em voz alta.
    #[must_use]
    pub fn is_empty(self) -> bool {
        false
    }

    /// **Onde a âncora do chamador vai parar** — o ÚLTIMO dab, nunca o ponteiro.
    ///
    /// ⚠️ **Esta é a metade da lei que mora no tipo, e a razão é o terceiro
    /// sítio de chamada.** O resíduo entre o último dab e o ponteiro tem de
    /// sobreviver ao evento; escrito `= [x, y]` em cada chamador, ele é uma
    /// convenção que o próximo chamador não conhece — e o modo de falha é
    /// **silencioso** (o traço sai certo, e só a dependência de amostragem
    /// volta). Perguntado ao `Walk`, ele é a resposta de quem sabe.
    ///
    /// ⚠️ **Ele reusa a MESMA aritmética do iterador** (`step · steps` pelo
    /// mesmo `lerp`), e não uma re-derivação `from + steps·ms`: as duas
    /// concordam em álgebra e não em ponto flutuante, e a que importa é a que o
    /// kernel de fato carimbou — senão a âncora deriva do último dab por um ulp
    /// a cada evento, cumulativamente ao longo do traço.
    #[must_use]
    pub fn anchor(self) -> [f32; 2] {
        let t = self.step * f64::from(self.steps);
        [
            lerp(self.from[0], self.to[0], t),
            lerp(self.from[1], self.to[1], t),
        ]
    }
}

impl Iterator for Walk {
    type Item = [f32; 2];

    fn next(&mut self) -> Option<[f32; 2]> {
        if self.next > self.steps {
            return None;
        }
        // ⚠️ **O índice é INTEIRO, e o original acumula em float**
        // (`for (i = step; i <= 1.0; i += step)`). Com o acúmulo, o dab `k` sai
        // de `k` somas encadeadas em vez de um produto — ruído que CRESCE ao
        // longo do gesto, e que é o que a lei do traço existe para eliminar.
        // `k · step` é o mesmo número em qualquer granularidade.
        //
        // ⚠️ **E o último dab NÃO pousa no ponteiro** — ele pousa a
        // `steps · min_spacing` da âncora, que é o que torna a lista função pura
        // do caminho. Quem fica com o resíduo é a [`Walk::anchor`].
        let t = self.step * f64::from(self.next);
        self.next += 1;
        Some([
            lerp(self.from[0], self.to[0], t),
            lerp(self.from[1], self.to[1], t),
        ])
    }
}

fn lerp(a: f32, b: f32, t: f64) -> f32 {
    (f64::from(a) + (f64::from(b) - f64::from(a)) * t) as f32
}

/// Os dabs de um gesto que vai de `from` a `to`.
///
/// `None` significa **carry**: o gesto não andou o bastante para um dab, nada é
/// carimbado e **a âncora do chamador tem de ficar onde está** — é o resíduo
/// acumulando até valer um passo.
///
/// O primeiro ponto devolvido está a um passo de `from` (que já foi carimbado
/// pelo gesto anterior, ou pelo pen-down) e o último é **exatamente** `to`.
#[must_use]
pub fn walk(from: [f32; 2], to: [f32; 2], min_spacing: f32) -> Option<Walk> {
    let dist = ((to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2)).sqrt();
    let ms = min_spacing.max(f32::MIN_POSITIVE);
    // ⚠️ **`<=` FICA, e a tentativa de mover para `<` foi revertida.** Eu a
    // troquei nesta wave argumentando que em `dist == ms` cabe um passo e
    // recusá-lo atrasa o primeiro dab — verdade, e IRRELEVANTE: com a âncora no
    // último dab as duas fronteiras produzem a MESMA lista (`ms` recusado
    // apenas adia o dab para o evento seguinte, que o carimba no mesmo lugar).
    // O gate ao lado declara a escolha como deliberada há mais tempo do que
    // esta wave existe, e mover uma cerca por um argumento que não muda o
    // resultado é como um comportamento se perde.
    if !dist.is_finite() || dist <= ms {
        return None;
    }
    // `floor` e não `round`: com `round` o espaçamento poderia passar de
    // `min_spacing` para BAIXO — e o vão, não o excesso, é o que se vê.
    let steps = (dist / ms).floor().min(f32::from(u16::MAX)) as u32;
    Some(Walk {
        from,
        to,
        steps: steps.max(1),
        next: 1,
        step: f64::from(ms) / f64::from(dist),
    })
}

#[cfg(test)]
#[path = "spacing_tests.rs"]
mod tests;
