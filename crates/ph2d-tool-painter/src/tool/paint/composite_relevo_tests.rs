//! Gates do **relevo de cada camada da pilha** ([`super::super::composite_relevo`]). A régua de todos é
//! a mesma: **a ferramenta AVULSA**. O que o Brush, a borracha e o Tiling fazem ao relevo fora da
//! pilha é o que cada camada tem de fazer dentro dela — ao bit.

use super::composite::{CompositeLayer, CompositeOp, EscopoDaBorracha, N_CAMADAS};
use super::*;
use ph2d_painter_brush::{DrawTo, Falloff};

const S: u32 = 128;

fn tela(camadas: &[CompositeOp]) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 0.0,
        falloff: Falloff::Smooth,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_brush_impasto(true);
    t.set_brush_impasto_depth(1.0);
    t.set_brush_falloff(Falloff::Smooth as u8);
    t.paint.composite_enabled = !camadas.is_empty();
    for pos in 0..N_CAMADAS {
        t.paint.composite[pos] = camadas
            .get(pos)
            .map_or_else(CompositeLayer::default, |op| CompositeLayer::nova(*op));
    }
    t
}

fn traco(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2]) {
    t.on_canvas_pointer(cp(de, PointerPhase::Down));
    let n = 30;
    for i in 1..=n {
        let s = i as f32 / n as f32;
        let p = [de[0] + (ate[0] - de[0]) * s, de[1] + (ate[1] - de[1]) * s];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(ate, PointerPhase::Up));
}

fn horizontal(t: &mut PainterTool) {
    traco(t, [20.0, 64.0], [108.0, 64.0]);
}

fn vertical(t: &mut PainterTool) {
    traco(t, [64.0, 30.0], [64.0, 98.0]);
}

/// A altura assente da camada activa, em bits.
fn corpo(t: &PainterTool) -> Vec<u32> {
    let layer = t.layers.active().expect("camada");
    t.heights
        .get(&layer)
        .map(|h| h.iter().map(|v| v.to_bits()).collect())
        .unwrap_or_else(|| vec![0; (S * S) as usize])
}

fn soma(c: &[u32]) -> f32 {
    c.iter().map(|b| f32::from_bits(*b)).sum()
}

/// Igualdade AO BIT, com um veredito legível: quantos texels diferem e as duas somas.
#[track_caller]
fn igual(a: &[u32], b: &[u32], o_que: &str) {
    let n = a.iter().zip(b).filter(|(x, y)| x != y).count();
    assert!(
        n == 0,
        "{o_que}: {n} texels diferem · soma {:.3} contra {:.3}",
        soma(a),
        soma(b)
    );
}

/// **Com duas camadas Brush a pilha deposita o CORPO do pincel avulso, ao bit.** Antes da cura a
/// soma era `0`: os planos forçavam `Draw To = Color` e nenhuma camada chegava ao depósito de altura.
/// O envelope é um máximo, e as duas camadas pousam os mesmos dabs com a mesma força ⇒ o mesmo corpo.
#[test]
fn com_duas_camadas_o_corpo_e_o_do_pincel_avulso() {
    let mut avulso = tela(&[]);
    horizontal(&mut avulso);
    let mut pilha = tela(&[CompositeOp::Brush, CompositeOp::Brush]);
    assert!(pilha.composite_active());
    horizontal(&mut pilha);
    // Controlo: a fixtura contém o fenómeno — o avulso deposita corpo.
    assert!(soma(&corpo(&avulso)) > 100.0);
    igual(
        &corpo(&pilha),
        &corpo(&avulso),
        "a pilha perdeu o corpo do Brush",
    );
}

/// **A cadeia do depósito é POR CAMADA.** Uma camada menor por baixo de uma inteira: o corpo é o da
/// inteira (o envelope é um máximo e o dab menor cabe dentro do maior). Com a cadeia partilhada, a
/// camada de cima ligaria o 1.º dab de cada lote ao ÚLTIMO dab da de baixo, e o corpo dela ficaria
/// uma conta de rosário entre lotes.
#[test]
fn a_cadeia_do_relevo_e_de_cada_camada() {
    let mut avulso = tela(&[]);
    horizontal(&mut avulso);
    let mut pilha = tela(&[CompositeOp::Brush, CompositeOp::Brush]);
    pilha.paint.composite[0].size = 0.5;
    horizontal(&mut pilha);
    igual(
        &corpo(&pilha),
        &corpo(&avulso),
        "a camada inteira perdeu a cadeia dela",
    );
}

