#![forbid(unsafe_code)]
//! ⭐⭐ **O CONTACTO ENTRE PEÇAS** — doc 109, ordem do dono (2026-09-13): *«colidem sozinhas»*.
//!
//! Peças que se sobrepõem são afastadas até apenas se tocarem. É a lei do `motion.collide`
//! (restrição de não-penetração do *Position Based Dynamics*, Müller et al. 2007), com o mesmo
//! **Jacobi com média** que o tornou independente da ordem do stream (Macklin & Müller, *Unified
//! Particle Physics*, 2014): cada varredura lê UMA fotografia das posições, cada peça soma o que
//! os contatos dela pedem, e aplica a MÉDIA.
//!
//! ## As peças têm FORMA — disco ou caixa (doc 109 §5)
//!
//! Report do dono no mesmo dia, com foto: *«collider impreciso, o collider não é gerado conforme a
//! forma da Shape»*. Um quadrado declarado como o disco à volta dele deixava `41 %` de ar entre
//! peças que o olho lê como caixas. Um [`Colisor`] é um [`Forma::Disco`] ou uma [`Forma::Caixa`]
//! orientada, com o centro deslocado de `P` quando a arte não está centrada na origem da peça.
//!
//! - **disco × disco** — a lei de sempre, termo a termo.
//! - **caixa × caixa** — o teorema do eixo separador: quatro eixos, e o de MENOR sobreposição dá a
//!   normal e a profundidade. Nada mais — sem variedade de contacto, porque nada aqui roda.
//! - **disco × caixa** — o ponto da caixa mais próximo do centro do disco; com o centro já DENTRO,
//!   a face de menor penetração (a lei do `SHAPE_BOX` do `sim.collide`).
//!
//! ⚠️ **Nada aqui produz rotação.** Uma caixa pousada numa quina fica na quina: a resposta angular
//! é um corpo rígido, e o que este solver projecta são POSIÇÕES (doc 109 §5, aberto).
//!
//! ## A lei do par é calculada na ordem do PAR
//!
//! Cada par resolve-se sempre do índice MENOR para o MAIOR, e o maior recebe a normal simétrica —
//! os dois lados de um contacto são **exactamente** opostos, e dois centros coincidentes separam-se
//! em sentidos contrários sem desempate nenhum a inventar. Para dois discos isto dá os mesmos bits
//! da redacção que calculava de cada lado: `b − a = −(a − b)` e a divisão pelo mesmo `d` são
//! exactas em IEEE-754.
//!
//! ## Porque é uma folha
//!
//! Dois integradores a pedem — o `sim.step` (W2) e o `motion.integrate` (W4) — e o `sim.collide`
//! pergunta-lhe que forma uma peça declarou. Copiar a lei seria a segunda resposta à mesma
//! pergunta; um nó a depender de outro nó quebraria o isolamento.
//!
//! ## A grelha dá os MESMOS BITS que todos-os-pares, e isso é uma escolha
//!
//! O `motion.collide` de CPU é `O(n² · varreduras)`. Aqui cada peça procura parceiros só nas 9
//! células vizinhas de uma grelha de lado `2 · alcance_max`, onde o ALCANCE é o raio, a partir de
//! `P`, do círculo que contém o colisor — duas peças cujos colisores se tocam estão a
//! `alcance_i + alcance_j ≤ 2 · alcance_max`, nunca a mais de uma célula.
//!
//! ⚠️ **E os parceiros são somados em ordem CRESCENTE de índice**, porque é essa a ordem em que o
//! laço de todos-os-pares (`i < j`, `i` por fora) entrega as contribuições a uma peça. ⇒ o gate
//! [`tests`] exige **igualdade ao bit** com a referência, e não uma tolerância que esconderia uma
//! ordem trocada.
//!
//! ## O que conta como peça
//!
//! Um elemento só entra com **colisor válido e posição finita**. Os outros não empurram nem são
//! empurrados — é a declaração ausente vista por dentro. Um peso `w = 0` (o `inv_mass` do
//! `motion.pin_constraint`) é um **obstáculo**: não se move e os outros contornam-no.

