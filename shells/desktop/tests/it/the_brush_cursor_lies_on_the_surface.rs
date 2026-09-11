//! **O CURSOR DEITA NA SUPERFÍCIE** (ADR-0150) — arch-gate sobre a fiação.
//!
//! Ordem do Enio (2026-08-17): *"o gizmo da tool deve ter a direção das normais
//! onde incide (a nossa o gizmo da tool permanece na direção da tela)"*.
//!
//! ⚠️ **Este gate é de FONTE porque o produto não é alcançável sem device:** o
//! `Sculpt3dScene::cursor_mark` é método de uma cena que exige um
//! `wgpu::Device`, então nenhum teste headless o chama. A GEOMETRIA — que é
//! onde mora tudo o que pode estar errado por conta própria — é função LIVRE e
//! tem quatro gates de comportamento em
//! `sculpt3d::cursor::tests`; o que sobra, e só isto, é *o cursor PERGUNTA a
//! ela?*.
//!
//! ⚠️ **Ele afirma a PROPRIEDADE, nunca um endereço:** a lição que esta casa já
//! pagou várias vezes é que um gate ancorado em distância de bytes ou em nome
//! de vizinho envelhece verde. E ele traz CONTROLE POSITIVO — sem ele, um
//! arquivo renomeado deixaria as asserções passando sobre varredura vazia.

use std::path::Path;

/// ⛔⛔ **UM CENSO DE FONTE TEM DE LER A FONTE COM O ESPAÇO NORMALIZADO** — e este
/// não lia.
///
/// A agulha `ring_on_surface(&self.camera` é uma chamada com o **primeiro
/// argumento na mesma linha**, e o `rustfmt` quebra a chamada em sete linhas
/// assim que ela passa a largura. Medido em 2026-09-08: este gate estava VERDE
/// só porque o ficheiro tinha sido commitado **sem `cargo fmt`** por uma wave
/// anterior; à primeira formatação da árvore ele ficou vermelho **sobre produto
/// correcto**.
///
/// ⚠️ *Um censo cuja agulha atravessa uma quebra de linha mede o FORMATADOR, não
/// o código* — e o doc deste ficheiro já dizia *«ele afirma a PROPRIEDADE, nunca
/// um endereço»*, o que uma agulha com argumento dentro não cumpre.
fn achatado(src: &str) -> String {
    src.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_brush_cursor_asks_the_surface_for_its_orientation() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sculpt3d_cursor.rs");
    let bruto = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("o dono do cursor mudou-se: {} ({e})", path.display()));
    let src = achatado(&bruto);

    // CONTROLE POSITIVO.
    assert!(
        src.contains("fn cursor_mark("),
        "o dono do cursor mudou-se — reaponte o gate antes de confiar nele"
    );

    assert!(
        src.contains("self.surface_normal(i, hit)"),
        "o cursor tem de PERGUNTAR a orientação da superfície onde ele pousa; \
         sem isso o anel volta a ser a silhueta em toda parte"
    );
    assert!(
        src.contains("ring_on_surface( &self.camera"),
        "o anel tem de sair da porta que DEITA na superfície (`ring_on_surface`)"
    );
    // ⚠️ O círculo de tela FICA — ele é o recuo honesto para *"não sei a
    // orientação"* (normal degenerada, ou amostra atrás do olho), e apagá-lo
    // deixaria esses casos sem cursor nenhum.
    assert!(
        src.contains("ring(f64::from(cx), f64::from(cy), r)"),
        "a silhueta continua sendo o RECUO; apagá-la deixa o vazio e a normal \
         degenerada sem cursor"
    );

    // ⚠️ **A normal vem da família SUAVE, e é isto que mantém o cursor fora da
    // lacuna nomeada do `Hit::normal`** (um quad "gravata" devolve `[0,0,0]`,
    // e o gatilho declarado da cura dele é *"o primeiro leitor de produto"*).
    assert!(
        src.contains("mesh.normals()"),
        "a normal do cursor tem de ser a SUAVE (a régua do kernel), não a \
         geométrica do `Hit`"
    );
    assert!(
        !src.contains("hit.normal"),
        "ler `Hit::normal` faria deste cursor o primeiro leitor de produto de um \
         campo com lacuna NOMEADA — a cura dela mexe no laço quente do raycast"
    );
}
