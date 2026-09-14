//! ⭐⭐⭐ **A ÂNCORA DE IK, desenhada** — o losango do alvo e o tracejado que o liga à ponta.
//!
//! ⚠️ **Corte por RESPONSABILIDADE** (o teto de 700 LOC do HR-18 pediu-o a `723`): o [`super`]
//! desenha o **OSSO** (o corpo, a junta, a ponta, a mancha de influência); aqui desenha-se o que
//! **manda** nele. São dois assuntos, e o segundo cresceu por cima do primeiro.
//!
//! ⚠️ As leis de tamanho vêm do irmão numa porta só ([`super::joint_radius_px`],
//! [`super::BONE_JOINT_R_PX`]): o losango nasce do tamanho do anel que ele substitui, e não de um
//! número próprio que envelheceria ao lado daquele.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color as VelloColor, Point, Stroke, VectorScene};

use super::{BONE_JOINT_R_PX, BoneHover, BonePart, LINE_PX, joint_radius_px};

/// ⭐⭐⭐ **O RAIO DO LOSANGO** — a porta única do desenho e do dedo, e ela é **maior que a bolinha
/// de propósito**.
///
/// ⛔ **É um report do dono** (2026-09-07): *«quando colocamos um IK num bone no meio dos ossos, o
/// losango do IK e o círculo do outro osso ficam sobrepostos»*. Uma âncora criada no MEIO de uma
/// corrente nasce exactamente sobre a junta do osso seguinte — dois alvos concêntricos com verbos
/// diferentes, e o dedo não tem como escolher.
///
/// ⭐ **A cura é a que ele propôs, e é a única que funciona para alvos concêntricos: eles têm de
/// diferir em TAMANHO.** O losango fica por FORA, a bolinha por dentro, e o anel entre os dois é a
/// zona exclusiva da âncora.
///
/// ⚠️ **O PISO desse anel é DERIVADO, não escolhido:** ele é exactamente [`BONE_JOINT_R_PX`] — a
/// tolerância do dedo desta casa (o mesmo `HANDLE_HIT_PX` que toda alça do vector usa). ⇒ o artista
/// tem sempre **pelo menos um dedo inteiro** de anel para agarrar a âncora, seja qual for o zoom e
/// mesmo com a bolinha por dentro no tamanho máximo.
///
/// ⭐ **E por cima do piso vem o [`GOAL_BIGGER`], que é decisão do DONO** (2026-09-07: *«o losango
/// deve ser 25% maior»*), depois de ver a primeira versão na tela. ⛔ Ele **não** é um teto nem um
/// limite de recurso — é a leitura, e quem a julga é o smoke. Medido: num osso longo o losango passa
/// de `24` para **`30` px** de meia-diagonal, e o anel exclusivo de `12` para **`18`**.
#[must_use]
pub fn goal_radius_px(comp: f64) -> f64 {
    (joint_radius_px(comp) + BONE_JOINT_R_PX) * GOAL_BIGGER
}

/// Quanto o losango cresce por cima do piso derivado — **veredito do dono sobre a tela**, e por isso
/// um número de produto e não uma medição.
///
/// ⚠️ Ele multiplica o RAIO inteiro, e não só o anel: *«25% maior»* é sobre o losango que se vê. E
/// como o piso já garantia um dedo de anel, subir aqui só **alarga** a zona exclusiva da âncora —
/// nunca a aperta.
const GOAL_BIGGER: f64 = 1.25;

/// A espessura do traço do losango.
///
/// ⚠️ **Mais grossa que a das outras alças, e o dono pediu-o pelo nome** (*«que seu gizmo tenha
/// espessura maior»*): num par concêntrico o que está por FORA tem de se ler como o alvo maior,
/// senão o anel exclusivo existe e não se vê. ⛔ Número de PRODUTO, e declarado como tal — o recurso
/// que ele nomeia é a leitura da tela, e o smoke é quem o julga.
const GOAL_LINE_PX: f64 = 2.5;

