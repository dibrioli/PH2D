//! ⭐⭐⭐ **Uma fileira de botões é disposta pela PORTA — ninguém divide a largura à mão.**
//!
//! Enio, 2026-09-07, com três fotos: *«vários grupos de botões em vários painéis (flip, Audio
//! Editor, etc) ainda estão sem seus grupos de botões ajuntados no novo formato. Tudo o que puder
//! for ajuntado, ajunte. Apenas quando o grupo for nitidamente de função diferente é que deve
//! permanecer grupos afastados.»*
//!
//! # A forma que este censo procura
//!
//! `largura_disponível − vão × (n − 1), dividido por n` — a conta que o
//! [`ph2d_editor_core::widget::segment_rects`] faz desde a wave 10, reescrita no sítio da pintura.
//! ⚠️ **Ela não se lê como cópia:** lê-se como um painel que «ainda não foi convertido», e a
//! diferença só aparece a olho — as peças ficam com **vão** em vez de encostarem, e as quatro
//! quinas de cada uma arredondam em vez de só as de fora.
//!
//! ⛔ **O que este censo NÃO decide é se dois botões SÃO um grupo** — isso é produto, e a régua é a
//! do dono: *o mesmo assunto junta-se; função nitidamente diferente fica separada*. O que ele
//! impede é a **disposição** ser reinventada, que é o que faz o mesmo par (`Apply | Cancel`)
//! aparecer junto num painel e separado noutro.
//!
//! ⚠️⚠️ **E a causa dessa divergência era um widget que não conhecia a lei:** o chip segmentado
//! sabia-a desde a wave 10 e o [`ph2d_editor_core::widget::Button`] não — logo o editor de áudio
//! juntava `Apply | Cancel` e as **quatro** ferramentas de imagem desenhavam `Cancel | Apply`
//! separados. *Uma lei que só metade dos widgets conhece produz dois dialectos no mesmo app.*

use std::fs;
use std::path::{Path, PathBuf};

/// `(caminho relativo, porquê a divisão à mão fica)`.
///
/// ⚠️ **Cada entrada é uma superfície cujo layout NÃO é uma fileira de peças iguais** — e a metade
/// de baixo obriga-a a continuar a existir.
const HAND_LAYOUT_OK: &[(&str, &str)] = &[
    (
        "crates/ph2d-panel-audio-editor/src/paint_spectral.rs",
        "as duas metades hospedam CAMPOS numericos, nao botoes",
    ),
    (
        "crates/ph2d-panel-color-equalization/src/paint_sections.rs",
        "a metade esquerda e' um CHIP DE DROPDOWN (Posterize) e a direita um botao (Dither), cada \
         uma com o proprio rotulo por cima: duas especies de widget nao formam uma fileira de \
         pecas iguais",
    ),
    (
        "crates/ph2d-panel-painter-layers/src/number_field.rs",
        "as duas metades hospedam campos numericos",
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
            if p.file_name().and_then(|n| n.to_str()) != Some("target") {
                walk(&p, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn panel_sources() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") {
                walk(&p.join("src"), &mut out);
            }
        }
    }
    out.sort();
    out
}

/// Uma linha que ENCOSTA um botão ao anterior por uma largura calculada à mão — a forma
/// `x + <meia largura> + <vão>` dentro de um `Rect::new`.
///
/// ⚠️ **A régua é o `Rect::new` na mesma linha**, e não a existência de um `half`: um `half` que
/// serve para centrar texto, para partir um intervalo ou para achar o meio de um marcador não
/// dispõe peça nenhuma. *Um censo que procura o nome da variável acusa a aritmética inteira do
/// repo.*
fn hand_laid_rows() -> Vec<String> {
    let root = repo_root();
    let exempt: Vec<&str> = HAND_LAYOUT_OK.iter().map(|(f, _)| *f).collect();
    let mut out = Vec::new();
    for p in panel_sources() {
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        if exempt.contains(&rel.as_str()) {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        for (n, line) in src.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || !t.contains("Rect::new(") {
                continue;
            }
            let joined = ["+ half + gap", "+ half_btn + btn_gap", "+ half + btn_gap"];
            if joined.iter().any(|j| line.contains(j)) {
                out.push(format!("{rel}:{}: {t}", n + 1));
            }
        }
    }
    out
}

