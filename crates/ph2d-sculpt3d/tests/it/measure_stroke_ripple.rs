//! **DE QUE É FEITO O DENTE DE SERRA** — a sonda que mede a ondulação de um
//! traço ao longo do CAMINHO, e a atribui a um mecanismo.
//!
//! # A pergunta
//!
//! O report é de **dentes de serra ao longo do traço**: cada carimbo deixa a
//! própria protuberância e há vale entre eles. Isso é uma afirmação sobre a
//! grandeza `h(s)` — a altura depositada como função da posição AO LONGO do
//! caminho — e sobre **de que ela depende**. Esta sonda mede as seis metades que
//! têm curas DIFERENTES:
//!
//! 1. a **ONDULAÇÃO** de `h(s)` no miolo do traço, contra a altura média;
//! 2. a dependência do **ESPAÇAMENTO** — e a pergunta que decide a lei é se a
//!    **altura média** depende dele. Se depender, o depósito é uma **soma de
//!    carimbos** e não uma integral de caminho;
//! 3. a dependência da **TAXA DE EVENTOS** — o mesmo caminho em poucos eventos
//!    longos contra muitos curtos;
//! 4. a **AUTO-AMPLIFICAÇÃO** — o que N dabs no mesmo lugar fazem;
//! 5. o **DEGRAU** entre vértices vizinhos dentro da pegada, por dureza;
//! 6. quanto o **AUTOSMOOTH** esconde de cada um.
//!
//! ⚠️ **Ela é irmã da [`measure_hardness_and_falloff`], não uma segunda cópia
//! dela.** Aquela mede a superfície ATRAVESSADA pelo traço (o diedro, o
//! guarda-chuva, o perfil RADIAL) e atribuiu o report das *escamas*; esta mede a
//! grandeza AO LONGO do traço, que é a única que pode dizer se o defeito tem o
//! período do espaçamento — e o `docs/3D/BUGS_sculpt3d.md` #3 registra o preço
//! de medir a coisa certa no lugar errado.
//!
//! # A porta é a do PRODUTO
//!
//! Os dabs entram por `SculptStroke::begin` + `SculptStroke::dab`, que é o que o
//! shell chama (`sculpt3d_input.rs`), e o centro de cada um sai de um **pick**,
//! que é o que o `sculpt_at` faz a cada passo do `walk`. A metade 3 dirige o
//! [`walk`] REAL e pergunta a [`ph2d_sculpt3d::Walk::anchor`], sem uma linha de
//! re-expressão.
//!
//! ⚠️ **A régua do espaçamento é uma FRAÇÃO do raio nos dois mundos.** O shell
//! caminha em pixels de tela (`min_spacing(scene.radius_px())`) e nós em
//! unidades de mundo; o que atravessa a fronteira é `spacing / radius = 0,15`,
//! que é idêntico. Dirigir em mundo é fiel e tira a câmera da conta.
//!
//! Rodar:
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_stroke_ripple \
//!     -- --ignored --nocapture
//! ```

use ph2d_mesh::{Mesh, Ray, shapes::sculpt_sphere};
use ph2d_sculpt3d::{
    Brush, Dab, Falloff, MIN_SPACING_FRACTION, RefMode, SculptStroke, Symmetry, Verb, min_spacing,
    walk,
};

// ─────────────────────────────────────────────────────────────────────────────
// A FIXTURE
// ─────────────────────────────────────────────────────────────────────────────

/// O raio e a força do traço — os do produto, não números escolhidos aqui.
/// (`Brush::default()` ship `radius: 0,25` e `strength: 0,5`; o raio sobe para
/// `0,30` pelo mesmo motivo da sonda irmã: uma pegada maior põe mais vértices
/// sob cada dab e torna o perfil legível.)
const RADIUS: f32 = 0.30;
const STRENGTH: f32 = 0.5;

/// O arco do traço, em radianos de longitude no plano `y = 0`.
///
/// ⚠️ **Um ARCO e não uma reta, porque a superfície é uma esfera** — o caminho
/// tem de correr SOBRE ela. O span é de 1,2 rad ≈ 1,2 unidade de mundo, o que a
/// `min_spacing(0,30) = 0,045` dá **~26 dabs**: o bastante para o miolo conter
/// mais de dez períodos da ondulação que a metade 1 procura.
const ARC_FROM: f64 = -0.90;
const ARC_TO: f64 = 0.90;

/// Que fração do arco é o **MIOLO** — o resto é a rampa de entrada e de saída,
/// onde a altura sobe do zero e não descreve o regime.
///
/// ⚠️ **DERIVADA do raio do pincel, e a 1ª versão a escolheu (`0,5`) e ERROU.**
/// A cauda de um traço é longa: o último dab ainda alcança **um raio inteiro**
/// à frente, então tudo dentro de `1,5 · raio` de uma ponta recebe menos dabs
/// que o regime — não por defeito, mas porque *o traço acabou ali*. Com um
/// miolo escolhido a olho essa cauda entrava na conta e aparecia como uma
/// «tendência» de dezenas por cento; medido no controle de geometria, o perfil
/// era **plano em 102-105 e só caía no FIM**, e **espelhava** quando o traço era
/// percorrido ao contrário. *Uma janela que contém a rampa mede a rampa.*
const CORE: f64 = 1.0 - 3.0 * RADIUS as f64 / (ARC_TO - ARC_FROM);

/// A largura da banda em torno do eixo do traço, em **ARESTAS DA MALHA**.
///
/// ⚠️ **Ela é medida na malha e não no raio, e a 1ª versão desta sonda pagou por
/// isso.** Com `banda = 0,10 · raio` a malha de fábrica punha **um** vértice em
/// alguns bins, e o pico-a-pico passou a medir *qual vértice calhou de cair ali*
/// em vez do depósito: a razão saía **56 % · 83 % · 25 % · 156 %** ao longo de
/// uma varredura monotônica de espaçamento. *Um número que não reproduz não é
/// achado, é ruído com casas decimais.*
///
/// ⚠️ **E amarrá-la à ARESTA é o que torna as três densidades comparáveis:** o
/// bin também mede uma aresta, então os vértices por bin ficam ~constantes
/// enquanto o vazamento TRANSVERSAL encolhe com a malha — que é exatamente o
/// que a §0b mede em vez de eu afirmar.
const BAND_EDGES: f64 = 4.0;

