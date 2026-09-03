//! ⭐⭐⭐ **O BALDE** (plano 40) — a região que o clique aponta vira forma.
//!
//! Enio, 2026-08-31: *"Preenche áreas por linhas fechadas ou linhas sobrepostas."*
//!
//! # A lei
//!
//! > **A face que o clique aponta tem por fronteira uma sequência de ARCOS INTEIROS da rede.**
//!
//! ⭐ **É por isto que o SOLDAR veio primeiro.** Cortar todo contorno nos cruzamentos faz cada arco
//! ir de nó a nó, e então **nenhum ponto interior de um arco é fronteira de face**. A face é um
//! ciclo de arcos, e reconstruí-la em bézier é **concatenar arcos** — sem aproximação, sem faceta,
//! sem depender do zoom.
//!
//! ⚠️ **O artista não precisa de ter soldado**: o balde faz o mesmo corte numa cópia, pela mesma
//! porta ([`ph2d_vec_scene::trim_tool::crossings_against`] + [`ph2d_vec_scene::weld::split_at`] +
//! [`ph2d_vec_scene::weld::cluster_endpoints`]). Soldar é o verbo que torna a rede **autorada**; o
//! balde só precisa dela no instante do clique.
//!
//! # ⛔ Por que isto não é o Shape Builder
//!
//! O `ph2d_vec_boolean::arrangement` responde *"que face é esta?"* **sem DCEL**, porque uma face
//! tem definição conjuntista: `região(M) = ∩M − ∪¬M`. ⛔ **Essa definição não existe para um traço
//! ABERTO** — uma linha não tem dentro, e nenhuma pertinência a descreve. Este módulo é o DCEL que
//! aquele evitou, e existe pela fronteira que o próprio doc dele nomeia.
//!
//! # ⛔ E por que não é o balde do Flip
//!
//! O `ph2d-flip-fill` tem o arranjo certo e o substrato errado: ele fala `Vec2`/`f32`/**polilinha**,
//! e **apaga a proveniência** de propósito no `Half` — a saída dele são dois sacos de pontos. O
//! próprio doc dele diz *"se o traço um dia ganhar alças, o ajuste entra aqui"*: ele nunca ganhou.
//! Reconstruir a fronteira em **arcos** é a razão de existir deste módulo, e ela não sobrevive a
//! uma travessia por polígono.

use ph2d_vec_scene::{VecVertex, detection_polyline, point_in_polygon, trim_tool, weld};

/// **O piso da folga com que duas pontas são o mesmo nó**, como fracção da diagonal da arte.
///
/// ⚠️ **Fracção, e não um número absoluto**: a escala do documento é do artista, e um valor fixo
/// trataria uma peça de 5 unidades e uma de 5 000 igual. É o mesmo valor e a mesma lei do
/// `ph2d_flip_fill`'s `WELD_FRACTION` — a pergunta é a mesma, e ela já tinha resposta medida.
const NODE_WELD_FRACTION: f64 = 1e-5;

/// Um **arco** da rede: a geometria (bézier, em MUNDO) e os dois nós que ele liga.
#[derive(Clone, Debug)]
pub struct Arco {
    /// Os vértices do arco, na ordem em que ele é percorrido para a frente.
    pub verts: Vec<VecVertex>,
    /// O nó de partida.
    pub de: usize,
    /// O nó de chegada. ⚠️ **Pode ser o mesmo que [`Self::de`]**: um anel que não encontra ninguém
    /// é um LAÇO, e é assim que ele entra na rede.
    pub ate: usize,
}

