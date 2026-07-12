//! Primitivas de forma (ADR-0108): retângulo / elipse / polígono / estrela /
//! espiral / round-rect, e a cena-demo. Extraído de `lib.rs` para respeitar o teto
//! de 700 LOC de produção. Todas devolvem um [`VecPath`] **sem estilo** — quem
//! chama aplica fill/stroke.

use crate::{Paint, Rgba8, StrokeSpec, VecPath, VecScene, VecVertex, VertexKind};

/// Escala da cena-demo em world-units. A câmera-default enquadra uma região
/// pequena → 1% do tamanho original (Enio smoke 2026-07-05: "objetos gigantes,
/// canvas todo azul"). Um knob só, trivial de re-tunar.
const DEMO_SCALE: f64 = 0.01;

/// Constante canônica do círculo-Bézier (`k = r·0.55228…`): o comprimento de
/// handle que aproxima um quarto de círculo com uma cúbica. Não é constante
/// inventada — é o valor publicado (4/3·(√2−1)).
const KAPPA: f64 = 0.552_284_75;

/// Teto de lados de um polígono regular (defensivo contra `sides` absurdo vindo
/// da UI; o slider real fica em 3..12). Independente do `MAX_POLYGON_SIDES=128`
/// congelado do modelo *antigo* (`ph2d-vector-doc`); aqui é só um clamp local.
pub const MAX_POLYGON_SIDES: u32 = 128;

/// Retângulo eixo-alinhado a partir de dois cantos opostos (ordem livre): quatro
/// vértices de quina, fechado, **sem estilo** (o chamador aplica fill/stroke).
/// Base das ferramentas de forma (ADR-0108 Fase 1).
#[must_use]
pub fn rectangle(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
    let (y0, y1) = (a[1].min(b[1]), a[1].max(b[1]));
    VecPath {
        verts: vec![
            VecVertex::corner([x0, y0]),
            VecVertex::corner([x1, y0]),
            VecVertex::corner([x1, y1]),
            VecVertex::corner([x0, y1]),
        ],
        closed: true,
        ..VecPath::default()
    }
}

/// Elipse centrada em `center` com semi-eixos `rx`/`ry`: quatro vértices suaves
/// com os handles canônicos de círculo-Bézier ([`KAPPA`]). Fechada, sem estilo.
#[must_use]
pub fn ellipse(center: [f64; 2], rx: f64, ry: f64) -> VecPath {
    let (cx, cy) = (center[0], center[1]);
    let (kx, ky) = (rx * KAPPA, ry * KAPPA);
    let v = |ax: f64, ay: f64, ix: f64, iy: f64, ox: f64, oy: f64| {
        VecVertex::smooth([cx + ax, cy + ay], [cx + ix, cy + iy], [cx + ox, cy + oy])
    };
    VecPath {
        verts: vec![
            v(rx, 0.0, rx, -ky, rx, ky),
            v(0.0, ry, kx, ry, -kx, ry),
            v(-rx, 0.0, -rx, ky, -rx, -ky),
            v(0.0, -ry, -kx, -ry, kx, -ry),
        ],
        closed: true,
        ..VecPath::default()
    }
}

