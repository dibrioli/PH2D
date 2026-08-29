//! **A ESCALA — quão grande é um quad, e ONDE** (ADR-0160 §3-ii, asserção A6).
//!
//! É aqui que mora a metade *adaptativa* do pedido. Um remesh de escala única
//! devolve a mesma grade num nariz e numa barriga; um adaptativo põe **quads
//! menores onde a curvatura é alta** e maiores onde a forma é chapada — que é o
//! que o Houdini chama de *"more smaller quads in regions with many local
//! features"*.
//!
//! ⚠️ **A escala é um CAMPO por-vértice, e não um número, mesmo no modo
//! uniforme.** Uma porta única (`ScaleField`) que às vezes é constante custa um
//! `Vec` de `f32` e apaga o caso especial; duas portas (um `f32` e um campo)
//! seriam a pergunta *"qual das duas manda?"* respondida em cada consumidor.

use ph2d_mesh::Mesh;

/// **Quantas vezes o quad menor cabe no maior** — o teto da adaptação.
///
/// ⚠️ **É um limite de REPRESENTAÇÃO, não de conforto** (`CLAUDE.md` §0.0): a
/// extração liga células vizinhas da retícula, e duas células cujas escalas
/// diferem por mais do que isto deixam de ter aresta comum — a grade rasga em
/// vez de transitar. O número é a razão que a literatura de campo cruzado usa
/// para o *sizing field* graduado, e ele **tem gate**
/// (`the_adaptive_range_is_bounded`).
pub const MAX_ADAPTIVE_RATIO: f32 = 4.0;

/// **A escala de cada vértice** — o lado do quad que se quer ali, em unidades de
/// objeto.
#[derive(Clone, Debug, PartialEq)]
pub struct ScaleField {
    per_vertex: Vec<f32>,
}

impl ScaleField {
    /// O lado do quad pedido no vértice `v`.
    #[must_use]
    pub fn at(&self, v: usize) -> f32 {
        self.per_vertex[v]
    }

    /// Quantos vértices o campo cobre.
    #[must_use]
    pub fn len(&self) -> usize {
        self.per_vertex.len()
    }