/// **Uma âncora, como o desenho a lê** — `(osso, alvo, origem do osso, ponta do osso)` em MUNDO.
///
/// ⚠️ O segmento do osso vem junto porque o tamanho do losango sai da mesma porta da bolinha
/// ([`joint_radius_px`], sobre o comprimento **na tela**): sem ele o desenho teria de re-encontrar o
/// osso, que é a segunda resposta à mesma pergunta.
pub type Goal = (u64, [f64; 2], [f64; 2], [f64; 2]);

/// A espessura da faixa que marca a corrente governada — **mais grossa que o osso**, para ela se ler
/// como um realce POR BAIXO dele e não como mais uma linha.
const CHAIN_BAND_PX: f64 = 7.0;

/// ⭐⭐⭐ **A CORRENTE QUE A ÂNCORA GOVERNA, desenhada** — uma faixa por baixo dos ossos que ela
/// dobra, da raiz da corrente até à ponta.
///
/// ⛔⛔ **Report do dono** (2026-09-14): *«não temos uma linha indicativa do IK Chain»*. O `Chain` é
/// um número num painel, e o que ele significa é **quais ossos obedecem** — uma pergunta sobre a
/// cena. Sem isto, mudar o número de `2` para `4` não tem efeito visível nenhum até se arrastar o
/// alvo, e o artista fica a adivinhar qual foi a conta.
///
/// ⚠️ **Ela desenha-se ANTES dos ossos e das âncoras**, e é o que a torna um realce em vez de um
/// desenho novo: ela passa por baixo do que se agarra.
///
/// ⛔⛔ **E ela é uma RECTA AO LADO, nunca uma cobra por cima** (2.º report do dono, 2026-09-14:
/// *«a linha do IK Chain deve ser uma linha reta e não passar por dentro dos ossos»*). A 1.ª
/// redacção traçava a polilinha das juntas: numa corrente quase esticada ela caía **exactamente**
/// sobre os corpos dos ossos e lia-se como parte deles, e numa dobrada serpenteava. O que ela tem
/// de dizer é *até onde o `Chain` chega* — uma **extensão**, que é o que uma cota de desenho técnico
/// diz com uma recta deslocada. A geometria vive na porta [`chain_bar`], que é o que o gate mede.
///
/// ⚠️ **A cor segue a selecção**, como o losango: a corrente do osso escolhido acende, as outras
/// ficam apagadas. Sem isso, uma cena com quatro restrições seria uma teia de faixas todas iguais.
pub fn draw_chains(
    chains: &[(u64, Vec<[f64; 2]>)],
    selected: Option<u64>,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (aceso, apagado) = (vello(ColorToken::Accent), vello(ColorToken::AccentSoft));
    for (bits, juntas) in chains {
        let pontos: Vec<Point> = juntas
            .iter()
            .map(|j| transform * Point::new(j[0], j[1]))
            .collect();
        let cor = if Some(*bits) == selected {
            aceso
        } else {
            apagado
        };
        // ⚠️ `None` quando a raiz e a ponta coincidem (corrente dobrada sobre si mesma): ali não há
        // recta nenhuma a desenhar, e inventar uma direcção seria ler ruído de `f64`.
        if let Some((a, b)) = chain_bar(&pontos) {
            let mut faixa = BezPath::new();
            faixa.move_to(a);
            faixa.line_to(b);
            target.inner_mut().stroke(
                &Stroke::new(CHAIN_BAND_PX),
                Affine::IDENTITY,
                // ⚠️ **Meia opacidade**: ela é um fundo. A cheio, uma faixa mais grossa que o osso
                // esconderia a arte por baixo dela e o artista deixaria de ver o que está a posar.
                &Brush::Solid(cor.with_alpha(CHAIN_BAND_ALPHA)),
                None,
                &faixa,
            );
        }
        // ⭐ **A marca da RAIZ da corrente** — é ali que o `Chain` pára, e é a única parte do
        // realce que carrega informação nova: sem ela, duas correntes que se sobrepõem lêem-se
        // como uma. ⚠️ Ela fica na RAIZ de verdade, não na ponta da recta deslocada: o que ela
        // responde é *qual junta é a fronteira*, e a recta responde *até onde*.
        if let Some(&p) = pontos.first() {
            let r = CHAIN_BAND_PX;
            let mut risco = BezPath::new();
            risco.move_to(Point::new(p.x - r, p.y - r));
            risco.line_to(Point::new(p.x + r, p.y + r));
            risco.move_to(Point::new(p.x - r, p.y + r));
            risco.line_to(Point::new(p.x + r, p.y - r));
            target.inner_mut().stroke(
                &Stroke::new(LINE_PX),
                Affine::IDENTITY,
                &Brush::Solid(cor),
                None,
                &risco,
            );
        }
    }
}

