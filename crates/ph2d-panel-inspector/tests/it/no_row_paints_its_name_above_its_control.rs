//! ⭐⭐⭐ **NENHUMA LINHA DO INSPECTOR PÕE O NOME POR CIMA DO CONTROLO.**
//!
//! ⛔⛔ **Report do dono, 2026-09-14, com foto da secção LEG do Platform Player:** *«Label acima do
//! campo numérico! Muito ruim!»*. A wave daquele dia curou as rows de UM campo
//! (`sections/rows::num_row`, `visibility::number_row`) e **não alcançou** as outras onze — o dono
//! voltou a apontá-las em 2026-09-15.
//!
//! Medido nesse dia, o Inspector tinha **onze** sítios a empilhar o nome:
//!
//! | família | sítios | porta |
//! |---|---|---|
//! | rows de N campos numéricos | 3 doors / 19 chamadas | `anchors::field_row` · `slice_nine::pair_row` · `sampling::uv_pair_row` |
//! | segmentado com «nenhum aceso» | 4 | `sampling` ×2 · `material_blend` · `slice_nine` |
//! | rectângulo X/Y/W/H | 1 | `visibility` (`Rect2Editor`) |
//! | grelha 3×3 + atalhos | 1 | `slice_grid` ⏳ |
//! | 4 amostras + prévia do gradiente | 1 | `color_tint` ⏳ |
//!
//! ⇒ **nove** convertidos; **dois** ficam, e ficam **com a medição**, não por cansaço.
//!
//! # ⚠️ Porque o censo é TEXTUAL e mesmo assim vale
//!
//! O que se procura é o IDIOMA, não o efeito: `paint_text(…)` e depois avançar o `y` pela altura do
//! rótulo (`y + label_h` / `yy += label_h`). ⛔ A alternativa — medir onde o rótulo caiu no painel
//! pintado — **não é alcançável**: das oito ids de várias componentes, zero são registadas pelo
//! `MockPanelHost` (medido 2026-09-15; elas pedem uma sprite com aquele componente). *Uma régua que
//! pedisse o rect mediria a ausência.*

use std::fs;
use std::path::PathBuf;

/// ⏳ **Os que FICAM, cada um com a MEDIÇÃO que os mantém — e só ENCOLHE.**
///
/// ⛔ **Não acrescente entradas.** Uma linha nova nasce por `rows::fields_row`, `rows::seg_row` ou
/// `rows::property_label_row` — é uma chamada.
///
/// ⚠️⚠️ **A razão das duas é a MESMA e não é «é difícil»:** nos dois o controlo não é o bloco — é o
/// bloco **mais um companheiro à direita dele**, e o par mede `~144 px`. A coluna do controlo vale
/// `0,5 × interior − 14`, logo só chega a `144` com o painel acima de **`336 px`**; ⛔ o dock do
/// dono está em `273,3`. §0.0: *o limite diz de que recurso é* — é a LARGURA.
const AINDA_POR_CIMA: &[(&str, &str)] = &[
    (
        "slice_grid.rs",
        "grelha 3x3 + os dois atalhos A' DIREITA dela (~144 px) — nao cabe na coluna do controlo \
         abaixo de um painel de 336",
    ),
    (
        "color_tint.rs",
        "4 amostras + a previa do gradiente A' DIREITA delas (~144 px) — a mesma medicao",
    ),
];

fn sections_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sections")
}

/// **O idioma:** avançar o `y` pela altura de um rótulo pintado por cima.
///
/// ⛔⛔⛔ **A 1.ª redacção EXIGIA fronteira de palavra, e isso cegou-a ao caso maior.** Ela recusava
/// `+ label_h_used` por ser *«um nome de variável de OUTRA lei»* — e não era: o `transform_row.rs`
/// escrevia exactamente assim, e **empilhava o nome em toda largura de painel abaixo de `380`**,
/// que é acima do dock do dono. *O censo passou verde sobre a secção que o dono fotografou.*
///
/// ⇒ o prefixo `label_h` **basta**, com ou sem sufixo. ⚠️ O preço é um falso positivo possível num
/// nome que comece igual e não seja um empilhamento; ele custa uma entrada na tolerância, com a
/// razão — contra uma secção inteira por converter, que é o que o rigor demais custou.
///
/// ⏳ **E o limite que FICA, nomeado:** o Transform decidia por um **booleano** (`section_narrow`) e
/// não pelo idioma — uma secção que empilhe por outro caminho e com outro nome continua invisível a
/// uma régua textual. A régua estrutural (medir onde o rect REGISTADO caiu) não é alcançável: as
/// rows destas secções pedem uma selecção que o `MockPanelHost` não produz.
fn empilha_o_nome(src: &str) -> bool {
    src.lines().any(|l| {
        let Some(pos) = l.find("label_h") else {
            return false;
        };
        let antes = l[..pos].trim_end();
        antes.ends_with('+') || antes.ends_with("+=")
    })
}

