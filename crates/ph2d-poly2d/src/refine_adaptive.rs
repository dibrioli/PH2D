//! ⭐⭐⭐ **O REFINAMENTO ADAPTATIVO — `k` por TRIÂNGULO, e NENHUM nó pendurado.**
//!
//! # O defeito que o trouxe (auditoria de 2026-09-16)
//!
//! O botão **Smooth** do painel dos ossos era um **controlo morto**: ele entregava, bit a bit, a
//! mesma malha que o `Fast`. A causa é aritmética e não tem nada de subtil — a lei uniforme parte
//! **todos** os triângulos em `k × k`, logo o orçamento de peças do quadro impõe
//! `k ≤ ⌊√(orçamento / peças)⌋`, e **toda malha acima de `orçamento / 4` peças só admite `k = 1`**.
//!
//! | cena medida | peças | orçamento | `max_split` | `Smooth` |
//! |---|---:|---:|---:|---|
//! | malha de bind de 2026-09-15 | `2 430` | `1 543` | `1` | **== `Fast`** |
//! | a mesma, com o orçamento corrigido | `2 430` | `3 787` | `1` | **== `Fast`** |
//! | a do smoke do braço | `780` | `1 543` | `1` | **== `Fast`** |
//!
//! ⛔⛔ **Subir o orçamento NÃO cura** — a coluna do meio prova-o. Para `k = 2` numa malha de
//! `2 430` peças o orçamento teria de ser `9 720`, que a `1,08 µs`/peça é **`10,5 ms`**, dois
//! terços de um quadro de 60 fps. *O `k` global é a grandeza errada: ele paga refinamento onde a
//! arte está lisa para o comprar onde ela dobra.*
//!
//! # ⭐⭐⭐ A lei, e porque ela NÃO abre fendas
//!
//! A operação elementar aqui **não é «partir um triângulo»** — é **«partir uma ARESTA»**. Partir a
//! aresta `(u,v)` insere o ponto médio `m` e parte **os dois** triângulos que a possuem, ligando
//! cada um ao seu vértice oposto. Depois disso a aresta `(u,v)` **deixou de existir** dos dois
//! lados, logo `m` não fica pendurado em nada: a malha é conforme **depois de cada passo**, e não
//! só no fim.
//!
//! ⇒ é isto que refuta a frase que o cabeçalho da [`crate::refine`] carregava (*«um `k` por
//! triângulo abriria nós pendurados»*): ela descrevia a única construção que se tinha tentado —
//! uma grelha baricêntrica própria por triângulo —, não a pergunta.
//!
//! # ⚠️ Qual aresta se parte: a MAIS LONGA, e a razão é a FORMA
//!
//! Escolher sempre a aresta de pior desvio degenera os triângulos (bissecar sempre a mesma direcção
//! afina-os sem limite). A lei que se usa é a **bissecção da aresta mais longa** (Rivara): parte-se
//! a aresta mais longa do triângulo escolhido, e se o vizinho do outro lado **não** a tem como a
//! sua mais longa, refina-se primeiro o vizinho — a cadeia (`LEPP`) sobe por arestas
//! estritamente maiores até a um par que concorda, e ali parte-se. ⭐ Isso dá um **piso para os
//! ângulos** que depende só da malha de entrada, e a nossa entra de uma grelha.
//!
//! ⚠️⚠️ **A cadeia só termina porque a ordem das arestas é TOTAL.** Numa malha de grelha há
//! empates de comprimento **por construção** (toda célula tem a mesma diagonal), e com empates a
//! cadeia pode voltar a um triângulo já visitado e girar para sempre. ⇒ a chave de ordem é
//! `(comprimento, vértice menor, vértice maior)`, que não tem empates possíveis — ver [`maior`].
//!
//! # ⭐ Quem se parte a seguir: o pior DESVIO, e o custo disso é ZERO
//!
//! O desvio de uma **aresta** (quanto o ponto médio desenhado se afasta do campo) depende só das
//! duas pontas dela, então guarda-se **por aresta** e os dois triângulos donos partilham-no. E a
//! conta que o mede produz **exactamente o ponto que a bissecção vai inserir** — logo medir e
//! partir custam **uma** avaliação do campo, nunca duas.
//!
//! ⭐⭐ **O desvio de um triângulo VIVO nunca muda.** Partir qualquer aresta dele mata-o (ele é um
//! dos dois donos), e nada mais toca nos seus vértices. ⇒ a fila de prioridade não precisa de
//! actualizar entradas: basta saltar as dos triângulos mortos.
//!
//! # O que isto compra, medido — e ⚠️ **o ganho é função da ARTE**
//!
//! Cápsula de `216` peças, orçamento folgado, a mesma tolerância pedida às duas leis:
//!
//! | campo | tol | uniforme | adaptativa | razão |
//! |---|---:|---:|---:|---:|
//! | curva em **todo o lado** | `0,50 px` | `1 944` | `1 025` | `1,9×` |
//! | **junta** (a curvatura numa banda) | `0,50 px` | `13 824` (**não chega**) | `1 220` | **`11,3×`** |
//!
//! ⚠️⚠️ **A primeira medição desta wave mediu a pergunta errada e deu `1,9×`:** a fixtura que
//! existia roda cada ponto por um ângulo proporcional a `x`, logo a arte inteira curva — e sobre um
//! campo assim a lei uniforme está quase óptima. *A vantagem desta lei não é «partir menos»; é
//! partir **onde**.* O produto é o outro caso: numa pele de esqueleto o campo é **rígido** ao longo
//! de cada osso (um afim reproduz um movimento rígido **exactamente**) e vira todo na articulação.
//!
//! ⛔⛔ **E a coluna do desvio diz mais que a das peças: na junta a uniforme NUNCA CHEGA.** Ela
//! satura o orçamento (`k = 8`) e fica em `1,121 px` com a tolerância a pedir `0,25` — logo a razão
//! é um **piso**, e o custo verdadeiro dela é `+∞` dentro do orçamento.
//!
//! As tabelas e as barras: [`crate::refine_tests::adaptativo`].

