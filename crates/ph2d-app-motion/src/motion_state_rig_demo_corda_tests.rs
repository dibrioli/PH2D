//! **O QUE A CORDA ENTREGA E O QUE ELA DESENHA** — irmão dos gates da cena `=120`.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e foi imposto pelo tecto de LOC:** ali mede-se o que os
//! seis panos da cena fazem; aqui mede-se **o primeiro deles**, que é o que o dono trouxe em três
//! reports seguidos. As duas metades mudam por razões diferentes — uma quando a cena ganha um
//! pano, a outra quando a lei do segmento muda.
//!
//! ⛔⛔ **E há DUAS réguas de propósito, porque houve DOIS defeitos e só a segunda os via:** a
//! coluna que o nó escreve e a INSTÂNCIA que o desenho recebe. Um `rot` certo na coluna e lido
//! noutra unidade na borda lê-se, na tela, exactamente como um `rot` ausente — foi o que o dono
//! fotografou, com o gate da coluna VERDE por cima.

use super::super::rig_demo::tests::{CORDA, DT, FK, TIQUES, pontos, primeiro};
use super::build;
use crate::motion_state::MotionState;

/// ⭐⭐⭐ **A CORDA LÊ-SE COMO UM CORDÃO E NÃO COMO UM ROSÁRIO** — ordem do dono (2026-09-21):
/// *«o exemplo 1 (Rope) deve ser feito com Rope Segment e os segmentos devem ser conectados como
/// ossos senão a corda não parecerá um único objeto»*.
///
/// A régua é a que torna as duas coisas diferentes: cada peça carimbada tem de **ir de um ponto
/// da corda ao seguinte** — pousada no ponto `i` e virada para o ponto `i+1`. Uma conta não tem
/// direcção nenhuma, e é por isso que vinte contas se leem como vinte coisas.
///
/// ⚠️ **Ela mede a cena EM REGIME** (a corda nasce recta e uma corda recta não separa uma lei da
/// outra: todos os ângulos seriam iguais). Com `TIQUES` de queda a corda está curvada, e é aí que
/// um ângulo por segmento diz alguma coisa.
///
/// FALSIFICADO por qualquer elo da corrente: a corda deixar de publicar `parent`, o `rig.bones`
/// deixar de derivar o quadro, ou o pano voltar a carimbar directamente sobre a corda.
#[test]
fn os_segmentos_da_corda_apontam_ao_seguinte() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);

    let corda = primeiro(&m.doc.graph, "motion.verlet_rope");
    let ossos = primeiro(&m.doc.graph, "rig.bones");
    // ⚠️ **O laço é o do [`corre`], e as duas linhas dele são obrigatórias:** cozer o SINK (que é
    // quem puxa a cadeia inteira) e `advance_tick` (que é quem entrega o estado ao tique
    // seguinte). Sem a segunda a corda fica no repouso e a banda de ângulos lê `0,0000` — foi o
    // CONTROLO deste gate que o apanhou.
    let sink = sinks[CORDA];
    let mut t = 0.0f64;
    for _ in 0..TIQUES {
        let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, sink, t);
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
    let juntas = pontos(
        m.pump
            .cook
            .cook(&m.doc.graph, &m.registry, corda, t)
            .expect("a corda coze")[0]
            .as_stream(),
    );
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, ossos, t)
        .expect("os ossos cozem");
    let s = saida[0].as_stream();
    let cabecas = pontos(s);
    let Some(ph2d_nodegraph::attr::Column::Scalar(rot)) = s.get("rot") else {
        panic!("cada segmento tem de carregar o angulo dele");
    };

    assert_eq!(
        cabecas.len(),
        juntas.len() - 1,
        "uma corda de n pontos da n-1 segmentos"
    );
    // A corda TEM de estar curvada, senão a régua abaixo passaria sobre uma lei errada.
    let banda =
        rot.iter().fold(f32::MIN, |a, &b| a.max(b)) - rot.iter().fold(f32::MAX, |a, &b| a.min(b));
    assert!(
        banda > 0.1,
        "a corda esta curvada, logo os angulos diferem (banda {banda:.4} graus)"
    );

    for i in 0..cabecas.len() {
        let d = [
            juntas[i + 1][0] - juntas[i][0],
            juntas[i + 1][1] - juntas[i][1],
        ];
        assert!(
            (cabecas[i][0] - juntas[i][0]).abs() < 1e-5
                && (cabecas[i][1] - juntas[i][1]).abs() < 1e-5,
            "o segmento {i} e' pousado na junta {i}"
        );
        let esperado = d[1].atan2(d[0]).to_degrees();
        assert!(
            (rot[i] - esperado).abs() < 1e-3,
            "o segmento {i} aponta a junta {}: {:.4} contra {esperado:.4}",
            i + 1,
            rot[i]
        );
    }
}

