//! Os gates da **ordem por TRAÇO** ([`super::composite_pilha`]) e da **borracha de escopo**.
//!
//! ⚠️ Todos medem o BARRO pela porta do produto (`on_canvas_pointer`), com o traço entregue em
//! **vários** lotes — que é exactamente o regime em que o defeito do report vivia. *Um gate que
//! entrega o traço num lote só passa com o defeito de pé: dentro de um lote a ordem sempre esteve
//! certa.*

use super::composite::EscopoDaBorracha;
use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

const SIZE: u32 = 256;
const RAIO: f32 = 12.0;
const Y: f32 = 128.0;
const X0: f32 = 60.0;
const X1: f32 = 196.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O traço recto, entregue em passos de `passo` px — **é o passo que decide quantos LOTES há**, e é
/// por isso que ele é um parâmetro e não uma constante.
fn traco(t: &mut PainterTool, passo: f32) {
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    let mut x = X0;
    while x < X1 {
        x += passo;
        t.on_canvas_pointer(cp([x.min(X1), Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
}

/// Disco duro sobre papel branco, composite ligado e as cinco posições CALADAS.
fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = RAIO;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..composite::N_CAMADAS {
        t.paint.composite[pos].strength = 0.0;
    }
    t
}

/// A média de R e de B ao longo da linha central do traço — a régua que diz *de que COR ele ficou*.
fn media_rb(t: &PainterTool) -> (f32, f32) {
    let y = Y as usize;
    let (mut sr, mut sb, mut n) = (0f32, 0f32, 0f32);
    for x in (X0 as usize)..=(X1 as usize) {
        let i = (y * SIZE as usize + x) * 4;
        sr += f32::from(t.canvas_rgba[i]);
        sb += f32::from(t.canvas_rgba[i + 2]);
        n += 1.0;
    }
    (sr / n, sb / n)
}

/// ⭐⭐⭐ **O DEFEITO DO REPORT, medido: dois Brushes de cores opostas, o de cima tem de VENCER.**
///
/// Report do dono, 2026-09-20: *«o brush de cima sempre deve ser desenhado por cima do Brush de
/// baixo. Atualmente o brush de baixo cobre o brush de cima do carimbo anterior.»*
///
/// **Medido antes da cura** (passo `2`, o que um rato real entrega): `R 167,3 · B 87,7` — dois
/// terços de vermelho e um terço de azul, com o fundo a aparecer em toda a parte. **Depois:**
/// `R 255,0 · B 0,0`.
///
/// ⚠️ **O CONTROLO é a metade que torna o gate honesto:** com só a camada de BAIXO armada a mesma
/// régua tem de ler azul puro. Sem ele, um motor que simplesmente não pintasse a de baixo passaria.
#[test]
fn o_brush_de_cima_vence_o_de_baixo_ao_longo_do_traco() {
    let arma = |t: &mut PainterTool, com_fundo: bool| {
        t.paint.composite[0] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: 1.0,
            color: Some([1.0, 0.0, 0.0]),
            ..CompositeLayer::default()
        };
        t.paint.composite[1] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: if com_fundo { 1.0 } else { 0.0 },
            color: Some([0.0, 0.0, 1.0]),
            ..CompositeLayer::default()
        };
    };
    for passo in [2.0f32, 8.0] {
        let mut t = tela();
        arma(&mut t, true);
        traco(&mut t, passo);
        let (r, b) = media_rb(&t);
        assert!(
            r > 254.0 && b < 1.0,
            "passo {passo}: o topo VERMELHO tem de vencer o fundo azul ao longo do traço \
             (lido R {r:.1} · B {b:.1}; antes da cura: R 167,3 · B 87,7)"
        );
    }
    // CONTROLO: sem a camada de baixo a régua lê o mesmo — logo ela não distingue nada sozinha; é
    // o par (com fundo, sem fundo) que prova que a de baixo ESTÁ a ser pintada e coberta.
    let mut so_topo = tela();
    arma(&mut so_topo, false);
    traco(&mut so_topo, 2.0);
    let (r, b) = media_rb(&so_topo);
    assert!(r > 254.0 && b < 1.0, "controlo: só o topo ⇒ vermelho puro");
    // …e o CONTROLO do controlo: só o fundo dá azul puro, senão a fixtura não pinta nada.
    let mut so_fundo = tela();
    arma(&mut so_fundo, true);
    so_fundo.paint.composite[0].strength = 0.0;
    traco(&mut so_fundo, 2.0);
    let (r, b) = media_rb(&so_fundo);
    assert!(
        r < 1.0 && b > 254.0,
        "controlo: só o fundo ⇒ azul puro (lido R {r:.1} · B {b:.1})"
    );
}

/// ⭐⭐⭐ **A RÉGUA UNIVERSAL: o mesmo traço numa tacada ou em N lotes tem de dar a MESMA imagem.**
///
/// Ela é universal porque não sabe o que cada camada faz — só que *a ordem é da PILHA e não da taxa
/// do rato*. É a que apanha os três reports de uma vez, e a que apanha o quarto que ainda não
/// aconteceu.
///
/// **Medido** (`|Δ| médio` sobre a linha central, contra a entrega num lote só):
///
/// | topo sobre Brush azul | passo 2 (antes) | passo 2 (depois) |
/// |---|---|---|
/// | Brush vermelho | `41,59` | **`0,00`** |
/// | Erase | `0,00` | `0,00` |
/// | Smear | `0,00` | `0,00` |
/// | Blur | `0,00` | `0,00` |
///
/// ⚠️ **As três linhas que já liam `0,00` não eram um motor certo — era a FIXTURA a não conter o
/// fenómeno** (apagar a força toda satura; borrar o miolo chapado de um traço é um no-op). Elas
/// ficam porque a régua tem de reprovar no dia em que alguém as quebrar.
#[test]
fn a_ordem_e_da_pilha_e_nao_da_taxa_do_rato() {
    for topo in [
        CompositeOp::Brush,
        CompositeOp::Erase,
        CompositeOp::Smear,
        CompositeOp::Blur,
    ] {
        let monta = |passo: f32| {
            let mut t = tela();
            t.paint.composite[0] = CompositeLayer {
                op: topo,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([0.0, 0.0, 1.0]),
                ..CompositeLayer::default()
            };
            traco(&mut t, passo);
            (*t.canvas_rgba).clone()
        };
        let um_lote = monta(X1 - X0);
        for passo in [2.0f32, 8.0] {
            let n_lotes = monta(passo);
            let (soma, pior) =
                um_lote
                    .iter()
                    .zip(n_lotes.iter())
                    .fold((0u64, 0u8), |(s, p), (&a, &b)| {
                        let d = a.abs_diff(b);
                        (s + u64::from(d), p.max(d))
                    });
            let medio = soma as f64 / um_lote.len() as f64;
            assert!(
                pior == 0,
                "{topo:?} em passos de {passo}: a entrega em lotes MUDOU a imagem \
                 (|Δ| médio {medio:.2}, pior {pior})"
            );
        }
    }
}

/// ⛔ **Com MENOS DE DUAS camadas activas a recomposição nem abre** — e é isso que mantém o traço
/// de quem não empilha nada byte-idêntico ao de sempre.
///
/// ⚠️ A régua é o ESTADO (`pilha.pre` vazio) e não a imagem: *comparar a imagem consigo mesma é a
/// tautologia que o gate da opção do puxão da escultura já pagou*. Com o `pre` vazio nenhuma das
/// seis etapas correu, e o caminho é o `for pos in (0..N).rev()` de sempre.
#[test]
fn uma_camada_so_nao_abre_a_recomposicao() {
    let mut t = tela();
    t.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        ..CompositeLayer::default()
    };
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    t.on_canvas_pointer(cp([X0 + 30.0, Y], PointerPhase::Move));
    assert!(
        t.paint.pilha.pre.is_empty(),
        "uma camada só: a pilha não tem ordem para arrumar e não paga a fotografia"
    );
    // CONTROLO: com a SEGUNDA camada armada, a mesma entrega abre a pilha.
    let mut u = tela();
    u.paint.composite[0] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        ..CompositeLayer::default()
    };
    u.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        ..CompositeLayer::default()
    };
    u.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    u.on_canvas_pointer(cp([X0 + 30.0, Y], PointerPhase::Move));
    assert_eq!(
        u.paint.pilha.pre.len(),
        (SIZE * SIZE * 4) as usize,
        "controlo: duas camadas ⇒ a pilha abre"
    );
}

