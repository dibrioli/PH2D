//! Os gates da **pilha que ACUMULA** ([`super::composite_acumulado`]), cada um sobre um defeito que
//! a wave de 2026-09-21 pagou.

use super::composite::{CompositeLayer, CompositeOp, N_CAMADAS};
use super::diag_auditoria_da_pilha::Caso;
use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const SIZE: u32 = 512;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// ⚠️ A tela leva ARTE por baixo e nasce com **alfa 0** — as duas metades da fixtura que a
/// auditoria de 2026-09-21 teve de descobrir: sobre branco opaco o Smear é invisível, e sobre uma
/// tela vazia ele é **inerte** (ele é a camada de baixo, corre sobre o `pre`, e não há o que
/// esfregar). *Uma fixtura no ponto neutro não testa a lei que ela diz testar.*
fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    let mut px = vec![255u8; (SIZE * SIZE * 4) as usize];
    for p in px.as_chunks_mut::<4>().0 {
        p[3] = 0;
    }
    t.set_source(px, SIZE, SIZE);
    t.paint.brush.radius_px = 20.0;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = false;
    t.paint.brush.color = [0.15, 0.35, 0.75];
    for k in 0..3 {
        let y = 180.0 + k as f32 * 60.0;
        t.on_canvas_pointer(cp([140.0, y], PointerPhase::Down));
        let mut x = 140.0;
        while x < 370.0 {
            x += 6.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([370.0, y], PointerPhase::Up));
    }
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.composite_enabled = true;
    for pos in 0..N_CAMADAS {
        t.paint.composite[pos] = CompositeLayer::default();
    }
    t
}

/// ⚠️ **Um RABISCO e não um traço recto** — num recto as caixas dos lotes afastam-se e a janela do
/// replay não cresce; foi essa a fixtura que deixou o custo quadrático passar despercebido.
fn rabisco(n: usize) -> Vec<[f32; 2]> {
    (0..=n)
        .map(|i| {
            let s = i as f32 * 2.0 / 45.0;
            [
                256.0 + 55.0 * (3.0 * s).sin(),
                256.0 + 55.0 * (2.0 * s).cos(),
            ]
        })
        .collect()
}

fn corre(t: &mut PainterTool, pts: &[[f32; 2]]) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
}

/// A diferença **que se VÊ**: os dois lados compostos sobre branco.
fn pior_visivel(a: &[u8], b: &[u8]) -> u8 {
    diferenca(a, b).0
}

/// `(pior, quantos, a caixa que os contém)` — o gate IMPRIME a caixa, porque é a forma dela que
/// nomeia o defeito (um rectângulo = um limite regional).
fn diferenca(a: &[u8], b: &[u8]) -> (u8, u64, String) {
    let sobre_branco = |px: &[u8], k: usize| {
        let al = f32::from(px[3]) / 255.0;
        f32::from(px[k]).mul_add(al, 255.0 * (1.0 - al))
    };
    let (mut pior, mut n) = (0u8, 0u64);
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for (i, (pa, pb)) in a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .enumerate()
    {
        let mut d = 0u8;
        for k in 0..3 {
            d = d.max((sobre_branco(pa, k) - sobre_branco(pb, k)).abs().round() as u8);
        }
        if d > 0 {
            pior = pior.max(d);
            n += 1;
            let (px, py) = (i as u32 % SIZE, i as u32 / SIZE);
            x0 = x0.min(px);
            y0 = y0.min(py);
            x1 = x1.max(px);
            y1 = y1.max(py);
        }
    }
    let caixa = if n == 0 {
        "—".to_string()
    } else {
        format!("{x0},{y0} {}×{}", x1 - x0 + 1, y1 - y0 + 1)
    };
    (pior, n, caixa)
}

fn com(t: &mut PainterTool, pos: usize, op: CompositeOp, s: f32, cor: Option<[f32; 3]>) {
    t.paint.composite[pos] = CompositeLayer {
        op,
        strength: s,
        color: cor,
        ..CompositeLayer::default()
    };
}

