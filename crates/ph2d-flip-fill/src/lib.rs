#![forbid(unsafe_code)]
//! `ph2d-flip-fill` — **o balde do Flip** (ADR-0114 W4), clean-room do pixel solver do
//! Grease Pencil 5.2 (`docs/Flip/02 §6`), com os upgrades decididos na `04 §3`.
//!
//! Entra: a geometria das linhas + o ponto clicado. Sai: **geometria** — o contorno da
//! região, com os buracos. É essa a promessa do GP: o preenchimento é um traço comum,
//! então selecionar, mover, apagar, animar e *tweenar* um fill é a mesma coisa que
//! fazer isso com um traço, sem sistema paralelo nenhum.
//!
//! O pipeline (cada etapa num módulo, cada uma testável sozinha):
//!
//! ```text
//! linhas + clique
//!   → gap::closures         fecha os vãos (pontas + quinas), cortando na colisão
//!   → raster::Grid          rasteriza as fronteiras a MEIA espessura (radius_scale 0.5)
//!   → raster::flood         span fill 4-conexo + filtro de vazamento CRUZADO
//!   → raster::expand_under_ink  a cor entra por baixo da linha e PARA na silhueta dela
//!   → raster::grow          Grow/Shrink do usuário (ajuste fino; default 0)
//!   → trace::trace_contours marching squares — os buracos saem de graça
//!   → trace::simplify_ring  RDP + binomial leve
//!   = FillResult { outer, holes, closures }
//! ```
//!
//! **CPU, de propósito.** O fill é uma operação de CLIQUE, não de frame: o span fill é
//! ~10× o BFS por pixel e roda em poucos ms num buffer de milhões de pixels. A GPU
//! seria o primitivo errado — o JFA **salta paredes** (não é geodésico), e o readback
//! para vetorizar o contorno seria inevitável de qualquer jeito (`04 §3`).
//!
//! HR-5: zero transcendentais fora de `sqrt` (que é exata em IEEE-754).

mod gap;
mod raster;
mod trace;

pub use gap::{Boundary, Closure};
pub use raster::Grid;
pub use trace::{RDP_EPSILON_PX, signed_area, simplify_ring, trace_contours};

use ph2d_core::Vec2;

/// Margem, em pixels, entre a arte e a borda da grade. Precisa existir: sem ela, uma
/// linha encostada na borda da bbox faria o flood "vazar" pela lateral da grade e o
/// fill seria recusado sem motivo.
const MARGIN_PX: usize = 20;
/// Teto do lado da grade (um clique num documento gigante não pode alocar um giga).
/// Ao bater no teto, a resolução efetiva cai — o fill fica mais grosseiro, não quebra.
const MAX_SIDE: usize = 4096;
/// O filtro de vazamento cruzado, em pixels (o valor do GP).
const LEAK_PX: usize = 3;
/// Piso da resolução da grade: só impede um `scale` zero/negativo. **Não existe TETO** —
/// ver o comentário no `fill_at` (um teto em unidades de documento é um teto na unidade
/// errada, e foi ele que fez o zoom estragar o balde).
const MIN_SCALE: f32 = 1e-3; // CLAMP-OK: guarda contra scale degenerado

/// Como o balde trata o que já está pintado (`04 §3` — a semântica de balde de
/// ANIMAÇÃO, do Toon Boom).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum FillMode {
    /// Preenche a região (o balde de sempre).
    #[default]
    Paint,
    /// Preenche **por baixo** do que já está pintado — o fluxo de colorir sem tocar na
    /// linha nem nos fills anteriores.
    PaintBehind,
    /// **Remove** o preenchimento que estiver sob o clique.
    Unpaint,
}