use std::collections::BTreeMap;

use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, SIZE_IDENTITY, Stream,
    par_build,
};

mod trig;

/// Abaixo disto dois centros coincidem e a normal não existe (o `EPS` do `motion.collide`).
const EPS: f32 = 1e-9;

/// **A forma de um colisor, já em unidades de MUNDO.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Forma {
    /// Um disco de raio `r`.
    Disco(f32),
    /// Uma caixa orientada: as MEIAS extensões e o eixo `x` dela, `[cos, sin]` unitário.
    Caixa { meia: [f32; 2], eixo: [f32; 2] },
}

/// **O colisor de uma peça**: a forma, e onde o centro dela fica em relação a `P`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colisor {
    pub forma: Forma,
    /// O centro do colisor menos o `P` da peça, em mundo. `[0, 0]` na arte centrada.
    pub desvio: [f32; 2],
}

impl Colisor {
    /// Um disco centrado em `P` — o colisor de antes do doc 109 §5.
    pub const fn disco(r: f32) -> Self {
        Self {
            forma: Forma::Disco(r),
            desvio: [0.0, 0.0],
        }
    }

    /// Uma caixa centrada em `P`, de meias extensões `meia`, com o eixo `x` em `eixo`.
    pub const fn caixa(meia: [f32; 2], eixo: [f32; 2]) -> Self {
        Self {
            forma: Forma::Caixa { meia, eixo },
            desvio: [0.0, 0.0],
        }
    }

    /// O centro do colisor de uma peça em `p`.
    ///
    /// ⚠️ **Sem desvio devolve `p` tal e qual**, e não `p + 0`: `−0 + 0` é `+0`, e o disco centrado
    /// é a lei que já shipou — os mesmos bits, sem excepção de sinal.
    pub fn centro(&self, p: [f32; 2]) -> [f32; 2] {
        if self.desvio == [0.0, 0.0] {
            p
        } else {
            [p[0] + self.desvio[0], p[1] + self.desvio[1]]
        }
    }

    /// **O ALCANCE**: o raio do círculo centrado em `P` que contém o colisor inteiro — o que decide
    /// o lado da grelha.
    pub fn alcance(&self) -> f32 {
        let longe = match self.forma {
            Forma::Disco(r) => r,
            Forma::Caixa { meia, .. } => meia[0].hypot(meia[1]),
        };
        if self.desvio == [0.0, 0.0] {
            longe
        } else {
            longe + self.desvio[0].hypot(self.desvio[1])
        }
    }

    /// **A meia-largura do colisor ao longo da direcção unitária `n`** — o suporte de um convexo
    /// simétrico, que é o que um plano precisa para pousar a peça pela FACE e não pelo centro.
    pub fn suporte(&self, n: [f32; 2]) -> f32 {
        match self.forma {
            Forma::Disco(r) => r,
            Forma::Caixa { meia, eixo } => {
                meia[0] * dot(eixo, n).abs() + meia[1] * dot(perp(eixo), n).abs()
            }
        }
    }

    /// Os quatro cantos de uma caixa com `P` em `p`, no sentido anti-horário a partir de `(+, +)`;
    /// `None` para um disco.
    pub fn cantos(&self, p: [f32; 2]) -> Option<[[f32; 2]; 4]> {
        let Forma::Caixa { meia, eixo } = self.forma else {
            return None;
        };
        let (c, v) = (self.centro(p), perp(eixo));
        let canto = |sx: f32, sy: f32| {
            [
                c[0] + sx * meia[0] * eixo[0] + sy * meia[1] * v[0],
                c[1] + sx * meia[0] * eixo[1] + sy * meia[1] * v[1],
            ]
        };
        Some([
            canto(1.0, 1.0),
            canto(-1.0, 1.0),
            canto(-1.0, -1.0),
            canto(1.0, -1.0),
        ])
    }

