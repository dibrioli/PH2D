//! **A FORMA SÓLIDA** — o caminho fechado do gesto vira uma região preenchida, com cobertura
//! **exata por área**.
//!
//! É a metade de motor do `Style: Solid` (o *Style* do Alchemy: *"toggle between drawing with a
//! line or solid fill"*). ⚠️ E o que ela preenche **não é a área varrida pelo pincel** — essa o
//! depósito já dá, como envelope dos dabs. É a **área CERCADA pelo gesto**: um laço fino e solto
//! pinta uma mancha gorda, e a espessura do traço deixa de entrar.
//!
//! # Por que a acumulação de área, e não supersample
//!
//! O composite booleano do Painter rasteriza a `SS = 3` porque o que ele quer é **traçar** o
//! contorno — e um traçado joga a cobertura fora. Medido (`line_probe` do tool, W0 do plano 38),
//! `SS = 3` erra mais de um degrau visível (> 8/255) em **51% dos pixels de borda**, e subir o
//! supersample é quadrático: `SS = 8` custa 6,8× e ainda erra 1,4%.
//!
//! A acumulação de área com sinal — a técnica dos rasterizadores de fonte, aqui escrita a partir da
//! descrição matemática — dá a **área exata** do pixel coberto, numa passada `O(arestas + área)` e
//! **sem buffer supersampleado**. Ou seja: é ao mesmo tempo o melhor resultado possível e o mais
//! barato dos três. Foi por isso que a §5.2 do plano 38 a escolheu.
//!
//! A ideia numa frase: cada aresta deposita, nas células que ela cruza, a **derivada** da cobertura
//! ao longo da linha de varredura (com sinal, pelo sentido em que a aresta sobe ou desce); a soma
//! corrida dessa derivada ao longo da linha **é** a cobertura. Nenhuma amostragem acontece.
//!
//! # A regra de preenchimento
//!
//! `|soma corrida|` limitada a 1 é a regra **não-zero**: um laço que se cruza fica cheio, e dois
//! contornos de sentidos opostos furam um ao outro. É a regra que o SVG e o Illustrator usam por
//! default, e é a que faz um gesto solto ler como uma mancha em vez de um xadrez.

/// A cobertura de uma região fechada, `0..=255` por pixel, sobre uma janela `w × h` cuja origem
/// (canto superior-esquerdo, em pixels de canvas) é `origin`.
///
/// Cada laço é uma polilinha **fechada implicitamente**: o último ponto liga de volta ao primeiro,
/// que é a decisão §5.3 do plano 38 (*"um gesto que não fecha, fecha sozinho"*) escrita onde ela
/// mora. Laços com menos de dois pontos são ignorados — não cercam área nenhuma.
///
/// ⚠️ **A janela leva UMA coluna de folga por dentro.** A aresta deposita massa na célula
/// `x1` (o teto do intervalo que ela cobre), que pode ser a coluna imediatamente à direita da
/// última visível; sem a folga esse depósito cairia na coluna 0 da linha seguinte e desenharia um
/// fantasma na borda esquerda. A folga é interna: a saída tem exatamente `w × h`.
#[must_use]
pub fn fill_coverage(loops: &[Vec<[f32; 2]>], w: usize, h: usize, origin: [f32; 2]) -> Vec<u8> {
    // Piso do pool MEDIDO — ver [`PAR_MIN_AREA`]. A escolha mora AQUI, e a rota em
    // [`fill_coverage_routed`], para um gate poder pedir as duas sobre a MESMA entrada.
    fill_coverage_routed(loops, w, h, origin, w * h >= PAR_MIN_AREA)
}