/// **Um pincel de só-relevo não pinta COR na pilha** — e deposita o corpo. Antes da cura a tela saía
/// com a tinta das duas camadas (`393 834` níveis) e sem corpo nenhum.
#[test]
fn so_relevo_na_pilha_nao_pinta_cor() {
    let branca = vec![255u8; (S * S * 4) as usize];
    let mut pilha = tela(&[CompositeOp::Brush, CompositeOp::Brush]);
    pilha.paint.brush.impasto_draw_to = DrawTo::Depth;
    horizontal(&mut pilha);
    assert!(
        *pilha.canvas_rgba == branca,
        "a pilha pintou cor com Draw To = Depth"
    );
    let mut avulso = tela(&[]);
    avulso.paint.brush.impasto_draw_to = DrawTo::Depth;
    horizontal(&mut avulso);
    igual(&corpo(&pilha), &corpo(&avulso), "e o corpo é o do avulso");
    // Controlo: com cor e corpo a mesma pilha pinta — senão a primeira metade não distingue nada.
    let mut com_cor = tela(&[CompositeOp::Brush, CompositeOp::Brush]);
    horizontal(&mut com_cor);
    assert!(*com_cor.canvas_rgba != branca);
}

/// **A borracha de escopo `Tudo` morde o relevo ASSENTE como a borracha avulsa, ao bit; a de escopo
/// `Traco` não lhe toca** (ela devolve o `pre`, e o `pre` do relevo assente é ele próprio).
#[test]
fn a_borracha_da_pilha_morde_como_a_avulsa() {
    let pintado = || {
        let mut t = tela(&[]);
        horizontal(&mut t);
        t
    };
    let antes = corpo(&pintado());
    let mut avulsa = pintado();
    avulsa.paint.eraser = true;
    vertical(&mut avulsa);
    // Controlo: a borracha avulsa morde de facto.
    assert_ne!(corpo(&avulsa), antes);
    for (escopo, esperado) in [
        (EscopoDaBorracha::Tudo, corpo(&avulsa)),
        (EscopoDaBorracha::Traco, antes.clone()),
    ] {
        let mut t = pintado();
        t.paint.composite_enabled = true;
        t.paint.composite[0] = CompositeLayer::nova(CompositeOp::Blur);
        t.paint.composite[1] = CompositeLayer::nova(CompositeOp::Erase);
        t.paint.composite[1].erase_scope = escopo;
        assert!(t.composite_active());
        vertical(&mut t);
        igual(&corpo(&t), &esperado, &format!("escopo {escopo:?}"));
    }
}

/// **Com Tiling o corpo embrulha como o do avulso.** A cópia embrulhada liga-se ao antecessor do
/// ORIGINAL dela; sem os grupos ela ligar-se-ia à cópia vizinha na lista, uma barra atravessada.
#[test]
fn com_tiling_o_corpo_embrulha_como_o_do_avulso() {
    let atravessa = |t: &mut PainterTool| {
        t.paint.tiling = [true, true];
        traco(t, [90.0, 50.0], [160.0, 70.0]);
    };
    let mut avulso = tela(&[]);
    atravessa(&mut avulso);
    let mut pilha = tela(&[CompositeOp::Brush, CompositeOp::Brush]);
    atravessa(&mut pilha);
    assert!(soma(&corpo(&avulso)) > 100.0);
    igual(&corpo(&pilha), &corpo(&avulso), "o Tiling");
}

/// Uma pilha com a 2.ª camada quase a zero — arma a pilha e não muda o resultado.
fn pilha_de_uma(op: CompositeOp) -> PainterTool {
    let mut t = tela(&[op, CompositeOp::Brush]);
    t.paint.composite[1].strength = 1e-6;
    t
}

