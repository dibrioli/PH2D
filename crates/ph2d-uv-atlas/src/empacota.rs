//! ⭐⭐⭐ **O EMPACOTADOR POR MÁSCARA** — arruma a FORMA de cada peça, não a caixa dela.
//!
//! # A medição que o encomendou
//!
//! O relatório do atlas publicava `aproveitamento`, que é **caixas / quadrado**, e ele lia
//! `67,7 %`. O que o artista vê é outra coluna — **tinta / quadrado** — e ela lia
//! **`18,7 %`**. ⛔ *Uma régua que mede o invólucro não mede o que está lá dentro.* A
//! decomposição diz onde está a perda:
//!
//! | peça | tinta/quadrado | caixas/quadrado | tinta DENTRO da caixa |
//! |---|---|---|---|
//! | `sculpt_antes` CRUA | `18,7 %` | `67,7 %` | **`27,6 %`** |
//! | `sculpt_antes` F1 | `26,0 %` | `67,7 %` | **`38,5 %`** |
//!
//! ⇒ **`56`–`60 %` do desperdício é DENTRO das caixas**, e nenhum empacotador de
//! rectângulos lhe toca. *Uma ilha esguia e curva num rectângulo é um rectângulo quase
//! vazio, por melhor que os rectângulos se arrumem entre si.*
//!
//! # A lei, e a cerca que a torna segura
//!
//! Cada peça vira uma **máscara de células** e o quadrado vira um **mapa de ocupação**.
//! Uma peça entra onde a máscara dela não bate na ocupação, o mais em baixo e o mais à
//! esquerda possível.
//!
//! ⛔⛔ **A rasterização é CONSERVADORA — uma célula que o triângulo TOCA fica marcada.**
//! É isso, e só isso, que faz a garantia valer em `[0,1]²` e não só na grelha: a cobertura
//! real é um subconjunto das células marcadas, logo duas peças que não partilham uma
//! célula não partilham um ponto. *Uma rasterização por amostra do centro deixaria uma
//! lasca a atravessar a fronteira sem marcar nada.*

/// A silhueta de uma peça em células, já com a folga da costura à volta.
#[derive(Debug, Clone, Default)]
pub struct Mascara {
    /// Largura em células.
    pub larg: usize,
    /// Altura em células.
    pub alt: usize,
    /// `larg × alt`, linha a linha de baixo para cima.
    pub celulas: Vec<bool>,
}

impl Mascara {
    fn cheia(&self, x: usize, y: usize) -> bool {
        self.celulas[y * self.larg + x]
    }

    /// Por coluna, a célula cheia mais baixa — `None` numa coluna vazia.
    fn piso(&self) -> Vec<Option<usize>> {
        (0..self.larg)
            .map(|x| (0..self.alt).find(|&y| self.cheia(x, y)))
            .collect()
    }

    /// Quantas células ela ocupa. ⛔ **Piso de população:** uma máscara vazia caberia em
    /// qualquer sítio e não ocuparia nada — é um defeito, não uma peça pequena.
    #[must_use]
    pub fn ocupadas(&self) -> usize {
        self.celulas.iter().filter(|&&c| c).count()
    }
}

/// O caixote onde as máscaras entram.
///
/// ⭐⭐⭐ **A INVARIANTE que torna o empacotador correcto SEM um teste de colisão:**
/// `topo[c]` é a **marca de água** da coluna `c` — a linha acima da última célula cheia.
/// Logo, por construção, *toda célula em `linha ≥ topo[c]` está vazia*.
struct Ocupacao {
    topo: Vec<usize>,
}

impl Ocupacao {
    fn nova(lado: usize) -> Self {
        Self {
            topo: vec![0; lado],
        }
    }

    fn marca(&mut self, m: &Mascara, x0: usize, y0: usize) {
        for y in 0..m.alt {
            for x in 0..m.larg {
                if m.cheia(x, y) {
                    self.topo[x0 + x] = self.topo[x0 + x].max(y0 + y + 1);
                }
            }
        }
    }
}

