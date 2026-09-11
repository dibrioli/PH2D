//! **O AJUSTE — quais pontos o traço guarda.**
//!
//! ⚠️ Irmão do [`super`], e o corte é por assunto: lá mora *o que se faz com uma lista de pontos*
//! (alisar, reamostrar); aqui, ***quantos pontos ela deve ter***.
//!
//! # A LEI — contra a curva que será DESENHADA, e da esquerda para a direita
//!
//! ## ⚠️ A propriedade que este ajuste tem e o anterior não tinha: ele é PREFIXO-ESTÁVEL
//!
//! Report do Enio (2026-07-30, terceira rodada): *"o traço deve parar de reconstruir a curva toda;
//! os pontos já postos não devem ser modificados"*.
//!
//! A versão anterior era **divide-e-conquista com fila por pior erro**: a primeira partição saía do
//! máximo **GLOBAL**, então uma amostra nova na ponta podia re-decidir um corte no COMEÇO — e o
//! preview roda o ajuste **a cada frame**. Medido no pipeline do produto, traço de 1200 amostras
//! (`measure_how_often_an_already_placed_point_is_moved`): **762 de 1001 frames** mexiam em pontos
//! já postos, atrás do cursor. O deslocamento por frame é pequeno (~1 % da espessura), e é
//! exatamente por isso que ele **não** foi curado pelas duas rodadas anteriores: o que o artista vê
//! não é a amplitude, é 1 % **mudando a cada frame** sobre um traço que ele já considera pronto.
//!
//! Andar da esquerda para a direita apaga a classe inteira: **a decisão de um nó só olha amostras
//! que já chegaram**, então `fit(amostras[..n])` é prefixo de `fit(amostras[..m])` para todo
//! `m ≥ n`, a menos do último span — o que está literalmente sob a caneta. Não é uma calibração
//! melhor, é a propriedade que faltava.
//!
//! ## Como
//!
//! Do último nó guardado, estende o span **enquanto a reconstrução couber na tolerância**, e fecha o
//! nó no último ponto que coube. A reconstrução vem da porta do produto ([`resample_smooth`], a
//! MESMA que o traço usa) — uma cópia da avaliação da curva aqui recriaria o desacordo que este
//! ajuste existe para acabar (era o defeito da rodada 1: o `simplify_rdp` cobrava contra a CORDA
//! RETA enquanto quem desenha traça uma Catmull-Rom, e num gancho fechado a corda parecia ótima
//! com o traço final a **8,46 %** da espessura de onde a mão passou).
//!
//! ⚠️ **A tangente do fim do span sai do [`espelho`], nunca de um nó futuro** — é a única
//! dependência de "frente" que existe aqui, ela vale **um span** (não o traço inteiro), e é o que
//! mantém o erro cobrado contra a curva que de fato será desenhada. Medido
//! (`measure_how_far_behind_the_pen_a_point_freezes`): a re-decisão mais funda fica a **12 amostras
//! atrás do cursor**.
//!
//! ⚠️ **Não há mais TETO de pontos, e a remoção é consequência da lei, não economia.** Um teto
//! global é uma decisão sobre o traço INTEIRO: sob ele, o mesmo começo recebe orçamentos diferentes
//! conforme o traço cresce — que é precisamente o defeito acima. Quem limita a contagem agora é o
//! piso anti-redundância, que é **local** (uma pergunta por span). Medido pelo pipeline do produto
//! num traço de 9000 amostras: **919 pontos em 1,92 ms** — contra o teto anterior de 512.
//!
//! # E é a lei que tornou o CACHE possível ([`FitCache`], 2026-07-31)
//!
//! A lei parou de *re-decidir*; ela não parava de *re-computar*. O ajuste percorria o traço inteiro
//! a cada frame do preview — medido (`measure_what_a_live_preview_frame_is_made_of`), **0,33 ms a
//! 1200 amostras e 2,42 ms a 9000**, dos quais o ajuste era **95 %**. Com a decisão global de
//! antes, um prefixo em cache **mentiria**; prefixo-estável, ele é apenas verdade guardada.
//!
//! Medido pela sonda das duas rotas (`measure_the_preview_frame_with_and_without_the_cache`), o
//! **quadro inteiro** do preview (alisar + ajustar + reamostrar):
//!
//! | amostras | ajuste sem | ajuste com | quadro sem | quadro com |
//! |---|---|---|---|---|
//! | 1200 | 0,306 | 0,006 | 0,311 | **0,012** |
//! | 3000 | 0,659 | 0,016 | 0,673 | **0,029** |
//! | 9000 | 1,973 | 0,076 | 2,014 | **0,117** |
//!
//! ⚠️ **E o que sobra NÃO é `O(1)`, é `O(n)` de VERIFICAÇÃO** — o cache compara o prefixo e copia a
//! entrada a cada chamada, e só a *decisão* virou `O(cauda)`. É o preço de medir em vez de
//! prometer, e a 9000 amostras ele vale 0,076 ms contra 1,973.

