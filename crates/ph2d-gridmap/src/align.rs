//! ⭐⭐⭐ **O SALTO DA GRADE AO DAR UMA VOLTA** — a régua do espiral.
//!
//! # O que ela mede, e por que NÃO é «o salto de uma costura»
//!
//! O `ACHADO_ordem_das_fases` §23.9 pediu *«quantas células a linha salta ao atravessar
//! cada costura»*. ⛔ **Essa grandeza, isolada, não existe** — e a razão é o
//! [`crate::gauge`]: somar uma constante ao `(u, v)` de um patch muda a translação de
//! **todas** as costuras que lhe tocam sem mudar coisa nenhuma na peça. *Uma coluna por
//! costura mediria a escolha de origem de cada carta.*
//!
//! ⚠️ E há uma segunda razão, mais forte: **depois da soldadura ela é zero por
//! construção.** As duas metades de uma costura satisfazem `z_b = R^k·z_a + t`
//! **exactamente** (é uma substituição, não um acordo), e o G5 põe `t` em inteiros ⇒ as
//! isolinhas inteiras de um lado encontram as do outro, sempre. *A régua que aquela
//! secção pedia só podia imprimir `0`.*
//!
//! # ⭐ A grandeza que sobrevive ao calibre é a HOLONOMIA DE UM CICLO
//!
//! Percorra-se um ciclo do grafo de patches e componham-se as transições. O que volta é
//! um par `(R^K, T)`. ⭐ **Quando `K = 0`, `T` é invariante de calibre** — a mudança de
//! origem entra por `T ↦ T + (I − R^K)·c`, que se anula. E então `T` tem uma leitura
//! directa, que é a do artista:
//!
//! > **dar a volta àquele ciclo desloca a linha de grade em `T` células.**
//!
//! ⛔⛔ **E `T ≠ 0` NÃO é o espiral — a 1.ª redacção desta régua leu-o assim e o padrão
//! desmentiu-a:** no corpus os saltos saíam **`0` ou enormes** (`51`, `75`, `103`,
//! `130`), nunca no meio. Um parafuso de geometria daria um contínuo; uma bimodal
//! daquelas é a régua a medir outra coisa.
//!
//! ⭐⭐⭐ **`T` tem duas metades, e só uma é defeito.** Num tubo cortado ao comprido a
//! transição é `z ↦ z + (N, 0)`: dar a volta **custa** `N` células, e é o comprimento do
//! anel — os anéis fecham na mesma. Num tubo em **parafuso** ela é `(N, 1)`, e o anel
//! que sai de `v = j` volta a `v = j+1`: **não fecha**. *O `N` é inofensivo e grande; o
//! `1` é o defeito e é pequeno* — uma norma `∞` lê o primeiro e enterra o segundo.
//!
//! ⇒ A leitura exacta é **por família**, e não precisa de heurística nenhuma:
//!
//! | componente | o que diz |
//! |---|---|
//! | `a = 0` | as linhas de `u` **fecham** ao dar esta volta |
//! | `b = 0` | as linhas de `v` **fecham** |
//! | ⛔ nenhum `0` | **as duas famílias espiralam** naquela volta |
//!
//! ⚠️ *Uma volta com exactamente um zero é o caso NORMAL* — é o tubo.
//!
//! # ⛔⛔⛔ E a CONTAGEM por ciclo depende da árvore — a grandeza que não depende é o RETICULADO
//!
//! ⚠️ **Contar ciclos que espiralam não é uma medição bem posta**, e o toro prova-o: com
//! a base `{(N,0), (0,M)}` nenhum ciclo espirala, e com a base `{(N,0), (N,M)}` — a
//! mesma peça, outra árvore de expansão — um deles espirala. *A base é minha; a peça
//! não.* E um ciclo «diagonal» com as duas componentes não nulas não é defeito nenhum:
//! ele só diz que **nenhuma linha de grade dá aquela volta**.
//!
//! ⭐⭐⭐ **O que é da peça é o SUBGRUPO `L ⊂ ℤ²` gerado por todas as holonomias planas.**
//! Ele não depende de base nenhuma, e a pergunta do artista lê-se nele directamente:
//!
//! | condição | o que ela quer dizer |
//! |---|---|
//! | `L ∩ ({0}×ℤ) ≠ 0` | **existe** volta que só desloca `v` ⇒ as linhas de `u` podem fechar |
//! | `L ∩ (ℤ×{0}) ≠ 0` | existe volta que só desloca `u` ⇒ as linhas de `v` podem fechar |
//! | ⛔ nenhuma das duas | **nenhuma família fecha em volta nenhuma** — é o parafuso |
//!
//! Um tubo em parafuso gera `L = ℤ·(N,1)`: nenhum múltiplo `k·(N,1)` tem uma componente
//! nula ⇒ as duas intersecções são triviais, e é exactamente por isso que os anéis dele
//! não fecham. Um toro limpo gera `L = Nℤ × Mℤ` ⇒ as duas são não triviais, e as duas
//! famílias fecham. *A mesma conta responde aos dois casos.*
//!
//! ⚠️ **É condição NECESSÁRIA, não suficiente:** `L` diz que a volta existe em ℤ², não
//! que alguma linha de grade a percorre de facto. *Quem mede a realização é o censo de
//! anéis na malha de saída; esta régua mede o que o mapa permite.*
//!
//! ⛔⛔ **E o reticulado NÃO explica o espiral — medido em 8 peças:** o `cube` tem o
//! melhor período possível (`2`) e **zero** anéis fechados; a `wrinkled` tem o pior
//! (`0`) e tem **sete**. *`L` agrega todas as voltas, incluindo as que linha de grade
//! nenhuma percorre* — uma ponte que abre um anel envenena o subgrupo inteiro. Tabela:
//! `ACHADO_ordem_das_fases` §23.11.
//!
//! # ⭐⭐⭐ OS CICLOS QUE RODAM — e o invariante deles é o PONTO FIXO
//!
//! Por peça há `12` a `32` voltas com `K ≠ 0`, e numa esfera são **quase todas**: uma
//! volta que não encerra cone nenhum é rara. Ali `T` é grandeza de calibre e não se lê;
//! o que se lê é o **ponto fixo** de `w ↦ R^K·w + T`:
//!
//! ```text
//!     w* = (I − R^K)⁻¹ · T
//! ```
//!
//! ⭐ **Ele é o sítio onde o cone está, na carta.** E a distância dele à grade inteira é
//! invariante ao calibre — mudar a origem daquela carta desloca `w*` de um **inteiro**.
//!
//! ⛔⛔⛔ **E o denominador é `2`, sempre:** `det(I − R^K)` vale `2` para um quarto de
//! volta e `4` para meia, e as inversas trazem `½` em todas as entradas. ⇒ *um ponto
//! fixo genérico cai em MEIO-INTEIRO* — que é exactamente a meia célula que o
//! [`crate::weld_flat`] já nomeava ao contar `det = 2` nos seus pivôs.
//!
//! ⚠️ Um vértice de valência `3` da malha de saída **é** um vértice de grade: a
//! extracção tem de o pôr num inteiro. Se o mapa o quer a meia célula, alguém encaixa —
//! e um encaixe de meia célula é um rasgo. *Esta coluna é o que verifica, de ponta a
//! ponta, se o `singular_pinned` do G5 de facto chegou lá.*
//!
//! ⚠️ **Ciclos que rodam (`K ≠ 0`) são contados à parte e não entram na estatística:**
//! ali `T` depende do calibre e o que é invariante é o **ponto fixo** — a posição do
//! cone. *Misturá-los daria uma mediana de duas grandezas diferentes.*
//!
//! # ⛔ O que esta régua NÃO é
//!
//! Ela não é a soldadura a perguntar a si própria se soldou. Os fechos que a
//! [`crate::weld_flat`] elimina são ciclos do grafo de **cópias** — a volta à roda de
//! **um vértice**. Os ciclos daqui são do grafo de **patches**, e um deles pode dar a
//! volta à peça inteira. *As duas famílias só coincidem numa superfície onde toda volta
//! seja contraível à volta de um vértice, que não é o caso quando há cones pelo meio.*

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{GridMap, turn2};

