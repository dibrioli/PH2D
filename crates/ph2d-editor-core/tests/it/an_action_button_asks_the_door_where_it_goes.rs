//! ⭐⭐⭐ **Um botão de acção de LINHA INTEIRA pergunta à porta onde fica — em TODO painel.**
//!
//! ⛔⛔ Report do dono, 2026-09-24, foto do cartão da junta no Inspector: *«esses botões que
//! atravessam de lado a lado talvez fiquem melhor na coluna do lado direito»*. A cura foi a porta
//! [`ph2d_editor_core::property_row::caixa_do_botao`] (a coluna do valor quando o rótulo lá cabe, a
//! linha inteira quando não cabe) e, no mesmo dia, o smoke aprovou-a e mandou alargá-la aos outros
//! painéis. O Inspector tem o censo dele (`nenhum_botao_atravessa_a_linha_a_mao`); este é o dos
//! outros **26**, porque cada painel escreve o idioma com nomes seus (`inner_x`/`inner_w`,
//! `layout.inner_x`, `self.inner_x`, `x`/`content_w`…).
//!
//! # A forma que este censo procura
//!
//! `let <v> = Rect::new(<início da linha>, _, <largura da linha>, _)` seguido de um
//! `paint_button` que usa `<v>` — um botão que ATRAVESSA a linha por decisão escrita à mão. Os
//! pares «início, largura» são os que os painéis usam para a linha inteira ([`PARES_DA_LINHA`]); uma
//! CÉLULA (metade, terço, quadrado) usa outros nomes e não entra.
//!
//! ⛔ **O que este censo NÃO decide é se um botão é de PROPRIEDADE ou de RODAPÉ** — isso é produto,
//! e a lista [`LINHA_INTEIRA_OK`] nomeia cada excepção com o porquê. ⚠️ A metade de baixo obriga
//! cada uma a continuar a existir: uma excepção cujo sítio sumiu é uma catraca que virou licença.

use std::fs;
use std::path::{Path, PathBuf};

/// Os pares `(início, largura)` com que os painéis escrevem «a linha inteira».
const PARES_DA_LINHA: &[(&str, &str)] = &[
    ("inner_x", "inner_w"),
    ("layout.inner_x", "layout.inner_w"),
    ("self.inner_x", "self.inner_w"),
    ("x", "w"),
    ("x", "content_w"),
    ("list.x", "list.w"),
];

/// `(caminho relativo, variável, porquê a linha inteira fica)`.
///
/// ⚠️ **Cada entrada é uma decisão de PRODUTO medida, nunca «ainda não convertido».**
const LINHA_INTEIRA_OK: &[(&str, &str, &str)] = &[
    (
        "crates/ph2d-panel-bgremoval/src/paint_sections.rs",
        "reset_rect",
        "RODAPE: o Reset fica na linha de cima do par Cancel | Apply, que atravessa a linha; na \
         coluna do valor ele ficaria por cima do Apply, e o cabecalho do bloco diz que o destrutivo \
         nao encosta no CTA",
    ),
    (
        "crates/ph2d-panel-color-equalization/src/paint_sections.rs",
        "reset_rect",
        "RODAPE: o mesmo bloco Reset / Cancel | Apply das ferramentas de imagem",
    ),
    (
        "crates/ph2d-panel-equalize-sizes/src/paint_actions.rs",
        "reset_rect",
        "RODAPE: o mesmo bloco Reset / Cancel | Apply das ferramentas de imagem",
    ),
    (
        "crates/ph2d-panel-padding/src/paint.rs",
        "reset_rect",
        "RODAPE: o mesmo bloco Reset / Cancel | Apply das ferramentas de imagem",
    ),
    (
        "crates/ph2d-panel-upscale/src/paint.rs",
        "reset_rect",
        "RODAPE: o mesmo bloco Reset / Cancel | Apply das ferramentas de imagem",
    ),
    (
        "crates/ph2d-panel-painter-layers/src/paint_mask.rs",
        "apply_rect",
        "CTA de fecho: o Apply Mask (accent) coze a mascara e fecha a seccao, como o Apply do \
         rodape das ferramentas de imagem",
    ),
    (
        "crates/ph2d-panel-grid-snap/src/paint_helpers.rs",
        "rect",
        "CTA-heroi: o botao Snap e' a accao do painel inteiro e tem altura propria (44 px)",
    ),
    (
        "crates/ph2d-panel-physics/src/paint/body.rs",
        "rect",
        "OUTRA COLUNA: o `toggle`/`command` deste painel. Medido em 2026-09-24 pelo \
         `dentro_de_um_troco_o_valor_arranca_numa_coluna_so`: a coluna que o painel de fisica \
         arranca e' a da grelha das camadas (`x = 26`), e o `Enabled` do sono na coluna da porta \
         (`x = 136`) abria uma SEGUNDA; alinha-lo pede o painel inteiro na coluna do valor, que e' \
         outra wave",
    ),
    (
        "crates/ph2d-panel-timeline/src/tracks.rs",
        "r",
        "MENU: a lista de propriedades a acrescentar e' um menu flutuante, cada botao e' uma linha \
         do menu e nao uma linha de propriedade",
    ),
    (
        "crates/ph2d-panel-vector/src/paint_text_sections.rs",
        "rect",
        "CELULA: o `arrow_button` recebe o quadrado `<`/`>` do seletor de fonte do chamador; os \
         parametros chamam-se `x`/`w` mas sao a celula",
    ),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// As fontes de PRODUTO dos painéis, fora o Inspector (que tem o censo dele).
fn panel_sources() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") && name != "ph2d-panel-inspector" {
                walk(&p.join("src"), &mut out);
            }
        }
    }
    out.retain(|p| {
        let n = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        !n.ends_with("_tests.rs") && !n.starts_with("seam_")
    });
    out.sort();
    out
}