/// A **rede**: os arcos, os nós, e a polilinha de cada arco (a mesma da detecção de cruzamentos).
#[derive(Clone, Debug, Default)]
pub struct Rede {
    /// Os arcos, na ordem em que foram cortados.
    pub arcos: Vec<Arco>,
    /// A posição de cada nó, em MUNDO.
    pub nos: Vec<[f64; 2]>,
    /// ⛔ **O documento passou do tecto de amostragem**
    /// ([`ph2d_vec_scene::trim_tool::MAX_SAMPLES_BATCH`]) e a rede está **vazia** — não há faces, e
    /// quem chama tem de o DIZER em vez de agir.
    ///
    /// ⚠️ A resposta antiga era devolver zero cruzamentos, e é a pior possível: sem cruzamentos toda
    /// forma volta a ser um anel inteiro, e o preenchimento **salta para a forma toda** em silêncio.
    pub recusada: bool,
    /// A polilinha de cada arco — o achatado com que as faces são medidas.
    ///
    /// ⚠️⚠️ **É a MESMA amostragem com que os cruzamentos foram achados**
    /// ([`detection_polyline`]). Duas amostragens diferentes discordariam sobre a existência de um
    /// cruzamento, e a face apareceria num sítio e não noutro.
    poly: Vec<Vec<[f64; 2]>>,
}

/// Uma **face**: o ciclo de meias-arestas que a cerca.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    /// `(índice do arco, para a frente?)`, na ordem do passeio.
    pub arcos: Vec<(usize, bool)>,
    /// A área com sinal do achatado. Positiva = face limitada (o interior fica à esquerda).
    pub area: f64,
}

/// **A rede de uma lista de contornos** — já cozidos e no MUNDO.
///
/// Cada contorno é `(vértices, fecha?)`. Contornos degenerados são ignorados.
#[must_use]
pub fn rede(contornos: &[(Vec<VecVertex>, bool)]) -> Rede {
    let esc = escala(contornos);
    // ── 1. CORTAR nos cruzamentos.
    //
    // ⛔⛔ **UMA passagem, e não uma por contorno.** Perguntar por contorno é `O(n³)` — o
    // `crossings_against` soma as arestas do alvo MAIS as de todos os outros, então o mesmo total
    // era construído e comparado com o mesmo tecto `n` vezes. Medido em círculos que se cruzam: a
    // `64` deles a resposta é certa e custa **764 ms**; a `65` **todos os cruzamentos desapareciam**
    // e cada forma voltava a ser um anel inteiro, sem um aviso.
    let Some(por_contorno) = trim_tool::crossings_all(contornos, esc) else {
        // ⛔ Acima do tecto a resposta é uma RECUSA, e a rede sai VAZIA — sem faces. ⚠️ Devolver
        // zero cruzamentos daria faces ERRADAS (cada forma inteira), e o preenchimento saltaria
        // para elas em silêncio.
        return Rede {
            recusada: true,
            ..Rede::default()
        };
    };
    let mut geom: Vec<(Vec<VecVertex>, bool)> = Vec::new();
    for (k, (verts, closed)) in contornos.iter().enumerate() {
        let mut xings = por_contorno[k].clone();
        // ⚠️ **Um anel que não encontra ninguém tem de virar um LAÇO**, e não ficar fechado: um
        // contorno fechado sem ponta não tem meia-aresta, e a face que ele delimita ficaria
        // invisível para o passeio. Cortá-lo num sítio qualquer dá **um** arco aberto cujas duas
        // pontas caem no mesmo nó — que é exactamente um laço.
        if *closed && xings.is_empty() {
            xings.push(0.5);
        }
        for (v, c) in weld::split_at(verts, *closed, &xings) {
            if !c && v.len() >= 2 {
                geom.push((v, false));
            }
        }
    }
    // ── 2. OS NÓS: as pontas que caem no mesmo sítio são um só.
    // ⚠️⚠️ **A folga tem DOIS pisos, e o segundo não é conforto.** O primeiro é a flecha dobrada
    // (cada lado de um cruzamento erra a sua, em direcções opostas). ⛔ Mas numa RECTA a flecha é
    // **zero** — a corda é a curva —, e aí os dois lados calculam o mesmo cruzamento com um
    // resíduo de ponto flutuante de `~1e-15`: com folga zero eles **não** se juntam, cada ponta
    // vira um nó próprio, a rede fica desligada e **não existe face nenhuma**. Foi assim que o
    // gate das quatro linhas soltas reprovou.
    //
    // ⚠️ O piso é uma **fracção da diagonal** da arte, e não um número: é a lei que o
    // `ph2d-flip-fill` já usa para a mesma pergunta (`WELD_FRACTION`), pela mesma razão — *a escala
    // do documento é do artista*.
    let tol = (2.0
        * contornos
            .iter()
            .map(|(v, c)| trim_tool::sampling_error(v, *c))
            .fold(0.0_f64, f64::max))
    .max(esc * NODE_WELD_FRACTION)
    // ⭐⭐⭐ **E o mesmo piso do TOQUE.** Quando uma ponta pousa numa parede, a parede ganha um nó na
    // projecção e a ponta fica a até essa folga dele — se o agrupamento não a alcançasse, o toque
    // seria reconhecido e o NÓ ficaria partido em dois, que é o mesmo que não o reconhecer.
    // *Duas perguntas com a mesma resposta têm de ter a mesma régua.*
    .max(esc * ph2d_vec_scene::trim_tool::TOUCH_FRACTION);
    let pontas: Vec<[f64; 2]> = geom
        .iter()
        .flat_map(|(v, _)| [v[0].anchor, v[v.len() - 1].anchor])
        .collect();
    let (de_quem, mut nos) = weld::cluster_endpoints(&pontas, tol);
    // ⚠️ **Uma ponta sozinha também é um nó** — ela é o fim de uma linha solta, e o passeio precisa
    // de a poder atravessar (é ali que a face dá meia-volta). O agrupador não lhe dá nó porque a
    // pergunta DELE é *"isto é uma junta?"*.
    let ids: Vec<usize> = de_quem
        .iter()
        .enumerate()
        .map(|(k, n)| {
            n.unwrap_or_else(|| {
                nos.push(pontas[k]);
                nos.len() - 1
            })
        })
        .collect();
    let arcos: Vec<Arco> = geom
        .iter()
        .enumerate()
        .map(|(i, (v, _))| Arco {
            verts: v.clone(),
            de: ids[i * 2],
            ate: ids[i * 2 + 1],
        })
        .collect();
    let poly: Vec<Vec<[f64; 2]>> = geom
        .iter()
        .map(|(v, _)| detection_polyline(v, false))
        .collect();
    descartar_duplicados(
        Rede {
            arcos,
            nos,
            poly,
            recusada: false,
        },
        tol,
    )
}

