//! Os gates de VALOR da **cerca do borrão** (auditoria de 2026-09-23).
//!
//! A cura de 2026-09-22 aplica o borrão só na `caixa_nova` quando ninguém acima dele lê
//! vizinhança. O único gate que a media era uma CONTA (`o_borrao_no_topo_e_aplicado_so_na_caixa_nova`,
//! o tamanho da região), e a auditoria achou que **nenhum gate de valor compunha uma camada que lê
//! vizinhança POR CIMA do borrão** — a frase *«sem a cerca a imagem muda»* nunca tinha sido medida.

use super::composite::{CompositeLayer, CompositeOp, N_CAMADAS};
use super::composite_acumulado::{CERCA_DESLIGADA, CERCA_DO_BORRAO, CURA_REVERTIDA};
use super::*;
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

fn traco(t: &mut PainterTool, passo: f32) {
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    let mut x = X0;
    while x < X1 {
        x += passo;
        t.on_canvas_pointer(cp([x.min(X1), Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
}

/// ⚠️ **A tela TEM TEXTURA** — numa tela chapada a soma corrente da caixa dá o mesmo comece onde
/// começar, e um borrão sobre um campo uniforme é um no-op: *a fixtura chapada não contém o
/// fenómeno* (foi assim que a barra `pior == 0` do `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` se
/// leu verdadeira sobre um produto que não a cumpre).
/// A semente que devolve a tela CHAPADA (`255` em toda parte) — a do `a_ordem_…`.
const CHAPADA: u64 = u64::MAX;

fn tela(semente: u64) -> PainterTool {
    let mut t = PainterTool::default();
    let mut s = semente
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    let mut px = vec![255u8; (SIZE * SIZE * 4) as usize];
    for (i, b) in px.iter_mut().enumerate() {
        if i % 4 == 3 || semente == CHAPADA {
            continue;
        }
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *b = (s >> 56) as u8;
    }
    t.set_source(px, SIZE, SIZE);
    t.paint.brush.radius_px = RAIO;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..N_CAMADAS {
        t.paint.composite[pos].strength = 0.0;
    }
    t
}

fn camada(op: CompositeOp, size: f32) -> CompositeLayer {
    CompositeLayer {
        op,
        strength: 1.0,
        size,
        color: Some([0.0, 0.0, 1.0]),
        ..CompositeLayer::default()
    }
}

/// A tela depois do traço entregue em lotes de `passo` px, com a cerca no modo `modo`.
fn monta(pilha: &[(CompositeOp, f32)], semente: u64, passo: f32, modo: u8) -> Vec<u8> {
    CERCA_DO_BORRAO.with(|c| c.set(modo));
    let mut t = tela(semente);
    for (pos, &(op, size)) in pilha.iter().enumerate() {
        t.paint.composite[pos] = camada(op, size);
    }
    traco(&mut t, passo);
    CERCA_DO_BORRAO.with(|c| c.set(0));
    (*t.canvas_rgba).clone()
}

/// O mesmo traço num lote só e em lotes de `passo` px, com a cerca no modo `modo`.
/// Devolve `(pior, médio, bytes diferentes)` sobre a FAIXA do traço — a média sobre a tela inteira
/// dilui um viés sistemático de um byte (a auditoria mediu: `médio < 0,01` sobre `256²×4` deixa
/// passar `~2 600` bytes errados, `~20 %` da faixa).
fn diferenca(pilha: &[(CompositeOp, f32)], semente: u64, passo: f32, modo: u8) -> (u8, f64, usize) {
    let um = monta(pilha, semente, X1 - X0, modo);
    let n = monta(pilha, semente, passo, modo);
    let (y0, y1) = ((Y - 3.0 * RAIO) as usize, (Y + 3.0 * RAIO) as usize);
    let (x0, x1) = ((X0 - 3.0 * RAIO) as usize, (X1 + 3.0 * RAIO) as usize);
    let (mut soma, mut pior, mut conta, mut total) = (0u64, 0u8, 0usize, 0usize);
    for y in y0..y1 {
        for x in x0..x1 {
            for c in 0..4 {
                let i = (y * SIZE as usize + x) * 4 + c;
                let d = um[i].abs_diff(n[i]);
                soma += u64::from(d);
                pior = pior.max(d);
                conta += usize::from(d > 0);
                total += 1;
            }
        }
    }
    (pior, soma as f64 / total as f64, conta)
}

/// A sonda que escolhe as fixturas e as barras dos gates abaixo.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --lib diag_a_cerca_do_borrao -- --ignored --nocapture`
#[test]
#[ignore = "sonda"]
fn diag_a_cerca_do_borrao() {
    let pilhas: [(&str, Vec<(CompositeOp, f32)>); 8] = [
        (
            "Smear/Brush",
            vec![(CompositeOp::Smear, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "Blur/Brush",
            vec![(CompositeOp::Blur, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "Brush/Brush",
            vec![(CompositeOp::Brush, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "Erase/Brush",
            vec![(CompositeOp::Erase, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "Smear/Blur/Brush",
            vec![
                (CompositeOp::Smear, 1.0),
                (CompositeOp::Blur, 1.0),
                (CompositeOp::Brush, 1.0),
            ],
        ),
        (
            "Smear/Blur(2)/Brush",
            vec![
                (CompositeOp::Smear, 1.0),
                (CompositeOp::Blur, 2.0),
                (CompositeOp::Brush, 1.0),
            ],
        ),
        (
            "Blur/Blur/Brush",
            vec![
                (CompositeOp::Blur, 1.0),
                (CompositeOp::Blur, 1.0),
                (CompositeOp::Brush, 1.0),
            ],
        ),
        (
            "Blur(2)/Blur(1)/Brush",
            vec![
                (CompositeOp::Blur, 2.0),
                (CompositeOp::Blur, 1.0),
                (CompositeOp::Brush, 1.0),
            ],
        ),
    ];
    // A pergunta do `a_ordem_…`: o byte de diferença do Blur sobre um Brush — de quem é ele?
    for (modo, nome) in [(0u8, "cura"), (CURA_REVERTIDA, "antes")] {
        let (mut com, mut pior_de_todos) = (Vec::new(), 0u8);
        for semente in (1..=40u64).chain([CHAPADA]) {
            for passo in [2.0f32, 8.0] {
                let (p, _, c) = diferenca(
                    &[(CompositeOp::Blur, 1.0), (CompositeOp::Brush, 1.0)],
                    semente,
                    passo,
                    modo,
                );
                pior_de_todos = pior_de_todos.max(p);
                if p > 0 {
                    com.push((semente, passo, p, c));
                }
            }
        }
        eprintln!("Blur/Brush {nome}: pior {pior_de_todos}, casos com diferença {com:?}");
    }
    for (nome, pilha) in &pilhas {
        for modo in [0u8, CERCA_DESLIGADA, CURA_REVERTIDA] {
            for semente in [7u64, 101, 999] {
                for passo in [2.0f32, 8.0] {
                    let (p, m, c) = diferenca(pilha, semente, passo, modo);
                    eprintln!(
                        "{nome:<22} modo {modo} semente {semente:>3} passo {passo} | pior {p:>3} médio {m:.4} bytes {c}"
                    );
                }
            }
        }
    }
}

const SMEAR_BLUR_BRUSH: [(CompositeOp, f32); 3] = [
    (CompositeOp::Smear, 1.0),
    (CompositeOp::Blur, 1.0),
    (CompositeOp::Brush, 1.0),
];

/// ⭐⭐⭐ **COM UM ESFREGÃO POR CIMA, A CURA NÃO MUDA UM BIT — e sem a cerca a imagem muda.**
///
/// O esfregão desloca píxeis, logo LÊ a orla que o borrão de baixo deixa. A cerca devolve ao
/// borrão o `alvo` inteiro sempre que há uma camada destas acima dele — e então o caminho é
/// **exactamente** o de antes da cura, ao bit, em cada lote.
///
/// **Medido** (`diag_a_cerca_do_borrao`, faixa do traço, três sementes com textura, passos `2`/`8`):
///
/// | cerca | pior (um lote contra N) |
/// |---|---|
/// | a do produto | `25`–`28` |
/// | revertida (o código de antes) | `25`–`28`, **idêntico** |
/// | DESLIGADA (o controlo) | **`72`–`78`** |
///
/// ⚠️ **A primeira linha não é zero, e isso NÃO é da cura:** o esfregão sobre o borrão já dependia
/// da entrega em lotes antes dela (a linha do meio é a mesma). É dívida PRÉ-EXISTENTE, registada
/// na fila do handoff (§33.12) — e é por isso que este gate compara o produto com o código de antes
/// e não com um lote só: *a lei que a cura promete é «não muda nada aqui», não «isto está certo»*.
/// ⚠️ Ela não é o caso do dono (na pilha dele o esfregão está ABAIXO do borrão).
///
/// **Mutações que sangram:** a cerca devolver sempre `false` · a lista esquecer o `Smear` · a
/// cerca olhar para baixo.
#[test]
fn com_um_esfregao_por_cima_a_cura_nao_muda_um_bit() {
    let mut casos = 0usize;
    for semente in [7u64, 101, 999] {
        for passo in [2.0f32, 8.0] {
            let produto = monta(&SMEAR_BLUR_BRUSH, semente, passo, 0);
            let antes = monta(&SMEAR_BLUR_BRUSH, semente, passo, CURA_REVERTIDA);
            assert!(
                produto == antes,
                "com um esfregão acima do borrão, a cura tem de ser o caminho de antes AO BIT \
                 (semente {semente}, passo {passo})"
            );
            casos += 1;
        }
    }
    assert!(casos >= 6, "o corpus encolheu: {casos}");
    // O CONTROLO: sem a cerca a fixtura CONTÉM o fenómeno — senão a igualdade acima não prova nada.
    let (pior_sem_cerca, _, _) = diferenca(&SMEAR_BLUR_BRUSH, 7, 2.0, CERCA_DESLIGADA);
    let (pior_produto, _, _) = diferenca(&SMEAR_BLUR_BRUSH, 7, 2.0, 0);
    assert!(
        pior_sem_cerca >= 2 * pior_produto && pior_sem_cerca > 40,
        "controlo: sem a cerca o esfregão tem de ler a orla POR BORRAR e a imagem mudar \
         (sem cerca pior {pior_sem_cerca}, produto {pior_produto})"
    );
}

/// ⭐⭐ **DOIS BORRÕES EMPILHADOS não dependem da taxa do rato** — a metade LATENTE da cerca.
///
/// Hoje o painel só deixa ter UM borrão (`quota_da_operacao`), e o gate monta os dois à mão por
/// isso mesmo: é o dia em que a quota subir que ele protege. Ele cobre DUAS coisas que só este
/// arranjo exerce — o braço `Blur` da cerca e o avental ser a SOMA dos núcleos e não o máximo (com
/// tamanhos diferentes, o de baixo tem de ser exacto até ao alcance do de cima).
///
/// **Medido** (`diag_a_cerca_do_borrao`): `Blur(2)/Blur(1)/Brush` lê `pior 1` (o último byte da
/// quantização, o mesmo do `a_ordem_…`) com a cerca, e **`12`–`13`** sem ela.
///
/// **Mutações que sangram:** a lista esquecer o `Blur` · a cerca apagada.
///
/// ⛔ **O avental voltar a ser o MÁXIMO NÃO sangra, e é EQUIVALENTE hoje** (medido pela prova de
/// mutação de 2026-09-23): o avental é `k·P + 1`, e o borrão de caixa só LÊ
/// `Σ box_radii(k·P) ≈ 1,5·√(2kP)` à volta da região — o avental sobre-provisiona o alcance por
/// uma ordem de grandeza, logo o máximo de dois já cobre a soma dos dois alcances. A SOMA fica
/// porque é a resposta que não depende dessa folga: no dia em que o avental passar a ser o alcance
/// real, o máximo parte isto e este gate sangra.
#[test]
fn dois_borroes_empilhados_nao_dependem_da_taxa_do_rato() {
    let pilha = [
        (CompositeOp::Blur, 2.0),
        (CompositeOp::Blur, 1.0),
        (CompositeOp::Brush, 1.0),
    ];
    let mut casos = 0usize;
    for semente in [7u64, 101, 999] {
        for passo in [2.0f32, 8.0] {
            let (pior, _, bytes) = diferenca(&pilha, semente, passo, 0);
            assert!(
                pior <= 1 && bytes <= 16,
                "dois borrões empilhados: a entrega em lotes mudou a imagem \
                 (semente {semente}, passo {passo}: pior {pior}, {bytes} bytes)"
            );
            casos += 1;
        }
    }
    assert!(casos >= 6, "o corpus encolheu: {casos}");
    let (pior_sem_cerca, _, _) = diferenca(&pilha, 7, 2.0, CERCA_DESLIGADA);
    assert!(
        pior_sem_cerca >= 8,
        "controlo: sem a cerca o borrão de cima lê a orla por borrar (pior {pior_sem_cerca})"
    );
}

/// ⭐⭐ **O BYTE que o `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` tolera é ANTERIOR à cura** — a
/// afirmação que o afrouxamento daquela barra fazia sem prova reproduzível (auditoria 2026-09-23).
///
/// O borrão de uma sub-região não é o miolo do borrão da região maior (a soma corrente carrega o
/// sítio onde começou, `~6e-4` em `f32`), logo com textura um pixel raro muda UM byte conforme a
/// entrega em lotes. **Medido** em `41` telas (`40` com textura e a chapada) × dois passos:
///
/// | | casos com diferença | pior |
/// |---|---|---|
/// | a cura | `12` de `82` | `1` |
/// | o código de antes | `25` de `82` | `1` |
///
/// ⇒ a barra `pior ≤ 1` é o último byte da quantização e existia antes da cura; o que a cura mudou
/// foi QUAIS telas o sorteiam (a chapada passou a sorteá-lo, e é a do `a_ordem_…`).
#[test]
fn o_byte_do_borrao_em_lotes_e_anterior_a_cura() {
    let pilha = [(CompositeOp::Blur, 1.0), (CompositeOp::Brush, 1.0)];
    let mede = |modo: u8| {
        let (mut casos, mut com, mut pior, mut max_bytes) = (0usize, 0usize, 0u8, 0usize);
        for semente in (1..=40u64).chain([CHAPADA]) {
            for passo in [2.0f32, 8.0] {
                let (p, _, b) = diferenca(&pilha, semente, passo, modo);
                casos += 1;
                com += usize::from(p > 0);
                pior = pior.max(p);
                max_bytes = max_bytes.max(b);
            }
        }
        (casos, com, pior, max_bytes)
    };
    let (casos, com_cura, pior_cura, bytes_cura) = mede(0);
    let (_, com_antes, pior_antes, _) = mede(CURA_REVERTIDA);
    assert_eq!(casos, 82, "o corpus mudou");
    assert!(
        pior_cura <= 1 && bytes_cura <= 16,
        "a cura: o borrão em lotes passou do último byte (pior {pior_cura}, {bytes_cura} bytes)"
    );
    // A PREMISSA, agora medida: o código de antes também sorteava o byte.
    assert!(
        pior_antes == 1 && com_antes >= 10,
        "a premissa morreu: o código de antes da cura deixou de ter o byte \
         (pior {pior_antes}, {com_antes} casos) — reveja a barra do `a_ordem_…`"
    );
    assert!(
        com_cura <= com_antes,
        "a cura passou a sortear o byte MAIS vezes do que o código de antes \
         ({com_cura} contra {com_antes})"
    );
}