/// ⭐⭐⭐ **Arruma as máscaras num quadrado de `lado` células.**
///
/// Devolve o canto inferior-esquerdo de cada uma, ou `None` se alguma não coube.
///
/// ⚠️ A ordem é por **área ocupada**, da maior para a menor: as grandes precisam de espaço
/// contíguo e as pequenas entram nos buracos que elas deixam. *Arrumar as pequenas
/// primeiro enche o chão de confete e as grandes deixam de caber.*
///
/// # ⛔⛔⛔ Porque não há teste de colisão aqui, e porque eu escrevi um
///
/// A 1.ª redacção tinha um `bate()` que verificava a máscara contra a ocupação, e uma
/// **mutação que o apagava SOBREVIVEU**. A razão não é uma fixtura fraca — é uma PROVA:
/// com `y ≥ topo[x+c] − piso[c]` em toda coluna, a célula mais baixa da peça em cada
/// coluna cai **em cima ou acima da marca de água**, e acima dela a coluna está vazia por
/// construção. *O teste nunca podia disparar.*
///
/// ⚠️ **E eu tentei torná-lo necessário antes de o apagar:** deixar as peças pequenas
/// procurarem `24` células ABAIXO do céu, para entrarem debaixo de uma saliência. Medido
/// na escultura do dono, isso compra **`0,0` pontos** (`29,6 %` e `38,7 %`, iguais ao
/// dígito) e custa `4 %` de relógio ⇒ apagado, com o número ao lado. *Uma heurística que
/// não move a coluna que o dono lê é ruído com código à volta.*
#[must_use]
pub fn arruma(mascaras: &[Mascara], lado: usize, folga: usize) -> Option<Vec<(usize, usize)>> {
    let mut ordem: Vec<usize> = (0..mascaras.len()).collect();
    ordem.sort_by_key(|&i| {
        let m = &mascaras[i];
        (std::cmp::Reverse(m.ocupadas()), std::cmp::Reverse(m.alt), i)
    });
    let mut oc = Ocupacao::nova(lado);
    let mut pos = vec![(0usize, 0usize); mascaras.len()];
    for i in ordem {
        let solida = &mascaras[i];
        // ⛔⛔ **Uma máscara VAZIA é recusada e não arrumada.** Ela «cabe» em qualquer
        // sítio e não marca nada, logo tudo o que vier a seguir passa por cima dela —
        // *um piso de população só serve se a porta o IMPUSER*, senão ele é uma nota.
        if solida.ocupadas() == 0 {
            return None;
        }
        let m = com_folga(solida, folga);
        if m.larg > lado || m.alt > lado {
            return None;
        }
        let piso = m.piso();
        // ⭐ Para cada `x`, o «céu» dá o `y` mais baixo em que a peça não bate em nada —
        // ver a prova no cabeçalho. Escolhe-se o mais baixo, e entre iguais o mais à
        // esquerda, que é o `sort` de um par `(y, x)`.
        let mut candidatos: Vec<(usize, usize)> = (0..=(lado - m.larg))
            .map(|x| {
                let y = (0..m.larg)
                    .filter_map(|c| piso[c].map(|p| oc.topo[x + c].saturating_sub(p)))
                    .max()
                    .unwrap_or(0);
                (y, x)
            })
            .collect();
        candidatos.sort_unstable();
        let (y, x) = candidatos
            .iter()
            .find(|&&(y, _)| y + m.alt <= lado)
            .map(|&(y, x)| (y, x))?;
        // ⭐⭐⭐ **Quem PERGUNTA é a máscara com folga; quem MARCA é a SÓLIDA.**
        //
        // ⛔ Engordar os dois lados paga a folga a DOBRO: com cada peça a crescer `g`,
        // entre duas delas ficam `2g` células e a cadeia de mips só pede `g`. Aqui a
        // auréola de A tem de evitar o CORPO de B e o corpo de A tem de evitar a auréola
        // de B ⇒ a separação é exactamente `g`, nos dois sentidos, por construção.
        oc.marca(solida, x + folga, y + folga);
        pos[i] = (x + folga, y + folga);
    }
    Some(pos)
}

