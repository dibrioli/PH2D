//! **O ASPECTO DE UMA FOLHA** — o tamanho final e os dois sorteios.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! responsabilidade: o irmão mede ONDE a folha nasce e em que ordem se desenha, e este mede COM
//! QUE CARA.
//!
//! ⛔⛔ **A lição que os dois gates daqui pagaram:** o primeiro mede a LEI (pura) e o segundo o
//! CONSUMO dela pela membrana — e a mutação que apagava o consumo **sobreviveu** enquanto só
//! havia o primeiro. *Gatear a lei e não o consumidor é gatear metade.*

use crate::render_loop::motion_lsystem_gen::publish;
use crate::render_loop::motion_lsystem_testkit::*;
use ph2d_node_source_lsystem as ls;

/// ⭐⭐ **O TAMANHO FINAL E OS DOIS SORTEIOS** — report do Enio (2026-08-30): *"não temos
/// parâmetros para o tamanho final da folha nem jitter de scale e posição"*.
///
/// ⚠️ **A primeira afirmação protege as outras duas:** os três nascem NEUTROS e o neutro tem de
/// ser exacto — uma feature nova não pode mexer um bit no que já shipou.
#[test]
fn the_leaf_has_a_final_size_and_two_jitters() {
    use crate::render_loop::motion_lsystem_rows::LeafLook;
    let neutro = LeafLook {
        front: 0.0,
        keep_own_colour: true,
        size: 1.0,
        size_jitter: 0.0,
        pos_jitter: 0.0,
        // ⚠️ Fixa nestas fixturas de propósito: aqui a pergunta é *«a lei varia entre
        // MARCAS?»*, e é a marca que percorre `0..64`. Que a semente também mova os sorteios
        // é a pergunta do gate irmão, na costura (`re_rolling_the_seed_moves_the_leaves`).
        seed: 0,
    };
    // 1. ⛔ **O neutro é a identidade AO BIT.**
    for i in 0..64 {
        let (s, d) = neutro.at(i as u32);
        assert_eq!(
            s.to_bits(),
            1.0f32.to_bits(),
            "o tamanho neutro tem de ser 1 exacto"
        );
        assert_eq!(d, [0.0, 0.0], "o empurrao neutro tem de ser zero exacto");
    }
    // 2. O `Leaf Size` multiplica, e sozinho não sorteia nada.
    let dobro = LeafLook {
        size: 2.0,
        ..neutro
    };
    for i in 0..64 {
        assert_eq!(
            dobro.at(i as u32).0,
            2.0,
            "sem jitter todas as folhas tem o mesmo tamanho"
        );
    }
    // 3. O `Size Jitter` varia ENTRE folhas, dentro da faixa, e é determinístico.
    let variado = LeafLook {
        size: 1.0,
        size_jitter: 0.4,
        ..neutro
    };
    let tamanhos: Vec<f32> = (0..64).map(|i| variado.at(i as u32).0).collect();
    let (mn, mx) = tamanhos
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), s| (a.min(*s), b.max(*s)));
    assert!(
        mn >= 0.8 - 1e-5 && mx <= 1.2 + 1e-5,
        "fora da faixa +-20%: {mn}..{mx}"
    );
    assert!(mx - mn > 0.2, "as folhas nao variaram entre si: {mn}..{mx}");
    for (i, t) in tamanhos.iter().enumerate() {
        assert_eq!(
            variado.at(i as u32).0.to_bits(),
            t.to_bits(),
            "o sorteio reproduz"
        );
    }
    // 4. O `Position Jitter` empurra, dentro de meia folha, nos DOIS eixos.
    let empurrado = LeafLook {
        pos_jitter: 1.0,
        ..neutro
    };
    let ds: Vec<[f32; 2]> = (0..64).map(|i| empurrado.at(i as u32).1).collect();
    assert!(
        ds.iter()
            .all(|d| d[0].abs() <= 0.5 + 1e-5 && d[1].abs() <= 0.5 + 1e-5),
        "o empurrao saiu de meia folha"
    );
    let span = |k: usize| {
        let (a, b) = ds
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), d| (a.min(d[k]), b.max(d[k])));
        b - a
    };
    assert!(span(0) > 0.5 && span(1) > 0.5, "um dos eixos nao se mexeu");
    // ⛔⛔ **E OS DOIS SORTEIOS SÃO INDEPENDENTES** — com uma LANE só, a folha maior seria
    // sempre a mais empurrada, e isso lê-se como um padrão, não como acaso.
    let ambos = LeafLook {
        size: 1.0,
        size_jitter: 1.0,
        pos_jitter: 1.0,
        ..neutro
    };
    let (xs, ys): (Vec<f32>, Vec<f32>) = (0..256)
        .map(|i| (ambos.at(i as u32).0, ambos.at(i as u32).1[0]))
        .unzip();
    let (mx_, my) = (
        xs.iter().sum::<f32>() / xs.len() as f32,
        ys.iter().sum::<f32>() / ys.len() as f32,
    );
    let cov: f32 = xs
        .iter()
        .zip(&ys)
        .map(|(a, b)| (a - mx_) * (b - my))
        .sum::<f32>()
        / xs.len() as f32;
    let (sx, sy) = (
        (xs.iter().map(|a| (a - mx_).powi(2)).sum::<f32>() / xs.len() as f32).sqrt(),
        (ys.iter().map(|b| (b - my).powi(2)).sum::<f32>() / ys.len() as f32).sqrt(),
    );
    let r = cov / (sx * sy);
    assert!(
        r.abs() < 0.25,
        "o tamanho e o empurrao estao correlacionados (r = {r:.3}) — uma LANE so'"
    );
}
/// ⛔⛔ **E A MEMBRANA TEM DE OS CONSUMIR** — a lei acima é pura, e uma lei que ninguém aplica
/// passa em todos os gates dela.
///
/// Medido: a mutação que apagava o `scale` do tamanho da linha **SOBREVIVEU** à suíte inteira.
/// *Gatear a lei e não o consumidor é gatear metade.*
#[test]
fn the_membrane_applies_the_size_and_the_jitters() {
    let publicar = |size: f32, size_jitter: f32, pos_jitter: f32| -> (Vec<f32>, Vec<[f32; 2]>) {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state.doc.graph.set_param(n, ls::param::LEAF_SIZE, size);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_SIZE_JITTER, size_jitter);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_POS_JITTER, pos_jitter);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let inst = instances_of(&state, &key);
        (
            inst.iter().map(|i| i.size[0]).collect(),
            inst.iter().map(|i| i.world_pos).collect(),
        )
    };
    let (base, pos_base) = publicar(1.0, 0.0, 0.0);
    assert!(base.len() > 8, "so' {} folhas", base.len());
    // 1. O `Leaf Size` chega à instância.
    let (dobro, _) = publicar(2.0, 0.0, 0.0);
    for (a, b) in base.iter().zip(&dobro) {
        assert!(
            (b - a * 2.0).abs() < 1e-4,
            "o Leaf Size nao chegou: {a} -> {b}"
        );
    }
    // 2. O `Size Jitter` faz as folhas diferirem UMAS DAS OUTRAS.
    let (variado, _) = publicar(1.0, 0.6, 0.0);
    let distintos = {
        let mut v: Vec<i64> = variado.iter().map(|s| (s * 1e4) as i64).collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    };
    assert!(
        distintos > base.len() / 2,
        "so' {distintos} tamanhos distintos em {} folhas — o jitter nao chegou",
        variado.len()
    );
    // 3. O `Position Jitter` desencosta a folha da âncora, sem a perder de vista.
    let (_, pos) = publicar(1.0, 0.0, 1.0);
    let mut mexidas = 0;
    for (a, b) in pos_base.iter().zip(&pos) {
        let d = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        if d > 1e-5 {
            mexidas += 1;
        }
        // A folha publicada mede `2 × 3`; meia folha é `1` no maior eixo.
        assert!(d < 2.0, "a folha fugiu do ramo: {d}");
    }
    assert!(
        mexidas * 4 >= pos_base.len() * 3,
        "so' {mexidas} de {} folhas se mexeram",
        pos_base.len()
    );
}