use super::{Vec2, resample_smooth};

/// **O ajuste sem memória** — ⚠️ **REFERÊNCIA CONGELADA, sem chamador de produção desde
/// 2026-07-31**, exatamente como o [`super::simplify_rdp`] um degrau abaixo.
///
/// Quem shipa é o [`FitCache::simplify`], e é contra ESTA função que ele é provado (gate
/// `the_cached_fit_is_the_fit`, índice a índice, quadro a quadro). Ela fica sob `cfg(test)` pelo
/// motivo de sempre: um `pub(crate)` sem chamador não é código morto silencioso, é uma **segunda
/// resposta** esperando alguém chamá-la.
///
/// A LEI que ela implementa está no doc do MÓDULO — ela descreve o produto, e o produto é o
/// [`fit_resumido`].
#[cfg(test)]
#[must_use]
pub(crate) fn simplify_to_curve(points: &[Vec2], tol: f32, step: f32) -> Vec<usize> {
    fit(points, tol, step, SAFETY)
}

/// **A MARGEM DE ACEITAÇÃO** — o span é fechado quando o erro ESTIMADO passa de `tol × SAFETY`.
///
/// ⚠️ **Ela existe porque a estimativa é aproximada por construção** (a tangente do fim do span sai
/// do [`espelho`], um palpite sobre onde o próximo nó vai cair), e o erro que importa é o da curva
/// DESENHADA. Sem margem o desenhado saía a **1,27× a tolerância**; a varredura mediu o gancho da
/// fixture de precisão e o arco liso ao lado dela:
///
/// | margem | gancho (% da espessura) | pts do gancho | pts do arco liso |
/// |---|---|---|---|
/// | 1,00 | 2,53 | 11 | 16 |
/// | 0,80 | 1,94 | 11 | 17 |
/// | **0,65** | **1,40** | **13** | **19** |
/// | 0,50 | 0,92 | 22 | **49** |
///
/// **0,65 bate o divide-e-conquista na precisão** (1,40 % contra 1,85 %) **e ainda é econômico**;
/// **0,50 é o joelho e ele é do lado errado** — o arco LISO salta de 19 para 49 pontos, que é
/// literalmente a queixa de 2026-07-18 (*"muitos pontos muito próximos e até sobrepostos"*)
/// voltando pela porta dos fundos. Abaixo de 0,65 a margem deixa de comprar precisão e passa a
/// comprar pontos.
const SAFETY: f32 = 0.65;

/// O ajuste com a margem como PARÂMETRO — é assim que a sonda a varre. A produção passa a
/// [`SAFETY`] pelo [`FitCache`]; ninguém mais tem escolha, e por isso ela também é `cfg(test)`.
#[cfg(test)]
fn fit(points: &[Vec2], tol: f32, step: f32, safety: f32) -> Vec<usize> {
    fit_resumido(points, tol, step, safety, Vec::new())
}

