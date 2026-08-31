//! ⭐⭐⭐ **UM LAYOUT SÓ COMANDA O QUE NENHUMA PONTE POSSUI** — e é essa fronteira que o report do
//! Enio de 2026-08-31 descobriu.
//!
//! # O report, e por que ele não era do grafo
//!
//! *«Se abro Nodes e depois Model, o grafo de Nodes persiste. Procure outros problemas
//! similares.»*
//!
//! ⛔⛔ **A lista de abertos de um layout diz-se ABSOLUTA e não é a última palavra.** Ela é escrita
//! na PINTURA (fim do quadro); as pontes das ferramentas correm **antes** da pintura do quadro
//! seguinte e reescrevem, todas elas, a visibilidade dos painéis delas a partir de
//! `tools.active()`:
//!
//! ```text
//! motion_bridge:  panel_visibility.insert("motion_params", motion_active)   // TODO o quadro
//! vector_bridge:  panel_visibility.insert("vector",        vector_active)
//! painter_bridge: panel_visibility.insert("painter_layers", painter_is_active)
//! ```
//!
//! ⇒ enquanto o *Model* e o *Animate* não largavam a ferramenta em mãos, os painéis dela voltavam
//! **um quadro depois** de o layout os fechar. *A cura foi o `CanvasOwner` (nenhum layout herda a
//! ferramenta do anterior); este gate defende a outra metade — que a tabela não volte a NOMEAR o
//! que não lhe pertence.*
//!
//! # ⭐ O censo é DERIVADO da árvore, e a classificação é mecânica
//!
//! | o que a ponte escreve | é… | porquê |
//! |---|---|---|
//! | `insert(<id>, <identificador>)` | **POSSE** | o valor é um facto sobre a ferramenta, recalculado a cada quadro |
//! | `insert(<id>, true)` / `insert(<id>, false)` | **empurrão** | uma decisão tomada UMA vez, numa borda |
//! | `insert(<id>, !x)` | **empurrão** | a tomada de conta (*«o painel da ferramenta substitui o inspector»*), também de borda |
//!
//! ⚠️ Um empurrão pode ser desfeito por quem quer que seja depois; uma posse não. É por isso que o
//! `timeline` (que a `motion_bridge` **abre** por cortesia e nunca fecha) continua a ser do layout,
//! e o `motion_graph` não.
//!
//! ⚠️ **A varredura vê os ids escritos por LITERAL e por constante `PANEL_*`**, e nada mais — um
//! `insert` com um id calculado escaparia. Os controlos abaixo medem o tamanho do censo e exigem
//! nele três nomes conhecidos, para que uma expressão regular partida reprove em vez de aprovar.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ph2d_editor::screens::task_layout::TaskLayout;

/// Onde as pontes vivem. ⚠️ Uma ponte nova fora desta pasta escapa ao censo — e o piso abaixo é o
/// que torna essa fuga visível quando ela levar painéis com ela.
const BRIDGE_DIR: &str = "src/render_loop";

/// Como é que a ponte escreve a visibilidade.
const WRITE: &str = "panel_visibility.insert(";

/// Um `insert` lido: o id do painel e o que lhe foi atribuído.
fn writes() -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let dir = Path::new(BRIDGE_DIR);
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("{BRIDGE_DIR} existe"))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 10,
        "controlo: só {} ficheiros em {BRIDGE_DIR} — a varredura mudou de sítio",
        files.len()
    );
    for f in files {
        let src = fs::read_to_string(&f).expect("legível");
        // ⚠️ O `insert` das pontes quebra linha (o `rustfmt` parte-o em três), então a varredura
        // é sobre o ficheiro colado — nunca linha a linha.
        // ⚠️ E o `" ."` volta a colar: o `rustfmt` parte `hero.panel_visibility` de `.insert(`, e
        // sem isto o `painter_bridge` — a única ponte com essa quebra — escapava ao censo inteiro.
        // Foi o controlo de nome conhecido que o apanhou.
        let flat: String = src
            .lines()
            .map(str::trim)
            .filter(|l| !l.starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ")
            .replace(" .", ".");
        let mut rest = flat.as_str();
        while let Some(i) = rest.find(WRITE) {
            rest = &rest[i + WRITE.len()..];
            let Some(close) = rest.find(')') else {
                continue;
            };
            let args = &rest[..close];
            let Some((id_expr, value)) = args.split_once(',') else {
                continue;
            };
            if let Some(id) = panel_id(id_expr.trim()) {
                out.entry(id).or_default().push(value.trim().to_string());
            }
        }
    }
    out
}

