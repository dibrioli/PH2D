//! ⭐⭐ **A AUDITORIA DO GRUPO DO RIG E DOS CORPOS MOLES** (ciclo 9, passo 2 — doc 103 §5, doc 114).
//!
//! *«Coisas que se seguram»*: os oito ciclos anteriores puseram coisas na tela, fizeram-nas andar,
//! dobraram-nas, escolheram quem é afectado, entregaram-nas a uma lei, deram-lhes um cérebro, uma
//! cara, e disseram de onde vêm. Este grupo responde à pergunta que sobra: **o que mantém as partes
//! de uma coisa PRESAS umas às outras** — um osso ao pai, uma pele ao osso, um nó da corda ao
//! seguinte, uma partícula do bando aos vizinhos.
//!
//! ⚠️ **O grupo tem DUAS METADES em categorias diferentes da paleta**, e isso é o grupo e não um
//! acidente de arrumação: cinco nós **produzem** a coisa que se segura (categoria `Source`) e cinco
//! **agem** sobre ela (categoria `Transform`).
//!
//! As sondas de auditoria são `#[ignore]` e correm-se à mão, pelo filtro `audit_the_rig_group`.

use ph2d_node_registry::NodeUiCategory;

/// ⚠️ **Os nós da categoria `Source` que têm dono NOUTRO ciclo** — é a subtracção que torna a
/// metade dos corpos moles **derivada** em vez de escrita à mão.
///
/// ⛔ Sem isto, a metade mole seria uma lista de quatro nomes, e *uma lista escrita à mão envelhece
/// em silêncio no dia em que um nó nasce* — que é exactamente a doença que o
/// [`crate::motion_ciclo_probe::familia`] existe para evitar do outro lado. Com a subtracção, um nó
/// `Source` NOVO que ninguém reclame **cai neste grupo**, obrigando alguém a dar-lhe dono.
///
/// ⚠️ A lista é **censada** por [`the_owners_elsewhere_are_still_sources`]: um nome que deixe de ser
/// `Source` sai daqui, senão a subtracção passa a tirar o que já não está lá.
const COM_DONO_NOUTRO_CICLO: &[(&str, &str)] = &[
    // Ciclo 1 — ARRANJO: pôr muitos objectos na tela.
    ("motion.grid", "ciclo 1 (arranjo)"),
    ("motion.scatter", "ciclo 1 (arranjo)"),
    ("motion.distribute_radial", "ciclo 1 (arranjo)"),
    ("motion.distribute_curve", "ciclo 1 (arranjo)"),
    ("motion.fibonacci", "ciclo 1 (arranjo)"),
    ("motion.lattice", "ciclo 1 (arranjo)"),
    ("motion.voronoi", "ciclo 1 (arranjo)"),
    // Ciclo 5 — SIMULAÇÃO.
    ("sim.spawn", "ciclo 5 (simulacao)"),
    // Ciclo 6 — VALOR & PULSO.
    ("value.pattern", "ciclo 6 (valor e pulso)"),
    // Ciclo 8 — FONTES & DADOS (o emissor; as `source.*` saem pelo prefixo).
    ("motion.emitter", "ciclo 8 (fontes e dados)"),
];

/// Todo tipo oferecido na categoria `Source` da paleta — a população de que a metade mole é o resto.
fn fontes_oferecidas() -> Vec<&'static str> {
    let m = crate::motion_state::MotionState::new();
    m.registry
        .manifests()
        .filter(|man| !m.registry.is_fixture(man.id))
        .filter(|man| {
            m.registry
                .ui_manifest(man.id)
                .is_some_and(|u| u.category == NodeUiCategory::Source)
        })
        .map(|man| man.name)
        .collect()
}

/// A metade que **PRODUZ** — categoria `Source`, menos as `source.*` e menos quem tem dono noutro
/// ciclo. Em 2026-09-17: `motion.boids` · `motion.soft_body` · `motion.verlet_rope` ·
/// `motion.wave` · `rig.skeleton`.
pub fn metade_que_produz() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = fontes_oferecidas()
        .into_iter()
        .filter(|n| !n.starts_with("source."))
        .filter(|n| !COM_DONO_NOUTRO_CICLO.iter().any(|(d, _)| d == n))
        .collect();
    v.sort_unstable();
    v
}