/// ⭐⭐⭐ **O PREÇO: o trabalho de um evento NÃO cresce com o traço.**
///
/// É a lei inteira desta wave, e a régua é uma **CONTAGEM** e não um relógio — *um gate de relógio
/// nesta casa é mais um membro da família de flakes sob fan-out*.
///
/// **Medido antes da cura** (rota de replay, a mesma fixtura): o pior evento passa de `24` para
/// `81` dabs quando o traço quadruplica, e o total de `1 386` para `18 542`. **Depois:** o pior
/// evento fica em `6` nas duas, e o total cresce em LINHA RECTA com o número de eventos.
///
/// ⚠️ O CONTROLO é a metade que torna o gate honesto: ele corre a rota de replay e **exige que ela
/// cresça**. Sem ele, um motor que simplesmente não depositasse nada passaria.
#[test]
fn o_trabalho_de_um_evento_nao_cresce_com_o_traco() {
    let mede = |n: usize, replay: bool| {
        super::composite_pilha::CONTA_DA_PILHA.with(|c| c.set((0, 0, 0)));
        let mut t = tela();
        t.paint.pilha_por_replay = replay;
        com(&mut t, 0, CompositeOp::Brush, 1.0, Some([1.0, 0.0, 0.0]));
        com(&mut t, 1, CompositeOp::Brush, 1.0, Some([0.0, 0.0, 1.0]));
        corre(&mut t, &rabisco(n));
        super::composite_pilha::CONTA_DA_PILHA.with(std::cell::Cell::get)
    };
    let (ev_c, _, pior_c) = mede(60, false);
    let (ev_l, _, pior_l) = mede(240, false);
    assert!(
        ev_l > ev_c * 3,
        "a fixtura não contém o fenómeno: o traço longo tem de ter muito mais eventos \
         ({ev_c} contra {ev_l})"
    );
    assert_eq!(
        pior_c, pior_l,
        "o pior evento da ACUMULAÇÃO tem de ser o mesmo nos dois traços — ele é o número de dabs \
         NOVOS, que é do passo do ponteiro e não da história (curto {pior_c}, longo {pior_l})"
    );
    // CONTROLO: a rota que esta wave substituiu CRESCE, e é isso que a torna a rota errada.
    let (_, _, pior_rc) = mede(60, true);
    let (_, _, pior_rl) = mede(240, true);
    assert!(
        pior_rl > pior_rc * 2,
        "o CONTROLO falhou: a rota de replay tinha de crescer com o traço (curto {pior_rc}, \
         longo {pior_rl}) — se ela não cresce, esta fixtura não mede o defeito que a wave curou"
    );
}

/// ⭐⭐⭐ **A acumulação é EXACTA no Brush, no Erase e no Smear.**
///
/// As três leis compõem-se por álgebra (`over` é associativo; a borracha é a mesma `lerp` com o
/// `c` do traço inteiro; o esfregão já era um campo por traço). Só o Blur diverge, e essa
/// divergência tem gate próprio.
#[test]
fn a_acumulacao_e_exacta_menos_no_borrao() {
    let casos: &[Caso] = &[
        // ⚠️ **O topo é PARCIAL de propósito.** Com ele a `1,0` a tinta satura e esconde tudo o
        // que está por baixo — a fixtura ficava verde sobre uma composição que compunha a pilha
        // DUAS vezes (medido: a mutação que apaga o `escreve_do_pre` sobrevivia a ela).
        ("dois Brushes, o de cima PARCIAL", |t| {
            com(t, 0, CompositeOp::Brush, 0.4, Some([1.0, 0.0, 0.0]));
            com(t, 1, CompositeOp::Brush, 1.0, Some([0.0, 0.0, 1.0]));
        }),
        ("Erase sobre Brush", |t| {
            com(t, 0, CompositeOp::Erase, 0.7, None);
            com(t, 1, CompositeOp::Brush, 1.0, Some([1.0, 0.0, 0.0]));
        }),
        ("Smear sob Brush", |t| {
            com(t, 0, CompositeOp::Brush, 1.0, Some([1.0, 0.0, 0.0]));
            com(t, 1, CompositeOp::Smear, 1.0, None);
        }),
    ];
    let pts = rabisco(90);
    for &(nome, monta) in casos {
        let img = |replay: bool| {
            let mut t = tela();
            t.paint.pilha_por_replay = replay;
            monta(&mut t);
            corre(&mut t, &pts);
            (*t.canvas_rgba).clone()
        };
        let a = img(true);
        let b = img(false);
        assert_eq!(
            pior_visivel(&a, &b),
            0,
            "{nome}: a acumulação tinha de dar a MESMA imagem do replay"
        );
        // CONTROLO: as duas corridas pintaram alguma coisa — senão comparam-se duas telas vazias.
        let nada = {
            let t = tela();
            (*t.canvas_rgba).clone()
        };
        assert!(
            pior_visivel(&a, &nada) > 8,
            "{nome}: a fixtura não pintou nada, e duas telas iguais não afirmam lei nenhuma"
        );
    }
}

