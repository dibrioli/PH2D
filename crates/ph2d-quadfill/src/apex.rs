//! ⭐⭐⭐ **A LEI DO ÁPICE — o que é uma PONTA, e a unidade em que se mede.**
//!
//! Irmã de [`crate::local`] (a forma de UM quad) e de [`crate::tips`] (o desvio e a grade
//! junto de cada ponta) por RESPONSABILIDADE: este módulo responde **quais** vértices da
//! escultura são pontas — o censo — e as outras duas medem **o que a saída fez** a cada uma.
//! Nasceu do tecto de LOC do workspace (`700`) em 2026-09-02, quando a lei ganhou o filtro de
//! forma e o `local.rs` passou a `819` linhas: a fronteira já existia no doc de [`apices`].
//!
//! ⛔⛔⛔ **Por que a lei tem a forma que tem** (handoff de 2026-09-01 §0 e a jornada de
//! 2026-09-02): três réguas consultavam uma lei com o piso a `0,55` do raio e o corte em
//! `12`, e as três pontas de que o dono se queixava estavam a `0,43`–`0,47` do raio —
//! **nenhuma régua desta linha alguma vez as mediu**. Baixar o piso sem um filtro de FORMA
//! chamaria «ponta» a cada bossa do corpo, e as bossas lêem grade `1,0`–`1,47` **mesmo na
//! malha que ele aprovou**. O cone ([`cone_of`]) é o que separa as duas populações, e a
//! tabela está em [`apices`].

use ph2d_mesh::Mesh;

use super::local::{dist, dot, sub};

/// ⚠️ **A BARRA DA FORMA — o que separa um ESPINHO de um BOTÃO ou de uma BOSSA.** `1,0` é
/// o meio-ângulo de `45°` **em todas as faixas até `9 × unit`** ([`cone_of`]), e ele sai da
/// tabela em [`apices`]: os espinhos afiados das duas peças do dono lêem `≤ 0,99` nas
/// densidades do produto, os botões e cúpulas — incluindo o `7328` que o QRemeshify deixa
/// grosso **com aprovação do dono** — lêem `≥ 1,01`, e as bossas do corpo `≥ 1,21`.
pub const CONE_MAX: f32 = 1.0;

