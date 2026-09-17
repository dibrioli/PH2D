//! ⭐⭐⭐ **O COLISOR DECLARADO ATRAVESSA UM DUPLICADOR?** — a ordem do dono de 2026-09-17:
//! *«O botão Collide da Shape deve funcionar para todo e qualquer duplicador»* (doc 114 §12).
//!
//! ⚠️⚠️ **Esta sonda mede o PRIMEIRO elo, e ele decide o tamanho do trabalho inteiro.** O
//! `source.shape` DECLARA o colisor em três colunas (`ph2d_collider` · `ph2d_collider_box` ·
//! `ph2d_collider_offset`); se um duplicador as deixar cair, nenhum separador a jusante as pode
//! ler e a ordem custa uma wave por duplicador. Se elas sobreviverem, o que falta é **um leitor**.
//!
//! ⛔ *Um censo de «quem escreve» e um de «quem lê» não respondem a isto* — a pergunta é sobre o
//! que acontece ao CAMINHO no meio, e só cozer o mede.
//!
//! ⛔⛔ **E o `source.shape` NÃO COZE sem as membranas do produto.** Ele é uma das cinco fontes que
//! LEEM um external que a shell publica (a geometria vectorial viva), e um arnês que coza sem
//! publicar recebe **zero peças e zero colunas** — que se lê exactamente como *«o duplicador
//! deitou o colisor fora»*. ⭐ *Foi o CONTROLO desta sonda que o apanhou*, e é por isso que ele é a
//! primeira coisa que ela mede: a fonte sozinha, antes de qualquer duplicador.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib colisor_duplicador -- --ignored --nocapture
//! ```

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, Stream,
};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// As três colunas que o `source.shape` escreve quando o `Collide` está ligado.
const COLUNAS: &[&str] = &[COLLIDER_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN];

/// O param de contagem que um duplicador possa ter — tentado por nome, e ausente não é erro.
///
/// ⚠️ **`motion.mirror` duplica SEM contagem** (ele espelha), logo um censo que exigisse o param
/// deixaria-o de fora e a resposta a *«todo e qualquer duplicador»* teria um buraco.
const CONTAGEM: &[&str] = &["count", "copies"];

fn no(g: &mut Graph, tipo: &str, x: f32) -> NodeId {
    let n = g.add_node(tipo.to_string());
    g.set_pos(n, Pos { x, y: 0.0 });
    n
}

fn liga(g: &mut Graph, de: NodeId, para: (NodeId, u16)) {
    g.connect(Edge {
        from: (de, 0),
        to: para,
        delayed: false,
    })
    .expect("fio");
}

/// A forma com o `Collide` LIGADO — a fonte de todas as cadeias desta sonda.
fn forma_com_colisor(g: &mut Graph) -> NodeId {
    let f = no(g, "source.shape", -400.0);
    g.set_param(f, ph2d_node_motion_shape::param::COLLIDE, 1.0);
    f
}

fn quais_colunas(s: &Stream) -> Vec<&'static str> {
    COLUNAS
        .iter()
        .copied()
        .filter(|c| s.get(c).is_some())
        .collect()
}

/// Coze `montar` no documento de um `MotionState` NOVO, **com as membranas publicadas**.
///
/// ⚠️ **As membranas entram pela porta do PRODUTO** (`motion_externals::publish_all`), e não por um
/// atalho: é ela que põe a geometria da forma ao alcance do `source.shape`.
fn coze(montar: impl FnOnce(&mut Graph) -> NodeId) -> Option<Stream> {
    let mut m = MotionState::new();
    let sink = montar(&mut m.doc.graph);
    crate::motion_externals::publish_all(&mut m, 0.0);
    // ⛔⛔ **O cozedor tem de ser o do PUMP, e não um `Cook::new()`.** As membranas são publicadas
    // no canal de externals DAQUELE cozedor; um cozedor novo nasce sem nenhuma, e o `source.shape`
    // lê a chave dele, não acha nada e devolve **stream vazio, sem erro** (está escrito no `eval`
    // dele: *«a key with no published shape … is the empty external»*). ⚠️ *Foi a SEGUNDA vez nesta
    // investigação que o arnês se leu como uma capacidade ausente* — e as duas vezes quem o apanhou
    // foi o controlo, nunca a leitura do código.
    m.pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, 0.0)
        .ok()
        .map(|v| v[0].as_stream().clone())
}