/// ⛔⛔ **DOIS ARCOS SOBREPOSTOS destroem o passeio — e não só a região deles.**
///
/// Report do Enio (2026-09-01): *"a depender da posição dos pontos o preenchimento some."*
///
/// ⚠️ **Medido:** com uma parede a cair **exactamente em cima** de outra (o artista arrasta um nó
/// até a curva encostar na aresta vizinha), a rede inteira passava de **3 faces a 1** — e a região
/// do outro lado do desenho, que nada tinha a ver, perdia o preenchimento junto. *Duas
/// meias-arestas com a mesma direcção de saída são indistinguíveis para o passeio, e ele fecha um
/// ciclo só, gigante.* É o mesmo mecanismo que fazia um preenchimento devolvido à rede envenenar as
/// vizinhas.
///
/// ⇒ Dois arcos que ligam **o mesmo par de nós** e passam **pelo mesmo sítio** são o mesmo arco: o
/// segundo cai. ⛔ **O par de nós sozinho não chega** — duas curvas diferentes entre os mesmos dois
/// nós são uma lente, e ela é uma região legítima; por isso a comparação inclui o ponto do MEIO.
fn descartar_duplicados(r: Rede, tol: f64) -> Rede {
    let t2 = (tol.max(1e-9)) * (tol.max(1e-9));
    let meio = |p: &[[f64; 2]]| p.get(p.len() / 2).copied().unwrap_or([0.0, 0.0]);
    let mut fica: Vec<bool> = vec![true; r.arcos.len()];
    for i in 0..r.arcos.len() {
        if !fica[i] {
            continue;
        }
        let (a, b) = (
            r.arcos[i].de.min(r.arcos[i].ate),
            r.arcos[i].de.max(r.arcos[i].ate),
        );
        let mi = meio(&r.poly[i]);
        for (j, vivo) in fica.iter_mut().enumerate().skip(i + 1) {
            if !*vivo {
                continue;
            }
            let (c, d) = (
                r.arcos[j].de.min(r.arcos[j].ate),
                r.arcos[j].de.max(r.arcos[j].ate),
            );
            if (a, b) != (c, d) {
                continue;
            }
            let mj = meio(&r.poly[j]);
            let v = [mi[0] - mj[0], mi[1] - mj[1]];
            if v[0].mul_add(v[0], v[1] * v[1]) <= t2 {
                *vivo = false;
            }
        }
    }
    if fica.iter().all(|k| *k) {
        return r;
    }
    let (mut arcos, mut poly) = (Vec::new(), Vec::new());
    for (i, k) in fica.iter().enumerate() {
        if *k {
            arcos.push(r.arcos[i].clone());
            poly.push(r.poly[i].clone());
        }
    }
    Rede {
        arcos,
        nos: r.nos,
        poly,
        recusada: r.recusada,
    }
}