/// ⛔⛔⛔ **UMA FOLHA QUE JÁ EXISTIA NÃO MUDA DE TAMANHO QUANDO A PLANTA CRESCE** — report do
/// Enio (2026-08-30): *"nem todas as folhas crescem, algumas aparecem já grandes"*.
///
/// A causa era a IDENTIDADE: o sorteio saía do **índice da âncora na lista**, e ao crescer a
/// planta insere marcas **no meio** (a travessia é em profundidade) ⇒ o índice de uma folha
/// antiga muda, ela recebe outro número, e salta de tamanho. O mesmo valia para o lado
/// (frente/trás): as folhas trocavam de lado sozinhas enquanto a planta crescia.
///
/// ⚠️ **A régua identifica a folha pelo que NÃO muda** — a posição dela, para as gerações que
/// já pararam de crescer. *A 1.ª sonda desta caça usou a posição para TODAS as folhas e
/// inventou 209 saltos de 420: a marca da geração mais nova move-se enquanto o ramo estica.*
#[test]
fn a_leaf_that_already_existed_keeps_its_size_as_the_plant_grows() {
    let colher = |gens: f32| -> Vec<([i64; 2], i64)> {
        let (mut state, n) = factory_plant_with_leaf(gens, false);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_SIZE_JITTER, 0.8);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_POS_JITTER, 0.0);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        instances_of(&state, &key)
            .iter()
            .map(|i| {
                (
                    [(i.world_pos[0] * 1e3) as i64, (i.world_pos[1] * 1e3) as i64],
                    (i.size[0] * 1e3) as i64,
                )
            })
            .collect()
    };
    // As gerações 3 e 4 já pararam de crescer em `g = 5`, então as folhas delas estão no
    // mesmo sítio nos dois retratos — e é por elas que se compara.
    let antes: std::collections::BTreeMap<[i64; 2], i64> = colher(4.0).into_iter().collect();
    let depois: std::collections::BTreeMap<[i64; 2], i64> = colher(5.0).into_iter().collect();
    let mut comuns = 0;
    for (p, tamanho) in &antes {
        let Some(agora) = depois.get(p) else { continue };
        comuns += 1;
        assert_eq!(
            tamanho, agora,
            "a folha em {p:?} mudou de tamanho quando a planta cresceu — o sorteio voltou a \
             sair do INDICE"
        );
    }
    // ⚠️ **O CONTROLE**: sem folhas em comum a comparação acima passa vazia.
    assert!(
        comuns >= 8,
        "so' {comuns} folhas em comum entre as duas gerações"
    );
}

