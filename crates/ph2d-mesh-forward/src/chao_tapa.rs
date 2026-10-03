//! ⭐⭐⭐ **O CHÃO QUE TAPA AS PEÇAS** — o chão é um plano infinito em `y = chão`: todo raio que desce
//! de um ponto acima dele acerta-o. A metade de baixo do céu que a peça via deixa de ser o céu e
//! passa a ser o CHÃO, iluminado pelo céu de cima e escurecido onde as peças o tapam — o mesmo
//! escurecimento que o quadro desenha no chão (o passe do céu do chão, `ceu_chao.wgsl`).
//!
//! Longe das peças o chão vê o céu todo e devolve o que o estúdio já pinta em baixo: nada muda.
//! Debaixo de uma peça pousada ele está à sombra dela, e a barriga da peça escurece. Precedente: o
//! *«Lower Hemisphere Is Solid Color»* da luz do céu do Unreal — aqui a cor de baixo é a do chão
//! **naquele ponto**, não uma constante.
//!
//! A lei: `D(p, n) = ∫_{d.y < 0} (1 − V(x(d))) · max(n·d, 0) dω / π`, com `x(d)` o ponto onde o raio
//! acerta o chão e `V` a visibilidade do céu do chão. O total de baixo tem forma fechada
//! (`(1 − n.y)/2`); as [`RAIOS_CHAO`] direcções fixas (espiral de Fibonacci no hemisfério de baixo,
//! sem base tangente — nenhuma costura quando `n` roda) só dão a MÉDIA de `1 − V` ponderada pelo
//! cosseno. Com `V` constante a resposta é exacta.
//!
//! **O reflexo** ([`reflexo`]): o lobo GGX de `α` à volta da direcção reflectida em dois anéis de
//! [`TAPS_ANEL`] direcções, nos quantis `1/4` e `3/4` da distribuição das meias-direcções
//! (`tan² θh = α² q/(1 − q)`, a reflectida a `2 θh`), pesadas pelo cosseno à reflectida — a
//! amostragem do pré-filtro do céu com `n = v = r`. Cada direcção que desce lê o chão onde o acerta;
//! as que sobem são céu. Com `α → 0` os anéis fecham-se na reflectida: o reflexo nítido, exacto.
//! ⛔ Recusado (medido): misturar o nítido com o largo de cosseno por `α` — `0,060` onde o chão
//! escurece no metal de rugosidade `0,5`, os dois sinais (os dois anéis: `0,023`); e um TERCEIRO anel —
//! mais perto do lobo denso na CPU e nada contra o Cycles (o resto é o pré-filtro com `n = v = r`).
//!
//! Oráculo: `docs/3DModeling/ferramentas/oraculo_chao_tapa_blender.py` → `fixtures/oraculo_chao_tapa.csv`.

/// Direcções no hemisfério de baixo.
pub const RAIOS_CHAO: usize = 16;

/// As direcções: uniformes em ângulo sólido no hemisfério de baixo (espiral de Fibonacci).
#[must_use]
pub fn direcoes() -> [[f32; 3]; RAIOS_CHAO] {
    std::array::from_fn(|k| espiral(k, RAIOS_CHAO))
}

/// A direcção `k` de `total` da espiral de Fibonacci no hemisfério de baixo.
pub(crate) fn espiral(k: usize, total: usize) -> [f32; 3] {
    let ouro = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    let y = -(k as f32 + 0.5) / total as f32;
    let r = (1.0 - y * y).sqrt();
    let f = k as f32 * ouro;
    [r * f.cos(), y, r * f.sin()]
}

/// `D(p, n)`: a fracção da irradiância do céu (em unidades de `π`) que o chão em `y = chao` apaga ao
/// ponto `p` de normal `n`; `v(x, z)` é a visibilidade do céu do chão.
#[must_use]
pub fn escurecimento(p: [f32; 3], n: [f32; 3], chao: f32, v: impl Fn(f32, f32) -> f32) -> f32 {
    escurecimento_com(&direcoes(), p, n, chao, v)
}