/// `forma(Collide) → <nó> → output`, cozida com as membranas. `None` quando a cadeia não monta
/// (o nó não aceita uma corrente de instâncias na porta 0, ou o grafo não valida).
fn cadeia(tipo: &str) -> Option<Stream> {
    coze(|g| {
        let f = forma_com_colisor(g);
        let d = no(g, tipo, 0.0);
        // A contagem, se o nó tiver uma: é ela que o faz DUPLICAR em vez de passar adiante.
        for c in CONTAGEM {
            g.set_param(d, *c, 6.0);
        }
        let o = no(g, "motion.output", 400.0);
        liga(g, f, (d, 0));
        liga(g, d, (o, 0));
        o
    })
}

/// ⭐⭐⭐ **O CENSO DERIVADO — todo nó do catálogo que MULTIPLICA, e o que ele faz ao colisor.**
///
/// ⛔⛔ **A população sai do REGISTO, nunca de uma lista escrita à mão.** A ordem do dono é *«todo
/// e qualquer duplicador»*, e uma lista de três nomes responde por três: o que decide se um nó é
/// duplicador é o que ele FAZ — recebe uma peça e devolve mais do que uma.
#[test]
#[ignore = "sonda de investigação — corra à mão"]
fn colisor_duplicador_o_censo_derivado() {
    // O CONTROLO, outra vez: a forma sozinha declara o colisor.
    let base = coze(|g| {
        let f = forma_com_colisor(g);
        let o = no(g, "motion.output", 400.0);
        liga(g, f, (o, 0));
        o
    })
    .expect("a forma coze");
    assert_eq!(base.count(), 1, "a forma e' UMA peca");
    assert!(
        !quais_colunas(&base).is_empty(),
        "a fonte nao declarou colisor — o censo mediria o nada"
    );

    let m = MotionState::new();
    let mut nomes: Vec<&str> = m.registry.manifests().map(|x| x.name).collect();
    nomes.sort_unstable();

    let mut multiplicam: Vec<(&str, usize, bool, bool)> = Vec::new();
    let mut varridos = 0usize;
    let mut fontes = 0usize;
    for tipo in nomes {
        // ⛔⛔ **UMA FONTE NÃO É UM DUPLICADOR, e o `connect` não o diz.** Ligar um fio a uma porta
        // que não existe **não falha** — o grafo aceita a aresta, o cozedor ignora-a, e o nó emite
        // a nuvem dele a partir dos params. A 1.ª redacção deste censo contava assim o
        // `motion.scatter`, o `motion.grid` e mais onze, e imprimia *«13 duplicadores perdem o
        // colisor»* sobre nós que **nunca receberam nada**. ⚠️ *Foi a TERCEIRA vez nesta
        // investigação que o arnês se leu como um defeito de produto.*
        let entradas = m
            .registry
            .manifests()
            .find(|x| x.name == tipo)
            .map_or(0, |x| x.inputs.len());
        if entradas == 0 {
            fontes += 1;
            continue;
        }
        let Some(s) = cadeia(tipo) else { continue };
        varridos += 1;
        if s.count() > base.count() {
            let sobrevive = ph2d_contact::colisores(&s)
                .is_some_and(|v| v.iter().filter(|c| c.is_some()).count() == s.count());
            // ⭐⭐⭐ **O DISCRIMINADOR: o `geometry_id` é a testemunha.** Ele é a coluna que diz
            // *«esta peça é a forma que entrou»*. Se ele sobrevive e o colisor não, o nó **deitou
            // o colisor fora** — é um defeito. Se nenhum dos dois sobrevive, o nó **não é um
            // duplicador da forma**: ele gera uma nuvem NOVA a partir dos params dele e a corrente
            // que entra serve outra coisa. ⚠️ *As duas leituras dão a mesma linha numa tabela de
            // «o colisor não chegou», e as curas são OPOSTAS.*
            let herda = s.get("geometry_id").is_some();
            multiplicam.push((tipo, s.count(), sobrevive, herda));
        }
    }

    eprintln!("\n  ═══ CENSO DERIVADO: quem MULTIPLICA, e o colisor sobrevive? ═══\n");
    eprintln!(
        "  {fontes} FONTES saltadas (não têm porta de entrada — não são duplicadores).\n  \
         Varridos {varridos} nós que ACEITAM a corrente da forma; {} multiplicam-na.\n",
        multiplicam.len()
    );
    eprintln!(
        "  {:<26} │ {:>6} │ {:>9} │ o colisor declarado",
        "duplicador", "peças", "é a forma"
    );
    eprintln!("  ---------------------------|--------|-----------|--------------------");
    for (tipo, n, ok, herda) in &multiplicam {
        eprintln!(
            "  {tipo:<26} │ {n:>6} │ {:>9} │ {}",
            if *herda { "sim" } else { "NAO" },
            if *ok {
                "✅ chega a TODAS"
            } else if *herda {
                "⛔ DEITADO FORA"
            } else {
                "— nuvem NOVA"
            }
        );
    }
    let deitam_fora: Vec<&str> = multiplicam
        .iter()
        .filter(|(_, _, ok, herda)| !ok && *herda)
        .map(|(t, _, _, _)| *t)
        .collect();
    let nuvem_nova: Vec<&str> = multiplicam
        .iter()
        .filter(|(_, _, _, herda)| !herda)
        .map(|(t, _, _, _)| *t)
        .collect();
    eprintln!(
        "\n  ⛔ DEITAM FORA um colisor que receberam ({}): {deitam_fora:?}",
        deitam_fora.len()
    );
    eprintln!(
        "  — geram nuvem NOVA, nunca receberam forma ({}): {nuvem_nova:?}\n",
        nuvem_nova.len()
    );
    eprintln!(
        "  ⚠️ As duas listas leem-se iguais numa tabela de «o colisor não chegou», e as CURAS são\n  \
         opostas: a primeira é um defeito de passagem; a segunda é a pergunta de produto *«uma\n  \
         peça que este nó INVENTA deve herdar o colisor da forma que entrou?»*\n"
    );
    assert!(
        multiplicam.len() >= 3,
        "o censo achou {} duplicadores — piso de populacao",
        multiplicam.len()
    );
}