/// **O MESMO ajuste, retomado de uma semente** — e ele é o ÚNICO laço: o [`fit`] o chama com a
/// semente vazia e o [`FitCache`] com o prefixo que já decidiu.
///
/// ⚠️ **Não existe versão "incremental" para divergir da completa.** O que o cache traz é a
/// *semente*, nunca uma segunda aritmética — a lição do `walk_rows` do Wet Paint (um corpo, dois
/// caminhantes) e do `power_stroke` do vetor (um motor, preview e Apply).
///
/// `keep` tem de ser um PREFIXO válido da resposta (começando em `0`); vazio significa começar do
/// zero. Quem garante isso é o [`FitCache::congelados`], e ele o faz medindo, não supondo.
fn fit_resumido(
    points: &[Vec2],
    tol: f32,
    step: f32,
    safety: f32,
    mut keep: Vec<usize>,
) -> Vec<usize> {
    let n = points.len();
    if n < 3 || tol <= 0.0 {
        return (0..n).collect();
    }
    // ⚠️ **Um corte que o RENDER não consegue representar não é precisão, é lixo.** O
    // `resample_smooth` densifica a curva a cada `step`, então dois pontos guardados a menos de
    // meio `step` carregam informação que o traço desenhado não tem como mostrar — e é exatamente a
    // queixa de 2026-07-18 (*"pontos muito próximos e até sobrepostos"*).
    //
    // ⚠️ **A régua NÃO é minha: é a que o gate anti-redundância já cobra** — `0,05 × espessura`
    // (`the_resampled_stroke_tracks_the_drawing_without_redundant_points`), que nesta unidade é
    // `step × 0,125`, a MESMA razão `STROKE_SIMPLIFY_FRACTION / RESAMPLE_STEP_FRACTION` que o
    // `resample_smooth` documenta. Uma primeira tentativa usou `step × 0,5` e era grande demais:
    // dá `0,4 × raio` contra a cerca de vizinhos do render, que é `0,1875 × raio`.
    let minimo = step * 0.125;
    let longe = |i: usize, j: usize| -> bool {
        let d = Vec2::new(points[j].x - points[i].x, points[j].y - points[i].y);
        (d.x * d.x + d.y * d.y).sqrt() >= minimo
    };
    if keep.is_empty() {
        keep.push(0);
    }
    let mut a = *keep.last().expect("acabou de ser semeado");
    while a < n - 1 {
        // O nó ANTERIOR já guardado dá a tangente de entrada do span. Ele é passado, nunca futuro.
        let antes = (keep.len() >= 2).then(|| keep[keep.len() - 2]);
        let mut b = a + 1;
        while b < n - 1 {
            let candidato = b + 1;
            // Abaixo do piso o span é estendido SEM perguntar o erro: guardar ali seria guardar o
            // que o render não desenha.
            if longe(a, b) {
                let depois = espelho(a, candidato, n);
                if span_error(points, (antes, a, candidato, depois), step, tol) > tol * safety {
                    break;
                }
            }
            b = candidato;
        }
        keep.push(b);
        a = b;
    }
    keep
}

/// O ajuste com a margem forçada — só a sonda que a escolheu chama por aqui.
#[cfg(test)]
#[must_use]
pub(crate) fn fit_tuned(points: &[Vec2], tol: f32, step: f32, safety: f32) -> Vec<usize> {
    fit(points, tol, step, safety)
}

/// **Onde estará o PRÓXIMO nó** — a amostra a uma distância de span adiante de `b`, espelhando o
/// span que acabou de ser medido.
///
/// ⚠️ **Isto é a única dependência de "frente" do ajuste, e ela precisa ser esta.** A tangente que a
/// curva final terá em `b` sai dos vizinhos `(a, próximo nó)`, não de `(a, amostra colada em b)` —
/// e a diferença NÃO é sutil: com a amostra adjacente o gancho da fixture de precisão saiu a
/// **14,62 %** da espessura da mão, contra **1,85 %** com o espelho. Numa região de curvatura
/// parecida os spans saem parecidos, então espelhar o span é o palpite certo sobre onde o vizinho
/// vai cair.
///
/// ⚠️ **E o preço é nomeado:** a decisão sobre `b` passa a depender de amostras até `b + (b − a)`,
/// ou seja a franja provisória do traço deixa de ser "dois nós" e passa a ser "um span" — que é o
/// pedaço sob a caneta de qualquer forma. Passado isso, congela.
fn espelho(a: usize, b: usize, n: usize) -> Option<usize> {
    let alvo = (b + (b - a)).min(n - 1);
    (alvo > b).then_some(alvo)
}

/// O pior desvio das amostras de `[a, b]` contra a curva que o produto desenharia ali.
///
/// ⚠️ **Reconstrói só a VIZINHANÇA** (`antes, a, b, depois`), e é isso que mantém o custo em
/// `O(n · m_local)` em vez de `O(k · n · m)`: a Catmull-Rom é local — a tangente em `a` só olha o
/// vizinho anterior —, então o span não precisa do traço inteiro para saber a própria curva.
fn span_error(
    points: &[Vec2],
    span: (Option<usize>, usize, usize, Option<usize>),
    step: f32,
    tol: f32,
) -> f32 {
    let (antes, a, b, depois) = span;
    let mut viz: Vec<Vec2> = Vec::with_capacity(4);
    viz.extend(antes.map(|i| points[i]));
    viz.push(points[a]);
    viz.push(points[b]);
    viz.extend(depois.map(|i| points[i]));
    let prs = vec![1.0_f32; viz.len()];
    let (curva, _) = resample_smooth(&viz, &prs, step, tol);
    let mut pior = 0.0_f32;
    for q in points.iter().take(b).skip(a + 1) {
        let mut d = f32::MAX;
        for w in curva.windows(2) {
            d = d.min(dist_to_seg(*q, w[0], w[1]));
        }
        pior = pior.max(d);
    }
    pior
}