/// A que distância de um inteiro se deixa de chamar inteiro a um número, em células.
///
/// ⚠️ **Não é uma folga de conforto:** as translações saem do G5 já pregadas, e o que
/// as separa de um inteiro exacto é o resíduo de `f32` da substituição. *Uma barra
/// muito apertada leria o chão da representação como um defeito do mapa.*
pub const INT_TOL: f32 = 1.0e-3;

/// Uma aresta do grafo de patches: a costura, orientada do lado `0` para o lado `1`.
struct Edge {
    a: usize,
    b: usize,
    /// Os quartos de volta que separam as duas molduras: `z_b = R^k·z_a + t`.
    k: i32,
    t: [f32; 2],
}

/// ⭐⭐⭐ **O que a grade faz ao dar uma volta.**
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Alignment {
    /// Patches no grafo.
    pub patches: usize,
    /// Costuras ao todo.
    pub seams: usize,
    /// ⛔ Costuras sem salto de período lido — não entram em ciclo nenhum.
    pub loose: usize,
    /// Ciclos independentes do grafo de patches (`arestas − patches + componentes`).
    pub cycles: usize,
    /// ⚠️ Os que **rodam** (`K ≠ 0`): há cone lá dentro, e `T` ali é de calibre.
    pub turning_cycles: usize,
    /// ⭐ Os que fecham em rotação — os únicos cujo salto é invariante de calibre.
    pub flat_cycles: usize,
    /// ⭐⭐⭐ Desses, quantos fecham nas **duas** famílias (`a = 0` **e** `b = 0`).
    pub closed_cycles: usize,
    /// ⚠️ Os que fecham numa família só — **o caso normal**, é o que um tubo faz.
    pub one_family_cycles: usize,
    /// ⛔⛔⛔ **OS QUE NÃO FECHAM EM FAMÍLIA NENHUMA — o espiral, em número.**
    pub spiral_cycles: usize,
    /// ⛔ Saltos que nem sequer são inteiros — o mapa não está em grade naquele ciclo.
    pub fractional: usize,
    /// ⭐⭐⭐ **A DERIVA** — `min(|a|, |b|)`, em células: a mediana sobre os ciclos planos.
    ///
    /// ⚠️ É `0` sempre que ao menos uma família fecha. *A metade grande de `T` é o
    /// comprimento da volta e não é defeito nenhum* — ver o doc do módulo.
    pub drift_p50: f32,
    /// O `p90` da deriva.
    pub drift_p90: f32,
    /// A pior deriva.
    pub drift_max: f32,
    /// A soma das derivas — quantas células de espiral a peça carrega ao todo.
    pub drift_sum: f32,
    /// ⭐⭐⭐ **A ORDEM DO RETICULADO `L`** das holonomias planas: `0`, `1` ou `2`.
    ///
    /// `0` = não há volta não trivial nenhuma.
    pub lattice_rank: usize,
    /// ⭐⭐⭐ **O PERÍODO DA FAMÍLIA `u`** — o gerador de `L ∩ ({0}×ℤ)`, em células.
    ///
    /// ⛔ `0` com [`Self::lattice_rank`] `≥ 1` é o defeito: *não existe volta nenhuma
    /// que deixe `u` onde estava*, logo **nenhuma linha de `u` pode fechar**.
    pub u_period: i64,
    /// ⭐⭐⭐ **O PERÍODO DA FAMÍLIA `v`** — o gerador de `L ∩ (ℤ×{0})`, em células.
    pub v_period: i64,
    /// ⭐⭐⭐ **CONES NA GRADE** — voltas que rodam cujo ponto fixo cai num ponto inteiro.
    pub cone_on_lattice: usize,
    /// ⛔⛔ **CONES A MEIA CÉLULA** — o ponto fixo cai em meio-inteiro num eixo ao menos.
    ///
    /// ⚠️ *A extracção tem de os encaixar num inteiro, e meia célula de encaixe é um
    /// rasgo.*
    pub cone_half: usize,
    /// A distância mediana do ponto fixo à grade inteira, em células.
    pub cone_frac_p50: f32,
    /// A pior.
    pub cone_frac_max: f32,
    /// O maior `|T|` (norma `∞`) — o **comprimento** da maior volta, em células.
    ///
    /// ⚠️ Está aqui como **escala**, não como defeito: uma deriva de `2` numa volta de
    /// `100` células e outra numa de `4` não são o mesmo defeito.
    pub span_max: f32,
}

