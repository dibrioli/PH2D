//! **QUEM CONSOME O `falloff`, E QUEM O DIAGNOSER CONSEGUE VER.**
//!
//! A folha 07 marca uma linha do `motion.strobe` como *"defeito de side-metadata,
//! não de param: o nó **lê `falloff`** e **não** chama
//! `register_couplings(Consumes("falloff"))`, enquanto `delay` e `step` chamam —
//! ~4 linhas"*.
//!
//! ⚠️ **A célula estava certa sobre o nó e ERRADA sobre o TAMANHO.** O canal do
//! ADR-0155 tem duas metades: um nó com kernel de GPU **DERIVA** o papel da
//! própria `ColumnBinding` (`access.reads()` e não é produtor), e um nó CPU-only
//! não tem de onde derivar — ele tem de DECLARAR. Então a pergunta honesta não é
//! *"quem esqueceu de declarar?"*, é ***quais leitores de `falloff` são
//! CPU-only?*** — e é essa lista que esta sonda mede.
//!
//! ⚠️ **E ela é um GATE, não uma sonda:** um nó CPU-only novo que leia `falloff`
//! sem declarar nasce VERMELHO aqui. A alternativa — uma lista escrita à mão de
//! quem precisa declarar — é a enumeração que apodrece na primeira crate-nó nova
//! [[feedback_a_condition_that_enumerates_its_readers_rots]].

use ph2d_node_registry::{Coupling, NodeRegistry};
use ph2d_nodegraph::gpu::ColumnAccess;
use ph2d_nodegraph::node::NodeTypeId;

/// A coluna de modulação que TODO comportamento multiplica.
const FALLOFF: &str = "falloff";

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// O nó DECLARA que consome a coluna?
fn declares(reg: &NodeRegistry, ty: NodeTypeId) -> bool {
    reg.couplings(ty).is_some_and(|cs| {
        cs.iter()
            .any(|c| matches!(c, Coupling::Consumes(col) if *col == FALLOFF))
    })
}

/// O diagnoser DERIVA que ele consome — uma binding de GPU que LÊ e não re-produz?
///
/// Espelha o `consumes` do `ph2d-motion-diagnose` (que é privado): binding que
/// `reads()` e cujo acesso não é de produtor.
fn derivable(reg: &NodeRegistry, ty: NodeTypeId) -> bool {
    use ph2d_nodegraph::gpu::KernelResolver;
    reg.gpu_kernel(ty).is_some_and(|k| {
        k.bindings.iter().any(|b| {
            b.column == FALLOFF
                && b.access.reads()
                && !matches!(b.access, ColumnAccess::ReadWrite | ColumnAccess::Write)
        })
    })
}

/// **TODO leitor CPU-only do `falloff` o DECLARA.**
///
/// ⚠️ O oráculo é o FONTE do nó, não uma lista: a crate cujo `src/` menciona a
/// string `"falloff"` num contexto de coluna é um leitor candidato, e se ela não
/// tem binding de GPU que a nomeie, então o único canal que resta é a declaração.
/// Um nó que a esqueça fica **invisível ao diagnóstico** — o `falloff` que ele
/// consome não conta como consumido, e um campo ligado só a ele é reportado como
/// inerte.
#[test]
fn every_cpu_only_falloff_reader_declares_it() {
    let reg = registry();
    let mut cpu_only_readers = Vec::new();
    let mut derived = Vec::new();
    let mut missing = Vec::new();

    for m in reg.manifests() {
        if !consumes_falloff_in_source(m.name) {
            continue;
        }
        if derivable(&reg, m.id) {
            derived.push(m.name);
        } else if declares(&reg, m.id) {
            cpu_only_readers.push(m.name);
        } else {
            missing.push(m.name);
        }
    }

    eprintln!(
        "falloff: {} derivados da binding · {} declarados (CPU-only) · {} SEM canal",
        derived.len(),
        cpu_only_readers.len(),
        missing.len()
    );
    assert!(
        missing.is_empty(),
        "estes leitores de `falloff` não têm binding nem declaração — o diagnoser \
         não os vê: {missing:?}"
    );
    // ⚠️ CONTROLE: sem esta metade a asserção acima passa por VÁCUO no dia em que
    // o scanner de fonte parar de casar (renomeio de arquivo, um `const` no lugar
    // do literal). Um gate cujo universo pode ficar vazio é um gate que se cala.
    assert!(
        derived.len() + cpu_only_readers.len() >= 15,
        "a varredura tem de ACHAR leitores: {} derivados + {} declarados",
        derived.len(),
        cpu_only_readers.len()
    );
}

