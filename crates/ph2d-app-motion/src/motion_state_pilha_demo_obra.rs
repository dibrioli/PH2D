//! ⭐⭐⭐ **AS MEDIÇÕES QUE DESENHAM A OBRA ENCOMENDADA** — as do
//! **[doc 111](../../../docs/Motion%20Nodes/111_o_motor_de_contacto_com_memoria.md)**, ordem do dono
//! de 2026-09-15 (*«encomendas o motor de contacto novo»*).
//!
//! Irmão do [`super::curas`] pelo tecto de LOC (HR-18) e por ASSUNTO: lá medem-se as curas
//! **REFUTADAS** do zumbido (doc 109 §8); aqui medem-se os **factos que decidem o desenho** do motor
//! novo — se as peças têm material, se os contactos duram, quantos encostos tem uma peça, e se há
//! identidade para lhes servir de chave.
//!
//! ⚠️ **Três destas sondas já mudaram a espec**: a primeira dissolveu uma wave inteira (as peças
//! nunca foram gelo), a terceira transformou a wave do dispositivo numa consequência (o cache cabe
//! numa coluna), e a quarta impôs uma cerca à lei que ainda não existe (sem `id`, a memória só vale
//! com a população fixa).

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// **SONDA — a W0: as peças da `=114` são MESMO gelo?**
///
/// ⚠️⚠️ O doc 109 §8.5 e o doc 111 §2 afirmam que sim (*«a cena não escreve material nenhum»*). Esta
/// sonda lê a coluna do stream COZIDO em vez de a supor — *uma ausência afirmada sem olhar a coluna
/// é um palpite com cara de medição*.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_as_pecas_sao_gelo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_as_pecas_sao_gelo() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let s = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, sinks[1], 1.0)
        .expect("cozinha")[0]
        .as_stream()
        .clone();
    for col in [
        ph2d_nodegraph::attr::FRICTION_COLUMN,
        ph2d_nodegraph::attr::BOUNCE_COLUMN,
        ph2d_nodegraph::attr::ROLLING_COLUMN,
    ] {
        match s.get(col) {
            Some(Column::Scalar(v)) => eprintln!(
                "  {col:<10} PRESENTE, {} linhas, valor[0] = {:.3}",
                v.len(),
                v.first().copied().unwrap_or(f32::NAN)
            ),
            _ => eprintln!("  {col:<10} AUSENTE  ⇒ o par cai no `Material::LISO`"),
        }
    }
    let mats = ph2d_contact::materiais(&s);
    eprintln!(
        "\n  `ph2d_contact::materiais` devolve {} ⇒ μ do par = {:.3}",
        if mats.is_some() { "Some" } else { "None" },
        mats.as_ref()
            .and_then(|m| m.first())
            .map_or(0.0, |m| ph2d_contact::atrito::mu(m.atrito, m.atrito))
    );
}

/// **SONDA — W2 do [doc 111]: os CONTACTOS sobrevivem ao tique seguinte?**
///
/// ⭐⭐⭐ É o facto que decide se a obra encomendada é sequer APLICÁVEL. O `λ` acumulado (*warm
/// starting*) precisa de uma chave estável entre tiques: se as feições mudarem todas, **não há o que
/// aquecer**, e um `λ` guardado numa chave que mudou é **pior que nenhum** — ele aquece o contacto
/// errado.
///
/// A chave medida é o PAR `(id menor, id maior)`, que é o mais grosseiro possível: se nem ela
/// sobreviver, uma chave com a FEIÇÃO dentro sobrevive ainda menos.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_os_contactos_sobrevivem -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_os_contactos_sobrevivem() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let mut anterior: Vec<(usize, usize)> = Vec::new();
    let (mut vivos, mut novos, mut tiques) = (0_usize, 0_usize, 0_usize);
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sinks[1], t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if t >= 2.0
            && let (Some(Column::Vec2(p)), Some(cols)) = (s.get("P"), ph2d_contact::colisores(&s))
        {
            let mut agora = Vec::new();
            for i in 0..p.len() {
                for j in (i + 1)..p.len() {
                    if let (Some(a), Some(b)) = (cols[i], cols[j])
                        && ph2d_contact::contato(&a, p[i], &b, p[j], (i + j) % 2 == 0).is_some()
                    {
                        agora.push((i, j));
                    }
                }
            }
            if !anterior.is_empty() {
                vivos += agora.iter().filter(|c| anterior.contains(c)).count();
                novos += agora.iter().filter(|c| !anterior.contains(c)).count();
                tiques += 1;
            }
            anterior = agora;
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let total = vivos + novos;
    eprintln!("\n  sobre {tiques} tiques da janela assente:");
    eprintln!("  contactos que SOBREVIVEM ao tique anterior: {vivos}");
    eprintln!("  contactos NOVOS                           : {novos}");
    eprintln!(
        "  ⇒ taxa de sobrevivência: {:.1} %",
        100.0 * vivos as f32 / total.max(1) as f32
    );
    eprintln!("\n  ⚠️ abaixo de ~80 % nao ha o que aquecer: doc 111 §5 W2.");
}