/// ⭐⭐⭐ **A CORDA DESENHADA: cada peça RODA, e roda em volta da BASE** — os dois relatos do dono
/// de 2026-09-21 (*«a rot não acontece e o centro da rot não é a base do segmento»*), medidos na
/// INSTÂNCIA que o desenho recebe e não na coluna que o nó escreve.
///
/// ⛔⛔⛔ **O gate irmão estava VERDE enquanto o dono fotografava a corda por rodar**, e a razão é
/// a fronteira: o `rig.bones` escrevia o ângulo em RADIANOS e as duas rotas de lowering lêem o
/// `rot` em GRAUS (`ph2d_eval_motion::lower`, que o declara por escrito e converte na borda) ⇒ o
/// primeiro segmento saía `−1,999` e desenhava-se a **−2°**. *Uma lei medida a montante de uma
/// conversão não afirma nada sobre a conversão* — e o gate da coluna nasceu na MESMA wave que a
/// lei, logo herdou a unidade dela.
///
/// As três metades, e cada uma responde a uma frase:
/// 1. **a peça pousa na junta** (`world_pos`) e o pivô é a origem local (`anchor == 0`) — e a
///    outra metade desta frase vive num gate que já existia: o `a_cabeca_do_osso_cai_sobre_a_
///    posicao` mede que um símbolo de rig é cortado de `[0, 2]` (contra `[−1, 1]` de um carimbo
///    comum, que é o CONTROLO dele) ⇒ a origem local É a cabeça, e *o centro da rotação é a base
///    do segmento*. ⚠️ **Aqui não se re-mede isso**: as duas metades vivem em crates diferentes e
///    cada uma reprova por um motivo diferente;
/// 2. **o eixo desenhado aponta à junta seguinte** — a `basis` é `[cos, sin, −sin, cos]`, logo a
///    primeira coluna dela é para onde o `+x` local vai no mundo;
/// 3. **ela CHEGA à junta seguinte** — o comprimento DESENHADO (`2 × size`) contra o vão, ver
///    [`TOLERANCIA_DO_VAO`]. ⛔⛔ **Esta metade FALTAVA e o nome do gate prometia-a** (escrita a
///    2026-09-21, na varredura que o dono pediu depois do smoke da unidade): o comprimento de uma
///    peça de rig é um número da FORMA e o comprimento verdadeiro é a coluna `len` da corrente —
///    e **nada os liga**. Aqui eles batem porque o `CORDA_PECA` é derivado do vão; um `Count`
///    diferente no painel desenha `0,47×` ou `1,85×` do vão, que é a foto do rosário outra vez;
/// 4. **o CONTROLO**: a corda está curvada, logo as direcções DIFEREM — sem ele uma corda direita
///    (ou uma lei que não rodasse nada) passaria as primeiras.
///
/// FALSIFICADO por devolver o `atan2` cru no `derive_frame` (a metade 2 lê um eixo quase `+x`
/// enquanto a corda desce), ou por dar ao sink um pivô que não seja a origem (a metade 1 cai).
#[test]
fn cada_peca_da_corda_e_desenhada_da_junta_ate_a_seguinte() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let corda = primeiro(&m.doc.graph, "motion.verlet_rope");
    let sink = sinks[CORDA];

    let mut t = 0.0f64;
    for _ in 0..TIQUES {
        let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, sink, t);
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
    let juntas = pontos(
        m.pump
            .cook
            .cook(&m.doc.graph, &m.registry, corda, t)
            .expect("a corda coze")[0]
            .as_stream(),
    );
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, t)
        .expect("o sink coze");
    let mut pecas = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(
        saida[0].as_stream(),
        ph2d_render::SinkStyle::PLAIN,
        &mut pecas,
    );
    assert_eq!(
        pecas.len(),
        juntas.len() - 1,
        "uma corda de n juntas desenha n-1 pecas"
    );

    // O CONTROLO primeiro: uma corda direita deixaria as duas leis abaixo indistinguíveis.
    let eixos: Vec<f32> = pecas.iter().map(|p| p.basis[1].atan2(p.basis[0])).collect();
    let banda = eixos.iter().fold(f32::MIN, |a, &b| a.max(b))
        - eixos.iter().fold(f32::MAX, |a, &b| a.min(b));
    assert!(
        banda > 0.05,
        "a corda esta curvada, logo os eixos desenhados diferem (banda {banda:.4} rad)"
    );

    // A peça `0` pousa na junta `0` a menos do deslocamento do pano — o `motion.move` leva a
    // corda inteira para o sítio dela, e o que este gate mede é a RELAÇÃO com as juntas.
    let desvio = [
        pecas[0].world_pos[0] - juntas[0][0],
        pecas[0].world_pos[1] - juntas[0][1],
    ];
    for (i, peca) in pecas.iter().enumerate() {
        assert_eq!(
            peca.anchor,
            [0.0, 0.0],
            "o pivo da peca {i} e' a origem local dela, que num simbolo de rig e' a CABECA"
        );
        assert!(
            (peca.world_pos[0] - juntas[i][0] - desvio[0]).abs() < 1e-5
                && (peca.world_pos[1] - juntas[i][1] - desvio[1]).abs() < 1e-5,
            "a peca {i} e' pousada na junta {i}"
        );
        let d = [
            juntas[i + 1][0] - juntas[i][0],
            juntas[i + 1][1] - juntas[i][1],
        ];
        let n = d[0].hypot(d[1]);
        let (ux, uy) = (d[0] / n, d[1] / n);
        assert!(
            (peca.basis[0] - ux).abs() < 1e-3 && (peca.basis[1] - uy).abs() < 1e-3,
            "o eixo DESENHADO da peca {i} aponta a junta {}: [{:.4}, {:.4}] contra [{ux:.4}, {uy:.4}]",
            i + 1,
            peca.basis[0],
            peca.basis[1]
        );
        // ⭐⭐⭐ **E ela CHEGA** — a metade que faltava, e o nome deste gate prometia-a desde que
        // ele existe. Ver [`TOLERANCIA_DO_VAO`].
        let desenhado = 2.0 * peca.size[0];
        assert!(
            (desenhado - n).abs() / n < TOLERANCIA_DO_VAO,
            "a peca {i} CHEGA a junta {}: desenhada {desenhado:.5} contra um vao de {n:.5} \
             ({:.1}% de erro)",
            i + 1,
            (desenhado - n).abs() / n * 100.0
        );
    }
}