/// **No Impasto a COR da pilha é a do pincel avulso, ao byte** — o pigmento de um pincel que deposita
/// corpo é cortado num FILME, e os planos forçavam `Draw To = Color`, o que desligava o corte: a
/// camada pintava a tinta cheia do Digital (`4 290` bytes diferentes, pior `178`). O CONTROLO é o
/// Digital, que já era idêntico.
#[test]
fn no_impasto_a_cor_da_pilha_e_a_do_avulso() {
    for impasto in [false, true] {
        let mut avulso = tela(&[]);
        let mut pilha = pilha_de_uma(CompositeOp::Brush);
        for t in [&mut avulso, &mut pilha] {
            t.set_brush_impasto(impasto);
            horizontal(t);
        }
        assert!(pilha.composite_active());
        assert!(
            *avulso.canvas_rgba == *pilha.canvas_rgba,
            "impasto={impasto}: a cor da pilha divergiu da do avulso"
        );
        igual(&corpo(&pilha), &corpo(&avulso), "e o corpo");
    }
    // Controlo: o filme existe — sem ele as duas metades acima não distinguiam nada.
    let mut com_corpo = tela(&[]);
    horizontal(&mut com_corpo);
    let mut sem_corpo = tela(&[]);
    sem_corpo.paint.brush.impasto_draw_to = DrawTo::Color;
    horizontal(&mut sem_corpo);
    assert!(*com_corpo.canvas_rgba != *sem_corpo.canvas_rgba);
}

/// **Com Tiling a pilha pinta do OUTRO LADO da costura, ao byte.** A região da pilha era medida com
/// os dabs crus, e um lote com o cursor já para lá da borda saía cedo: a borda esquerda ficava com
/// `0` px de cor contra `738` do avulso.
///
/// ⚠️ **Sem grão, de propósito:** com grão ALEATÓRIO a pilha diverge do avulso mesmo LONGE da costura
/// (sonda `diag_o_tiling_ao_byte`: `3 393` bytes sem atravessar), porque cada camada tem o SEU fluxo
/// aleatório (`rng_camada`) — desenho declarado da pilha, não defeito do Tiling. Com grão o gate
/// mediria o fluxo e não a costura.
#[test]
fn com_tiling_a_pilha_pinta_do_outro_lado_da_costura() {
    let corre = |t: &mut PainterTool| {
        t.set_brush_impasto(false);
        t.paint.tiling = [true, true];
        traco(t, [90.0, 50.0], [160.0, 70.0]);
    };
    let mut avulso = tela(&[]);
    corre(&mut avulso);
    let mut pilha = pilha_de_uma(CompositeOp::Brush);
    corre(&mut pilha);
    let esquerda = |t: &PainterTool| {
        (0..S)
            .flat_map(|y| (0..40u32).map(move |x| ((y * S + x) * 4) as usize))
            .filter(|&i| t.canvas_rgba[i] < 250)
            .count()
    };
    assert!(esquerda(&avulso) > 300, "a fixtura atravessa a costura");
    assert!(
        *avulso.canvas_rgba == *pilha.canvas_rgba,
        "a pilha perdeu a cor do outro lado da costura (esquerda: {} contra {})",
        esquerda(&pilha),
        esquerda(&avulso)
    );
}

/// **Numa tela em Tiling, o traço deslocado de UMA LARGURA pinta a MESMA imagem** — a tela é um toro,
/// e a pilha tem de o saber com grão ALEATÓRIO. Deslocado, cada dab fica fora da tela e só a cópia
/// embrulhada pinta: com os GRUPOS as duas partilham a moldura do dab, e o fluxo avança uma vez por
/// dab como no traço de dentro; sem eles cada cópia tirava a sua, e o grão do traço inteiro mudava.
///
/// ⚠️ É a régua dos grupos no depósito da pilha: a paridade com o avulso não a pode ser, porque cada
/// camada tem o seu próprio fluxo aleatório (`rng_camada`) — ver o gate do Tiling acima.
#[test]
fn com_tiling_o_traco_deslocado_de_uma_tela_pinta_o_mesmo() {
    let corre = |dx: f32| {
        let mut t = pilha_de_uma(CompositeOp::Brush);
        t.set_brush_impasto(false);
        t.paint.brush.texture.kind = ph2d_painter_brush::TextureKind::Noise;
        t.paint.brush.texture.mapping = ph2d_painter_brush::TextureMapping::Random;
        t.paint.tiling = [true, true];
        traco(&mut t, [30.0 + dx, 50.0], [90.0 + dx, 70.0]);
        t
    };
    let (dentro, deslocado) = (corre(0.0), corre(S as f32));
    assert!(
        dentro.canvas_rgba.iter().any(|&b| b != 255),
        "a fixtura pinta"
    );
    let dif = dentro
        .canvas_rgba
        .iter()
        .zip(deslocado.canvas_rgba.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        dif, 0,
        "o traço deslocado de uma tela pintou outra imagem ({dif} bytes)"
    );
}