/// ⭐⭐⭐ **A LEI DE «O QUE É UMA PONTA», numa PORTA** — o centroide por vértice e a lista
/// de ápices, ordenada do mais longo para o mais curto.
///
/// ⛔⛔ **Ela é uma função e não um bloco copiado porque TRÊS réguas a consultam** —
/// [`tip_survival`] (o suporte), [`super::tip_deviation`] (o desvio local e a amputação) e
/// [`super::tip_density`] (a grade no bico). *Uma lei escrita em dois sítios ainda não é
/// uma lei; só uma porta é* — e o modo de falha aqui é silencioso: as réguas mediriam
/// **pontas diferentes** e as tabelas leriam-se como se falassem da mesma.
///
/// # ⛔⛔⛔ O piso de `0,55` ESCONDIA as pontas da foto (2026-09-02)
///
/// Até 2026-09-01 um ápice tinha de estar a `≥ 0,55` do raio máximo, e a lista cortava em
/// `12`. Na escultura do dono isso dava **4** pontas — e as de que ele se queixava
/// (*«absolutamente nenhuma melhoria»*, sobre réguas que diziam o contrário) estavam a
/// `0,43`–`0,47` do raio: **nenhuma régua desta linha alguma vez as mediu.** Medido pela
/// saída do botão a `Detail 1,00` sobre `_base_sculpt.obj`, em unidades da aresta mediana:
///
/// | ápice | raio | grade a `3 h` do bico | visto pelo piso `0,55`? |
/// |---|---|---|---|
/// | `9663` | `1,00` | `0,60` | sim |
/// | `1463` | `0,61` | `1,10` | sim |
/// | `12074` · `15909` | `0,59` · `0,58` | `0,50` · `0,50` | sim |
/// | ⛔ `3138` | **`0,47`** | **`1,36`** | **não** |
/// | ⛔ `1943` | **`0,43`** | **`1,29`** | **não** |
///
/// # ⭐ O que separa um ESPINHO de uma BOSSA é a FORMA, não o raio
///
/// Baixar o piso sem mais chamaria «ponta» a cada bossa do corpo (`42` máximos locais na
/// peça dele, `27` na outra) — ⛔ **e as bossas lêem grade `1,0`–`1,47` mesmo na malha
/// aprovada** (`8` de `21` acima de `1,0` no `Sculpt_Blender.obj`), logo medi-las seria
/// acusar a peça aprovada. ⇒ o critério é o **cone** ([`cone_of`]): a razão entre o raio da
/// secção da entrada e a profundidade, na **pior** faixa de `2 × unit` entre `3` e `9 × unit`
/// do ápice. Um espinho é cónico até fundo; um botão ou cúpula salta para o corpo. Medido nas
/// duas peças do dono, em quatro densidades (`unit` de `0,029` a `0,10`):
///
/// | população | `unit 0,029` | `0,046` | `0,058` | `0,10` |
/// |---|---|---|---|---|
/// | espinhos AFIADOS de `_base_sculpt.obj` (`9663` · `12074` · `15909` · `3138` · `10230`) | `≤ 0,99` | `≤ 0,68` | `≤ 0,69` | `≤ 0,87` |
/// | espinhos AFIADOS de `sculpt_antes.obj` (`3810` · `8449` · `8285` · `4454`) | `≤ 1,07` | `≤ 0,80` | `≤ 0,72` | `≤ 0,91` |
/// | cúpulas e botões (`1463` · `1943` · `15341` · `7328` · `9776`) | `0,82`–`1,24` | `0,87`–`1,58` | `0,77`–`1,64` | `0,63`–`1,42` |
/// | bossas do corpo (`54`) | `≥ 1,31` | `≥ 1,33` | `≥ 1,27` | `≥ 0,90` |
///
/// ⇒ [`CONE_MAX`]` = 1,0`: os afiados entram (⚠️ a `unit 0,029` dois da outra peça lêem
/// `1,03`–`1,07` — o topo arredondado à escala fina — e ficam de fora **naquela** densidade,
/// onde a grade os resolve de qualquer maneira); os botões que o dono aceita grossos (`7328`
/// a `1,22`–`1,54`) ficam de fora em todas; as cúpulas `1463`/`1943` entram só onde são
/// cónicas àquela resolução. ⛔ **Uma barra mais alta (`1,2`) meteria o `7328` dentro por
/// `0,02`** e a régua acusaria a malha aprovada.
///
/// ⚠️ **O cone é relativo à RESOLUÇÃO, de propósito** — daí o `unit`. Uma cúpula que a `9 h`
/// ainda é cónica é, *àquela* resolução, um bico que a grade devia resolver; a uma grade mais
/// grossa a mesma cúpula é uma cúpula. ⛔ Uma versão sem `h` (anel a `2`–`6 %` do raio da
/// peça) foi medida e **não separa**: o espinho `3810` da peça aprovada lê `1,76` e uma bossa
/// lê `1,47` — a profundidade fixa cai onde o topo arredondado ainda é uma cúpula.
///
/// ⚠️ **Um ápice cujo anel não tem amostra CONTA como espinho** — é a agulha cuja entrada é
/// mais grosseira que `6 × unit` junto do bico (a ponta `4849` de `sculpt_antes.obj`), e
/// recusá-la seria medir a fixtura, não a peça.
///
/// ⚠️ **O centroide é o dos VÉRTICES, de propósito, e é o mesmo para a entrada e para a
/// saída.** Ele não é usado como medida de forma nenhuma — só para ordenar raios *dentro
/// da mesma malha* e para fixar as direcções que a saída depois responde. ⛔ Para medir a
/// **forma** (o alcance) o centroide tem de ser o da área: ver [`super::reach`].
///
/// ⚠️ **`MAX_TIPS = 32` é CUSTO, não lei**: cada ápice paga um Dijkstra curto aqui e outro
/// em cada régua. Uma peça com mais de `32` espinhos verdadeiros mede os `32` mais longos.
///
/// ⛔ **`unit` não positivo ⇒ lista vazia**: sem unidade não há cone, e uma lista sem o
/// filtro de forma diria que a peça tem `42` pontas.
///
/// ⚠️ **No PRODUTO a `unit` é o ALVO do slider e não a mediana de cada candidata**, de
/// propósito: o censo tem de ser o MESMO em todas as candidatas de um clique, senão o
/// selector compara *«3 pontas más de 8»* com *«2 de 7»* — listas diferentes. A mediana
/// ([`median_edge`]) é a unidade da bancada e das sondas, onde a saída de outra ferramenta
/// não tem alvo; as duas diferem `~8 %` e o registo imprime qual foi usada.
#[must_use]
pub fn apices(input: &Mesh, unit: f32) -> ([f32; 3], Vec<usize>) {
    const FLOOR: f32 = 0.25;
    const MAX_TIPS: usize = 32;
    let pos = input.positions();
    if pos.is_empty() || !unit.is_finite() || unit <= 0.0 {
        return ([0.0; 3], Vec::new());
    }
    if pos.is_empty() {
        return ([0.0; 3], Vec::new());
    }
    let mut c = [0.0f64; 3];
    for p in pos {
        for k in 0..3 {
            c[k] += f64::from(p[k]);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n = pos.len() as f64;
    #[allow(clippy::cast_possible_truncation)]
    let mid = [(c[0] / n) as f32, (c[1] / n) as f32, (c[2] / n) as f32];
    let r: Vec<f32> = pos.iter().map(|p| dist(*p, mid)).collect();
    let far = r.iter().copied().fold(0.0f32, f32::max).max(1.0e-9);

    let nbr = adjacency(input);
    let mut apex: Vec<usize> = (0..pos.len())
        .filter(|&i| r[i] >= FLOOR * far)
        .filter(|&i| nbr[i].iter().all(|&j| r[j as usize] <= r[i]))
        .collect();
    apex.sort_by(|&a, &b| r[b].total_cmp(&r[a]));
    apex.retain(|&i| cone_of(pos, &nbr, i, unit).is_none_or(|c| c <= CONE_MAX));
    apex.truncate(MAX_TIPS);
    (mid, apex)
}

/// **A adjacência vértice→vértices**, pelas arestas de todas as faces.
#[must_use]
pub fn adjacency(mesh: &Mesh) -> Vec<Vec<u32>> {
    let mut nbr: Vec<Vec<u32>> = vec![Vec::new(); mesh.positions().len()];
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
            nbr[a].push(v[(k + 1) % v.len()]);
            nbr[b].push(v[k]);
        }
    }
    nbr
}