    /// Um campo sem vértices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.per_vertex.is_empty()
    }

    /// O par (menor, maior) — a régua da asserção A6.
    #[must_use]
    pub fn range(&self) -> (f32, f32) {
        self.per_vertex
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(*s), hi.max(*s)))
    }

    /// **UNIFORME** — o mesmo lado em toda parte.
    ///
    /// ⚠️ **A razão do [`Self::range`] tem de sair `1,0` EXATO aqui**, e é o
    /// controle da A6: um modo "uniforme" que variasse um por cento seria um
    /// adaptativo fraco a fingir-se de uniforme, e nenhum gate de aparência veria.
    #[must_use]
    pub fn uniform(mesh: &Mesh, edge: f32) -> Self {
        Self {
            per_vertex: vec![edge.max(MIN_EDGE); mesh.vert_count()],
        }
    }

    /// **ADAPTATIVA** — o lado encolhe onde a curvatura aperta.
    ///
    /// A lei é a da literatura de *sizing field*: o lado do quad é proporcional
    /// ao **raio de curvatura** (`1/|κ|`), porque é o raio que diz quantos quads
    /// uma feição precisa para não sair facetada. `strength = 0` devolve o campo
    /// uniforme **ao bit**; `1` usa a faixa inteira.
    ///
    /// ⚠️ **A curvatura entra NORMALIZADA pelo percentil, não pelo máximo.** Um
    /// único vértice patológico (um pico de uma importação, um polo de uma
    /// esfera UV) tem curvatura ordens de grandeza acima do resto, e dividir pelo
    /// máximo esmagaria o modelo inteiro contra o piso — a adaptação inteira
    /// ficaria a servir um vértice. A mediana é a estatística que não se move com
    /// ele.
    ///
    /// ⚠️ **E o resultado é CLAMPADO pela [`MAX_ADAPTIVE_RATIO`]**: não é
    /// conforto, é o que impede a grade de rasgar entre células de escalas
    /// incompatíveis.
    #[must_use]
    pub fn adaptive(mesh: &Mesh, edge: f32, strength: f32) -> Self {
        Self::adaptive_with(mesh, edge, strength, FLOOR_IN_INPUT_EDGES)
    }

    /// **ADAPTATIVA, com o piso do CHAMADOR** — ver
    /// [`resolvable_edge_range_with`].
    ///
    /// ⛔⛔ **Sem isto o campo COLAPSA para uma constante, e o knob morre em
    /// silêncio.** O `lo`/`hi` abaixo são recortados pelo piso, e o piso era sempre
    /// o do motor LOCAL (`3,0` arestas de entrada). Quando o chamador pede um alvo
    /// mais fino que esse piso — que é exactamente o que a cadeia global faz desde
    /// que ganhou o seu próprio piso —, `lo` e `hi` batem no mesmo número e **todo
    /// vértice recebe o mesmo tamanho**. Medido em 2026-08-21 na esfera com cristas:
    /// `min = mediana = max = 0,2301` com alvo `0,0910`, em `adapt = 0,5` **e** em
    /// `adapt = 1,0` — dois valores do knob, saída idêntica.
    ///
    /// ⚠️ **E o sintoma não era *"não adapta"*, era *"perdeu dois terços das
    /// faces"***: o campo constante valia `2,5×` o alvo, então a peça saía com 451
    /// quads em vez de 1 336. *Um knob que colapsa não fica neutro — ele passa a
    /// mandar.*
    #[must_use]
    pub fn adaptive_with(mesh: &Mesh, edge: f32, strength: f32, floor_in_input_edges: f32) -> Self {
        let edge = edge.max(MIN_EDGE);
        let s = strength.clamp(0.0, 1.0);
        if s == 0.0 {
            // ⚠️ **Saída ANTECIPADA e não `mix(uniform, adaptive, 0)`:** o
            // caminho aritmético devolveria `edge * (1 - 0) + x * 0`, que em
            // `f32` é `edge` só quando `x` é finito. Uma curvatura `NaN` numa
            // malha importada envenenaria o modo uniforme por um caminho que
            // ninguém suspeitaria.
            return Self::uniform(mesh, edge);
        }

        let curv = mesh.curvatures();
        // A mediana do |κ| — a régua que um pico não move.
        let mut mags: Vec<f32> = curv.iter().map(|k| k.abs()).collect();
        mags.sort_by(f32::total_cmp);
        let median = mags.get(mags.len() / 2).copied().unwrap_or(0.0);

        // ⚠️ **O PISO DO RECURSO VALE AQUI TAMBÉM, e esta linha é a foto do
        // Enio de 2026-08-19 a repetir-se.** A `edge_for_detail` foi corrigida
        // para nunca pedir menos que `FLOOR_IN_INPUT_EDGES` arestas de entrada, e
        // o doc dela diz *"todo valor do curso é legal, por construção"* — o que
        // era verdade **só do caminho uniforme**. Esta função construía o seu
        // limite inferior a partir do `edge` e mais nada:
        //
        // ```text
        // lo = edge / √4 = edge / 2
        // ```
        //
        // Com `detail = 1,00` o `edge` **é** o piso (3,00× a aresta de entrada),
        // então `lo` aterrava em **1,50×** — que é, textualmente, a linha da
        // tabela do [`FLOOR_IN_INPUT_EDGES`] anotada como *"a foto do Enio"*
        // (ciclo de 352 lados, 58 % do volume perdido). *A cura e a reincidência
        // moravam no mesmo arquivo, a 130 linhas uma da outra.*
        //
        // ⚠️ **E nenhum gate percorria este eixo:** os três testes de produto
        // que chamam `quad_remesh` pinam `adaptive = 0.0`, que é o único valor em
        // que esta função sai antecipadamente.
        //
        // Medido depois desta linha (grelha `detail` × `adapt`, malha da cena
        // `=35`): as arestas de borda vão de **203** (`1,00`/`0,95`) e **191**
        // com 15 componentes (`1,00`/`1,00`) para **zero** em toda a grelha.
        //
        // ⚠️ **A consequência honesta:** em `detail = 1,00` a adaptação para
        // BAIXO fica sem curso — não há folga sob o piso. Isso é o recurso a
        // dizer o que é, não uma perda: a adaptação para CIMA (quads maiores onde
        // a forma é chapada) continua inteira, e é ela que o `hi` carrega.
        let floor = resolvable_edge_range_with(mesh, floor_in_input_edges).0;
        let lo = (edge / MAX_ADAPTIVE_RATIO.sqrt()).max(floor);
        let hi = (edge * MAX_ADAPTIVE_RATIO.sqrt()).max(lo);
        let per_vertex = curv
            .iter()
            .map(|k| {
                // `r = 1/|κ|` normalizado pela mediana: 1 no vértice mediano,
                // menor onde aperta, maior onde a forma é chapada.
                let rel = if median > 1.0e-9 {
                    (median / k.abs().max(1.0e-9))
                        .clamp(1.0 / MAX_ADAPTIVE_RATIO.sqrt(), MAX_ADAPTIVE_RATIO.sqrt())
                } else {
                    1.0
                };
                // `strength` interpola entre o uniforme e a lei cheia.
                let f = s.mul_add(rel - 1.0, 1.0);
                (edge * f).clamp(lo, hi)
            })
            .collect();
        Self { per_vertex }
    }
}

