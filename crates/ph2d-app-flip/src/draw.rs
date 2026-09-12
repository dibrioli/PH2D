//! ADR-0114 W2 T2.5/T2.6 — a interação de DESENHO do Flip no shell (o documento
//! + a interação vivem aqui, não na tool; mesmo padrão do Vector).
//!
//! `FlipDraw` acumula as amostras do traço em curso (mundo + pressão); no pen-up
//! o traço é assado num `FlipStroke` e empurrado no desenho ativo do `FlipDoc`.
//! O estilo (cor/largura/dureza/opacidade) vem do cache que o `flip_bridge`
//! publica (downcast lá, não aqui — mantém o `input_dispatch` livre de downcast).
//!
//! **1º corte (T2.6):** amostragem simples com override a <2px (evita pontos
//! redundantes); pressão→largura linear. O active smoothing (o "assentar"
//! premium) é T2.7; RDP no pen-up é T2.8.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};
use ph2d_tool_flip::FlipStyleSnapshot;
use ph2d_vec_scene::Xform;

/// O traço do Flip em curso: amostras em MUNDO + pressão por amostra.
#[derive(Default)]
pub struct FlipDraw {
    points: Vec<Vec2>,
    pressures: Vec<f32>,
    active: bool,
    /// ⚠️ **A última amostra RECUSADA pelo `MIN_SAMPLE_PX`** — é ela que o pen-up promove.
    ///
    /// Sem isto o traço acaba onde caiu a última amostra ACEITA, que fica até `MIN_SAMPLE_PX` de
    /// onde a mão soltou: medido no gancho, o pior desvio contra a mão caía exatamente no ÚLTIMO
    /// ponto, valendo **1,0 px (8,3 % da espessura)**. Um traço que termina curto é imprecisão que
    /// o artista vê em TODO traço, e mais ainda nos curtos.
    ///
    /// Não precisa de posição no pen-up: a posição em que a mão soltou **é** a última que a
    /// ferramenta viu, e ela já passa por aqui — recusada.
    pending: Option<(Vec2, f32)>,
    /// O ajuste já decidido deste traço — estado DERIVADO, e por isso mora ao lado das amostras de
    /// que deriva. O preview roda o pipeline inteiro por quadro; sem isto ele re-decide, a cada
    /// quadro, um traço que a lei já garante que não muda.
    fit: crate::smooth::FitCache,
}

/// Distância mínima (px de tela) entre amostras — abaixo disso o move é
/// ignorado (override), evitando pontos redundantes num pixel parado.
const MIN_SAMPLE_PX: f32 = 2.0;

impl FlipDraw {
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Começa um traço com a 1ª amostra (mundo + pressão).
    pub fn begin(&mut self, world: Vec2, pressure: f32) {
        self.points.clear();
        self.pressures.clear();
        self.pending = None;
        self.fit.clear();
        self.points.push(world);
        self.pressures.push(pressure);
        self.active = true;
    }

    /// Adiciona uma amostra se andou ≥ `MIN_SAMPLE_PX` desde a última (medido em
    /// tela, via `px_per_world`). Devolve `true` se aceitou (pra o caller pintar).
    pub fn extend(&mut self, world: Vec2, pressure: f32, px_per_world: f32) -> bool {
        let Some(&last) = self.points.last() else {
            return false;
        };
        let d = world - last;
        let dist_px = (d.x * d.x + d.y * d.y).sqrt() * px_per_world;
        if dist_px < MIN_SAMPLE_PX {
            // Guardada, não descartada: se o gesto acabar aqui, ela é o fim do traço.
            self.pending = Some((world, pressure));
            return false;
        }
        self.pending = None;
        self.points.push(world);
        self.pressures.push(pressure);
        true
    }

    /// As amostras **e o cache do ajuste**, emprestados juntos — o preview precisa dos três de uma
    /// vez, e emprestar o `FlipDraw` inteiro travaria o cache contra as próprias amostras.
    pub fn preview_parts(&mut self) -> (&[Vec2], &[f32], &mut crate::smooth::FitCache) {
        (&self.points, &self.pressures, &mut self.fit)
    }

