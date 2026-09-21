//! Os gates da pilha de **CINCO camadas** com cor e tamanho por camada (ordem do dono, 2026-09-20).
//!
//! ⚠️ **Todos medem o BARRO pela porta do produto** (`on_canvas_pointer`), nunca a tabela de
//! camadas: uma tabela é um resumo da lei, e um resumo não tem de conter tudo (a lição que o censo
//! dos knobs da escultura pagou). O que cada gate afirma está no `///` dele.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

const SIZE: u32 = 200;

fn cp5(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma ferramenta de disco duro sobre papel branco — a fixtura determinista dos gates.
fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = 8.0;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.color = [1.0, 0.0, 0.0];
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    t
}

fn arma(t: &mut PainterTool, ops: &[(CompositeOp, f32)]) {
    for pos in 0..composite::N_CAMADAS {
        match ops.get(pos) {
            Some(&(op, s)) => {
                t.paint.composite[pos] = CompositeLayer {
                    op,
                    strength: s,
                    ..CompositeLayer::default()
                }
            }
            None => t.paint.composite[pos] = CompositeLayer::default(),
        }
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp5([40.0, y], PointerPhase::Down));
    let mut x = 40.0f32;
    while x < 160.0 {
        x += 2.0;
        t.on_canvas_pointer(cp5([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp5([160.0, y], PointerPhase::Up));
}

fn texel(t: &PainterTool, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * SIZE + x) * 4) as usize;
    [
        t.canvas_rgba[i],
        t.canvas_rgba[i + 1],
        t.canvas_rgba[i + 2],
        t.canvas_rgba[i + 3],
    ]
}

/// **A pilha tem CINCO posições, e as duas novas nascem CALADAS.**
///
/// As duas metades, porque cada uma sozinha mente: a pilha corre mesmo cinco posições (arma-se a
/// quinta e o barro muda), e com os valores de fábrica o traço é **byte-idêntico** ao de uma pilha
/// de três — que é o que faz esta extensão não mexer no desenho de quem já a usava.
#[test]
fn a_pilha_tem_cinco_posicoes_e_as_duas_novas_nascem_caladas() {
    assert_eq!(composite::N_CAMADAS, 5);

    // Metade A — de fábrica, a 4.ª e a 5.ª não mexem num byte.
    let mut fabrica = tela();
    traco(&mut fabrica, 100.0);
    let mut so_tres = tela();
    for pos in 3..composite::N_CAMADAS {
        so_tres.paint.composite[pos].strength = 0.0;
    }
    traco(&mut so_tres, 100.0);
    assert_eq!(
        fabrica.canvas_rgba, so_tres.canvas_rgba,
        "de fábrica a 4.ª e a 5.ª camadas TÊM de ser inertes"
    );

    // Metade B — a quinta posição EXISTE: armada, ela pinta.
    //
    // ⚠️⚠️ **A 1.ª redacção desta metade não continha o fenómeno, e reprovou sobre um motor
    // CORRECTO.** A posição 5 é o FUNDO da pilha e corre PRIMEIRO; com o Brush do topo a Strength
    // `1,0` — opaco — ele cobre-a exactamente, e as duas telas saíam iguais ao bit. *Estava certo, e
    // a fixtura é que não tinha onde a ver.* ⇒ a camada de baixo leva um tamanho `2×`, logo ela
    // pinta uma faixa mais LARGA do que a de cima consegue tapar, e a diferença mora nas orlas.
    let com_fundo = |strength: f32| {
        let mut t = tela();
        arma(
            &mut t,
            &[
                (CompositeOp::Brush, 1.0),
                (CompositeOp::Smear, 0.5),
                (CompositeOp::Blur, 0.5),
                (CompositeOp::Brush, 0.0),
                (CompositeOp::Brush, strength),
            ],
        );
        t.paint.composite[4].color = Some([0.0, 0.0, 1.0]);
        t.paint.composite[4].size = 2.0;
        traco(&mut t, 100.0);
        t
    };
    let ligada = com_fundo(1.0);
    let desligada = com_fundo(0.0);
    assert_ne!(
        ligada.canvas_rgba, desligada.canvas_rgba,
        "a 5.ª posição existe mas o motor não a corre"
    );
    // E o que ela pintou é a COR dela, na orla que o Brush do topo não alcança.
    let orla = texel(&ligada, 100, 100 - 12);
    assert!(
        orla[2] > 200 && orla[0] < 90,
        "a orla da camada de baixo tinha de ser AZUL e é {orla:?}"
    );
}

/// ⛔⛔ **UM SEGUNDO BRUSH DEPOSITA — e antes desta wave ele não depositava nada.**
///
/// Medido em 2026-09-20, antes da cura: com Strength `0,6`, UM Brush deixava `61,08` de tinta e
/// DOIS deixavam `61,04` — *o cap de Accumulate é um `stroke_mask` por TRAÇO, e a primeira camada
/// levava-o ao tecto*. O número lido era exactamente o cap (`0,6 × 0,4 × 255 = 61,2`), quando dois
/// passes dariam `1 − 0,4² = 0,84` ⇒ `~86`.
///
/// ⚠️ **A régua é a TINTA e o CONTROLO é a mesma pilha com uma camada só** — sem ele um número
/// sozinho não diz se a segunda camada trabalhou.
///
/// **Mutação que tem de sangrar:** apagar o `mem::swap` do `stroke_mask` em
/// [`super::composite::PainterTool::stamp_dabs_composite`] ⇒ as duas colunas voltam a ser iguais.
#[test]
fn um_segundo_brush_na_pilha_deposita_a_tinta_dele() {
    let tinta = |ops: &[(CompositeOp, f32)]| -> f64 {
        let mut t = tela();
        arma(&mut t, ops);
        traco(&mut t, 100.0);
        (50u32..150)
            .map(|x| f64::from(255 - texel(&t, x, 100)[1])) // o VERDE cai com tinta vermelha
            .sum::<f64>()
            / 100.0
    };
    let um = tinta(&[(CompositeOp::Brush, 0.6)]);
    let dois = tinta(&[(CompositeOp::Brush, 0.6), (CompositeOp::Brush, 0.6)]);
    // Dois passes de 0,6 compõem para `1 − 0,4² = 0,84`; um só dá `0,6`. A barra fica no meio do
    // vale, longe dos dois lados, e é uma FRACÇÃO — uma barra absoluta mediria a cor da fixtura.
    assert!(
        dois > um * 1.25,
        "o 2.º Brush não depositou: {dois:.2} contra {um:.2} de uma camada só \
         (dois passes de 0,6 têm de compor para ~0,84 da cobertura)"
    );
}

/// **A COR de uma camada chega ao barro, e o valor de fábrica SEGUE o pincel.**
///
/// As duas metades: sem a primeira o campo é decoração; sem a segunda, uma cor autorada por
/// omissão teria apagado em silêncio o caminho por onde o Randomize Color passa.
#[test]
fn a_cor_da_camada_chega_ao_barro_e_o_default_segue_o_pincel() {
    // Metade A — autorada: a camada pinta a cor DELA, não a do pincel.
    let mut azul = tela();
    arma(&mut azul, &[(CompositeOp::Brush, 1.0)]);
    azul.paint.composite[0].color = Some([0.0, 0.0, 1.0]);
    traco(&mut azul, 100.0);
    let p = texel(&azul, 100, 100);
    assert!(
        p[2] > 200 && p[0] < 60,
        "a camada tinha de pintar AZUL e pintou {p:?}"
    );

    // Metade B — sem cor autorada ela segue o pincel (que é vermelho nesta fixtura).
    let mut segue = tela();
    arma(&mut segue, &[(CompositeOp::Brush, 1.0)]);
    traco(&mut segue, 100.0);
    let q = texel(&segue, 100, 100);
    assert!(
        q[0] > 200 && q[2] < 60,
        "sem cor autorada a camada tinha de seguir o pincel (vermelho) e pintou {q:?}"
    );
}

/// **O TAMANHO de uma camada chega ao barro — e a lista dela ENCOLHE quando ela cresce.**
///
/// ⚠️ A segunda metade é a que separa esta implementação da barata: reutilizar a lista inteira num
/// carimbo `4×` maior custa `×4` (medido). Aqui o gate conta os dabs que a camada de facto recebe
/// pela porta [`super::composite::PainterTool::camada_dabs`], e exige que ela SUBAMOSTRE.
#[test]
fn o_tamanho_da_camada_chega_ao_barro_e_a_lista_dela_encolhe() {
    // Metade A — o traço fica mais LARGO.
    let largura = |mult: f32| -> u32 {
        let mut t = tela();
        arma(&mut t, &[(CompositeOp::Brush, 1.0)]);
        t.paint.composite[0].size = mult;
        traco(&mut t, 100.0);
        (0u32..SIZE).filter(|&y| texel(&t, 100, y)[1] < 200).count() as u32
    };
    let (um, dois) = (largura(1.0), largura(2.0));
    assert!(
        dois >= um * 2 - 2,
        "uma camada 2× tinha de pintar uma faixa ~2× mais larga: {dois} contra {um}"
    );

    // Metade B — e ela recebe MENOS dabs, que é o que a torna barata.
    let mut t = tela();
    arma(&mut t, &[(CompositeOp::Brush, 1.0)]);
    let dabs: Vec<ph2d_painter_brush::Dab> = (0..40u16)
        .map(|i| ph2d_painter_brush::Dab {
            center: [40.0 + f32::from(i) * 1.6, 100.0],
            radius_px: 8.0,
            coverage: 1.0,
            color: [1.0, 0.0, 0.0],
            rotation: [1.0, 0.0],
            dir: [1.0, 0.0],
            arc_len: f32::from(i) * 1.6,
            stroke_radius_px: 8.0,
        })
        .collect();
    t.paint.composite[0].size = 4.0;
    let subamostrados = t
        .camada_dabs_para_teste(0, &dabs)
        .map_or(dabs.len(), |v| v.len());
    assert!(
        subamostrados * 3 < dabs.len(),
        "uma camada 4× tinha de subamostrar a lista e ficou com {subamostrados} de {}",
        dabs.len()
    );
}

/// **Uma camada `Erase` APAGA — e não tinge o que apagou.**
///
/// A segunda metade é a cura do pigmento de 2026-09-20 vista daqui: o `EraseAlpha` devolve o RGB do
/// destino letra por letra, e uma camada que o tingisse deixaria a cor do pincel num modo em que o
/// artista nem a vê.
#[test]
fn uma_camada_erase_apaga_e_nao_tinge() {
    let mut t = tela();
    // Primeiro pinta VERDE com o pincel normal, depois apaga metade com a pilha.
    t.paint.composite_enabled = false;
    t.paint.brush.color = [0.0, 1.0, 0.0];
    traco(&mut t, 100.0);
    let antes = texel(&t, 100, 100);
    assert_eq!(antes[3], 255, "a fixtura tinha de deixar tinta opaca");

    t.paint.composite_enabled = true;
    t.paint.brush.color = [1.0, 0.0, 0.0]; // o pincel é VERMELHO: se a borracha tingir, vê-se
    arma(&mut t, &[(CompositeOp::Erase, 1.0)]);
    traco(&mut t, 100.0);
    let depois = texel(&t, 100, 100);
    assert!(
        depois[3] < 40,
        "a camada Erase tinha de apagar o alfa e ele ficou em {}",
        depois[3]
    );
    assert_eq!(
        [depois[0], depois[1], depois[2]],
        [antes[0], antes[1], antes[2]],
        "a borracha TINGIU o que apagou"
    );
}

/// **Os dois extremos do tamanho são DERIVADOS, e o piso é o `spacing` do pincel** (§0.0).
///
/// O vão entre dois dabs é `spacing × 2r`: uma camada de raio `size × r` só deixa de os sobrepor
/// quando `size < spacing`, e é exactamente aí que a lista partilhada sairia em CONTAS. ⇒ mexer no
/// Spacing MOVE o piso, e o gate mede isso — uma constante escrita à mão não se moveria.
#[test]
fn o_piso_do_tamanho_e_o_spacing_e_o_tecto_e_o_medido() {
    let mut t = tela();
    arma(&mut t, &[(CompositeOp::Brush, 1.0)]);
    t.paint.composite[0].size = 0.0; // pede o impossível
    t.paint.brush.spacing = 0.10;
    assert!((t.tamanho_da_camada(0) - 0.10).abs() < 1e-6);
    // ⭐ O piso ANDA com o Spacing — é isto que prova que ele é derivado e não um literal.
    t.paint.brush.spacing = 0.25;
    assert!((t.tamanho_da_camada(0) - 0.25).abs() < 1e-6);
    // E o tecto é o número MEDIDO (o relógio de um quadro), não o que a pista aceitaria.
    t.set_composite_layer_size(0, 99.0);
    assert!((t.tamanho_da_camada(0) - composite::MAX_TAMANHO_DA_CAMADA).abs() < 1e-6);
}

/// **O chip da operação CICLA as quatro, e a volta FECHA** — sem ele as duas posições novas
/// nasceriam presas ao que o default declarou, que é a forma exacta do controlo inalcançável.
///
/// ⚠️⚠️ **A 1.ª redacção afirmava só «viu as quatro», e uma MUTAÇÃO SOBREVIVEU a ela:** com o
/// ciclo a `% 3` e a fixtura a partir do `Erase`, as quatro primeiras amostras ainda são quatro
/// operações distintas — só que o `Erase` deixa de ser alcançável DEPOIS de se sair dele. *Ver
/// todos os estados uma vez não é a mesma propriedade que o ciclo fechar*, e é a segunda que torna
/// um chip usável. ⇒ hoje a sequência é afirmada **por igualdade, na ordem, e de volta ao início**.
#[test]
fn o_chip_da_operacao_cicla_as_quatro_e_a_volta_fecha() {
    let mut t = tela();
    let id = crate::ids::PAINTER_BRUSH_COMPOSITE_OP[3];
    t.set_composite_layer_op(3, 0); // parte de um estado CONHECIDO
    let mut vistas = vec![t.paint.composite[3].op];
    for _ in 0..4 {
        assert!(
            t.route_composite_event(&ph2d_editor_core::tool::PanelEvent::Click(id)),
            "o chip da operação não foi consumido"
        );
        vistas.push(t.paint.composite[3].op);
    }
    assert_eq!(
        vistas,
        vec![
            CompositeOp::Brush,
            CompositeOp::Smear,
            CompositeOp::Blur,
            CompositeOp::Erase,
            CompositeOp::Brush,
        ],
        "o ciclo do chip tem de passar pelas quatro e VOLTAR ao princípio"
    );
}

/// **A volta à cor do pincel é alcançável** — o `PanelEvent` é contrato congelado e não tem
/// clique-direito, logo sem este botão o `None` (quem mantém o Randomize Color vivo) seria
/// alcançável só até ao primeiro clique na amostra.
#[test]
fn a_volta_a_cor_do_pincel_e_alcancavel() {
    let mut t = tela();
    t.set_composite_layer_color(1, [0.2, 0.4, 0.8]);
    assert!(t.composite_color_authored()[1]);
    let id = crate::ids::PAINTER_BRUSH_COMPOSITE_COLOR_CLEAR[1];
    assert!(t.route_composite_event(&ph2d_editor_core::tool::PanelEvent::Click(id)));
    assert!(
        !t.composite_color_authored()[1],
        "o botão de volta não devolveu a camada à cor do pincel"
    );
}