/// O grupo inteiro: a família `rig.*` (pelo registry, nunca à mão) unida à metade que produz.
///
/// ⚠️ `rig.skeleton` está nos dois lados — ele é o `Source` do rig —, e a união dedupe-o.
pub fn grupo() -> Vec<&'static str> {
    let mut v = crate::motion_ciclo_probe::familia("rig.");
    for n in metade_que_produz() {
        if !v.contains(&n) {
            v.push(n);
        }
    }
    v.sort_unstable();
    v
}

/// A metade que **AGE** — o grupo menos a metade que produz.
pub fn metade_que_age() -> Vec<&'static str> {
    let produz = metade_que_produz();
    grupo()
        .into_iter()
        .filter(|n| !produz.contains(n))
        .collect()
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_rig_group() {
    crate::motion_ciclo_probe::retrato(&grupo());
}

/// **OS PARAMS DE CADA NÓ** — o que a auditoria compara contra Rive · Spine · Blender · RubberHose.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_rig_node_offers() {
    crate::motion_ciclo_probe::params_de(&grupo());
}

/// **O QUE O CARTÃO PINTA**, com o rótulo da tela.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_rig_card_shows() {
    crate::motion_ciclo_probe::cartao(&grupo());
}

/// **OS NOMES dos cartões** — o que o tutorial terá de escrever.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_rig_card_names() {
    crate::motion_ciclo_probe::nomes(&grupo());
}

/// ⚠️ **O GRUPO ESTÁ VIVO** — piso de população e **as duas metades**.
///
/// ⛔ Sem o piso, uma categoria renomeada deixava as sondas acima a auditar **zero** nós, e todas
/// passariam caladas. E sem as metades asseguradas **em separado**, um filtro que só apanhasse os
/// `rig.*` leria `6` e outro que só apanhasse os moles leria `5` — e nenhum dos dois é o grupo.
#[test]
fn the_rig_group_is_derived_and_not_empty() {
    let g = grupo();
    assert!(
        g.len() >= 10,
        "so' {} no(s) no grupo do ciclo 9 (10 em 2026-09-17): {g:?}",
        g.len()
    );
    let produz = metade_que_produz();
    let age = metade_que_age();
    assert!(
        produz.len() >= 5,
        "a metade que PRODUZ encolheu (5 em 2026-09-17): {produz:?}"
    );
    assert!(
        age.len() >= 5,
        "a metade que AGE encolheu (5 em 2026-09-17): {age:?}"
    );
    assert!(
        produz.contains(&"rig.skeleton"),
        "o esqueleto e' o `Source` do rig e tem de estar na metade que produz: {produz:?}"
    );
    assert!(
        age.iter().all(|n| n.starts_with("rig.")),
        "a metade que AGE e' toda `rig.*` hoje; se isso mudou, o doc 114 §1 tem de o dizer: {age:?}"
    );
}