/// O quanto o comprimento DESENHADO de uma peça pode afastar-se do vão que ela atravessa.
///
/// ⛔⛔ **A barra sai de uma MEDIÇÃO e não de um gosto:** a corda é um solver, e às `TIQUES` de
/// queda o vão dela já não é o de repouso — o pior segmento estica **`2,52 %`** (medido pelo
/// [`diag_a_peca_contra_o_vao`], que imprime os vinte). Os `5 %` são esse número com folga, e são
/// **`9×` mais apertados** que o defeito que este gate existe para apanhar (`0,47×` e `1,85×`).
const TOLERANCIA_DO_VAO: f32 = 0.05;

/// ⭐⭐⭐ **A PEÇA CONTRA O VÃO** — a sonda que nomeia o que a varredura de 2026-09-21 achou
/// (ordem do dono, depois do smoke da unidade: *«veja se erro similar acontece em outros locais
/// do módulo»*).
///
/// ⛔⛔⛔ **O comprimento DESENHADO de uma peça de rig é um número da FORMA; o comprimento
/// VERDADEIRO é a coluna `len` da corrente — e o `len` não tem UM consumidor de desenho em toda a
/// casa.** Ele é escrito pelo `rig.bones` e pelo `source.lsystem`, e lido só pelo `fk::resolve`
/// (que reconstrói `P` com ele) e pelo próprio `rig.bones` (que pergunta se ele já lá está). Quem
/// decide o tamanho na tela é a coluna `size`, que vem da forma.
///
/// ⚠️ **Nas duas cenas isto bate porque o número foi DERIVADO à mão** (`CORDA_PECA` do vão da
/// corda, `OSSO_PECA` do `OSSO_LEN`), e é por isso que nenhum gate o via: eles leem `P`, `rot` e
/// `size`, e os três estão certos. O que nenhum lê é a RELAÇÃO entre `size` e o vão.
///
/// Medido (pela porta do produto, com a `TIQUES` de queda na corda):
///
/// | knob do painel | vão | desenhado | razão | o que se vê |
/// |---|---|---|---|---|
/// | `Count = 10`   | `0,21201` | `0,10000` | **`0,47×`** | um rosário, com buracos entre as contas |
/// | `Count = 20`   | `0,10252` | `0,10000` | `0,98×` | o cordão que o dono aprovou |
/// | `Count = 30`   | `0,06944` | `0,10000` | `1,44×` | as peças montam umas nas outras |
/// | `Count = 40`   | `0,05393` | `0,10000` | **`1,85×`** | uma barra contínua |
/// | `Length = 0,2` | `0,20000` | `0,45000` | **`2,25×`** | o mesmo, na fileira dos ossos |
/// | `Length = 0,45`| `0,45000` | `0,45000` | `1,00×` | a cadeia que ladrilha |
/// | `Length = 0,9` | `0,90000` | `0,45000` | **`0,50×`** | ossos soltos, um vão de cada dois vazio |
///
/// ⏳ **DECISÃO DO DONO** (as duas saídas, com o preço): (a) ficar como está — o artista escreve o
/// tamanho da peça a condizer com a corrente, e o painel não o ajuda; (b) uma peça de rig VESTIR o
/// osso dela (o `size` por elemento sai do `len`), que é o que faz a corda ler-se como um cordão
/// **em qualquer `Count`** e custa o `size` deixar de ser o que o artista escreveu na forma.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn diag_a_peca_contra_o_vao() {
    eprintln!("\n=== a peca DESENHADA contra o vao que ela atravessa ===");
    for count in [10.0f32, 20.0, 30.0, 40.0] {
        let (vao, desenhado, n) = mede_a_corda(Some(count));
        eprintln!(
            "  Count={count:>5}  pecas={n:>3}  vao={vao:.5}  desenhado={desenhado:.5}  \
             razao={:.2}x",
            desenhado / vao
        );
    }
    for length in [0.2f32, 0.45, 0.9] {
        let (vao, desenhado, n) = mede_o_osso(length);
        eprintln!(
            "  Length={length:>4}  pecas={n:>3}  vao={vao:.5}  desenhado={desenhado:.5}  \
             razao={:.2}x",
            desenhado / vao
        );
    }
}

