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
//!   → raster::Grid          rasteriza as fronteiras NO EIXO da polilinha (BUGS #14)
//!   → raster::flood         span fill 4-conexo + filtro de vazamento CRUZADO
//!   → raster::expand_under_ink  a borda da cor crava EM CIMA do eixo
//!   → raster::grow          Grow/Shrink do usuário (offset assinado do eixo; default 0)
//!   → trace::trace_contours marching squares — os buracos saem de graça
//!   → trace::simplify_ring  RDP + binomial leve
//!   = FillResult { outer, holes, closures }
//! ```
//!
//! **A âncora é o EIXO da linha, não a silhueta.** A espessura do traço é absoluta em
//! px de TELA e o fill é assado em unidades de DOCUMENTO — âncoras derivadas da
//! espessura descolam quando o zoom muda depois do clique (BUGS #14). O eixo é
//! geometria pura, e a linha renderizada sempre o cavalga: a cor que termina nele fica
//! certa em qualquer zoom e qualquer espessura, sem ajuste nenhum.
//!
//! **CPU, de propósito.** O fill é uma operação de CLIQUE, não de frame: o span fill é
//! ~10× o BFS por pixel e roda em poucos ms num buffer de milhões de pixels. A GPU
//! seria o primitivo errado — o JFA **salta paredes** (não é geodésico), e o readback
//! para vetorizar o contorno seria inevitável de qualquer jeito (`04 §3`).
//!
//! HR-5: zero transcendentais fora de `sqrt` (que é exata em IEEE-754).

mod arrange;
mod ball;
mod dilate;
mod edt;
mod gap;
mod raster;
mod trace;
mod weld;

pub use arrange::{Region, region_at};
pub use ball::{TrapBall, TrapRegion};
pub use dilate::{
    contour_widths, local_line, nearest_on_axis, nearest_on_axis_indexed, outward_normals,
};
pub use edt::sq_distance_to_set;
pub use gap::{Boundary, Closure};
pub use raster::{BOUNDARY, FILLED, Grid, INK};
pub use trace::{RDP_EPSILON_PX, signed_area, simplify_ring, trace_contours};
pub use weld::{Weld, welds};

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
/// Passes do `expand_under_ink` que cravam a borda da cor em cima do eixo. O filamento
/// de `INK` tem ≤ 2 px na transversal (raio 0 + quantização); 3 passes o cobrem com
/// folga — e o teto TEM de ser pequeno, porque a expansão rasteja ao longo do filamento
/// ~1 px por passe (ver o passo 5 do `fill_at`).
const AXIS_COVER_PASSES: usize = 3;
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
    /// **Grow/Shrink**, em pixels do BUFFER — um offset ASSINADO do contorno a partir
    /// do **EIXO** da linha (a âncora de tudo, BUGS #14). Contínuo em 0:
    ///
    /// - `0` → a borda da cor crava no eixo: sem vão, sem transbordo, em qualquer
    ///   espessura E em qualquer zoom posterior (o default certo por construção);
    /// - `> 0` → o contorno avança `grow` px além do eixo — por baixo da linha até
    ///   `w/2`, e além disso sangra (o "off-register" da animação 2D);
    /// - `< 0` → o contorno recua `|grow|` px do eixo — o vão VISÍVEL começa quando o
    ///   recuo passa de `w/2` (aparência é função da espessura e da câmera; a
    ///   geometria, não).
    ///
    /// O chamador converte de px de tela para px de buffer (multiplicando pela
    /// `precision`) — senão subir a Precision encolheria o Grow em silêncio.
    pub grow: i32,
    /// **Trap** — o raio da *trapped ball*, em pixels do BUFFER. `0` = desligado.
    ///
    /// Com `trap_px > 0` o flood é feito por uma **bola de raio `trap_px`**, que não
    /// atravessa um vão mais estreito que `2·trap_px` (Zhang et al., TVCG 2009 — ver
    /// [`crate::TrapBall`]). É a alternativa ao Gap Closure para o caso comum: em vez de
    /// o artista achar o vão e calibrar um alcance, ele diz "a tinta é grossa assim".
    ///
    /// **`0` deixa o pipeline byte a byte como era** (o `Grid::flood` de sempre) — a wave
    /// Colorize é opt-in inteira (`docs/Flip/09_colorize.md`).
    ///
    /// Como o `grow`, o chamador converte de px de TELA para px de buffer multiplicando
    /// pela `precision` — senão subir a Precision encolheria a bola em silêncio, que é
    /// exatamente o acoplamento escondido do BUGS #11.
    pub trap_px: f32,
    /// O modo (Paint / Paint-Behind / Unpaint).
    pub mode: FillMode,
}