/// **SONDA — W3 do [doc 111] §4.2: quantos encostos tem UMA peça?**
///
/// ⭐⭐⭐ É a pergunta que decide a **MORADA** do cache. Um `λ` por PAR não é uma coluna — mas se
/// cada peça tiver no máximo `K` parceiros, o cache é **por ELEMENTO e de largura fixa**, logo *é*
/// uma coluna: viaja no laço do estado como o `age` e o `sim_t`, e um kernel do dispositivo lê-o
/// sem substrato novo. ⇒ **a W4 deixa de ser uma wave e passa a ser uma consequência.**
///
/// ⚠️ Se a cauda for longa, o cache tem de ser uma tabela lateral, e aí ele **não atravessa a
/// fronteira do dispositivo** — que é o defeito que a obra vem curar, um nível acima.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_quantos_encostos_por_peca -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_quantos_encostos_por_peca() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let mut hist = [0_usize; 16];
    let (mut amostras, mut pior) = (0_usize, 0_usize);
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sinks[1], t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if t >= 2.0
            && let (Some(Column::Vec2(p)), Some(cols)) = (s.get("P"), ph2d_contact::colisores(&s))
        {
            for i in 0..p.len() {
                let mut c = 0;
                for j in 0..p.len() {
                    if i != j
                        && let (Some(a), Some(b)) = (cols[i], cols[j])
                        && ph2d_contact::contato(&a, p[i], &b, p[j], (i + j) % 2 == 0).is_some()
                    {
                        c += 1;
                    }
                }
                hist[c.min(15)] += 1;
                pior = pior.max(c);
                amostras += 1;
            }
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    eprintln!("\n  encostos | peças-tique | acumulado");
    let mut acc = 0;
    for (c, n) in hist.iter().enumerate() {
        if *n == 0 && c > pior {
            break;
        }
        acc += n;
        eprintln!(
            "  {c:>8} | {n:>11} | {:>7.2} %",
            100.0 * acc as f32 / amostras as f32
        );
    }
    eprintln!("\n  pior caso: {pior} encostos numa peça (sobre {amostras} peças-tique)");
}

/// **SONDA — W3: a CHAVE existe? A `=114` traz a coluna `id`?**
///
/// ⛔⛔ Sem `id` a chave de um contacto tem de ser o ÍNDICE, e um índice **não sobrevive a um
/// nascimento nem a uma morte** (o `sim.spawn` e o `sim.lifetime` reindexam a corrente). Um `λ`
/// guardado numa chave que se deslocou aquece o **contacto errado**, que é pior que não aquecer.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_a_pilha_tem_identidade -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_pilha_tem_identidade() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let s = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, sinks[1], 2.5)
        .expect("cozinha")[0]
        .as_stream()
        .clone();
    match s.get("id") {
        Some(Column::Scalar(v)) => {
            let mut u: Vec<f32> = v.clone();
            u.sort_by(f32::total_cmp);
            u.dedup();
            eprintln!(
                "  `id` PRESENTE: {} linhas, {} valores distintos, [0]={:.0} [n-1]={:.0}",
                v.len(),
                u.len(),
                v.first().copied().unwrap_or(f32::NAN),
                v.last().copied().unwrap_or(f32::NAN)
            );
        }
        _ => eprintln!("  `id` AUSENTE ⇒ a chave teria de ser o ÍNDICE"),
    }
    eprintln!(
        "  colunas do stream: {:?}",
        s.columns().map(|(k, _)| k.as_str()).collect::<Vec<_>>()
    );
}

