//! **O ARO VIRA A QUINA** — a distância à fronteira, para o aro da lavagem parar de sumir na axila
//! de dois traços que se cruzam (Enio, 2026-08-12, com foto; estudo em
//! [`docs/Painter/36_o_entalhe_no_cruzamento.md`]).
//!
//! ## O que estava errado, e é uma frase
//!
//! O aro é um **unsharp**: `edge = gain·(cw − inner)` com `inner = blur(hard)`. Um borrão linear
//! mede a **FRAÇÃO da vizinhança que está FORA**, e o aro quer a **DISTÂNCIA à fronteira**. Num
//! flanco reto as duas coincidem; numa quina elas divergem, e o sinal da divergência é geométrico:
//!
//! | | exterior que o disco vê | efeito |
//! |---|---|---|
//! | flanco reto | ~1/2 | o aro que o modelo calibra |
//! | quina **côncava**, por dentro | ~1/4 | aro **METADE** — ele some antes de virar a quina |
//! | quina côncava, por fora | ~3/4 de interior | lobo pálido **1,5×** — a franja come a tinta vizinha |
//!
//! Medido na cruz do report (`crossing_probe`): o ombro solitário na borda vale **0,624** e a axila
//! **0,282** — défice **0,34 ≈ 87/255**. É a cunha branca da foto.
//!
//! ## A cura, e por que ela é um TETO e não uma substituição
//!
//! `inner` passa a ser **limitado pelo que um flanco RETO daria à mesma distância**:
//!
//! ```text
//!   inner := min(blur(hard), P(sd, r))
//!   P(sd, r) = clamp((sd + r + 0.5) / (2r + 1), 0, 1)   // a resposta do box blur a um degrau
//! ```
//!
//! **Teto** (`min`) e não substituição porque a correção que a quina côncava precisa **só tem um
//! sinal**: ela precisa de MAIS aro e de MENOS franja, e as duas saem de um `inner` menor. Um `min`
//! nunca pode enfraquecer o aro em lugar nenhum — o pior que ele faz é não agir. `inner := P`
//! reescreveria todo flanco reto.
//!
//! ⚠️ **E o teto vale nos DOIS lados — a versão só-por-dentro foi construída, medida e é METADE da
//! cura.** O raciocínio que a defendia (*"por fora o teto mexeria no lobo pálido de TODA borda"*)
//! estava direcionalmente certo e **errado na magnitude**, e as duas metades da medição são estas:
//!
//! * **Só por dentro NÃO fecha a cunha.** O aro engrossa ao APROXIMAR-SE da quina (mapa: `44543` →
//!   `66542`) e o vão continua exatamente onde estava, porque o vão está FORA (`sd < 0`), onde o
//!   teto era inerte. Pior: fortalecer o aro ao lado de um vão intocado **aumenta o contraste** que
//!   faz a cunha ser vista.
//! * **Nos dois lados o vão FECHA** — no mapa da cruz, `2222` vira `333333` e o buraco de 5 px cai
//!   para 2 — e o preço num flanco RETO sobre tinta existente é **≤ 10/255 num único pixel** (o
//!   último do ombro), com os outros quinze byte-idênticos, e na direção certa (menos pálido).
//!
//! ⚠️ **Fora da lavagem o composite nem escreve**, então o lobo pálido só existe no OMBRO — é por
//! isso que o preço é de um pixel e não de uma banda. O EDGE-3 (Curtis §4.3.3) fica: o lobo
//! negativo continua lá, apenas deixa de ser amplificado por concavidade.
//!
//! ## A EDT é a que já existe
//!
//! [`super::sculpt_close::distance_inside`] é a EDT exata (Felzenszwalb-Huttenlocher, `O(área)`,
//! separável) que o fechamento do Inflate usa, e o doc dela diz que um segundo consumidor sempre foi
//! a intenção: *"duas cópias de uma EDT exata seriam duas respostas a que distância é esta"*. Esta é
//! a terceira, e ela não traz kernel novo.

