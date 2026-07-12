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
//!   → raster::grow          Grow/Shrink (mata o halo do AA; entra por baixo da linha)
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
    /// **Grow/Shrink**, em pixels do buffer. Positivo cresce (o preenchimento entra por
    /// baixo da linha e o halo do anti-aliasing some); negativo encolhe.
    pub grow: i32,
    /// O modo (Paint / Paint-Behind / Unpaint).
    pub mode: FillMode,
}

impl Default for FillParams {
    fn default() -> Self {
        Self {
            precision: 4.0,
            gap_reach: 0.0,
            // +2px por default: o contorno é traçado a MEIA espessura, então crescer 2px
            // enfia a cor por baixo do corpo da linha em vez de deixar um fio claro
            // entre o preenchimento e o traço (o halo).
            grow: 2,
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
    let scale = params.precision.clamp(0.5, 64.0);
    let mut grid = Grid::new(lo, hi, scale, MARGIN_PX, MAX_SIDE);

    // 3. Rasteriza as fronteiras a MEIA espessura + os fechamentos (1px, como o GP).
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
            // `radius_scale = 0.5`: metade da meia-espessura, em pixels da grade.
            let r_px = 0.5 * 0.5 * (ra + rb) * scale;
            grid.stroke_capsule(a, b, r_px);
        }
    }
    for c in &closures {
        grid.stroke_capsule(c.a, c.b, 0.0); // o fechamento é fino (1px)
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

    // 5. Grow/Shrink — o preenchimento entra por baixo da linha (mata o halo do AA).
    if params.grow != 0 {
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
}
