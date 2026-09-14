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

/// A espessura da linha da corrente — **a mesma do losango**, e não um número próprio.
///
/// ⛔⛔ **Era `7.0` e foi um defeito reportado com foto** (Enio, 2026-09-14: *«linha muito grossa»*).
/// Aquele número existia para ela se ler como uma FAIXA por baixo do osso; desde que ela deixou de
/// passar por cima dos ossos, uma faixa larga não descreve nada — é uma linha, e a linha desta
/// família já tem largura. ⚠️ **Um `const` próprio aqui seria a segunda resposta à mesma pergunta**,
/// e a que envelhece.
const CHAIN_LINE_PX: f64 = GOAL_LINE_PX;

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
/// ⛔⛔ **E ela é a RECTA ENTRE AS DUAS PONTAS da corrente, nunca a polilinha das juntas** (2.º
/// report do dono, 2026-09-14: *«deve ser uma linha reta e não passar por dentro dos ossos»*). A 1.ª
/// redacção traçava a polilinha: numa corrente quase esticada ela caía **exactamente** sobre os
/// corpos dos ossos e lia-se como parte deles, e numa dobrada serpenteava. Numa corrente dobrada a
/// corda passa por FORA dos ossos sozinha — é a corda de um arco.
///
/// ⛔⛔ **E ela NÃO se desloca** (3.º report, com foto: *«linha muito grossa e deslocada das pontas
/// dos ossos»*). A 2.ª redacção afastava-a para o lado livre para garantir folga em toda pose — e o
/// preço foi ela deixar de **tocar** as duas coisas que descreve: a junta onde o `Chain` pára e o
/// losango do alvo. *Um indicador de extensão que não encosta nas pontas não diz qual extensão é.*
/// ⇒ ela liga as duas pontas, e a folga em pose esticada paga-se sendo uma **linha fina** em vez de
/// uma faixa. A geometria vive na porta [`chain_bar`], que é o que os gates medem.
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
                &Stroke::new(CHAIN_LINE_PX),
                Affine::IDENTITY,
                // ⚠️ **A cheio, e a meia opacidade morreu com a faixa**: ela existia porque uma
                // faixa de `7 px` por cima da arte esconderia o que o artista está a posar. Uma
                // linha da largura do losango não esconde nada, e a meio tom ela desaparecia.
                &Brush::Solid(cor),
                None,
                &faixa,
            );
        }
        // ⭐ **A marca da RAIZ da corrente** — é ali que o `Chain` pára, e é a única parte do
        // realce que carrega informação nova: sem ela, duas correntes que se sobrepõem lêem-se
        // como uma. ⚠️ Ela fica na RAIZ de verdade, não na ponta da recta deslocada: o que ela
        // responde é *qual junta é a fronteira*, e a recta responde *até onde*.
        if let Some(&p) = pontos.first() {
            // ⚠️ **A cruz é INSCRITA na bolinha da junta que ela marca** — `r` é o raio daquela
            // bolinha dividido por `√2`, logo os quatro braços tocam-lhe a borda. ⛔ Um número
            // próprio aqui desalinharia a marca da alça no dia em que o raio da bolinha mudasse, e
            // ele já muda com o comprimento do osso.
            let comp = pontos.get(1).map_or(0.0, |q| (q.x - p.x).hypot(q.y - p.y));
            let r = crate::joint_radius_px(comp) / std::f64::consts::SQRT_2;
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

/// ⭐⭐⭐ **A RECTA QUE DIZ ATÉ ONDE O `Chain` CHEGA** — da junta onde a corrente começa à ponta
/// dela, em píxeis de ECRÃ.
///
/// ⚠️ **Ela toca as duas pontas, e é isso que ela promete** (3.º report do dono, 2026-09-14): as
/// duas coisas que ela descreve são a junta onde o `Chain` pára (marcada com a cruz) e a ponta da
/// corrente (onde vive o losango do alvo). Deslocada, ela deixa de as tocar e o artista fica sem
/// saber qual extensão ela mede.
///
/// ⛔⛔ **A 2.ª redacção afastava-a para o lado livre**, por fora da excursão da corrente, para
/// garantir folga contra os ossos em TODA pose. Está **medido e recusado por veredito de produto**:
/// a folga que ela comprava não valia as pontas que ela perdia. A folga que fica é a de uma linha
/// **fina** — e numa corrente dobrada a corda passa por fora dos ossos sozinha, porque é a corda de
/// um arco.
///
/// ⚠️ `None` quando a raiz e a ponta coincidem: ali não há recta, e inventar uma direcção seria ler
/// ruído de `f64`.
#[must_use]
pub fn chain_bar(pontos: &[Point]) -> Option<(Point, Point)> {
    let (&a, &b) = (pontos.first()?, pontos.last()?);
    if (b.x - a.x).hypot(b.y - a.y) <= f64::EPSILON {
        return None;
    }
    Some((a, b))
}

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

    /// ⭐⭐⭐ **A RECTA ENCOSTA NAS DUAS PONTAS DA CORRENTE** — o 3.º report do dono, dito como
    /// número (*«linha muito grossa e deslocada das pontas dos ossos»*).
    ///
    /// ⛔⛔ **A 2.ª redacção deslocava-a** para o lado livre, para garantir folga contra os ossos em
    /// toda pose — e o preço foi ela deixar de tocar as duas coisas que descreve: a junta onde o
    /// `Chain` pára (a cruz) e a ponta da corrente (o losango do alvo). *Um indicador de extensão
    /// que não encosta nas pontas não diz qual extensão é.*
    ///
    /// ⚠️ A igualdade é EXACTA de propósito: qualquer deslocamento, por pequeno que seja, é o
    /// defeito reportado.
    #[test]
    fn the_chain_bar_ends_on_the_chain_ends() {
        let mut casos = 0;
        for n in [2usize, 3, 5, 8] {
            for comp in [24.0_f64, 107.52, 192.0] {
                for passo in [0.0_f64, 0.2, 0.5, 0.9, -0.3, -0.8] {
                    let pts = corrente(n, comp, passo);
                    let Some((a, b)) = chain_bar(&pts) else {
                        continue;
                    };
                    casos += 1;
                    assert_eq!(
                        (a.x, a.y),
                        (pts[0].x, pts[0].y),
                        "com {n} ossos de {comp} px a dobrar {passo} rad, a recta não começa na \
                         RAIZ da corrente"
                    );
                    let ponta = pts[pts.len() - 1];
                    assert_eq!(
                        (b.x, b.y),
                        (ponta.x, ponta.y),
                        "com {n} ossos de {comp} px a dobrar {passo} rad, a recta não acaba na \
                         PONTA da corrente"
                    );
                }
            }
        }
        assert!(casos >= 60, "o corpus encolheu para {casos} casos");
    }

    /// ⭐⭐ **E ela é uma CORDA, não a polilinha das juntas** — a metade que impede a volta ao 1.º
    /// defeito reportado (*«não passar por dentro dos ossos»*).
    ///
    /// ⚠️ Numa corrente DOBRADA as juntas do meio ficam fora da corda, e é isso que faz a corda
    /// passar por fora dos ossos sozinha. A barra é o **corpo** do osso do meio
    /// ([`crate::bone_half_width_px`]), lido do produto: abaixo dela a recta estaria por dentro
    /// dele.
    #[test]
    fn the_chain_bar_is_a_chord_not_the_joint_polyline() {
        for passo in [0.3_f64, 0.5, 0.9, -0.4] {
            let pts = corrente(5, 107.52, passo);
            let (a, b) = chain_bar(&pts).expect("a corrente tem extensão");
            let meia = crate::bone_half_width_px(107.52);
            for (i, q) in pts.iter().enumerate().take(pts.len() - 1).skip(1) {
                let d = dist_ao_segmento(*q, a, b);
                assert!(
                    d > meia,
                    "a junta {i} está a {d} px da recta (o corpo do osso ocupa {meia}): a recta \
                     está a seguir as juntas em vez de as atravessar (dobra {passo})"
                );
            }
        }
    }
}