/// **SONDA — os SUB-PASSOS: a última saída de pé** (doc 111 §5.7, ordem do dono 2026-09-15).
///
/// ⭐⭐⭐ As três formas de comprar silêncio caíram — baixar o ganho (§8.9), filtrar (§5.5) e
/// amolecer (§5.6). Sobra a que o oráculo usa e que esta obra nunca tocou: **partir o tique em `N`
/// passos**, cada um com a sua integração **e** o seu contacto. É por isso que um solver de impulsos
/// assenta a **rigidez plena**, sem amolecer nada — e ⭐ **o knob já existe no cartão da zona**.
///
/// ⚠️⚠️ **AS QUATRO RÉGUAS, porque três não chegaram nenhuma vez:** o balanço vê o tremor, o giro vê
/// o rodopio, o `y` vê a pilha congelada no ar, e o **VÃO** vê a pilha colapsada. Cada cura desta
/// caça melhorou a grandeza medida e estragou uma que ninguém media.
///
/// ⚠️ **E o PREÇO vai na mesma tabela**, que é o que o `CLAUDE.md` §0.0 exige de qualquer tecto:
/// `N` sub-passos custam `N` vezes o solver.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_os_sub_passos -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_os_sub_passos() {
    eprintln!("\n  substeps | balanço pior | giro pior  |  y final | VÃO típico | cook");
    eprintln!("  ---------|--------------|------------|----------|------------|--------");
    for n in [1_u32, 2, 4, 8, 16] {
        let (mut bs, mut gs) = (Vec::new(), Vec::new());
        for k in 0..5 {
            #[expect(clippy::cast_precision_loss, reason = "um indice pequeno")]
            let eps = (k as f32 - 2.0) * 1e-3;
            let (b, g) = realizacao_com_substeps(eps, n);
            bs.push(b);
            gs.push(g);
        }
        bs.sort_by(f32::total_cmp);
        gs.sort_by(f32::total_cmp);
        let (y, vao, ms) = altura_vao_e_relogio(n);
        eprintln!(
            "  {n:>8} | {:>5.3}..{:>5.3} | {:>4.1}..{:>4.1} | {y:>8.2} | {vao:>10.4} | {ms:>5.2} ms",
            bs[0],
            bs[bs.len() - 1],
            gs[0],
            gs[gs.len() - 1],
        );
    }
    eprintln!("\n  ⚠️ barra: balanço ≤ 0,15 · giro ≤ 28 · y ≪ −2 · VÃO ≈ 0,31 (o de hoje).");
    eprintln!("  ⚠️ hoje (substeps 1): 3,79..5,69 · 24,3..28,4 · −2,56 · 0,3094.");
}

/// A cena com `substeps` escrito na zona da DIREITA e o berço deslocado `eps`.
///
/// ⚠️ **`pub(super)` porque o irmão [`super::salto`] a consome** — uma 2.ª cópia desta montagem
/// seria a 2.ª resposta à pergunta *«que cena estou a medir?»*, e é dela que sai a perturbação do
/// berço que dá o ruído entre realizações.
pub(super) fn com_substeps(eps: f32, substeps: u32) -> (MotionState, NodeId) {
    com_substeps_e_arrasto(eps, substeps, None)
}

/// Idem, com o `damping` do `sim.step` escrito (⚠️ `1,0` = SEM arrasto, que é o default do nó).
///
/// ⭐ Ele existe porque a cena **nunca o usou**: enquanto a resposta de velocidade matava a
/// velocidade ABSOLUTA de cada peça, o monte assentava por sobre-amortecimento e ninguém reparou
/// que não havia travão nenhum declarado. Com o impulso do par (doc 111 §5.12) a física ficou
/// certa e o arrasto passou a ser a pergunta.
pub(super) fn com_substeps_e_arrasto(
    eps: f32,
    substeps: u32,
    arrasto: Option<f32>,
) -> (MotionState, NodeId) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    let tipo = |state: &MotionState, t: &str| -> Vec<NodeId> {
        state
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == t)
            .map(|n| n.id)
            .collect()
    };
    // ⛔⛔ **AS DUAS zonas, e a lição custou uma tabela inteira.** O relógio do sub-passo é do
    // GRAFO, não do nó: [`ph2d_nodegraph::cook::graph_substeps`] toma o **MÁXIMO** sobre todo nó
    // que declara o param. Esta sonda escrevia só na zona da DIREITA (`.last()`) — e no dia em que
    // a cura assou `SUBSTEPS = 8` no `build()`, a zona da ESQUERDA passou a fixar um CHÃO de `8`:
    // pedir `1`, `2`, `4` ou `8` devolvia a mesma célula, e só o `16` se mexia.
    //
    // ⚠️⚠️ *A régua ficou cega exactamente abaixo do valor que a cura shipou* — ela media
    // `max(8, pedido)` e lia-se como uma varredura. É a mesma forma do censo que varre por prefixo
    // e passa a varrer zero: **a cura mudou o sujeito da medição, e a medição não soube**.
    for zona in tipo(&state, "sim.zone") {
        #[expect(clippy::cast_precision_loss, reason = "uma contagem pequena")]
        let v = substeps as f32;
        state.doc.graph.set_param(zona, "substeps", v);
    }
    if let Some(alto) = tipo(&state, "motion.transform").last().copied() {
        let x = state
            .doc
            .graph
            .node_param_overrides(alto)
            .and_then(|o| o.get("offset_x").copied())
            .unwrap_or(0.0);
        state.doc.graph.set_param(alto, "offset_x", x + eps);
    }
    if let Some(a) = arrasto {
        for passo in tipo(&state, "sim.step") {
            state.doc.graph.set_param(passo, "damping", a);
        }
    }
    crate::motion_shape_gen::publish(&mut state, 0.0);
    (state, sinks[1])
}