pub(crate) fn escurecimento_com(
    dirs: &[[f32; 3]],
    p: [f32; 3],
    n: [f32; 3],
    chao: f32,
    v: impl Fn(f32, f32) -> f32,
) -> f32 {
    let baixo = ((1.0 - n[1]) * 0.5).clamp(0.0, 1.0);
    if baixo < 1.0e-4 {
        return 0.0;
    }
    let h = (p[1] - chao).max(0.0);
    let (mut soma, mut pesos) = (0.0f32, 0.0f32);
    for d in dirs {
        let w = n[0] * d[0] + n[1] * d[1] + n[2] * d[2];
        if w <= 0.0 {
            continue;
        }
        let t = h / -d[1];
        soma += w * (1.0 - v(p[0] + d[0] * t, p[2] + d[2] * t));
        pesos += w;
    }
    if pesos <= 0.0 {
        return 0.0;
    }
    baixo * soma / pesos
}

/// ⭐⭐ **A pegada de uma peça no chão** — `(centro x, centro z, raio)` da caixa local `caixa` posta no
/// mundo por `modelo` (colunas). O REFLEXO da peça só lê o escurecimento do chão na zona dela: inteiro
/// até `1,5` raios, a nada aos `3` (a sombra de uma esfera pousada aí já é `(1/√10)³ ≈ 3 %`).
/// ⛔ Report do dono (03/10): o reflexo mostrava a sombra de uma VIZINHA sem a vizinha — ele sabe o céu
/// e o chão, não as outras peças. ⛔ Recusado (medido): a grelha do contacto DELA no chão (a sombra só
/// dela, sem palpite de distância) — imprecisa rente à peça, onde o chão está: base de baixo `0,073`
/// contra `0,0055`, e `334` sombras órfãs fora da zona.
#[must_use]
pub fn pegada(modelo: &[[f32; 4]; 4], (lo, hi): ([f32; 3], [f32; 3])) -> [f32; 3] {
    let (mut a, mut b) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for k in 0..8 {
        let c = [
            if k & 1 == 0 { lo[0] } else { hi[0] },
            if k & 2 == 0 { lo[1] } else { hi[1] },
            if k & 4 == 0 { lo[2] } else { hi[2] },
        ];
        let w = [0, 2].map(|e| modelo[3][e] + (0..3).map(|j| modelo[j][e] * c[j]).sum::<f32>());
        for e in 0..2 {
            a[e] = a[e].min(w[e]);
            b[e] = b[e].max(w[e]);
        }
    }
    [
        0.5 * (a[0] + b[0]),
        0.5 * (a[1] + b[1]),
        0.5 * (b[0] - a[0]).max(b[1] - a[1]),
    ]
}

/// Direcções por anel do lobo do reflexo. ⚠️ Medido (03/10, contra o Cycles no metal de rugosidade
/// `0,5`): `8` por anel `0,0233` onde o chão escurece, `4` por anel `0,0268` a METADE do custo
/// (`+0,166 → +0,075 ms` em ecrã cheio a 1080p, RTX 5060 Ti). Na difusa a mesma troca custava `+50 %`
/// de erro (`0,0098 → 0,0149`): lá ficam as [`RAIOS_CHAO`] `16`.
pub const TAPS_ANEL: usize = 4;

/// Os quantis dos dois anéis na distribuição das meias-direcções do GGX.
const QUANTIS: [f32; 2] = [0.25, 0.75];

/// O ângulo à reflectida de cada anel, para `α`.
fn angulos(alpha: f32) -> [f32; 2] {
    QUANTIS.map(|q| 2.0 * (alpha * (q / (1.0 - q)).sqrt()).atan())
}

/// A base à volta de `r`: contínua em todo lado menos em `r = ±y` (onde o anel gira, sem salto).
fn base(r: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let c = [-r[2], 0.0, r[0]];
    let l = (c[0] * c[0] + c[2] * c[2]).sqrt();
    let t1 = if l < 1.0e-4 {
        [1.0, 0.0, 0.0]
    } else {
        [c[0] / l, 0.0, c[2] / l]
    };
    let t2 = [
        r[1] * t1[2] - r[2] * t1[1],
        r[2] * t1[0] - r[0] * t1[2],
        r[0] * t1[1] - r[1] * t1[0],
    ];
    (t1, t2)
}

