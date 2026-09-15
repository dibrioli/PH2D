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