/// A distância de um número ao inteiro mais próximo, em `[0, ½]`.
fn dist_to_int(x: f32) -> f32 {
    (x - x.round()).abs()
}

/// ⭐⭐⭐ **O PONTO FIXO de `w ↦ R^K·w + g`**, para `K ≠ 0` (mod 4).
///
/// ⚠️ **As inversas escrevem-se à mão, e o `½` está à vista de propósito:** é ele o
/// mecanismo do meio-inteiro. *Chamar um solver genérico esconderia exactamente o número
/// que esta função existe para mostrar.*
fn solve_fixed(k: i32, g: [f32; 2]) -> [f32; 2] {
    match k.rem_euclid(4) {
        // `I − R` = [[1,1],[−1,1]], `det = 2`.
        1 => [0.5 * (g[0] - g[1]), 0.5 * (g[0] + g[1])],
        // `I − R²` = `2·I`, `det = 4`.
        2 => [0.5 * g[0], 0.5 * g[1]],
        // `I − R³` = [[1,−1],[1,1]], `det = 2`.
        3 => [0.5 * (g[0] + g[1]), 0.5 * (g[1] - g[0])],
        // ⛔ Sem rotação não há ponto fixo — quem chama já o filtrou.
        _ => [0.0, 0.0],
    }
}