use crate::refine::fatia;
use crate::{DeformAttrs, Mesh2d, RefineLaw, RefineOptions, RefineReport};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

/// A chave canónica de uma aresta — sempre `(menor, maior)`, para os dois donos concordarem.
fn chave(u: u32, v: u32) -> (u32, u32) {
    if u < v { (u, v) } else { (v, u) }
}

/// O que se sabe do meio de uma aresta: onde ele fica, para onde o campo o leva, e quanto isso
/// difere da corda que o desenho pinta.
///
/// ⛔ **Os atributos NÃO vivem aqui, de propósito.** Guardá-los custaria um `Vec` por aresta
/// (~`1,5 ×` peças, por quadro, por imagem) para um valor que se recalcula por **aritmética pura**
/// no momento do corte — e a recomputação é bit-a-bit a mesma, porque é a mesma expressão sobre as
/// mesmas pontas.
#[derive(Clone, Copy)]
struct Meio {
    rest: [f64; 2],
    posed: [f64; 2],
    desvio: f64,
}

/// ⭐⭐ **TUDO O QUE SE SABE DE UMA ARESTA, NUMA ENTRADA SÓ** — quem a possui e onde é o meio dela.
///
/// ⛔⛔ **As duas metades viviam em DOIS mapas, e a sonda de custo disse porque não podem:** a
/// decisão de *não* refinar custava `0,52 µs` por peça contra `0,096` da lei uniforme (`5,4×`),
/// e o preço era o livro de contas — dois `BTreeMap` (⇒ o dobro dos percursos) e **um `Vec` por
/// aresta** para guardar até dois donos, que é `~1,5 ×` peças alocações por quadro por imagem.
///
/// ⇒ um mapa só, e os donos num par FIXO: *uma aresta de uma malha sã tem no máximo dois donos, e
/// isso não é um palpite — é o que o [`super::refine_tests`] afirma sobre a saída*.
#[derive(Clone, Copy)]
struct Aresta {
    donos: [u32; 2],
    n: u8,
    meio: Option<Meio>,
}

impl Aresta {
    const VAZIA: Self = Self {
        donos: [0; 2],
        n: 0,
        meio: None,
    };

    fn junta(&mut self, t: u32) {
        if let Some(slot) = self.donos.get_mut(self.n as usize) {
            *slot = t;
            self.n += 1;
        }
    }

    /// Tira `t` da lista; devolve `true` se a aresta ficou sem donos.
    fn larga(&mut self, t: u32) -> bool {
        let mut fora = [0u32; 2];
        let mut n = 0u8;
        for i in 0..self.n as usize {
            if self.donos[i] != t
                && let Some(slot) = fora.get_mut(n as usize)
            {
                *slot = self.donos[i];
                n += 1;
            }
        }
        self.donos = fora;
        self.n = n;
        n == 0
    }