    /// Um colisor que ocupa espaço, com números finitos.
    fn valido(&self) -> bool {
        let forma = match self.forma {
            Forma::Disco(r) => r.is_finite() && r > 0.0,
            Forma::Caixa { meia, eixo } => {
                meia.iter().chain(&eixo).all(|x| x.is_finite())
                    && meia[0] >= 0.0
                    && meia[1] >= 0.0
                    && meia[0].max(meia[1]) > 0.0
            }
        };
        forma && self.desvio.iter().all(|x| x.is_finite())
    }
}

fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// O eixo `y` de uma caixa cujo eixo `x` é `e`.
fn perp(e: [f32; 2]) -> [f32; 2] {
    [0.0 - e[1], e[0]]
}

/// O eixo `x` de uma peça girada `graus`, `[cos, sin]` unitário e SEM transcendentais (HR-5).
/// `0°` dá `[1, 0]` ao bit, sem passar pela aproximação.
fn eixo_de(graus: f32) -> [f32; 2] {
    if graus == 0.0 {
        return [1.0, 0.0];
    }
    let (c, s) = trig::cos_sin_cycles(graus / 360.0);
    let inv = 1.0 / (c * c + s * s).sqrt();
    [c * inv, s * inv]
}

/// **O colisor que uma linha DECLARA**, lido dos valores das colunas dela — a porta única que o
/// `sim.step`, o `sim.collide` e o gizmo do cartão da forma perguntam.
///
/// - `caixa` válida (as duas meias finitas, `≥ 0`, uma delas `> 0`) **ganha**: é a declaração de
///   uma forma com `Collider Shape = Box`.
/// - senão `raio` finito e `> 0` é um disco.
/// - senão a peça não colide — e é isto que o `[0, 0]` / `0` preenchido pela união do
///   `motion.combine` quer dizer.
///
/// De geometria para mundo: as meias escalam por `|size|` (um espelho não encolhe), o raio por
/// `max(|sx|, |sy|)` (o disco que contém a arte, a lei do `motion.collide`), e o desvio pelo `size`
/// COM sinal (um espelho leva o centro para o outro lado) e depois pela rotação `graus`.
pub fn declarado(
    raio: Option<f32>,
    caixa: Option<[f32; 2]>,
    desvio: [f32; 2],
    size: [f32; 2],
    graus: f32,
) -> Option<Colisor> {
    let eixo = eixo_de(graus);
    let caixa =
        caixa.filter(|m| m.iter().all(|x| x.is_finite() && *x >= 0.0) && m[0].max(m[1]) > 0.0);
    let forma = match caixa {
        Some(m) => Forma::Caixa {
            meia: [m[0] * size[0].abs(), m[1] * size[1].abs()],
            eixo,
        },
        None => Forma::Disco(
            raio.filter(|r| r.is_finite() && *r > 0.0)? * size[0].abs().max(size[1].abs()),
        ),
    };
    let desvio = if desvio == [0.0, 0.0] {
        [0.0, 0.0]
    } else {
        let d = [desvio[0] * size[0], desvio[1] * size[1]];
        [
            d[0] * eixo[0] - d[1] * eixo[1],
            d[0] * eixo[1] + d[1] * eixo[0],
        ]
    };
    let c = Colisor { forma, desvio };
    c.valido().then_some(c)
}

fn escalares<'a>(s: &'a Stream, nome: &str) -> Option<&'a [f32]> {
    match s.get(nome) {
        Some(Column::Scalar(v)) if v.len() == s.count() => Some(v),
        _ => None,
    }
}

fn pares<'a>(s: &'a Stream, nome: &str) -> Option<&'a [[f32; 2]]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) if v.len() == s.count() => Some(v),
        _ => None,
    }
}