/// Os parâmetros do balde (o que o painel expõe).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FillParams {
    /// Resolução do buffer, em pixels por unidade do documento. É o **Precision** do
    /// painel: mais alto = contorno mais fiel, buffer maior.
    pub precision: f32,
    /// Alcance do **Gap Closure** (unidades do documento). `0` = desligado.
    pub gap_reach: f32,
    /// **Grow/Shrink**, em pixels do BUFFER — medido da borda onde a cor **começa a
    /// aparecer** (a borda INTERNA do traço), não da silhueta externa dele. É essa âncora
    /// que torna o efeito VISÍVEL independente da espessura da linha:
    ///
    /// - `0` → a cor entra por baixo da linha e para na silhueta: sem vão, sem transbordo,
    ///   em qualquer espessura;
    /// - `< 0` → a cor recua: um vão visível de exatamente `|grow|` px, numa linha de 1 px
    ///   ou de 40;
    /// - `> 0` → a cor sangra `grow` px além da linha (o "off-register" da animação 2D).
    ///
    /// O chamador converte de px de tela para px de buffer (multiplicando pela
    /// `precision`) — senão subir a Precision encolheria o Grow em silêncio.
    pub grow: i32,
    /// O modo (Paint / Paint-Behind / Unpaint).
    pub mode: FillMode,
}

impl Default for FillParams {
    fn default() -> Self {
        Self {
            precision: 4.0,
            gap_reach: 0.0,
            // **Zero, e não +2.** A fronteira é rasterizada a um QUARTO da espessura
            // (`radius_scale = 0.5` sobre a meia-espessura), então o preenchimento já
            // nasce por baixo do corpo da linha — não há halo para matar. O +2 do 1º
            // corte empurrava a cor 1px para FORA do traço, que é exatamente o defeito
            // que ele dizia estar evitando. Medido: com grow 0, a borda do fill fica
            // 1,8px DENTRO da borda externa de uma linha de 6px.
            grow: 0,
            mode: FillMode::Paint,
        }
    }
}

/// O resultado: a região preenchida, como geometria.
#[derive(Clone, Debug, PartialEq)]
pub struct FillResult {
    /// O contorno externo (fechado), em coordenadas do documento.
    pub outer: Vec<Vec2>,
    /// Os buracos (anéis fechados) subtraídos dele.
    pub holes: Vec<Vec<Vec2>>,
    /// Os fechamentos de gap que a solução USOU — o chamador os materializa como traços
    /// **invisíveis persistentes** (o twist do Harmony): assim o vão fica fechado para
    /// sempre, e o re-fill (outra cor, outro quadro, amanhã) não depende de a ferramenta
    /// estar com os mesmos parâmetros.
    pub closures: Vec<Closure>,
}

/// Por que um fill não aconteceu (o chamador vira um toast, em vez de pintar o mundo).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FillError {
    /// O clique caiu EM CIMA de uma linha.
    OnBoundary,
    /// A região é aberta: o preenchimento escapou para o "oceano". É o "No fill
    /// created" do GP — e é a hora de sugerir o **Gap Closure**.
    Leaked,
    /// Não há linha nenhuma para delimitar coisa alguma.
    Empty,
    /// O contorno resultante é degenerado (área ~zero).
    Degenerate,
}

