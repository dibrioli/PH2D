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

use super::super::rig_demo::tests::{CAMPO, CORDA, DT, FK, TIQUES, pontos, primeiro};
use super::build;
use crate::motion_state::MotionState;

/// As SONDAS desta cena — ver o cabeçalho do irmão. ⚠️ Elas vivem noutro ficheiro por
/// RESPONSABILIDADE (o tecto de LOC impôs o corte): aqui afirma-se, ali mede-se.
#[path = "motion_state_rig_demo_corda_sondas.rs"]
mod sondas;

/// ⭐⭐⭐ **A CORDA LÊ-SE COMO UM CORDÃO E NÃO COMO UM ROSÁRIO** — ordem do dono (2026-09-20):
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
/// de 2026-09-20 (*«a rot não acontece e o centro da rot não é a base do segmento»*), medidos na
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
///    2026-09-20, na varredura que o dono pediu depois do smoke da unidade): o comprimento de uma
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
/// ⛔⛔⛔ **A PREMISSA DESTE NÚMERO MORREU em 2026-09-20, e a morte está à vista no diff.** Ele
/// valia `0,05` e a justificação era: *«a corda é um solver, e às `TIQUES` de queda o vão já não é
/// o de repouso — o pior segmento estica `2,52 %`»*. Isso era verdade enquanto o comprimento da
/// peça fosse um número ESCRITO À MÃO, de que o vão se afastava ao esticar.
///
/// ⭐ Desde a ordem do dono (*«DEVE SIM»*) o [`ph2d_node_rig_bones::veste`] escreve o `size` de
/// cada osso do `len` dele, e o `len` sai das MESMAS posições de que o vão é medido ⇒ a peça já
/// não se afasta do vão: **ela segue-o**. O esticão deixou de ser um erro para ser a coisa que a
/// peça acompanha, e a barra passa a medir só a aritmética entre os dois (uma divisão e uma
/// multiplicação por dois, as duas exactas em binário).
const TOLERANCIA_DO_VAO: f32 = 1e-5;

/// ⭐⭐⭐ **A PEÇA VESTE O OSSO EM TODO O CURSO DO KNOB** — ordem do dono (2026-09-20), e este gate
/// é a **tabela da sonda virada catraca**.
///
/// O [`diag_a_peca_contra_o_vao`] mediu o defeito célula a célula (`0,47×` · `1,85×` · `2,25×` ·
/// `0,50×`) e a tabela dele está no doc do [`ph2d_node_rig_bones::veste`]. Depois da lei, as
/// **sete** células leem `1,00×` — e é isso que fica preso aqui, porque *uma tabela impressa numa
/// sonda que ninguém corre não impede regressão nenhuma*.
///
/// ⚠️ **As duas metades, e a segunda é o CONTROLO:**
///
/// 1. as sete células de uma cadeia de RIG (a corda por `Count`, os ossos por `Length`) vestem o
///    osso;
/// 2. o pano do **CAMPO** — que não passa por `rig.bones` — continua a desenhar o tamanho
///    AUTORADO. Sem ela, um `peca_contra_vao` que devolvesse o vão nos dois lados passaria a
///    primeira metade **afirmando nada**, e a lei ficaria indistinguível de uma tautologia do
///    arnês.
///
/// FALSIFICADO por apagar a chamada ao `veste` (a 1.ª metade lê `19,53×` na corda), por lhe tirar
/// o meio (`2,00×`) ou por pôr a lei a alcançar quem não é rig (a 2.ª metade cai).
#[test]
fn a_peca_veste_o_osso_em_todo_o_curso_dos_knobs() {
    for count in [10.0f32, 20.0, 30.0, 40.0] {
        let (vao, desenhado, n) = mede_a_corda(Some(count));
        assert!(n > 0, "Count={count}: a corda desenha alguma coisa");
        assert!(
            (desenhado - vao).abs() / vao < TOLERANCIA_DO_VAO,
            "Count={count}: a peca veste o osso — desenhada {desenhado:.6} contra um vao de \
             {vao:.6} ({:.2}x)",
            desenhado / vao
        );
    }
    for length in [0.2f32, 0.45, 0.9] {
        let (vao, desenhado, n) = mede_o_osso(length);
        assert!(n > 0, "Length={length}: a cadeia desenha alguma coisa");
        assert!(
            (desenhado - vao).abs() / vao < TOLERANCIA_DO_VAO,
            "Length={length}: a peca veste o osso — desenhada {desenhado:.6} contra um vao de \
             {vao:.6} ({:.2}x)",
            desenhado / vao
        );
    }
    // ⭐ **O CONTROLO: a lei é dos OSSOS e não do carimbo.** O pano do campo é uma grelha que
    // nunca passa por `rig.bones`, logo a peça dele **não** mede o vão — ela mede o que a cadeia
    // dele escreveu (o `size` do próprio `motion.wave`, que a cena multiplica). Se a lei
    // alcançasse todo carimbo, este número seria o vão da grelha.
    //
    // ⚠️ A barra é a RAZÃO e não um número: o que se afirma é que os dois são grandezas
    // diferentes, e `1,5×` separa-os com folga (medido: `3,9×`).
    // ⭐⭐ **E a peça veste-o nos DOIS eixos NUMA CADEIA UNIFORME** — a fileira dos ossos tem
    // `len` todos iguais, logo o menor É o `len`, e as duas leis (a de hoje e a de ontem)
    // coincidem ao bit. ⚠️ **Esta metade deixou de afirmar a LEI e passou a afirmar a
    // DEGENERESCÊNCIA dela:** é ela que prova que o desenho que o dono aprovou não se mexeu. A
    // lei a sério — a espessura ser da CADEIA e não do osso — é o
    // [`a_corda_nao_afina_no_final`], que a mede onde o `len` varia.
    let (sx, sy) = mede_os_dois_eixos_do_osso();
    assert!(
        (sx - sy).abs() / sx < TOLERANCIA_DO_VAO,
        "numa cadeia UNIFORME as duas leis coincidem: size = [{sx:.6}, {sy:.6}]"
    );
    let desenhado = mede_a_peca_do_campo();
    let vao_da_grelha = super::CAMPO_VAO;
    let razao = vao_da_grelha / desenhado;
    assert!(
        razao > 1.5,
        "o pano do CAMPO nao passa por `rig.bones`, logo a peca dele NAO mede o vao: desenhada \
         {desenhado:.6} contra um vao de {vao_da_grelha:.6} ({razao:.2}x)"
    );
}