impl Default for FillParams {
    fn default() -> Self {
        Self {
            precision: 4.0,
            gap_reach: 0.0,
            // **Zero.** Com a âncora no eixo, o default já é exato por construção — em
            // qualquer espessura e em qualquer zoom. O Grow é só o ajuste estilístico.
            grow: 0,
            // **Zero = o balde de sempre, ao bit.** A bola é opt-in: ela muda o que
            // conta como região, e mudar isso por default reescreveria o comportamento
            // que o Enio já aprovou no smoke do W4.
            trap_px: 0.0,
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
    /// **A resolução que a grade de fato ENTREGOU** (px de buffer por unidade de
    /// documento) — e não a que o chamador pediu.
    ///
    /// As duas divergem sempre que o `MAX_SIDE` do `Grid::new` morde: ali quem cede é a
    /// RESOLUÇÃO, então o erro do contorno vetorizado para de melhorar enquanto a
    /// precisão *pedida* continua subindo.
    ///
    /// ⚠️ **Quem tolera o erro do contorno tem de derivar a tolerância DAQUI.** Medido em
    /// 2026-07-18: o `fill_click` calculava o "abraço" da precisão pedida e o comparava
    /// com um erro nascido da entregue — dois números sobre o mesmo fato, de fontes
    /// diferentes. Acima de ~3200 px de arte na tela a grade saturava, a tolerância
    /// continuava encolhendo, e a rota do arranjo (a que faz a malha do fill se apoiar nos
    /// vértices das linhas) **se recusava em silêncio** justamente no regime de alta
    /// precisão. O smoke do Enio caiu exatamente ali: *"nenhuma melhoria e nenhuma
    /// mudança"*.
    pub scale: f32,
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
    /// A **bola do Trap não cabe** onde se clicou — o lugar é mais estreito que ela.
    ///
    /// Erro próprio, e não um `Leaked` reaproveitado, porque a saída para o usuário é a
    /// OPOSTA: aqui ele tem de **baixar** o Trap (ou clicar num lugar mais largo), e
    /// mandá-lo subir o Gap Closure o levaria para longe da solução.
    BallTooFat,
}

/// **Preenche** a região que contém `click`, delimitada por `strokes`.
///
/// `strokes` são as polilinhas de fronteira (`(pontos, meia-espessura por ponto,
/// fechado)`), no espaço do documento. **A espessura NÃO entra no raster** — a parede e
/// a âncora do resultado são o **EIXO** da polilinha (ver o passo 3, e por quê); ela só
/// folga o bbox da grade.
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

    // 3. As fronteiras rasterizam **NO EIXO da polilinha** — raio ZERO (BUGS #14).
    //
    //    A espessura do traço é ABSOLUTA em px de TELA (Enio 2026-07-11) e a geometria
    //    do fill é assada em unidades de DOCUMENTO no instante do clique: a relação
    //    entre as duas MUDA quando a câmera aproxima. Qualquer âncora derivada da
    //    espessura (silhueta, borda interna) fica presa ao zoom do clique — dar zoom
    //    depois transborda `(w/2)·(zoom−1)` px. O eixo é a única âncora que é geometria
    //    pura, e a linha renderizada sempre o cavalga, metade para cada lado: a cor que
    //    termina nele nunca aparece por fora nem descola da linha, em qualquer zoom e
    //    qualquer espessura.
    //
    //    Dois bits na MESMA cápsula:
    //    - `BOUNDARY` = a parede do flood (com a folga de AA de ½px que a mantém
    //      estanque);
    //    - `INK` = a linha de pixels que o eixo atravessa (sem folga): o alvo do
    //      passo 5, que crava a borda da cor EM CIMA do eixo — sem ele, a cor pararia
    //      na face interna da parede, ~1 px de buffer aquém.
    //
    //    ⚠️ **CORREÇÃO (2026-07-21, BUGS #23).** O comentário que morava aqui dizia:
    //    *"duas linhas cujos CORPOS se sobrepõem mas cujos eixos não se cruzam deixam de
    //    selar sozinhas — é o Gap Closure que fecha, e o toast do vazamento já o sugere"*.
    //    O **mecanismo** estava certo e o **veredito** errado. Medido numa caixa de mão de
    //    4 traços (o vão entre os eixos das quinas: 0,0045 a 0,0404 doc, contra 0,26 de
    //    tinta): a partir da precisão **80** o balde devolve `Leaked` e o artista lê *"raise
    //    Gap Closure to seal the outline"* — sobre uma quina que na tela está fechada com
    //    **6× de folga**. Pior: a 40 preenchia, então **subir a Precision quebrava o
    //    balde**. Mandar o artista fechar à mão um vão que ele já cobriu de tinta é a
    //    ferramenta pedindo que ele conserte o que não está quebrado.
    for (pts, _, closed) in strokes {
        let n = pts.len();
        if n < 2 {
            continue;
        }
        let last = if *closed { n } else { n - 1 };
        for i in 0..last {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            grid.stroke_capsule(a, b, 0.0); // a parede, no eixo
            grid.ink_capsule(a, b, 0.0); // a linha do eixo (alvo do passo 5)
        }
    }
    // **As JUNTAS** (`weld.rs`): onde a tinta que o artista pintou cobre o vão entre duas
    // partes, a parede é contínua ali — porque na TELA ela é. Regra DERIVADA da arte
    // (`d ≤ meia-largura + meia-largura`), sem knob e sem constante.
    //
    // ⚠️ **Não é contagem dupla com o Gap Closure, e os dois são DISJUNTOS por construção**
    // ([[feedback_a_new_remedy_makes_the_old_one_double_counting]]): a solda só dispara onde
    // a tinta JÁ cobre — um vão deliberado (um C aberto, um esboço) tem as pontas longe uma
    // da outra e ela nunca o toca. O Gap Closure segue sendo a ferramenta de quem quer
    // fechar o que decidiu deixar aberto; a solda faz o **default** parar de contradizer a
    // tela. E ela **não entra em `closures`**: um fechamento é artefato AUTORADO, que o
    // chamador materializa e o artista vê — a solda não é nada disso.
    for (a, b) in weld::welds(strokes) {
        grid.stroke_capsule(a, b, 0.0);
        grid.ink_capsule(a, b, 0.0);
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
    if params.trap_px > 0.0 {
        // **A bola** (opt-in): inunda com um disco de raio `trap_px`, que não passa por
        // um vão mais estreito que `2r`. Só a PERGUNTA "quais pixels são a região" muda —
        // o resto do pipeline (o `expand_under_ink`, o Grow, a vetorização) é o mesmo
        // código, lendo o mesmo bit `FILLED`. Duas respostas para "onde a região termina"
        // divergiriam, e é a doença de todo bug desta linha.
        let ball = TrapBall::new(&grid);
        let Some(region) = ball.region_from(seed, params.trap_px) else {
            // A bola não cabe na semente: o vão é mais estreito que ela. É o mesmo
            // "não deu" do clique em cima da linha — e o toast manda baixar o Trap.
            return Err(FillError::BallTooFat);
        };
        if region.escaped {
            return Err(FillError::Leaked);
        }
        for (i, &m) in region.mask.iter().enumerate() {
            if m {
                grid.flags[i] |= raster::FILLED;
            }
        }
    } else if !grid.flood(seed, LEAK_PX) {
        return Err(FillError::Leaked);
    }

    // 5. A borda da cor crava EM CIMA do eixo: cobre a linha de pixels que ele
    //    atravessa. Poucos passes DE PROPÓSITO — a expansão rasteja ao longo do
    //    filamento de `INK` ~1 px por passe (num cruzamento, ela subiria pelo eixo do
    //    outro traço), e tudo que ela alcança está sob a linha renderizada de qualquer
    //    jeito.
    grid.expand_under_ink(AXIS_COVER_PASSES);
    // 6. O Grow do usuário: um offset ASSINADO a partir do eixo, sem ramo — é isso que
    //    torna o slider CONTÍNUO em 0 (a âncora dupla do corte anterior saltava w+1 px
    //    entre 0 e −1, BUGS #14). Positivo avança por baixo da linha (além de w/2 vira
    //    o "off-register" da animação 2D); negativo recua (o vão visível começa quando
    //    |grow| passa de w/2 — o preço, aceito, da âncora zoom-proof).
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
        // A ENTREGUE, lida da grade depois do `Grid::new` — nunca `params.precision`.
        scale: grid.scale,
        closures,
    })
}

#[cfg(test)]
mod tests;