fn censo() -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for entry in fs::read_dir(sections_dir()).expect("sections/ existe") {
        let path = entry.expect("entrada legível").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let nome = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("nome utf-8")
            .to_string();
        let src = fs::read_to_string(&path).expect("ficheiro legível");
        out.push((nome, empilha_o_nome(&src)));
    }
    out.sort();
    out
}

/// **Ninguém empilha o nome, a não ser os dois blocos declarados.**
///
/// **Mutação que deve sangrar:** repor o `paint_text` + `yy += label_h` em qualquer secção
/// convertida — é literalmente o que o dono fotografou duas vezes.
#[test]
fn no_row_paints_its_name_above_its_control() {
    let censo = censo();
    // ⚠️ **Piso de população, MEDIDO** — `sections/` tinha **35** ficheiros em 2026-09-15. Sem ele,
    //    uma varredura que deixasse de achar o directório devolveria zero acusados e leria-se como
    //    aprovada (`CLAUDE.md` §5.0, a falha MUDA de mover código).
    assert!(
        censo.len() >= 30,
        "a varredura leu {} ficheiro(s) em sections/ — ela deixou de alcancar o directorio",
        censo.len()
    );
    let tolerados: Vec<&str> = AINDA_POR_CIMA.iter().map(|(f, _)| *f).collect();
    let maus: Vec<&String> = censo
        .iter()
        .filter(|(nome, empilha)| *empilha && !tolerados.contains(&nome.as_str()))
        .map(|(nome, _)| nome)
        .collect();
    assert!(
        maus.is_empty(),
        "estas seccoes poem o NOME POR CIMA do controlo (report do dono, 2026-09-14 e 15): \
         {maus:?}\n\
         cura: `rows::fields_row` (N campos numericos) · `rows::seg_row` (segmentado por tabela) \
         · `rows::property_label_row` (controlo construido a' mao)."
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — uma tolerância que já não descreve nada sai da lista.
///
/// ⛔ *Uma catraca sem censo de obsolescência não desce: ela vira LICENÇA* (`CLAUDE.md` §5.0).
#[test]
fn the_tolerated_stacked_labels_still_describe_something() {
    let censo = censo();
    let mut mortas = Vec::new();
    for (ficheiro, razao) in AINDA_POR_CIMA {
        match censo.iter().find(|(n, _)| n == ficheiro) {
            None => mortas.push(format!("{ficheiro} — o ficheiro sumiu ({razao})")),
            Some((_, false)) => mortas.push(format!(
                "{ficheiro} — ja' NAO empilha o nome; apague a entrada ({razao})"
            )),
            Some((_, true)) => {}
        }
    }
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE na tolerancia:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO do detector** — sem ele, um `empilha_o_nome` que respondesse sempre `false`
/// deixaria as duas metades verdes sobre o painel inteiro por converter.
#[test]
fn the_detector_can_see_a_stacked_label() {
    for fonte in [
        "    let row_y = y + label_h;",
        "        yy += label_h;",
        "    cur_y += label_h;",
        "    let grid_y = y + label_h;",
    ] {
        assert!(
            empilha_o_nome(fonte),
            "o detector nao ve o empilhamento em {fonte:?}"
        );
    }
    // ⭐ **E o sufixo também conta** — foi assim que o `transform_row.rs` escapou.
    assert!(
        empilha_o_nome("        + label_h_used"),
        "o detector nao ve o empilhamento escrito com sufixo — foi este o buraco de 2026-09-15"
    );
    // ⛔ E NÃO inventa um onde não há.
    for fonte in [
        "    let row = property_row_columns(x, w, y, ROW_H_PX);",
        "    // o nome POR CIMA ficaria aqui, e nao fica",
        "    let label_h = label_font + Spacing::Xs.px();",
    ] {
        assert!(
            !empilha_o_nome(fonte),
            "o detector inventa um empilhamento em {fonte:?}"
        );
    }
}