/// **QUANTAS ARESTAS DA ENTRADA UM QUAD PRECISA DE MEDIR** — o piso, MEDIDO.
///
/// ⚠️ **Uma retopologia não pode resolver uma grade mais fina que a malha que ela
/// lê**, e não é uma opinião: cada vértice da saída é a média de um punhado de
/// vértices da entrada. Peça um quad menor que isso e a célula fica com um
/// vértice — o campo não tem o que quantizar, o grafo sai com buracos, e o passeio
/// de faces os contorna em ciclos gigantes.
///
/// **Medido** (sonda `measure_where_quality_collapses`), com o lado do quad em
/// múltiplos da aresta média da entrada:
///
/// | razão | esfera 48×64 | uv 96×144 | uv 96×144 **amassada** |
/// |---|---|---|---|
/// | 1,00× | **malha vazia** | **malha vazia** | **malha vazia** |
/// | 1,25× | **malha vazia** | **malha vazia** | **malha vazia** |
/// | 1,50× | ciclo de **62** | ciclo de **147** | ciclo de **352**, volume 1,59 de 3,78 |
/// | 1,75× | ciclo de 10 | ciclo de 20 | ciclo de **50** |
/// | 2,00× | ciclo de 7 | ciclo de 9 | ciclo de **39** |
/// | 2,50× | ciclo de 6 | ciclo de 6 | ciclo de 20 |
/// | **3,00×** | **ciclo de 6** | **ciclo de 7** | **ciclo de 8** |
/// | 4,00× | ciclo de 5 | ciclo de 5 | ciclo de 6 |
///
/// ⚠️ **A linha de 1,50× na amassada é a foto do Enio** (2026-08-19): um ciclo de
/// 352 lados vira 350 triângulos em leque, e a peça perde **58 % do volume**. O
/// painel oferecia `0,02` como mínimo, que naquela malha é **0,66×** — fundo do
/// poço. *O piso não estava errado por pouco: ele não era do recurso.*
///
/// **3,00×** é o primeiro degrau em que as três fixturas concordam.
pub const FLOOR_IN_INPUT_EDGES: f32 = 3.0;

/// **O PISO DA CADEIA GLOBAL** — MEDIDO, e ele é **quatro vezes mais fino** que o
/// do motor local.
///
/// ⛔ **O `3,0` acima é do motor LOCAL e estava a definir o teto do global**, que é
/// outro algoritmo: ele não extrai de retícula nenhuma. Medido em 2026-08-21 na
/// esfera amassada, **sem tocar no mapa de patches** (28 patches, 67 arcos, `work`
/// de 5 136 triângulos), só descendo o alvo:
///
/// | alvo | quads | dobras | mediana | **aresta máx / diagonal** | F4 |
/// |---|---|---|---|---|---|
/// | `3,00×` (o piso local) | 1 336 | 0,00 % | 1,03× | 7,2 % | 24 ms |
/// | `1,50×` | 4 885 | 0,02 % | 1,05× | 6,4 % | 148 ms |
/// | ⭐ **`0,75×` (este)** | **20 039** | **0,03 %** | **1,03×** | **5,1 %** | **2,7 s** |
/// | `0,54×` | 38 315 | 0,08 % | 1,03× | 4,5 % | 1,5 s |
/// | ⛔ `0,375×` | 78 883 | 0,09 % | 1,04× | 4,8 % | ⛔ **50 s, sem prova** |
///
/// ⭐ **O detalhe NÃO se perde ao pedir quads mais finos que o `work`**: todo ponto
/// de interior é reprojectado sobre a malha **ORIGINAL** do artista, não sobre a
/// remalhada — então quads mais finos apanham detalhe que o F1 já tinha deitado
/// fora.
///
/// ⚠️ **De que recurso é este limite:** do **relógio da quantização**, e de mais
/// nada. As dobras ficam em 0,03 %, a mediana em 1,03× e a aresta máxima em
/// **fração da peça** até melhora. O que explode é a busca do F4 — 2,7 s aqui,
/// 50 s um degrau abaixo, e sem prova de ótimo. *O porte do solver de refinamento
/// do libSatsuma é o que move este número, e é a próxima peça do plano.*
pub const GLOBAL_FLOOR_IN_INPUT_EDGES: f32 = 0.75;