use super::Region;
use super::sculpt_close::distance_inside;

/// O limiar que define a fronteira da lavagem: a meia-altura da cobertura endurecida.
const HALF: f32 = 0.5;

/// **OS CAMPOS DO ARO** — a cobertura endurecida, o conjunto de borrões que dá o `inner`, e a régua
/// do teto. Nascem juntos, só o aro os lê, e por isso a receita mora aqui e não no composite.
pub(super) struct RimFields {
    /// Um borrão por `core_r` DISTINTO entre os donos (quase sempre um só).
    pub(super) blurs: Vec<(usize, Vec<f32>)>,
    /// A régua do teto — `None` quando ninguém na janela tem aro (ver [`field_for`]).
    pub(super) sd: Option<Vec<f32>>,
}

/// Constrói [`RimFields`] a partir da cobertura CRUA da janela.
pub(super) fn rim_fields(
    cov: &[f32],
    rw: usize,
    rh: usize,
    core_r: usize,
    table: &[super::watercolor_field::WetStrokeStyle],
    brush: &ph2d_painter_brush::BrushSpec,
) -> RimFields {
    // The feather blur (`inner`) must SATURATE to ~1 in a pool's core — `core_r` caps it at
    // ~half the brush (a wider Spread otherwise read the whole pool as "rim" and the edge
    // flooded the centre, Enio 2026-07-07); `spread <= radius/2` is a no-op. EDGE-3: blur of
    // the HARDENED coverage (the raw plateau shifted the interior tone with Edge). One blur
    // per DISTINCT per-owner core_r (usually one): a baked wash keeps ITS feather — the live
    // brush's radius re-blurred the neighbour's rim (doc 13 "mudanca no brush propaga").
    let hard: Vec<f32> = cov
        .iter()
        .map(|&c| {
            super::watercolor_field::smoothstep(
                super::watercolor_render::SS0,
                super::watercolor_render::SS1,
                c,
            )
        })
        .collect();
    let blurs = super::watercolor_rewet_px::inner_blur_set(&hard, rw, rh, table, core_r);
    let sd = field_for(&hard, cov, rw, rh, core_r, table, brush);
    RimFields { blurs, sd }
}

/// **O CAMPO DE RÉGUA do teto do aro — e a decisão de construí-lo.**
///
/// O ARO VIRA A QUINA (doc 36): a distância assinada à fronteira, que limita o `inner` ao que
/// um flanco RETO daria à mesma distância. Sem ela, o borrão linear mede a FRAÇÃO da
/// vizinhança que está fora em vez da DISTÂNCIA, e numa quina côncava o aro some (medido:
/// ombro 0,624 contra axila 0,282 — a cunha branca do report).
///
/// ⚠️ **Só é computada se ALGUM dono tem aro.** `edge_gain = 0` em toda a sessão ⇒ o teto não
/// teria consumidor, e a EDT seria trabalho por quadro que ninguém lê. O `any` cobre
/// a tabela de donos E o estilo vivo, porque um dos dois pode ser o único com aro.
/// ⚠️ O lado VIVO é lido do pincel AUTORADO (`brush.edge_gain`), não do `cur_style`, que só
/// nasce 100 linhas abaixo. É conservador de propósito: com Dilution 1 o `wash_flow` zera o
/// ganho e o campo seria computado à toa, mas nunca acontece o contrário.
///
/// ⚠️ **E ele é REDUNDANTE hoje — medido, não suposto.** A mutação que o remove **não sangra**
/// em nenhuma das duas cenas de quina (nem na cruz de dois traços, nem no PRIMEIRO traço da
/// sessão): quando um composite tem aro, o estilo já está na tabela. Ele fica porque a
/// pergunta certa é *"algum estilo que ESTE composite vai usar tem aro?"*, e a tabela sozinha
/// é meia resposta — mas o doc diz o que a medição diz, e não que está gateado.
pub(super) fn field_for(
    hard: &[f32],
    cov: &[f32],
    rw: usize,
    rh: usize,
    core_r: usize,
    table: &[super::watercolor_field::WetStrokeStyle],
    brush: &ph2d_painter_brush::BrushSpec,
) -> Option<Vec<f32>> {
    let wants_rim = table.iter().any(|s| s.edge_gain > 0.0) || brush.edge_gain > 0.0;
    wants_rim
        .then(|| signed_distance(hard, cov, rw, rh, core_r))
        .flatten()
}