impl Rede {
    /// **A face que contém `p`** — a de menor área entre as limitadas que o contêm.
    ///
    /// ⚠️ **A menor, e é ela que resolve o aninhamento**: um ponto dentro de um quadrado que está
    /// dentro doutro está dentro das duas faces, e a que o dedo aponta é a de dentro.
    ///
    /// `None` quando o ponto não está fechado por nada (o clique caiu no lado de fora) — e essa é a
    /// resposta certa: *preencher o infinito não é preencher*.
    #[must_use]
    pub fn face_em(&self, p: [f64; 2]) -> Option<Face> {
        let mut melhor: Option<Face> = None;
        for f in self.faces() {
            if f.area <= 0.0 {
                continue; // a face de fora, e as degeneradas
            }
            if !point_in_polygon(&self.contorno(&f), p) {
                continue;
            }
            if melhor.as_ref().is_none_or(|m| f.area < m.area) {
                melhor = Some(f);
            }
        }
        melhor
    }

    /// **TODAS as faces da rede**, cada uma como o ciclo de meias-arestas que a cerca.
    ///
    /// A meia-aresta `2i` percorre o arco `i` para a frente e `2i+1` para trás; o gémeo é `h ^ 1`.
    #[must_use]
    pub fn faces(&self) -> Vec<Face> {
        let n = self.arcos.len() * 2;
        // Em cada nó, as meias-arestas que SAEM dele, ordenadas pelo ângulo de saída.
        let mut saem: Vec<Vec<usize>> = vec![Vec::new(); self.nos.len()];
        for h in 0..n {
            saem[self.origem(h)].push(h);
        }
        for lista in &mut saem {
            lista.sort_by(|a, b| self.angulo(*a).total_cmp(&self.angulo(*b)));
        }
        let mut visto = vec![false; n];
        let mut out = Vec::new();
        for inicio in 0..n {
            if visto[inicio] {
                continue;
            }
            let mut ciclo = Vec::new();
            let mut h = inicio;
            loop {
                visto[h] = true;
                ciclo.push((h / 2, h.is_multiple_of(2)));
                h = self.proxima(&saem, h);
                if h == inicio {
                    break;
                }
                if visto[h] {
                    break; // rede malformada: não deixa o passeio girar para sempre
                }
            }
            let face = Face {
                arcos: ciclo,
                area: 0.0,
            };
            let area = area_com_sinal(&self.contorno(&face));
            out.push(Face { area, ..face });
        }
        out
    }