/// ⭐ **Ninguém encosta um botão ao anterior somando meia largura e um vão.**
///
/// **Mutação que deve sangrar:** repor no `ph2d-panel-padding` o
/// `Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h)` — é exactamente o estado em que
/// esta wave encontrou as quatro ferramentas de imagem.
#[test]
fn no_panel_lays_a_button_row_out_by_hand() {
    let found = hand_laid_rows();
    assert!(
        found.is_empty(),
        "{} sitio(s) dispoem uma fileira de botoes a' mao em vez de chamar \
         `ph2d_editor_core::widget::segment_rects` (ou `block_cells`, se forem varias fileiras do \
         mesmo assunto). Cada um pinta pecas SEPARADAS onde a casa junta:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}

/// ⛔ **A metade que impede a isenção de virar licença.**
#[test]
fn the_hand_layout_exemptions_still_describe_something() {
    let mut stale = Vec::new();
    for (rel, _) in HAND_LAYOUT_OK {
        match fs::read_to_string(Path::new(&repo_root()).join(rel)) {
            Err(_) => stale.push(format!("{rel}: o ficheiro nao existe")),
            Ok(src) => {
                if !src.contains("half") {
                    stale.push(format!(
                        "{rel}: ja' nao divide largura nenhuma a' mao — a isencao descreve nada"
                    ));
                }
            }
        }
    }
    assert!(
        stale.is_empty(),
        "{} isencao(oes) obsoleta(s):\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}

/// ⭐⭐⭐ **O `Button` conhece a lei do grupo — e um botão SOZINHO é byte-idêntico ao de antes.**
///
/// ⚠️ **As duas metades são obrigatórias.** Sem a primeira, `Cancel | Apply` continuaria a pintar
/// quatro quinas em cada peça mesmo dentro de um `segment_rects` — o par encostaria e continuaria
/// a ler-se como dois controlos. Sem a segunda, dar a lei ao widget teria repintado os ~100 botões
/// soltos do app.
#[test]
fn a_grouped_button_rounds_only_its_outer_corners_and_a_lone_one_is_unchanged() {
    use ph2d_editor_core::widget::{Button, ButtonKind, GroupCell, GroupPos, paint_button};
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    use ph2d_vector::VectorScene;

    let rect = Rect::new(0.0, 0.0, 80.0, 22.0);
    let mut ts = TextSystem::without_system_fonts();

    let paint = |b: &Button, ts: &mut TextSystem| {
        let mut scene = VectorScene::new();
        paint_button(b, rect, &mut scene, ts, Theme::Dark);
        scene.inner().encoding().n_path_segments
    };

    // ⚠️ **ACENTO, e é o piso a dizê-lo:** um botão `Default` num tema moderno é FANTASMA — não
    // pinta fundo nem moldura, e a 1.ª redacção deste gate mediu `0` segmentos e reprovou. *Uma
    // régua de geometria precisa de uma fixtura que emita geometria.*
    let lone = Button::new(ph2d_a11y::NodeId(1), "Apply").kind(ButtonKind::Accent);
    let explicit_only = Button::new(ph2d_a11y::NodeId(1), "Apply")
        .kind(ButtonKind::Accent)
        .in_group(GroupCell {
            col: GroupPos::Only,
            row: GroupPos::Only,
        });
    let first = Button::new(ph2d_a11y::NodeId(1), "Apply")
        .kind(ButtonKind::Accent)
        .in_group(GroupCell {
            col: GroupPos::First,
            row: GroupPos::Only,
        });

    let (a, b, c) = (
        paint(&lone, &mut ts),
        paint(&explicit_only, &mut ts),
        paint(&first, &mut ts),
    );
    assert!(a > 0, "o botao nao pintou geometria nenhuma");
    assert_eq!(
        a, b,
        "declarar `Only` mudou o desenho — o neutro deixou de ser neutro"
    );
    assert_ne!(
        a, c,
        "uma peca de grupo (`First`) emitiu a MESMA geometria de um botao sozinho: as quinas de \
         dentro continuam arredondadas e o par encostado le^-se como dois controlos"
    );
}

/// ⭐⭐⭐ **E A FILEIRA MEDE AS PALAVRAS — uma que CABE INTEIRA nunca corta a peça mais larga.**
///
/// ⛔⛔ **Segunda metade da lei, medida em 2026-09-19** pela varredura de elisões com o Inspector
/// armado: o [`ph2d_editor_core::widget::segment_rects`] reparte em partes **IGUAIS**, e *uma
/// média não é um máximo* — `w/n` pode ser menor do que UMA das peças precisa mesmo quando a soma
/// delas cabe de sobra. `+ Add Transition | x Remove Transition` recebia `118 px` cada e pintava
/// **`x Remove Transi…`** com a fileira a caber inteira.
///
/// ⚠️ **A fixtura é a do report**, com a largura DERIVADA (a soma das duas larguras naturais mais
/// o traço), e não um número escolhido: é isso que torna a premissa — *ela cabia* — parte do gate
/// em vez de uma afirmação no comentário.
///
/// ⭐ **As duas metades são obrigatórias.** A negativa (a divisão igual CORTA) é o controlo
/// positivo: sem ela, uma fixtura em que nada é cortado passaria com a lei apagada.
///
/// (Mutação: `segment_rects_for` devolver `segment_rects(row, labels.len())` ⇒ a 1.ª metade acusa
/// a peça larga, RED.)
#[test]
fn uma_fileira_que_cabe_nunca_corta_a_peca_mais_larga() {
    use ph2d_editor_core::paint::{label_budget, rect_for_label};
    use ph2d_editor_core::widget::{button_label_font, segment_rects, segment_rects_for};
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;

    let mut ts = TextSystem::without_system_fonts();
    let font = button_label_font();
    let rotulos = [
        ph2d_i18n::tr("panel.inspector.statemachine.add_transition"),
        ph2d_i18n::tr("panel.inspector.statemachine.x_remove_transition"),
    ];
    let palavras: Vec<f32> = rotulos.iter().map(|r| ts.prefix_width(r, font)).collect();
    // A coluna é a que a fileira PEDE — nem um pixel a mais. ⚠️ **Ao PIXEL**, porque a costura de
    // um grupo cai em inteiros: pedir a soma fraccionária seria pedir uma coluna que não existe, e
    // a lei leria «não cabe» sobre uma fileira que o artista vê caber.
    let w = rect_for_label(palavras[0]).ceil() + rect_for_label(palavras[1]).ceil() + 1.0;
    let row = Rect::new(0.0, 0.0, w, 30.0);

    // ── A LEI ────────────────────────────────────────────────────────────────
    let seg = segment_rects_for(row, &rotulos, font, &mut ts);
    assert_eq!(seg.len(), 2);
    for (i, (rect, _)) in seg.iter().enumerate() {
        assert!(
            label_budget(rect.w) >= palavras[i],
            "a peca {i} ({:?}) pede {:.1} px de palavra e recebeu uma caixa de {:.1} \
             (orcamento {:.1}) — e a fileira CABIA em {w:.1}",
            rotulos[i],
            palavras[i],
            rect.w,
            label_budget(rect.w)
        );
    }
    // A fileira acaba exactamente na borda da coluna.
    let fim = seg[1].0.x + seg[1].0.w;
    assert!(
        (fim - w).abs() < 1e-3,
        "a fileira acaba em {fim:.2} numa coluna de {w:.2}"
    );

    // ── O CONTROLO POSITIVO: em partes IGUAIS, a mais larga É cortada ────────
    let iguais = segment_rects(row, 2);
    let largo = if palavras[1] >= palavras[0] { 1 } else { 0 };
    assert!(
        label_budget(iguais[largo].0.w) < palavras[largo],
        "a fixtura nao contem o fenomeno: em partes iguais a peca mais larga ({:?}, {:.1} px) \
         ainda cabia em {:.1} — sem isto a metade de cima passa com a lei apagada",
        rotulos[largo],
        palavras[largo],
        iguais[largo].0.w
    );
}

/// ⭐⭐ **A DEGENERADA: palavras do mesmo tamanho devolvem o que a divisão igual devolvia, AO
/// PIXEL.**
///
/// ⛔ Sem ela, medir as palavras teria movido toda fileira do app cujas peças já eram do mesmo
/// tamanho (`M | S` do mixer, os nove tipos de grelha) — e o diff passaria a ser ilegível.
#[test]
fn com_palavras_iguais_a_fileira_nao_se_mexe() {
    use ph2d_editor_core::widget::{button_label_font, segment_rects, segment_rects_for};
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;

    let mut ts = TextSystem::without_system_fonts();
    let row = Rect::new(7.0, 3.0, 240.0, 22.0);
    let rotulos = ["Mute", "Mute", "Mute"];
    let medido = segment_rects_for(row, &rotulos, button_label_font(), &mut ts);
    let igual = segment_rects(row, rotulos.len());
    assert_eq!(medido.len(), igual.len());
    for (i, ((a, ca), (b, cb))) in medido.iter().zip(igual.iter()).enumerate() {
        assert!(
            (a.x - b.x).abs() < 1e-3 && (a.w - b.w).abs() < 1e-3,
            "a peca {i} mudou de sitio: medida x={:.2} w={:.2}, igual x={:.2} w={:.2}",
            a.x,
            a.w,
            b.x,
            b.w
        );
        assert_eq!(ca, cb, "a peca {i} mudou de posicao no grupo");
    }
}

/// ⛔⛔ **NENHUM PAINEL REPARTE UMA FILEIRA DE PEÇAS COM RÓTULO EM PARTES IGUAIS.**
///
/// ⚠️ O [`ph2d_editor_core::widget::segment_rects`] continua a existir e a ser a base de toda a
/// família (o `block_cells` chama-o), mas o que um PAINEL dispõe tem sempre uma palavra dentro —
/// e para essas a porta é a [`ph2d_editor_core::widget::segment_rects_for`], que as mede.
///
/// ⭐ A catraca está a **ZERO**: os 13 sítios que existiam passaram à porta nova em 2026-09-19, e
/// já não há linha onde escrever uma fileira que corta a palavra mais larga.
#[test]
fn nenhum_painel_reparte_uma_fileira_de_rotulos_em_partes_iguais() {
    let root = repo_root();
    let mut found = Vec::new();
    for p in panel_sources() {
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        for (n, line) in src.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") {
                continue;
            }
            if line.contains("segment_rects(") {
                found.push(format!("{rel}:{}: {t}", n + 1));
            }
        }
    }
    assert!(
        found.is_empty(),
        "{} sitio(s) repartem uma fileira em partes IGUAIS. Uma media nao e' um maximo: a fileira \
         pode caber inteira e cortar a peca mais larga na mesma. Chame \
         `ph2d_editor_core::widget::segment_rects_for`, que mede as palavras:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}