/// **O CENTRO DO DAB, COMO O PRODUTO O ACHA** — um raio de fora para dentro.
///
/// ⚠️ **Ele re-pica a cada passo, e isso não é detalhe de fixture — é o
/// auto-limite.** Cada passo do `walk` no shell chama `sculpt_at(sx, sy)`, que é
/// um pick; com o centro pregado na superfície de PARTIDA ele fica ENTERRADO
/// sob o barro que sobe, e o pincel deixa de se esgotar. A metade 4 mede os dois
/// mundos lado a lado exatamente porque a diferença entre eles é grande.
fn pick(mesh: &Mesh, dir: [f32; 3]) -> Option<[f32; 3]> {
    let origin = [dir[0] * 3.0, dir[1] * 3.0, dir[2] * 3.0];
    let ray = Ray::new(origin, [-dir[0], -dir[1], -dir[2]]);
    mesh.raycast(&ray).map(|h| h.point)
}

/// A direção do raio para o parâmetro `t ∈ [0, 1]` do arco.
fn ray_dir(t: f64) -> [f32; 3] {
    let a = ARC_FROM + t * (ARC_TO - ARC_FROM);
    [a.cos() as f32, 0.0, a.sin() as f32]
}

/// O **pincel de FÁBRICA** de um verbo — o modo em que ele nasce, a curva que
/// esse modo declara e o `accumulate` do perfil dele.
///
/// ⚠️ **Nada aqui é escolhido por mim.** Escrever `falloff: Falloff::Plateau` à
/// mão mediria um pincel que o artista não recebe — e o `docs/3D/BUGS_sculpt3d.md`
/// #4 é literalmente o defeito de sete verbos nascerem num modo que não os
/// declara. As três portas (`birth_for`, `default_falloff`, `default_accumulate`)
/// são as que a shell e o painel perguntam.
fn factory(verb: Verb) -> Brush {
    let mode = RefMode::birth_for(verb);
    Brush {
        verb,
        mode,
        falloff: verb.default_falloff(mode),
        accumulate: verb.default_accumulate(),
        radius: RADIUS,
        strength: STRENGTH,
        ..Brush::default()
    }
}

/// As malhas da medição.
///
/// ⚠️ **A última NÃO é a do produto, e é o CONTROLE DE GEOMETRIA.** A esfera de
/// fábrica é um cubo subdividido: o doc dela mede o raio a variar **3,09 %** e a
/// razão entre a maior e a menor aresta em **3,9×**, então *o arco atravessa
/// geometria que muda*. Uma `uv_sphere` tem raio EXACTAMENTE constante e, ao
/// longo do equador, espaçamento uniforme — ela é péssima para esculpir (o doc
/// do `sculpt_sphere` mede 30,6× de razão de aresta nos polos) e é exactamente
/// por ser uniforme ONDE O TRAÇO PASSA que ela responde *«a tendência é da lei
/// de depósito ou da malha?»* sem que eu tenha de adivinhar.
fn mesh_at(label: &str) -> Mesh {
    if let Some(n) = label.strip_prefix('+') {
        let mut m = sculpt_sphere(1.0);
        for _ in 0..n.parse::<usize>().expect("subdivisões") {
            m = ph2d_mesh::subdivide(&m);
            m.rebuild();
        }
        return m;
    }
    // 512 segmentos no equador dão a MESMA aresta que 256 anéis: no equador,
    // onde o traço corre, ela é isotrópica.
    let mut m = ph2d_mesh::shapes::uv_sphere(256, 512, 1.0);
    m.rebuild();
    m
}

/// O comprimento MÉDIO de aresta — a régua que decide se a malha resolve a
/// ondulação.
fn mean_edge(mesh: &Mesh) -> f64 {
    let adj = mesh.adjacency();
    let pos = mesh.positions();
    let (mut sum, mut n) = (0.0f64, 0u64);
    for i in 0..mesh.vert_count() {
        let a = pos[i];
        for &j in adj.vert_verts.neighbours(i) {
            let b = pos[j as usize];
            sum += f64::from(
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
            );
            n += 1;
        }
    }
    sum / n as f64
}

// ─────────────────────────────────────────────────────────────────────────────
// O QUE UM TRAÇO DEIXA
// ─────────────────────────────────────────────────────────────────────────────

/// O estado de ANTES, para o depósito ser medido contra ele.
struct Before {
    pos: Vec<[f32; 3]>,
    nrm: Vec<[f32; 3]>,
}

impl Before {
    fn of(mesh: &Mesh) -> Self {
        Self {
            pos: mesh.positions().to_vec(),
            nrm: mesh.normals().to_vec(),
        }
    }
}

/// **A ALTURA de um vértice** — o deslocamento projetado na normal CONGELADA.
///
/// ⚠️ **Nem `|p| − 1` nem `|Δp|`, e as duas alternativas estão erradas por
/// motivos diferentes.** A [`sculpt_sphere`] **não é uma esfera** (ela é o
/// limite de Catmull-Clark de um cubo, e o doc dela mede o raio a variar
/// **3,09 %**), então `|p| − 1` leria a forma da própria malha como ondulação —
/// ordens de grandeza acima do que se procura. E `|Δp|` não tem SINAL, o que
/// tornaria o Crease indistinguível do Draw.
fn height(b: &Before, mesh: &Mesh, v: usize) -> f64 {
    let (p, q, n) = (mesh.positions()[v], b.pos[v], b.nrm[v]);
    f64::from((p[0] - q[0]) * n[0] + (p[1] - q[1]) * n[1] + (p[2] - q[2]) * n[2])
}

/// O que a medição de um perfil devolve.
///
/// ⚠️ **As DUAS grandezas são separadas porque têm curas OPOSTAS, e a 1ª versão
/// desta sonda as somava num número só.** Um pico-a-pico cru sobre o miolo
/// inteiro não distingue *o cume fica mais alto ao longo do traço* (uma RAMPA)
/// de *cada carimbo deixa a própria protuberância* (o DENTE DE SERRA), e o
/// resultado era irreprodutível entre densidades — **56 % · 14 % · 52 %** para o
/// MESMO traço. O perfil passa por uma média móvel de exactamente UM período de
/// espaçamento: o que ela devolve é a [`Profile::trend`], o que sobra dela é a
/// [`Profile::ripple`], e a §0b prova que os dois se separam.
struct Profile {
    /// A altura média no MIOLO.
    mean: f64,
    /// **O DENTE DE SERRA** — o pico-a-pico do que sobra depois de a média
    /// móvel de um período ser removida.
    ripple: f64,
    /// **A RAMPA** — o pico-a-pico da própria média móvel.
    trend: f64,
    /// Quantos bins do miolo tinham vértices — a saúde da fixture, ao lado do
    /// [`Profile::min_per_bin`].
    #[allow(dead_code)]
    bins: usize,
    /// O menor número de vértices num bin do miolo — a saúde da fixture.
    min_per_bin: usize,
    /// A média de vértices por bin do miolo.
    mean_per_bin: f64,
    /// A média móvel ao longo do MIOLO — a forma da tendência, para ela poder
    /// ser OLHADA em vez de inferida de um pico-a-pico.
    core_smooth: Vec<f64>,
}