/// O máximo divisor comum, sempre `≥ 0`.
fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Bezout: devolve `(g, s, t)` com `s·a + t·b = g = gcd(a, b)` e `g ≥ 0`.
fn egcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        return if a < 0 { (-a, -1, 0) } else { (a, 1, 0) };
    }
    let (g, s, t) = egcd(b, a % b);
    (g, t, s - (a / b) * t)
}

/// ⭐⭐⭐ **O RETICULADO gerado pelas holonomias**, em forma escalonada: `L = ℤ·(p, q) +
/// ℤ·(0, r)`.
///
/// ⚠️ A forma escalonada não é decoração: é o que torna as duas intersecções com os
/// eixos **contas fechadas** em vez de uma busca.
fn lattice(vs: &[[i64; 2]]) -> (i64, i64, i64) {
    let (mut p, mut q, mut r) = (0i64, 0i64, 0i64);
    for v in vs {
        let (x, y) = (v[0], v[1]);
        if x == 0 {
            r = gcd(r, y);
            continue;
        }
        if p == 0 {
            let sign = if x < 0 { -1 } else { 1 };
            p = x * sign;
            q = y * sign;
            continue;
        }
        let (g, s, t) = egcd(p, x);
        // A combinação que ANULA a primeira coordenada — é ela que alimenta `r`.
        let kill = (p / g) * y - (x / g) * q;
        q = s * q + t * y;
        p = g;
        r = gcd(r, kill);
    }
    if r != 0 {
        q = q.rem_euclid(r);
    }
    (p, q, r)
}

impl Alignment {
    /// A fracção de ciclos planos em que **alguma** família fecha. `1,0` = nenhuma
    /// volta espirala nas duas.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn closed_fraction(&self) -> f32 {
        if self.flat_cycles == 0 {
            return 1.0;
        }
        (self.flat_cycles - self.spiral_cycles) as f32 / self.flat_cycles as f32
    }
}