/// `forma(Collide) → <duplicador> → output`, cozida com as membranas./// ⭐⭐ **O que a forma DECLARA quando a caixa dela não é um quadrado.**
///
/// ⚠️ **A pergunta não é «o `motion.collide` separa?» — é «ele separa pelo colisor DECLARADO?».**
/// Ele separa por um `radius` que é param DELE, igual para toda peça: uma caixa comprida declarada
/// pela forma seria tratada como um disco do tamanho que o artista escreveu noutro cartão.
#[test]
#[ignore = "sonda de investigação — corra à mão"]
fn colisor_duplicador_o_que_a_forma_declara() {
    let s = coze(|g| {
        let f = forma_com_colisor(g);
        // Uma caixa DELIBERADAMENTE comprida: um disco nunca a descreve.
        g.set_param(f, ph2d_node_motion_shape::param::COLLIDER_WIDTH, 4.0);
        let o = no(g, "motion.output", 400.0);
        liga(g, f, (o, 0));
        o
    })
    .expect("coze");
    let c = ph2d_contact::colisores(&s).and_then(|v| v.first().copied().flatten());
    eprintln!("\n  com `Collider Width = 4`, a forma declara:\n    {c:?}\n");
}

/// ⭐⭐⭐ **O que o botão MUDA, medido nos dois lados** — a sonda que escreveu o gate.
#[test]
#[ignore = "sonda de investigação — corra à mão"]
fn colisor_duplicador_o_botao_ligado_e_desligado() {
    for ligado in [false, true] {
        let s = coze(|g| {
            let f = no(g, "source.shape", -400.0);
            g.set_param(
                f,
                ph2d_node_motion_shape::param::COLLIDE,
                if ligado { 1.0 } else { 0.0 },
            );
            g.set_param(f, ph2d_node_motion_shape::param::COLLIDER_WIDTH, 2.0);
            let c = no(g, "motion.clone", -200.0);
            g.set_param(c, "count", 5.0);
            g.set_param(c, "distance", 0.0);
            let sep = no(g, "motion.collide", 0.0);
            let o = no(g, "motion.output", 400.0);
            liga(g, f, (c, 0));
            liga(g, c, (sep, 0));
            liga(g, sep, (o, 0));
            o
        })
        .expect("coze");
        let p = match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        eprintln!(
            "\n  Collide = {}: {:?}",
            if ligado { "ON " } else { "OFF" },
            p.iter()
                .map(|q| format!("({:.2},{:.2})", q[0], q[1]))
                .collect::<Vec<_>>()
        );
    }
    eprintln!();
}