/// ⚠️ **O CENSO DE OBSOLESCÊNCIA da subtracção** — *uma catraca sem censo vira licença*
/// (`CLAUDE.md` §5.0).
///
/// ⛔ Um nome em [`COM_DONO_NOUTRO_CICLO`] que deixe de ser um `Source` oferecido passa a subtrair
/// **nada**, e a lista lê-se como se ainda estivesse a proteger o grupo. As duas leituras — *«o
/// dono mudou de categoria»* e *«o nó foi apagado»* — dão o mesmo byte aqui, e as duas exigem que
/// alguém volte a esta lista.
#[test]
fn the_owners_elsewhere_are_still_sources() {
    let fontes = fontes_oferecidas();
    assert!(
        fontes.len() >= 21,
        "piso da categoria Source (21 em 2026-09-17): {}",
        fontes.len()
    );
    for (dono, ciclo) in COM_DONO_NOUTRO_CICLO {
        assert!(
            fontes.contains(dono),
            "`{dono}` ja' nao e' um `Source` oferecido — a subtracao do {ciclo} deixou de subtrair \
             alguma coisa, e esta lista deixou de descrever o grupo (doc 114 §1)"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// A ROTA — o que o PLANEADOR faz com uma cadeia montada (ciclo 9, W0 — doc 114 §2).
// ---------------------------------------------------------------------------------------------

/// **Onde corre a cadeia deste nó** — `true` se o planeador a põe INTEIRA no dispositivo.
///
/// ⚠️⚠️ **A forma da cadeia é diferente para cada metade, e a escolha é o MÉTODO:**
///
/// - quem **PRODUZ** é o primeiro nó ⇒ `X → scale → output`, e um `X` sem kernel derruba tudo;
/// - quem **AGE** mede-se atrás de uma fonte que **JÁ ESTÁ no dispositivo** (`motion.grid`) ⇒
///   `grid → scale → X → output`.
///
/// ⛔ **A segunda metade NÃO pode ser medida atrás do `rig.skeleton`**, que seria a cadeia que o
/// artista escreve: ele próprio não tem kernel, logo a cadeia cai para a CPU **por causa da fonte**
/// e o nó medido nunca é a causa — *uma régua em que o sujeito não pode falhar sozinho não mede o
/// sujeito*. A pergunta aqui é a do planeador (*«este nó tem rota?»*), e ela não depende de as
/// colunas da entrada fazerem sentido para a lei dele.
fn cadeia_no_dispositivo(no: &str) -> bool {
    use ph2d_nodegraph::graph::{Edge, NodeId};
    let produz = metade_que_produz().contains(&no);
    let mut m = crate::motion_state::MotionState::new();
    let mut fios: Vec<(NodeId, NodeId)> = Vec::new();

    let x = m.doc.graph.add_node(no.to_string());
    let s = m.doc.graph.add_node("motion.scale".to_string());
    let o = m.doc.graph.add_node("motion.output".to_string());
    if produz {
        fios.push((x, s));
        fios.push((s, o));
    } else {
        let g = m.doc.graph.add_node("motion.grid".to_string());
        m.doc.graph.set_param(g, "rows", 320.0);
        m.doc.graph.set_param(g, "cols", 320.0);
        fios.push((g, s));
        fios.push((s, x));
        fios.push((x, o));
    }
    for (de, para) in fios {
        m.doc
            .graph
            .connect(Edge {
                from: (de, 0),
                to: (para, 0),
                delayed: false,
            })
            .expect("fio");
    }
    let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
    ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, o, &dirigidos).is_fully_gpu()
}

/// ⭐⭐⭐ **A CATRACA DA ROTA DO GRUPO** (ciclo 9, W0 — doc 114 §2).
///
/// ⚠️ **A posição dos nós na cadeia é o que faz este número doer mais do que o do ciclo 7:** lá o
/// nó acusado era o **último** do grafo; aqui cinco são o **primeiro** e cinco são do meio ⇒ *todo
/// grafo que segure seja o que for corre inteiro na CPU*.
///
/// ⚠️⚠️ **As DUAS metades:** um nó FORA da lista tem de ficar no dispositivo (a regressão que
/// voltaria em silêncio), e um nó DENTRO dela tem de continuar na CPU — senão a lista deixou de o
/// descrever, e *uma catraca sem censo de obsolescência vira licença* (`CLAUDE.md` §5.0). Quem puser
/// um destes no dispositivo **apaga a linha dele no mesmo commit**.
#[test]
fn the_rig_group_route_only_improves() {
    /// Os que levam a cadeia para a CPU, cada um com a razão — doc 114 §2.
    ///
    /// ⚠️ Esta lista nasce com **nove** de dez, que é o estado medido em 2026-09-17 e a razão de
    /// ser deste ciclo. Ela só encolhe.
    const NA_CPU: &[(&str, &str)] = &[
        ("motion.soft_body", "sem kernel — a W4 do doc 114 §5"),
        ("motion.verlet_rope", "sem kernel — a W4 do doc 114 §5"),
        ("motion.wave", "sem kernel — a W4 do doc 114 §5"),
        ("rig.skeleton", "a familia `rig.*` inteira nao tem kernel"),
        ("rig.fk", "a familia `rig.*` inteira nao tem kernel"),
        ("rig.ik_2bone", "a familia `rig.*` inteira nao tem kernel"),
        ("rig.fabrik", "a familia `rig.*` inteira nao tem kernel"),
        (
            "rig.rubber_hose",
            "a familia `rig.*` inteira nao tem kernel",
        ),
        (
            "rig.skin_deformer",
            "a familia `rig.*` inteira nao tem kernel",
        ),
    ];
    let g = grupo();
    assert!(g.len() >= 10, "piso de populacao: {g:?}");
    for (no, razao) in NA_CPU {
        assert!(
            g.contains(no),
            "`{no}` esta' na lista da CPU mas ja' nao e' do grupo do ciclo 9 — apague a linha"
        );
        assert!(
            !cadeia_no_dispositivo(no),
            "`{no}` ja' fica no dispositivo — apague a linha dele da NA_CPU («{razao}»)"
        );
    }
    for no in &g {
        if NA_CPU.iter().any(|(n, _)| n == no) {
            continue;
        }
        assert!(
            cadeia_no_dispositivo(no),
            "`{no}` leva a cadeia para a CPU e nao esta' nomeado na NA_CPU — doc 114 §2"
        );
    }
    // ⭐ O CONTROLO POSITIVO da régua: sem ele, um `cadeia_no_dispositivo` que devolvesse SEMPRE
    // `false` deixava as nove asserções acima verdes e a décima nunca corria — e a catraca lia-se
    // como a funcionar sobre um instrumento morto.
    assert!(
        cadeia_no_dispositivo("motion.boids"),
        "o controlo positivo caiu: se o `motion.boids` saiu do dispositivo, a regua tem de o dizer \
         antes de acusar os outros nove"
    );
}

// ---------------------------------------------------------------------------------------------
// A CANETA — §5.0: MEDIR se a composição já exprime o item ANTES de o construir.
// ---------------------------------------------------------------------------------------------

/// Quantas juntas a medição da caneta usa. Quatro chegam para haver TRÊS ossos com comprimentos
/// diferentes, que é o mínimo que distingue *«varia»* de *«dois valores»*.
const JUNTAS: f32 = 4.0;

/// **Os comprimentos que o `rig.fk` de facto RESOLVE**, medidos nas POSIÇÕES de saída.
///
/// A cadeia é `rig.skeleton → [motion.drive(Custom, "len") ← value.instance_field(Ramp)] → rig.fk`,
/// e o booleano tira o escritor do meio — *é ele o CONTROLO*.
///
/// ⚠️ Mede-se a GEOMETRIA e não a coluna: ler `len` de volta provaria que o `motion.drive` escreveu,
/// e a pergunta é se o **solver obedece**. São duas afirmações e só a segunda fecha a célula.
fn comprimentos_dos_ossos(com_a_caneta: bool) -> Vec<f32> {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::Edge;
    let mut m = crate::motion_state::MotionState::new();
    let esqueleto = m.doc.graph.add_node("rig.skeleton".to_string());
    m.doc.graph.set_param(esqueleto, "joints", JUNTAS);
    m.doc.graph.set_param(esqueleto, "length", 1.0);
    m.doc.graph.set_param(esqueleto, "angle", 0.0);
    m.doc.graph.set_param(esqueleto, "root_angle", 0.0);
    let fk = m.doc.graph.add_node("rig.fk".to_string());

    let fio = |g: &mut ph2d_nodegraph::graph::Graph, de, para, porta| {
        g.connect(Edge {
            from: (de, 0),
            to: (para, porta),
            delayed: false,
        })
        .expect("fio");
    };
    if com_a_caneta {
        let campo = m.doc.graph.add_node("value.instance_field".to_string());
        // `Ramp` dá `0..1` ao longo dos elementos — um número DIFERENTE por osso.
        m.doc.graph.set_param(campo, "mode", 1.0);
        // ⚠️ **A porta dele é lida só pela CONTAGEM, e desligada devolve UM valor degenerado**
        // (di-lo o manifesto). A 1.ª redacção desta sonda deixou-a solta e mediu `[0, 0, 0]`
        // contra um controlo de `[1, 1, 1]`: *a caneta escreveu, e escreveu ZEROS* — um defeito
        // do ARNÊS que se lê exactamente como uma capacidade ausente.
        fio(&mut m.doc.graph, esqueleto, campo, 0);
        let caneta = m.doc.graph.add_node("motion.drive".to_string());
        m.doc.graph.set_param(caneta, "channel", 9.0); // `Custom…`
        m.doc.graph.set_param(caneta, "mode", 1.0); // `Set`
        m.doc.graph.set_param(caneta, "scale", 1.0);
        m.doc
            .graph
            .set_text_param(caneta, "column", "len".to_string());
        fio(&mut m.doc.graph, esqueleto, caneta, 0);
        fio(&mut m.doc.graph, campo, caneta, 1);
        fio(&mut m.doc.graph, caneta, fk, 0);
    } else {
        fio(&mut m.doc.graph, esqueleto, fk, 0);
    }

    let mut cook = Cook::new();
    let s = cook.cook(&m.doc.graph, &m.registry, fk, 0.0).expect("coze")[0]
        .as_stream()
        .clone();
    let Some(Column::Vec2(p)) = s.get("P") else {
        return Vec::new();
    };
    p.windows(2)
        .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
        .collect()
}

/// ⭐⭐⭐ **A CANETA JÁ EXISTE — e é o `motion.drive` no canal `Custom…`.**
///
/// A folha [16_rig.md](../../../docs/Motion%20Nodes/89_conferencia/16_rig.md) §0 diz que o catálogo
/// sabe escrever **exactamente cinco** colunas (`X`/`Y`/`Rotation`/`Size`/`Opacity`) e que por isso
/// `parent` e `len` — *«as duas colunas que FAZEM de uma corrente um esqueleto»* — **não têm
/// escritor nenhum**, o que ela nomeia como a causa mecânica de SEIS células inexprimíveis.
///
/// ⛔⛔ **Isso era verdade em 2026-08-09 e já não é:** o `motion.drive` ganhou depois o canal
/// **`Custom…`** (`CH_CUSTOM = 9`) mais o text param `column`, e com eles escreve **qualquer**
/// coluna com os oito modos dele. Esta medição é o que separa *«a nota envelheceu»* de *«eu
/// acreditei nela»* — a lei do `CLAUDE.md` §5.0: *antes de construir um item de lista aberta, meça
/// se a composição já o exprime*.
///
/// ⚠️ **O CONTROLO é metade do valor:** sem a cadeia sem-escritor a devolver comprimentos IGUAIS,
/// «eles variam» não distingue a caneta de um esqueleto que já nascia irregular.
#[test]
fn a_caneta_que_a_folha_diz_nao_existir_ja_escreve_o_comprimento_do_osso() {
    let controlo = comprimentos_dos_ossos(false);
    let com = comprimentos_dos_ossos(true);
    assert!(
        controlo.len() >= 3 && com.len() == controlo.len(),
        "a cadeia nao produziu ossos: controlo {controlo:?} com {com:?}"
    );
    // O CONTROLO: sem escritor, a corrente tem UM comprimento para todos os ossos.
    let (lo, hi) = (
        controlo.iter().cloned().fold(f32::MAX, f32::min),
        controlo.iter().cloned().fold(f32::MIN, f32::max),
    );
    assert!(
        hi - lo < 1e-4,
        "o CONTROLO ja' tinha ossos diferentes ({controlo:?}) — a medicao nao distingue nada"
    );
    // E COM a caneta: eles deixam de ser todos iguais.
    let (clo, chi) = (
        com.iter().cloned().fold(f32::MAX, f32::min),
        com.iter().cloned().fold(f32::MIN, f32::max),
    );
    assert!(
        chi - clo > 0.1,
        "o `motion.drive(Custom, \"len\")` nao chegou ao solver: {com:?} (controlo {controlo:?})"
    );
}

/// A TABELA da caneta — os números que o doc 114 §3.1 cita.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn print_the_pen_measurement() {
    eprintln!(
        "\n  comprimentos dos ossos (`rig.skeleton(joints=4, length=1)` -> `rig.fk`)\n\
         \n  sem a caneta (CONTROLO) : {:?}\
         \n  com `drive(Custom,len)` : {:?}\n",
        comprimentos_dos_ossos(false),
        comprimentos_dos_ossos(true)
    );
}

// ---------------------------------------------------------------------------------------------
// A COSTURA — o envelope por osso chega ao BARRO pelo caminho do produto (ciclo 9, W1).
// ---------------------------------------------------------------------------------------------

/// As posições da pele depois de um esqueleto DOBRADO, com e sem o envelope por osso escrito
/// pela caneta genérica.
///
/// ```text
/// motion.grid ─────────────────────────────────────────────> skin.in
/// rig.skeleton ─[drive(Custom,"bone_weight") ← ramp]─> fk ──> skin.rest
///              └─[drive(Custom,"rot")        ← ramp]─> fk ──> skin.posed
/// ```
///
/// ⚠️ **A pose tem de DOBRAR e não rodar em bloco:** numa rotação rígida todo osso sofre a mesma
/// mudança de referencial e a pele sai no mesmo sítio **seja qual for o peso** — a `ph2d-boundary`
/// do skin di-lo por escrito, e a 1.ª redacção dos gates de unidade reprovou por isso. Aqui o `rot`
/// é conduzido por uma rampa, logo cada junta dobra um bocado diferente.
fn pele_com_envelope(envelope: Option<f32>) -> Vec<[f32; 2]> {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};
    let mut m = crate::motion_state::MotionState::new();
    let fio = |g: &mut Graph, de, para, porta| {
        g.connect(Edge {
            from: (de, 0),
            to: (para, porta),
            delayed: false,
        })
        .expect("fio");
    };
    /// Um `motion.drive` no canal `Custom…`, a escrever `coluna` com uma rampa por elemento.
    fn caneta(
        m: &mut crate::motion_state::MotionState,
        fonte: ph2d_nodegraph::graph::NodeId,
        coluna: &str,
        escala: f32,
    ) -> ph2d_nodegraph::graph::NodeId {
        let campo = m.doc.graph.add_node("value.instance_field".to_string());
        m.doc.graph.set_param(campo, "mode", 1.0); // `Ramp`
        let d = m.doc.graph.add_node("motion.drive".to_string());
        m.doc.graph.set_param(d, "channel", 9.0); // `Custom…`
        m.doc.graph.set_param(d, "mode", 1.0); // `Set`
        m.doc.graph.set_param(d, "scale", escala);
        m.doc.graph.set_text_param(d, "column", coluna.to_string());
        m.doc
            .graph
            .connect(Edge {
                from: (fonte, 0),
                to: (campo, 0),
                delayed: false,
            })
            .expect("fio");
        m.doc
            .graph
            .connect(Edge {
                from: (fonte, 0),
                to: (d, 0),
                delayed: false,
            })
            .expect("fio");
        m.doc
            .graph
            .connect(Edge {
                from: (campo, 0),
                to: (d, 1),
                delayed: false,
            })
            .expect("fio");
        d
    }

    let esqueleto = m.doc.graph.add_node("rig.skeleton".to_string());
    m.doc.graph.set_param(esqueleto, "joints", JUNTAS);
    m.doc.graph.set_param(esqueleto, "length", 1.0);
    m.doc.graph.set_param(esqueleto, "angle", 0.0);

    // O REPOUSO — com ou sem o envelope escrito por cima.
    let fonte_repouso = match envelope {
        Some(e) => caneta(&mut m, esqueleto, "bone_weight", e),
        None => esqueleto,
    };
    let fk_repouso = m.doc.graph.add_node("rig.fk".to_string());
    fio(&mut m.doc.graph, fonte_repouso, fk_repouso, 0);

    // A POSE — o mesmo esqueleto com o `rot` conduzido por uma rampa (dobra progressiva).
    let dobra = caneta(&mut m, esqueleto, "rot", 40.0);
    let fk_pose = m.doc.graph.add_node("rig.fk".to_string());
    fio(&mut m.doc.graph, dobra, fk_pose, 0);

    let grelha = m.doc.graph.add_node("motion.grid".to_string());
    m.doc.graph.set_param(grelha, "rows", 3.0);
    m.doc.graph.set_param(grelha, "cols", 3.0);

    let pele = m.doc.graph.add_node("rig.skin_deformer".to_string());
    fio(&mut m.doc.graph, grelha, pele, 0);
    fio(&mut m.doc.graph, fk_repouso, pele, 1);
    fio(&mut m.doc.graph, fk_pose, pele, 2);

    let mut cook = Cook::new();
    let s = cook
        .cook(&m.doc.graph, &m.registry, pele, 0.0)
        .expect("coze")[0]
        .as_stream()
        .clone();
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐⭐⭐ **O ENVELOPE POR OSSO CHEGA AO BARRO — a costura inteira, pelo caminho do produto.**
///
/// O `CLAUDE.md` §5.0 diz que *nenhum instrumento deste repo pergunta se o VALOR chega a um
/// consumidor*: os gates de unidade da `rig.skin_deformer` provam a LEI, e este prova a **rota** —
/// a caneta (`motion.drive` em `Custom…`) escreve `bone_weight`, ele atravessa um `rig.fk` e o
/// solver da pele obedece-lhe.
///
/// ⚠️ **O CONTROLO é a mesma cadeia SEM a caneta**, senão *«mudou»* não separa o envelope de um
/// grafo diferente.
#[test]
fn o_envelope_por_osso_chega_a_pele_pelo_caminho_do_produto() {
    let sem = pele_com_envelope(None);
    let com = pele_com_envelope(Some(1.0));
    assert!(
        sem.len() >= 9 && com.len() == sem.len(),
        "a cadeia nao produziu pele: sem {} com {}",
        sem.len(),
        com.len()
    );
    let maior = sem
        .iter()
        .zip(&com)
        .map(|(a, b)| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max);
    assert!(
        maior > 1e-3,
        "o `bone_weight` nao chegou ao solver da pele (maior desvio {maior:e})"
    );
}