/// ⭐⭐⭐ **A BORRACHA DE ESCOPO `Traco` devolve o pixel ao que ele era ANTES do gesto.**
///
/// Ordem do dono, 2026-09-20: *«uma opção em erase: se a borracha atua só no próprio traço do Brush
/// ou se ela apaga também a camada da imagem abaixo»*.
///
/// A fixtura tem **imagem por baixo** (papel VERDE opaco), uma camada Brush vermelha e a borracha
/// por cima. As três colunas que o gate afirma:
///
/// * `Tudo` (o valor de fábrica) — o alfa vai a `0`: *a imagem foi-se com a tinta*.
/// * `Traco` — o pixel volta ao VERDE opaco: *só a tinta deste traço saiu*.
/// * CONTROLO — sem a borracha, o pixel é VERMELHO.
///
/// ⚠️ **O CONTROLO é o que separa «a borracha do traço funciona» de «a borracha não faz nada»**:
/// sem ele, uma camada inerte passaria a coluna do meio.
#[test]
fn a_borracha_do_traco_devolve_o_pre_e_a_de_tudo_come_a_imagem() {
    let monta = |escopo: Option<EscopoDaBorracha>| {
        let mut t = PainterTool::default();
        // Papel VERDE opaco — a «imagem por baixo» de que o report fala.
        let mut fundo = vec![0u8; (SIZE * SIZE * 4) as usize];
        for px in fundo.as_chunks_mut::<4>().0 {
            px.copy_from_slice(&[0, 200, 0, 255]);
        }
        t.set_source(fundo, SIZE, SIZE);
        t.paint.brush.radius_px = RAIO;
        t.paint.brush.hardness = 1.0;
        t.paint.brush.falloff = Falloff::Constant;
        t.paint.brush.strength = 1.0;
        t.paint.brush.space_attenuation = false;
        t.paint.composite_enabled = true;
        for pos in 0..composite::N_CAMADAS {
            t.paint.composite[pos].strength = 0.0;
        }
        t.paint.composite[1] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: 1.0,
            color: Some([1.0, 0.0, 0.0]),
            ..CompositeLayer::default()
        };
        if let Some(e) = escopo {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Erase,
                strength: 1.0,
                erase_scope: e,
                ..CompositeLayer::default()
            };
        }
        traco(&mut t, 2.0);
        let i = ((Y as usize) * SIZE as usize + ((X0 + X1) as usize / 2)) * 4;
        [
            t.canvas_rgba[i],
            t.canvas_rgba[i + 1],
            t.canvas_rgba[i + 2],
            t.canvas_rgba[i + 3],
        ]
    };
    let sem = monta(None);
    assert!(
        sem[0] > 250 && sem[1] < 6 && sem[3] == 255,
        "controlo: sem borracha o miolo é a tinta VERMELHA (lido {sem:?})"
    );
    let tudo = monta(Some(EscopoDaBorracha::Tudo));
    assert!(
        tudo[3] == 0,
        "escopo Tudo: o alfa tem de ir a ZERO — ela come a imagem por baixo (lido {tudo:?})"
    );
    let so_o_traco = monta(Some(EscopoDaBorracha::Traco));
    assert!(
        so_o_traco[3] == 255 && so_o_traco[1] > 190 && so_o_traco[0] < 12,
        "escopo Traco: o pixel volta ao VERDE opaco de antes do gesto (lido {so_o_traco:?})"
    );
}