/// **O AJUSTE INCREMENTAL** — o mesmo resultado, sem re-decidir o que já foi decidido.
///
/// ## Por que ele só existe agora
///
/// A lei anterior (divide-e-conquista por pior erro GLOBAL) **não podia ser cacheada**: uma amostra
/// nova na ponta re-decidia nós no começo, então um prefixo em cache MENTIRIA. A lei de hoje é
/// prefixo-estável (ver [`simplify_to_curve`]), e é isso — e só isso — que torna este cache
/// possível. O ganho é consequência da correção, não de uma otimização paralela.
///
/// ## O que ele NÃO faz: prometer
///
/// ⚠️ **Ele guarda a entrada exata do último ajuste e MEDE o prefixo comum.** A alternativa óbvia
/// — guardar só `n` e confiar que as amostras são append-only — seria uma promessa do chamador, e
/// uma promessa errada aqui **não falha**: ela devolve um traço plausível, decidido sobre dados que
/// já não existem. Medir custa uma varredura de `f32` (µs contra os ~2,4 ms do ajuste) e vale por
/// **qualquer** chamador, inclusive os que ainda não existem.
///
/// ⚠️ E medir apaga uma dependência que eu teria de acertar por fora: o insumo do ajuste é o array
/// **SUAVIZADO**, e o `active_smooth` reescreve a cauda a cada amostra nova (raio = nº de
/// iterações). Um cache que soubesse esse raio ficaria errado no dia em que o kernel mudasse; este
/// **descobre** a fronteira, porque ela É o prefixo que de fato coincide.
///
/// ## Quais nós sobrevivem
///
/// Um nó é FIRME quando a decisão que o produziu leu **apenas** índices dentro do prefixo
/// verificado. A decisão do nó `k` (a partir do anterior `a`) probe candidatos até `k+1` e olha um
/// span à frente pelo [`espelho`] ⇒ o índice mais fundo que ela toca é `2·(k+1) − a`.
///
/// ⚠️ **A segunda condição — o fim do array — é IMPLICADA pela primeira, e isso foi MEDIDO.** Um nó
/// em `n−1` não veio de um break por erro, veio de *acabar a entrada*: ele é provisório por
/// natureza. Mas os nós crescem estritamente (`a < k`), logo `2·(k+1) − a ≥ k+3`, e exigir
/// `2·(k+1) − a ≤ comum−1 ≤ n−1` já força `k ≤ n−4`. A mutação que apaga essa condição
/// **sobrevive à suíte inteira** — ela é defesa em camada, não uma cerca observável. Fica porque
/// volta a carregar peso no dia em que a regra de alcance for afrouxada, e fica NOMEADA para
/// ninguém a "descobrir" como buraco.
#[derive(Default)]
pub(crate) struct FitCache {
    /// A entrada EXATA do último ajuste (o array SUAVIZADO, não as amostras cruas).
    entrada: Vec<Vec2>,
    tol: f32,
    passo: f32,
    keep: Vec<usize>,
    /// Quantos nós a última chamada REAPROVEITOU.
    ///
    /// ⚠️ Existe para o gate de identidade poder declarar a própria premissa: um cache que nunca
    /// reusa é trivialmente idêntico ao ajuste completo, e sem este número o gate ficaria **verde
    /// por vácuo** no dia em que alguém enfraquecesse o [`Self::congelados`].
    #[cfg(test)]
    semente: usize,
}