/// [`fill_coverage`] com a rota da soma corrida ESCOLHIDA pelo chamador.
///
/// ⚠️ **Ela existe para o gate de identidade poder existir**, e não como knob: toda fixture de Solid
/// do repo roda abaixo do piso do pool, então sem esta porta a rota paralela shipava **sem um único
/// teste** — foi exactamente o buraco que o irmão dela no tool já tinha.
fn fill_coverage_routed(
    loops: &[Vec<[f32; 2]>],
    w: usize,
    h: usize,
    origin: [f32; 2],
    par: bool,
) -> Vec<u8> {
    let mut out = vec![0u8; w * h];
    if w == 0 || h == 0 {
        return out;
    }
    let stride = w + 1; // a folga de uma coluna (ver o doc acima)
    let mut acc = vec![0.0f32; stride * h];
    for lp in loops {
        if lp.len() < 2 {
            continue;
        }
        let n = lp.len();
        for i in 0..n {
            let a = [lp[i][0] - origin[0], lp[i][1] - origin[1]];
            let b = [
                lp[(i + 1) % n][0] - origin[0],
                lp[(i + 1) % n][1] - origin[1],
            ];
            accumulate_edge(a, b, stride, h, &mut acc);
        }
    }
    // ── A SOMA CORRIDA É POR LINHA, E AS LINHAS SÃO DISJUNTAS ───────────────────────────────────
    // A cobertura de uma linha é a soma corrida da derivada DAQUELA linha: nada atravessa a
    // fronteira horizontal, e a ordem das somas DENTRO de uma linha é a mesma nas duas rotas ⇒
    // **byte-idêntico por construção**, e não por promessa (ADR-0158). Sem RNG e sem transcendental.
    //
    // ⚠️ **O depósito das ARESTAS fica SERIAL, e o mecanismo é aritmético:** `acc[cell] += d` em
    // `f32` **não é associativo**, então duas arestas que caem na mesma célula têm de ser somadas na
    // mesma ORDEM. Uma banda só preserva isso se a célula pertencer a uma banda só — o que vale — mas
    // o `x` de uma aresta é caminhado INCREMENTALMENTE linha a linha (`x = x_next`), então uma banda
    // que começasse no meio de uma aresta teria de recomputar `x` direto e sairia um `f32` diferente.
    // Bandear as arestas exige que cada banda caminhe a aresta desde o começo dela, e o pré-filtro
    // disso (29 k arestas × 32 bandas) foi MEDIDO como mais caro que o passe serial inteiro.
    if par {
        use rayon::prelude::*;
        out.par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, orow)| sum_row(&acc[y * stride..y * stride + w], orow));
    } else {
        out.chunks_mut(w)
            .enumerate()
            .for_each(|(y, orow)| sum_row(&acc[y * stride..y * stride + w], orow));
    }
    out
}

/// **Onde o pool de threads se paga na soma corrida**, em texels da janela — medido, não escolhido
/// (`ph2d-tool-painter::tool::paint::measure_solid_cost`). É o MESMO piso do kernel de dab, porque a
/// pergunta é a mesma: *quantos texels é preciso percorrer para o fork valer a pena?*
const PAR_MIN_AREA: usize = crate::PARALLEL_MIN_AREA;

/// A cobertura de UMA linha: a soma corrida da derivada, saturada, quantizada.
///
/// ⚠️ **Um corpo, dois walkers** — não existe versão paralela deste laço para divergir da serial; o
/// `par` de [`fill_coverage`] escolhe só quem o percorre.
fn sum_row(acc_row: &[f32], out_row: &mut [u8]) {
    let mut sum = 0.0f32;
    for (a, o) in acc_row.iter().zip(out_row.iter_mut()) {
        sum += *a;
        // Não-zero: o sinal diz o sentido, o módulo diz quanto, e `1` é cheio.
        let c = sum.abs().min(1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            *o = (c * 255.0 + 0.5) as u8;
        }
    }
}