/// ⭐ **A BOLA DE CAMINHO** — os vértices a menos de `lim` de distância **sobre as arestas**
/// da semente, com a distância de cada um.
///
/// ⚠️ **De CAMINHO e não em linha recta, e isso não é preciosismo:** uma vizinhança esférica
/// sobre um espinho fino apanha o **outro lado** do corpo. Medido numa versão anterior da
/// régua da grade: uma ponta de raio `1,32` leu `25,60` porque a fatia atravessava a peça.
///
/// ⚠️ `BTreeMap` e não `HashMap` — a ordem de iteração entra em somas de `f32`.
///
/// ⚠️ **Dijkstra com HEAP, e não a pilha que a 1.ª versão (dentro da régua da grade) usava:**
/// uma busca de correcção de rótulos por pilha re-relaxa cada vértice tantas vezes quantas
/// for melhorado, e numa bola grande isso explode — medido no portão das fixturas, a bola
/// de `16 × 0,056` sobre a escultura do dono levava **`71 s`** contra `5 s` da mesma régua
/// com bola mais pequena. A chave do heap é o par `(bits da distância, vértice)`: para
/// `f32` não negativos os bits ordenam como o valor, e o vértice desempata de forma
/// determinista.
#[must_use]
pub fn path_ball(
    pos: &[[f32; 3]],
    nbr: &[Vec<u32>],
    seed: usize,
    lim: f32,
) -> std::collections::BTreeMap<usize, f32> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let mut seen: std::collections::BTreeMap<usize, f32> =
        std::collections::BTreeMap::from([(seed, 0.0)]);
    let mut fila: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::new();
    fila.push(Reverse((0.0f32.to_bits(), seed)));
    while let Some(Reverse((bits, u))) = fila.pop() {
        let du = f32::from_bits(bits);
        if du > seen.get(&u).copied().unwrap_or(f32::MAX) {
            continue;
        }
        for &v in &nbr[u] {
            let v = v as usize;
            let nd = du + dist(pos[u], pos[v]);
            if nd <= lim && nd < seen.get(&v).copied().unwrap_or(f32::MAX) {
                seen.insert(v, nd);
                fila.push(Reverse((nd.to_bits(), v)));
            }
        }
    }
    seen
}

