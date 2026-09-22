//! ⭐⭐⭐ **QUE CONTROLO É QUE FICA ABAIXO DA DOBRA — pelo NOME, e não em píxeis.**
//!
//! # ⛔⛔ O buraco que ela fecha
//!
//! O censo irmão ([`super::quantas_entradas_tem_cada_painel`]) responde **quanto** de um painel
//! cai fora do ecrã (`+1 217 px`, `138 %`) e **onde começa cada secção**. Nenhuma das duas colunas
//! diz ***o quê***, e a pergunta da triagem da `D2` é essa: *o que o artista não alcança é o
//! `Radius` ou é a margem de fim de secção?* — e as duas leituras mandam a wave para sítios opostos.
//!
//! ⚠️ **É a mesma forma que a `line/sculpt3d` pagou quatro vezes:** *as três réguas da ponta mediam
//! ápice a ápice e **deitavam fora o índice** antes de devolver* (`CLAUDE.md` §5.1). Uma régua que
//! agrega diz que há dívida; só uma que **nomeia** diz o que fazer com ela.
//!
//! # ⛔⛔⛔ E na primeira corrida ela corrigiu a MINHA nota, em espécie
//!
//! Eu tinha escrito que faltavam **`22 px`** para a secção `Brush` caber acima da dobra. Medido
//! aqui, fileira a fileira, no estado do **dia a dia**:
//!
//! ```text
//!   740..762     sculpt3d.mask_op.{0..3}      as quatro operações de máscara
//!   768..790     sculpt3d.extract
//!   796..818     sculpt3d.extract_thick(+num)
//!   821..843     sculpt3d.extract_smooth(+num)
//!   866..888  ⛔ sculpt3d.transform.{0,1,2}
//!   ────── a dobra: 880 ──────
//!   902..925  ⛔ sculpt3d.sec.symmetry
//! ```
//!
//! ⇒ os `22` eram a distância até ao **fim PADDED da secção** (`902`), e o conteúdo acaba em `888`:
//! o que de facto fica cortado é **UMA fileira de chips**, e por **`8 px`**. *Uma régua que mede
//! até ao fim de uma secção mede a MARGEM dela e chama-lhe controlo.*
//!
//! # ⭐⭐ O nome é DERIVADO, nunca uma lista
//!
//! Um `NodeId` de widget é o FNV-1a de um *slug* ([`ph2d_tool_registry::hash_node_id`]), logo o
//! mapa inverso constrói-se lendo os literais `hash_node_id("…")` das crates que os declaram e
//! re-hashando cada um. ⛔ Nada aqui é uma lista de nomes escrita à mão — e o que o varrimento não
//! souber nomear aparece como `(sem nome)`, que é o que a impede de mentir por omissão.
//!
//! ⚠️ **Piso de população no próprio varrimento** (HOWTO §2.7): um extractor que deixasse de casar
//! devolveria o mapa vazio, **toda** linha leria `(sem nome)` e a sonda continuaria a imprimir uma
//! tabela — *que é a forma exacta de um instrumento partido se ler como um produto limpo*.
//!
//! # ⚠️ Ela mede num ARNÊS, e há uma segunda régua que mede na APP
//!
//! A `line/sculpt3d` tem a irmã desta (`diag_onde_cai_a_pista_do_pente`, na costura daquele
//! painel), com outra fixtura e outro verbo na mão — e os dois números **não** se somam nem se
//! comparam. Elas concordam no que interessa (a ordem das secções e o que cruza a dobra) e
//! discordam nos píxeis, porque armam estados diferentes. ⛔ *Misturar dois instrumentos numa conta
//! é a forma exacta de fabricar uma medição*, e por isso cada tabela deste ficheiro diz de qual é.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ph2d_editor_core::NodeId;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_ui_testkit::MockPanelHost;

use super::quantas_entradas_tem_cada_painel::{DOBRA, VIEWPORT};

/// As crates cujos literais este mapa lê.
///
/// ⚠️ **Duas e não uma:** os ids que só o painel lê desceram para a crate dele em 2026-09-12
/// (auditoria A5b), e os que o *z-order walk* da fundação percorre ficaram lá. *Ler só uma devolve
/// metade dos nomes, e metade lê-se como um painel cheio de anónimos.*
const FONTES: [&str; 2] = ["../ph2d-panel-sculpt3d/src", "../ph2d-editor-core/src/ids"];

/// ⛔ **Piso de população.** Medido em 2026-09-20: `1 035` slugs distintos nas duas árvores.
const PISO_DE_SLUGS: usize = 600;