/// **O MENOR NÚMERO DE QUADS que ainda descreve uma forma** — o teto, MEDIDO.
///
/// Do outro lado da faixa a malha deixa de ter faces suficientes para a forma
/// sobreviver. Medido na esfera amassada (volume da entrada **3,78**): 96 faces
/// guardam **3,45** (91 %) · 40 faces guardam 2,57 (68 %) · 23 faces guardam
/// **1,90** (50 %). O joelho está em ~**100** faces, e é daí que sai o teto
/// `s = √(área / 100)`.
pub const MIN_QUADS: f32 = 100.0;

/// **A ARESTA MÉDIA da malha** — o que ela é capaz de resolver.
#[must_use]
pub fn mean_edge(mesh: &Mesh) -> f32 {
    let p = mesh.positions();
    let (mut sum, mut count) = (0.0f64, 0usize);
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            sum += f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
            count += 1;
        }
    }
    if count == 0 {
        MIN_EDGE
    } else {
        (sum / count as f64) as f32
    }
}

/// **A ÁREA da superfície** — a régua do teto.
#[must_use]
pub fn surface_area(mesh: &Mesh) -> f32 {
    let p = mesh.positions();
    let mut sum = 0.0f64;
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
            let (u, w) = (
                [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            );
            let n = [
                u[1].mul_add(w[2], -(u[2] * w[1])),
                u[2].mul_add(w[0], -(u[0] * w[2])),
                u[0].mul_add(w[1], -(u[1] * w[0])),
            ];
            sum += f64::from(n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt()) * 0.5;
        }
    }
    sum as f32
}

/// **A FAIXA LEGAL do lado do quad para ESTA malha** — `(mais fino, mais grosso)`.
///
/// ⚠️ **É a faixa que o produto oferece, e ela é da MALHA e não do slider.** Um
/// mínimo fixo em unidades de objeto (`0,02`, que é o que o painel tinha) é
/// destrutivo numa malha grossa e conservador numa fina — o mesmo número quer
/// dizer coisas opostas em dois modelos. Aqui os dois extremos saem da medição:
/// o piso do [`FLOOR_IN_INPUT_EDGES`], o teto do [`MIN_QUADS`].
///
/// ⚠️ **O teto é forçado a não descer abaixo do piso**: numa malha já grossa os
/// dois se cruzam, e uma faixa invertida devolveria um `edge` menor que o piso
/// por aritmética — que é o defeito que esta função existe para impedir.
#[must_use]
pub fn resolvable_edge_range(mesh: &Mesh) -> (f32, f32) {
    resolvable_edge_range_with(mesh, FLOOR_IN_INPUT_EDGES)
}