impl Profile {
    /// O DENTE DE SERRA em fração da altura média — a grandeza livre de escala,
    /// e a única comparável entre linhas de uma tabela cujas alturas diferem.
    fn ratio(&self) -> f64 {
        self.frac(self.ripple)
    }

    /// A RAMPA em fração da altura média.
    fn trend_ratio(&self) -> f64 {
        self.frac(self.trend)
    }

    fn frac(&self, v: f64) -> f64 {
        if self.mean.abs() < 1e-12 {
            f64::NAN
        } else {
            v / self.mean.abs()
        }
    }
}

/// **O PERFIL AO LONGO DO TRAÇO** — a altura média por bin de longitude, dentro
/// de uma banda estreita em torno do eixo.
///
/// `bin_arc` é a largura de um bin em unidades de MUNDO; ela vem do comprimento
/// de aresta da malha, e é impressa, porque um perfil binado mais grosso que o
/// espaçamento não pode ver a ondulação que tem o período dele.
fn profile(b: &Before, mesh: &Mesh, bin_arc: f64, radius_world: f64, period_world: f64) -> Profile {
    let band = (bin_arc * BAND_EDGES) as f32;
    let span = ARC_TO - ARC_FROM;
    let bin_theta = bin_arc / radius_world;
    let nbins = ((span / bin_theta).round() as usize).max(4);
    let mut sum = vec![0.0f64; nbins];
    let mut cnt = vec![0usize; nbins];

    for v in 0..mesh.vert_count() {
        let p0 = b.pos[v];
        if p0[1].abs() > band {
            continue;
        }
        let theta = f64::from(p0[2]).atan2(f64::from(p0[0]));
        let u = (theta - ARC_FROM) / span;
        if !(0.0..1.0).contains(&u) {
            continue;
        }
        let k = ((u * nbins as f64) as usize).min(nbins - 1);
        sum[k] += height(b, mesh, v);
        cnt[k] += 1;
    }

    let val: Vec<Option<f64>> = (0..nbins)
        .map(|k| (cnt[k] > 0).then(|| sum[k] / cnt[k] as f64))
        .collect();

    // ⚠️ **A JANELA É UM PERÍODO DE ESPAÇAMENTO, e o número não é escolhido:**
    // uma média móvel retangular de exactamente um período ANIQUILA aquela
    // frequência, então o que ela devolve é a tendência e o que sobra é o dente
    // de serra. A §0b prova as duas metades com campos que eu escrevi.
    let w = ((period_world / bin_arc).round() as usize).max(3) | 1;
    let half = w / 2;

    // O MIOLO: a fração central do arco. As pontas são a rampa do traço.
    let lo = ((1.0 - CORE) * 0.5 * nbins as f64) as usize;
    let hi = nbins - lo;
    let mut core_smooth = Vec::new();
    let (mut rn, mut rx) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut tn, mut tx) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut acc, mut bins, mut min_per_bin, mut verts) = (0.0f64, 0usize, usize::MAX, 0usize);
    for k in lo..hi {
        let Some(h) = val[k] else { continue };
        // A média móvel centrada, encolhendo nas pontas em vez de inventar
        // valores fora do domínio.
        let (mut s, mut n) = (0.0f64, 0usize);
        for x in val[k.saturating_sub(half)..(k + half + 1).min(nbins)]
            .iter()
            .flatten()
        {
            s += *x;
            n += 1;
        }
        let smooth = s / n as f64;
        core_smooth.push(smooth);
        let resid = h - smooth;
        rn = rn.min(resid);
        rx = rx.max(resid);
        tn = tn.min(smooth);
        tx = tx.max(smooth);
        acc += h;
        bins += 1;
        verts += cnt[k];
        min_per_bin = min_per_bin.min(cnt[k]);
    }
    let none = bins == 0;
    Profile {
        mean: if none { 0.0 } else { acc / bins as f64 },
        ripple: if none { 0.0 } else { rx - rn },
        trend: if none { 0.0 } else { tx - tn },
        bins,
        min_per_bin: if min_per_bin == usize::MAX {
            0
        } else {
            min_per_bin
        },
        mean_per_bin: if none {
            0.0
        } else {
            verts as f64 / bins as f64
        },
        core_smooth,
    }
}

/// Carimba o arco com o espaçamento dado, re-picando a cada passo, e devolve
/// quantos dabs caíram.
fn stroke_arc(mesh: &mut Mesh, brush: &Brush, spacing: f64, radius_world: f64) -> usize {
    stroke_arc_dir(mesh, brush, spacing, radius_world, false)
}

/// O mesmo traço, com a opção de o percorrer ao CONTRÁRIO.
///
/// ⚠️ **É o controle que separa as duas explicações da tendência**, e nenhuma
/// medição sobre um traço só as distingue: se o perfil ACOMPANHA o sentido do
/// gesto, a tendência é da LEI DE DEPÓSITO; se ele fica ancorado no mesmo sítio
/// da malha, é da GEOMETRIA (a esfera de fábrica é um cubo subdividido — o raio
/// varia 2,2 % ao longo do arco e a aresta varia bem mais).
fn stroke_arc_dir(
    mesh: &mut Mesh,
    brush: &Brush,
    spacing: f64,
    radius_world: f64,
    reverse: bool,
) -> usize {
    stroke_arc_full(mesh, brush, spacing, radius_world, reverse, true)
}