fn raiz(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn varre(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        panic!("nao consegui ler {}", dir.display());
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            varre(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Os *slugs* de `hash_node_id("…")` de um ficheiro.
fn slugs_de(src: &str) -> Vec<String> {
    // ⚠️ A agulha monta-se: este ficheiro é lido por um extractor irmão, e um censo textual que se
    //    lê a si mesmo encontra sempre o que procura (a lei que a `line/sculpt3d` pagou no gate do
    //    device). Aqui ela só custa uma linha.
    let abre = concat!("hash_node_id", "(\"");
    let mut out = Vec::new();
    let mut resto = src;
    while let Some(i) = resto.find(abre) {
        resto = &resto[i + abre.len()..];
        match resto.find('"') {
            Some(j) => {
                out.push(resto[..j].to_string());
                resto = &resto[j..];
            }
            None => break,
        }
    }
    out
}

/// ⭐⭐⭐ **O MAPA INVERSO** — `NodeId` → o *slug* que o produziu.
/// ⭐ **Dois leitores desde 2026-09-21**: a sonda daqui e o censo da altura de uma marca
/// ([`super::a_marca_tem_a_altura_da_linha`]) — *uma segunda cópia deste extractor seria a segunda
/// resposta a «de que slug veio este id?»*.
pub(crate) fn nomes() -> BTreeMap<NodeId, String> {
    nomes_de(&FONTES, PISO_DE_SLUGS)
}

/// ⭐⭐ **O MESMO extractor, sobre as árvores que a PERGUNTA pede.**
///
/// ⚠️ A lista de fontes e o piso são **por pergunta** — o censo da altura de uma marca varre os
/// ids do Inspector, que esta sonda não precisa de ler. ⛔ O que NÃO se duplica é o
/// [`slugs_de`]: *uma segunda cópia do FNV-1a seria a segunda resposta a «que id é este?»*.
pub(crate) fn nomes_de(fontes: &[&str], piso: usize) -> BTreeMap<NodeId, String> {
    let mut ficheiros = Vec::new();
    for f in fontes {
        varre(&raiz(f), &mut ficheiros);
    }
    let mut mapa = BTreeMap::new();
    for f in &ficheiros {
        let src = std::fs::read_to_string(f).unwrap_or_default();
        for s in slugs_de(&src) {
            mapa.insert(ph2d_editor_core::registry::hash_node_id_runtime(&s), s);
        }
    }
    assert!(
        mapa.len() >= piso,
        "o varrimento leu {} slugs e esperava >= {piso} — o extractor cegou, e um mapa \
         vazio faz esta sonda imprimir `(sem nome)` em toda linha",
        mapa.len()
    );
    mapa
}

/// O que o painel da escultura pinta no estado do **dia a dia**, de cima para baixo.
fn fileiras_do_dia_a_dia() -> Vec<(f32, f32, NodeId)> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "sculpt3d")
            .expect("o painel da escultura tem de estar no registo");
        let mut host = MockPanelHost::new();
        super::o_sculpt3d_armado::arma_o_dia_a_dia();
        painel.populate(host.store_mut());
        let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        let pintados = host.registos_da_ultima_pintura();
        // ⛔ O estado que uma fixtura deixa para trás é o estado que a régua seguinte mede.
        super::o_sculpt3d_armado::desarma();

        let mut v: Vec<(f32, f32, NodeId)> = pintados
            .iter()
            .filter(|(id, _)| host.store().get(*id).is_some())
            .map(|(id, r)| (r.y, r.h, *id))
            .collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
        v
    })
}

/// ⭐⭐⭐ **A SONDA — o painel inteiro, fileira a fileira, com a dobra marcada.**
///
/// ⚠️ Ela **não reprova**: quem reprova são os dois gates da costura daquele painel
/// (`os_controlos_proprios_de_um_pincel_cabem_no_encaixe`, cuja catraca esta linha levou a ZERO ao
/// tirar o selector de pincéis, e `o_roteiro_da_49_diz_onde_o_edge_flow_esta`, que é uma
/// EQUIVALÊNCIA e reprova no dia em que a arrumação subir aquela fileira). Esta existe para dizer
/// *o que cortar a seguir*, que é outra pergunta.
///
/// ⛔ **Imprime TUDO de propósito, sem janela à volta da dobra.** A 1.ª redacção só mostrava os
/// `120 px` acima dela, e a triagem da `D2` pergunta pela secção inteira: *destas fileiras, quantas
/// são COMANDOS?* — uma janela responde por meia dúzia e deixa a conta por fazer.
#[test]
fn diag_o_que_fica_abaixo_da_dobra_na_escultura() {
    let nomes = nomes();
    let fileiras = fileiras_do_dia_a_dia();
    println!("\n  === a escultura no DIA A DIA (dobra = {DOBRA:.0} px) ===");
    for (y, h, id) in &fileiras {
        let fundo = y + h;
        println!(
            "  {y:>6.0}..{fundo:<6.0} {}  {}",
            if fundo > DOBRA { "⛔" } else { "  " },
            nomes.get(id).map(String::as_str).unwrap_or("(sem nome)"),
        );
    }
    println!("  ({} fileiras)\n", fileiras.len());
}