/// O `src/` da crate-nó de `name` **CONSOME** a coluna `falloff` — lê sem
/// re-produzir?
///
/// ⚠️ **DUAS regras, e as duas são a definição que o diagnoser usa, não
/// heurística.**
///
/// **(1) LÊ:** o literal aparece numa linha que consulta um stream (`get(` ou um
/// helper `…_col(`). É isso que separa `stream.get("falloff")` de três coisas que
/// o literal também casa e que **não são leituras de máscara**: a lista de
/// transientes do `sim.zone` (`const TRANSIENTS = ["accel", "falloff", "hit"]` —
/// o que ele NÃO guarda no estado), o PARAM homônimo do `rig.skin_deformer`
/// (`ctx.param("falloff")`, cuja *Stiffness* se chama assim por acidente de
/// nascimento) e a tabela de canais do `value.attribute`, cujo caso é o
/// interessante — ele lê **a coluna que o ARTISTA nomeou**, então uma declaração
/// estática seria FALSA em sete dos oito canais; o vocabulário do `Coupling` não
/// tem forma condicional, e a ausência ali é correta.
///
/// **(2) NÃO RE-PRODUZ:** a crate não contém `set("falloff"`. Um re-produtor
/// (as cinco `field.*` e o `motion.falloff`, que COMPÕEM a máscara) **não é
/// consumidor** — é literalmente a frase do `consumes` do diagnoser, *"é isso
/// que mantém duas forças sem integrador inertes"*, e declará-lo faria a
/// primeira `field.*` da cadeia contar como consumida pela segunda.
fn consumes_falloff_in_source(name: &str) -> bool {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(format!("ph2d-node-{}", name.replace(['.', '_'], "-")));
    let Ok(entries) = std::fs::read_dir(dir.join("src")) else {
        return false;
    };
    let files: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .filter(|p| {
            let f = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            !f.ends_with("_tests.rs") && f != "tests.rs"
        })
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();

    let produces = files.iter().any(|s| s.contains("set(\"falloff\""));
    let inline = files.iter().any(|s| {
        s.lines().any(|l| {
            l.contains("\"falloff\"")
                && (l.contains("get(") || l.contains("_col("))
                && !l.trim_start().starts_with("//")
        })
    });
    (inline || reads_via_const(&files)) && !produces
}

/// A SEGUNDA grafia de uma leitura, e ela é a CONVENÇÃO deste módulo: o nome da
/// coluna numa `const` própria do leitor — *"soletrada LOCALMENTE por cada leitor
/// (como `P` / `accel`) em vez de acoplar as crates"*, que é o que o
/// `motion.collide`, o `motion.soft_body`, a `verlet_rope` e o `boids` fazem.
///
/// ⚠️ **Sem esta metade o scanner era CEGO a essa família inteira** — o literal
/// mora na linha do `const` e o `get(` na linha da leitura, então a regra
/// *"literal e `get(` na MESMA linha"* nunca casa, e o gate afirmava *"todo
/// leitor CPU-only declara"* sobre um universo do qual todo leitor que segue a
/// convenção estava fora. Achado quando o `motion.soft_body` passou a ler a
/// coluna e a mutação que apaga a declaração dele **SOBREVIVEU**.
///
/// A regra continua sendo a do diagnoser e não uma heurística mais frouxa: uma
/// `const` LIGADA ao literal, cujo identificador aparece numa CONSULTA a stream.
/// Ela não alcança os três falsos positivos que o irmão nomeia — a lista de
/// transientes do `sim.zone` é um ARRAY, o param do `rig.skin_deformer` sai de
/// `ctx.param`, e a tabela de canais do `value.attribute` não liga uma const ao
/// literal.
fn reads_via_const(files: &[String]) -> bool {
    let names: Vec<String> = files
        .iter()
        .flat_map(|s| s.lines())
        .filter(|l| !l.trim_start().starts_with("//"))
        .filter_map(|l| {
            let (head, tail) = l.split_once('=')?;
            if !tail.trim_start().starts_with("\"falloff\"") {
                return None;
            }
            let head = head.trim();
            let idx = head.find("const ")? + "const ".len();
            Some(head[idx..].split(':').next()?.trim().to_string())
        })
        .filter(|n| !n.is_empty())
        .collect();
    names.iter().any(|n| {
        files.iter().any(|s| {
            s.lines().any(|l| {
                l.contains(n.as_str())
                    && (l.contains("get(") || l.contains("_col("))
                    && !l.trim_start().starts_with("//")
            })
        })
    })
}