    /// **A GEOMETRIA da face**, em bézier — os arcos concatenados, na ordem do passeio.
    ///
    /// ⚠️ **A ponta partilhada entra UMA vez.** O último vértice de um arco e o primeiro do
    /// seguinte são o mesmo nó; emitir os dois deixaria um segmento de comprimento zero em cada
    /// canto — invisível no ecrã e venenoso para tudo o que mede comprimento de arco.
    ///
    /// ⚠️ **Um arco percorrido para trás tem as ALÇAS trocadas**: o `in_handle` de um vértice é o
    /// que chega, e ao inverter o sentido ele passa a ser o que sai.
    #[must_use]
    pub fn geometria(&self, face: &Face) -> Vec<VecVertex> {
        let mut out: Vec<VecVertex> = Vec::new();
        for &(i, frente) in &face.arcos {
            let mut vs = self.arcos[i].verts.clone();
            if !frente {
                vs.reverse();
                for v in &mut vs {
                    std::mem::swap(&mut v.in_handle, &mut v.out_handle);
                }
            }
            if out.is_empty() {
                out = vs;
            } else {
                // A ponta partilhada: a alça de SAÍDA vem do arco novo, a de chegada fica.
                if let (Some(fim), Some(ini)) = (out.last_mut(), vs.first()) {
                    fim.out_handle = ini.out_handle;
                }
                out.extend(vs.into_iter().skip(1));
            }
        }
        // O fecho: o último vértice é o primeiro nó outra vez.
        if out.len() > 1 {
            let ultimo = out.remove(out.len() - 1);
            if let Some(primeiro) = out.first_mut() {
                primeiro.in_handle = ultimo.in_handle;
            }
        }
        out
    }

    /// ⭐⭐⭐ **UM PONTO BEM DENTRO da face** — o sítio onde uma semente sobrevive à edição.
    ///
    /// ⚠️ **A semente do clique fica onde o dedo caiu, e isso pode ser encostado à borda.** Medido:
    /// com a semente a `0,5` de uma parede, arrastar essa parede **por cima dela** faz a face
    /// deixar de a conter, e o preenchimento perde a região que estava a seguir. Re-semear aqui a
    /// cada re-cozimento mantém o ponto **fundo** — a parede tem de varrer o miolo inteiro para o
    /// perder, que é quando a região de facto deixou de existir.
    ///
    /// O centroide serve quando cai dentro (o caso comum); numa face **côncava** ele pode cair
    /// fora, e aí a resposta é a amostra da grelha mais **afastada da borda**.
    #[must_use]
    pub fn interior_point(&self, face: &Face) -> Option<[f64; 2]> {
        let poly = self.contorno(face);
        if poly.len() < 3 {
            return None;
        }
        let n = poly.len() as f64;
        let c = [
            poly.iter().map(|p| p[0]).sum::<f64>() / n,
            poly.iter().map(|p| p[1]).sum::<f64>() / n,
        ];
        if point_in_polygon(&poly, c) {
            return Some(c);
        }
        let mut melhor: Option<([f64; 2], f64)> = None;
        for p in self.interior_samples(face) {
            let d = poly
                .windows(2)
                .map(|w| dist_ponto_segmento(p, w[0], w[1]))
                .fold(f64::INFINITY, f64::min);
            if melhor.as_ref().is_none_or(|(_, b)| d > *b) {
                melhor = Some((p, d));
            }
        }
        melhor.map(|(p, _)| p)
    }