/// ⭐⭐⭐ **MEDE O SALTO DA GRADE** sobre um mapa vivo.
///
/// ⚠️ **Tem de correr sobre o mapa FINAL** (depois do G5): as translações são o que ela
/// compõe, e o arredondamento move-as. *Medi-la no contínuo seria medir outro mapa.*
#[must_use]
pub fn measure_alignment(cut: &CutMesh, combed: &Combed, map: &GridMap) -> Alignment {
    let np = cut.origin.len();
    let mut out = Alignment {
        patches: np,
        seams: cut.seams.len(),
        ..Alignment::default()
    };

    // ── As arestas: uma por costura com salto lido.
    // ⚠️ Uma costura pode ter o MESMO patch dos dois lados (a ponte que abre um anel).
    // Ela é um LAÇO, e um laço é um ciclo por si só — deixá-la de fora perderia
    // exactamente a volta que a ponte criou.
    let mut edges: Vec<Edge> = Vec::with_capacity(cut.seams.len());
    for (s, seam) in cut.seams.iter().enumerate() {
        let Some(k) = combed.jump.get(s).copied().flatten() else {
            out.loose += 1;
            continue;
        };
        let t = map.shift.get(s).copied().unwrap_or([0.0, 0.0]);
        edges.push(Edge {
            a: seam.side[0].patch as usize,
            b: seam.side[1].patch as usize,
            k,
            t,
        });
    }

    // ── A floresta de expansão. `frame[p] = (K, C)` leva o quadro da raiz ao do patch:
    // `z_p = R^K·z_raiz + C`.
    let mut adj: Vec<Vec<(usize, bool)>> = vec![Vec::new(); np];
    for (i, e) in edges.iter().enumerate() {
        if e.a == e.b || e.a >= np || e.b >= np {
            continue;
        }
        adj[e.a].push((i, true));
        adj[e.b].push((i, false));
    }
    let mut frame: Vec<Option<(i32, [f32; 2])>> = vec![None; np];
    let mut tree = vec![false; edges.len()];
    for root in 0..np {
        if frame[root].is_some() {
            continue;
        }
        frame[root] = Some((0, [0.0, 0.0]));
        let mut queue = std::collections::VecDeque::from([root]);
        while let Some(p) = queue.pop_front() {
            let Some((kp, cp)) = frame[p] else { continue };
            for &(i, forward) in &adj[p] {
                let e = &edges[i];
                let q = if forward { e.b } else { e.a };
                if frame[q].is_some() {
                    continue;
                }
                frame[q] = Some(if forward {
                    let r = turn2(cp, e.k);
                    (kp + e.k, [r[0] + e.t[0], r[1] + e.t[1]])
                } else {
                    let d = [cp[0] - e.t[0], cp[1] - e.t[1]];
                    (kp - e.k, turn2(d, -e.k))
                });
                tree[i] = true;
                queue.push_back(q);
            }
        }
    }

    // ── Um ciclo independente por aresta fora da árvore.
    let mut jumps: Vec<f32> = Vec::new();
    let mut holonomies: Vec<[i64; 2]> = Vec::new();
    let mut cones: Vec<f32> = Vec::new();
    for (i, e) in edges.iter().enumerate() {
        if tree[i] {
            continue;
        }
        let (Some((ka, ca)), Some((kb, cb))) = (
            frame.get(e.a).copied().flatten(),
            frame.get(e.b).copied().flatten(),
        ) else {
            continue;
        };
        out.cycles += 1;
        let dk = (ka + e.k - kb).rem_euclid(4);
        if dk != 0 {
            out.turning_cycles += 1;
            // ⭐ O ponto fixo, na carta do lado `b`: `w* = (I − R^K)⁻¹·(R^k·C_a + t − R^K·C_b)`.
            let r = turn2(ca, e.k);
            let rb = turn2(cb, dk);
            let g = [r[0] + e.t[0] - rb[0], r[1] + e.t[1] - rb[1]];
            let w = solve_fixed(dk, g);
            let f = [dist_to_int(w[0]), dist_to_int(w[1])];
            let d = f[0].max(f[1]);
            if d <= INT_TOL {
                out.cone_on_lattice += 1;
            } else if (d - 0.5).abs() <= INT_TOL {
                out.cone_half += 1;
            }
            cones.push(d);
            continue;
        }
        out.flat_cycles += 1;
        // O mesmo ponto, lido pelos dois caminhos: pela árvore e por esta costura.
        let r = turn2(ca, e.k);
        let d = [r[0] + e.t[0] - cb[0], r[1] + e.t[1] - cb[1]];
        if d.iter().any(|x| (x - x.round()).abs() > INT_TOL) {
            out.fractional += 1;
        }
        // ⭐ Cada componente responde por UMA família: `a` pelas linhas de `u`, `b`
        // pelas de `v`. *A conta é por família porque a pergunta é por família.*
        let closed = [d[0].abs() <= INT_TOL, d[1].abs() <= INT_TOL];
        match usize::from(closed[0]) + usize::from(closed[1]) {
            2 => out.closed_cycles += 1,
            1 => out.one_family_cycles += 1,
            _ => out.spiral_cycles += 1,
        }
        let drift = d[0].abs().min(d[1].abs());
        out.span_max = out.span_max.max(d[0].abs().max(d[1].abs()));
        out.drift_sum += drift;
        jumps.push(drift);
        #[allow(clippy::cast_possible_truncation)]
        holonomies.push([d[0].round() as i64, d[1].round() as i64]);
    }
    cones.sort_by(f32::total_cmp);
    out.cone_frac_p50 = cones.get(cones.len() / 2).copied().unwrap_or(0.0);
    out.cone_frac_max = cones.last().copied().unwrap_or(0.0);
    // ⭐⭐⭐ O reticulado — a leitura que não depende da árvore que escolhi.
    {
        let (p, q, r) = lattice(&holonomies);
        out.lattice_rank = usize::from(p != 0) + usize::from(r != 0);
        out.u_period = r;
        out.v_period = if r != 0 {
            p * (r / gcd(q, r))
        } else if q == 0 {
            p
        } else {
            0
        };
    }
    jumps.sort_by(f32::total_cmp);
    out.drift_p50 = jumps.get(jumps.len() / 2).copied().unwrap_or(0.0);
    out.drift_p90 = jumps.get(jumps.len() * 9 / 10).copied().unwrap_or(0.0);
    out.drift_max = jumps.last().copied().unwrap_or(0.0);
    out
}