/// Uma pilha Erase (topo, dura e `1,5×` maior — cobre a pegada inteira do Brush) sobre um Brush.
fn borracha_cheia_sobre_brush(escopo: EscopoDaBorracha, borracha_em_cima: bool) -> PainterTool {
    let camadas = if borracha_em_cima {
        [CompositeOp::Erase, CompositeOp::Brush]
    } else {
        [CompositeOp::Brush, CompositeOp::Erase]
    };
    let mut t = tela(&camadas);
    let e = usize::from(!borracha_em_cima);
    t.paint.composite[e].erase_scope = escopo;
    t.paint.composite[e].hardness = Some(1.0);
    t.paint.composite[e].size = 1.5;
    t
}

/// A cobertura assente da camada activa (o FILME que a luz pesa).
fn filme(t: &PainterTool) -> u64 {
    let layer = t.layers.active().expect("camada");
    t.covers
        .get(&layer)
        .map_or(0, |c| c.iter().map(|v| u64::from(*v)).sum())
}

/// ⭐ **8b — a borracha POR CIMA de um Brush apaga o CORPO que ele pôs neste traço**, não só a cor.
/// Antes da cura a tinta ia a `0` e o relevo ficava em `974,24` — o de um Brush sozinho: um corpo
/// sem tinta nenhuma, que a luz sombreava. Nos dois escopos, e o FILME vai com ele.
///
/// O CONTROLO é a mesma borracha POR BAIXO do Brush: ali ela não toca no traço, e o corpo é o do
/// pincel avulso ao bit — senão a primeira metade passaria com uma borracha que apaga tudo.
#[test]
fn a_borracha_de_cima_apaga_o_corpo_do_brush_de_baixo() {
    let mut avulso = tela(&[]);
    horizontal(&mut avulso);
    assert!(soma(&corpo(&avulso)) > 100.0, "a fixtura deposita corpo");
    for escopo in [EscopoDaBorracha::Traco, EscopoDaBorracha::Tudo] {
        let mut t = borracha_cheia_sobre_brush(escopo, true);
        assert!(t.corpo_por_camada());
        horizontal(&mut t);
        assert_eq!(
            soma(&corpo(&t)),
            0.0,
            "{escopo:?}: a borracha de cima deixou o corpo do Brush"
        );
        assert_eq!(filme(&t), 0, "{escopo:?}: e o filme");
        let mut baixo = borracha_cheia_sobre_brush(escopo, false);
        assert!(!baixo.corpo_por_camada());
        horizontal(&mut baixo);
        igual(
            &corpo(&baixo),
            &corpo(&avulso),
            &format!("{escopo:?}: a borracha de BAIXO tocou no corpo do Brush"),
        );
    }
    // A cor da de cima foi-se de facto (escopo `Traco` devolve o `pre`, que é branco).
    let mut t = borracha_cheia_sobre_brush(EscopoDaBorracha::Traco, true);
    horizontal(&mut t);
    assert!(t.canvas_rgba.iter().all(|&b| b == 255));
}

/// **O que a luz mostra A MEIO do traço já é o corpo apagado** — a recomposição é por EVENTO, não
/// só ao soltar. Com o envelope a recompor-se só no commit, o artista veria o corpo fantasma enquanto
/// arrasta e ele desapareceria ao largar.
#[test]
fn a_meio_do_traco_o_corpo_ja_vem_apagado() {
    let a_meio = |em_cima: bool| {
        let mut t = borracha_cheia_sobre_brush(EscopoDaBorracha::Traco, em_cima);
        t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
        for i in 1..=15 {
            t.on_canvas_pointer(cp([20.0 + 3.0 * i as f32, 64.0], PointerPhase::Move));
        }
        t.paint.relief.stroke_height.iter().sum::<f32>()
    };
    assert!(
        a_meio(false) > 50.0,
        "controlo: sem borracha por cima há corpo vivo"
    );
    assert_eq!(a_meio(true), 0.0, "o corpo vivo não foi apagado");
}