/// O vão do 1.º segmento da corda e o comprimento que o desenho lhe dá, depois de `TIQUES`.
fn mede_a_corda(count: Option<f32>) -> (f32, f32, usize) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let corda = primeiro(&m.doc.graph, "motion.verlet_rope");
    if let Some(c) = count {
        m.doc.graph.set_param(corda, "count", c);
    }
    let sink = sinks[CORDA];
    let mut t = 0.0f64;
    for _ in 0..TIQUES {
        let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, sink, t);
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
    let juntas = pontos(
        m.pump
            .cook
            .cook(&m.doc.graph, &m.registry, corda, t)
            .expect("a corda coze")[0]
            .as_stream(),
    );
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, t)
        .expect("o sink coze");
    peca_contra_vao(saida[0].as_stream(), &juntas)
}

/// O mesmo, na fileira do FK — a cadeia de ossos do `rig.skeleton`.
fn mede_o_osso(length: f32) -> (f32, f32, usize) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let esq = primeiro(&m.doc.graph, "rig.skeleton");
    m.doc.graph.set_param(esq, "length", length);
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sinks[FK], 0.0)
        .expect("o sink coze");
    let s = saida[0].as_stream();
    let pos = pontos(s);
    peca_contra_vao(s, &pos)
}

/// O vão entre as duas primeiras cabeças contra `2 × size` da 1.ª peça desenhada.
fn peca_contra_vao(s: &ph2d_nodegraph::attr::Stream, juntas: &[[f32; 2]]) -> (f32, f32, usize) {
    let mut pecas = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(s, ph2d_render::SinkStyle::PLAIN, &mut pecas);
    let d = [juntas[1][0] - juntas[0][0], juntas[1][1] - juntas[0][1]];
    (d[0].hypot(d[1]), 2.0 * pecas[0].size[0], pecas.len())
}