/// ⭐⭐⭐ **O QUE O F4 EXIGE CONTRA O QUE O MAPA FEZ.**
///
/// # Por que ela existe
///
/// A rota da extracção **não chama o F4** (`ACHADO_ordem_das_fases` §23.13): o mapa
/// resolve livre e a contagem de cada arco é o que calhar. O F4 decide, com óptimo
/// demonstrado, **quantas arestas de quad leva cada arco** — e o A/B mediu que ligá-lo
/// melhora as voltas nas três peças.
///
/// ⚠️ **Antes de construir a restrição, mede-se a discordância.** *Uma restrição que o
/// mapa já satisfaz não muda nada, e teria custado uma wave a descobri-lo.*
///
/// # As duas metades, e a segunda é a que ninguém olhou
///
/// No plano, um arco vai do canto `A` ao canto `B`, e o deslocamento `z(B) − z(A)` diz
/// duas coisas ao mesmo tempo:
///
/// | metade | o que é | o que devia ser |
/// |---|---|---|
/// | **ao longo** | o maior componente | ⭐ o número que o F4 pede |
/// | **atravessado** | o menor | ⛔ **zero** — senão o arco não é uma isolinha |
///
/// ⚠️ *A segunda não depende do F4 nenhum:* uma separatriz que atravessa `k` células na
/// direcção transversal **não é uma linha de grade**, e nenhuma régua desta cadeia a
/// media.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ArcQuant {
    /// Costuras com arco e com contagem pedida — as que entraram.
    pub arcs: usize,
    /// ⛔ Costuras SEM arco (cortes que o G1 abriu) — o F4 não tem número para elas.
    pub cut_only: usize,
    /// ⭐ Arcos em que o mapa já dá o que o F4 pede.
    pub agree: usize,
    /// A discordância mediana, em arestas de quad.
    pub diff_p50: f32,
    /// A pior.
    pub diff_max: f32,
    /// A soma das discordâncias — quantas arestas de quad a peça inteira desloca.
    pub diff_sum: f32,
    /// ⛔⛔ Arcos cujo componente **atravessado** não é zero: eles não são isolinhas.
    pub off_axis: usize,
    /// A travessia mediana, em células.
    pub across_p50: f32,
    /// A pior.
    pub across_max: f32,
}