/// A fracção do lobo do reflexo (direcção `r`, rugosidade `α` do GGX) que o chão escurece; `v(x, z)`
/// é a visibilidade do céu do chão.
#[must_use]
pub fn reflexo(
    p: [f32; 3],
    r: [f32; 3],
    alpha: f32,
    chao: f32,
    v: impl Fn(f32, f32) -> f32,
) -> f32 {
    let h = (p[1] - chao).max(0.0);
    let (t1, t2) = base(r);
    let (mut soma, mut pesos) = (0.0f32, 0.0f32);
    for (a, phi) in angulos(alpha).into_iter().enumerate() {
        let (cf, sf) = (phi.cos(), phi.sin());
        if cf <= 0.0 {
            continue;
        }
        for j in 0..TAPS_ANEL {
            let psi = (j as f32 + 0.5 * a as f32) * std::f32::consts::TAU / TAPS_ANEL as f32;
            let (cp, sp) = (psi.cos(), psi.sin());
            let d: [f32; 3] = std::array::from_fn(|e| cf * r[e] + sf * (cp * t1[e] + sp * t2[e]));
            pesos += cf;
            if d[1] < 0.0 {
                let t = h / -d[1];
                soma += cf * (1.0 - v(p[0] + d[0] * t, p[2] + d[2] * t));
            }
        }
    }
    if pesos <= 0.0 { 0.0 } else { soma / pesos }
}

/// O gémeo WGSL de [`escurecimento`]: `fn chao_tapa(p, n) -> f32`, com a altura em `quadro.chao.x` e
/// a visibilidade lida por `ceu_do_chao(p)` (do `forward.wgsl`). As direcções CALCULAM-SE no laço (a
/// mesma espiral da CPU), sem arrays: nada que um compilador GLES tenha de indexar.
#[must_use]
pub fn wgsl() -> String {
    let ouro = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    let q = QUANTIS.map(|q| format!("{:?}", (q / (1.0 - q)).sqrt()));
    format!(
        "{CEU_DO_CHAO}\n{MEMO}\n{RAIO}\nfn chao_tapa(p: vec3<f32>, n: vec3<f32>) -> f32 {{\n{ABRE}\
         \x20   for (var k = 0u; k < {k}u; k = k + 1u) {{\n\
         \x20       let y = -(f32(k) + 0.5) / {kf:?};\n\
         \x20       let r = sqrt(1.0 - y * y);\n\
         \x20       let f = f32(k) * {ouro:?};\n\
         \x20       a = a + chao_raio(p, n, h, vec3<f32>(r * cos(f), y, r * sin(f)));\n\
         \x20   }}\n{FECHA}}}\n\n\
         {TAP}\nfn chao_reflexo(p: vec3<f32>, r: vec3<f32>, alpha: f32) -> f32 {{\n\
         \x20   let h = max(p.y - quadro.chao.x, 0.0);\n\
         \x20   let c0 = 2.0 * atan(alpha * {q0});\n\
         \x20   let c1 = 2.0 * atan(alpha * {q1});\n{BASE}    var b = vec2<f32>(0.0);\n\
         \x20   for (var j = 0u; j < {t2}u; j = j + 1u) {{\n\
         \x20       let anel = j / {t}u;\n\
         \x20       let psi = (f32(j % {t}u) + 0.5 * f32(anel)) * {passo:?};\n\
         \x20       b = b + reflexo_tap(p, r, h, select(c0, c1, anel == 1u), cos(psi) * t1 + sin(psi) * t2);\n\
         \x20   }}\n{FECHA_REFLEXO}}}\n",
        k = RAIOS_CHAO,
        kf = RAIOS_CHAO as f32,
        t = TAPS_ANEL,
        t2 = 2 * TAPS_ANEL,
        passo = std::f32::consts::TAU / TAPS_ANEL as f32,
        q0 = q[0],
        q1 = q[1],
    )
}

/// ⭐ A MESMA pergunta no mesmo pixel tem a mesma resposta: o material avalia o lobo dielétrico e o
/// metálico com a mesma reflectida e a mesma rugosidade, e cada um refazia as leituras do chão.
const MEMO: &str = r"var<private> memo_n: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);
var<private> memo_r: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, -1.0);
var<private> memo_ra: f32 = -1.0;