/// Os argumentos de topo de uma chamada cujo `(` já foi consumido (até ao `)` que a fecha).
fn top_args(src: &str) -> Vec<String> {
    let mut depth = 1;
    let mut cur = String::new();
    let mut out = Vec::new();
    for c in src.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                cur.push(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    // A vírgula final de uma chamada partida por linhas deixa um argumento vazio.
                    if !cur.trim().is_empty() {
                        out.push(cur.trim().to_string());
                    }
                    return out;
                }
                cur.push(c);
            }
            ',' if depth == 1 => out.push(std::mem::take(&mut cur).trim().to_string()),
            _ => cur.push(c),
        }
    }
    out
}

/// `(variável, linha)` de cada botão que atravessa a linha por um rect escrito à mão.
fn botoes_de_linha_inteira(src: &str) -> Vec<(String, usize)> {
    let linhas: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, l) in linhas.iter().enumerate() {
        let t = l.trim();
        let Some(resto) = t.strip_prefix("let ") else {
            continue;
        };
        let Some((var, rhs)) = resto.split_once(" = ") else {
            continue;
        };
        let Some(depois) = rhs.strip_prefix("Rect::new(") else {
            continue;
        };
        // A chamada pode partir-se por várias linhas: junta-se a janela e lêem-se os argumentos.
        let janela = std::iter::once(depois)
            .chain(linhas[i + 1..(i + 8).min(linhas.len())].iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        let args = top_args(&janela);
        if args.len() != 4 {
            continue;
        }
        let de_linha = PARES_DA_LINHA
            .iter()
            .any(|(a, b)| args[0] == *a && args[2] == *b);
        if !de_linha {
            continue;
        }
        let seguinte = linhas[i + 1..(i + 30).min(linhas.len())].join("\n");
        let var = var.trim();
        let usa = seguinte.find("paint_button(").is_some_and(|k| {
            seguinte[k..].split(';').next().is_some_and(|chamada| {
                chamada
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|w| w == var)
            })
        });
        if usa {
            out.push((var.to_string(), i + 1));
        }
    }
    out
}