    fn outro(&self, t: u32) -> Option<u32> {
        self.donos[..self.n as usize]
            .iter()
            .copied()
            .find(|&o| o != t)
    }
}

/// Um `f64` ordenável — a fila precisa de `Ord` e o desvio é sempre finito e não-negativo.
#[derive(Clone, Copy, PartialEq)]
struct Peso(f64);
impl Eq for Peso {}
impl Ord for Peso {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&o.0)
    }
}
impl PartialOrd for Peso {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

/// `a > b` na ordem TOTAL das arestas — comprimento primeiro, e a chave canónica a desempatar.
///
/// ⚠️⚠️ **O desempate é load-bearing e não é estética:** sem ele, numa grelha (onde toda diagonal
/// mede o mesmo) a cadeia de vizinhos pode fechar um ciclo e o laço não termina.
fn maior(a: (f64, u32, u32), b: (f64, u32, u32)) -> bool {
    a.0 > b.0 || (a.0 == b.0 && (a.1, a.2) > (b.1, b.2))
}

/// A obra em curso: a malha a crescer, quem possui cada aresta, e o que já se mediu.
struct Obra {
    stride: usize,
    rest: Vec<[f64; 2]>,
    attrs: Vec<f64>,
    posed: Vec<[f64; 2]>,
    tris: Vec<[u32; 3]>,
    vivo: Vec<bool>,
    vivos: usize,
    /// aresta → donos vivos + o meio dela, calculado uma vez. **Um** mapa, por medição.
    arestas: BTreeMap<(u32, u32), Aresta>,
    /// `(desvio, triângulo)`, maior desvio primeiro; empates pelo índice menor (determinismo).
    fila: BinaryHeap<(Peso, Reverse<u32>)>,
}

impl Obra {
    fn attr(&self, v: u32, c: usize) -> f64 {
        self.attrs
            .get(v as usize * self.stride + c)
            .copied()
            .unwrap_or(0.0)
    }

    /// Os atributos do meio de `(u,v)` — a MÉDIA das pontas, a mesma expressão que a lei uniforme
    /// usa ([`crate::refine::deviation_attrs`]), para as duas leis medirem o mesmo campo.
    fn attrs_do_meio(&self, e: (u32, u32), out: &mut [f64]) {
        for (c, o) in out.iter_mut().enumerate() {
            *o = f64::midpoint(self.attr(e.0, c), self.attr(e.1, c));
        }
    }

    /// Mede o meio de `(u,v)` se ainda não estiver medido, e devolve-o. ⭐ Esta é a **única**
    /// chamada ao campo por aresta em toda a corrida.
    fn meio(
        &mut self,
        e: (u32, u32),
        deform: &mut DeformAttrs<'_>,
        scratch: &mut Vec<f64>,
    ) -> Option<Meio> {
        // ⚠️ `Meio` é `Copy`, então esta leitura **não** segura empréstimo nenhum — é o que deixa o
        // campo ser chamado a seguir sem clonar o mapa.
        if let Some(m) = self.arestas.get(&e).and_then(|a| a.meio) {
            return Some(m);
        }
        let (a, b) = (self.rest[e.0 as usize], self.rest[e.1 as usize]);
        let rest = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
        scratch.clear();
        scratch.resize(self.stride, 0.0);
        self.attrs_do_meio(e, scratch);
        let posed = deform(rest, scratch);
        let (pa, pb) = (self.posed[e.0 as usize], self.posed[e.1 as usize]);
        let reta = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
        let desvio = (posed[0] - reta[0]).hypot(posed[1] - reta[1]);
        let m = Meio {
            rest,
            posed,
            desvio,
        };
        let a = self.arestas.get_mut(&e)?;
        a.meio = Some(m);
        Some(m)
    }

    /// O pior desvio das três arestas — e ele é FIXO enquanto o triângulo viver (ver o cabeçalho).
    fn desvio_do_tri(
        &mut self,
        t: u32,
        deform: &mut DeformAttrs<'_>,
        scratch: &mut Vec<f64>,
    ) -> f64 {
        let tri = self.tris[t as usize];
        let mut pior = 0.0_f64;
        for k in 0..3 {
            let e = chave(tri[k], tri[(k + 1) % 3]);
            pior = pior.max(self.meio(e, deform, scratch).map_or(0.0, |m| m.desvio));
        }
        pior
    }