/// ⛔⛔ **O cap de Accumulate de cada camada tem de ser LIMPO na região recomposta.**
///
/// Ele é um mapa de cobertura **por TRAÇO** que o `!accumulate && strength < 1` arma — o tecto que
/// impede um traço lento de escurecer sozinho. A recomposição reconstrói a região a partir do
/// `pre`, logo **o cap ali tem de recomeçar do zero**: deixá-lo cheio faz o replay encontrar o
/// tecto já atingido e depositar **ZERO**, e o traço desaparece à medida que a mão anda.
///
/// ⚠️ **Este gate precisa de `strength < 1`, e é isso que o torna necessário:** os irmãos correm a
/// força cheia, onde o cap nem é armado (`stroke_cover_wanted`) — *um corpus no ponto neutro de um
/// mecanismo não testa esse mecanismo*, e a mutação que apaga a limpeza SOBREVIVE a todos eles.
#[test]
fn o_cap_de_cada_camada_recomeca_na_regiao_recomposta() {
    let monta = |duas: bool| {
        let mut t = tela();
        t.paint.brush.strength = 0.6;
        t.paint.composite[0] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: 0.6,
            color: Some([1.0, 0.0, 0.0]),
            ..CompositeLayer::default()
        };
        t.paint.composite[1] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: if duas { 0.6 } else { 0.0 },
            color: Some([1.0, 0.0, 0.0]),
            ..CompositeLayer::default()
        };
        traco(&mut t, 2.0);
        // O verde do miolo: quanto MENOR, mais tinta vermelha lá está.
        let y = Y as usize;
        let mut pior = 0u8;
        for x in (X0 as usize + 20)..=(X1 as usize - 20) {
            let i = (y * SIZE as usize + x) * 4;
            pior = pior.max(t.canvas_rgba[i + 1]);
        }
        pior
    };
    let uma = monta(false);
    let duas = monta(true);
    assert!(
        uma < 200,
        "fixtura: uma camada a 0,6 já tem de depositar tinta a sério (verde do miolo {uma})"
    );
    assert!(
        duas < uma,
        "duas camadas a 0,6 têm de depositar MAIS que uma — com o cap por limpar o replay \
         encontra o tecto cheio e deposita zero (verde: uma {uma}, duas {duas})"
    );
}