/// ⭐⭐⭐ **O CONE de um ápice** — o PIOR `Σ raio / Σ profundidade` das faixas de `2 × unit`
/// entre `3` e `9 × unit` abaixo do bico, ao longo do eixo local. `None` = *«entrada sem
/// amostra»*, que [`apices`] lê como espinho.
///
/// O eixo é do ápice ao centro dos vértices a `2`–`6 × unit` de caminho (⛔ uma PCA da
/// vizinhança apanha o CORPO num espinho curto — medido: `70°`/`85°`/`88°` de desvio em três
/// pontas). Em cada faixa `[3,5)`, `[5,7)`, `[7,9) × unit` de profundidade, a razão
/// `Σ perp / Σ t` dos vértices da bola de caminho é, num cone, a tangente do meio-ângulo —
/// e o que se devolve é a **pior** faixa.
///
/// # ⭐⭐⭐ Por que o PIOR das faixas, e por que ATÉ `9 × unit` (2026-09-02)
///
/// Um espinho é cónico **até fundo**; um botão é cónico junto ao topo e salta para o corpo.
/// Medido na peça aprovada (`sculpt_antes.obj`, `unit = 0,046`), raio da secção por unidade
/// de profundidade:
///
/// | ápice | `t = 3–4` | `4–5` | `5–6` | `6–7` | `7–8` | o que é |
/// |---|---|---|---|---|---|---|
/// | `4454` | `3,13` | `3,31` | `3,80` | `3,91` | `4,30` | espinho — `r/t` de `0,89` a `0,57` |
/// | `3810` | `2,60` | `2,81` | `2,96` | `3,14` | `3,32` | espinho |
/// | ⛔ `7328` | `3,51` | `3,98` | **`7,28`** | `7,20` | `7,51` | botão de `5` células: a `t = 5` está no CORPO |
/// | ⛔ `9776` | `2,42` | **`7,50`** | `6,08` | `5,92` | `6,30` | botão de `4` células |
///
/// ⭐ O QRemeshify deixa o `7328` com quads `1,35×` a mediana **e o dono aprovou** — um botão
/// de cinco células não é uma ponta que a grade tenha de resolver. Uma medida só junto do
/// topo (`2,5`–`4,5 × unit`) lia-o a `0,95` e chamava-lhe espinho; a pior faixa lê `1,32`.
///
/// # ⛔⛔ Duas AUSÊNCIAS que se lêem igual e são opostas (a 1.ª redacção confundiu-as)
///
/// - **A bola quase vazia** (`< 4` vértices): a ENTRADA é mais grosseira que a bola junto
///   do bico — a agulha `4849` de `sculpt_antes.obj`. ⇒ `None`, e conta como **espinho**.
/// - **A bola cheia sem ninguém em profundidade**: a superfície é PLANA à escala da bola —
///   uma bossa larga, ou o corpo da peça num empate de raio. ⇒ `+∞`, e conta como **bossa**.
///
/// A 1.ª redacção usava uma bola de `6 × unit` e devolvia `None` nos dois casos: uma bossa
/// gaussiana de `σ = 0,5 rad` nunca desce `4 × unit` dentro de `6 × unit` de caminho, e o
/// gate sintético devolveu a bossa **e** um vértice do corpo como espinhos. ⚠️ A bola é de
/// `16 × unit` por isso: numa esfera unitária, `9 × unit` de profundidade estão a
/// `≈ √(18·unit)` de caminho (`1,08` para `unit = 0,065`), e `16 × unit` cobre-o.
///
/// # ⚠️ O limite MEDIDO da lei: `unit ≳ 0,15 × raio do corpo`
///
/// Visto do ápice, um corpo esférico de raio `R` lê `r/t = cot(θ/2)`, que desce abaixo de
/// `1` quando a faixa chega a `t ≈ 0,6 R`. Com faixas até `9 × unit` isso acontece para
/// `unit ≳ 0,07 R`; medido, o corpo de `sculpt_antes.obj` (`R ≈ 1`) tem uma bossa a `0,90`
/// com `unit = 0,10` — a `Detail ≈ 0,5` a lei começa a chamar espinho a bossas do corpo. ⛔
/// Nas densidades do produto que o dono usa (`unit = 0,03`–`0,06`) as bossas lêem `≥ 1,21`.
#[must_use]
pub(crate) fn cone_of(pos: &[[f32; 3]], nbr: &[Vec<u32>], apex: usize, unit: f32) -> Option<f32> {
    const BALL: f32 = 16.0;
    let ball = path_ball(pos, nbr, apex, BALL * unit);
    if ball.len() < 4 {
        return None;
    }
    let a = pos[apex];
    let mut m = [0.0f64; 3];
    let mut n = 0usize;
    for (&v, &d) in &ball {
        if (2.0 * unit..=6.0 * unit).contains(&d) {
            for k in 0..3 {
                m[k] += f64::from(pos[v][k]);
            }
            n += 1;
        }
    }
    if n < 3 {
        return None;
    }
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let m = [
        (m[0] / n as f64) as f32,
        (m[1] / n as f64) as f32,
        (m[2] / n as f64) as f32,
    ];
    let u = sub(a, m);
    let len = dist(a, m);
    if len <= 1.0e-9 {
        return None;
    }
    let u = [u[0] / len, u[1] / len, u[2] / len];
    let mut worst: Option<f32> = None;
    for (lo, hi) in [(3.0f32, 5.0f32), (5.0, 7.0), (7.0, 9.0)] {
        let (mut perp_sum, mut depth_sum, mut cnt) = (0.0f64, 0.0f64, 0usize);
        for &v in ball.keys() {
            let q = sub(pos[v], a);
            let along = dot(q, u);
            let t = -along;
            if t >= lo * unit && t < hi * unit {
                let perp = dist(q, [u[0] * along, u[1] * along, u[2] * along]);
                perp_sum += f64::from(perp);
                depth_sum += f64::from(t);
                cnt += 1;
            }
        }
        if cnt >= 2 && depth_sum > 0.0 {
            #[allow(clippy::cast_possible_truncation)]
            let r = (perp_sum / depth_sum) as f32;
            worst = Some(worst.map_or(r, |w| w.max(r)));
        }
    }
    // A bola está cheia e ninguém desce: a superfície é plana à escala da bola — bossa.
    Some(worst.unwrap_or(f32::INFINITY))
}