    /// Encerra o traço e devolve as amostras (mundo, pressão), limpando o estado.
    /// `None` se não há amostras suficientes (< 2 pontos = um toque, sem traço).
    pub fn take(&mut self) -> Option<(Vec<Vec2>, Vec<f32>)> {
        self.active = false;
        // ⭐ **O pen-up PROMOVE a amostra pendente** — o traço acaba onde a mão soltou, não onde
        // caiu a última amostra que passou do limiar.
        if let Some((w, pr)) = self.pending.take() {
            self.points.push(w);
            self.pressures.push(pr);
        }
        if self.points.len() < 2 {
            self.points.clear();
            self.pressures.clear();
            return None;
        }
        Some((
            std::mem::take(&mut self.points),
            std::mem::take(&mut self.pressures),
        ))
    }
}

/// sRGB8 → `Rgba` linear straight-alpha (o `FlipDoc` guarda linear; o picker/tool
/// dá sRGB). Transfer padrão; fora de qualquer caminho de sim (não é HR-5).
pub fn srgb8_to_linear(c: [u8; 4]) -> Rgba {
    fn ch(b: u8) -> f32 {
        let v = b as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    Rgba::new(ch(c[0]), ch(c[1]), ch(c[2]), c[3] as f32 / 255.0)
}

/// **A tolerância da simplificação: uma FRAÇÃO da espessura do traço.**
///
/// A grandeza é adimensional de propósito — **um desvio muito menor que a própria linha é invisível
/// por definição**, e é a linha que diz o que é "muito menor". Um limiar em px de TELA viraria uma
/// distância minúscula em MUNDO quando se desenha com a câmera perto.
///
/// ## ⚠️ O número mudou de SIGNIFICADO em 2026-07-30, e é por isso que ele mudou de valor
///
/// Até aqui a tolerância era cobrada contra a **CORDA RETA** (o `simplify_rdp`), enquanto quem
/// desenha é o `resample_smooth`, que traça uma **Catmull-Rom** pelos sobreviventes. Os dois
/// discordavam sobre o que um ponto guardado significa, e num gancho fechado a corda parecia ótima
/// enquanto a curva reconstruída saía de onde a mão passou.
///
/// **A história da constante é a prova de que ela era o instrumento errado** — ela foi reclamada
/// nas DUAS pontas: `0,0008` deu *"muitos pontos muito próximos e até sobrepostos"* (Enio,
/// 2026-07-18) e `0,05` deu *"poucos pontos … precisamos de mais precisão"* (Enio, 2026-07-30, com
/// screenshot). Um terceiro ajuste do mesmo número seria o remédio novo pagando o velho
/// ([[feedback_a_new_remedy_makes_the_old_one_double_counting]]).
///
/// Hoje o erro é cobrado contra **a curva que será desenhada** (`simplify_to_curve`), então o número
/// diz o que promete: *o traço guardado não se afasta da mão mais que esta fração da espessura*.
///
/// ## E aí o joelho MUDOU de lugar (medido no gancho da foto + num arco liso, espessura 12)
///
/// | fração | gancho: pts / desvio | arco liso: pts / desvio |
/// |---|---|---|
/// | 0,05 | 13 / **3,83 %** | 5 / 1,75 % |
/// | **0,02** | **13 / 2,86 %** | **5 / 1,75 %** |
/// | 0,01 | 26 / 2,28 % | 8 / 0,46 % |
///
/// ## ⚠️ E aí o Enio pediu **o TRIPLO de pontos** (2026-07-30, com screenshot dos pontos guardados)
///
/// Isso é decisão de produto, não de engenharia, e a tabela diz o preço. Varrida abaixo do joelho:
///
/// | fração | gancho: pts | arco liso: pts | ms/ajuste (gancho) |
/// |---|---|---|---|
/// | 0,02 | 13 | 5 | 0,218 |
/// | 0,01 | 26 | 8 | 0,451 |
/// | 0,005 | 32 | 8 | 0,868 |
/// | **0,0025** | **43** | **14** | 1,206 |
///
/// **0,0025 é o triplo pedido** (13 → 43 e 5 → 14). O desvio contra a mão vai a 1,78 %, que é o
/// piso do próprio Smoothing — abaixo disso a tolerância deixa de comprar precisão e só compra
/// pontos.
///
/// ⚠️ **O custo obrigou a reescrever o ajuste ANTES de aplicar o pedido.** A medição de custo
/// original usava o gancho (duas pernas retas, 13 pontos) e não continha o fenômeno; num traço
/// LONGO e serpenteado o ajuste guloso — que reconstruía o traço inteiro a cada inserção — custava
/// **27 ms/frame já no 0,02**, e 64 ms no 0,0025. Reconstruindo só a VIZINHANÇA do span (a
/// Catmull-Rom é local) o mesmo caso caiu para **1,6 e 2,0 ms**.
///
/// ⚠️ **A queixa antiga não volta, e é estrutural:** no arco liso a Catmull-Rom reconstrói bem, então
/// o ajuste guarda **5 pontos de 240**. Poucos pontos onde a curva os dispensa, pontos onde a
/// precisão precisa deles — as duas queixas param de disputar o mesmo número.
///
/// A cerca de 2026-07-11 (*"o desenho em tempo real está mais suave que o traço cosido"*) segue
/// honrada por ESTRUTURA e não por calibração: o preview ao vivo passa pelo MESMO
/// `stroke_from_samples`, então o traço assado é idêntico ao que o artista viu.
const STROKE_SIMPLIFY_FRACTION: f32 = 0.0025; // adimensional: fracao da espessura, MEDIDO

/// A tolerância em unidades de MUNDO (os pontos crus são mundo; a conversão para local é
/// do `build_stroke`, depois).
fn simplify_tolerance(style: &FlipStyleSnapshot) -> f32 {
    STROKE_SIMPLIFY_FRACTION * ph2d_tool_flip::size_to_world(style.width_px)
}

/// **Das amostras CRUAS ao traço** — smoothing + decimação invisível + estilo.
///
/// É `pub(crate)` porque os testes o dirigem direto, sem passar pelo gesto do
/// painel: mudar o Smoothing exige refazer *a partir das amostras*, não do traço assado
/// (o smoothing filtra o insumo; um traço já filtrado não tem como "desfiltrar").
pub fn stroke_from_samples(
    style: &FlipStyleSnapshot,
    points: &[Vec2],
    pressures: &[f32],
    world_to_local: &Xform,
) -> FlipStroke {
    stroke_from_samples_cached(
        style,
        points,
        pressures,
        world_to_local,
        &mut crate::smooth::FitCache::default(),
    )
}

/// **A MESMA porta, com a memória do ajuste** — e é por isso que ela é a de baixo: um cache recém-
/// nascido está VAZIO, então [`stroke_from_samples`] (o bake, e todo teste) percorre o caminho
/// completo **por construção**, não por calibração.
///
/// ⚠️ Isto NÃO é uma segunda rota. O preview e o bake continuam sendo a mesma função — a cerca de
/// 2026-07-11 (*"o desenho em tempo real está mais suave que o traço cosido"*) segue valendo por
/// ESTRUTURA. O que o cache muda é quanto trabalho é refeito, nunca o resultado: o
/// [`FitCache::simplify`] devolve exatamente o que o `simplify_to_curve` devolveria, e há gate
/// afirmando isso índice a índice, quadro a quadro, sobre o pipeline do produto.
pub fn stroke_from_samples_cached(
    style: &FlipStyleSnapshot,
    points: &[Vec2],
    pressures: &[f32],
    world_to_local: &Xform,
    fit: &mut crate::smooth::FitCache,
) -> FlipStroke {
    let smoothed = crate::smooth::active_smooth(points, style.smoothing);
    // ⚠️ **Contra a CURVA que será desenhada, não contra a corda reta** (Enio 2026-07-30). O
    // `simplify_rdp` cobrava a tolerância contra a corda enquanto o `resample_smooth` desenha uma
    // Catmull-Rom pelos sobreviventes — num gancho a corda parecia boa e o traço ficava a 8,46 % da
    // espessura da mão, com 11 pontos de 240.
    let keep = fit.simplify(&smoothed, simplify_tolerance(style), resample_step(style));
    let pts: Vec<Vec2> = keep.iter().map(|&i| smoothed[i]).collect();
    let prs: Vec<f32> = keep.iter().map(|&i| pressures[i]).collect();
    // **Reamostragem SUAVE** (T2.8): o RDP e o render ligam os pontos por RETAS, então poucos
    // pontos = curvas facetadas ("tracejado", Enio 2026-07-25). Interpola uma Catmull-Rom pelos
    // pontos (o traço passa exato por eles) e a densifica — as curvas ficam arredondadas, as quinas
    // ficam. É a MESMA porta do preview e do bake, então os dois seguem idênticos.
    // ⚠️ A 4ª entrada é a tolerância do PRÓPRIO RDP acima: a reamostragem não re-adiciona
    // pontos num span que o simplificador acabou de declarar reto (ver `resample_smooth`).
    let (pts, prs) =
        crate::smooth::resample_smooth(&pts, &prs, resample_step(style), simplify_tolerance(style));
    build_stroke(style, &pts, &prs, world_to_local)
}

/// **O passo da reamostragem suave, em MUNDO: uma fração da espessura.**
///
/// Os segmentos da curva reamostrada ficam ~`RESAMPLE_STEP_FRACTION × espessura` de comprimento —
/// abaixo da própria espessura, então o render (cápsulas dessa espessura) esconde as facetas e a
/// curva lê redonda. A grandeza é adimensional (fração da espessura) pela mesma razão do
/// `STROKE_SIMPLIFY_FRACTION`: é a linha que diz qual comprimento de segmento é "pequeno". O passo
/// cai com a espessura (pincel fino ⇒ passo fino ⇒ curva fina lisa), mas o cap por-span
/// (`MAX_SUB_PER_SPAN`) impede explosão. MEDIDO no smoke `PH2D_FLIP_RESAMPLE_SMOKE=1`.
const RESAMPLE_STEP_FRACTION: f32 = 0.4;

fn resample_step(style: &FlipStyleSnapshot) -> f32 {
    RESAMPLE_STEP_FRACTION * ph2d_tool_flip::size_to_world(style.width_px)
}

/// Constrói um `FlipStroke` a partir das amostras (MUNDO) + estilo. Compartilhado
/// pelo bake (pen-up) e pelo preview ao vivo (durante o arrasto).
///
/// A largura é guardada em **unidades de MUNDO** (ADR-0114 §4.C.6 — `size_to_world` é a
/// porta única; o render multiplica por `px_per_world`, então dar zoom engrossa o traço
/// na tela, como qualquer arte). ADR-0111: a geometria é LOCAL (o gizmo pode ter movido/
/// escalado o objeto), então a largura recua pela escala do objeto
/// (`world_to_local.mean_scale`) — o render refaz `× object_scale`. Objeto não-movido =
/// `wscale=1`. Com isto, POSIÇÃO e LARGURA do `Point` ficam finalmente na MESMA unidade.
fn build_stroke(
    style: &FlipStyleSnapshot,
    points: &[Vec2],
    pressures: &[f32],
    world_to_local: &Xform,
) -> FlipStroke {
    let color = srgb8_to_linear(style.stroke);
    let wscale = world_to_local.mean_scale() as f32;
    // Size → MUNDO (porta única), recuado pela escala do objeto (ADR-0111).
    let base_w = ph2d_tool_flip::size_to_world(style.width_px) * wscale;
    let mut s = FlipStroke::new();
    for (&p, &pr) in points.iter().zip(pressures.iter()) {
        let l = world_to_local.apply([f64::from(p.x), f64::from(p.y)]);
        s.push_point(Point {
            pos: Vec2::new(l[0] as f32, l[1] as f32),
            // Pressão→largura pela **dinâmica de caneta** (porta única `pressure_width_factor`): o
            // Min Width (piso) + a Response (curva macia⇔dura). No mouse `pr = 1` ⇒ largura cheia.
            width: base_w
                * ph2d_tool_flip::pressure_width_factor(
                    pr,
                    style.pressure_min_width,
                    style.pressure_response,
                ),
            opacity: style.opacity,
            color,
        });
    }
    s.hardness = style.hardness;
    // **A PONTA do traço.** ⚠️ O motor honra `cap` ponta a ponta desde que o percurso landou — o
    // bit no `pack`, o semi-plano no `stroke_silhouette`, o ramo do `flip.wgsl`, tudo gateado e
    // com paridade CPU×device provada. O que NÃO existia era esta linha: sem ela todo traço saía
    // no `Cap::default()` e a ponta reta era alcançável só de um teste. Uma capacidade sem porta
    // é pior que uma que falta, porque ela passa nos gates.
    //
    // ⚠️ **O par recebe o MESMO valor nas duas pontas**: o par existe porque a borracha, ao partir
    // um traço, pode dar pontas diferentes às metades — não porque o artista as autore separadas.
    s.cap = (style.cap, style.cap);
    // **O *tip* pontilhado** (03 §8): o traço herda a ponta do pincel (linha cheia ou
    // contas). `dot_spacing` é um MÚLTIPLO do diâmetro (relativo à espessura), direto para o
    // modelo — o fragment o escala pela largura de referência do traço.
    s.tip = style.tip;
    s.dot_spacing = style.dot_spacing as f32;
    // **Self Overlap** (03 §8): o traço herda do pincel se cruzar a si mesmo ACUMULA (escurece)
    // ou fica a união chapada. Default OFF ⇒ o traço de sempre.
    s.self_overlap = style.self_overlap;
    s.airbrush = style.airbrush;
    // **O traço PREENCHIDO** (o material stroke+fill do GP — como o Suzanne é feito):
    // o fill é a triangulação dos pontos DESTE traço, então linha e cor são UMA
    // geometria. Esculpir a linha move a cor exatamente junto, no mesmo frame — nada a
    // re-preencher, nada para ficar para trás. E ele fecha: uma forma preenchida é uma
    // forma fechada (o traço à mão quase nunca encontra a própria ponta).
    if style.draw_filled {
        s.closed = true;
        s.fill = Some(ph2d_flip::Fill {
            color: srgb8_to_linear(style.fill_color),
            opacity: 1.0,
        });
    } else {
        s.closed = false;
    }
    s
}

#[cfg(test)]
#[path = "draw_tests.rs"]
mod tests;

#[cfg(test)]
mod pen_up_tests {
    use super::*;

    /// ⭐ **O TRAÇO ACABA ONDE A MÃO SOLTOU** — não onde caiu a última amostra que passou do
    /// `MIN_SAMPLE_PX`.
    ///
    /// ⚠️ Foi o pior desvio contra a mão na sonda de captura, e ele caía **exatamente no último
    /// ponto** (`1,0 px`, 8,3 % da espessura). É imprecisão que aparece em TODO traço.
    #[test]
    fn the_stroke_ends_where_the_hand_lifted() {
        let mut d = FlipDraw::default();
        d.begin(Vec2::new(0.0, 0.0), 1.0);
        // Três aceitas (10 px de tela cada) e uma RECUSADA por estar a 1 px — o pen-up.
        assert!(d.extend(Vec2::new(10.0, 0.0), 1.0, 1.0));
        assert!(d.extend(Vec2::new(20.0, 0.0), 1.0, 1.0));
        assert!(!d.extend(Vec2::new(21.0, 0.0), 1.0, 1.0));
        let (pts, prs) = d.take().expect("dois pontos ou mais");
        assert_eq!(pts.len(), prs.len());
        let fim = *pts.last().expect("nao vazio");
        assert!(
            (fim.x - 21.0).abs() < 1e-6,
            "o traco acabou em {fim:?}, e a mao soltou em (21, 0)"
        );
    }

    /// E a pendente **não sobrevive ao gesto**: um traço novo não pode herdar o fim do anterior.
    #[test]
    fn a_new_stroke_does_not_inherit_the_previous_pen_up() {
        let mut d = FlipDraw::default();
        d.begin(Vec2::new(0.0, 0.0), 1.0);
        assert!(d.extend(Vec2::new(10.0, 0.0), 1.0, 1.0));
        assert!(!d.extend(Vec2::new(10.5, 0.0), 1.0, 1.0));
        d.begin(Vec2::new(100.0, 100.0), 1.0);
        assert!(d.extend(Vec2::new(110.0, 100.0), 1.0, 1.0));
        let (pts, _) = d.take().expect("dois pontos");
        assert_eq!(pts.len(), 2, "herdou a pendente do traco anterior: {pts:?}");
    }
}
