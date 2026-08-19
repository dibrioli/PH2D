//! As sondas — o que transforma "parece bom" em número.
//!
//! Cada uma responde a **uma** pergunta do `03_plano_implicito.md` §6, e nenhuma delas olha para a
//! imagem: a imagem é para o Enio julgar caráter, estas são para a linha julgar correção.
//!
//! ⚠️ **Sem `rand`.** O gerador aqui é um LCG escrito à mão e semeado por constante, para que duas
//! corridas do spike deem o **mesmo** conjunto de pontos. Uma sonda que muda de amostra a cada
//! corrida não mede regressão — ela produz ruído com cara de medição (HR-5).

use fidget::context::{Context, Node, Tree};

/// Gerador determinístico. Numerical Recipes, LCG de 32 bits.
pub struct Lcg(u32);

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.0
    }
    /// Uniforme em `[-1, 1)`.
    fn next_pm1(&mut self) -> f64 {
        (self.next_u32() as f64 / u32::MAX as f64) * 2.0 - 1.0
    }
}

/// Avaliador de conveniência: importa a árvore uma vez e responde `f(x,y,z)`.
pub struct Field {
    ctx: Context,
    root: Node,
}

impl Field {
    pub fn new(tree: &Tree) -> Self {
        let mut ctx = Context::new();
        let root = ctx.import(tree);
        Self { ctx, root }
    }

    pub fn at(&self, x: f64, y: f64, z: f64) -> f64 {
        self.ctx.eval_xyz(self.root, x, y, z).unwrap_or(f64::NAN)
    }

    /// Gradiente por diferença central. `eps` tem de ser bem maior que o épsilon de `f32` do
    /// avaliador e bem menor que o raio do filete — 1e-4 satisfaz os dois numa cena de escala ~1.
    pub fn grad(&self, x: f64, y: f64, z: f64, eps: f64) -> [f64; 3] {
        [
            (self.at(x + eps, y, z) - self.at(x - eps, y, z)) / (2.0 * eps),
            (self.at(x, y + eps, z) - self.at(x, y - eps, z)) / (2.0 * eps),
            (self.at(x, y, z + eps) - self.at(x, y, z - eps)) / (2.0 * eps),
        ]
    }
}

/// Quanto o campo se afastou de ser uma **distância**.
///
/// Um campo de distância honesto satisfaz a equação de Eikonal: `‖∇f‖ = 1` em toda parte.
/// É essa propriedade que faz o raio pedido ser o raio entregue, a casca ter a espessura pedida e a
/// marcha de raios não atravessar a superfície (`03_plano_implicito.md` §3.1).
#[derive(Debug, Clone, Copy)]
pub struct Eikonal {
    pub samples: usize,
    pub mean_norm: f64,
    pub min_norm: f64,
    pub max_norm: f64,
    /// O que interessa: o maior desvio de 1, em módulo.
    pub max_deviation: f64,
    /// **Onde** foi o pior desvio, e quanto vale o campo lá.
    ///
    /// ⚠️ Sem isto, um `‖∇f‖ = 1,41` é um susto sem endereço. Com isto dá para responder a única
    /// pergunta que importa: *o utilizador chega a esse ponto?* — um desvio num canto de tampa que
    /// ninguém arredonda não é o mesmo problema que um desvio no filete que o módulo existe para
    /// entregar.
    pub worst_at: [f64; 3],
    pub worst_f: f64,
}

/// Mede `‖∇f‖` numa **banda em volta da superfície** — que é onde a propriedade importa; longe
/// dela ninguém consulta o campo.
pub fn eikonal(field: &Field, band: f64, want: usize, seed: u32) -> Eikonal {
    let mut rng = Lcg::new(seed);
    let (mut n, mut sum, mut lo, mut hi) = (0usize, 0.0, f64::INFINITY, f64::NEG_INFINITY);
    let (mut worst_dev, mut worst_at, mut worst_f) = (0.0f64, [f64::NAN; 3], f64::NAN);
    // Teto de tentativas: a banda pode ser fina, e um laço sem teto trava em vez de reportar.
    let mut tries = 0usize;
    while n < want && tries < want * 400 {
        tries += 1;
        let (x, y, z) = (rng.next_pm1(), rng.next_pm1(), rng.next_pm1());
        if self_abs(field.at(x, y, z)) > band {
            continue;
        }
        let g = field.grad(x, y, z, 1e-4);
        let norm = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        if !norm.is_finite() {
            continue;
        }
        n += 1;
        sum += norm;
        lo = lo.min(norm);
        hi = hi.max(norm);
        let dev = (norm - 1.0).abs();
        if dev > worst_dev {
            worst_dev = dev;
            worst_at = [x, y, z];
            worst_f = field.at(x, y, z);
        }
    }
    let mean = if n > 0 { sum / n as f64 } else { f64::NAN };
    Eikonal {
        samples: n,
        mean_norm: mean,
        min_norm: lo,
        max_norm: hi,
        max_deviation: (1.0 - lo).abs().max((hi - 1.0).abs()),
        worst_at,
        worst_f,
    }
}