    /// ⭐⭐⭐ **AS AMOSTRAS DO MIOLO de uma face** — com que se mede *quanto* de uma face era de quem.
    ///
    /// # Porque uma face precisa de MUITOS pontos, e não de um
    ///
    /// Report do Enio (2026-09-02): *"quando atravessamos uma linha com um nó, os preenchimentos se
    /// quebram"*. Medido sobre um quadrado de 400 com uma linha ao meio:
    ///
    /// | o gesto | o que a rede passa a ter | com UMA semente |
    /// |---|---|---|
    /// | a linha entra e parte a região | 2 faces de `200` | a tinta fica com **uma**: metade some |
    /// | um nó atravessa o topo e funde | `4,17` + `395,83` | as DUAS sementes caem na mesma face |
    ///
    /// ⇒ *uma semente diz **onde** a tinta estava; ela não diz **quanto** da face era dela.* A
    /// resposta a «de quem é esta face?» é uma **votação**, e são estas as amostras que votam.
    ///
    /// ⚠️ **Uniformes sobre a CAIXA da face**, e por isso a contagem de votos de uma região é
    /// proporcional à **área** que ela cobre dentro desta face — que é a grandeza que a lei do
    /// *Live Paint* pede (*"a face fundida fica com a tinta da maior"*).
    ///
    /// # ⛔⛔ E porque é uma VARREDURA, e não uma grelha
    ///
    /// Report do Enio (2026-09-02, com fotos): *"comportamento bem melhor mas com inconsistências e
    /// falhas"* — arrastar um nó para longe cria uma **espiga**, e as regiões dela nasciam sem cor
    /// nenhuma. Medido, com a grelha de 15×15 sobre a caixa:
    ///
    /// | a região | área | densidade na caixa | amostras |
    /// |---|---|---|---|
    /// | espiga larga | `2000` | `6,7%` | `8` |
    /// | espiga fina | `400` | `1,3%` | **`0`** |
    /// | espiga finíssima | `100` | `0,3%` | **`0`** |
    ///
    /// ⇒ *uma região comprida e magra na diagonal não tem UM ponto da grelha dentro dela*, e uma
    /// face sem amostra não vota, não é votada e não herda tinta nenhuma. ⛔ E o defeito lia-se como
    /// intermitente — a grelha acerta ou falha conforme o ângulo.
    ///
    /// A varredura acha os **intervalos interiores** de cada linha e semeia-os na MESMA malha de
    /// `x` — mesma densidade, mesma proporcionalidade —, e um intervalo mais estreito do que a
    /// malha recebe o **ponto do meio** em vez de nada.
    #[must_use]
    pub fn interior_samples(&self, face: &Face) -> Vec<[f64; 2]> {
        let poly = self.contorno(face);
        if poly.len() < 3 {
            return Vec::new();
        }
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in &poly {
            lo = [lo[0].min(p[0]), lo[1].min(p[1])];
            hi = [hi[0].max(p[0]), hi[1].max(p[1])];
        }
        const N: usize = 15;
        let (dx, dy) = ((hi[0] - lo[0]) / N as f64, (hi[1] - lo[1]) / N as f64);
        let mut out = Vec::new();
        for j in 0..N {
            let y = lo[1] + dy * (j as f64 + 0.5);
            // Onde a fronteira atravessa esta linha — os extremos dos intervalos INTERIORES.
            let mut xs: Vec<f64> = Vec::new();
            for k in 0..poly.len() {
                let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
                if (a[1] <= y) == (b[1] <= y) {
                    continue;
                }
                let t = (y - a[1]) / (b[1] - a[1]);
                xs.push(t.mul_add(b[0] - a[0], a[0]));
            }
            xs.sort_by(f64::total_cmp);
            for par in xs.as_chunks::<2>().0 {
                let (a, b) = (par[0], par[1]);
                // A malha de `x` é GLOBAL (ancorada em `lo`), e não relativa a cada intervalo: é
                // isso que mantém a densidade uniforme, e a densidade uniforme é o que faz a
                // contagem de votos ser proporcional à ÁREA.
                let mut i = ((a - lo[0]) / dx).floor().max(0.0);
                let mut emitiu = false;
                loop {
                    let x = dx.mul_add(i + 0.5, lo[0]);
                    if x > b {
                        break;
                    }
                    if x >= a {
                        out.push([x, y]);
                        emitiu = true;
                    }
                    i += 1.0;
                }
                if !emitiu {
                    // ⚠️ Um intervalo mais estreito do que a malha ficaria MUDO — e é justamente o
                    // caso que o report de 2026-09-02 trouxe. Ele recebe o ponto do meio: pesa um
                    // pouco mais do que a área dele, e é infinitamente melhor do que zero.
                    out.push([f64::midpoint(a, b), y]);
                }
            }
        }
        // ⭐ **O filtro é a MESMA contenção que o resto usa.** A varredura é só um gerador de
        // candidatos esperto; quem decide *dentro* continua a ser uma porta só, senão uma face
        // não-simples seria amostrada por uma regra e reclamada por outra.
        out.retain(|p| point_in_polygon(&poly, *p));
        out
    }