/// ⛔ **A rasterização CONSERVADORA de um triângulo:** marca toda célula que ele toca.
///
/// O teste é o dos eixos separadores entre a célula (um rectângulo alinhado) e o
/// triângulo: dois convexos são disjuntos se e só se existe um eixo — as duas normais do
/// rectângulo e as três das arestas do triângulo — em que as projecções não se cruzam.
pub fn marca_triangulo(m: &mut Mascara, t: [[f32; 2]; 3], celula: f32) {
    let f = |v: f32| v / celula;
    let (x0, x1) = (
        f(t[0][0].min(t[1][0]).min(t[2][0])).floor().max(0.0) as usize,
        f(t[0][0].max(t[1][0]).max(t[2][0])).ceil().max(0.0) as usize,
    );
    let (y0, y1) = (
        f(t[0][1].min(t[1][1]).min(t[2][1])).floor().max(0.0) as usize,
        f(t[0][1].max(t[1][1]).max(t[2][1])).ceil().max(0.0) as usize,
    );
    for cy in y0..y1.min(m.alt) {
        for cx in x0..x1.min(m.larg) {
            let lo = [cx as f32 * celula, cy as f32 * celula];
            let hi = [lo[0] + celula, lo[1] + celula];
            if toca(t, lo, hi) {
                m.celulas[cy * m.larg + cx] = true;
            }
        }
    }
}

/// Um triângulo toca o rectângulo `[lo, hi]`?
fn toca(t: [[f32; 2]; 3], lo: [f32; 2], hi: [f32; 2]) -> bool {
    // Os dois eixos do rectângulo.
    for k in 0..2 {
        let (a, b) = (
            t[0][k].min(t[1][k]).min(t[2][k]),
            t[0][k].max(t[1][k]).max(t[2][k]),
        );
        if b < lo[k] || a > hi[k] {
            return false;
        }
    }
    // As três normais das arestas do triângulo.
    let cantos = [lo, [hi[0], lo[1]], hi, [lo[0], hi[1]]];
    for k in 0..3 {
        let (p, q) = (t[k], t[(k + 1) % 3]);
        let n = [-(q[1] - p[1]), q[0] - p[0]];
        let proj = |z: [f32; 2]| z[0].mul_add(n[0], z[1] * n[1]);
        let (mut ta, mut tb) = (f32::MAX, f32::MIN);
        for &z in &t {
            let v = proj(z);
            ta = ta.min(v);
            tb = tb.max(v);
        }
        let (mut ra, mut rb) = (f32::MAX, f32::MIN);
        for &z in &cantos {
            let v = proj(z);
            ra = ra.min(v);
            rb = rb.max(v);
        }
        if tb < ra || rb < ta {
            return false;
        }
    }
    true
}

/// Cresce a máscara `g` células em todas as direcções — é a auréola da FOLGA, e ela é
/// interna ao [`arruma`]: quem chama entrega a máscara SÓLIDA e o `g`, e recebe o canto da
/// SÓLIDA. *Uma porta que aceitasse a máscara já engordada deixaria quem a chama escolher
/// entre pagar a folga uma vez ou duas.*
#[must_use]
pub fn com_folga(m: &Mascara, g: usize) -> Mascara {
    if g == 0 {
        return m.clone();
    }
    let (larg, alt) = (m.larg + 2 * g, m.alt + 2 * g);
    let mut out = Mascara {
        larg,
        alt,
        celulas: vec![false; larg * alt],
    };
    for y in 0..m.alt {
        for x in 0..m.larg {
            if !m.cheia(x, y) {
                continue;
            }
            for dy in 0..=(2 * g) {
                for dx in 0..=(2 * g) {
                    out.celulas[(y + dy) * larg + x + dx] = true;
                }
            }
        }
    }
    out
}