/// **Os colisores de um stream inteiro** — `None` quando ele não declara colisor nenhum (nem a
/// coluna do raio nem a da caixa), que é o que mantém toda cena sem colisor **byte-idêntica**: quem
/// pergunta sai antes de tocar em nada.
pub fn colisores(s: &Stream) -> Option<Vec<Option<Colisor>>> {
    let (raio, caixa) = (escalares(s, COLLIDER_COLUMN), pares(s, COLLIDER_BOX_COLUMN));
    if raio.is_none() && caixa.is_none() {
        return None;
    }
    let (desvio, size, rot) = (
        pares(s, COLLIDER_OFFSET_COLUMN),
        pares(s, "size"),
        escalares(s, "rot"),
    );
    Some(
        (0..s.count())
            .map(|i| {
                declarado(
                    raio.map(|v| v[i]),
                    caixa.map(|v| v[i]),
                    desvio.map_or([0.0, 0.0], |v| v[i]),
                    size.map_or(SIZE_IDENTITY, |v| v[i]),
                    rot.map_or(0.0, |v| v[i]),
                )
            })
            .collect(),
    )
}

/// **O contacto de um par**: a normal unitária de `a` para `b` e a profundidade, ou `None` se não
/// se tocam. `eixo_x` escolhe o eixo de dois centros de disco COINCIDENTES (`true` ⇒ `x`, senão
/// `y`) — o desempate do `motion.collide`, que o solver tira da paridade do par.
pub fn contato(
    a: &Colisor,
    pa: [f32; 2],
    b: &Colisor,
    pb: [f32; 2],
    eixo_x: bool,
) -> Option<([f32; 2], f32)> {
    let (ca, cb) = (a.centro(pa), b.centro(pb));
    match (a.forma, b.forma) {
        (Forma::Disco(ra), Forma::Disco(rb)) => discos(ca, ra, cb, rb, eixo_x),
        (Forma::Caixa { meia: ma, eixo: ea }, Forma::Caixa { meia: mb, eixo: eb }) => {
            caixas(ca, ma, ea, cb, mb, eb)
        }
        // O ponto da caixa mais próximo dá a normal da CAIXA para o DISCO.
        (Forma::Caixa { meia, eixo }, Forma::Disco(r)) => disco_caixa(cb, r, ca, meia, eixo),
        (Forma::Disco(r), Forma::Caixa { meia, eixo }) => {
            disco_caixa(ca, r, cb, meia, eixo).map(|(n, d)| ([-n[0], -n[1]], d))
        }
    }
}

fn discos(ca: [f32; 2], ra: f32, cb: [f32; 2], rb: f32, eixo_x: bool) -> Option<([f32; 2], f32)> {
    let min_dist = ra + rb;
    let min_d2 = min_dist * min_dist;
    let dx = cb[0] - ca[0];
    let dy = cb[1] - ca[1];
    let d2 = dx * dx + dy * dy;
    if d2 >= min_d2 {
        return None;
    }
    Some(if d2 > EPS {
        let d = d2.sqrt();
        ([dx / d, dy / d], min_dist - d)
    } else if eixo_x {
        ([1.0, 0.0], min_dist)
    } else {
        ([0.0, 1.0], min_dist)
    })
}

/// **Duas caixas orientadas** — o eixo separador de menor sobreposição, e a normal a apontar de
/// `a` para `b`. Um centro exactamente sobre o eixo aponta para `+eixo`, e o simétrico do outro lado
/// do par separa os dois.
fn caixas(
    ca: [f32; 2],
    ma: [f32; 2],
    ea: [f32; 2],
    cb: [f32; 2],
    mb: [f32; 2],
    eb: [f32; 2],
) -> Option<([f32; 2], f32)> {
    let d = [cb[0] - ca[0], cb[1] - ca[1]];
    let (va, vb) = (perp(ea), perp(eb));
    let mut melhor: Option<([f32; 2], f32)> = None;
    for eixo in [ea, va, eb, vb] {
        let dist = dot(d, eixo);
        let ra = ma[0] * dot(ea, eixo).abs() + ma[1] * dot(va, eixo).abs();
        let rb = mb[0] * dot(eb, eixo).abs() + mb[1] * dot(vb, eixo).abs();
        let sobra = ra + rb - dist.abs();
        if sobra <= 0.0 {
            return None;
        }
        if melhor.is_none_or(|(_, s)| sobra < s) {
            let n = if dist < 0.0 {
                [-eixo[0], -eixo[1]]
            } else {
                eixo
            };
            melhor = Some((n, sobra));
        }
    }
    melhor
}