/// O id do painel escrito neste argumento — `None` para uma forma que a varredura não sabe ler.
///
/// ⚠️ Duas formas, as duas mecânicas: `"motion_params"` e a constante. `ph2d_panel_model3d::PANEL_ID`
/// leva o nome no CAMINHO (a crate é `ph2d-panel-<id>`); `…hero::PANEL_MOTION_GRAPH` leva-o no
/// próprio nome, em maiúsculas.
fn panel_id(expr: &str) -> Option<String> {
    if let Some(lit) = expr.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return Some(lit.to_string());
    }
    let last = expr.rsplit("::").next()?;
    if last == "PANEL_ID" {
        let krate = expr.split("::").next()?;
        return krate
            .strip_prefix("ph2d_panel_")
            .map(std::string::ToString::to_string);
    }
    last.strip_prefix("PANEL_")
        .map(str::to_ascii_lowercase)
        .filter(|s| !s.is_empty())
}

/// ⭐ **Quem uma ponte POSSUI** — ver a tabela do cabeçalho.
fn owned_by_a_bridge() -> Vec<String> {
    writes()
        .into_iter()
        .filter(|(_, vals)| {
            vals.iter()
                .any(|v| v != "true" && v != "false" && !v.starts_with('!') && !v.contains(' '))
        })
        .map(|(id, _)| id)
        .collect()
}

/// ⭐⭐⭐ **A tabela dos layouts não nomeia nenhum painel de ferramenta.**
#[test]
fn no_layout_opens_a_panel_that_a_tool_bridge_owns() {
    let owned = owned_by_a_bridge();
    // Controlo: uma varredura partida devolveria pouco ou nada e o gate aprovaria tudo.
    assert!(
        owned.len() >= 8,
        "controlo: o censo achou só {} painéis de ferramenta ({owned:?}) — a varredura partiu-se",
        owned.len()
    );
    for known in ["motion_graph", "vector", "painter_layers"] {
        assert!(
            owned.iter().any(|o| o == known),
            "controlo: `{known}` é escrito por uma ponte a cada quadro e o censo não o vê: {owned:?}"
        );
    }
    // …e os DOIS empurrões conhecidos não podem entrar nele, senão a lei tira do layout coisas
    // que são dele.
    for nudged in ["inspector", "timeline"] {
        assert!(
            !owned.iter().any(|o| o == nudged),
            "controlo: `{nudged}` é um empurrão de borda e o censo classificou-o como POSSE — o \
             layout perderia o comando de um painel que é dele"
        );
    }

    let mut sins = Vec::new();
    for l in TaskLayout::ALL {
        for id in l.spec().open {
            if owned.iter().any(|o| o == id) {
                sins.push(format!(
                    "{l:?} abre `{id}`, que a ponte da ferramenta reescreve a cada quadro"
                ));
            }
        }
    }
    assert!(
        sins.is_empty(),
        "um layout comanda painéis que não são dele — a lista de abertos deixa de ser a última \
         palavra e a aba passa a mentir:\n  {}",
        sins.join("\n  ")
    );
}

/// ⚠️ **E o `inspector` continua fora da tabela, por outro motivo** — ele tem DOIS escritores com
/// uma ordem fixa.
///
/// ⛔ Seis pontes escrevem-no na borda de uma tomada (`insert("inspector", !active)`) **depois** de
/// o layout ter pintado. ⇒ o que o layout dissesse sobre ele era desmentido em exactamente as
/// transições que interessam — foi assim que a foto do Enio de 31/08 mostrou as abas
/// *Inspector | Vector* num layout que declarava o inspector aberto e a ponte declarava fechado.
/// *Um campo com dois escritores e uma ordem fixa tem um dono só, e não é quem escreve primeiro.*
#[test]
fn no_layout_claims_the_inspector_because_the_takeover_owns_it() {
    let nudgers = writes();
    assert!(
        nudgers
            .get("inspector")
            .is_some_and(|v| v.iter().any(|s| s.starts_with('!'))),
        "controlo: nenhuma ponte faz a tomada de conta do inspector — esta lei ficou obsoleta e a \
         tabela pode voltar a nomeá-lo"
    );
    for l in TaskLayout::ALL {
        assert!(
            !l.spec().open.contains(&"inspector"),
            "{l:?} declara o inspector, e a ponte da ferramenta desmente-o um quadro depois"
        );
    }
}