impl FitCache {
    /// O [`simplify_to_curve`] com memória. **Devolve exatamente o que ele devolveria** — o gate
    /// `the_cached_fit_is_the_fit` afirma isso índice a índice, quadro a quadro.
    pub(crate) fn simplify(&mut self, points: &[Vec2], tol: f32, passo: f32) -> &[usize] {
        let n = points.len();
        if n < 3 || tol <= 0.0 {
            // O caminho degenerado do [`fit`], reproduzido aqui: `entrada` fica VAZIA, então o
            // quadro seguinte não reusa nada — e é o certo, porque não houve walk nenhum.
            self.entrada.clear();
            self.keep = (0..n).collect();
            self.tol = tol;
            self.passo = passo;
            return &self.keep;
        }
        // ⚠️ Bit a bit, nos DOIS números: `tol`/`passo` mudam quando o artista mexe na espessura, e
        // "quase igual" não é a pergunta — a resposta do walk é função pura destes três.
        let mesmos = self.tol.to_bits() == tol.to_bits() && self.passo.to_bits() == passo.to_bits();
        let comum = if mesmos {
            prefixo_comum(&self.entrada, points)
        } else {
            0
        };
        let firmes = self.congelados(comum);
        #[cfg(test)]
        {
            self.semente = firmes;
        }
        self.keep.truncate(firmes);
        let semente = core::mem::take(&mut self.keep);
        self.keep = fit_resumido(points, tol, passo, SAFETY, semente);
        self.entrada.clear();
        self.entrada.extend_from_slice(points);
        self.tol = tol;
        self.passo = passo;
        &self.keep
    }

    /// Quantos nós a última chamada reaproveitou (ver [`Self::semente`]).
    #[cfg(test)]
    pub(crate) fn reaproveitados(&self) -> usize {
        self.semente
    }

    /// Solta a memória — o traço acabou, e um `Vec` do tamanho do gesto anterior não tem por que
    /// atravessar o próximo.
    ///
    /// ⚠️ **Não é correção, é higiene.** O cache é um memo puro: se por acaso o prefixo de um traço
    /// NOVO coincidisse bit a bit com o do anterior, reusar os nós ainda seria a resposta certa.
    pub(crate) fn clear(&mut self) {
        self.entrada.clear();
        self.entrada.shrink_to_fit();
        self.keep.clear();
    }

    /// Quantos nós do último ajuste sobrevivem, dado que `points[..comum]` está VERIFICADO igual.
    ///
    /// Devolve um COMPRIMENTO de prefixo, e a varredura para no primeiro que falha: o walk é
    /// sequencial, então pular um nó provisório e reusar o seguinte não significaria nada.
    fn congelados(&self, comum: usize) -> usize {
        // O nó 0 é firme sempre: ele é o começo, não é decidido por nada.
        let base = usize::from(!self.keep.is_empty());
        if comum == 0 {
            return base;
        }
        let n_velho = self.entrada.len();
        let mut firmes = base;
        for i in 1..self.keep.len() {
            let (k, a) = (self.keep[i], self.keep[i - 1]);
            // (a) Fim do array: provisório por natureza (não houve break por erro).
            if k + 1 >= n_velho {
                break;
            }
            // (b) Tudo o que a decisão leu — inclusive o [`espelho`] — está no prefixo verificado.
            if 2 * (k + 1) - a > comum - 1 {
                break;
            }
            firmes = i + 1;
        }
        firmes
    }
}

/// Quantos elementos iniciais os dois arrays têm **idênticos ao BIT**.
///
/// ⚠️ `to_bits`, não `==`: a pergunta é *"esta é literalmente a mesma entrada?"*, e `NaN != NaN`
/// responderia "não" para dois arrays iguais enquanto `0.0 == -0.0` responderia "sim" para dois
/// diferentes. Nenhum dos dois é o que o cache precisa saber.
fn prefixo_comum(a: &[Vec2], b: &[Vec2]) -> usize {
    let n = a.len().min(b.len());
    let mut i = 0;
    while i < n && a[i].x.to_bits() == b[i].x.to_bits() && a[i].y.to_bits() == b[i].y.to_bits() {
        i += 1;
    }
    i
}

/// Distância de `q` ao SEGMENTO `a→b` (clampada), que é o que a curva desenhada de fato ocupa — a
/// [`perp_dist`] mede até a reta INFINITA, e usá-la aqui cobraria distância a prolongamentos que
/// ninguém desenha.
fn dist_to_seg(q: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = Vec2::new(b.x - a.x, b.y - a.y);
    let len2 = ab.x * ab.x + ab.y * ab.y;
    let t = if len2 < 1e-12 {
        0.0
    } else {
        (((q.x - a.x) * ab.x + (q.y - a.y) * ab.y) / len2).clamp(0.0, 1.0)
    };
    let (dx, dy) = (q.x - (a.x + ab.x * t), q.y - (a.y + ab.y * t));
    (dx * dx + dy * dy).sqrt()
}