/// **Um disco contra uma caixa** — a normal da CAIXA para o DISCO e a profundidade.
///
/// ⚠️ Os dois ramos são geometricamente diferentes e ambos necessários: **fora**, a distância ao
/// ponto mais próximo decide e arredonda as quinas; **dentro**, não há direcção «para fora» única, e
/// sai-se pela face de MENOR penetração (sem este ramo, um disco que atravessou a face num tique
/// grande ficaria preso).
pub fn disco_caixa(
    cd: [f32; 2],
    r: f32,
    cc: [f32; 2],
    meia: [f32; 2],
    eixo: [f32; 2],
) -> Option<([f32; 2], f32)> {
    let v = perp(eixo);
    let d = [cd[0] - cc[0], cd[1] - cc[1]];
    let (lx, ly) = (dot(d, eixo), dot(d, v));
    let (hx, hy) = (meia[0], meia[1]);
    let (qx, qy) = (lx.clamp(-hx, hx), ly.clamp(-hy, hy)); // CLAMP-OK: meias validadas >= 0
    let (ex, ey) = (lx - qx, ly - qy);
    let e2 = ex * ex + ey * ey;
    let mundo = |l: [f32; 2]| [l[0] * eixo[0] + l[1] * v[0], l[0] * eixo[1] + l[1] * v[1]];
    if e2 > EPS {
        let e = e2.sqrt();
        if e >= r {
            return None;
        }
        return Some((mundo([ex / e, ey / e]), r - e));
    }
    let (px, py) = (hx - lx.abs(), hy - ly.abs());
    let nl = if px < py {
        [if lx < 0.0 { -1.0 } else { 1.0 }, 0.0]
    } else {
        [0.0, if ly < 0.0 { -1.0 } else { 1.0 }]
    };
    Some((mundo(nl), px.min(py) + r))
}

/// Afasta as peças sobrepostas, `varreduras` vezes. `p` é reescrito no sítio.
///
/// # Panics
///
/// Se `colisores` ou `pesos` não tiverem o comprimento de `p` — três colunas de uma mesma corrente
/// com comprimentos diferentes não são uma pergunta com resposta.
pub fn separate(
    p: &mut [[f32; 2]],
    colisores: &[Option<Colisor>],
    pesos: &[f32],
    varreduras: usize,
) {
    let n = p.len();
    assert_eq!(colisores.len(), n, "um colisor por peca");
    assert_eq!(pesos.len(), n, "um peso por peca");
    let ativo: Vec<bool> = (0..n).map(|i| ativo(p[i], colisores[i].as_ref())).collect();
    let alcance_max = (0..n)
        .filter(|&i| ativo[i])
        .filter_map(|i| colisores[i].map(|c| c.alcance()))
        .fold(0.0_f32, f32::max);
    if alcance_max <= 0.0 {
        return;
    }
    let lado = 2.0 * alcance_max;
    for _ in 0..varreduras {
        let foto = p.to_vec();
        let grelha = grelha(&foto, &ativo, lado);
        let novas: Vec<Option<[f32; 2]>> = par_build(n, |k| {
            if !ativo[k] {
                return None;
            }
            let (cx, cy) = celula(foto[k], lado);
            let mut parceiros: Vec<usize> = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(v) = grelha.get(&(cx + dx, cy + dy)) {
                        parceiros.extend_from_slice(v);
                    }
                }
            }
            parceiros.sort_unstable();
            corrigida(k, parceiros.into_iter(), &foto, colisores, pesos, &ativo)
        });
        for (k, nova) in novas.into_iter().enumerate() {
            if let Some(q) = nova {
                p[k] = q;
            }
        }
    }
}