/// **A borracha MACIA tira TINTA, e o corpo deriva-se do que sobra** — fica entre nada e o todo, e
/// é o que a derivação da tinta RESTANTE dá (o commit re-deriva o corpo da tinta; uma borracha que
/// multiplicasse só a altura seria desfeita ali).
#[test]
fn a_borracha_macia_de_cima_deixa_o_corpo_da_tinta_que_sobra() {
    let mut avulso = tela(&[]);
    horizontal(&mut avulso);
    let todo = soma(&corpo(&avulso));
    let mut t = tela(&[CompositeOp::Erase, CompositeOp::Brush]);
    horizontal(&mut t);
    let parte = soma(&corpo(&t));
    assert!(
        parte > 0.0 && parte < 0.5 * todo,
        "a borracha macia deixou {parte:.2} de {todo:.2}"
    );
}

/// **A pergunta tem UMA porta, e só uma Erase VIVA por CIMA de um Brush VIVO a arma** — um Blur por
/// cima, uma borracha por baixo ou uma borracha a zero deixam o envelope partilhado (o caminho de
/// sempre, byte-idêntico).
#[test]
fn so_uma_borracha_viva_por_cima_arma_o_corpo_por_camada() {
    use CompositeOp::{Blur, Brush, Erase};
    assert!(tela(&[Erase, Brush]).corpo_por_camada());
    assert!(tela(&[Erase, Blur, Brush]).corpo_por_camada());
    assert!(!tela(&[Blur, Brush]).corpo_por_camada());
    assert!(!tela(&[Brush, Erase]).corpo_por_camada());
    let mut a_zero = tela(&[Erase, Brush]);
    a_zero.paint.composite[0].strength = 0.0;
    assert!(!a_zero.corpo_por_camada());
}

/// ⭐⭐ **O volume assente sobrevive à última composição da pilha** — report do dono (2026-09-30,
/// com fotos): pilha Blur/Smear/Brush no Impasto, e o corpo do traço NOVO sumia num RECTÂNGULO
/// depois de soltar, só por cima de tinta com volume.
///
/// O mecanismo: o pen-up à mão livre (`commit_drag_preview`) assentava o volume ANTES de compor a
/// região pendente do quadro, e a camada `Smear` dessa composição reescreve o relevo da camada a
/// partir da cópia congelada no início do traço — sem este traço. Medido: `588 929` de corpo contra
/// `782 492` da rota por evento, com um rectângulo a ZERO.
///
/// A régua: o MESMO par de traços com a composição por quadro (a do app) e por evento dá o mesmo
/// relevo AO BIT. O CONTROLO é que a fixtura chega ao soltar com composição PENDENTE — sem isso as
/// duas rotas seriam a mesma e a igualdade não afirmaria nada.
#[test]
fn a_ultima_composicao_nao_apaga_o_corpo_assente() {
    let corre = |por_quadro: bool| -> (Vec<u32>, bool) {
        let mut t = tela(&[CompositeOp::Blur, CompositeOp::Smear, CompositeOp::Brush]);
        t.set_compor_por_quadro(por_quadro);
        t.paint.composite[0].size = 1.339;
        let risca = |t: &mut PainterTool, de: [f32; 2], ate: [f32; 2]| -> bool {
            t.on_canvas_pointer(cp(de, PointerPhase::Down));
            for i in 1..=30 {
                let s = i as f32 / 30.0;
                t.on_canvas_pointer(cp(
                    [de[0] + (ate[0] - de[0]) * s, de[1] + (ate[1] - de[1]) * s],
                    PointerPhase::Move,
                ));
                if i % 3 == 0 {
                    t.compoe_o_pendente();
                }
            }
            t.on_canvas_pointer(cp([ate[0] + 2.0, ate[1]], PointerPhase::Move));
            let pendente = t.paint.pilha.pendente.is_some();
            t.on_canvas_pointer(cp(ate, PointerPhase::Up));
            pendente
        };
        risca(&mut t, [20.0, 64.0], [108.0, 64.0]);
        let pendente = risca(&mut t, [64.0, 20.0], [64.0, 108.0]);
        (corpo(&t), pendente)
    };
    let (por_evento, _) = corre(false);
    let (por_quadro, pendente) = corre(true);
    assert!(
        pendente,
        "controlo: a fixtura tem de chegar ao soltar com composição pendente"
    );
    assert!(
        soma(&por_evento) > 100.0,
        "controlo: a fixtura deposita corpo"
    );
    igual(
        &por_quadro,
        &por_evento,
        "a última composição apagou o corpo assente",
    );
}
