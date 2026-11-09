//! ⭐ **As ferramentas de imagem agem na folha VISÍVEL** (dono 08/11), censo em
//! `docs/GameEngineFuncional/27_ferramentas_na_folha_visivel.md`. A lei (ler, gravar, onde o objeto fica) é de
//! `ph2d-anim-sheets::shown_edit` e tem os testes dela; este portão prende a FIAÇÃO de cada verbo, que é onde o
//! defeito morava (todos liam e gravavam a folha `0`).

use std::fs;
use std::path::Path;

fn src(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {p:?}: {e}"))
}

/// Os verbos: cada ficheiro de `hero_intents/image_edit/` que escreve píxeis.
const VERBOS: [&str; 9] = [
    "trim_transparency",
    "make_square",
    "padding",
    "upscale",
    "rasterize",
    "bgremoval",
    "color_equalization",
    "equalize_sizes",
    "painter",
];

/// Quem só LÊ para mostrar ao artista a mesma folha em que o verbo vai gravar (a prévia) ou para copiar a imagem
/// que ele vê.
const LEITORES: [&str; 6] = [
    "render_loop/bgremoval_preview.rs",
    "render_loop/upscale_bridge.rs",
    "render_loop/color_equalization_bridge.rs",
    "render_loop/fase_send_to_board.rs",
    "render_loop/fase_use_as_brush.rs",
    "render_loop/fase_use_as_paper.rs",
];

fn verbo(nome: &str) -> String {
    src(&format!("hero_intents/image_edit/{nome}.rs"))
}

/// A lista é a pasta inteira: um verbo novo que nasça sem entrar aqui reprova. *Mutação: tirar um nome ⇒ vermelho.*
#[test]
fn a_lista_dos_verbos_e_a_pasta_inteira() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/hero_intents/image_edit");
    let mut na_pasta: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(".rs").map(str::to_owned))
        .filter(|n| n != "mod" && n != "undo")
        .collect();
    na_pasta.sort();
    let mut lista: Vec<String> = VERBOS.iter().map(|s| (*s).to_owned()).collect();
    lista.sort();
    assert_eq!(na_pasta, lista);
}

/// ⭐ Cada verbo e cada prévia lê pelo leitor da folha visível, nunca pelo da folha `0`. *Mutação: voltar um verbo
/// ao `read_sprite_source` ⇒ vermelho com o nome dele.*
#[test]
fn cada_verbo_le_a_folha_visivel() {
    for nome in VERBOS {
        let s = verbo(nome);
        assert!(!s.contains("read_sprite_source("), "`{nome}` lê a folha 0");
        assert!(s.contains("read_shown_source("), "`{nome}` não lê nada");
    }
    for rel in LEITORES {
        let s = src(rel);
        assert!(!s.contains("read_sprite_source("), "`{rel}` lê a folha 0");
        assert!(s.contains("read_shown_source("), "`{rel}` não lê nada");
    }
}

/// Os argumentos de cada chamada a `call` em `s`, até ao `)` que a fecha.
fn chamadas<'a>(s: &'a str, call: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(j) = s[i..].find(call) {
        let a = i + j + call.len();
        let mut fundo = 1;
        let mut k = a;
        for (n, c) in s[a..].char_indices() {
            match c {
                '(' => fundo += 1,
                ')' => fundo -= 1,
                _ => {}
            }
            if fundo == 0 {
                k = a + n;
                break;
            }
        }
        out.push(&s[a..k]);
        i = k;
    }
    out
}

/// ⭐ Cada gravação leva a folha que foi LIDA (o 2.º argumento), e na folha extra quem anda é o Pivot dela, não o
/// objeto. *Mutações: passar `0` num verbo ⇒ vermelho; voltar a escrever o `Transform` no Trim ⇒ vermelho.*
#[test]
fn cada_verbo_grava_na_folha_que_leu() {
    for nome in VERBOS {
        let s = verbo(nome);
        let mut n = 0;
        for call in ["commit_edited_texture(", "commit_geometric_edit("] {
            for args in chamadas(&s, call) {
                let segundo = args.split(',').nth(1).unwrap_or("").trim();
                assert!(
                    segundo.ends_with("sheet"),
                    "`{nome}`: {call} com `{segundo}`"
                );
                n += 1;
            }
        }
        assert!(n > 0, "`{nome}` não grava");
    }
    for nome in ["trim_transparency", "make_square", "padding", "bgremoval"] {
        let s = verbo(nome);
        assert!(
            !s.contains("get_mut::<ph2d_ecs::Transform>(entity)"),
            "`{nome}` move o objeto: numa folha extra é o Pivot dela que anda"
        );
        assert!(
            s.contains("shown_edit::place("),
            "`{nome}` não diz onde fica"
        );
    }
}

/// ⭐ O desfazer de imagem antigo só sabe da folha `0` (devolveria o tamanho da folha extra à sprite): cada entrada
/// dele está atrás de `sheet == 0`; a folha extra desfaz-se pelo documento. *Mutação: tirar um `if` ⇒ vermelho.*
#[test]
fn so_a_folha_zero_usa_o_desfazer_de_imagem() {
    for nome in VERBOS {
        let s = verbo(nome);
        let linhas: Vec<&str> = s.lines().collect();
        for (i, l) in linhas.iter().enumerate() {
            if l.contains("pending_undo_entries.push(ImageEditSnapshot") {
                let antes = linhas[..i]
                    .iter()
                    .rev()
                    .find(|x| !x.trim().is_empty())
                    .unwrap();
                assert!(
                    antes.trim_start().starts_with("if ") && antes.contains("sheet == 0"),
                    "`{nome}` linha {}: `{}`",
                    i + 1,
                    antes.trim()
                );
            }
        }
    }
}

/// ⭐ Auditoria do fecho 08/11 (A): o `Transform` é de todas as folhas. O Rasterize (assa a escala e a rotação do
/// OBJETO) recusa numa folha extra; o Upscale só muda a densidade dela e só a folha 0 repõe a escala. *Mutações: tirar
/// a recusa ou o `src.sheet == 0` ⇒ vermelho.*
#[test]
fn so_a_folha_zero_mexe_no_transform_do_objeto() {
    let r = verbo("rasterize");
    assert!(r.contains("if src.sheet != 0") && r.contains("shell.rasterize.first_sheet_only"));
    let u = verbo("upscale");
    assert!(
        u.contains("shown_edit::density_size("),
        "o tamanho da folha extra cresce"
    );
    assert!(
        u.contains("if src.sheet == 0\n                && let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>"),
        "o Upscale repõe a escala do objeto numa folha extra"
    );
}