fn chao_tapa_no_pixel(p: vec3<f32>, n: vec3<f32>) -> f32 {
    if (memo_n.w < 0.0 || any(memo_n.xyz != n)) {
        memo_n = vec4<f32>(n, chao_tapa(p, n));
    }
    return memo_n.w;
}

fn chao_reflexo_no_pixel(p: vec3<f32>, r: vec3<f32>, alpha: f32) -> f32 {
    if (memo_r.w < 0.0 || any(memo_r.xyz != r) || memo_ra != alpha) {
        memo_r = vec4<f32>(r, chao_reflexo(p, r, alpha));
        memo_ra = alpha;
    }
    return memo_r.w;
}
";

const BASE: &str = "    let c = vec2<f32>(-r.z, r.x);
    let l = length(c);
    var t1 = vec3<f32>(1.0, 0.0, 0.0);
    if (l >= 1.0e-4) {
        t1 = vec3<f32>(c.x / l, 0.0, c.y / l);
    }
    let t2 = cross(r, t1);
";

const TAP: &str = r"fn reflexo_tap(p: vec3<f32>, r: vec3<f32>, h: f32, phi: f32, lado: vec3<f32>) -> vec2<f32> {
    let cf = cos(phi);
    if (cf <= 0.0) {
        return vec2<f32>(0.0);
    }
    let d = cf * r + sin(phi) * lado;
    if (d.y >= 0.0) {
        return vec2<f32>(0.0, cf);
    }
    let x = p + d * (h / -d.y);
    let no_chao = vec3<f32>(x.x, quadro.chao.x, x.z);
    return vec2<f32>(cf * sombra_propria(no_chao, 1.0 - ceu_do_chao_tapado(no_chao)), cf);
}
";

const FECHA_REFLEXO: &str = "    if (b.y <= 0.0) {
        return 0.0;
    }
    return b.x / b.y;
";

/// A porta da visibilidade do céu do chão (`ceu_chao.wgsl`): `(V × validade, validade)` filtradas,
/// `(1, 1)` sem chão ou fora do quadro. Dois leitores, duas leituras do DENTRO de uma peça pousada:
/// - `ceu_do_chao` (a sombra do chão): `V` das vizinhas válidas, `1` dentro — a peça tapa esse chão;
/// - `ceu_do_chao_tapado` (esta lei): o chão debaixo de uma peça encostada está ÀS ESCURAS, `0`.
///   ⛔ Lido como `1`, o reflexo do cromo via a base das vizinhas ACESA (report do dono, 03/10).
const CEU_DO_CHAO: &str = r"fn ceu_do_chao_cru(p: vec3<f32>) -> vec2<f32> {
    if (quadro.sombra.w < 0.5) {
        return vec2<f32>(1.0);
    }
    let c = quadro.ceu_vp * vec4<f32>(p, 1.0);
    let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return vec2<f32>(1.0);
    }
    return textureSampleLevel(ceu_chao, liso, uv, 0.0).rg;
}

fn ceu_do_chao(p: vec3<f32>) -> f32 {
    let s = ceu_do_chao_cru(p);
    return select(1.0, s.x / s.y, s.y > 1.0e-3);
}

fn ceu_do_chao_tapado(p: vec3<f32>) -> f32 {
    return ceu_do_chao_cru(p).x;
}
";

const RAIO: &str = r"fn chao_raio(p: vec3<f32>, n: vec3<f32>, h: f32, d: vec3<f32>) -> vec2<f32> {
    let w = dot(n, d);
    if (w <= 0.0) {
        return vec2<f32>(0.0);
    }
    let x = p + d * (h / -d.y);
    return vec2<f32>(w * (1.0 - ceu_do_chao_tapado(vec3<f32>(x.x, quadro.chao.x, x.z))), w);
}
";

const ABRE: &str = "    let baixo = clamp((1.0 - n.y) * 0.5, 0.0, 1.0);
    if (baixo < 1.0e-4) {
        return 0.0;
    }
    let h = max(p.y - quadro.chao.x, 0.0);
    var a = vec2<f32>(0.0);
";

const FECHA: &str = "    if (a.y <= 0.0) {
        return 0.0;
    }
    return baixo * a.x / a.y;
";
