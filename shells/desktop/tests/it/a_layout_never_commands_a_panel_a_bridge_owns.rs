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

/// ⚠️ **O SEGUNDO sítio onde uma ponte pode viver, desde a Fase C (2026-09-12):** a
/// `motion_bridge` mudou-se para `ph2d-app-motion`. Varrer só o `render_loop` deixaria o
/// controlo positivo abaixo a acusar o `motion_graph` de não ter ponte — que é precisamente
/// o que ele fez, e por isso ele existe.
const BRIDGE_DIR_FAM: &str = "../../crates/ph2d-app-motion/src";

/// Como é que a ponte escreve a visibilidade.
const WRITE: &str = "panel_visibility.insert(";

/// Um `insert` lido: o id do painel e o que lhe foi atribuído.
fn writes() -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    // ⚠️ DOIS sítios desde a Fase C (2026-09-12) — ver [`BRIDGE_DIR_FAM`].
    let mut files: Vec<_> = [BRIDGE_DIR, BRIDGE_DIR_FAM]
        .iter()
        .flat_map(|d| {
            fs::read_dir(d)
                .unwrap_or_else(|_| panic!("{d} existe"))
                .filter_map(Result::ok)
                .map(|e| e.path())
        })
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

/// ⭐⭐⭐ **O `inspector` está nos layouts cuja ferramenta NÃO o toma, e em mais nenhum.**
///
/// > Enio, 2026-08-31: *«em animate o inspector está sendo escondido. Por padrão deve ficar
/// > visível.»*
///
/// ⛔⛔ **A 1.ª redacção desta wave tirou-o de TODOS os layouts** com o argumento de que ele tem
/// dois escritores — e **fechou-o em toda parte**, porque no `layout_switch::apply` *não nomear é
/// fechar*: a lista é absoluta sobre o registry inteiro. *«O layout não o comanda» e «o layout
/// comanda-o fechado» são a mesma linha de código, e só a segunda é o que acontece.*
///
/// ⭐ A lei é **derivável dos dois lados**: uma ponte `<tool>_bridge.rs` que escreva
/// `insert("inspector", !…)` é a ferramenta `<tool>` a **substituir** o inspector na coluna. Um
/// layout cujo dono do canvas faz isso não o pode nomear; um cujo dono não a faz **tem** de o
/// nomear, senão ele fecha e não há quem o reabra.
#[test]
fn a_layout_names_the_inspector_exactly_when_its_canvas_owner_does_not_take_it_over() {
    use ph2d_editor::screens::task_layout::CanvasOwner;
    let takeover = tools_that_take_over_the_inspector();
    // ⭐⭐⭐ **A LISTA ESTÁ VAZIA DESDE 2026-09-09, e isso é o PRODUTO, não a varredura partida.**
    //
    // > *«algumas ferramentas ou painéis não criam abas»* — Enio, 2026-09-08.
    //
    // As oito pontes que **substituíam** o inspector deixaram de o fazer: com a fileira de abas os
    // dois são OCUPANTES do mesmo encaixe, e esconder um deles era o modelo anterior às abas. Há
    // gate a proibi-lo pelo nome (`no_bridge_hides_the_inspector_to_take_its_slot`).
    //
    // ⇒ a lei abaixo **não muda**: ela continua a ser *«nomeia-o exactamente quando o dono do
    // canvas NÃO o toma»*. O que mudou é que ninguém o toma, logo **todo** layout tem de o nomear
    // — e foram precisos DOIS (o `vector` e o `flip`, que abriam só a hierarquia porque a
    // ferramenta deles o tomava). *Tirar o takeover sem reconciliar a tabela reintroduzia o report
    // de 31/08 em dois layouts: o inspector fechava e ninguém o reabria.*
    //
    // ⚠️ **O CONTROLO passa a ser o do INSTRUMENTO, não o da população:** com a população a zero,
    // exigir `>= 4` seria exigir para sempre uma tomada de conta que o produto apagou. O que tem de
    // continuar vivo é a varredura — ela lê a pasta das pontes, e uma pasta renomeada devolveria
    // zero pelo motivo errado.
    let bridges = fs::read_dir(BRIDGE_DIR)
        .expect("a pasta das pontes existe")
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with("_bridge"))
        })
        .count();
    assert!(
        bridges >= 8,
        "controlo do instrumento: só {bridges} pontes varridas em {BRIDGE_DIR} — a varredura \
         perdeu o alvo, e a lista vazia abaixo não significaria nada"
    );
    assert!(
        takeover.is_empty(),
        "uma ponte voltou a SUBSTITUIR o inspector ({takeover:?}) — o modelo anterior às abas. A \
         lei abaixo continua a valer (o layout dessa ferramenta deixa de o nomear), mas o gate \
         `no_bridge_hides_the_inspector_to_take_its_slot` diz por que isso não devia acontecer"
    );
    // ⚠️ **O `motion` SAIU desta lista em 2026-09-07, e não foi a varredura que se partiu:** os
    // params dos nós passaram a viver dentro dos cartões (doc 103) e o `motion_params` deixou de
    // ocupar a coluna da direita — sem ninguém a tomar o lugar, esconder o inspector seria tirar
    // uma superfície e não pôr nenhuma. *Um controlo que nomeia ferramentas tem de ser reconciliado
    // quando o PRODUTO muda; deixá-lo cá com o `motion` faria este gate exigir para sempre uma
    // tomada de conta que já não existe.*
    // ⛔ **O controlo por NOME (`vector`, `flip`) SAIU no mesmo dia**: ele exigia que aquelas duas
    //    pontes fizessem a tomada de conta, e elas deixaram de a fazer. *Um controlo que nomeia
    //    ferramentas tem de ser reconciliado quando o produto muda* — a nota logo acima já o dizia
    //    do `motion`, em 2026-09-07, e a lição voltou dois dias depois com duas ferramentas.

    let mut named = 0usize;
    for l in ph2d_editor::screens::task_layout::TaskLayout::ALL {
        let names = l.spec().open.contains(&"inspector");
        let taken = match l.spec().canvas {
            CanvasOwner::Tool(id) => takeover.iter().any(|t| t == id),
            // O modelador não é uma ferramenta e não tem ponte a substituir o inspector.
            CanvasOwner::Model3d => false,
        };
        assert_eq!(
            names,
            !taken,
            "{l:?}: a ferramenta dele {} o inspector e o layout {}-o — {}",
            if taken { "TOMA" } else { "não toma" },
            if names { "nomeia" } else { "não nomeia" },
            if taken {
                "a ponte desmente-o um quadro depois"
            } else {
                "e por isso ele fecha e ninguém o reabre (o report do Enio de 31/08)"
            }
        );
        named += usize::from(names);
    }
    assert!(
        named >= 3,
        "só {named} layouts mostram o inspector — ele voltou a ser fechado em toda parte"
    );
}

/// As ferramentas cujas pontes **substituem** o inspector — derivadas do nome do ficheiro da
/// ponte (`<tool>_bridge.rs`), que é a convenção deste directório.
fn tools_that_take_over_the_inspector() -> Vec<String> {
    let mut out = Vec::new();
    // ⚠️ As pontes vivem em DOIS sítios desde a Fase C: a `motion_bridge` saiu para a crate
    // da família e as outras nove continuam no `render_loop`.
    let mut files: Vec<_> = [BRIDGE_DIR, BRIDGE_DIR_FAM]
        .iter()
        .flat_map(|d| {
            fs::read_dir(d)
                .unwrap_or_else(|_| panic!("{d} existe"))
                .filter_map(Result::ok)
                .map(|e| e.path())
        })
        .collect();
    files.sort();
    for f in files {
        let Some(name) = f.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(tool) = name.strip_suffix("_bridge") else {
            continue;
        };
        let src = fs::read_to_string(&f).unwrap_or_default();
        let flat = src.replace('\n', " ").replace(" .", ".");
        if flat.contains("panel_visibility.insert(\"inspector\", !") {
            out.push(tool.to_string());
        }
    }
    out
}
