//! ⛔⛔ **O CENSO das rotas que desenham SEM a arte do pincel** (auditoria de 2026-08-30).
//!
//! # O que este gate é, e o que ele NÃO é
//!
//! Ele **não** cura nada. Ele impede a lista de crescer **em silêncio** — que é a única coisa que
//! separa uma fronteira declarada de uma dívida esquecida. *Toda lista de dívida tolerada deste
//! repo declara-se «só encolhe», e nenhuma encolhe sozinha* (`CLAUDE.md` §5.0).
//!
//! # ⭐⭐⭐ E a auditoria acusou o mecanismo ERRADO — medido
//!
//! Ela leu isto como *"quatro sítios a que falta um argumento"*. Não é. A arte de um pincel é
//! indexada pela **forma ANFITRIÃ** (`BrushArts: VecPathId -> Vec<VecPath>`), e estas rotas
//! desenham geometria que **não tem anfitriã nenhuma**:
//!
//! | rota | o que ela desenha | porque o mapa não serve |
//! |---|---|---|
//! | `instance.rs` | instâncias de `geometry_id` | um `geometry_id` **não é** um `VecPathId` |
//! | `lib.rs::draw_path` | os passos VIRTUAIS de um blend | eles não estão na cena |
//! | `stroke_draw.rs` | as CÓPIAS de um pincel | uma cópia não tem pincel próprio — seria recursão |
//!
//! ⇒ curá-las **não é passar um parâmetro**: é a arte viajar **com a geometria** em vez de num mapa
//! ao lado — a mesma espécie de mudança que a `PatternSource` foi. Isso é uma wave com desenho
//! próprio, e a decisão de **o que é a arte de um passo virtual** é de produto.
//!
//! ⚠️ **O overlay do blend é o que mais custa**, e o doc dele já escreve a lei que ele quebra:
//! *"desenhá-lo por um caminho próprio faria a transição divergir do que a mesma forma pareceria
//! como path real"*. Para um traço com pincel, ela **diverge**.

use std::fs;
use std::path::Path;

fn code(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// **Exactamente estas rotas desenham sem arte de pincel — nem mais, nem menos.**
///
/// ⚠️ **As duas metades importam.** *Nem mais*: uma rota nova que nasça sem a arte tem de vir aqui
/// declarar-se, e o autor descobre o mecanismo ao fazê-lo. *Nem menos*: quando uma for curada, a
/// linha correspondente tem de cair — senão o censo vira **licença**, que é o defeito que o
/// `CLAUDE.md` §5.0 nomeia sobre toda catraca deste repo.
#[test]
fn exactly_these_routes_draw_without_the_brush_art() {
    const CENSO: [(&str, &str, &str); 5] = [
        (
            "instance.rs",
            "draw_path_with(path, tess, transform, target, None, None, None);",
            "instancia: o sujeito e' um `geometry_id`, e a arte e' indexada por `VecPathId`",
        ),
        (
            "instance.rs",
            "crate::draw_stroke_with(path, tess, transform, target, None, None);",
            "idem, o ramo do traco",
        ),
        (
            "lib.rs",
            "draw_path_tiled(path, transform, target, None, None, None);",
            "`draw_path`: a porta sem cena, usada pelo overlay do blend (passos VIRTUAIS)",
        ),
        (
            "stroke_draw.rs",
            "crate::draw_path_tiled(&copia, transform, target, None, None, None);",
            "as COPIAS de um pincel: uma copia nao tem pincel proprio, e da-lo seria recursao",
        ),
        // ⚠️⚠️ **ESTA LINHA NASCEU DE UM PONTO CEGO DESTE GATE** (2026-09-05): a lista de ficheiros
        // e' escrita a' mao, entao o `stack_draw.rs` — modulo NOVO — passou verde sem se declarar,
        // e o filtro de contagem tambem nao conhecia o nome da porta nova (`draw_one_stroke`).
        // *Um censo que enumera FICHEIROS nao ve^ o ficheiro que ainda nao existe.*
        (
            "stack_draw.rs",
            "stroke_draw::draw_one_stroke(path, s, tess, transform, target, None, None);",
            "as CAMADAS da pilha de aparencia: a arte e' memoizada pela forma ANFITRIA, e uma \
             camada nao e' uma forma",
        ),
    ];
    for (ficheiro, agulha, porque) in CENSO {
        assert!(
            code(ficheiro).contains(agulha),
            "o censo descreve uma rota que ja' nao existe em `{ficheiro}` ({porque}) - se ela foi \
             CURADA, apague esta linha; um censo que nao encolhe vira licenca"
        );
    }
    for (ficheiro, esperado) in [
        ("instance.rs", 2usize),
        ("lib.rs", 1),
        ("stroke_draw.rs", 1),
        ("stack_draw.rs", 1),
    ] {
        let n = code(ficheiro)
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .filter(|l| {
                (l.contains("draw_path_with(")
                    || l.contains("draw_path_tiled(")
                    || l.contains("draw_stroke_with(")
                    || l.contains("draw_one_stroke("))
                    && l.contains("None)")
            })
            .count();
        assert_eq!(
            n, esperado,
            "`{ficheiro}` passou a ter {n} rotas sem arte de pincel (esperava {esperado}) - uma \
             rota nova nasceu sem a arte, e o artista ve' a cor de recurso onde devia ver o motivo"
        );
    }
}