/// ⭐⭐⭐ **A CORDA NÃO AFINA NO FINAL** — report do dono (2026-09-20), e o gate que a lei nova
/// precisa de ter onde ela se distingue da anterior.
///
/// ⛔⛔ **A régua é a ESPESSURA (`size.y`), e a sonda que a mediu primeiro não a tinha:** ela
/// imprimia só o COMPRIMENTO (`size.x`), que **deve** seguir o vão — é isso que faz a peça ir de
/// uma junta à seguinte. Com a cura no sítio a tabela dela saiu **idêntica**, e a leitura ingénua
/// era *«a cura não fez nada»*. *Uma régua que lê um eixo não vê o outro.*
///
/// ⚠️ **As DUAS metades, e a segunda é o CONTROLO:**
///
/// 1. a espessura é a MESMA do 1.º segmento ao último, em todo o curso do `Count`;
/// 2. o **comprimento** continua a seguir o vão, e a `Count = 80` ele varia `0,72×` — *sem esta,
///    uma lei que congelasse os DOIS eixos passaria a primeira e devolveria o rosário que a wave
///    da peça justa existiu para curar*.
///
/// FALSIFICADO por devolver a escala uniforme ao [`ph2d_node_rig_bones::veste`] (a 1.ª metade lê
/// `0,72×`), ou por congelar também o comprimento (a 2.ª cai).
#[test]
fn a_corda_nao_afina_no_final() {
    for count in [20.0f32, 40.0, 80.0] {
        let (compr, espess, n) = mede_as_pontas_da_corda(count);
        assert!(n >= 10, "Count={count}: a corda desenha {n} pecas");
        assert!(
            (espess - 1.0).abs() < 1e-5,
            "Count={count}: a ESPESSURA e' a mesma de ponta a ponta ({espess:.4}x)"
        );
        if count >= 80.0 {
            assert!(
                compr < 0.85,
                "Count={count}: o COMPRIMENTO continua a seguir o vao ({compr:.4}x) — uma lei que \
                 congelasse os dois eixos devolvia o rosario"
            );
        }
    }
}

/// `(razão do comprimento, razão da espessura, nº de peças)` do 1.º segmento ao último, em regime.
fn mede_as_pontas_da_corda(count: f32) -> (f32, f32, usize) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let corda = primeiro(&m.doc.graph, "motion.verlet_rope");
    m.doc.graph.set_param(corda, "count", count);
    let sink = sinks[CORDA];
    let mut t = 0.0f64;
    for _ in 0..TIQUES {
        let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, sink, t);
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
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
    let (p, u) = (
        *pecas.first().expect("ha' pecas"),
        *pecas.last().expect("ha' pecas"),
    );
    (u.size[0] / p.size[0], u.size[1] / p.size[1], pecas.len())
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

/// **Os dois semi-eixos da 1.ª peça da fileira dos OSSOS** — a metade do gate que prende a escala
/// a ser um PAR e não um comprimento.
fn mede_os_dois_eixos_do_osso() -> (f32, f32) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sinks[FK], 0.0)
        .expect("o sink coze");
    let mut pecas = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(
        saida[0].as_stream(),
        ph2d_render::SinkStyle::PLAIN,
        &mut pecas,
    );
    assert!(!pecas.is_empty(), "a fileira dos ossos desenha");
    (pecas[0].size[0], pecas[0].size[1])
}

/// **O comprimento DESENHADO da 1.ª peça do pano do CAMPO** — o controlo do gate acima.
///
/// ⚠️ Ele não é um vão: o campo é uma grelha e o tamanho da peça dele é AUTORADO, que é
/// precisamente o que este número existe para mostrar.
fn mede_a_peca_do_campo() -> f32 {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sinks[CAMPO], 0.0)
        .expect("o sink coze");
    let mut pecas = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(
        saida[0].as_stream(),
        ph2d_render::SinkStyle::PLAIN,
        &mut pecas,
    );
    assert!(!pecas.is_empty(), "o pano do campo desenha alguma coisa");
    2.0 * pecas[0].size[0]
}

/// O vão entre as duas primeiras cabeças contra `2 × size` da 1.ª peça desenhada.
fn peca_contra_vao(s: &ph2d_nodegraph::attr::Stream, juntas: &[[f32; 2]]) -> (f32, f32, usize) {
    let mut pecas = Vec::new();
    ph2d_eval_motion::lower_to_vector_instances_onto(s, ph2d_render::SinkStyle::PLAIN, &mut pecas);
    let d = [juntas[1][0] - juntas[0][0], juntas[1][1] - juntas[0][1]];
    (d[0].hypot(d[1]), 2.0 * pecas[0].size[0], pecas.len())
}