impl ArcQuant {
    /// A fracção de arcos em que o mapa já concorda com o F4.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn agree_fraction(&self) -> f32 {
        if self.arcs == 0 {
            return 1.0;
        }
        self.agree as f32 / self.arcs as f32
    }
}

/// ⭐⭐⭐ **MEDE A DISCORDÂNCIA ENTRE O MAPA E O F4.**
///
/// `demand[a]` é quantas arestas de quad o F4 dá ao arco `a` — a mesma indexação de
/// `PatchLayout`, que é a que a [`crate::cut::Seam::arc`] guarda.
///
/// ⚠️ **Ela recebe um `&[u32]` e não o tipo do F4 de propósito:** *a régua pergunta «o
/// que se exige deste arco», e quem responde não é problema dela* — assim a
/// `ph2d-gridmap` não ganha uma dependência para uma sonda.
#[must_use]
pub fn measure_arc_quantization(cut: &CutMesh, map: &GridMap, demand: &[u32]) -> ArcQuant {
    let mut out = ArcQuant::default();
    let mut diffs: Vec<f32> = Vec::new();
    let mut across: Vec<f32> = Vec::new();
    for seam in &cut.seams {
        let Some(arc) = seam.arc else {
            out.cut_only += 1;
            continue;
        };
        let Some(&want) = demand.get(arc as usize) else {
            out.cut_only += 1;
            continue;
        };
        // ⚠️ Os extremos são o primeiro e o último local **presentes**: uma posição
        // `None` é um vértice que nenhuma face daquele lado alcançou, e tratá-la como
        // zero poria o canto na origem da carta.
        let side = &seam.side[0];
        let p = side.patch as usize;
        let Some(first) = side.local.iter().flatten().next() else {
            continue;
        };
        let Some(last) = side.local.iter().flatten().next_back() else {
            continue;
        };
        let Some(row) = map.uv.get(p) else { continue };
        let (Some(za), Some(zb)) = (row.get(*first as usize), row.get(*last as usize)) else {
            continue;
        };
        let d = [zb[0] - za[0], zb[1] - za[1]];
        let along = d[0].abs().max(d[1].abs());
        let cross = d[0].abs().min(d[1].abs());
        out.arcs += 1;
        #[allow(clippy::cast_precision_loss)]
        let diff = (along - want as f32).abs();
        if diff <= INT_TOL {
            out.agree += 1;
        }
        if cross > INT_TOL {
            out.off_axis += 1;
        }
        out.diff_sum += diff;
        diffs.push(diff);
        across.push(cross);
    }
    diffs.sort_by(f32::total_cmp);
    across.sort_by(f32::total_cmp);
    out.diff_p50 = diffs.get(diffs.len() / 2).copied().unwrap_or(0.0);
    out.diff_max = diffs.last().copied().unwrap_or(0.0);
    out.across_p50 = across.get(across.len() / 2).copied().unwrap_or(0.0);
    out.across_max = across.last().copied().unwrap_or(0.0);
    out
}

#[cfg(test)]
#[path = "align_tests.rs"]
mod tests;