/// Deposita a derivada da cobertura de UMA aresta nas células que ela cruza.
///
/// ⚠️ **Uma aresta horizontal não deposita nada, e isso é a definição, não uma otimização:** ela não
/// cruza linha de varredura nenhuma, então não muda a cobertura de ninguém. Tratá-la como caso
/// especial "por segurança" é o que faz um rasterizador vazar.
fn accumulate_edge(a: [f32; 2], b: [f32; 2], stride: usize, h: usize, acc: &mut [f32]) {
    if (a[1] - b[1]).abs() < f32::EPSILON {
        return;
    }
    // O sinal do sentido: subir e descer se cancelam, que é o que torna a soma corrida um winding.
    let (dir, p0, p1) = if a[1] < b[1] {
        (1.0f32, a, b)
    } else {
        (-1.0f32, b, a)
    };
    let dxdy = (p1[0] - p0[0]) / (p1[1] - p0[1]);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let y_start = p0[1].max(0.0).floor() as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let y_end = (p1[1].ceil().max(0.0) as usize).min(h);
    if y_start >= y_end {
        return;
    }
    // A aresta pode começar acima da janela: avança `x` até a primeira linha visível.
    #[allow(clippy::cast_precision_loss)]
    let mut x = p0[0] + dxdy * ((y_start as f32).max(p0[1]) - p0[1]);
    let last_col = stride - 1;
    for y in y_start..y_end {
        #[allow(clippy::cast_precision_loss)]
        let (top, bot) = (y as f32, y as f32 + 1.0);
        let dy = bot.min(p1[1]) - top.max(p0[1]);
        if dy <= 0.0 {
            continue;
        }
        let x_next = x + dxdy * dy;
        let d = dy * dir;
        let (x0, x1) = if x < x_next { (x, x_next) } else { (x_next, x) };
        // Fora da janela na horizontal: à esquerda a massa toda cai na coluna 0 (a cobertura à
        // direita dela é a mesma), à direita não entra na linha.
        #[allow(clippy::cast_precision_loss)]
        let hi = last_col as f32;
        let x0 = x0.clamp(0.0, hi);
        let x1 = x1.clamp(0.0, hi);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let x0i = (x0.floor() as usize).min(last_col);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let x1i = (x1.ceil() as usize).min(last_col);
        let row = y * stride;
        if x1i <= x0i + 1 {
            // A aresta atravessa uma célula só: reparte pela posição do ponto médio dentro dela.
            #[allow(clippy::cast_precision_loss)]
            let xmf = (0.5 * (x0 + x1) - x0i as f32).clamp(0.0, 1.0);
            acc[row + x0i] += d * (1.0 - xmf);
            acc[row + (x0i + 1).min(last_col)] += d * xmf;
        } else {
            // Várias células: as duas pontas levam a área do triângulo que sobra em cada uma, e o
            // miolo divide o resto em partes iguais.
            let s = (x1 - x0).recip();
            #[allow(clippy::cast_precision_loss)]
            let x0f = x0 - x0i as f32;
            let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
            #[allow(clippy::cast_precision_loss)]
            let x1f = x1 - (x1i as f32 - 1.0);
            let am = 0.5 * s * x1f * x1f;
            acc[row + x0i] += d * a0;
            if x1i == x0i + 2 {
                acc[row + x0i + 1] += d * (1.0 - a0 - am);
            } else {
                let a1 = s * (1.5 - x0f);
                acc[row + x0i + 1] += d * (a1 - a0);
                for xi in (x0i + 2)..(x1i - 1) {
                    acc[row + xi] += d * s;
                }
                #[allow(clippy::cast_precision_loss)]
                let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                acc[row + x1i - 1] += d * (1.0 - a2 - am);
            }
            acc[row + x1i] += d * am;
        }
        x = x_next;
    }
}

/// A caixa inteira que um conjunto de laços toca, em pixels de canvas, já recortada a `w × h`.
/// `None` quando não sobra área nenhuma dentro da tela.
#[must_use]
pub fn loops_bbox(loops: &[Vec<[f32; 2]>], w: usize, h: usize) -> Option<[usize; 4]> {
    let mut bb: Option<[f32; 4]> = None;
    for lp in loops {
        for p in lp {
            bb = Some(bb.map_or([p[0], p[1], p[0], p[1]], |r| {
                [
                    r[0].min(p[0]),
                    r[1].min(p[1]),
                    r[2].max(p[0]),
                    r[3].max(p[1]),
                ]
            }));
        }
    }
    let r = bb?;
    // ⚠️ **A rejeição vem ANTES do clamp, e não é higiene.** Recortar primeiro colapsa toda figura
    // fora da tela num retângulo de 1 px na origem — a caixa passa a existir, o preenchimento roda,
    // e um laço a cem pixels à esquerda do canvas pinta um ponto no canto.
    #[allow(clippy::cast_precision_loss)]
    let (wf, hf) = (w as f32, h as f32);
    if r[2] < 0.0 || r[3] < 0.0 || r[0] >= wf || r[1] >= hf {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let x0 = (r[0].floor().max(0.0) as usize).min(w.saturating_sub(1));
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let y0 = (r[1].floor().max(0.0) as usize).min(h.saturating_sub(1));
    // O pixel que contém `r[2]` é `floor(r[2])`, e o fim é exclusivo ⇒ `+1`, nunca `ceil()+1` (que
    // reclamaria uma coluna a mais em toda figura de coordenada quebrada).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let x1 = ((r[2].floor().max(0.0) as usize) + 1).min(w);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let y1 = ((r[3].floor().max(0.0) as usize) + 1).min(h);
    (x1 > x0 && y1 > y0).then_some([x0, y0, x1 - x0, y1 - y0])
}

#[cfg(test)]
#[path = "solid_tests.rs"]
mod tests;