/// A distância ASSINADA (px) à fronteira `hard = 0.5`, positiva DENTRO. `None` quando a janela não
/// tem fronteira nenhuma (tudo dentro ou tudo fora) — ali não há aro a corrigir e o chamador pula o
/// teto inteiro em vez de carregar um campo constante.
pub(super) fn signed_distance(
    hard: &[f32],
    cov: &[f32],
    rw: usize,
    rh: usize,
    core_r: usize,
) -> Option<Vec<f32>> {
    let n = rw * rh;
    if n == 0 || hard.len() < n {
        return None;
    }
    let mut inside = vec![0u8; n];
    let mut any_in = false;
    let mut any_out = false;
    for (m, &h) in inside.iter_mut().zip(hard.iter()) {
        if h >= HALF {
            *m = 255;
            any_in = true;
        } else {
            any_out = true;
        }
    }
    if !any_in || !any_out {
        return None;
    }
    let region = Region {
        x: 0,
        y: 0,
        w: rw as u32,
        h: rh as u32,
    };
    // UMA EDT, e o sinal vem da máscara.
    //
    // ⚠️ **A 1ª versão chamava a porta DUAS vezes** (distância de dentro ao fora, e a gêmea sobre o
    // complemento) — e medido contra o `box_blur` que o composite já paga isso custava **3,67
    // borrões**, no caminho quente. A pergunta certa é mais barata: a distância ao **CONJUNTO
    // FRONTEIRA** é UMA transformada, e de que lado o pixel está a máscara já responde de graça.
    // Medido depois da troca, costas-com-costas na mesma corrida: **1,69 borrões**.
    //
    // A semente é o pixel que TEM vizinho do outro lado (4-vizinhança): é a definição de fronteira
    // discreta, e é ela que faz `sd ≈ 0` na borda em vez de `±0,5` conforme o lado.
    let mut seed = vec![255u8; n];
    for y in 0..rh {
        for x in 0..rw {
            let i = y * rw + x;
            let me = inside[i] != 0;
            let border = (x > 0 && (inside[i - 1] != 0) != me)
                || (x + 1 < rw && (inside[i + 1] != 0) != me)
                || (y > 0 && (inside[i - rw] != 0) != me)
                || (y + 1 < rh && (inside[i + rw] != 0) != me);
            if border {
                seed[i] = 0;
            }
        }
    }
    let d = distance_inside(&seed, rw, region, 128);
    if d.len() < n {
        return None;
    }
    // A SEGUNDA RÉGUA (doc 36 §10): a distância à frente MAIS PRÓXIMA, lida da própria cobertura.
    // As duas entram por `min` — como o teto, ela só pode DAR aro, nunca tirar.
    //
    // ⚠️ **Fundida NESTE laço, e só na FAIXA onde o teto pode agir** — e as duas metades da faixa não
    // são simétricas, o que é o único jeito de o corte ser provado inerte em vez de escolhido:
    //
    // * `geom ≤ −(r+½)` ⇒ `P` já satura em 0, e `min(geom, front) ≤ geom` ⇒ satura também. Inerte.
    // * `geom ≥ (r+1½)·2` ⇒ nem a maior correção que a régua pode fazer nesta faixa desce o argumento
    //   abaixo de `r+½`, onde `P` satura em 1. Inerte.
    //
    // ⚠️ **O `2` é um LIMITE NOMEADO, não um arredondamento:** numa quina de ângulo `θ` a frente está
    // a `geom·sen(θ/2)`, e `2` cobre até **60°** — dois traços cruzando mais agudo que isso mantêm o
    // aro de hoje na quina. O `½` extra é a discretização: a EDT mede até o pixel de fronteira, meio
    // pixel além do nível `0,5` que a régua lê, e com `√2` cravado o gate de `t = 7` caía **fora** da
    // faixa por 0,02 px.
    //
    // Fora dessa faixa a régua nem é calculada — e é o corte que a torna barata, porque a faixa é o
    // PERÍMETRO e não a janela. Medido pela sonda `the_coverage_ruler_costs_this_many_blurs`,
    // intercalado e com o mínimo como redutor: **≈1 borrão** varrida a janela inteira contra
    // **0,16 borrão** com o corte.
    let lo = -(core_r as f32 + 0.5);
    let hi = (core_r as f32 + 1.5) * 2.0;
    Some(
        (0..n)
            .map(|i| {
                let geom = if inside[i] != 0 { d[i] } else { -d[i] };
                if geom <= lo || geom >= hi {
                    return geom;
                }
                geom.min(coverage_distance_at(cov, rw, rh, i))
            })
            .collect(),
    )
}