/// ⛔⛔⛔ **RE-SORTEAR A SEMENTE MEXE AS FOLHAS** — o alcance que uma isenção já invocava e que
/// não existia.
///
/// # O defeito
///
/// Auditoria de seis lentes, doc 96 §4.4. Os dois sorteios de folha e o lado (frente/trás) saíam
/// só de `hash01_lane(marca, lane)`, onde a marca é a IDENTIDADE `(geração, ordinal)`. ⇒ o botão
/// *re-roll* do `Seed` e o número dele **não mudavam uma folha**: os três sorteios eram
/// irreroláveis, e o `Leaf Spread` (que a tartaruga semeia) era o único que respondia.
///
/// ⚠️⚠️ **E a isenção do `Seed` no censo do nó invocava exactamente este alcance:** *«ele é
/// também semeado pelo `Leaf Size Jitter` e pelo `Leaf Pos Jitter`, que a SHELL lê»*. Era essa
/// frase que o mantinha visível apesar de dormir em 8 dos 9 moldes — **e ela não tinha leitor**.
/// *Uma promessa que justifica uma decisão tem de ter quem a leia.*
///
/// # A régua
///
/// Duas sementes, tudo o resto igual, pela porta do produto — e as três respostas têm de mexer.
/// ⚠️ **A identidade da marca continua a mandar dentro de UMA semente** (é o que faz uma folha
/// que já existia não saltar de tamanho quando a planta cresce, o gate irmão acima); o que a
/// semente muda é o SORTEIO INTEIRO, de uma vez.
#[test]
fn re_rolling_the_seed_moves_the_leaves() {
    let publicar = |seed: f32| -> (Vec<f32>, Vec<[f32; 2]>, Vec<f32>) {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state.doc.graph.set_param(n, ls::param::SEED, seed);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_SIZE_JITTER, 0.8);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_POS_JITTER, 0.8);
        // ⚠️ **`Leaves In Front` fica em ZERO de propósito.** Acima de zero a TERCEIRA MÉDIA
        // manda a copa inteira para o passe do vector, e o `instances_of` — que lê o passe das
        // sprites — devolve **zero folhas**. A 1.ª redacção deste gate punha `0,5` e media uma
        // lista vazia: *uma fixtura que não contém o fenómeno passa a acusar a régua.*
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let inst = instances_of(&state, &key);
        (
            inst.iter().map(|i| i.size[0]).collect(),
            inst.iter().map(|i| i.world_pos).collect(),
            inst.iter().map(|i| i.world_pos[1]).collect(),
        )
    };
    let (t1, p1, o1) = publicar(1.0);
    let (t2, p2, o2) = publicar(7.0);
    assert!(t1.len() > 8, "só {} folhas — fixtura fraca", t1.len());
    assert_eq!(t1.len(), t2.len(), "a semente não pode mudar a CONTAGEM");

    let difere = |a: &[f32], b: &[f32]| {
        a.iter()
            .zip(b)
            .filter(|(x, y)| (*x - *y).abs() > 1e-5)
            .count()
    };
    assert!(
        difere(&t1, &t2) * 2 >= t1.len(),
        "o `Size Jitter` não seguiu a semente: só {} de {} folhas mudaram de tamanho",
        difere(&t1, &t2),
        t1.len()
    );
    let mexeu = p1
        .iter()
        .zip(&p2)
        .filter(|(a, b)| ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt() > 1e-5)
        .count();
    assert!(
        mexeu * 2 >= p1.len(),
        "o `Position Jitter` não seguiu a semente: só {mexeu} de {} folhas se mexeram",
        p1.len()
    );
    assert!(
        difere(&o1, &o2) > 0,
        "nada na disposição mudou com a semente"
    );

    // ⚠️ E a metade oposta: a MESMA semente reproduz ao bit. Sem ela, «mexeu» seria satisfeito
    // por não-determinismo, que é o defeito oposto e igualmente mau.
    let (t1b, p1b, _) = publicar(1.0);
    assert_eq!(
        t1.iter().map(|f| f.to_bits()).collect::<Vec<_>>(),
        t1b.iter().map(|f| f.to_bits()).collect::<Vec<_>>(),
        "a mesma semente tem de reproduzir os tamanhos AO BIT"
    );
    assert_eq!(p1, p1b, "e as posições também");
}