/// ⭐⭐⭐ **O limite REGIONAL quase não custa imagem** — o carimbo rectangular do report encolheu
/// de uma ordem de grandeza.
///
/// A régua é a composição da região contra a do canvas INTEIRO: se elas discordam, a fronteira da
/// discordância é um **rectângulo**, e é isso que o dono fotografou em 2026-09-21.
///
/// **Medido nesta fixtura:** replay **`103`** de 255 · acumulação **`12`**, e a barra de `16` sai desse vale. ⏳ E o resíduo que fica
/// está ATRIBUÍDO: ele só existe com **Blur e Smear juntos** (Brush só, Blur+Brush e Brush+Smear
/// leem `0`), e é a base congelada do esfregão, que é refrescada só dentro da região — o esfregão
/// lê `p − disp(p)`, que pode cair fora dela. A cura tem endereço e não é desta wave: com os
/// planos, *«a tela como as camadas de baixo a deixaram»* é calculável em qualquer região.
///
/// ⚠️ **O CONTROLO é a metade que torna a barra honesta:** ele exige que a rota de REPLAY continue
/// a falhar por muito mais. Sem ele, um motor que não pintasse nada passaria.
#[test]
fn a_regiao_quase_nao_deixa_rectangulo() {
    const BARRA: u8 = 16;
    let pts = rabisco(90);
    let img = |global: bool, replay: bool| {
        super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
        let mut t = tela();
        t.paint.pilha_por_replay = replay;
        com(&mut t, 0, CompositeOp::Blur, 0.4, None);
        com(&mut t, 1, CompositeOp::Brush, 0.3, Some([1.0, 0.0, 0.0]));
        com(&mut t, 3, CompositeOp::Smear, 0.6, None);
        corre(&mut t, &pts);
        super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
        (*t.canvas_rgba).clone()
    };
    let (pior, n, caixa) = diferenca(&img(true, false), &img(false, false));
    assert!(
        pior <= BARRA,
        "compor só a região discorda de compor o canvas inteiro por {pior} de 255 \
         ({n} px, caixa {caixa}) — a fronteira dessa discordância é um RECTÂNGULO, que é o \
         artefacto do report de 2026-09-21"
    );
    let (pior_replay, _, _) = diferenca(&img(true, true), &img(false, true));
    assert!(
        pior_replay > pior * 2,
        "o CONTROLO falhou: a rota de replay tinha de discordar por MUITO mais \
         (replay {pior_replay}, acumulação {pior}) — se as duas são iguais, esta fixtura não \
         contém o defeito que a wave curou"
    );
}