/// Abaixo disto o campo é chato (miolo saturado ou papel nu) e não há frente por perto: a régua da
/// cobertura se cala e o teto fica com a geométrica.
const FLAT: f32 = 1.0e-4;

/// O valor de cobertura que o endurecimento leva a `0,5` — a MESMA fronteira que a EDT semeia.
/// `smoothstep(SS0, SS1, cov) = 0.5` ⟺ `cov = (SS0 + SS1)/2`.
const COV_HALF: f32 = (super::watercolor_render::SS0 + super::watercolor_render::SS1) * 0.5;

/// **A PROFUNDIDADE QUE A COBERTURA IMPLICA** — a régua que a geométrica não sabe dar numa quina
/// côncava (doc 36 §9–§10).
///
/// ## O que a geométrica erra, e por quê
///
/// A cobertura da lavagem é um **`max`** sobre os depósitos, então o VALOR dela num texel é o da faixa
/// que o cobre MELHOR: `cov = f(profundidade nessa faixa)`. E é essa profundidade que decide o TOM da
/// silhueta ali — o mesmo tom que o flanco reto tem à mesma profundidade.
///
/// ⚠️ **Ela não é a distância à frente mais próxima, e a diferença importa:** numa quina côncava não
/// existe frente a `t` px (o ponto a `t` do eixo de um braço está DENTRO do outro), e a fronteira da
/// união fica a **`t·√2`**. As três leituras discordam, e a que o artista vê é a da COBERTURA — é ela
/// que diz "aqui é a beirada". Medido: com `spread/raio = 0,09` o aro da quina vale **dois terços**
/// do aro do flanco reto, e a 110 px ele **rompe** — a cunha branca da foto do Enio.
///
/// ## A régua
///
/// `cov = COV_HALF` **é** a fronteira (é o mesmo nível que a EDT semeia, via o endurecimento), então a
/// profundidade é a estimativa analítica clássica `(cov − COV_HALF) / |∇cov|` — imune à quina, porque
/// lê o VALOR e não a forma do nível.
///
/// ⚠️ **E lê a cobertura CRUA, não a endurecida:** com `hard` a régua morre onde o `smoothstep` satura
/// (±6 px num pincel grande) — exatamente onde o aro vive. Medida com `hard` a cura valia 4 pontos
/// percentuais; com `cov` ela vale 20.
///
/// ⚠️ **O termo de UM LADO não é higiene, é a CRISTA:** na bissetriz exata de um `max` de dois campos
/// lisos a diferença central mede **metade** do que deveria (para a frente o campo é constante, para
/// trás ele sobe), e `|∇|` sai `√2` pequeno demais — exatamente na linha que a quina tem. Tomar
/// também as quatro derivadas de UM LADO e ficar com a MAIOR devolve `|f'|` na crista **e** concorda
/// com a central em toda parte lisa. Sem ele a cura teria uma costura de 1 px correndo pela bissetriz,
/// que é a forma de trocar uma cunha por uma rachadura.
///
/// ⚠️ **E `|∇|` (não o máximo por eixo) é quem serve o flanco INCLINADO:** numa borda a 45° as duas
/// derivadas de um lado valem `|f'|/√2` e só a norma do gradiente devolve `|f'|`. As duas metades do
/// `max` cobrem casos opostos, e nenhuma sozinha é isotrópica.
///
/// Devolve `+∞` onde não há frente (campo chato) — o `min` do chamador o descarta.
#[inline]
fn coverage_distance_at(cov: &[f32], rw: usize, rh: usize, i: usize) -> f32 {
    let (x, y) = (i % rw, i / rw);
    if x == 0 || y == 0 || x + 1 >= rw || y + 1 >= rh {
        return f32::INFINITY;
    }
    let h = cov[i];
    let (l, r) = (cov[i - 1], cov[i + 1]);
    let (u, d) = (cov[i - rw], cov[i + rw]);
    let gx = (r - l) * 0.5;
    let gy = (d - u) * 0.5;
    let g = (gx * gx + gy * gy)
        .sqrt()
        .max((h - l).abs())
        .max((r - h).abs())
        .max((h - u).abs())
        .max((d - h).abs());
    if g > FLAT {
        (h - COV_HALF) / g
    } else {
        f32::INFINITY
    }
}