/// ⛔⛔ **E O LADO (frente/trás) TAMBÉM SEGUE A SEMENTE** — a terceira resposta, que o gate
/// irmão não podia medir.
///
/// ⚠️ Ele mantém `Leaves In Front` em **zero** de propósito (acima disso a copa muda de passe e
/// o leitor de sprites devolve lista vazia), e por isso a mutação que tira a semente do sorteio
/// do LADO **sobrevivia lá**. A pergunta pede a fracção a meio e o leitor do passe vectorial.
///
/// ⚠️ **A régua é QUAIS folhas ficam à frente, não QUANTAS:** a fracção é a mesma nas duas
/// sementes por construção, então contar não distinguiria nada. O que muda é a partição.
#[test]
fn re_rolling_the_seed_changes_which_leaves_go_in_front() {
    let particao = |seed: f32| -> Vec<bool> {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state.doc.graph.set_param(n, ls::param::SEED, seed);
        state.doc.graph.set_param(n, ls::param::LEAF_FRONT, 0.5);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        // A planta é a única linha com geometria; as folhas são as outras, e a posição delas
        // relativamente a ela é o LADO.
        let v = vector_instances_of(&state, &key);
        let planta = v
            .iter()
            .position(|i| i.geometry_id > 0)
            .expect("a planta tem de estar no passe vectorial");
        v.iter()
            .enumerate()
            .filter(|(_, i)| i.geometry_id == 0)
            .map(|(k, _)| k > planta)
            .collect()
    };
    let a = particao(1.0);
    let b = particao(7.0);
    assert!(a.len() > 8, "só {} folhas — fixtura fraca", a.len());
    assert_eq!(a.len(), b.len(), "a semente não pode mudar a contagem");
    let a_frente = a.iter().filter(|x| **x).count();
    assert!(
        a_frente > 0 && a_frente < a.len(),
        "a fracção 0,5 tem de partir a copa em dois ({a_frente} de {})",
        a.len()
    );
    let trocaram = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    assert!(
        trocaram > 0,
        "nenhuma folha trocou de lado com outra semente — o sorteio do LADO não é semeado"
    );
    assert_eq!(a, particao(1.0), "e a mesma semente reproduz a partição");
}
