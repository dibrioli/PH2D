//! ⭐⭐ **A AUDITORIA DO GRUPO DO FOCO — os CAMPOS** (ciclo 4, passo 2 — doc 103 §5).
//!
//! *«Nem todos ao mesmo tempo»*: este grupo é o que decide **quem** um animador ou um
//! deformador afecta, e **quanto**. Um `motion.falloff` sem campo é um interruptor; com campo é
//! um pincel.
//!
//! ⚠️ A régua é a mesma dos ciclos 1–3 e vive numa porta só
//! ([`crate::motion_ciclo_probe`]) — o que muda entre ciclos é a lista de nós, nunca o
//! instrumento.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture audit_the_field_group
//! ```

/// Os sete do ciclo 4 (doc 103 §5).
pub(crate) const GRUPO: [&str; 7] = [
    "motion.falloff",
    "field.box",
    "field.radial_sweep",
    "field.index_range",
    "field.remap",
    "field.combine",
    "field.shape",
];

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_field_group() {
    crate::motion_ciclo_probe::retrato(&GRUPO);
}

/// **OS PARAMS DE CADA NÓ DO GRUPO, um a um** — o que a auditoria compara contra as
/// referências. ⚠️ Sem esta lista, «falta X» é um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_each_field_offers
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_field_offers() {
    crate::motion_ciclo_probe::params_de(&GRUPO);
}

/// ⭐⭐ **AS ROWS QUE O CARTÃO DE FACTO PINTA** — ver a porta.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_the_field_card_shows
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_field_card_shows() {
    crate::motion_ciclo_probe::cartao(&GRUPO);
}

/// ⭐⭐ **O VOCABULÁRIO do grupo** — dois campos que guardam a mesma pergunta chamam-lhe o mesmo
/// nome? É o achado §2.3 do ciclo 3 (*«seis vocabulários para onde é o centro»*) virado régua.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture the_field_vocabulary
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_field_vocabulary() {
    crate::motion_ciclo_probe::vocabulario(&GRUPO);
}

/// ⭐⭐⭐ **DOIS IRMÃOS QUE PARTILHAM UM PARAM PÕEM-NO NO MESMO SÍTIO** (ciclo 4, W2).
///
/// A população é **derivada**: os nós do grupo que declaram secções com o **mesmo
/// vocabulário** — o `field.box` e o `field.radial_sweep` falam `Placement`/`Falloff`, o
/// `field.remap` fala `Range`/`Output` e responde a outras perguntas, logo não entra na
/// comparação. ⚠️ Sem esse recorte o gate acusaria o `invert` (que no `remap` vive em `Range`)
/// e mandaria alinhar duas coisas que não são a mesma.
///
/// ⚠️⚠️ **«O mesmo sítio» inclui FICAR SOLTO, e a 1.ª redacção não o dizia.** Ela comparava só
/// os params que os dois **agrupavam** — e a mutação que devolvia o `soft` do `field.box` para
/// fora de toda secção **SOBREVIVEU**, porque um param solto simplesmente saía da população.
/// *Um censo que só olha o que foi declarado é cego a uma omissão*, e a omissão era exactamente
/// a divergência que esta wave veio curar. Hoje o «sítio» de um param partilhado é o título da
/// secção **ou `(solto)`**, e os dois têm de bater.
///
/// ⛔ **Ele já se pagou:** a 1.ª redacção da tabela do `field.box` deixava o `soft` solto, e o
/// irmão põe-no em `Falloff` — *quando o objectivo é alinhar dois irmãos, a autoridade é o
/// irmão*.
#[test]
fn the_two_spatial_boxes_group_a_shared_param_the_same_way() {
    use std::collections::BTreeMap;
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
    // nó -> (param -> sítio), só para quem fala o vocabulário ESPACIAL.
    let mut casas: Vec<(&str, BTreeMap<&str, &str>)> = Vec::new();
    for nome in GRUPO {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let grupos = reg.param_groups(tid);
        if !grupos.iter().any(|g| g.group == "Placement") {
            continue;
        }
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = reg.resolve(tid) else { continue };
            op.manifest()
        };
        let mut casa: BTreeMap<&str, &str> = BTreeMap::new();
        for spec in man.params {
            let onde = grupos
                .iter()
                .find(|g| g.param == spec.name)
                .map_or("(solto)", |g| g.group);
            casa.insert(spec.name, onde);
        }
        casas.push((nome, casa));
    }
    assert!(
        casas.len() >= 2,
        "so' {} no(s) com o vocabulario espacial -- o gate compara IRMAOS",
        casas.len()
    );
    let (n0, c0) = &casas[0];
    let mut partilhados = 0usize;
    for (n, c) in &casas[1..] {
        for (param, onde) in c {
            let Some(onde0) = c0.get(param) else { continue };
            partilhados += 1;
            assert_eq!(
                onde, onde0,
                "`{param}` vive em `{onde}` no {n} e em `{onde0}` no {n0} -- dois irmaos com o \
                 mesmo vocabulario te^m de o arrumar igual (e `(solto)` e' um sitio)"
            );
        }
    }
    // Piso contra o vácuo: dois irmãos sem param nenhum em comum não provariam nada.
    assert!(
        partilhados >= 4,
        "so' {partilhados} param(s) partilhado(s) -- a comparacao ficou vazia"
    );
}