/// ⛔ **O `pre` da pilha é do TRAÇO: um gesto novo não recompõe a partir da tela do anterior.**
///
/// Sem o `fecha()` no pen-down, o segundo traço restaura a região dele a partir da tela de ANTES do
/// primeiro — e o primeiro traço é apagado onde os dois se cruzam. ⚠️ Nenhum dos irmãos o via: eles
/// fazem **um** traço sobre uma ferramenta acabada de construir.
#[test]
fn um_traco_novo_nao_recompoe_da_tela_do_anterior() {
    let mut t = tela();
    t.paint.composite[0] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        color: Some([1.0, 0.0, 0.0]),
        ..CompositeLayer::default()
    };
    t.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        color: Some([0.0, 0.0, 1.0]),
        ..CompositeLayer::default()
    };
    traco(&mut t, 2.0); // 1.º traço, horizontal
    // ⚠️ **A outra metade: no pen-up a pilha larga a tela.** O `pre` é `w·h·4` bytes, e segurá-lo
    // entre gestos é memória parada — mais a segunda resposta a *«de que tela este traço parte?»*.
    assert!(
        t.paint.pilha.pre.is_empty() && t.paint.pilha.lotes.is_empty(),
        "no pen-up a pilha tem de largar a tela e a história dos lotes"
    );
    // 2.º traço: uma barra VERTICAL que cruza o primeiro ao meio.
    let x = (X0 + X1) * 0.5;
    t.on_canvas_pointer(cp([x, Y - 40.0], PointerPhase::Down));
    let mut yy = Y - 40.0;
    while yy < Y + 40.0 {
        yy += 2.0;
        t.on_canvas_pointer(cp([x, yy.min(Y + 40.0)], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x, Y + 40.0], PointerPhase::Up));
    // O primeiro traço tem de continuar lá, longe do cruzamento.
    let i = ((Y as usize) * SIZE as usize + (X0 as usize + 25)) * 4;
    assert!(
        t.canvas_rgba[i] > 254 && t.canvas_rgba[i + 2] < 1,
        "o 2.º traço apagou o 1.º: a pilha recompôs de um `pre` que já não é o dela \
         (lido {:?})",
        &t.canvas_rgba[i..i + 4]
    );
}