/// ⭐⭐⭐ **A RECTA QUE DIZ ATÉ ONDE O `Chain` CHEGA** — a extensão da corrente, deslocada para
/// **fora** dela, em píxeis de ECRÃ.
///
/// ⛔⛔ **Ela tem de LIMPAR os ossos, e o quanto é DERIVADO, não escolhido** (2.º report do dono,
/// 2026-09-14): o que está desenhado ao longo de cada osso é o **corpo**
/// ([`crate::bone_half_width_px`]) e a **bolinha da junta** ([`crate::joint_radius_px`]), as duas
/// já portas desta crate e as duas função do comprimento do osso **na tela**. O afastamento é o
/// maior dos dois ao longo da corrente, mais meia faixa, mais a folga.
///
/// ⚠️ **A folga é `2 × LINE_PX`, e é o mesmo recurso que o [`crate::BONE_HALF_MIN_PX`] já nomeia:**
/// abaixo de duas larguras de contorno as duas bordas **fundem-se numa risca só**, e o realce volta
/// a ler-se como parte do osso — que é exactamente o defeito reportado.
///
/// ⭐ **Para que lado, e quanto.** Para o lado mais LIVRE — aquele em que a corrente se afasta
/// menos da corda —, e por fora dessa excursão. ⚠️ Medido em ECRÃ, não em mundo: uma câmara
/// espelhada troca a mão, e um lado escolhido em mundo apareceria do lado errado.
///
/// ⛔⛔ **Um afastamento constante não chega**, e quem o disse foi o gate: uma corrente que se
/// enrola mais de meia volta tem bojo dos DOIS lados e vem por trás da recta (medido: `18,39 px` de
/// um osso que ocupa `18,75`).
///
/// ⚠️ `None` quando a raiz e a ponta coincidem: ali não há recta, e inventar uma direcção seria ler
/// ruído de `f64`.
#[must_use]
pub fn chain_bar(pontos: &[Point]) -> Option<(Point, Point)> {
    let (&a, &b) = (pontos.first()?, pontos.last()?);
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let comp = dx.hypot(dy);
    if comp <= f64::EPSILON {
        return None;
    }
    let (ux, uy) = (dx / comp, dy / comp);
    let (px, py) = (-uy, ux);
    // ⭐⭐ **De que lado, e QUANTO** — as duas perguntas são uma só, e a resposta é a EXCURSÃO da
    // corrente para cada lado da corda. O lado é o mais livre dos dois, e o afastamento tem de
    // passar por fora da excursão desse lado.
    //
    // ⛔⛔ **Um afastamento CONSTANTE não chega, e foi o gate que o disse:** uma corrente que se
    // enrola mais de meia volta (8 ossos a `0,9 rad` por junta) dá a volta e vem **por trás** da
    // recta — ela passava a `18,39 px` de um osso que ocupa `18,75`. *Fugir do bojo não basta
    // quando a corrente tem bojo dos dois lados.*
    let (mais, menos) = pontos.iter().fold((0.0_f64, 0.0_f64), |(p1, m1), q| {
        let e = (q.x - a.x) * px + (q.y - a.y) * py;
        (p1.max(e), m1.max(-e))
    });
    let (lado, excursao) = if mais <= menos {
        (1.0, mais)
    } else {
        (-1.0, menos)
    };
    let mut alcance: f64 = 0.0;
    for w in pontos.windows(2) {
        let c = (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
        alcance = alcance
            .max(crate::bone_half_width_px(c))
            .max(crate::joint_radius_px(c));
    }
    let afastamento = excursao + alcance + CHAIN_BAND_PX * 0.5 + CHAIN_GAP_PX;
    let (ox, oy) = (px * lado * afastamento, py * lado * afastamento);
    Some((
        Point::new(a.x + ox, a.y + oy),
        Point::new(b.x + ox, b.y + oy),
    ))
}

/// A folga entre a recta da corrente e o que está desenhado sobre os ossos. Ver [`chain_bar`] — o
/// recurso é o **contorno**, o mesmo que o [`crate::BONE_HALF_MIN_PX`] nomeia.
const CHAIN_GAP_PX: f64 = 2.0 * crate::LINE_PX;

/// Quanto a faixa da corrente deixa passar. ⚠️ Número de PRODUTO (a leitura da tela), como o
/// [`GOAL_LINE_PX`]: ela é um FUNDO, e a cheio esconderia a arte que o artista está a posar.
const CHAIN_BAND_ALPHA: f32 = 0.35;

/// ⭐⭐⭐ **A ÂNCORA DE IK** — o losango do alvo, mais o tracejado que o liga à ponta da corrente.
///
/// # Por que um LOSANGO e não mais um anel
///
/// Porque o app já tem duas alças redondas (a junta e a ponta) e um quadrado (a força), e uma
/// terceira redonda obrigaria o artista a **decorar** qual é qual. *A forma carrega o verbo*: o
/// losango é o alvo, e ele lê-se como alvo em toda a referência (o Blender desenha o alvo de IK
/// como um *empty*, o Spine como uma cruz).
///
/// ⭐⭐ **E ele SUBSTITUI o anel da ponta, não se soma a ele.** Num osso com âncora a ponta deixa de
/// ser agarrável — o que se arrasta é o alvo — e desenhar as duas coisas por cima uma da outra
/// prometeria dois verbos onde há um. Quem decide é o `draw_bones`, que já recebe a lista de
/// pontas: a shell tira dela quem tem âncora.
///
/// ⚠️ **O tamanho sai da MESMA porta da bolinha** ([`joint_radius_px`], sobre o comprimento do osso
/// na tela): o losango nasce exactamente do tamanho do anel que ele substitui, e não de um número
/// próprio que envelheceria ao lado daquele.
///
/// ⚠️ **O tracejado só se vê quando eles se separam** — o que acontece quando o alvo está FORA DE
/// ALCANCE, e é aí que ele é informação: ele diz *«a corrente esticou e não chegou»*, que é o único
/// estado em que o artista precisa de ver os dois pontos.
pub fn draw_goals(
    goals: &[Goal],
    selected: Option<u64>,
    hover: Option<BoneHover>,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (aceso, apagado) = (vello(ColorToken::Accent), vello(ColorToken::AccentSoft));
    for &(bits, ancora, a, b) in goals {
        let (pa, pb) = (
            transform * Point::new(a[0], a[1]),
            transform * Point::new(b[0], b[1]),
        );
        let comp = (pb.x - pa.x).hypot(pb.y - pa.y);
        if comp <= f64::EPSILON {
            continue;
        }
        let p = transform * Point::new(ancora[0], ancora[1]);
        let sel = Some(bits) == selected;
        let sob = hover.is_some_and(|h| h.bone == bits && h.part == BonePart::Tip);
        let cor = if sel || sob { aceso } else { apagado };
        // O TRACEJADO, primeiro: ele é um fundo, e o losango desenha-se por cima dele.
        let mut fio = BezPath::new();
        fio.move_to(pb);
        fio.line_to(p);
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX).with_dashes(0.0, DASH),
            Affine::IDENTITY,
            &Brush::Solid(apagado),
            None,
            &fio,
        );
        let r = goal_radius_px(comp);
        let mut losango = BezPath::new();
        losango.move_to(Point::new(p.x, p.y - r));
        losango.line_to(Point::new(p.x + r, p.y));
        losango.line_to(Point::new(p.x, p.y + r));
        losango.line_to(Point::new(p.x - r, p.y));
        losango.close_path();
        // ⛔ **O losango NÃO se enche, ao contrário das outras alças.** Ele é o de FORA de um par
        // concêntrico: enchê-lo tapava a bolinha do osso que vive por dentro dele, e o artista
        // deixava de ver o alvo que o clique de dentro pega. *A gramática do preenchimento vale
        // para alças que estão sozinhas.* Aqui quem diz «escolhido» é a COR, que é o outro canal.
        let _ = sel;
        target.inner_mut().stroke(
            &Stroke::new(GOAL_LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(cor),
            None,
            &losango,
        );
    }
}