/// ⭐⭐⭐ **O PREÇO DO GRUPO** (ciclo 4, passo 5 — doc 103 §1).
///
/// ⚠️ **Um campo é `Pure` e escreve a coluna `falloff`** — sozinho na cadeia ele não move um
/// pixel. A tabela mede-o **na cadeia do produto**, `grid → <campo> → output`, que é onde o
/// planeador decide se a coisa fica no dispositivo.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_field_group
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_field_group() {
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    crate::motion_ciclo_probe::tabela(&GRUPO, lado);
}

/// ⭐ **O DESPERTAR TEM DE ACORDAR** também neste grupo — ver a porta partilhada.
#[test]
fn waking_a_field_takes_it_off_the_identity() {
    let mudos = crate::motion_ciclo_probe::quem_o_despertar_nao_acorda(&GRUPO);
    assert!(
        mudos.is_empty(),
        "o despertar nao mexeu nestes: {mudos:?} -- ou o widget deles nao esta' na lista de \
         controlos continuos, ou a fraccao poe o no' de volta na identidade"
    );
}

/// ⭐⭐ **A CENA `=112` CONSTRÓI, É ALCANÇÁVEL E FICA NO DISPOSITIVO** (ciclo 4, passo 7).
///
/// ⚠️ **As três perguntas são independentes**, e a casa já pagou por confundi-las: um grafo que
/// coze não é um grafo que se VÊ (a 1.ª redacção da cena `=111` punha o pano a `300` unidades,
/// fora do alcance do zoom, e o report foi *«funciona nos nós mas não aparece no canvas»*), e um
/// grafo que se vê não é um grafo que o dispositivo reivindica.
#[test]
fn the_focus_scene_builds_and_stays_on_the_device() {
    let _trava = crate::motion_demo_legend::trava();
    let mut m = crate::motion_state::MotionState::new();
    let sinks = crate::motion_state::demo_router::build_level(Some("112"), &mut m.doc, &m.registry);
    let sink = *sinks.first().expect("a cena 112 tem um sink");

    // 1. Ela COZE, e produz as peças que promete.
    let out = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .expect("coze");
    let st = out[0].as_stream();
    assert_eq!(st.count(), 6400, "80 x 80 pecas");

    // 2. O campo MORDE — nem todas as peças têm o mesmo tamanho. ⚠️ É a régua da premissa do
    //    ciclo (*«nem todos ao mesmo tempo»*): sem ela a cena podia crescer o pano inteiro.
    let Some(ph2d_nodegraph::attr::Column::Vec2(size)) = st.get("size") else {
        panic!("a cena escreve `size` -- o `motion.scale` e' quem le^ o campo");
    };
    let (mut menor, mut maior) = (f32::MAX, f32::MIN);
    for s in size {
        menor = menor.min(s[0]);
        maior = maior.max(s[0]);
    }
    assert!(
        maior > menor * 1.5,
        "a mancha tem de se ver: menor {menor} maior {maior} -- se forem iguais, o campo nao \
         esta' a ser lido"
    );

    // 3. E a cadeia inteira é REIVINDICADA pelo dispositivo.
    assert!(
        ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink).is_fully_gpu(),
        "a cena do ciclo 4 tem de correr no dispositivo -- os sete campos menos um la' estao"
    );
}