fn self_abs(v: f64) -> f64 {
    v.abs()
}

/// **O raio pedido é o raio entregue?** — a sonda analítica, sem malha no meio.
///
/// Monta a união de dois meios-espaços (`x ≤ 0` e `y ≤ 0`). O sólido é tudo menos o quadrante
/// `x>0, y>0`, e a quina em `(0,0)` é **côncava vista do material**. Arredondá-la é um **filete**:
/// ele ACRESCENTA material dentro do quadrante vazio, com o arco tangente às duas faces e centro
/// em `(r, r)`.
///
/// ⚠️ **O centro do arco de um filete fica FORA do sólido** (o de um arredondamento convexo ficaria
/// dentro) — logo `f(r, r, 0)` tem de valer exatamente `+r`, e não `−r`.
/// *A primeira versão desta sonda tinha o sinal trocado e acusou **200 % de erro em todo raio**,
/// que é assinatura de bug de convenção e não de motor: um erro real varia com o raio.*
///
/// Devolve `f(r, r, 0) − r`: zero é perfeito, negativo = filete menor que o pedido.
pub fn radius_error(op: &dyn Fn(&Tree, &Tree) -> Tree, r: f64) -> f64 {
    let a = crate::sdf::sd_half_space_x();
    let b = crate::sdf::sd_half_space_y();
    let field = Field::new(&op(&a, &b));
    field.at(r, r, 0.0) - r
}

/// Erro da malha medido **nos vértices** — a pergunta "a malha está sobre a superfície?".
///
/// ⚠️ **Substitui o baricentro como medida primária.** Um triângulo que atravessa uma quina viva
/// tem os três vértices exatamente sobre a superfície e mesmo assim o **baricentro cai dentro do
/// sólido** — puro fato geométrico de cortar um canto, não defeito de malha. Medir pelo baricentro
/// mistura "a malha errou" com "a quina existe", que são a resposta e a pergunta.
pub fn vertex_error(field: &Field, verts: &[[f32; 3]]) -> SurfaceError {
    let (mut sum, mut worst, mut most_neg) = (0.0, 0.0f64, 0.0f64);
    let mut n = 0usize;
    for v in verts {
        let f = field.at(v[0] as f64, v[1] as f64, v[2] as f64);
        if !f.is_finite() {
            continue;
        }
        n += 1;
        sum += f.abs();
        worst = worst.max(f.abs());
        most_neg = most_neg.min(f);
    }
    SurfaceError {
        mean_abs: if n == 0 { f64::NAN } else { sum / n as f64 },
        max_abs: worst,
        most_negative: most_neg,
    }
}

/// **A quina foi capturada?** Distância do vértice de malha mais próximo a cada canto teórico.
///
/// É o teste direto de preservação de feição: o Dual Contouring Manifold resolve um QEF por célula
/// justamente para pousar o vértice **no canto**. Se ele arredondar, o canto fica órfão e esta
/// distância salta para a ordem do tamanho da célula.
pub fn corner_capture(verts: &[[f32; 3]], corners: &[[f64; 3]]) -> (f64, f64) {
    let mut worst = 0.0f64;
    let mut sum = 0.0f64;
    for c in corners {
        let mut best = f64::INFINITY;
        for v in verts {
            let d = ((v[0] as f64 - c[0]).powi(2)
                + (v[1] as f64 - c[1]).powi(2)
                + (v[2] as f64 - c[2]).powi(2))
            .sqrt();
            best = best.min(d);
        }
        worst = worst.max(best);
        sum += best;
    }
    (sum / corners.len() as f64, worst)
}