    fn ordem(&self, e: (u32, u32)) -> (f64, u32, u32) {
        let (a, b) = (self.rest[e.0 as usize], self.rest[e.1 as usize]);
        ((a[0] - b[0]).hypot(a[1] - b[1]), e.0, e.1)
    }

    fn maior_aresta(&self, t: u32) -> (u32, u32) {
        let tri = self.tris[t as usize];
        let mut melhor = chave(tri[0], tri[1]);
        let mut ord = self.ordem(melhor);
        for k in 1..3 {
            let e = chave(tri[k], tri[(k + 1) % 3]);
            let o = self.ordem(e);
            if maior(o, ord) {
                melhor = e;
                ord = o;
            }
        }
        melhor
    }

    /// O outro dono de `e`; `None` se a aresta é de bordo.
    fn vizinho(&self, t: u32, e: (u32, u32)) -> Option<u32> {
        self.arestas.get(&e)?.outro(t)
    }

    fn regista(&mut self, t: u32) {
        let tri = self.tris[t as usize];
        for k in 0..3 {
            self.arestas
                .entry(chave(tri[k], tri[(k + 1) % 3]))
                .or_insert(Aresta::VAZIA)
                .junta(t);
        }
    }

    fn desregista(&mut self, t: u32) {
        let tri = self.tris[t as usize];
        for k in 0..3 {
            let e = chave(tri[k], tri[(k + 1) % 3]);
            // ⚠️ Uma aresta sem donos nunca mais é lida; largá-la mantém o mapa do tamanho da
            // malha VIVA e não do histórico dela.
            if self.arestas.get_mut(&e).is_some_and(|a| a.larga(t)) {
                self.arestas.remove(&e);
            }
        }
    }

    fn acrescenta(&mut self, tri: [u32; 3], deform: &mut DeformAttrs<'_>, scratch: &mut Vec<f64>) {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o orçamento de peças limita a malha muito antes de 2^32 triângulos"
        )]
        let t = self.tris.len() as u32;
        self.tris.push(tri);
        self.vivo.push(true);
        self.vivos += 1;
        self.regista(t);
        let d = self.desvio_do_tri(t, deform, scratch);
        self.fila.push((Peso(d), Reverse(t)));
    }

    /// ⭐⭐⭐ **PARTE UMA ARESTA — os DOIS donos ao mesmo tempo.** É isto que torna a malha conforme
    /// depois de cada passo, e não só no fim.
    fn parte_aresta(
        &mut self,
        e: (u32, u32),
        deform: &mut DeformAttrs<'_>,
        scratch: &mut Vec<f64>,
    ) {
        let Some(m) = self.meio(e, deform, scratch) else {
            return;
        };
        #[expect(
            clippy::cast_possible_truncation,
            reason = "um vértice por peça acrescentada, e as peças têm tecto"
        )]
        let novo = self.rest.len() as u32;
        self.rest.push(m.rest);
        self.posed.push(m.posed);
        scratch.clear();
        scratch.resize(self.stride, 0.0);
        self.attrs_do_meio(e, scratch);
        self.attrs.extend_from_slice(scratch);
        // ⭐ A aresta é retirada ANTES de partir os donos: ela deixa de existir dos dois lados no
        // mesmo acto, que é exactamente o que impede o nó pendurado.
        let Some(a) = self.arestas.remove(&e) else {
            return;
        };
        for i in 0..a.n as usize {
            self.parte_tri(a.donos[i], e, novo, deform, scratch);
        }
    }

    fn parte_tri(
        &mut self,
        t: u32,
        e: (u32, u32),
        m: u32,
        deform: &mut DeformAttrs<'_>,
        scratch: &mut Vec<f64>,
    ) {
        let tri = self.tris[t as usize];
        let Some(i) = (0..3).find(|&k| chave(tri[k], tri[(k + 1) % 3]) == e) else {
            return;
        };
        // Rodar o triângulo é uma permutação CÍCLICA, logo a orientação sobrevive — e os dois
        // filhos herdam-na.
        let (a, b, c) = (tri[i], tri[(i + 1) % 3], tri[(i + 2) % 3]);
        self.desregista(t);
        self.vivo[t as usize] = false;
        self.vivos -= 1;
        self.acrescenta([a, m, c], deform, scratch);
        self.acrescenta([m, b, c], deform, scratch);
    }

    /// Refina `alvo` até ele deixar de existir, subindo a cadeia de arestas mais longas.
    ///
    /// Devolve `false` quando o ORÇAMENTO parou o trabalho — e a malha fica **conforme na mesma**,
    /// porque cada [`Obra::parte_aresta`] é atómica.
    fn refina(
        &mut self,
        alvo: u32,
        deform: &mut DeformAttrs<'_>,
        scratch: &mut Vec<f64>,
        tecto: usize,
    ) -> bool {
        while self.vivo[alvo as usize] {
            // Cada corte acrescenta no máximo `+1` peça por dono, e uma aresta tem no máximo dois.
            if self.vivos + 2 > tecto {
                return false;
            }
            let mut cur = alvo;
            // ⚠️ A cadeia sobe por arestas ESTRITAMENTE maiores (ordem total), logo não pode
            // repetir um triângulo; este contador é a tradução disso em código, não um palpite.
            let mut passos = 0usize;
            loop {
                passos += 1;
                if passos > self.vivos + 1 {
                    debug_assert!(
                        false,
                        "a cadeia LEPP excedeu a malha viva — ordem nao-total?"
                    );
                    return false;
                }
                let e = self.maior_aresta(cur);
                match self.vizinho(cur, e) {
                    None => {
                        self.parte_aresta(e, deform, scratch);
                        break;
                    }
                    Some(n) if self.maior_aresta(n) == e => {
                        self.parte_aresta(e, deform, scratch);
                        break;
                    }
                    Some(n) => cur = n,
                }
            }
        }
        true
    }
}