/// **O TETO** — o `inner` que um flanco RETO daria a esta distância, para o raio de borrão `r`.
///
/// É a resposta analítica do box blur (lado `2r+1`) a um degrau: uma rampa linear de 0 em
/// `sd = −(r + 0.5)` a 1 em `sd = +(r + 0.5)`, valendo exatamente `0,5` na fronteira.
///
/// Devolve `1.0` (teto inerte) com `r = 0` — sem borrão não há o que limitar —, respondido aqui e
/// não no chamador, para o `min` do composite ser incondicional.
#[inline]
pub(super) fn straight_edge_cap(sd: f32, r: usize) -> f32 {
    if r == 0 {
        return 1.0;
    }
    let r = r as f32;
    ((sd + r + 0.5) / (2.0 * r + 1.0)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O teto vale meia-altura NA fronteira e satura a `r + 0.5` — os dois pontos que o tornam a
    /// resposta do box blur a um degrau, e não uma rampa escolhida.
    #[test]
    fn the_cap_is_the_box_blurs_own_straight_edge_ramp() {
        let r = 6usize;
        // NA fronteira o box blur ve metade do kernel dentro.
        assert!(
            (straight_edge_cap(0.0, r) - 0.5).abs() < 1e-6,
            "na fronteira: {}",
            straight_edge_cap(0.0, r)
        );
        // A rampa e' SIMETRICA em torno da fronteira — e' o mesmo degrau visto dos dois lados.
        for d in [1.0f32, 3.0, 5.0] {
            let a = straight_edge_cap(d, r);
            let b = straight_edge_cap(-d, r);
            assert!((a + b - 1.0).abs() < 1e-6, "assimetrica em {d}: {a} vs {b}");
        }
        // Satura em `±(r + 0.5)` e nunca sai de [0,1].
        assert!((straight_edge_cap(r as f32 + 0.5, r) - 1.0).abs() < 1e-6);
        assert!(straight_edge_cap(-(r as f32) - 0.5, r).abs() < 1e-6);
        assert!((straight_edge_cap(100.0, r) - 1.0).abs() < 1e-6);
        assert!(straight_edge_cap(-100.0, r).abs() < 1e-6);
        // `r = 0` nao tem borrao, logo nao tem teto.
        assert!((straight_edge_cap(3.0, 0) - 1.0).abs() < 1e-6);
        assert!((straight_edge_cap(-3.0, 0) - 1.0).abs() < 1e-6);
    }

    /// A distância assinada é positiva dentro, negativa fora, e ~0 na fronteira.
    #[test]
    fn the_signed_distance_knows_which_side_it_is_on() {
        let (rw, rh) = (32usize, 32usize);
        // Faixa horizontal: |y − 16| < 8.
        let hard: Vec<f32> = (0..rw * rh)
            .map(|i| {
                let y = (i / rw) as i32;
                if (y - 16).abs() < 8 { 1.0 } else { 0.0 }
            })
            .collect();
        let sd = signed_distance(&hard, &hard, rw, rh, 7).expect("a faixa tem fronteira");
        let at = |x: usize, y: usize| sd[y * rw + x];
        assert!(at(16, 16) > 6.0, "o miolo esta fundo: {}", at(16, 16));
        assert!(at(16, 2) < -4.0, "fora e' negativo: {}", at(16, 2));
        assert!(at(16, 9).abs() < 2.0, "perto da borda: {}", at(16, 9));
    }

    /// A largura da rampa da fixture (px) e onde ela cruza `0,5`.
    const W: f32 = 24.0;
    const S: f32 = 40.0;
    const DIM: usize = 96;

    /// A COBERTURA de DUAS faixas ortogonais, composta por `max` — a lavagem em miniatura. A faixa
    /// horizontal ocupa `y ≤ S`, a vertical `x ≤ S`; a quina côncava fica em `(S, S)`.
    ///
    /// A rampa é LINEAR de propósito: com ela `(cov − COV_HALF)/|∇cov|` vale `S − s` **exatamente**,
    /// então o gate mede a régua e não o erro de curvatura de um falloff.
    fn two_bands_cov() -> Vec<f32> {
        let ramp = |s: f32| (COV_HALF + (S - s) / W).clamp(0.0, 1.0);
        (0..DIM * DIM)
            .map(|i| {
                let (x, y) = ((i % DIM) as f32, (i / DIM) as f32);
                ramp(x).max(ramp(y))
            })
            .collect()
    }

    /// O endurecimento que o composite aplica — a fronteira que a EDT semeia é `hard = 0.5`.
    fn harden(cov: &[f32]) -> Vec<f32> {
        cov.iter()
            .map(|&c| {
                super::super::watercolor_field::smoothstep(
                    super::super::watercolor_render::SS0,
                    super::super::watercolor_render::SS1,
                    c,
                )
            })
            .collect()
    }

    /// As duas metades que `signed_distance` consome, da mesma fixture.
    fn two_bands() -> (Vec<f32>, Vec<f32>) {
        let cov = two_bands_cov();
        let hard = harden(&cov);
        (hard, cov)
    }

    /// **O GATE da régua da cobertura — a quina côncava mede o que o flanco mede.**
    ///
    /// A `hard` de duas faixas vale `f(min(dx, dy))`, então um ponto na bissetriz a `t` px de CADA
    /// frente tem **a mesma cobertura** que um ponto do flanco reto a `t` px da sua — e é isso que o
    /// aro tem de ver. A régua GEOMÉTRICA discorda: da fronteira da união aquele ponto está a `t·√2`,
    /// porque o vizinho mais próximo é o vértice da quina.
    ///
    /// ⚠️ **O CONTROLE está na própria asserção:** a fixture só prova alguma coisa se `t·√2` estiver
    /// FORA da tolerância — senão as duas réguas seriam indistinguíveis e o gate passaria por vácuo.
    ///
    /// **Mutações que têm de sangrar:** tirar o `.min(front[i])` de [`signed_distance`] (a quina volta
    /// a `t·√2`) · tirar os quatro termos de UM LADO de [`coverage_distance`] (na crista do `max` a
    /// diferença central mede metade, `|∇|` sai `√2` pequeno, e a quina volta a `t·√2`).
    #[test]
    fn the_concave_corner_measures_what_the_straight_flank_measures() {
        let (hard, cov) = two_bands();
        let sd = signed_distance(&hard, &cov, DIM, DIM, 7).expect("a fixture tem fronteira");
        let at = |x: usize, y: usize| sd[y * DIM + x];
        // ⚠️ `t` fica DENTRO da faixa em que o teto age (`t <= core_r + 1/2`): alem dela `P`
        // satura em 1 e a regua nao e' sequer calculada — o que o teste seguinte afirma como propriedade.
        for t in [4.0f32, 6.0, 7.0] {
            let inside = (S - t) as usize;
            // Flanco reto: longe da outra faixa (a rampa dela ja' zerou), borda VERTICAL.
            let far = (S + W) as usize;
            let flank = at(inside, far);
            // Axila: `t` px de CADA frente — e EXATAMENTE na bissetriz, que e' a crista do `max`.
            let pit = at(inside, inside);
            // Um passo FORA da bissetriz: a crista tem 1 px de largura e o gate tem de cobrir os dois.
            // ⚠️ Ali a cobertura é a da profundidade `t + 1` (o `max` fica com a faixa que cobre
            // MELHOR), então o flanco com que ele se compara é o de `t + 1` — a propriedade é *mesma
            // cobertura, mesmo aro*, e não *mesma distância à frente mais próxima*.
            let off = at(inside, inside - 1);
            let flank_off = at((S - t - 1.0) as usize, far);
            let geom = t * std::f32::consts::SQRT_2;
            assert!(
                (geom - t) > 1.0,
                "a fixture nao contem o fenomeno: t={t} e t*raiz2={geom} estao dentro da tolerancia"
            );
            assert!(
                (pit - flank).abs() < 1.0,
                "t={t}: a axila mede {pit:.2} e o flanco {flank:.2} (a geometrica daria {geom:.2})"
            );
            assert!(
                (off - flank_off).abs() < 1.0,
                "t={t}: fora da bissetriz a axila mede {off:.2} contra o flanco de mesma \
                 cobertura {flank_off:.2}"
            );
        }
    }

    /// **Fora da faixa em que o teto age, a régua NÃO é calculada — e isso é INERTE, não um atalho.**
    ///
    /// Acima de `core_r + ½` o `P` do teto satura em 1 e o `min` do composite não tem o que limitar;
    /// abaixo de `−(core_r + ½)` ele satura em 0 e a régua só poderia baixar ainda mais. O corte é o
    /// que faz a régua custar **0,16 borrão** em vez de ≈1 (a faixa é o perímetro, não a janela), e
    /// aqui ele é afirmado pelo que o CHAMADOR vê: as duas leituras dão o mesmo teto.
    #[test]
    fn beyond_the_caps_reach_the_two_rulers_give_the_same_cap() {
        let (hard, cov) = two_bands();
        let sd = signed_distance(&hard, &cov, DIM, DIM, 7).expect("a fixture tem fronteira");
        let at = |x: usize, y: usize| sd[y * DIM + x];
        let t = 12.0f32;
        let inside = (S - t) as usize;
        let pit = at(inside, inside);
        let flank = at(inside, (S + W) as usize);
        assert!(
            (pit - flank).abs() > 1.0,
            "a fixture nao contem o fenomeno: as duas leituras ja concordam ({pit:.2} vs {flank:.2})"
        );
        assert_eq!(straight_edge_cap(pit, 7), straight_edge_cap(flank, 7));
        assert_eq!(straight_edge_cap(pit, 7), 1.0);
    }

    /// **E o flanco reto NÃO é sequestrado pela régua nova.** Ela entra por `min`, então o risco é ela
    /// morder onde a geometria já estava certa — o que fortaleceria o aro de TODA borda em vez de só
    /// da quina. Num flanco a estimativa analítica e a EDT medem a mesma coisa, e o gate exige isso.
    #[test]
    fn the_straight_flank_keeps_its_geometric_reading() {
        let (hard, cov) = two_bands();
        let sd = signed_distance(&hard, &cov, DIM, DIM, 7).expect("a fixture tem fronteira");
        let far = (S + W) as usize;
        for t in [0.0f32, 2.0, 4.0, 6.0, 9.0] {
            let got = sd[far * DIM + (S - t) as usize];
            assert!(
                (got - t).abs() < 0.75,
                "flanco a t={t}: a regua devolveu {got:.2}"
            );
        }
    }

    /// **O PREÇO da régua da cobertura, na moeda que o composite já paga.**
    ///
    /// Wall-clock absoluto nesta máquina não decide nada (ela é compartilhada), então a medida é uma
    /// RAZÃO contra o `box_blur` que o composite já roda várias vezes por janela — a mesma régua com
    /// que a EDT foi precificada (**1,69 borrões**, doc 36 §6).
    ///
    /// `cargo test -p ph2d-tool-painter --release the_coverage_ruler_costs -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda; roda com --ignored --nocapture"]
    fn the_coverage_ruler_costs_this_many_blurs() {
        use std::time::Instant;
        const D: usize = 512;
        let cov: Vec<f32> = {
            let ramp = |s: f32| (COV_HALF + (200.0 - s) / 24.0).clamp(0.0, 1.0);
            (0..D * D)
                .map(|i| {
                    let (x, y) = ((i % D) as f32, (i / D) as f32);
                    ramp(x).max(ramp(y))
                })
                .collect()
        };
        let hard = harden(&cov);
        // ⚠️ Esta máquina é COMPARTILHADA e o relógio dela deriva 4× sob carga. As três medidas são
        // INTERCALADAS (a carga vira fator comum) e o redutor é o MÍNIMO — que é o certo aqui porque
        // toda amostra faz exatamente o mesmo trabalho.
        let bench = |f: &dyn Fn()| -> f64 {
            let mut best = f64::INFINITY;
            for _ in 0..12 {
                let t0 = Instant::now();
                f();
                best = best.min(t0.elapsed().as_secs_f64() * 1000.0);
            }
            best
        };
        let mut b = (f64::INFINITY, f64::INFINITY, f64::INFINITY);
        for _ in 0..6 {
            b.0 = b.0.min(bench(&|| {
                std::hint::black_box(super::super::watercolor_field::box_blur(&hard, D, D, 7));
            }));
            b.1 = b.1.min(bench(&|| {
                std::hint::black_box(signed_distance(&hard, &cov, D, D, 0));
            }));
            b.2 = b.2.min(bench(&|| {
                std::hint::black_box(signed_distance(&hard, &cov, D, D, 7));
            }));
        }
        let bench = |label: &str, ms: f64| -> f64 {
            println!("   {label:36} {ms:7.3} ms");
            ms
        };
        let blur = bench("box_blur(r=7)", b.0);
        // ⚠️ O que interessa é o DELTA que o produto paga, não o custo da régua rodada sozinha sobre a
        // janela inteira — ela só roda na FAIXA. `core_r = 0` fecha a faixa a quase nada e serve de
        // linha de base sem precisar de uma segunda implementação para divergir.
        let base = bench("signed_distance (faixa fechada)", b.1);
        let whole = bench("signed_distance (r = 7, o produto)", b.2);
        println!(
            "   => a EDT custa {:.2} borroes; a regua acrescenta {:.2}",
            base / blur,
            (whole - base) / blur
        );
    }

    /// Janela sem fronteira ⇒ `None`: o chamador pula o teto em vez de carregar um campo constante.
    #[test]
    fn a_window_with_no_boundary_has_no_cap() {
        assert!(signed_distance(&vec![1.0f32; 64], &vec![1.0f32; 64], 8, 8, 7).is_none());
        assert!(signed_distance(&vec![0.0f32; 64], &vec![0.0f32; 64], 8, 8, 7).is_none());
    }
}
