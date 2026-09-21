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

/// **A posição mais FUNDA da pilha existe, e o motor corre-a.**
///
/// ⛔⛔ **Este gate tinha duas metades e uma morreu por ordem do dono (2026-09-21).** Ele chamava-se
/// `a_pilha_tem_cinco_posicoes_e_as_duas_novas_nascem_caladas` e afirmava (a) `N_CAMADAS == 5` e
/// (b) que com os valores de FÁBRICA o traço era byte-idêntico ao de uma pilha de três. As duas
/// premissas caíram no mesmo dia: o tecto passou a ser a SOMA das quotas (`7`) e **não há mais
/// pilha de fábrica** — ela nasce vazia.
///
/// ⭐ O que fica é a metade que ainda afirma alguma coisa: *a última posição do máximo não é
/// decoração*. ⚠️ E a fixtura dela foi curada uma vez e a cura FICA: com o Brush do topo a `1,0`
/// (opaco) ele cobre a de baixo exactamente e as duas telas saem iguais ao bit — *estava certo, e
/// a fixtura é que não tinha onde o ver*; por isso a de baixo leva um tamanho `2×` e a diferença
/// mora nas orlas.
#[test]
fn a_posicao_mais_funda_da_pilha_e_corrida() {
    assert_eq!(
        composite::MAX_CAMADAS,
        7,
        "o tecto é a SOMA das quotas do dono (3 Brush + 2 Erase + 1 Blur + 1 Smear)"
    );
    let fundo = composite::MAX_CAMADAS - 1;
    let com_fundo = |strength: f32| {
        let mut t = tela();
        let mut ops = vec![(CompositeOp::Brush, 1.0f32); composite::MAX_CAMADAS];
        ops[fundo] = (CompositeOp::Brush, strength);
        for o in ops.iter_mut().take(fundo).skip(1) {
            *o = (CompositeOp::Brush, 0.0);
        }
        arma(&mut t, &ops);
        t.paint.composite[fundo].color = Some([0.0, 0.0, 1.0]);
        t.paint.composite[fundo].size = 2.0;
        traco(&mut t, 100.0);
        t
    };
    let ligada = com_fundo(1.0);
    let desligada = com_fundo(0.0);
    assert_ne!(
        ligada.canvas_rgba, desligada.canvas_rgba,
        "a posição {fundo} existe mas o motor não a corre"
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

/// ⭐⭐⭐ **A PILHA MONTA-SE À MÃO: nasce vazia, o `+` cria e o `x` retira** (ordem do dono,
/// 2026-09-21).
///
/// ⛔⛔ **Este gate substitui o `o_chip_da_operacao_cicla_as_quatro_e_a_volta_fecha`, e a premissa
/// dele morreu por ordem de produto.** Ele afirmava que o chip da operação ciclava as quatro e
/// fechava a volta — *«sem ele as duas posições novas nasceriam presas ao que o default
/// declarou»*. Com as QUOTAS (`3 Brush · 2 Erase · 1 Blur · 1 Smear`) um ciclo livre torna a quota
/// uma mentira: dois cliques punham dois Blurs na pilha. A operação passa a ser escolhida na
/// CRIAÇÃO, e o chip saiu.
///
/// ⚠️ O gesto medido é o do PRODUTO (`route_composite_event`), nunca os `set_*` internos.
#[test]
fn a_pilha_nasce_vazia_e_monta_se_pelo_mais_e_pelo_x() {
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = tela();
    assert_eq!(t.composite_len(), 0, "a pilha tem de NASCER vazia");

    let cria = |t: &mut PainterTool, op: u8| {
        assert!(
            t.route_composite_event(&PanelEvent::SelectOption(
                crate::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
                op.to_string(),
            )),
            "o menu do `+` não consumiu a escolha"
        );
        assert!(
            t.route_composite_event(&PanelEvent::Click(crate::ids::PAINTER_BRUSH_COMPOSITE_ADD)),
            "o `+` não foi consumido"
        );
    };

    // Uma camada criada à mão NASCE A TRABALHAR — uma que não faz nada lê-se como a ferramenta
    // partida, e antes de hoje o zero era como uma posição se calava.
    cria(&mut t, 2); // Blur
    assert_eq!(t.composite_len(), 1);
    assert_eq!(t.paint.composite[0].op, CompositeOp::Blur);
    assert!(t.paint.composite[0].strength > 0.0, "ela nasce a trabalhar");

    // O `x` retira, as de baixo sobem, e a CAUDA volta a `strength = 0` — é essa linha que mantém
    // o motor (que pula uma camada pela força) alheio a esta wave.
    cria(&mut t, 0); // Brush, por baixo
    assert_eq!(t.composite_len(), 2);
    assert!(t.route_composite_event(&PanelEvent::Click(
        crate::ids::PAINTER_BRUSH_COMPOSITE_REMOVE[0]
    )));
    assert_eq!(t.composite_len(), 1);
    assert_eq!(
        t.paint.composite[0].op,
        CompositeOp::Brush,
        "a de baixo subiu para o lugar da que saiu"
    );
    assert_eq!(
        t.paint.composite[1].strength, 0.0,
        "a cauda TEM de voltar a zero: com os bytes da camada retirada lá, o motor continuaria a \
         pintá-la"
    );
}

/// ⭐⭐ **O reordenar pára na última camada VIVA, não na última POSIÇÃO.**
///
/// ⛔ Com o tecto no `MAX_CAMADAS` a camada de baixo trocaria com uma posição da CAUDA — que está
/// a `strength = 0` e não é pintada —, e ela **saía da lista** aos olhos do artista. *A cauda
/// existe no array e não existe na pilha.*
///
/// ⚠️ **A metade do DESCER é a que tem dentes:** a do subir é defensiva e a mutação que a troca
/// pelo `MAX_CAMADAS` **não é observável** (subir a posição `0` é um no-op nas duas versões, e o
/// painel nunca oferece a seta de uma posição que não existe) — declarado no doc do próprio
/// método em vez de coberto por um gate que afirmaria o nada.
#[test]
fn o_reordenar_para_na_ultima_camada_viva() {
    let mut t = tela();
    t.acrescenta_camada(0);
    t.acrescenta_camada(2);
    assert_eq!(t.composite_len(), 2);
    let antes: Vec<CompositeOp> = (0..composite::MAX_CAMADAS)
        .map(|i| t.paint.composite[i].op)
        .collect();
    // A última VIVA a descer é um no-op; a primeira a subir também.
    t.move_composite_layer_down(1);
    t.move_composite_layer_up(0);
    let depois: Vec<CompositeOp> = (0..composite::MAX_CAMADAS)
        .map(|i| t.paint.composite[i].op)
        .collect();
    assert_eq!(
        antes, depois,
        "o reordenar nas pontas moveu alguma coisa — a de baixo caiu para a cauda morta"
    );
    // CONTROLO: no MEIO da lista ele move mesmo.
    t.acrescenta_camada(1);
    t.move_composite_layer_down(1);
    assert_eq!(
        t.paint.composite[1].op,
        CompositeOp::Smear,
        "CONTROLO: o reordenar tem de funcionar no meio da lista"
    );
}

/// ⭐⭐⭐ **A QUOTA: `3 Brush · 2 Erase · 1 Blur · 1 Smear`, e o menu encolhe à medida que ela gasta.**
///
/// ⚠️ **As três metades são independentes e cada uma sozinha mente:** o menu deixar de oferecer ·
/// o `+` recusar mesmo que alguém peça · e o tecto da pilha ser a SOMA das quotas. Sem a segunda,
/// um menu bem pintado com uma rota permissiva por baixo passaria.
#[test]
fn a_quota_de_cada_operacao_e_respeitada_e_o_menu_encolhe() {
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = tela();
    // Metade A — gastar a quota do Blur (1) tira-o do menu.
    assert!(t.ops_com_quota_livre()[2], "o Blur começa disponível");
    t.acrescenta_camada(2);
    assert!(
        !t.ops_com_quota_livre()[2],
        "com o único Blur criado, o menu tem de deixar de o oferecer"
    );
    // Metade B — e a ROTA recusa, não só o menu.
    t.acrescenta_camada(2);
    assert_eq!(
        t.composite_len(),
        1,
        "o `+` criou um SEGUNDO Blur: o menu esconde-o e a rota deixou passar"
    );
    // Metade C — o tecto é a SOMA das quotas, e a pilha enche exactamente.
    for op in [0u8, 0, 0, 1, 3, 3] {
        t.acrescenta_camada(op);
    }
    assert_eq!(t.composite_len(), composite::MAX_CAMADAS);
    assert!(
        !t.pode_acrescentar_camada(),
        "com a quota toda gasta o `+` tem de estar desligado"
    );
    // ⚠️ E o menu SALTA para uma operação com quota depois de cada criação — sem isso o `+`
    // ficaria armado sobre uma esgotada.
    let mut u = tela();
    u.route_composite_event(&PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
        "2".to_string(),
    ));
    u.route_composite_event(&PanelEvent::Click(crate::ids::PAINTER_BRUSH_COMPOSITE_ADD));
    assert_ne!(
        u.composite_add_op(),
        2,
        "o menu ficou armado sobre o Blur, que já não tem quota"
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

/// **O núcleo de CAIXA é do Blur da PILHA, e só dele** (ordem do dono, 2026-09-20: *«só no Blur do
/// composite; o Blur como ferramenta isolada não deve ser modificado»*).
///
/// TRÊS metades, e cada uma sozinha mente:
/// 1. as duas rotas **discordam** — sem isto a pilha podia estar a usar o binomial em silêncio e o
///    ganho medido seria de outra coisa;
/// 2. elas discordam **POUCO** — é esta que torna «baixar a qualidade» uma afirmação e não uma
///    esperança (medido a `raio 96`: pior byte `1`, média `0,007`);
/// 3. o `Caixa` é pedido em **UM ficheiro só** de toda a crate da ferramenta — o do laço da pilha —,
///    a metade que impede a rota isolada de o herdar por descuido amanhã.
///
/// ⚠️ A 3.ª é textual **de propósito**: a rota isolada entra por uma porta SEM argumento
/// ([`super::blur_route::PainterTool::stamp_dabs_blur`]), logo o binomial dela é por CONSTRUÇÃO e
/// não há barro onde medir a ausência. *Uma ausência prova-se contando os sítios, não olhando.*
///
/// ⚠️⚠️ E a agulha é montada em runtime porque **este ficheiro seria ele próprio um acerto** — a
/// lição do gate auto-referente da escultura (*um censo textual que se lê a si mesmo encontra
/// sempre o que procura*). A 1.ª redacção esperava `composite.rs` **e este ficheiro**, e reprovou
/// alto: eu tinha escrito a expectativa como se ele fosse cúmplice e o código já o punha de fora.
#[test]
fn o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele() {
    let borra = |pilha: bool| -> Vec<u8> {
        let mut t = tela();
        t.paint.composite_enabled = false;
        traco(&mut t, 100.0); // uma marca para o blur ter o que borrar
        if pilha {
            t.paint.composite_enabled = true;
            arma(&mut t, &[(CompositeOp::Blur, 1.0)]);
        } else {
            t.paint.paint_mode = PaintMode::Blur;
        }
        traco(&mut t, 100.0);
        t.canvas_rgba.to_vec()
    };
    let (isolado, na_pilha) = (borra(false), borra(true));
    let pior = isolado
        .iter()
        .zip(na_pilha.iter())
        .map(|(a, b)| i32::from(*a).abs_diff(i32::from(*b)))
        .max()
        .unwrap_or(0);
    assert!(
        pior > 0,
        "as duas rotas dão a MESMA imagem — a pilha não trocou de núcleo"
    );
    assert!(
        pior <= 8,
        "a caixa tinha de borrar quase o mesmo e o pior byte é {pior}"
    );

    // 3.ª metade — o censo do alcance. ⚠️ A agulha é montada em runtime porque este ficheiro seria
    // ele próprio um acerto (a lição do gate auto-referente da escultura).
    let agulha = concat!("BlurKernel", "::", "Caixa");
    let mut sitios = Vec::new();
    for e in walk_rs(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")) {
        let txt = std::fs::read_to_string(&e).unwrap_or_default();
        if txt.contains(agulha) {
            sitios.push(e.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    sitios.sort();
    assert_eq!(
        sitios,
        vec![
            "composite_acumulado.rs".to_string(),
            "composite_pilha.rs".to_string(),
        ],
        "o núcleo de caixa só pode ser pedido pela pilha, e nos sítios nomeados aqui. \
         ⚠️⚠️ **A LEI não mudou duas vezes; o ENDEREÇO dela mudou duas vezes.** Em 2026-09-20 o \
         laço saiu do `composite.rs` para o `composite_pilha.rs` (a recomposição passou a ser por \
         CAMADA); em 2026-09-21 a pilha ganhou a rota de ACUMULAÇÃO e o pedido passou a existir \
         nas DUAS — no `composite_acumulado.rs` (a rota de omissão, uma passagem só) e no \
         `composite_pilha.rs` (o replay, a porta de bissecção `PH2D_COMPOSITE_REPLAY=1`). \
         ⛔ Um TERCEIRO sítio é o pedido a escapar da pilha, e é isso que este censo recusa."
    );
}

/// Varre os `.rs` de uma árvore — o instrumento do censo acima.
fn walk_rs(raiz: std::path::PathBuf) -> Vec<std::path::PathBuf> {
    let mut fora = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(d) = pilha.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                fora.push(p);
            }
        }
    }
    fora
}