/// ⭐⭐⭐ **O BOTÃO `Collide` DA FORMA MUDA A ARRUMAÇÃO DAS CÓPIAS — pela rota do PRODUTO.**
///
/// A ordem do dono (2026-09-17) é *«o botão Collide da Shape deve funcionar para todo e qualquer
/// duplicador»*. Este é o gate que o afirma, na cadeia que o artista escreve:
/// `source.shape(Collide) → motion.clone(distance 0) → motion.collide`, cozida com as membranas.
///
/// ⛔⛔ **E o que ele afirma NÃO é «as cópias separam-se» — esse era o gate errado, e o CONTROLO
/// apanhou-o.** Com um cartão `Collide` na cadeia as peças separam-se de qualquer maneira: é o
/// trabalho dele. O que o botão da FORMA muda é **por que colisor**, e isso vê-se na ARRUMAÇÃO:
///
/// - **desligado** — um disco não tem orientação, e as cinco espalham-se na DIAGONAL
///   (medido: `(−0,64,−0,64) … (0,64,0,64)`).
/// - **ligado** — a caixa declarada é larga e baixa, e as cinco empilham-se numa COLUNA, todas no
///   mesmo `x` (medido: `(0,00,−1,39) … (0,00,1,39)`). *Caixas arrumam-se como caixas.*
#[test]
fn o_botao_collide_da_forma_separa_as_copias() {
    let arrumacao = |ligado: bool| {
        let s = coze(|g| {
            let f = no(g, "source.shape", -400.0);
            g.set_param(
                f,
                ph2d_node_motion_shape::param::COLLIDE,
                if ligado { 1.0 } else { 0.0 },
            );
            // Uma caixa DELIBERADAMENTE larga: é o que um disco não sabe descrever.
            g.set_param(f, ph2d_node_motion_shape::param::COLLIDER_WIDTH, 2.0);
            let c = no(g, "motion.clone", -200.0);
            g.set_param(c, "count", 5.0);
            // ⚠️ **`distance = 0`: as cinco cópias nascem EMPILHADAS.** Um clone já espalhado não
            // tem nada para o colisor arrumar.
            // ⛔ A 1.ª redacção escreveu `step_x`/`step_y`, que **não existem** neste cartão — e um
            // `set_param` com um nome que o nó não tem **não falha**: ele fica guardado, ninguém o
            // lê, e o arranjo ficou o de fábrica (`distance = 2`). *Foi o controlo que o apanhou.*
            g.set_param(c, "distance", 0.0);
            let sep = no(g, "motion.collide", 0.0);
            let o = no(g, "motion.output", 400.0);
            liga(g, f, (c, 0));
            liga(g, c, (sep, 0));
            liga(g, sep, (o, 0));
            o
        })
        .expect("a cadeia coze");
        assert_eq!(s.count(), 5, "o clone tem de dar CINCO copias");
        match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        }
    };

    let espalha_x = |p: &[[f32; 2]]| {
        let (lo, hi) = p
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q[0]), b.max(q[0])));
        hi - lo
    };

    let desligado = arrumacao(false);
    let ligado = arrumacao(true);

    // ⚠️ **O CONTROLO é metade do gate:** com o disco as cópias TÊM de espalhar em `x`. Sem isso,
    // «o ligado alinhou» não separa o botão a funcionar de o clone já as ter posto em coluna.
    assert!(
        espalha_x(&desligado) > 0.5,
        "o CONTROLO (disco) tinha de espalhar em x, e espalhou {}",
        espalha_x(&desligado)
    );
    assert!(
        espalha_x(&ligado) < 1e-3,
        "com o botao LIGADO a caixa larga tinha de empilhar as copias numa COLUNA, \
         e elas espalharam {} em x",
        espalha_x(&ligado)
    );
    // E elas de facto se afastaram — cinco peças todas no mesmo sítio também têm `x` igual.
    let (lo, hi) = ligado
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q[1]), b.max(q[1])));
    assert!(
        hi - lo > 1.0,
        "as cinco caixas tinham de ocupar uma coluna ALTA, e ocuparam {}",
        hi - lo
    );
}