/// ⛔⛔ **O avental do borrão cobre o ALCANCE das `P` passagens** — e sem isso o resultado passa a
/// depender de em quantos lotes o traço chegou.
///
/// ⚠️ Este gate nasceu de um VERMELHO: com o avental de um raio só, o
/// `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` reprovou. A régua aqui é a LEI (o avental contém o
/// alcance das `P` passagens), porque a do irmão é a imagem e ela só acusa quando as duas se
/// cruzam.
///
/// ⛔⛔ **A PREMISSA DE ANTES MORREU em 2026-09-23, e à vista:** ele afirmava `pad >= k·P` — o
/// alcance do núcleo BINOMIAL — e o composite borra com o de CAIXA, cujo alcance para o mesmo
/// parâmetro é `Σ box_radii(k·P)` (`32` contra `256` na pilha do dono). O avental passou a ser esse
/// alcance e este gate passou a afirmá-lo; a igualdade da IMAGEM com o avental de antes é o gate
/// `composite_cerca_tests::o_avental_estreito_da_a_mesma_imagem_na_pilha_do_dono`.
///
/// ⚠️ A 2.ª metade é o CONTROLO de que as passagens contam: o alcance de `k·P` tem de ser maior que
/// o de `k`, senão a fixtura não distingue um avental de uma passagem de um de `P`.
#[test]
fn o_avental_do_borrao_cobre_o_alcance_de_todas_as_passagens() {
    let mut t = tela();
    com(&mut t, 0, CompositeOp::Blur, 1.0, None);
    com(&mut t, 1, CompositeOp::Brush, 1.0, Some([1.0, 0.0, 0.0]));
    let k = ph2d_painter_brush::kernel_radius(t.paint.brush.radius_px);
    let p = super::composite_acumulado::passagens_do_borrao(t.paint.brush.spacing) as usize;
    let pad = t.pad_do_borrao_para_teste() as usize;
    let caixa = ph2d_painter_brush::BlurKernel::Caixa;
    assert!(
        p > 1 && caixa.alcance(k * p) > caixa.alcance(k),
        "a fixtura não contém o fenómeno: com UMA passagem o alcance de `k` bastaria"
    );
    assert!(
        pad > caixa.alcance(k * p),
        "o avental ({pad}) não cobre o alcance ({}) das {p} passagens de raio {k} — a orla lê \
         bytes que a composição ainda não escreveu, e o traço passa a depender da taxa do rato",
        caixa.alcance(k * p)
    );
    assert!(
        pad < k * p,
        "o avental ({pad}) voltou a ser o alcance do BINOMIAL (`k·P = {}`) — a composição de toda \
         camada corre sobre essa área",
        k * p
    );
}

/// ⛔⛔ **O trinco de alfa NÃO se aplica enquanto uma camada acumula no plano dela.**
///
/// O plano da tinta nasce TRANSPARENTE: travar o alfa ali suprimiria o depósito inteiro e a
/// camada acumularia zero. O trinco é da CAMADA DO DOCUMENTO e corre uma vez, na composição.
#[test]
fn o_trinco_de_alfa_nao_apaga_a_acumulacao() {
    let pinta = |trancado: bool| {
        let mut t = tela();
        if let Some(l) = t.layers.active().and_then(|id| t.layers.get_mut(id)) {
            l.alpha_locked = trancado;
        }
        com(&mut t, 0, CompositeOp::Brush, 1.0, Some([1.0, 0.0, 0.0]));
        com(&mut t, 1, CompositeOp::Brush, 1.0, Some([0.0, 0.0, 1.0]));
        corre(&mut t, &rabisco(60));
        (*t.canvas_rgba).clone()
    };
    let solto = pinta(false);
    let trancado = pinta(true);
    let vazio = {
        let t = tela();
        (*t.canvas_rgba).clone()
    };
    // Com o trinco, a tinta só entra onde já havia alfa — mas ela ENTRA.
    assert!(
        pior_visivel(&trancado, &vazio) > 8,
        "com o trinco de alfa a pilha não pintou NADA: o plano transparente comeu o depósito"
    );
    // CONTROLO: e o trinco continua a fazer alguma coisa (senão o gate acima passaria com ele morto).
    assert!(
        pior_visivel(&trancado, &solto) > 8,
        "o trinco de alfa ficou INERTE — ele tem de continuar a restringir a tinta ao alfa que já lá estava"
    );
}