/// **A referência**: a mesma lei por todos-os-pares, na ordem do laço `i < j`. `O(n²)`.
///
/// Pública para que o gate de cada cliente possa comparar-se com ela; nenhum caminho de produto a
/// chama.
pub fn separate_all_pairs(
    p: &mut [[f32; 2]],
    colisores: &[Option<Colisor>],
    pesos: &[f32],
    varreduras: usize,
) {
    let n = p.len();
    assert_eq!(colisores.len(), n, "um colisor por peca");
    assert_eq!(pesos.len(), n, "um peso por peca");
    let ativo: Vec<bool> = (0..n).map(|i| ativo(p[i], colisores[i].as_ref())).collect();
    for _ in 0..varreduras {
        let foto = p.to_vec();
        let novas: Vec<Option<[f32; 2]>> = (0..n)
            .map(|k| {
                if ativo[k] {
                    corrigida(k, 0..n, &foto, colisores, pesos, &ativo)
                } else {
                    None
                }
            })
            .collect();
        for (k, nova) in novas.into_iter().enumerate() {
            if let Some(q) = nova {
                p[k] = q;
            }
        }
    }
}

fn ativo(p: [f32; 2], c: Option<&Colisor>) -> bool {
    p[0].is_finite() && p[1].is_finite() && c.is_some_and(Colisor::valido)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "uma celula de grelha; uma coordenada fora de i64 satura, e so' agrupa pior"
)]
fn celula(p: [f32; 2], lado: f32) -> (i64, i64) {
    ((p[0] / lado).floor() as i64, (p[1] / lado).floor() as i64)
}

/// As peças activas por célula, cada lista em ordem crescente de índice.
fn grelha(foto: &[[f32; 2]], ativo: &[bool], lado: f32) -> BTreeMap<(i64, i64), Vec<usize>> {
    let mut g: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    for (i, q) in foto.iter().enumerate() {
        if ativo[i] {
            g.entry(celula(*q, lado)).or_default().push(i);
        }
    }
    g
}

/// A posição de `k` depois desta varredura, ou `None` se nada lhe tocou. Os `parceiros` têm de vir
/// em ordem CRESCENTE — ver o cabeçalho.
fn corrigida(
    k: usize,
    parceiros: impl Iterator<Item = usize>,
    foto: &[[f32; 2]],
    colisores: &[Option<Colisor>],
    pesos: &[f32],
    ativo: &[bool],
) -> Option<[f32; 2]> {
    let mut delta = [0.0_f32; 2];
    let mut contatos = 0_u32;
    for j in parceiros {
        if j == k || !ativo[j] {
            continue;
        }
        // Dois obstáculos (ou dois pesos infinitos) não têm correcção a repartir.
        let soma_w = pesos[k] + pesos[j];
        if soma_w <= 0.0 {
            continue;
        }
        let (lo, hi) = (k.min(j), k.max(j));
        let (Some(clo), Some(chi)) = (colisores[lo], colisores[hi]) else {
            continue;
        };
        // O par na ordem do PAR (ver o cabeçalho): a normal vai do menor para o maior.
        let Some((n, penetracao)) = contato(&clo, foto[lo], &chi, foto[hi], (lo + hi) % 2 == 0)
        else {
            continue;
        };
        // A parte da penetração que cabe a `k`: `w_k / (w_k + w_j)`.
        let empurra = penetracao * (pesos[k] / soma_w);
        if k == lo {
            delta[0] -= n[0] * empurra;
            delta[1] -= n[1] * empurra;
        } else {
            delta[0] += n[0] * empurra;
            delta[1] += n[1] * empurra;
        }
        contatos += 1;
    }
    (contatos > 0).then(|| {
        #[expect(
            clippy::cast_precision_loss,
            reason = "uma contagem de contatos de uma peca, muito abaixo de 2^24"
        )]
        let inv = 1.0 / contatos as f32;
        [foto[k][0] + delta[0] * inv, foto[k][1] + delta[1] * inv]
    })
}

#[cfg(test)]
mod tests;