/// Polígono regular inscrito na elipse de raios `rx`/`ry`: `sides` vértices de
/// quina (arestas retas), primeiro vértice no topo (12h). `sides` clampado a
/// `[3, MAX_POLYGON_SIDES]`. Fechado, sem estilo. Usa cos/sin (geometria de
/// editor — não é sim determinística; kurbo/vello já usam trig internamente).
#[must_use]
pub fn regular_polygon(center: [f64; 2], rx: f64, ry: f64, sides: u32) -> VecPath {
    let n = sides.clamp(3, MAX_POLYGON_SIDES) as usize;
    let (cx, cy) = (center[0], center[1]);
    let step = std::f64::consts::TAU / n as f64;
    // A primeira ponta fica em 12h. O mundo é **Y-para-CIMA** (a câmera inverte na tela),
    // então 12h é `+π/2` — com `−π/2` o triângulo nascia apontando para BAIXO.
    let start = std::f64::consts::FRAC_PI_2;
    let verts = (0..n)
        .map(|i| {
            let a = start + step * i as f64;
            VecVertex::corner([cx + rx * a.cos(), cy + ry * a.sin()])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// Polígono regular com **canto redondo**: o mesmo de [`regular_polygon`], com cada
/// quina arredondada por `corner_radius` (unidades de MUNDO). `0` ⇒ idêntico ao
/// polígono de quinas vivas. O raio satura em meia-aresta (ver [`crate::corners`]).
#[must_use]
pub fn regular_polygon_rounded(
    center: [f64; 2],
    rx: f64,
    ry: f64,
    sides: u32,
    corner_radius: f64,
) -> VecPath {
    let base = regular_polygon(center, rx, ry, sides);
    if corner_radius <= 0.0 {
        return base;
    }
    let pts: Vec<[f64; 2]> = base.verts.iter().map(|v| v.anchor).collect();
    let radii = vec![corner_radius; pts.len()];
    crate::corners::round_closed_corners(&pts, &radii)
}

/// Teto de pontas de uma estrela (clamp defensivo; o slider real fica em 3..12).
pub const MAX_STAR_POINTS: u32 = 60;

/// Estrela de `points` pontas inscrita na elipse de raios `rx`/`ry`: `2·points`
/// vértices de quina alternando raio externo (`rx`,`ry`) e interno
/// (`rx·inner_ratio`,`ry·inner_ratio`), primeira ponta no topo. `points` clampado
/// a `[3, MAX_STAR_POINTS]`, `inner_ratio` a `[0.05, 0.95]`. Fechada, sem estilo.
#[must_use]
pub fn star(center: [f64; 2], rx: f64, ry: f64, points: u32, inner_ratio: f64) -> VecPath {
    let n = points.clamp(3, MAX_STAR_POINTS) as usize;
    let ratio = inner_ratio.clamp(0.05, 0.95);
    let (cx, cy) = (center[0], center[1]);
    let step = std::f64::consts::PI / n as f64; // meio passo (2n vértices em 2π)
    let start = std::f64::consts::FRAC_PI_2; // 12h no mundo Y-para-cima
    let verts = (0..2 * n)
        .map(|i| {
            let a = start + step * i as f64;
            let (sx, sy) = if i % 2 == 0 {
                (rx, ry)
            } else {
                (rx * ratio, ry * ratio)
            };
            VecVertex::corner([cx + sx * a.cos(), cy + sy * a.sin()])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// Estrela com as quinas arredondadas por DOIS raios independentes (unidades de
/// MUNDO): `outer_radius` nas **pontas** (quinas convexas, os vértices de raio externo)
/// e `inner_radius` nos **vales** (côncavas). Ambos `0` ⇒ idêntica a [`star`]. Cada
/// raio satura em meia-aresta (ver [`crate::corners`]), então nem uma estrela fina
/// consegue inverter.
#[must_use]
pub fn star_rounded(
    center: [f64; 2],
    rx: f64,
    ry: f64,
    points: u32,
    inner_ratio: f64,
    outer_radius: f64,
    inner_radius: f64,
) -> VecPath {
    let base = star(center, rx, ry, points, inner_ratio);
    if outer_radius <= 0.0 && inner_radius <= 0.0 {
        return base;
    }
    let pts: Vec<[f64; 2]> = base.verts.iter().map(|v| v.anchor).collect();
    // `star` alterna ponta (índice par) e vale (ímpar) — o raio segue a mesma paridade.
    let radii: Vec<f64> = (0..pts.len())
        .map(|i| {
            if i % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            }
        })
        .collect();
    crate::corners::round_closed_corners(&pts, &radii)
}

/// Teto de voltas de uma espiral (clamp defensivo; o slider real fica em 1..8).
pub const MAX_SPIRAL_TURNS: u32 = 8;

/// Espiral de Arquimedes ABERTA inscrita na elipse de raios `rx`/`ry`, com `turns` voltas
/// (clampado a `[1, MAX_SPIRAL_TURNS]`). Cresce do centro (`f = 0`) até a borda (`f = 1`),
/// primeira ponta no topo. Aberta, sem estilo.
///
/// **Cúbicas exatas, não amostragem.** A espiral era 24 vértices de QUINA por volta — um
/// polígono disfarçado, que aparece a olho nu quando a forma é grande (o mesmo defeito da
/// seta curvada). Aqui cada quarto de volta é uma cúbica de Hermite cujos handles carregam
/// a **tangente analítica** da espiral,
///
/// ```text
/// p(θ) = (cx + rx·f·cos θ,  cy + ry·f·sin θ),   f = (θ − start)/total
/// p'(θ) = (rx·(f'·cos θ − f·sin θ),  ry·(f'·sin θ + f·cos θ)),   f' = 1/total
/// ```
///
/// com o handle a `Δθ/3` da tangente (a conversão Hermite→Bézier). Quatro âncoras por
/// volta em vez de 24, curva lisa, e o path continua editável ponto a ponto.
#[must_use]
pub fn spiral(center: [f64; 2], rx: f64, ry: f64, turns: u32) -> VecPath {
    let t = turns.clamp(1, MAX_SPIRAL_TURNS);
    let (cx, cy) = (center[0], center[1]);
    let total = std::f64::consts::TAU * f64::from(t);
    let start = std::f64::consts::FRAC_PI_2; // 12h no mundo Y-para-cima
    // Oito cúbicas por volta. O erro se concentra no MIOLO (a espiral se enrola mais
    // rápido perto do centro): a 4/volta ele é 0,021 no raio 5, a 8/volta cai para 0,0024
    // — sub-pixel — e ainda assim são um terço dos 24 vértices do polígono antigo.
    let steps = t as usize * 8;
    let step = total / steps as f64;

    let verts = (0..=steps)
        .map(|i| {
            let f = i as f64 / steps as f64; // fração do raio E do ângulo
            let a = start + total * f;
            let (s, c) = a.sin_cos();
            let anchor = [cx + rx * f * c, cy + ry * f * s];
            // Tangente analítica × o comprimento de handle do ARCO. Não é o `Δθ/3` do
            // Hermite ingênuo: para um quarto de volta ele fica 1,5% curto (é justamente
            // a diferença entre `π/6 = 0,5236` e o `KAPPA = 0,5523`), e a espiral sai
            // visivelmente "murcha" para dentro. `(4/3)·tan(Δθ/4)` é exato no círculo e
            // degenera certo aqui, onde |p'| ≈ raio.
            let df = 1.0 / total;
            let h = (4.0 / 3.0) * (step / 4.0).tan();
            let (tx, ty) = (rx * (df * c - f * s) * h, ry * (df * s + f * c) * h);
            VecVertex::smooth(
                anchor,
                [anchor[0] - tx, anchor[1] - ty],
                [anchor[0] + tx, anchor[1] + ty],
            )
        })
        .collect();
    VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    }
}

/// Retângulo de cantos arredondados a partir de dois cantos opostos + raio
/// `radius` (world-units), clampado a metade do menor lado. Oito vértices de
/// quina: arestas retas + quartos-de-círculo (handles `KAPPA`). `radius ≈ 0` →
/// [`rectangle`]. Fechado, sem estilo.
#[must_use]
pub fn rounded_rect(a: [f64; 2], b: [f64; 2], radius: f64) -> VecPath {
    let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
    let (y0, y1) = (a[1].min(b[1]), a[1].max(b[1]));
    let r = radius.max(0.0).min((x1 - x0) * 0.5).min((y1 - y0) * 0.5);
    if r < 1e-9 {
        return rectangle(a, b);
    }
    let k = r * KAPPA;
    // 8 âncoras (sentido horário a partir da aresta de cima) com o handle do arco
    // no lado curvo e handle nulo (na âncora) no lado reto → quinas independentes.
    let corner = |anchor: [f64; 2], in_h: [f64; 2], out_h: [f64; 2]| VecVertex {
        anchor,
        in_handle: in_h,
        out_handle: out_h,
        kind: VertexKind::Corner,
    };
    let verts = vec![
        // v1: fim do arco sup-esq, início da aresta de cima.
        corner([x0 + r, y0], [x0 + r - k, y0], [x0 + r, y0]),
        // v2: fim da aresta de cima, início do arco sup-dir.
        corner([x1 - r, y0], [x1 - r, y0], [x1 - r + k, y0]),
        // v3: fim do arco sup-dir, início da aresta direita.
        corner([x1, y0 + r], [x1, y0 + r - k], [x1, y0 + r]),
        // v4: fim da aresta direita, início do arco inf-dir.
        corner([x1, y1 - r], [x1, y1 - r], [x1, y1 - r + k]),
        // v5: fim do arco inf-dir, início da aresta de baixo.
        corner([x1 - r, y1], [x1 - r + k, y1], [x1 - r, y1]),
        // v6: fim da aresta de baixo, início do arco inf-esq.
        corner([x0 + r, y1], [x0 + r, y1], [x0 + r - k, y1]),
        // v7: fim do arco inf-esq, início da aresta esquerda.
        corner([x0, y1 - r], [x0, y1 - r + k], [x0, y1 - r]),
        // v8: fim da aresta esquerda, início do arco sup-esq.
        corner([x0, y0 + r], [x0, y0 + r], [x0, y0 + r - k]),
    ];
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// Segmento reto de `a` a `b`: dois vértices de quina, **aberto** (sem fill — uma
/// linha não tem interior). A primitiva mais básica de um editor vetorial.
#[must_use]
pub fn line(a: [f64; 2], b: [f64; 2]) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner(a), VecVertex::corner(b)],
        closed: false,
        ..VecPath::default()
    }
}

/// Teto de graus de um arco (uma volta inteira). O slider real fica em 1..360.
pub const MAX_ARC_DEGREES: f64 = 360.0;

/// Arco de elipse centrado em `center` (semi-eixos `rx`/`ry`), abrindo `degrees`
/// graus a partir das 3h (0°), sentido anti-horário. **Aberto**, vértices suaves com
/// handles bézier tangentes ao círculo — liso ao renderizar e editável ponto a ponto.
///
/// Divide o arco em segmentos de ≤90° e usa o comprimento de handle exato de cada
/// segmento (`(4/3)·tan(α/4)·r`), a generalização de [`KAPPA`] (que é esse valor
/// para α=90°). `degrees` clampado a `[1, 360]`. Trig de geometria de editor (não é
/// sim determinística — vello já usa trig internamente), como [`regular_polygon`].
#[must_use]
pub fn arc(center: [f64; 2], rx: f64, ry: f64, degrees: f64) -> VecPath {
    let (cx, cy) = (center[0], center[1]);
    let total = degrees.clamp(1.0, MAX_ARC_DEGREES).to_radians();
    // Segmentos de no máximo 90° (π/2) para o bézier aproximar bem.
    let n_seg = (total / std::f64::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let seg = total / n_seg as f64;
    // Comprimento de handle (em fração do raio) que faz uma cúbica seguir o arco de
    // ângulo `seg`: (4/3)·tan(seg/4). Para seg=π/2 isto é exatamente KAPPA.
    let h = (4.0 / 3.0) * (seg / 4.0).tan();
    let mut verts = Vec::with_capacity(n_seg + 1);
    for i in 0..=n_seg {
        let a = seg * i as f64;
        let (s, c) = a.sin_cos();
        let anchor = [cx + rx * c, cy + ry * s];
        // Tangente ao arco em `a` é (−sin, cos); o handle é ela × h × raio.
        let (tx, ty) = (-rx * s * h, ry * c * h);
        verts.push(VecVertex::smooth(
            anchor,
            [anchor[0] - tx, anchor[1] - ty],
            [anchor[0] + tx, anchor[1] + ty],
        ));
    }
    VecPath {
        verts,
        closed: false,
        ..VecPath::default()
    }
}

/// Blob-círculo preenchido (usa [`ellipse`] com `rx = ry`), para a cena-demo.
pub(crate) fn blob(c: [f64; 2], r: f64, fill: Rgba8) -> VecPath {
    let mut p = ellipse(c, r, r);
    p.fill = Some(Paint::solid(fill));
    p
}

fn demo_blob() -> VecPath {
    blob(
        [0.0, 0.0],
        120.0 * DEMO_SCALE,
        Rgba8::new(90, 150, 230, 255),
    )
}

/// Arco aberto (uma cúbica), traçado claro — prova o caminho de stroke. Largura
/// proporcional ao raio (30%) para ficar visível em qualquer `DEMO_SCALE`.
fn demo_curve() -> VecPath {
    let p = |x: f64, y: f64| [x * DEMO_SCALE, y * DEMO_SCALE];
    let width = 120.0 * DEMO_SCALE * 0.3;
    VecPath {
        verts: vec![
            VecVertex {
                anchor: p(-160.0, -150.0),
                in_handle: p(-160.0, -150.0),
                out_handle: p(-40.0, -280.0),
                kind: VertexKind::Corner,
            },
            VecVertex {
                anchor: p(160.0, -150.0),
                in_handle: p(40.0, -280.0),
                out_handle: p(160.0, -150.0),
                kind: VertexKind::Corner,
            },
        ],
        closed: false,
        stroke: Some(StrokeSpec::new(Rgba8::new(240, 240, 245, 255), width)),
        ..VecPath::default()
    }
}

impl VecScene {
    /// Cena de demonstração da Fase 0: um blob fechado **preenchido** + uma curva
    /// aberta **traçada** — prova fill + stroke + curvatura Bézier ponta-a-ponta
    /// pela pipeline nova. Sai de cena quando as ferramentas de desenho entrarem
    /// (Fase 1); não é conteúdo persistido.
    pub fn demo() -> Self {
        let mut scene = Self::new();
        scene.push_path(demo_blob());
        scene.push_path(demo_curve());
        scene
    }

    /// Spike de escala (ADR-0108 §5): `n` blobs numa grade quadrada. Cada frame o
    /// dispatch re-encoda TUDO (sem dirty-tracking ainda) → mede o custo de
    /// re-encode **naive** e fixa o N do kill-criterion. Não é conteúdo real
    /// (dirigido por `PH2D_VEC_DEMO_N` no shell).
    pub fn demo_grid(n: usize) -> Self {
        let mut scene = Self::new();
        if n == 0 {
            return scene;
        }
        let cols = (n as f64).sqrt().ceil() as usize;
        // Empacota a grade num quadrado que CABE na viewport-default (~±4.8 world,
        // aferido no smoke) → todos os N ficam visíveis (a grade fica mais densa
        // conforme N sobe) e o teste de GPU é justo (nada é culled off-screen).
        let extent = 7.0_f64;
        let spacing = extent / cols as f64;
        let r = spacing * 0.4; // raio proporcional ao passo → sem overlap
        let half = extent * 0.5 - spacing * 0.5;
        for i in 0..n {
            let cx = (i % cols) as f64 * spacing - half;
            let cy = (i / cols) as f64 * spacing - half;
            scene.push_path(blob([cx, cy], r, Rgba8::new(90, 150, 230, 255)));
        }
        scene
    }
}