fn rel(p: &Path) -> String {
    p.strip_prefix(repo_root())
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// ⭐⭐⭐ **Todo botão de linha inteira escrito à mão está na lista de excepções — e só esses.**
///
/// **Mutação que deve sangrar:** repor o `Rect::new(inner_x, y, inner_w, row_h)` num dos botões
/// do remover-fundo (o `Separate Islands`, p. ex.) — é a forma que o dono fotografou no Inspector.
#[test]
fn an_action_button_asks_the_door_where_it_goes() {
    let fontes = panel_sources();
    // ⚠️ **Piso MEDIDO em 2026-09-24:** mais de `300` fontes de produto nos painéis. Uma varredura
    //    que devolvesse zero leria «nenhum acusado» e aprovaria tudo.
    assert!(
        fontes.len() >= 250,
        "a varredura leu só {} ficheiro(s) de painel — deixou de alcançar `crates/ph2d-panel-*`",
        fontes.len()
    );
    let mut achados: Vec<(String, String, usize)> = Vec::new();
    let mut pela_porta = 0usize;
    for f in &fontes {
        let src = fs::read_to_string(f).expect("fonte legível");
        pela_porta += src.matches("caixa_do_botao(").count();
        for (var, linha) in botoes_de_linha_inteira(&src) {
            achados.push((rel(f), var, linha));
        }
    }
    // ⚠️ **Piso MEDIDO em 2026-09-24: `19` botões pela porta fora do Inspector** (a chamada, com
    //    ou sem o caminho: o remover-fundo importa-a). É o que prova que
    //    a agulha ainda acha o idioma: uma porta renomeada deixaria os acusados a zero.
    assert!(
        pela_porta >= 16,
        "só {pela_porta} botão(ões) fora do Inspector passam pela `caixa_do_botao` — a porta mudou \
         de nome ou os botões voltaram a ser escritos à mão"
    );
    let novos: Vec<_> = achados
        .iter()
        .filter(|(f, v, _)| {
            !LINHA_INTEIRA_OK
                .iter()
                .any(|(ef, ev, _)| ef == f && ev == v)
        })
        .collect();
    assert!(
        novos.is_empty(),
        "estes botões atravessam a linha por um rect escrito à mão (report do dono de \
         2026-09-24): {novos:?}\n\
         cura: `ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, y, h, rótulo)` — \
         a coluna do valor quando o rótulo cabe, a linha inteira quando não cabe. Se o botão é de \
         RODAPÉ ou CTA, nomeie-o em `LINHA_INTEIRA_OK` com o porquê."
    );
    // ⛔ **A metade de baixo: a excepção cujo sítio sumiu SAI da lista.**
    let obsoletas: Vec<_> = LINHA_INTEIRA_OK
        .iter()
        .filter(|(ef, ev, _)| !achados.iter().any(|(f, v, _)| f == ef && v == ev))
        .map(|(ef, ev, _)| (*ef, *ev))
        .collect();
    assert!(
        obsoletas.is_empty(),
        "estas excepções já não descrevem nada — apague-as: {obsoletas:?}"
    );
}

/// O extractor apanha o idioma, numa linha ou partido, e não apanha uma célula nem um fundo.
#[test]
fn o_extractor_separa_a_linha_da_celula() {
    let mau = "let rect = Rect::new(inner_x, y, inner_w, row_h);\nlet b = Button::new(id, l);\n\
               paint_button(&b, rect, scene, ts, theme);";
    assert_eq!(botoes_de_linha_inteira(mau).len(), 1);
    let partido = "let rect = Rect::new(\n    self.inner_x,\n    y,\n    self.inner_w,\n    h,\n);\n\
                   paint_button(&b, rect, s, t, th);";
    assert_eq!(botoes_de_linha_inteira(partido).len(), 1);
    // Uma célula (metade) não conta.
    let celula = "let rect = Rect::new(x, y, half, row_h);\npaint_button(&b, rect, s, t, th);";
    assert!(botoes_de_linha_inteira(celula).is_empty());
    // Um fundo de linha inteira que não é botão não conta.
    let fundo = "let rect = Rect::new(inner_x, y, inner_w, h);\nfill_rect(scene, rect, cor);";
    assert!(botoes_de_linha_inteira(fundo).is_empty());
    // A porta não conta.
    let bom = "let rect = property_row::caixa_do_botao(ts, x, w, y, h, l);\n\
               paint_button(&b, rect, s, t, th);";
    assert!(botoes_de_linha_inteira(bom).is_empty());
}