/// ⭐ **A ARESTA MEDIANA de uma malha** — a UNIDADE de todas as réguas da ponta.
///
/// ⚠️ **A mediana da SAÍDA e não o alvo do slider, de propósito** (2026-09-02): o olho
/// compara o quad do bico com os quads do resto da mesma malha, e uma malha de outra
/// ferramenta não tem alvo nenhum — é o que torna a régua comparável com a retopologia que o
/// dono aprovou. ⚠️ Medido: o alvo e a mediana diferem `~8 %` no botão, e duas unidades
/// para a mesma régua dariam duas leituras que ninguém pode pôr na mesma tabela.
///
/// Devolve `0` numa malha sem arestas — *«não há unidade»*, e as réguas lêem-no como
/// *«não medido»*.
#[must_use]
pub fn median_edge(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let mut seen: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
    let mut lens: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let e = if a < b { (a, b) } else { (b, a) };
            if seen.insert(e) {
                lens.push(dist(pos[a as usize], pos[b as usize]));
            }
        }
    }
    if lens.is_empty() {
        return 0.0;
    }
    lens.sort_by(f32::total_cmp);
    lens[lens.len() / 2]
}

/// **QUANTAS PONTAS A CADEIA CORTOU** — ver [`tip_survival`].
#[derive(Debug, Clone, Copy, Default)]
pub struct TipSurvival {
    /// Quantos ápices perderam mais que [`TIP_CUT_PCT`] do alcance deles.
    pub cut: usize,
    /// Quantos ápices foram medidos.
    pub total: usize,
    /// A pior perda, em percentagem (negativa).
    pub worst_pct: f32,
}