/// **A FAIXA, com o piso do CHAMADOR** — ver [`FLOOR_IN_INPUT_EDGES`].
///
/// ⭐⭐ **Ela existe porque o piso de `3,0` é do motor LOCAL, e ele estava a
/// definir o teto do outro.** Aqui a extração liga células de uma retícula: um
/// quad mais fino que o triângulo de entrada devolve um ciclo de 352 lados com
/// 58 % do volume perdido (a foto de 2026-08-19), e daí sai o `3,0`. ⚠️ **A cadeia
/// global não extrai de retícula nenhuma** — ela reamostra arcos por comprimento e
/// amostra o interior dentro de um triângulo achatado —, então esse número não é
/// dela. *Nunca deixe o caminho mais limitado definir o teto do outro* (CLAUDE.md
/// §0.0).
#[must_use]
pub fn resolvable_edge_range_with(mesh: &Mesh, floor_in_input_edges: f32) -> (f32, f32) {
    let floor = (floor_in_input_edges * mean_edge(mesh)).max(MIN_EDGE);
    let ceiling = (surface_area(mesh) / MIN_QUADS).sqrt().max(floor);
    (floor, ceiling)
}

/// **O LADO DO QUAD que um `detail` de `0..1` pede a ESTA malha.**
///
/// `0` é o mais grosso que ainda descreve a forma, `1` o mais fino que a entrada
/// consegue resolver — e a interpolação é **geométrica**, não linear: um knob de
/// TAMANHO tem de andar em razão constante, senão metade do curso mora numa
/// oitava só (a lição já registrada em
/// `feedback_a_knob_consumed_as_a_per_step_rate_is_a_target_not_a_rate`).
///
/// ⚠️ **Todo valor do curso é legal, por construção** — que é a diferença entre
/// este knob e o que o painel tinha. Um slider cujo terço esquerdo destrói o
/// objeto não é um slider com um limite errado: é um slider que responde a uma
/// pergunta que a malha não tem como responder.
#[must_use]
pub fn edge_for_detail(mesh: &Mesh, detail: f32) -> f32 {
    edge_for_detail_with(mesh, detail, FLOOR_IN_INPUT_EDGES)
}

/// **O LADO DO QUAD, com o piso do CHAMADOR** — ver [`resolvable_edge_range_with`].
#[must_use]
pub fn edge_for_detail_with(mesh: &Mesh, detail: f32, floor_in_input_edges: f32) -> f32 {
    let (floor, ceiling) = resolvable_edge_range_with(mesh, floor_in_input_edges);
    // ⚠️ **`clamp` NÃO fecha o `NaN`** — ele o propaga, e um `NaN` aqui vira um
    // `scale` `NaN` que envenena o campo inteiro sem erro nenhum. O gate
    // `every_point_of_the_detail_slider_is_legal` o pegou. O `NaN` cai no
    // extremo GROSSO de propósito: é o único lado do curso que não tem como
    // destruir a peça.
    let d = if detail.is_nan() {
        0.0
    } else {
        detail.clamp(0.0, 1.0)
    };
    ceiling * (floor / ceiling).powf(d)
}