    /// **O achatado de uma face**, para medir área e contenção — o par plano da [`Self::geometria`],
    /// que devolve a mesma fronteira em cúbicas.
    ///
    /// ⚠️ **É esta a polilinha com que a rede foi construída** (`detection_polyline`), e não uma
    /// segunda amostragem: quem decide de quem é uma face tem de perguntar à MESMA curva que
    /// decidiu onde ela começa.
    #[must_use]
    pub fn contorno(&self, face: &Face) -> Vec<[f64; 2]> {
        let mut out: Vec<[f64; 2]> = Vec::new();
        for &(i, frente) in &face.arcos {
            let p = &self.poly[i];
            if frente {
                out.extend(p.iter().copied());
            } else {
                out.extend(p.iter().rev().copied());
            }
        }
        out
    }

    /// O nó de onde a meia-aresta `h` sai.
    fn origem(&self, h: usize) -> usize {
        let a = &self.arcos[h / 2];
        if h.is_multiple_of(2) { a.de } else { a.ate }
    }

    /// O ângulo da direcção com que `h` SAI do nó dela — a primeira corda da polilinha.
    fn angulo(&self, h: usize) -> f64 {
        let p = &self.poly[h / 2];
        if p.len() < 2 {
            return 0.0;
        }
        let (a, b) = if h.is_multiple_of(2) {
            (p[0], p[1])
        } else {
            (p[p.len() - 1], p[p.len() - 2])
        };
        (b[1] - a[1]).atan2(b[0] - a[0])
    }

    /// **A meia-aresta seguinte no passeio de face.**
    ///
    /// ⚠️ **É o gémeo girado UM passo em sentido horário** à volta do nó de chegada. Essa é a regra
    /// que faz as faces limitadas saírem com o interior à ESQUERDA (área positiva) e a face de fora
    /// com área negativa — e é o que torna *"a de menor área positiva"* uma pergunta bem-posta.
    fn proxima(&self, saem: &[Vec<usize>], h: usize) -> usize {
        let gemeo = h ^ 1;
        let v = self.origem(gemeo);
        let lista = &saem[v];
        let k = lista.iter().position(|&x| x == gemeo).unwrap_or(0);
        lista[(k + lista.len() - 1) % lista.len()]
    }
}

/// A distância de um ponto ao SEGMENTO — a régua de *"quão fundo"*.
fn dist_ponto_segmento(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let len2 = ab[0].mul_add(ab[0], ab[1] * ab[1]);
    if len2 <= f64::EPSILON {
        return ap[0].hypot(ap[1]);
    }
    let t = (ap[0].mul_add(ab[0], ap[1] * ab[1]) / len2).clamp(0.0, 1.0);
    (ap[0] - t * ab[0]).hypot(ap[1] - t * ab[1])
}

/// A área com sinal de um polígono (shoelace). Positiva = anti-horário.
fn area_com_sinal(poly: &[[f64; 2]]) -> f64 {
    if poly.len() < 3 {
        return 0.0;
    }
    let mut s = 0.0;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        s += (poly[j][0] - poly[i][0]) * (poly[j][1] + poly[i][1]);
        j = i;
    }
    s * 0.5
}

/// A diagonal da caixa de tudo o que entra — a régua com que duas travessias quase-coincidentes são
/// a mesma. ⛔ Um número fixo trataria uma peça de 5 unidades e uma de 5 000 igual.
fn escala(contornos: &[(Vec<VecVertex>, bool)]) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for (verts, _) in contornos {
        for v in verts {
            lo = [lo[0].min(v.anchor[0]), lo[1].min(v.anchor[1])];
            hi = [hi[0].max(v.anchor[0]), hi[1].max(v.anchor[1])];
        }
    }
    if !lo[0].is_finite() {
        return 1.0;
    }
    ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2))
        .sqrt()
        .max(1e-6)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