/// ⚠️ **A barra do que conta como CORTE.** Abaixo disto é reamostragem: a saída tem
/// outros vértices e a função de suporte cai um pouco só por a superfície ser
/// poliédrica. ⭐ Medido: as pontas **intactas** da peça do artista medem `−0,0 %` a
/// `−0,4 %`, e as **cortadas** medem `−5 %` a `−22 %`. *Há uma ordem de grandeza entre
/// as duas populações, e a barra vive nela.*
pub const TIP_CUT_PCT: f32 = -2.0;

/// ⭐⭐⭐ **UMA MEDIÇÃO POR PONTA** — e ela existe porque uma foto tinha uma seta VERDE
/// e uma VERMELHA na **mesma** peça (Enio, 2026-08-30).
///
/// ⛔⛔ **O alcance global não podia ver isso:** ele é a distância **máxima** ao
/// centroide, *um único extremo*. Uma ponta que sobrevive esconde outra cortada — e na
/// peça dele o alcance dizia `−16,2 %` enquanto **dez** das doze pontas estavam
/// intactas a `−0,1 %` e **duas** tinham perdido `20 %`.
///
/// # ⭐ Como uma ponta é achada
///
/// Um **ápice** é um vértice cujo raio ao centroide é maior que o de todos os vizinhos
/// — um máximo local no grafo da superfície. ⚠️ *Não é um limiar de raio*: numa peça com
/// espinhos de comprimentos diferentes, um limiar apanharia dois vértices do mais longo
/// e nenhum do mais curto.
///
/// # ⭐⭐ E a comparação é a FUNÇÃO DE SUPORTE
///
/// Para cada direcção de ápice `d`, mede-se `max(v · d)` na entrada e na saída — *até
/// onde a peça vai para aquele lado*. ⚠️ **Sobrevive à malha ser outra:** os vértices
/// não se correspondem entre entrada e saída, as **direcções** sim.
#[must_use]
pub fn tip_survival(input: &Mesh, output: &Mesh) -> TipSurvival {
    let pos = input.positions();
    // ⚠️ A unidade é a aresta mediana da SAÍDA ([`median_edge`]) — é ela que diz a que
    // resolução um máximo local é um espinho ([`apices`]).
    let unit = median_edge(output);
    if pos.is_empty() || output.positions().is_empty() || unit <= 0.0 {
        return TipSurvival::default();
    }
    let (mid, apex) = apices(input, unit);

    let support = |m: &Mesh, d: [f32; 3]| -> f32 {
        m.positions()
            .iter()
            .map(|p| {
                let q = sub(*p, mid);
                dot(q, d)
            })
            .fold(f32::MIN, f32::max)
    };
    let mut s = TipSurvival::default();
    for &i in &apex {
        let len = dist(pos[i], mid).max(1.0e-9);
        let d = [
            (pos[i][0] - mid[0]) / len,
            (pos[i][1] - mid[1]) / len,
            (pos[i][2] - mid[2]) / len,
        ];
        let (a, b) = (support(input, d), support(output, d));
        if a.abs() <= 1.0e-9 {
            continue;
        }
        let pct = 100.0 * (b / a - 1.0);
        s.total += 1;
        s.worst_pct = s.worst_pct.min(pct);
        if pct < TIP_CUT_PCT {
            s.cut += 1;
        }
    }
    s
}

#[cfg(test)]
#[path = "apex_tests.rs"]
mod tests;