/// **Preenche** a região que contém `click`, delimitada por `strokes`.
///
/// `strokes` são as polilinhas de fronteira **com a espessura visual de cada ponto**
/// (`(pontos, meia-espessura por ponto, fechado)`), no espaço do documento. A
/// meia-espessura entra a **50%** no raster (`radius_scale`) — ver
/// [`Grid::stroke_capsule`], que é onde essa decisão está explicada.
pub fn fill_at(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    click: Vec2,
    params: FillParams,
) -> Result<FillResult, FillError> {
    if strokes.is_empty() {
        return Err(FillError::Empty);
    }
    // 1. Gap Closure: as extensões que fecham os vãos (já cortadas na colisão).
    let bounds: Vec<Boundary<'_>> = strokes
        .iter()
        .map(|(p, _, c)| Boundary {
            points: p,
            closed: *c,
        })
        .collect();
    let closures = gap::closures(&bounds, params.gap_reach);

    // 2. A grade: bbox de tudo (linhas + clique + fechamentos), com margem.
    let (mut lo, mut hi) = (click, click);
    for (pts, w, _) in strokes {
        for (i, p) in pts.iter().enumerate() {
            let r = w.get(i).copied().unwrap_or(0.0);
            lo = Vec2::new(lo.x.min(p.x - r), lo.y.min(p.y - r));
            hi = Vec2::new(hi.x.max(p.x + r), hi.y.max(p.y + r));
        }
    }
    // **NÃO se capeia a resolução aqui.** O 1º corte fazia `clamp(0.5, 64.0)` — um teto
    // em "pixels de buffer por unidade de DOCUMENTO". Só que unidade de documento não é
    // uma medida fixa: o desenho vive em unidades de mundo, e com a câmera aproximada uma
    // forma de 500 px de tela ocupa POUCAS unidades. O teto então cortava a resolução em
    // pedaços — 1 px de buffer virava 5 px de TELA — e como o `grow` e a tolerância do RDP
    // vivem em pixels de buffer, o preenchimento inchava para fora da linha (12 px!) e o
    // contorno virava um polígono de 17 lados. *Dar zoom estragava o balde.*
    //
    // Quem limita a memória é o `MAX_SIDE` do `Grid::new` — e ele cede RESOLUÇÃO, não
    // cobertura. O `scale` aqui só não pode ser zero.
    let scale = params.precision.max(MIN_SCALE);
    let mut grid = Grid::new(lo, hi, scale, MARGIN_PX, MAX_SIDE);

    // 3. Duas rasterizações, com papéis DIFERENTES:
    //    - `BOUNDARY` = a **parede** do flood, a meia espessura (`radius_scale = 0.5`):
    //      cai dentro do corpo da linha, então o flood já para por baixo dela;
    //    - `INK` = a **silhueta visual** do traço, espessura cheia: o mapa de até onde a
    //      cor pode avançar ESCONDIDA. É ele que substitui o chute do `grow` (passo 5).
    let mut max_ink_px: f32 = 0.0;
    for (pts, w, closed) in strokes {
        let n = pts.len();
        if n < 2 {
            continue;
        }
        let last = if *closed { n } else { n - 1 };
        for i in 0..last {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            let ra = w.get(i).copied().unwrap_or(0.0);
            let rb = w.get((i + 1) % n).copied().unwrap_or(0.0);
            let half_px = 0.5 * (ra + rb) * scale; // a meia-espessura VISUAL, em px
            max_ink_px = max_ink_px.max(half_px);
            grid.ink_capsule(a, b, half_px); // a silhueta
            grid.stroke_capsule(a, b, 0.5 * half_px); // a parede (metade dela)
        }
    }
    for c in &closures {
        grid.stroke_capsule(c.a, c.b, 0.0); // o fechamento é fino (1px) e é só parede
    }

    // 4. O flood, a partir do clique.
    let Some(seed) = grid.pixel_of(click) else {
        return Err(FillError::Empty);
    };
    if grid.at(seed.0, seed.1) & raster::BOUNDARY != 0 {
        return Err(FillError::OnBoundary);
    }
    if !grid.flood(seed, LEAK_PX) {
        return Err(FillError::Leaked);
    }

    // 5+6. **O Grow, medido de onde a cor COMEÇA A APARECER.**
    //
    // A âncora é tudo aqui. Ancorado na borda EXTERNA do traço (o 1º corte), o mesmo
    // `grow = -8` abria 8,6 px de vão numa linha de 1 px e vão NENHUM numa de 40 — os
    // primeiros 40 px de recuo eram gastos por baixo do traço, onde ninguém vê. Um
    // controle assim exige um ajuste diferente para cada espessura, que é justamente o
    // que um controle não pode exigir.
    //
    // Ancorando na borda INTERNA — a fronteira do que se VÊ — os dois lados do slider
    // ficam independentes da espessura:
    //
    //   grow  = 0  → a cor entra por baixo da linha e para na silhueta dela.
    //                Sem vão, sem transbordo, em qualquer espessura. (O default.)
    //   grow  < 0  → a cor RECUA |grow| px da borda interna: um vão visível de exatamente
    //                |grow| px, seja a linha de 1 px ou de 40.
    //   grow  > 0  → a cor SANGRA |grow| px além da silhueta: um transbordo deliberado de
    //                exatamente |grow| px (o "off-register" da animação 2D).
    let passes = (max_ink_px.ceil() as usize).saturating_add(2);
    if params.grow >= 0 {
        // A cor cobre exatamente o que a linha esconde — a expansão é limitada pela
        // própria TINTA, então nenhuma constante é necessária.
        grid.expand_under_ink(passes);
        if params.grow > 0 {
            grid.grow(params.grow); // e sangra além dela
        }
    } else {
        // Descola da linha primeiro (a cor para onde ela começa a aparecer), e SÓ ENTÃO
        // recua — senão o recuo seria comido pela espessura do traço.
        grid.strip_ink();
        grid.grow(params.grow);
    }

    // 6. Vetoriza. O contorno de MAIOR área é o externo; os demais são candidatos a
    //    buraco. A tolerância do RDP é em px do BUFFER, então usa a resolução EFETIVA
    //    da grade (que pode ter cedido ao teto de tamanho), não a que se pediu.
    let eps = RDP_EPSILON_PX / grid.scale;
    let mut rings: Vec<Vec<Vec2>> = trace_contours(&grid)
        .into_iter()
        .map(|r| simplify_ring(&r, eps, 2))
        .filter(|r| r.len() >= 3)
        .collect();
    if rings.is_empty() {
        return Err(FillError::Degenerate);
    }
    // Desempate por área E por posição: a ordenação é total (determinismo, HR-5).
    rings.sort_by(|a, b| {
        signed_area(b)
            .abs()
            .total_cmp(&signed_area(a).abs())
            .then(a[0].x.total_cmp(&b[0].x))
            .then(a[0].y.total_cmp(&b[0].y))
    });
    let outer = rings.remove(0);
    let outer_area = signed_area(&outer);
    if outer_area.abs() < 1e-6 {
        return Err(FillError::Degenerate);
    }
    // **Buraco é ORIENTAÇÃO, não tamanho.** Classificar "todo anel que não é o maior é
    // buraco" estava errado: um `grow` negativo pode PARTIR a região em componentes
    // desconexos (um halter erodido no meio), e a segunda ilha — que é área PREENCHIDA —
    // virava um buraco a ser subtraído da primeira.
    //
    // O traçado anda com o preenchido à esquerda, então o contorno externo de um blob e
    // a borda de um furo têm sinais OPOSTOS, e uma ilha tem o MESMO sinal do externo.
    // Só os de sinal oposto são furos; as ilhas soltas ficam de fora do resultado (o
    // preenchimento é a região sob o CLIQUE, e o clique está numa só).
    let holes: Vec<Vec<Vec2>> = rings
        .into_iter()
        .filter(|r| signed_area(r).signum() != outer_area.signum())
        .collect();
    Ok(FillResult {
        outer,
        holes,
        closures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Um quadrado de linha, com meia-espessura `w`.
    fn square(a: f32, b: f32, w: f32) -> (Vec<Vec2>, Vec<f32>, bool) {
        (
            vec![
                Vec2::new(a, a),
                Vec2::new(b, a),
                Vec2::new(b, b),
                Vec2::new(a, b),
            ],
            vec![w; 4],
            true,
        )
    }

    /// O caminho feliz: clicar dentro de uma forma fechada devolve o contorno dela.
    #[test]
    fn clicking_inside_a_closed_shape_returns_its_contour() {
        let strokes = [square(0.0, 20.0, 0.3)];
        let r = fill_at(&strokes, Vec2::new(10.0, 10.0), FillParams::default())
            .expect("um quadrado fechado tem de preencher");
        assert!(r.holes.is_empty());
        let area = signed_area(&r.outer).abs();
        // ~20×20 = 400, menos a linha, mais o grow. A ordem de grandeza é o que importa.
        assert!(
            (300.0..=450.0).contains(&area),
            "a área bate com a forma: {area}"
        );
    }

    /// **A promessa dos buracos:** a letra "O". Clicar entre os dois quadrados devolve
    /// o externo E o furo — e é por isso que o `FlipStroke` tem `holes`.
    #[test]
    fn a_donut_returns_the_outer_ring_and_the_hole() {
        let strokes = [square(0.0, 30.0, 0.3), square(10.0, 20.0, 0.3)];
        let r = fill_at(&strokes, Vec2::new(3.0, 3.0), FillParams::default())
            .expect("a rosquinha tem de preencher");
        assert_eq!(r.holes.len(), 1, "um furo");
        let outer = signed_area(&r.outer).abs();
        let hole = signed_area(&r.holes[0]).abs();
        assert!(outer > 700.0, "o externo é o quadrado de 30: {outer}");
        assert!(
            (60.0..=140.0).contains(&hole),
            "o furo é o quadrado de 10: {hole}"
        );
    }

    /// **Uma forma ABERTA é recusada** — em vez de pintar o documento inteiro. É o
    /// momento em que a UI deve sugerir o Gap Closure.
    #[test]
    fn an_open_shape_is_refused_not_flooded() {
        // Um "C": três lados de um quadrado.
        let c = (
            vec![
                Vec2::new(20.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 20.0),
                Vec2::new(20.0, 20.0),
            ],
            vec![0.3; 4],
            false,
        );
        let err = fill_at(&[c], Vec2::new(10.0, 10.0), FillParams::default()).unwrap_err();
        assert_eq!(err, FillError::Leaked);
    }

    /// **E o Gap Closure fecha o "C"**: com alcance suficiente, as duas pontas se
    /// estendem, colidem, e a região passa a existir. Os fechamentos voltam no
    /// resultado — o chamador os materializa como traços invisíveis.
    #[test]
    fn gap_closure_makes_the_open_shape_fillable() {
        let c = (
            vec![
                Vec2::new(20.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 20.0),
                Vec2::new(20.0, 20.0),
            ],
            vec![0.3; 4],
            false,
        );
        // As duas pontas apontam para +x; elas não se veem. Mas a quina de cada uma…
        // Não: aqui o que fecha é uma parede à direita. Põe uma linha vertical em x=22.
        let wall = (
            vec![Vec2::new(22.0, -2.0), Vec2::new(22.0, 22.0)],
            vec![0.3; 2],
            false,
        );
        let params = FillParams {
            gap_reach: 5.0,
            ..Default::default()
        };
        let r = fill_at(&[c, wall], Vec2::new(10.0, 10.0), params)
            .expect("com Gap Closure, o C fecha contra a parede");
        assert!(
            !r.closures.is_empty(),
            "os fechamentos têm de voltar (viram traços invisíveis)"
        );
        assert!(signed_area(&r.outer).abs() > 300.0);
    }

    /// Clicar em cima da linha não preenche (e diz por quê).
    #[test]
    fn clicking_on_the_line_is_refused() {
        let strokes = [square(0.0, 20.0, 0.5)];
        let err = fill_at(&strokes, Vec2::new(0.0, 10.0), FillParams::default()).unwrap_err();
        assert_eq!(err, FillError::OnBoundary);
    }

    /// **Determinismo (HR-5):** a mesma entrada dá o mesmo contorno, bit a bit.
    #[test]
    fn the_same_click_gives_the_same_geometry() {
        let strokes = [square(0.0, 30.0, 0.3), square(10.0, 20.0, 0.3)];
        let a = fill_at(&strokes, Vec2::new(3.0, 3.0), FillParams::default()).unwrap();
        let b = fill_at(&strokes, Vec2::new(3.0, 3.0), FillParams::default()).unwrap();
        assert_eq!(a, b, "o balde tem de ser determinístico");
    }

    /// Sem linha nenhuma, não há o que preencher.
    #[test]
    fn nothing_to_fill_is_an_error_not_a_panic() {
        assert_eq!(
            fill_at(&[], Vec2::new(0.0, 0.0), FillParams::default()).unwrap_err(),
            FillError::Empty
        );
    }

    /// **O resultado do balde NÃO pode depender do ZOOM da câmera.**
    ///
    /// O desenho vive em unidades de mundo; aproximar a câmera faz a mesma forma ocupar
    /// MENOS unidades. Um teto de resolução em "px de buffer por unidade de documento"
    /// (o `clamp(0.5, 64.0)` do 1º corte) é um teto na unidade ERRADA: com zoom, ele
    /// cortava a resolução em pedaços — 1 px de buffer chegou a valer 5 px de tela — e
    /// como o `grow` e a tolerância do RDP vivem em px de buffer, o preenchimento inchava
    /// 12 px para FORA da linha e o contorno virava um polígono de 17 lados.
    ///
    /// Aqui: o MESMO círculo, visto com quatro zooms. A geometria de saída, medida em
    /// px de TELA, tem de ser a mesma nos quatro.
    #[test]
    fn the_fill_is_invariant_under_camera_zoom() {
        let mut results = Vec::new();
        for height_world in [10.0f32, 5.0, 2.0, 1.0] {
            let px_to_world = height_world / 800.0;
            // Um círculo de 250 px de RAIO na tela, com um traço de 6 px — em unidades
            // de mundo, que encolhem quando a câmera aproxima.
            let r = 250.0 * px_to_world;
            let n = 64;
            let pts: Vec<Vec2> = (0..n)
                .map(|i| {
                    // Polígono regular: sem transcendental no teste (HR-5), a forma exata
                    // não importa — importa ela ser a MESMA em px de tela.
                    let t = i as f32 / n as f32;
                    let (x, y) = unit_circle(t);
                    Vec2::new(r * x, r * y)
                })
                .collect();
            let half = 6.0 * 0.5 * px_to_world; // meia-espessura de um traço de 6 px
            let strokes = vec![(pts, vec![half; n], true)];

            let res = fill_at(
                &strokes,
                Vec2::new(0.0, 0.0),
                FillParams {
                    precision: 1.0 / px_to_world, // 1 px de buffer por px de tela
                    gap_reach: 0.0,
                    grow: 0,
                    mode: FillMode::Paint,
                },
            )
            .expect("o circulo fechado preenche em qualquer zoom");

            // O maior raio do contorno, em px de TELA.
            let max_r_px = res
                .outer
                .iter()
                .map(|p| (p.x * p.x + p.y * p.y).sqrt() / px_to_world)
                .fold(0.0f32, f32::max);
            results.push((height_world, res.outer.len(), max_r_px));
        }
        let (_, n0, r0) = results[0];
        for &(h, n, r) in &results[1..] {
            assert!(
                (r - r0).abs() < 2.0,
                "zoom h={h}: o contorno saiu a {r:.1} px do centro, mas com h=10 saiu a \
                 {r0:.1} px — o balde depende do zoom"
            );
            // A contagem de vértices mede o FACETAMENTO: cair pela metade = polígono.
            assert!(
                n * 2 >= n0,
                "zoom h={h}: o contorno desabou de {n0} para {n} vertices — o buffer \
                 ficou grosseiro e o fill virou um poligono"
            );
        }
        // **A regra, em números.** O traço tem 6 px: a linha VISÍVEL vai de 247 a 253 px
        // do centro. O preenchimento tem de chegar à borda EXTERNA (253) — cobrindo tudo
        // o que a linha esconde, sem deixar um fio claro na curva — e **nunca passar
        // dela** (senão a cor aparece por fora do desenho).
        for &(h, _, r) in &results {
            assert!(
                r <= 253.0 + 1.0,
                "zoom h={h}: o preenchimento vazou {:.1} px para FORA da linha",
                r - 253.0
            );
            assert!(
                r >= 250.0,
                "zoom h={h}: o preenchimento parou a {:.1} px do centro, aquem do corpo da \
                 linha (247..253) — vai abrir um fio claro na curva",
                r
            );
        }
    }

    /// Um ponto do círculo unitário em `t ∈ [0,1)`, sem `sin`/`cos` (HR-5).
    ///
    /// `u = tan(θ/2)` parametriza racionalmente o arco: com `u ∈ [0,1]` sai EXATAMENTE o
    /// 1º quadrante (θ de 0° a 90°), e os outros três saem por rotação de 90°. (A 1ª
    /// versão usava `u ∈ [-1,1]`, que é um SEMICÍRCULO — girado quatro vezes, ele saltava
    /// de (0,1) para (1,0) e o "círculo" tinha uma corda enorme. O solver estava certo; o
    /// teste é que descrevia outra forma.)
    fn unit_circle(t: f32) -> (f32, f32) {
        let q = (t * 4.0).floor() as i32 % 4;
        let u = (t * 4.0).fract(); // [0,1) → o quarto de arco
        let d = 1.0 + u * u;
        let (x, y) = ((1.0 - u * u) / d, 2.0 * u / d);
        match q {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, -y),
            _ => (y, -x),
        }
    }

    /// **A cor para na SILHUETA da linha — em QUALQUER espessura.**
    ///
    /// É a regra que substituiu o chute. Um `grow` cego não sabe onde a linha acaba, e a
    /// quantidade certa depende da espessura LOCAL do traço: o mesmo "+2 px" que fecha o
    /// fio claro de uma linha grossa transborda uma linha de 1 px. Nenhuma constante
    /// serve — por isso a expansão é limitada pela própria TINTA (`INK`).
    ///
    /// Aqui: o mesmo círculo com traços de 1 a 40 px. Em todos, o preenchimento tem de
    /// **alcançar a borda externa** do traço (senão o alisamento do contorno abre um fio
    /// claro na curva) e **não passar dela** (senão a cor aparece por fora do desenho).
    #[test]
    fn the_colour_stops_at_the_ink_silhouette_at_any_line_width() {
        let px_to_world = 10.0f32 / 800.0;
        for width_px in [1.0f32, 2.0, 6.0, 16.0, 40.0] {
            let r = 250.0 * px_to_world;
            let n = 96;
            let pts: Vec<Vec2> = (0..n)
                .map(|i| {
                    let (x, y) = unit_circle(i as f32 / n as f32);
                    Vec2::new(r * x, r * y)
                })
                .collect();
            let half = width_px * 0.5 * px_to_world;
            let strokes = vec![(pts, vec![half; n], true)];

            let res = fill_at(
                &strokes,
                Vec2::new(0.0, 0.0),
                FillParams {
                    precision: 1.0 / px_to_world,
                    gap_reach: 0.0,
                    grow: 0, // NENHUM ajuste: a tinta é que manda
                    mode: FillMode::Paint,
                },
            )
            .expect("o circulo fechado preenche");

            let max_r = res
                .outer
                .iter()
                .map(|p| (p.x * p.x + p.y * p.y).sqrt() / px_to_world)
                .fold(0.0f32, f32::max);
            // A linha VISÍVEL vai de (250 - w/2) a (250 + w/2).
            let outer_edge = 250.0 + width_px * 0.5;
            let inner_edge = 250.0 - width_px * 0.5;
            assert!(
                max_r <= outer_edge + 1.0,
                "linha de {width_px}px: a cor vazou {:.1} px para FORA do traco",
                max_r - outer_edge
            );
            assert!(
                max_r >= inner_edge,
                "linha de {width_px}px: a cor parou a {max_r:.1} px, ANTES do corpo do \
                 traco ({inner_edge:.1}..{outer_edge:.1}) — vai abrir um fio claro na curva"
            );
        }
    }

    /// **O Grow é medido de onde a cor COMEÇA A APARECER** — e por isso o efeito visível
    /// é o mesmo em qualquer espessura de linha.
    ///
    /// Ancorado na borda EXTERNA do traço (o 1º corte), o mesmo `grow = -8` abria 8,6 px
    /// de vão numa linha de 1 px e vão NENHUM numa de 40: os primeiros 40 px de recuo eram
    /// gastos por baixo do traço, onde ninguém vê. O usuário tinha de reajustar o slider a
    /// cada mudança de pincel — que é justamente o que um controle não pode exigir.
    ///
    /// Ancorado na borda INTERNA, `grow = -N` abre um vão de N px, ponto.
    #[test]
    fn a_negative_grow_opens_the_same_visible_gap_at_any_line_width() {
        let px_to_world = 10.0f32 / 800.0;
        for grow_px in [-2i32, -4, -8] {
            let mut gaps = Vec::new();
            for width_px in [1.0f32, 6.0, 16.0, 40.0] {
                let r = 250.0 * px_to_world;
                let n = 96;
                let pts: Vec<Vec2> = (0..n)
                    .map(|i| {
                        let (x, y) = unit_circle(i as f32 / n as f32);
                        Vec2::new(r * x, r * y)
                    })
                    .collect();
                let half = width_px * 0.5 * px_to_world;
                let res = fill_at(
                    &[(pts, vec![half; n], true)],
                    Vec2::new(0.0, 0.0),
                    FillParams {
                        precision: 1.0 / px_to_world,
                        gap_reach: 0.0,
                        grow: grow_px,
                        mode: FillMode::Paint,
                    },
                )
                .expect("preenche");
                let fill_edge = res
                    .outer
                    .iter()
                    .map(|p| (p.x * p.x + p.y * p.y).sqrt() / px_to_world)
                    .fold(0.0f32, f32::max);
                // A borda INTERNA da linha: onde a cor deixa de estar escondida.
                let inner_edge = 250.0 - width_px * 0.5;
                gaps.push((width_px, inner_edge - fill_edge));
            }
            let want = -grow_px as f32;
            for (w, gap) in &gaps {
                assert!(
                    (gap - want).abs() <= 1.0,
                    "grow {grow_px} px, linha de {w} px: o vao visivel saiu {gap:.1} px em vez \
                     de {want:.1} px — o ajuste depende da espessura, e nao pode"
                );
            }
        }
    }

    /// E o lado positivo é simétrico: `grow = +N` **sangra** N px além da silhueta (o
    /// "off-register" da animação 2D), também igual em qualquer espessura.
    #[test]
    fn a_positive_grow_bleeds_the_same_amount_at_any_line_width() {
        let px_to_world = 10.0f32 / 800.0;
        for width_px in [1.0f32, 6.0, 16.0, 40.0] {
            let r = 250.0 * px_to_world;
            let n = 96;
            let pts: Vec<Vec2> = (0..n)
                .map(|i| {
                    let (x, y) = unit_circle(i as f32 / n as f32);
                    Vec2::new(r * x, r * y)
                })
                .collect();
            let half = width_px * 0.5 * px_to_world;
            let res = fill_at(
                &[(pts, vec![half; n], true)],
                Vec2::new(0.0, 0.0),
                FillParams {
                    precision: 1.0 / px_to_world,
                    gap_reach: 0.0,
                    grow: 4,
                    mode: FillMode::Paint,
                },
            )
            .expect("preenche");
            let fill_edge = res
                .outer
                .iter()
                .map(|p| (p.x * p.x + p.y * p.y).sqrt() / px_to_world)
                .fold(0.0f32, f32::max);
            let outer_edge = 250.0 + width_px * 0.5;
            let bleed = fill_edge - outer_edge;
            assert!(
                (bleed - 4.0).abs() <= 1.5,
                "linha de {width_px} px: o sangramento saiu {bleed:.1} px em vez de 4,0"
            );
        }
    }
}