/// O tracejado do fio âncora→ponta. Padrão de marching-ants da casa, o mesmo das guias e da
/// timeline.
const DASH: [f64; 2] = [4.0, 3.0]; // LITERAL-PX-OK: marching-ants dash pattern

#[cfg(test)]
mod tests {
    use super::*;

    /// Distância de `p` ao segmento `a–b`.
    fn dist_ao_segmento(p: Point, a: Point, b: Point) -> f64 {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let n2 = dx * dx + dy * dy;
        let t = if n2 <= f64::EPSILON {
            0.0
        } else {
            (((p.x - a.x) * dx + (p.y - a.y) * dy) / n2).clamp(0.0, 1.0)
        };
        (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
    }

    /// A menor distância entre dois segmentos, amostrada finamente nos dois sentidos.
    fn dist_entre_segmentos(a: (Point, Point), b: (Point, Point)) -> f64 {
        let mut d = f64::INFINITY;
        for k in 0..=200 {
            let t = f64::from(k) / 200.0;
            let pa = Point::new(a.0.x + t * (a.1.x - a.0.x), a.0.y + t * (a.1.y - a.0.y));
            let pb = Point::new(b.0.x + t * (b.1.x - b.0.x), b.0.y + t * (b.1.y - b.0.y));
            d = d.min(dist_ao_segmento(pa, b.0, b.1));
            d = d.min(dist_ao_segmento(pb, a.0, a.1));
        }
        d
    }

    /// Uma corrente de `n` ossos de `comp` píxeis, dobrando `passo` radianos por junta.
    fn corrente(n: usize, comp: f64, passo: f64) -> Vec<Point> {
        let mut pts = vec![Point::new(0.0, 0.0)];
        let mut ang = 0.0;
        for i in 0..n {
            ang += if i == 0 { 0.0 } else { passo };
            let u = *pts.last().expect("a corrente tem raiz");
            pts.push(Point::new(u.x + comp * ang.cos(), u.y + comp * ang.sin()));
        }
        pts
    }

    /// ⭐⭐⭐ **A RECTA DA CORRENTE NÃO TOCA NENHUM OSSO** — o 2.º report do dono, dito como número
    /// (*«deve ser uma linha reta e não passar por dentro dos ossos»*).
    ///
    /// ⚠️ **A barra é DERIVADA, não escolhida**: ela é o que de facto está desenhado sobre cada
    /// osso — o corpo ([`crate::bone_half_width_px`]) ou a bolinha ([`crate::joint_radius_px`]), o
    /// maior dos dois — mais meia faixa. Uma barra em píxeis soltos mediria um número meu.
    ///
    /// ⭐ **O corpus varre a dobra INTEIRA, e é isso que importa:** o defeito reportado mora na
    /// corrente quase ESTICADA (`passo ≈ 0`), onde a polilinha antiga caía exactamente sobre os
    /// ossos; a dobrada é o outro extremo, onde ela serpenteava.
    #[test]
    fn the_chain_bar_never_touches_a_bone() {
        let mut casos = 0;
        for n in [2usize, 3, 5, 8] {
            for comp in [24.0_f64, 107.52, 192.0] {
                for passo in [0.0_f64, 0.05, 0.2, 0.5, 0.9, -0.3, -0.8] {
                    let pts = corrente(n, comp, passo);
                    let Some(barra) = chain_bar(&pts) else {
                        continue;
                    };
                    casos += 1;
                    for w in pts.windows(2) {
                        let c = (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
                        let alcance = crate::bone_half_width_px(c).max(crate::joint_radius_px(c));
                        // ⚠️ **A barra exige AR, não encosto** — a promessa não é «não sobrepõe»,
                        // é *«não se lê como parte do osso»*, e o que separa duas bordas é o
                        // contorno (o recurso que o [`crate::BONE_HALF_MIN_PX`] já nomeia).
                        //
                        // ⚠️⚠️ **UMA largura, e o produto entrega DUAS, de propósito:** exigir aqui
                        // exactamente o que o produto dá faz a asserção passar por **igualdade em
                        // `f64`** — e uma igualdade amostrada é um gate a morrer de pé (medido: a
                        // 1.ª redacção reprovou o PRODUTO por `1e-15`). Com metade da folga, o
                        // produto passa com margem e o mutante que a apaga cai por `2,5 px`.
                        let exigido = alcance + CHAIN_BAND_PX * 0.5 + crate::LINE_PX;
                        let d = dist_entre_segmentos(barra, (w[0], w[1]));
                        assert!(
                            d >= exigido,
                            "com {n} ossos de {comp} px a dobrar {passo} rad, a recta passa a {d} \
                             px do osso — o desenho dele ocupa {exigido} px, logo ela passa POR \
                             DENTRO"
                        );
                    }
                }
            }
        }
        assert!(casos >= 80, "o corpus encolheu para {casos} casos");
    }

    /// ⭐⭐ **A RECTA FICA DO LADO LIVRE, encostada à corrente** — e este gate mede COMPACIDADE, que
    /// é uma coisa diferente de limpeza.
    ///
    /// ⛔ **Uma mutação disse-o:** com o lado FIXO a recta continua a limpar todos os ossos (o
    /// afastamento já passa por fora da excursão daquele lado) — ela só fica **longe**, do outro
    /// lado do arco. Um indicador atirado para fora do desenho não diz *até onde o `Chain` chega*,
    /// diz que há uma risca algures. ⇒ o lado não é correcção, é LEITURA, e por isso tem gate
    /// próprio.
    #[test]
    fn the_chain_bar_takes_the_free_side() {
        for passo in [0.25_f64, 0.5, 0.9, -0.25, -0.5, -0.9] {
            let pts = corrente(5, 107.52, passo);
            let (a, b) = chain_bar(&pts).expect("a corrente tem extensão");
            let (raiz, ponta) = (pts[0], pts[pts.len() - 1]);
            let (dx, dy) = (ponta.x - raiz.x, ponta.y - raiz.y);
            let n = dx.hypot(dy);
            let (px, py) = (-dy / n, dx / n);
            // De que lado a corrente boja, e de que lado a recta ficou.
            let bojo: f64 = pts
                .iter()
                .map(|q| (q.x - raiz.x) * px + (q.y - raiz.y) * py)
                .sum();
            let barra = ((a.x + b.x) * 0.5 - (raiz.x + ponta.x) * 0.5) * px
                + ((a.y + b.y) * 0.5 - (raiz.y + ponta.y) * 0.5) * py;
            assert!(
                bojo * barra < 0.0,
                "com a corrente a bojar {bojo} a recta foi para o MESMO lado ({barra}) — ela fica \
                 do outro lado do arco, longe do que descreve (dobra {passo})"
            );
        }
    }

    /// ⭐ **E ela é mesmo uma RECTA** — dois pontos, e a direcção deles é a da raiz à ponta.
    ///
    /// ⚠️ A metade ANTI-VÁCUO é a 2.ª asserção: uma recta paralela à corda está certa; uma recta
    /// qualquer também passaria na 1.ª.
    #[test]
    fn the_chain_bar_is_parallel_to_the_chain_span() {
        for passo in [0.0_f64, 0.3, 0.7, -0.5] {
            let pts = corrente(5, 107.52, passo);
            let (a, b) = chain_bar(&pts).expect("a corrente tem extensão");
            let (raiz, ponta) = (pts[0], pts[pts.len() - 1]);
            let corda = ((ponta.x - raiz.x), (ponta.y - raiz.y));
            let barra = ((b.x - a.x), (b.y - a.y));
            let cruz = corda.0 * barra.1 - corda.1 * barra.0;
            let escala = corda.0.hypot(corda.1) * barra.0.hypot(barra.1);
            assert!(
                cruz.abs() <= 1e-9 * escala,
                "a recta não é paralela à extensão da corrente (dobra {passo})"
            );
            assert!(
                (corda.0 * barra.0 + corda.1 * barra.1) > 0.0,
                "a recta aponta ao contrário da corrente (dobra {passo})"
            );
        }
    }
}