/// ⭐⭐ **Um fluxo de RNG por CAMADA: com Randomize Color ligado a imagem NÃO pode depender da
/// taxa do rato.**
///
/// ⚠️ A recomposição corre **por camada** e não por lote, logo um fluxo partilhado seria consumido
/// noutra ordem a cada lote — *o Grain Random a cintilar por baixo da mão*. Cada lote guarda a
/// posição do fluxo **de cada camada** e o replay repõe-na.
///
/// ⛔ **Os irmãos não o podiam ver:** eles correm com o Randomize DESLIGADO, onde nenhum dab toca o
/// gerador — *um corpus no ponto neutro de um knob não testa esse knob*.
#[test]
fn com_randomize_a_imagem_nao_depende_da_taxa_do_rato() {
    let monta = |passo: f32| {
        let mut t = tela();
        t.paint.brush.color = [0.8, 0.2, 0.2];
        // ⚠️ **O jitter de COR sozinho NÃO chega, e isso é o achado deste gate:** ele é assado na
        // LISTA DE DABS (`Dab::color`), que o replay reutiliza tal e qual ⇒ ele é seguro por
        // construção e a mutação sobrevive. Quem consome o fluxo por-dab é o **GRÃO com mapeamento
        // aleatório** (`texture::dab_basis`), e é ele que torna o fluxo por camada observável.
        t.paint.brush.color_jitter_enabled = true;
        t.paint.brush.color_jitter_hue = 0.5;
        t.paint.brush.color_jitter_val = 0.5;
        t.paint.brush.texture.kind = ph2d_painter_brush::TextureKind::Noise;
        t.paint.brush.texture.mapping = ph2d_painter_brush::TextureMapping::Random;
        t.paint.brush.texture.size = [0.25, 0.25];
        t.paint.brush.grain_depth = 1.0;
        t.paint.composite[0] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: 1.0,
            ..CompositeLayer::default()
        };
        t.paint.composite[1] = CompositeLayer {
            op: CompositeOp::Brush,
            strength: 1.0,
            ..CompositeLayer::default()
        };
        traco(&mut t, passo);
        (*t.canvas_rgba).clone()
    };
    let um_lote = monta(X1 - X0);
    let n_lotes = monta(2.0);
    let pior = um_lote
        .iter()
        .zip(n_lotes.iter())
        .map(|(&a, &b)| a.abs_diff(b))
        .max()
        .unwrap_or(0);
    assert_eq!(
        pior, 0,
        "com Randomize Color as duas entregas deram imagens diferentes — o fluxo de RNG \
         de uma camada não está a ser reposto no replay"
    );
    // CONTROLO: a fixtura CONTÉM o fenómeno — ao longo da linha central a tinta de facto VARIA
    // de dab para dab. ⚠️ A régua é a variação ao longo de `x` e não `R != G` dentro de um pixel:
    // com um pincel cinzento o jitter de VALOR dá `r = g = b` e o controlo lia-se como ausente.
    let y = Y as usize;
    let cores: std::collections::BTreeSet<[u8; 3]> = ((X0 as usize + 20)..=(X1 as usize - 20))
        .map(|x| {
            let i = (y * SIZE as usize + x) * 4;
            [um_lote[i], um_lote[i + 1], um_lote[i + 2]]
        })
        .collect();
    let variou = cores.len() > 3;
    assert!(
        variou,
        "controlo: com Randomize ligado os dabs têm de sair de cores DIFERENTES ao longo do \
         traço (lidas {} distintas), senão este gate mede o mesmo que os irmãos",
        cores.len()
    );
}