/// `(balanço pior, giro líquido pior)` com `substeps`, **pela porta do PUMP**.
///
/// ⛔⛔ **A 1.ª versão desta sonda chamava o `cook` directamente e leu as cinco células IDÊNTICAS,
/// com o relógio a não subir.** O motor do substep é a MARCHA do pump (`advance_or_scrub_*`), não
/// o `Cook::cook` — *uma sonda que salta o pump mede um programa que não tem substeps*. É a mesma
/// forma de erro que o `CLAUDE.md` já regista sobre sondas que armam um módulo por outra porta.
fn realizacao_com_substeps(eps: f32, substeps: u32) -> (f32, f32) {
    use ph2d_nodegraph::attr::Column as C;
    let (mut state, sink) = com_substeps(eps, substeps);
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<f32>::new());
    let (mut liquido, mut n) = (Vec::<f32>::new(), 0_usize);
    for k in 0..=174_u64 {
        state.pump.mark_dirty();
        state.pump.advance_or_scrub_to_nodes_scoped(
            &state.doc.graph,
            &state.registry,
            &[sink],
            k,
            |t| {
                #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                let s = t as f64 / 60.0;
                s
            },
            &escopos,
        );
        let Some((_, saida)) = state
            .pump
            .boundary_streams()
            .iter()
            .find(|(no, _)| *no == sink)
        else {
            continue;
        };
        if let Some(C::Scalar(r)) = saida.get("rot") {
            if liquido.len() != r.len() {
                liquido = vec![0.0; r.len()];
                n = r.len();
            }
            if k >= 120 && anterior.len() == r.len() {
                passos.push(
                    r.iter()
                        .zip(&anterior)
                        .map(|(a, b)| (a - b).abs())
                        .collect(),
                );
                for i in 0..r.len() {
                    liquido[i] += r[i] - anterior[i];
                }
            }
            anterior = r.clone();
        }
    }
    let balanco = (0..n)
        .map(|i| super::tremor::mediana(&passos.iter().map(|l| l[i]).collect::<Vec<_>>()))
        .fold(0.0_f32, f32::max);
    (balanco, liquido.iter().fold(0.0_f32, |a, v| a.max(v.abs())))
}

/// `(y mediano, vão típico, ms por tique)` no fim — **pela porta do PUMP**.
///
/// ⛔⛔ **A 1.ª versão desta função marchava pelo `cook` directo**, e as colunas `y` e `VÃO` saíam
/// CONSTANTES em toda a varredura — porque estavam a medir a cena **sem substeps**, sempre. *Três
/// colunas de uma tabela podem vir da porta certa e duas da errada, e a tabela lê-se inteira.*
fn altura_vao_e_relogio(substeps: u32) -> (f32, f32, f64) {
    use ph2d_nodegraph::attr::Column as C;
    let (mut state, sink) = com_substeps(0.0, substeps);
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let (mut ys, mut vao, mut relogio) = (Vec::new(), 0.0_f32, 0.0_f64);
    for k in 0..=174_u64 {
        let agora = std::time::Instant::now();
        state.pump.mark_dirty();
        state.pump.advance_or_scrub_to_nodes_scoped(
            &state.doc.graph,
            &state.registry,
            &[sink],
            k,
            |t| {
                #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                let s = t as f64 / 60.0;
                s
            },
            &escopos,
        );
        if k >= 120 {
            relogio += agora.elapsed().as_secs_f64() * 1e3;
        }
        if k == 174
            && let Some((_, saida)) = state
                .pump
                .boundary_streams()
                .iter()
                .find(|(no, _)| *no == sink)
            && let Some(C::Vec2(p)) = saida.get("P")
        {
            ys = p.iter().map(|q| q[1]).collect();
            vao = super::tests::vizinho_mediano(p);
        }
    }
    (super::tremor::mediana(&ys), vao, relogio / 55.0)
}