/// ⭐⭐⭐ **A CONTAGEM DE QUADS DO EXTREMO FINO** — o outro fim da faixa de
/// [`MIN_QUADS`], e a metade que faz o slider dizer **a mesma coisa em todo aperto**.
///
/// # ⛔⛔⛔ Por que a faixa passou a ser CONTADA em vez de derivada da malha
///
/// Até 2026-08-28 o alvo saía de [`edge_for_detail_with`], cujo piso é
/// `floor_in_input_edges · mean_edge(mesh)` — **uma medida da malha que está na cena**.
/// Isso é honesto na primeira passagem (*não se resolve mais fino do que a entrada
/// tesselou*) e é uma armadilha na segunda: depois de uma retopologia a malha da cena
/// **é** a saída, então o piso sobe com ela e o mesmo ponto do slider pede quads cada vez
/// maiores.
///
/// Medido 2026-08-28 na peça do artista (`sculpt_t002`, 19 786 quads), `Detail` parado em
/// `0,50`, três apertos seguidos do mesmo botão:
///
/// | aperto | alvo | quads | perda |
/// |---|---|---|---|
/// | entrada | — | **19 786** | — |
/// | 1.º | `0,0891` | `1 747` | `−91 %` |
/// | 2.º | `0,1614` | `520` | `−97 %` |
/// | 3.º | `0,2161` | ⛔ **`281`** | ⛔ **`−98,6 %`** |
///
/// ⭐ *Um slider parado que devolve três densidades é um slider que não tem significado* —
/// e foi isto que o artista fotografou e chamou de «pontas com baixa resolução».
///
/// ⚠️ **A ÁREA é o que não se move.** O teto de [`MIN_QUADS`] já era absoluto
/// (`√(área/100)`); o piso é que era relativo. Ancorar os **dois** na área da superfície
/// torna o botão **idempotente**: a mesma posição do slider pede a mesma contagem, venha o
/// aperto de uma escultura ou de uma grade que este botão acabou de escrever.
///
/// ⚠️ **De que recurso é este número: do RELÓGIO, e de mais nada.** Medido no botão (as
/// duas tentativas, peça do artista):
///
/// | contagem pedida | quads | `χ` · bordo | enviesamento p50 | relógio |
/// |---|---|---|---|---|
/// | `~1 800` | `1 747` | `2` · `0` | `3,7°` | **`8,6 s`** |
/// | `~13 600` | `13 595` | `2` · `0` | `3,9°` | `22,3 s` |
/// | ⭐ `~24 200` | `24 190` | `2` · `0` | `4,5°` | ⚠️ `35,1 s` |
///
/// ⭐ **A forma NÃO se degrada com a densidade** — o `χ` fecha e o enviesamento anda `0,8°`
/// em 14× a contagem. *O extremo fino não é um precipício de qualidade; é uma espera.*
///
/// ⛔⛔ **E acima daqui há um SEGUNDO recurso, medido por outra linha: a TOPOLOGIA.** A
/// `line/3DModeling` implementou a escada inteira de densidades de exportação e o degrau
/// `Max` saiu com `316` arestas de bordo e `6` não-manifold depois de `27 min 29 s` —
/// *o limite da cadeia não é só o tempo*. ⇒ este número fica no ponto mais fino **medido
/// limpo** (`24 190` quads, `χ = 2`, zero bordo), e não no ponto em que o relógio ainda
/// se aguenta. ⚠️ **Quem quiser subi-lo mede a topologia, não o relógio.**
pub const MAX_QUADS: f32 = 25_000.0;

/// **QUANTOS QUADS um `detail` de `0..1` pede** — absoluto, ancorado na ÁREA.
///
/// `0` é [`MIN_QUADS`] e `1` é [`MAX_QUADS`], com interpolação **geométrica** pela mesma
/// razão de [`edge_for_detail`]: um knob de TAMANHO tem de andar em razão constante.
#[must_use]
pub fn quads_for_detail(detail: f32) -> f32 {
    // ⚠️ **`clamp` NÃO fecha o `NaN`** — a mesma armadilha de [`edge_for_detail_with`], e o
    // `NaN` cai no extremo GROSSO de propósito: é o único lado que não destrói a peça.
    let d = if detail.is_nan() {
        0.0
    } else {
        detail.clamp(0.0, 1.0)
    };
    MIN_QUADS * (MAX_QUADS / MIN_QUADS).powf(d)
}

/// ⭐⭐⭐ **O LADO DO QUAD, IDEMPOTENTE** — ver [`MAX_QUADS`].
///
/// ⚠️ **Ele lê da malha uma coisa só: a ÁREA**, que é da *superfície* e não da tesselação.
/// É isso que faz dois apertos seguidos pedirem a mesma coisa.
///
/// ⛔ **Ele NÃO substitui [`edge_for_detail_with`]**, que continua a responder outra
/// pergunta — *"o mais fino que ESTA malha resolve"* — e é a que o motor local precisa de
/// fazer, porque a extração por retícula dele rasga quando o quad é mais fino que o
/// triângulo de entrada.
#[must_use]
pub fn edge_for_detail_by_count(mesh: &Mesh, detail: f32) -> f32 {
    let area = surface_area(mesh);
    let quads = quads_for_detail(detail);
    if area.is_finite() && area > 0.0 && quads >= 1.0 {
        (area / quads).sqrt().max(MIN_EDGE)
    } else {
        MIN_EDGE
    }
}

/// O piso do lado de um quad, em unidades de objeto.
///
/// ⚠️ **Guarda de RECURSO e não de gosto:** o inverso da escala multiplica cada
/// coordenada na retícula do campo de posição, e um zero ali é uma divisão por
/// zero que sai como `inf` no meio de um campo — envenenando a suavização
/// inteira na varredura seguinte, sem erro nenhum.
pub const MIN_EDGE: f32 = 1.0e-6;

#[cfg(test)]
#[path = "scale_tests.rs"]
mod tests;
