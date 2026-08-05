//! **O ESPAÇAMENTO** — quantos dabs um gesto deposita, e onde.
//!
//! A [`crate::SculptStroke`] garante que o efeito de um traço não depende de
//! quão fino o motor amostrou o caminho. Isto aqui responde a outra metade da
//! mesma frase: **o caminho tem de ser amostrado**. Um evento de ponteiro que
//! salta 40 px deixaria um vão de 40 px entre dois dabs, e nenhuma lei de
//! envelope preenche o que ninguém carimbou.
//!
//! A lei é a do SculptGL (`SculptBase.js:126-151`), e as três partes dela são
//! independentes:
//!
//! ```text
//! dist = |mouse − âncora|
//! if dist <= min_spacing { return }          // ⇐ o CARRY: não carimba, e a âncora FICA
//! n = floor(dist / min_spacing)              // n dabs, espaçados dist/n ∈ [ms, 2·ms)
//! …carimba…
//! âncora = mouse                             // o ponteiro REAL, não o último dab
//! ```
//!
//! ⚠️ **O carry é o que torna o gesto lento igual ao rápido.** Sem ele, um
//! movimento de 2 px carimbaria um dab a 2 px do anterior, e mover o mouse
//! devagar depositaria dez vezes mais dabs pelo mesmo caminho. Com ele, o
//! resíduo se ACUMULA até valer um passo — e é por isso que a âncora não pode
//! andar quando nada foi carimbado.
//!
//! ⚠️ **Ele está no TIPO, não numa convenção:** [`walk`] devolve `None` para o
//! caso do carry, então "esqueci de não mover a âncora" não é um erro que se
//! comete distraído — é um `match` que não compila pela metade.
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

/// **QUANTO UM DAB SOMA ao envelope com o Accumulate armado** — a normalização
/// que torna a soma um fato do CAMINHO.
///
/// ⚠️ **Ela mora aqui, ao lado do espaçamento, porque É o espaçamento.** Com o
/// Accumulate a lei deixa de ser um envelope (`max`) e passa a ser uma SOMA
/// sobre a lista de dabs — e uma soma crua é exatamente a doença que este módulo
/// existe para não ter: ela depende de quantos dabs o motor emitiu, e um pincel
/// que ficasse 2× mais forte porque alguém afinou o espaçamento é um bug que
/// ninguém consegue nomear. A cura é a mesma que a `line/Painter` formulou no
/// doc 20: **uma INTEGRAL DE LINHA** (`∫ perfil · ds`), não `Σ perfil`.
///
/// Aqui ela é barata porque o passo já é geométrico: `Δs = MIN_SPACING_FRACTION
/// · r`, então `Δs / (2r)` — a fração do DIÂMETRO que um dab percorre — é a
/// constante `MIN_SPACING_FRACTION / 2`, independente do tamanho do pincel. Uma
/// passada reta pelo centro soma `∫ falloff ds / 2r`, que é a média do falloff
/// sobre a corda; a segunda passada soma outra vez, e é isso que o Accumulate
/// significa.
///
/// ⚠️ **Consequência MEDIDA, não estimada** (ver o gate
/// `the_first_accumulated_pass_is_weaker_than_the_envelope`): a primeira passada
/// fica mais FRACA que a do envelope, porque o envelope entrega o pico do
/// falloff e a integral entrega a média dele. É o preço honesto de a lei ser uma
/// soma — e é a partir da segunda passada que ela paga.
pub const ACCUM_PER_DAB: f32 = MIN_SPACING_FRACTION / 2.0;

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
}

impl Iterator for Walk {
    type Item = [f32; 2];

    fn next(&mut self) -> Option<[f32; 2]> {
        if self.next > self.steps {
            return None;
        }
        // ⚠️ **O índice é INTEIRO, e o original acumula em float**
        // (`for (i = step; i <= 1.0; i += step)`). Com o acúmulo, o último dab
        // cai perto de `to` em vez de EM `to`, e às vezes some inteiro quando a
        // soma passa de 1.0 por um ulp — ruído de amostragem, que é justamente
        // o que a lei do traço existe para eliminar. Aqui `i = steps` dá `t = 1`
        // exato, logo o último dab pousa no ponteiro. Divergência registrada no
        // livro-razão.
        let t = f64::from(self.next) / f64::from(self.steps);
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
    })
}

#[cfg(test)]
#[path = "spacing_tests.rs"]
mod tests;