/// **O serrilhado da aresta** — o defeito que a imagem mostra e que nenhuma outra sonda pega.
///
/// Fatia a aresta ideal (a reta `x = ex, y = ey`) em faixas de altura `slab` e pergunta, em cada
/// faixa: *qual o vértice de malha mais próximo da reta, medido perpendicularmente?* Se a aresta
/// foi capturada, a resposta é ~0 em toda faixa; um dente-de-serra aparece como uma fração de
/// célula que **não** encolhe ao refinar.
///
/// ⚠️ **Substitui a primeira versão, que estava confundida:** ela filtrava *todos* os vértices a
/// menos de 1,5 célula da reta e tirava a média das distâncias — mas um vértice legitimamente
/// pousado **na face**, a uma célula da aresta, não é um defeito, e entrava na conta. A sonda
/// media a densidade da malha, não o serrilhado.
/// ⚠️ **A JANELA (`slab * 2`) não é detalhe de implementação.** Sem ela, uma fatia que não tenha
/// nenhum vértice perto da aresta devolve o vértice mais próximo *que existir* — e num cubo isso é
/// um vértice da **face oposta**, a 0,9 de distância. A primeira versão fez exatamente isso e
/// reportou um "pior caso" de **115 células**, que não é um serrilhado: é a sonda a medir a
/// diagonal da peça. Fatia sem vértice na janela conta como **falha de captura**, que é a verdade,
/// em vez de virar um número gigante que contamina a média.
pub struct EdgeCapture {
    pub slabs: usize,
    /// Fatias em que existe vértice a menos de ¼ de célula da aresta ideal.
    pub captured: usize,
    /// Fatias sem vértice nenhum dentro da janela.
    pub empty: usize,
    pub mean_cells: f64,
    pub worst_cells: f64,
}

pub fn edge_capture(verts: &[[f32; 3]], ex: f64, ey: f64, z_span: f64, cell: f64) -> EdgeCapture {
    let window = cell * 2.0;
    let n_slabs = ((2.0 * z_span) / cell).ceil().max(1.0) as usize;
    let mut best = vec![f64::INFINITY; n_slabs];
    for v in verts {
        let (x, y, z) = (v[0] as f64, v[1] as f64, v[2] as f64);
        if z.abs() > z_span {
            continue;
        }
        let d = ((x - ex).powi(2) + (y - ey).powi(2)).sqrt();
        if d > window {
            continue;
        }
        let idx = (((z + z_span) / cell) as usize).min(n_slabs - 1);
        best[idx] = best[idx].min(d);
    }
    let valid: Vec<f64> = best.iter().cloned().filter(|d| d.is_finite()).collect();
    let empty = n_slabs - valid.len();
    let captured = valid.iter().filter(|d| **d < cell * 0.25).count();
    let (mean, worst) = if valid.is_empty() {
        (f64::NAN, f64::NAN)
    } else {
        (
            valid.iter().sum::<f64>() / valid.len() as f64 / cell,
            valid.iter().cloned().fold(0.0f64, f64::max) / cell,
        )
    };
    EdgeCapture {
        slabs: n_slabs,
        captured,
        empty,
        mean_cells: mean,
        worst_cells: worst,
    }
}

/// Quão longe a **malha** ficou da superfície verdadeira.
///
/// Avalia o campo no baricentro de cada triângulo: numa malha perfeita todos dariam 0. É a sonda da
/// **quina viva**: se o contorno arredondar um canto, os triângulos de lá caem para dentro do
/// sólido (valor negativo) e aparecem aqui — sem depender de olhar a imagem.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceError {
    pub mean_abs: f64,
    pub max_abs: f64,
    pub most_negative: f64,
}

pub fn surface_error(field: &Field, verts: &[[f32; 3]], tris: &[[u32; 3]]) -> SurfaceError {
    let (mut sum, mut worst, mut most_neg) = (0.0, 0.0f64, 0.0f64);
    for t in tris {
        let mut c = [0.0f64; 3];
        for i in t {
            let v = verts[*i as usize];
            c[0] += v[0] as f64 / 3.0;
            c[1] += v[1] as f64 / 3.0;
            c[2] += v[2] as f64 / 3.0;
        }
        let f = field.at(c[0], c[1], c[2]);
        if !f.is_finite() {
            continue;
        }
        sum += f.abs();
        worst = worst.max(f.abs());
        most_neg = most_neg.min(f);
    }
    SurfaceError {
        mean_abs: if tris.is_empty() {
            f64::NAN
        } else {
            sum / tris.len() as f64
        },
        max_abs: worst,
        most_negative: most_neg,
    }
}