/// O mesmo traço, com a opção de NÃO re-picar o centro.
///
/// ⚠️ **É a ablação da REALIMENTAÇÃO, e ela isola a única coisa que faz um dab
/// depender dos anteriores neste gesto.** O produto re-pica a cada passo do
/// `walk` (`sculpt_at` é um pick), então o centro SOBE com o barro — e um centro
/// que sobe muda a distância de todo vértice da pegada, que muda o peso, que
/// muda o depósito. Com os centros computados UMA vez sobre a superfície
/// virgem, a lista de dabs é a mesma e a realimentação some: o que sobrar de
/// variação ao longo do traço **não** é dela.
fn stroke_arc_full(
    mesh: &mut Mesh,
    brush: &Brush,
    spacing: f64,
    radius_world: f64,
    reverse: bool,
    repick: bool,
) -> usize {
    let span = ARC_TO - ARC_FROM;
    let steps = ((span * radius_world / spacing).floor() as usize).max(1);
    // Os centros da rota SEM realimentação saem da malha PRISTINA, antes de o
    // traço tocar nela.
    let pinned: Vec<Option<[f32; 3]>> = if repick {
        Vec::new()
    } else {
        (0..=steps)
            .map(|k| {
                let f = k as f64 / steps as f64;
                pick(mesh, ray_dir(if reverse { 1.0 - f } else { f }))
            })
            .collect()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(mesh);
    let mut n = 0;
    // ⚠️ **`enumerate` sobre `pinned` seria um BUG, não estilo.** Ele está VAZIO
    // no braço `repick` (a coluna do produto), então iterar por ele daria zero
    // dabs exatamente na metade que a tabela existe para medir; e o `k` também
    // computa o `f` do percurso, que não sai de coleção nenhuma.
    #[allow(clippy::needless_range_loop)]
    for k in 0..=steps {
        let f = k as f64 / steps as f64;
        let t = if reverse { 1.0 - f } else { f };
        let dir = ray_dir(t);
        let c = if repick { pick(mesh, dir) } else { pinned[k] };
        let Some(c) = c else { continue };
        let eye = [-dir[0], -dir[1], -dir[2]];
        stroke.dab(
            mesh,
            brush,
            &Dab::at(c, brush.radius, eye),
            Symmetry::default(),
        );
        n += 1;
    }
    n
}

/// O raio da SUPERFÍCIE ao longo do arco — medido, não assumido.
fn surface_radius(mesh: &Mesh) -> (f64, f64, f64) {
    let (mut lo, mut hi, mut acc, mut n) = (f64::INFINITY, f64::NEG_INFINITY, 0.0f64, 0u32);
    for k in 0..=40 {
        let t = k as f64 / 40.0;
        if let Some(c) = pick(mesh, ray_dir(t)) {
            let r = f64::from((c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt());
            lo = lo.min(r);
            hi = hi.max(r);
            acc += r;
            n += 1;
        }
    }
    (lo, hi, acc / f64::from(n))
}

/// **O CUME SINTÉTICO — a régua medida contra um campo que EU escrevi.**
///
/// Escreve, ao longo da normal congelada, uma altura que é o perfil TRANSVERSAL
/// do pincel multiplicado por uma ondulação LONGITUDINAL de amplitude conhecida
/// e período `period` em unidades de mundo:
///
/// ```text
/// h(v) = amp · curva(|y| / raio) · (1 + ripple · cos(2π · s / period))
/// ```
///
/// ⚠️ **Ele é a metade que faltava, e sem ele a §1 não significa nada.** O
/// «controle» da 1ª versão desta sonda era a malha INTOCADA, que mede
/// `0,000000` **por construção** (a altura é um deslocamento, e nada se
/// deslocou) — verde por vácuo, incapaz de detectar o ruído que de facto
/// contaminava a medição. Um controle tem de poder DISCRIMINAR:
///
/// * com `ripple = 0` o campo é **perfeitamente uniforme ao longo do traço**, e
///   o que a régua reportar é o **PISO DE RUÍDO** dela (o vazamento transversal
///   que a banda admite);
/// * com `ripple = r` a razão verdadeira é `2r` (pico a pico de um cosseno), e o
///   que a régua reportar é o **GANHO** dela — quanto de uma ondulação REAL, no
///   período exacto do dab, aquela malha consegue devolver.
///
/// ⚠️ **A recuperação é lida por CONSTRUÇÃO, não por promessa:** a altura é
/// escrita como `p0 + n0·h` e a régua lê `dot(p − p0, n0)`, então ela recupera
/// `h` exactamente — o que sobra na diferença é só a AMOSTRAGEM.
fn synthetic_ridge(
    base: &Mesh,
    edge: f64,
    radius_world: f64,
    amp: f64,
    ripple: f64,
    slope: f64,
    period: f64,
) -> (Mesh, Before, Profile) {
    let mut m = base.clone();
    let b = Before::of(&m);
    {
        let out = m.positions_mut();
        // ⚠️ O `zip` triplo é seguro aqui porque `out` É `m.positions_mut()` e o
        // `b` saiu de `Before::of(&m)` duas linhas acima — mesma malha, mesmo
        // comprimento por construção.
        for ((o, &p0), &n) in out.iter_mut().zip(&b.pos).zip(&b.nrm) {
            let t = f64::from(p0[1].abs()) / f64::from(RADIUS);
            if t >= 1.0 {
                continue;
            }
            let w = f64::from(Falloff::Plateau.weight(t as f32));
            let s = f64::from(p0[2]).atan2(f64::from(p0[0])) * radius_world;
            // A posição normalizada ao longo do arco, em [-1, 1], para a RAMPA.
            let mid = (ARC_FROM + ARC_TO) * 0.5 * radius_world;
            let half = (ARC_TO - ARC_FROM) * 0.5 * radius_world;
            let h = amp
                * w
                * (1.0
                    + ripple * (std::f64::consts::TAU * s / period).cos()
                    + slope * (s - mid) / half);
            *o = [
                p0[0] + n[0] * h as f32,
                p0[1] + n[1] * h as f32,
                p0[2] + n[2] * h as f32,
            ];
        }
    }
    let prof = profile(&b, &m, edge, radius_world, period);
    (m, b, prof)
}

// ─────────────────────────────────────────────────────────────────────────────
// A SONDA
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "medição, não gate: roda com --ignored --nocapture"]
fn what_the_sawtooth_along_a_stroke_is_made_of() {
    // ── 0. A FIXTURE ────────────────────────────────────────────────────────
    println!("\n=== 0. A FIXTURE — ela CONTÉM o fenômeno? ===\n");
    let ms = f64::from(min_spacing(RADIUS));
    let mut meshes = Vec::new();
    for label in ["+0", "+1", "+2", "uv"] {
        let m = mesh_at(label);
        let e = mean_edge(&m);
        let (lo, hi, mid) = surface_radius(&m);
        println!(
            "  malha {label:<3}: {:>8} vértices, {:>8} triângulos, aresta média {e:.5}",
            m.vert_count(),
            m.triangle_count()
        );
        println!(
            "     raio da superfície no arco {lo:.4}..{hi:.4} (média {mid:.4}) \
             — varia {:.2}%, e é por isso que a altura NÃO é |p|−1",
            (hi - lo) / mid * 100.0
        );
        println!(
            "     bins por período do espaçamento ({ms:.4}): {:.1}   ·   banda ±{:.4}",
            ms / e,
            e * BAND_EDGES
        );
        meshes.push((label, m, e, mid));
    }
    println!(
        "\n  pincel: raio {RADIUS}, força {STRENGTH}, espaçamento do produto \
         {ms:.4} (= {MIN_SPACING_FRACTION} · raio)"
    );

    // ── 0b. A CALIBRAÇÃO ────────────────────────────────────────────────────
    println!("\n=== 0b. A RÉGUA, MEDIDA CONTRA UM CUME QUE EU ESCREVI ===\n");
    println!(
        "  ⚠️ O controle da 1ª versão desta sonda era a malha INTOCADA, que mede\n     \
         `0,000000` POR CONSTRUÇÃO — verde por vácuo. Aqui o campo é conhecido:\n     \
         `ripple 0` diz o PISO DE RUÍDO da régua, `ripple 10 %` (razão verdadeira\n     \
         20 %) diz o GANHO dela no período EXACTO do dab.\n"
    );
    println!(
        "  {:>6}  {:>11}  {:>11}  {:>10} {:>10}  {:>10} {:>10}  {:>11}",
        "malha",
        "campo",
        "altura méd.",
        "SERRA real",
        "lida",
        "RAMPA real",
        "lida",
        "v/bin min·méd"
    );
    let mut leaks = Vec::new();
    for (mlabel, base, edge, rw) in &meshes {
        // ⚠️ **O valor REAL é derivado, e a 1ª versão desta tabela o escreveu à
        // mão e ERRADO:** um cosseno tem pico-a-pico `2·ripple` em qualquer
        // janela de um período, mas uma RAMPA só mostra, no miolo, a fracção do
        // arco que o miolo cobre — eu declarei `100%` e a régua leu `50%`, que
        // é a resposta certa (`slope · 2 · CORE`). *O valor declarado tem de
        // ser computado sobre a MESMA janela que a régua lê.*
        for &(rip, slope, label) in &[
            (0.0f64, 0.0f64, "nada"),
            (0.10f64, 0.0f64, "serra"),
            (0.0f64, 0.50f64, "rampa"),
        ] {
            let want_r = 2.0 * rip * 100.0;
            let want_t = slope * 2.0 * CORE * 100.0;
            let (_, _, p) = synthetic_ridge(base, *edge, *rw, 0.10, rip, slope, ms);
            println!(
                "  {:>6}  {:>11}  {:>11.6}  {:>9.2}% {:>9.2}%  {:>9.2}% {:>9.2}%  {:>4} · {:>4.1}",
                mlabel,
                label,
                p.mean,
                want_r,
                p.ratio() * 100.0,
                want_t,
                p.trend_ratio() * 100.0,
                p.min_per_bin,
                p.mean_per_bin
            );
            // ⚠️ **A DIAFONIA, e é ela que decide se a §1 diz alguma coisa:** a
            // média móvel é um retângulo, não um filtro ideal, então uma RAMPA
            // vaza um pouco para o canal do dente de serra. Os traços medidos
            // têm rampas de dezenas por cento — sem este número, uma «serra» de
            // 3 % podia ser inteirinha a rampa vista de lado.
            if slope > 0.0 {
                leaks.push(p.ratio() / p.trend_ratio());
            }
        }
    }
    println!(
        "\n  DIAFONIA rampa → serra (fracção da rampa que aparece como serra): {}",
        leaks
            .iter()
            .map(|l| format!("{:.4}", l))
            .collect::<Vec<_>>()
            .join(" · ")
    );

    // ── 1. A ONDULAÇÃO ──────────────────────────────────────────────────────
    println!("\n=== 1. A ONDULAÇÃO AO LONGO DO TRAÇO (verbo Draw de fábrica) ===\n");
    println!(
        "  ⚠️ Leia cada linha contra o PISO da §0b da MESMA malha: uma razão\n     \
         abaixo dele não é ondulação, é a régua.\n"
    );
    println!(
        "  {:>6}  {:>10}  {:>6}  {:>11}  {:>9}  {:>9}  {:>9}  {:>11}",
        "malha", "espaçam.", "dabs", "altura méd.", "SERRA", "piso", "RAMPA", "v/bin min·méd"
    );
    let draw = factory(Verb::Draw);
    for (i, (label, base, edge, rw)) in meshes.iter().enumerate() {
        for div in [1.0f64, 2.0, 4.0] {
            let sp = ms / div;
            let mut m = base.clone();
            let b = Before::of(&m);
            let n = stroke_arc(&mut m, &draw, sp, *rw);
            let p = profile(&b, &m, *edge, *rw, sp);
            println!(
                "  {:>6}  {:>10}  {:>6}  {:>11.6}  {:>8.2}%  {:>8.2}%  {:>8.2}%  {:>4} · {:>4.1}",
                (*label).to_string(),
                if (div - 1.0).abs() < 1e-9 {
                    "PRODUTO".to_string()
                } else {
                    format!("/{div:.0}")
                },
                n,
                p.mean,
                p.ratio() * 100.0,
                leaks[i] * p.trend_ratio() * 100.0,
                p.trend_ratio() * 100.0,
                p.min_per_bin,
                p.mean_per_bin
            );
        }
    }

    // ── 1b. A FORMA DA TENDÊNCIA ────────────────────────────────────────────
    println!("\n=== 1b. QUE FORMA A TENDÊNCIA TEM? ===\n");
    println!(
        "  Um pico-a-pico diz o TAMANHO e não a FORMA, e as curas são opostas:\n  \
         uma RAMPA monotônica é um traço que engrossa ao longo do caminho; um\n  \
         ARCO é a geometria da esfera; ruído largo é outra coisa. O miolo em 12\n  \
         amostras, normalizado pela própria média (100 = a média do traço):\n"
    );
    println!(
        "  ⚠️ E cada malha vem DUAS vezes: o MESMO caminho percorrido para a\n     \
         frente e ao contrário. Se o perfil se ESPELHA, a tendência é da LEI de\n     \
         depósito; se ele fica onde está, é da GEOMETRIA da malha.\n"
    );
    for (label, base, edge, rw) in &meshes {
        for rev in [false, true] {
            let mut m = base.clone();
            let b = Before::of(&m);
            stroke_arc_dir(&mut m, &draw, ms, *rw, rev);
            let p = profile(&b, &m, *edge, *rw, ms);
            let n = p.core_smooth.len();
            print!("  {label} {}: ", if rev { "<-" } else { "->" });
            for k in 0..12 {
                let idx = k * (n - 1) / 11;
                print!("{:>5.0}", p.core_smooth[idx] / p.mean * 100.0);
            }
            println!("   (média {:.6})", p.mean);
        }
    }

    // ── 1c. A REALIMENTAÇÃO ────────────────────────────────────────────────
    println!("\n=== 1c. A TENDÊNCIA É A REALIMENTAÇÃO DO PICK? ===\n");
    println!(
        "  A MESMA lista de dabs, com e sem o centro re-picado. O produto\n  \
         re-pica (o `sculpt_at` de cada passo do `walk` É um pick), então o\n  \
         centro SOBE com o barro; pregado na superfície virgem, ele não sobe.\n"
    );
    println!(
        "  {:>6}  {:>10}  {:>11}  {:>9}  {:>9}",
        "malha", "centro", "altura méd.", "SERRA", "RAMPA"
    );
    for (label, base, edge, rw) in &meshes {
        for repick in [true, false] {
            let mut m = base.clone();
            let b = Before::of(&m);
            stroke_arc_full(&mut m, &draw, ms, *rw, false, repick);
            let p = profile(&b, &m, *edge, *rw, ms);
            println!(
                "  {label:>6}  {:>10}  {:>11.6}  {:>8.2}%  {:>8.2}%",
                if repick { "re-picado" } else { "PREGADO" },
                p.mean,
                p.ratio() * 100.0,
                p.trend_ratio() * 100.0
            );
        }
    }

    // ── 2. O ESPAÇAMENTO ────────────────────────────────────────────────────
    println!("\n=== 2. A DEPENDÊNCIA DO ESPAÇAMENTO (malha de fábrica) ===\n");
    println!(
        "  MESMO caminho de mundo, MESMO pincel; só o passo muda.\n  \
         ⚠️ A pergunta que decide a LEI é a coluna «altura ÷ dabs»: se ela for\n     \
         constante, o depósito é uma SOMA DE CARIMBOS; se a «altura» for constante,\n     \
         ele é uma INTEGRAL DE CAMINHO.\n"
    );
    let (_, base0, edge0, rw0) = &meshes[0];
    println!(
        "  {:>10}  {:>6}  {:>11}  {:>9}  {:>9}  {:>13}",
        "espaçam.", "dabs", "altura méd.", "SERRA", "RAMPA", "altura ÷ dabs"
    );
    for div in [0.25f64, 0.5, 1.0, 2.0, 4.0, 8.0] {
        let sp = ms / div;
        let mut m = base0.clone();
        let b = Before::of(&m);
        let n = stroke_arc(&mut m, &draw, sp, *rw0);
        let p = profile(&b, &m, *edge0, *rw0, sp);
        println!(
            "  {:>10.5}  {:>6}  {:>11.6}  {:>8.2}%  {:>8.2}%  {:>13.7}",
            sp,
            n,
            p.mean,
            p.ratio() * 100.0,
            p.trend_ratio() * 100.0,
            p.mean / f64::from(n as u32)
        );
    }

    // ── 3. A TAXA DE EVENTOS ────────────────────────────────────────────────
    println!("\n=== 3. A DEPENDÊNCIA DA TAXA DE EVENTOS ===\n");
    println!(
        "  O MESMO caminho entregue em N eventos IRREGULARES, pelo `walk` REAL e\n  \
         pela porta `Walk::anchor`. O CONTROLE é a coluna «dabs»: a lei do passo\n  \
         exato promete que ela não se mexe.\n"
    );
    println!(
        "  {:>8}  {:>6}  {:>11}  {:>9}  {:>9}  {:>12}  {:>12}",
        "eventos", "dabs", "altura méd.", "SERRA", "RAMPA", "Δ altura", "Δ posição"
    );
    let mut ref_mean = f64::NAN;
    // ⚠️ **A COLUNA QUE TORNA A TABELA ATRIBUÍVEL.** A contagem de dabs é o
    // controle da lei do passo, e ela é PINADA — mas duas listas do mesmo
    // tamanho podem pousar em sítios diferentes, e aí a altura muda sem que a
    // lei do depósito tenha nada com isso. `Δ posição` é o maior desvio de
    // ARCO entre esta lista e a de um evento só, em unidades de mundo: se ele
    // for da ordem do espaçamento, o que a coluna da altura mede é ONDE os
    // dabs caíram, não QUANTO cada um deposita.
    let ref_ts = walk_ts(1, ms, *rw0);
    for repick in [true, false] {
        println!(
            "  --- centro {} ---",
            if repick {
                "RE-PICADO (o produto)"
            } else {
                "PREGADO (sem realimentação)"
            }
        );
        for events in [1usize, 2, 3, 5, 8, 20, 100] {
            let ts = walk_ts(events, ms, *rw0);
            let mut m = base0.clone();
            let b = Before::of(&m);
            let pinned: Vec<Option<[f32; 3]>> = if repick {
                Vec::new()
            } else {
                ts.iter().map(|&t| pick(&m, ray_dir(t))).collect()
            };
            let mut stroke = SculptStroke::default();
            stroke.begin(&m);
            // ⚠️ **A contagem é a dos dabs APLICADOS, não a dos passos do `walk`.**
            // Um pick que erra cai no `continue` **em silêncio**, e a 1ª versão desta
            // tabela imprimia `ts.len()` — 41 em toda linha, um «controle» que não
            // podia falhar. *Uma contagem que não conta o que aconteceu é um controle
            // decorativo.*
            //
            // ⚠️ **E ele PEGOU um defeito real, que era o do teste de triângulo.**
            // Antes da folga baricêntrica de `ph2d_mesh::ray::BARY_SLACK` esta coluna
            // lia `40, 40, 39, 39, 39, 38, 35` (re-picado) e `37, 37, 38, 38, 38, 36,
            // 36` (pregado) — picks que erravam uma superfície FECHADA. Com a folga
            // ela lê **41 em toda linha**, e o `Δ altura` cai de **26,1% para 0,000%**:
            // um quarto do depósito estava a ser perdido em silêncio no traço mais
            // subdividido. Medido por A/B no MESMO binário, mutando a const para zero.
            //
            // ⚠️ **A `walk` foi EXONERADA por esta mesma tabela**, e é o motivo de ela
            // existir: a contagem de PASSOS nunca dependeu do número de eventos (a
            // sonda irmã `measure_the_walk_loses_dabs` mede a lei em `f64` e a âncora
            // deriva 4e-8). *A perda era do pick, e o oráculo que a separou da lei do
            // passo foi ter as duas medidas lado a lado.*
            let mut applied = 0usize;
            for (k, &t) in ts.iter().enumerate() {
                let dir = ray_dir(t);
                let c = if repick { pick(&m, dir) } else { pinned[k] };
                let Some(c) = c else { continue };
                let eye = [-dir[0], -dir[1], -dir[2]];
                stroke.dab(&mut m, &draw, &Dab::at(c, RADIUS, eye), Symmetry::default());
                applied += 1;
            }
            let p = profile(&b, &m, *edge0, *rw0, ms);
            if events == 1 {
                ref_mean = p.mean;
            }
            let dpos = ts
                .iter()
                .zip(ref_ts.iter())
                .map(|(a, b)| ((a - b) * (ARC_TO - ARC_FROM) * *rw0).abs())
                .fold(0.0f64, f64::max);
            println!(
                "  {:>8}  {:>6}  {:>11.6}  {:>8.2}%  {:>8.2}%  {:>11.3}%  {:>12.4e}",
                events,
                applied,
                p.mean,
                p.ratio() * 100.0,
                p.trend_ratio() * 100.0,
                (p.mean - ref_mean).abs() / ref_mean.abs() * 100.0,
                dpos
            );
        }
    }

    // ── 4. A AUTO-AMPLIFICAÇÃO ──────────────────────────────────────────────
    println!("\n=== 4. A AUTO-AMPLIFICAÇÃO — N dabs no MESMO lugar ===\n");
    println!(
        "  ⚠️ `Verb::DrawSharp` NÃO EXISTE nesta crate — ele saiu com motivo\n     \
         (`21_plano…` §7.18: o que o nome promete mora na CURVA, e a curva de\n     \
         fábrica por-tool vive num `.blend` binário). No lugar dele vai o **Draw\n     \
         com `accumulate` DESARMADO**, que é o mecanismo que o nome nomeia — o\n     \
         `from_live` cai e a distância passa a sair do `pre` CONGELADO.\n"
    );
    println!(
        "  ⚠️ E o gesto não é alcançável pelo `walk`: parado, ele devolve `None`\n     \
         (carry) e NÃO carimba. Quem chega aqui é quem esfrega o mesmo sítio.\n"
    );
    for repick in [false, true] {
        println!(
            "  --- centro {} ---",
            if repick {
                "RE-PICADO a cada dab (o gesto do produto)"
            } else {
                "PREGADO no ponto de partida (fica enterrado sob o barro)"
            }
        );
        println!(
            "  {:>16}  {:>9}  {:>9}  {:>9}  {:>9}  {:>9}  {:>9}",
            "verbo", "1 dab", "2", "4", "8", "16", "32"
        );
        for (label, brush) in verbs_under_test() {
            let mut m = base0.clone();
            let b = Before::of(&m);
            let dir = ray_dir(0.5);
            let Some(c0) = pick(&m, dir) else { continue };
            let eye = [-dir[0], -dir[1], -dir[2]];
            let mut stroke = SculptStroke::default();
            stroke.begin(&m);
            let mut row = String::new();
            let mut next = 1usize;
            for k in 1..=32 {
                let c = if repick {
                    pick(&m, dir).unwrap_or(c0)
                } else {
                    c0
                };
                stroke.dab(
                    &mut m,
                    &brush,
                    &Dab::at(c, RADIUS, eye),
                    Symmetry::default(),
                );
                if k == next {
                    row.push_str(&format!("  {:>9.5}", peak(&b, &m, c0)));
                    next *= 2;
                }
            }
            println!("  {label:>16}{row}");
        }
        println!();
    }

    // ── 5. A DUREZA ─────────────────────────────────────────────────────────
    println!("=== 5. O DEGRAU ENTRE VÉRTICES VIZINHOS, POR DUREZA ===\n");
    println!(
        "  Degrau = maior |Δaltura| sobre uma ARESTA cujos dois extremos a pegada moveu.\n  \
         ⚠️ Ele CRESCE com o depósito, então a coluna livre de escala é «÷ altura».\n  \
         O CONTROLE é a linha `Falloff::Constant`: um disco duro TEM de dar o maior\n  \
         degrau da tabela — se não der, a régua não mede o que diz.\n"
    );
    println!(
        "  {:>22}  {:>11}  {:>11}  {:>10}  {:>9}",
        "pincel", "altura méd.", "degrau máx.", "÷ altura", "SERRA"
    );
    for (label, mut brush) in hardness_rows() {
        brush.radius = RADIUS;
        brush.strength = STRENGTH;
        let mut m = base0.clone();
        let b = Before::of(&m);
        stroke_arc(&mut m, &brush, ms, *rw0);
        let p = profile(&b, &m, *edge0, *rw0, ms);
        let step = max_edge_step(&b, &m);
        println!(
            "  {label:>22}  {:>11.6}  {:>11.6}  {:>10.4}  {:>8.2}%",
            p.mean,
            step,
            if p.mean.abs() < 1e-12 {
                f64::NAN
            } else {
                step / p.mean.abs()
            },
            p.ratio() * 100.0
        );
    }

    // ── 6. O AUTOSMOOTH ─────────────────────────────────────────────────────
    println!("\n=== 6. O AUTOSMOOTH — quanto ele esconde? ===\n");
    println!(
        "  ⚠️ A HIPÓTESE CAIU: «o default de fábrica E 0» são o MESMO número.\n     \
         `Brush::default()` ship `auto_smooth: 0,0`, que é o neutro do próprio\n     \
         passe (`auto_smooth_brush` devolve `None`) e o default do Blender.\n     \
         Não há par a comparar — há uma VARREDURA, e é ela que responde.\n"
    );
    println!(
        "  {:>12}  {:>11}  {:>9}  {:>9}  {:>11}  {:>10}",
        "auto_smooth", "altura méd.", "SERRA", "RAMPA", "degrau máx.", "÷ altura"
    );
    for &a in &[0.0f32, 0.25, 0.5, 1.0] {
        let brush = Brush {
            auto_smooth: a,
            ..factory(Verb::Draw)
        };
        let mut m = base0.clone();
        let b = Before::of(&m);
        stroke_arc(&mut m, &brush, ms, *rw0);
        let p = profile(&b, &m, *edge0, *rw0, ms);
        let step = max_edge_step(&b, &m);
        println!(
            "  {:>12.2}  {:>11.6}  {:>8.2}%  {:>8.2}%  {:>11.6}  {:>10.4}",
            a,
            p.mean,
            p.ratio() * 100.0,
            p.trend_ratio() * 100.0,
            step,
            if p.mean.abs() < 1e-12 {
                f64::NAN
            } else {
                step / p.mean.abs()
            }
        );
    }
    // E sobre a pior linha da metade 5: a dureza no máximo.
    println!("\n  o mesmo, sobre o pior pincel da metade 5 (`hardness = 1,0`):\n");
    println!(
        "  {:>12}  {:>11}  {:>9}  {:>9}  {:>11}  {:>10}",
        "auto_smooth", "altura méd.", "SERRA", "RAMPA", "degrau máx.", "÷ altura"
    );
    for &a in &[0.0f32, 0.25, 0.5, 1.0] {
        let brush = Brush {
            hardness: 1.0,
            auto_smooth: a,
            ..factory(Verb::Draw)
        };
        let mut m = base0.clone();
        let b = Before::of(&m);
        stroke_arc(&mut m, &brush, ms, *rw0);
        let p = profile(&b, &m, *edge0, *rw0, ms);
        let step = max_edge_step(&b, &m);
        println!(
            "  {:>12.2}  {:>11.6}  {:>8.2}%  {:>8.2}%  {:>11.6}  {:>10.4}",
            a,
            p.mean,
            p.ratio() * 100.0,
            p.trend_ratio() * 100.0,
            step,
            if p.mean.abs() < 1e-12 {
                f64::NAN
            } else {
                step / p.mean.abs()
            }
        );
    }
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// AS RÉGUAS AUXILIARES
// ─────────────────────────────────────────────────────────────────────────────

/// O pico do depósito em torno de `c` — a altura de maior módulo entre os
/// vértices a menos de um quarto de raio do centro.
fn peak(b: &Before, mesh: &Mesh, c: [f32; 3]) -> f64 {
    let r2 = (RADIUS * 0.25).powi(2);
    let mut best = 0.0f64;
    for v in 0..mesh.vert_count() {
        let p = b.pos[v];
        let d = (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
        if d > r2 {
            continue;
        }
        let h = height(b, mesh, v);
        if h.abs() > best.abs() {
            best = h;
        }
    }
    best
}

/// O maior salto de altura sobre uma aresta cujos DOIS extremos a pegada moveu.
///
/// ⚠️ **Os dois extremos, e não um.** Uma aresta com um extremo parado mede a
/// FRONTEIRA da pegada — que é o pouso da curva, não um degrau interno — e ela
/// existe em qualquer pincel, inclusive no mais macio.
fn max_edge_step(b: &Before, mesh: &Mesh) -> f64 {
    const MOVED: f64 = 1e-6;
    let adj = mesh.adjacency();
    let mut worst = 0.0f64;
    for i in 0..mesh.vert_count() {
        let hi = height(b, mesh, i);
        if hi.abs() < MOVED {
            continue;
        }
        for &j in adj.vert_verts.neighbours(i) {
            let hj = height(b, mesh, j as usize);
            if hj.abs() < MOVED {
                continue;
            }
            worst = worst.max((hi - hj).abs());
        }
    }
    worst
}

/// Os verbos da metade 4, com o pincel de fábrica de cada um.
fn verbs_under_test() -> Vec<(&'static str, Brush)> {
    vec![
        ("Draw", factory(Verb::Draw)),
        (
            "Draw !accum",
            Brush {
                accumulate: false,
                ..factory(Verb::Draw)
            },
        ),
        ("ClayStrips", factory(Verb::ClayStrips)),
        ("Crease", factory(Verb::Crease)),
        ("Blob", factory(Verb::Blob)),
        ("Layer", factory(Verb::Layer)),
        ("Inflate", factory(Verb::Inflate)),
    ]
}

/// As linhas da metade 5 — a dureza varrida sobre a curva de fábrica, mais as
/// duas curvas que servem de CONTROLE nas pontas.
fn hardness_rows() -> Vec<(String, Brush)> {
    let mut rows = Vec::new();
    for &h in &[0.0f32, 0.5, 0.9, 1.0] {
        rows.push((
            format!("hardness {h:.2} (fábrica)"),
            Brush {
                hardness: h,
                ..factory(Verb::Draw)
            },
        ));
    }
    rows.push((
        "Falloff::Constant".to_string(),
        Brush {
            falloff: Falloff::Constant,
            ..factory(Verb::Draw)
        },
    ));
    rows.push((
        "Falloff::Smooth".to_string(),
        Brush {
            falloff: Falloff::Smooth,
            ..factory(Verb::Draw)
        },
    ));
    rows
}

/// Os `t ∈ [0, 1]` dos dabs quando o MESMO caminho é entregue em `events`
/// eventos IRREGULARES — pelo [`walk`] real e pela porta [`ph2d_sculpt3d::Walk::anchor`].
///
/// ⚠️ **As fronteiras são irregulares de propósito, e isso é o CONTROLE da
/// metade 3:** com eventos do mesmo tamanho a grade dos dabs e a dos eventos se
/// alinham por construção e a coluna sairia invariante sem provar nada.
fn walk_ts(events: usize, spacing: f64, radius_world: f64) -> Vec<f64> {
    let len = (ARC_TO - ARC_FROM) * radius_world;
    // Um LCG minúsculo: determinístico, e sem dependência nova para uma sonda.
    let mut st: u64 = 0x2545_F491_4F6C_DD1D;
    let mut w: Vec<f64> = (0..events)
        .map(|_| {
            st = st
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            0.5 + (st >> 40) as f64 / f64::from(1u32 << 24)
        })
        .collect();
    let total: f64 = w.iter().sum();
    let mut acc = 0.0;
    for x in &mut w {
        acc += *x / total * len;
        *x = acc;
    }
    if let Some(l) = w.last_mut() {
        *l = len;
    }

    let mut out = vec![0.0f64];
    let mut anchor = [0.0f32, 0.0];
    for bnd in w {
        let to = [bnd as f32, 0.0];
        if let Some(steps) = walk(anchor, to, spacing as f32) {
            for p in steps {
                out.push(f64::from(p[0]) / len);
            }
            anchor = steps.anchor();
        }
    }
    out
}