/// ⭐⭐⭐ **A MALHA POSADA, refinada SÓ ONDE A DOBRA PEDE.** Ver o cabeçalho do módulo.
///
/// Devolve `(malha, posições, atributos, relatório)`. Quando nada precisa de ser partido a saída é
/// **byte-idêntica** à entrada — mesma ordem de vértices, mesma ordem de triângulos.
#[must_use]
pub fn refine_posed_adaptive(
    mesh: &Mesh2d,
    attrs: &[f64],
    stride: usize,
    deform: &mut DeformAttrs<'_>,
    opts: RefineOptions,
) -> (Mesh2d, Vec<[f64; 2]>, Vec<f64>, RefineReport) {
    let posed: Vec<[f64; 2]> = mesh
        .rest
        .iter()
        .enumerate()
        .map(|(v, &p)| deform(p, fatia(attrs, stride, v)))
        .collect();
    let mut o = Obra {
        stride,
        rest: mesh.rest.clone(),
        attrs: attrs.to_vec(),
        posed,
        tris: Vec::with_capacity(mesh.tris.len()),
        vivo: Vec::with_capacity(mesh.tris.len()),
        vivos: 0,
        arestas: BTreeMap::new(),
        fila: BinaryHeap::with_capacity(mesh.tris.len()),
    };
    let mut scratch: Vec<f64> = Vec::with_capacity(stride);
    for &tri in &mesh.tris {
        o.acrescenta(tri, deform, &mut scratch);
    }

    let tol = opts.tolerance_px.max(f64::MIN_POSITIVE);
    let mut rondas = 0usize;
    let mut travado = false;
    let mut pior = 0.0_f64;
    while let Some((Peso(d), Reverse(t))) = o.fila.pop() {
        if !o.vivo[t as usize] {
            continue;
        }
        // O primeiro VIVO que sai da fila é o pior de todos — ver o cabeçalho (o desvio de um
        // triângulo vivo é imutável).
        pior = d;
        // ⚠️ **O `NaN` é NOMEADO, e não deixado a um `!(d > tol)`:** um campo que devolve `NaN`
        // (uma pose singular) tem de PARAR o laço, e não refinar para sempre a tentar baixar um
        // desvio que não é um número.
        if d.is_nan() || d <= tol {
            break;
        }
        if !o.refina(t, deform, &mut scratch, opts.max_pieces) {
            travado = true;
            break;
        }
        rondas += 1;
    }

    let tris: Vec<[u32; 3]> = o
        .tris
        .iter()
        .zip(&o.vivo)
        .filter_map(|(t, &v)| v.then_some(*t))
        .collect();
    let pecas = tris.len();
    (
        Mesh2d {
            rest: o.rest,
            tris,
            size: mesh.size,
        },
        o.posed,
        o.attrs,
        RefineReport {
            pecas,
            desvio: Some(pior),
            travado_pelo_orcamento: travado,
            lei: RefineLaw::Adaptive { rondas },
        },
    )
}
