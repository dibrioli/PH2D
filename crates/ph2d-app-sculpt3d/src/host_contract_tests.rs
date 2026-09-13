//! ⭐⭐ **O CONTRATO DO EMPRÉSTIMO** — o que a shell promete à família quando lhe EMPRESTA a
//! cena, gateado do lado de quem depende da promessa.
//!
//! # ⛔ Os dois gates daqui eram PROMETIDOS por escrito e NÃO EXISTIAM
//!
//! O cabeçalho do `sculpt3d_host.rs` dizia *«é por isso que ela tem um:
//! `the_host_never_reads_the_borrowed_scene`»*, e o `the_sculpt_gesture_is_wired` dizia que o
//! `the_sculpt_host_refuses_without_a_scene` afirmava a guarda de runtime do lado da shell.
//! Medido em 2026-09-13: **nenhum dos dois tem commit no histórico do git**, com o controlo
//! positivo (a mesma busca sobre um gate vizinho) a devolver o dele. *Uma promessa de gate
//! lê-se exactamente como um gate* — e a primeira era a armadilha §2.10 do HOWTO (o elo novo
//! sem gate) com a nota a dizer que o elo estava coberto.
//!
//! # Por que moram na FAMÍLIA e não nos testes da shell
//!
//! O sujeito é um ficheiro da shell, mas a propriedade é da família: é a escultura que some
//! quando ela falha. E a shell **só desce** (`the_shell_only_shrinks`, a 987 linhas do tecto
//! em 2026-09-13) — um gate novo lá é crescimento da crate que a W2 existiu para encolher.
//!
//! # A régua
//!
//! Censo de FONTE, porque a `App` pede janela, superfície e device, e nenhum teste a constrói.
//! ⚠️ Os ficheiros entram por `include_str!`: se a shell os mudar de sítio este módulo **não
//! compila**, que é a espécie de gate partido que falha alto (`CLAUDE.md` §5.0). E todo corpo é
//! lido **sem comentários nem conteúdo de strings** (HOWTO §2.12): o cabeçalho do host EXPLICA
//! a cura com a palavra `sculpt3d`, e um censo que lesse prosa acusaria a documentação da cura.

use std::collections::BTreeSet;

const APP_HOST: &str = include_str!("../../../shells/desktop/src/app_host.rs");
const HOST_TRAIT: &str = include_str!("../../ph2d-app-host/src/lib.rs");
const CANVAS_AREA: &str = include_str!("../../ph2d-app-host/src/canvas_area.rs");
const CHROME_HIT: &str = include_str!("../../../shells/desktop/src/chrome_hit.rs");
const PALETTE: &str = include_str!("../../../shells/desktop/src/command_palette_input.rs");
const SCULPT_HOST: &str = include_str!("../../../shells/desktop/src/sculpt3d_host.rs");
const SCULPT_ABSENT: &str = include_str!("../../../shells/desktop/src/sculpt3d_absent.rs");

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// O texto com os comentários e o CONTEÚDO de strings e chars apagados. As quebras de linha
/// ficam, e cada string vira `""`, para que a forma do código continue legível.
fn code_only(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let fim = src[i + 2..].find("*/").map_or(b.len(), |r| i + 2 + r + 2);
                out.extend(src[i..fim].chars().filter(|&c| c == '\n'));
                i = fim;
            }
            b'r' if matches!(b.get(i + 1), Some(b'"' | b'#'))
                && (i == 0 || !is_ident(b[i - 1])) =>
            {
                let mut j = i + 1;
                while b.get(j) == Some(&b'#') {
                    j += 1;
                }
                if b.get(j) != Some(&b'"') {
                    out.push('r');
                    i += 1;
                    continue;
                }
                let fecho = format!("\"{}", "#".repeat(j - i - 1));
                i = src[j + 1..]
                    .find(&fecho)
                    .map_or(b.len(), |r| j + 1 + r + fecho.len());
                out.push_str("\"\"");
            }
            b'"' => {
                let mut j = i + 1;
                while j < b.len() && b[j] != b'"' {
                    j += if b[j] == b'\\' { 2 } else { 1 };
                }
                i = j + 1;
                out.push_str("\"\"");
            }
            b'\'' => {
                // Um char (`'x'`, `'\n'`, `'é'`) ou um lifetime (`'a`).
                let resto = &src[i + 1..];
                let fim = if resto.starts_with('\\') {
                    resto.find('\'').map(|r| i + 1 + r + 1)
                } else {
                    resto
                        .chars()
                        .next()
                        .filter(|c| resto[c.len_utf8()..].starts_with('\''))
                        .map(|c| i + 1 + c.len_utf8() + 1)
                };
                match fim {
                    Some(f) => {
                        out.push_str("''");
                        i = f;
                    }
                    None => {
                        out.push('\'');
                        i += 1;
                    }
                }
            }
            _ => {
                let c = src[i..]
                    .chars()
                    .next()
                    .expect("i está numa fronteira de char");
                out.push(c);
                i += c.len_utf8();
            }
        }
    }
    out
}

fn squeeze(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// O `}` que fecha o `{` em `abre` — o texto já não tem comentários nem strings.
fn fecha(code: &str, abre: usize) -> Option<usize> {
    let mut fundo = 0i32;
    for (k, c) in code[abre..].bytes().enumerate() {
        match c {
            b'{' => fundo += 1,
            b'}' => {
                fundo -= 1;
                if fundo == 0 {
                    return Some(abre + k);
                }
            }
            _ => {}
        }
    }
    None
}

/// Onde começa `fn name` (fronteira dos dois lados) e onde abre o corpo dela, se tiver.
fn localiza(code: &str, name: &str) -> Option<(usize, Option<usize>)> {
    let agulha = format!("fn {name}");
    let b = code.as_bytes();
    let mut de = 0;
    while let Some(r) = code[de..].find(&agulha) {
        let em = de + r;
        let depois = em + agulha.len();
        if (em == 0 || !is_ident(b[em - 1])) && b.get(depois).is_some_and(|&c| !is_ident(c)) {
            let (mut par, mut col) = (0i32, 0i32);
            for (k, c) in code[depois..].bytes().enumerate() {
                match c {
                    b'(' => par += 1,
                    b')' => par -= 1,
                    b'[' => col += 1,
                    b']' => col -= 1,
                    b'{' if par == 0 && col == 0 => return Some((em, Some(depois + k))),
                    b';' if par == 0 && col == 0 => return Some((em, None)),
                    _ => {}
                }
            }
            return None;
        }
        de = depois;
    }
    None
}

fn fn_body<'a>(code: &'a str, name: &str) -> Option<&'a str> {
    let (_, abre) = localiza(code, name)?;
    let abre = abre?;
    Some(&code[abre + 1..fecha(code, abre)?])
}

fn signature<'a>(code: &'a str, name: &str) -> Option<&'a str> {
    let (em, abre) = localiza(code, name)?;
    Some(&code[em..abre?])
}

/// O miolo do primeiro bloco `{…}` depois de `anchor`.
fn block<'a>(code: &'a str, anchor: &str) -> Option<&'a str> {
    let abre = code.find(anchor)? + code[code.find(anchor)?..].find('{')?;
    Some(&code[abre + 1..fecha(code, abre)?])
}

/// Os nomes de toda `fn` do texto, pela ordem.
fn fn_names(code: &str) -> Vec<String> {
    let b = code.as_bytes();
    let mut out = Vec::new();
    let mut de = 0;
    while let Some(r) = code[de..].find("fn ") {
        let em = de + r;
        de = em + 3;
        if em > 0 && is_ident(b[em - 1]) {
            continue;
        }
        let nome: String = code[de..]
            .bytes()
            .take_while(|&c| is_ident(c))
            .map(char::from)
            .collect();
        if !nome.is_empty() {
            out.push(nome);
        }
    }
    out
}

/// O miolo (squeezed) de cada `else{…}` de um corpo já squeezed.
fn else_blocks(squeezed: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut de = 0;
    while let Some(r) = squeezed[de..].find("else{") {
        let abre = de + r + "else".len();
        let fim = fecha(squeezed, abre).expect("um `else{` fecha");
        out.push(squeezed[abre + 1..fim].to_string());
        de = fim;
    }
    out
}

/// Os métodos chamados sobre `self` — `self.nome(`.
fn self_calls(squeezed: &str) -> Vec<String> {
    squeezed
        .match_indices("self.")
        .filter_map(|(i, _)| {
            let nome: String = squeezed[i + 5..]
                .bytes()
                .take_while(|&c| is_ident(c))
                .map(char::from)
                .collect();
            squeezed[i + 5 + nome.len()..]
                .starts_with('(')
                .then_some(nome)
        })
        .collect()
}

/// As chamadas por CAMINHO — `a::b(` —, que é por onde uma porta alcança uma função livre.
fn path_calls(squeezed: &str) -> Vec<String> {
    squeezed
        .match_indices('(')
        .filter_map(|(i, _)| {
            let inicio = squeezed[..i]
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
                .map_or(0, |k| k + 1);
            let caminho = &squeezed[inicio..i];
            caminho.contains("::").then(|| caminho.to_string())
        })
        .collect()
}

/// **As seis portas do `AppHost`** — o trait e a shell têm de ter exactamente estas.
const PORTAS: [&str; 6] = [
    "pointer",
    "mods",
    "pointer_over_chrome",
    "modal_takes_the_pointer",
    "note_authored_change",
    "canvas_visible",
];

/// `(porta, a leitura do AppGfx que ela tem LICENÇA de fazer)`, squeezed. ⚠️ As duas leem o
/// `hero_screen` — o índice de acerto e a área publicada —, e nunca a cena.
const LICENCAS: [(&str, &str); 2] = [
    (
        "pointer_over_chrome",
        "crate::chrome_hit::pointer_over_chrome(self.gfx.as_ref(),",
    ),
    (
        "canvas_visible",
        "self.gfx.as_ref().and_then(|g|g.hero_screen.as_ref())",
    ),
];

/// `(porta, a chamada, a fonte do chamado, o nome dele, a leitura licenciada lá dentro)`.
///
/// ⚠️ **É a rota que o gate SEGUE**: uma porta que delega a outra função é tão segura quanto
/// essa função, e o censo lê as duas.
const ROTAS: [(&str, &str, &str, &str, &str); 3] = [
    (
        "pointer_over_chrome",
        "crate::chrome_hit::pointer_over_chrome",
        CHROME_HIT,
        "pointer_over_chrome",
        "gfx.and_then(|g|g.hero_screen.as_ref())",
    ),
    (
        "modal_takes_the_pointer",
        "command_palette_open",
        PALETTE,
        "command_palette_open",
        "self.gfx.as_ref().and_then(|g|g.hero_screen.as_ref())",
    ),
    (
        "canvas_visible",
        "ph2d_app_host::canvas_area::visible",
        CANVAS_AREA,
        "visible",
        "",
    ),
];

/// ⭐⭐ **NENHUMA PORTA DO HOST LÊ A CENA EMPRESTADA** — nem directamente, nem pela função a
/// que delega.
///
/// O `sculpt3d_pointer_down` e o `sculpt3d_wheel` tiram a cena do `AppGfx` (`take`) e entregam
/// à família `&mut App` como `impl AppHost`. Se uma porta — ou a função para onde ela roteia —
/// passasse a consultar `gfx.sculpt3d`, ela veria `None` **durante o gesto**, e o sintoma seria
/// a escultura a desaparecer só enquanto o dedo está em baixo.
///
/// ⚠️ **O censo é por LICENÇA, não por proibição de uma palavra**: cada leitura do `gfx` e cada
/// chamada que uma porta faz tem de estar nas tabelas acima. Uma sétima porta, um `self.helper()`
/// novo ou um terceiro caminho até ao `AppGfx` reprovam aqui e mandam alguém ler o que eles
/// leem — que é a pergunta que uma lista de palavras proibidas nunca faria.
///
/// **Mutações que sangram:** um `|| self.gfx.as_ref().is_some_and(|g| g.sculpt3d.is_some())` no
/// `modal_takes_the_pointer` · o `pointer_over_chrome` a rotear para outra função · uma leitura
/// de `g.sculpt3d` dentro do `chrome_hit::pointer_over_chrome` · um método novo no trait.
#[test]
fn the_host_never_reads_the_borrowed_scene() {
    let host = code_only(APP_HOST);
    let imp = block(&host, "impl AppHost for App")
        .expect("a shell deixou de implementar o `AppHost` em `app_host.rs`");
    let trait_code = code_only(HOST_TRAIT);
    let tr = block(&trait_code, "pub trait AppHost").expect("o trait `AppHost` sumiu");
    let esperadas: BTreeSet<String> = PORTAS.iter().map(|p| (*p).to_string()).collect();
    assert_eq!(
        fn_names(tr).into_iter().collect::<BTreeSet<_>>(),
        esperadas,
        "o trait `AppHost` mudou de portas — leia a porta nova (e a rota dela) e diga aqui se \
         ela pode ler o `AppGfx.sculpt3d` enquanto a cena está emprestada"
    );
    assert_eq!(
        fn_names(imp).into_iter().collect::<BTreeSet<_>>(),
        esperadas,
        "a implementação da shell não responde exactamente às portas do trait"
    );

    for porta in PORTAS {
        let corpo = squeeze(
            fn_body(imp, porta).unwrap_or_else(|| panic!("a porta `{porta}` não tem corpo")),
        );
        assert!(
            !corpo.is_empty(),
            "controlo positivo: o corpo de `{porta}` veio VAZIO — as ausências abaixo seriam \
             verdadeiras por construção"
        );
        let mut resto = corpo.clone();
        for (p, licenca) in LICENCAS {
            if p == porta {
                assert!(
                    resto.contains(licenca),
                    "a leitura licenciada de `{porta}` mudou de forma — releia-a e reescreva a \
                     licença: `{licenca}`"
                );
                resto = resto.replace(licenca, "");
            }
        }
        assert!(
            !resto.contains("sculpt3d") && !resto.contains("gfx"),
            "`{porta}` passou a ler o `AppGfx` fora da licença — com a cena emprestada ela veria \
             `None` a meio do gesto: `{resto}`"
        );
        let rotas: Vec<_> = ROTAS.iter().filter(|r| r.0 == porta).collect();
        for chamada in self_calls(&resto) {
            assert!(
                rotas.iter().any(|r| r.1 == chamada),
                "`{porta}` passou a chamar `self.{chamada}(` — um método da `App` pode ler a cena \
                 emprestada, e este censo não o leu"
            );
        }
        for chamada in path_calls(&resto) {
            assert!(
                rotas.iter().any(|r| r.1 == chamada),
                "`{porta}` passou a chamar `{chamada}(` — ponha-o nas ROTAS depois de o ler"
            );
        }
        for (_, chamada, fonte, nome, licenca) in rotas {
            assert!(
                corpo.contains(chamada),
                "`{porta}` deixou de rotear para `{chamada}` — a rota que este gate lê já não é a \
                 que corre"
            );
            let alvo = squeeze(
                fn_body(&code_only(fonte), nome)
                    .unwrap_or_else(|| panic!("a rota `{nome}` de `{porta}` sumiu da fonte")),
            );
            assert!(!alvo.is_empty(), "controlo positivo: `{nome}` veio vazio");
            let fora = if licenca.is_empty() {
                alvo
            } else {
                assert!(
                    alvo.contains(licenca),
                    "a leitura licenciada de `{nome}` mudou de forma: `{licenca}`"
                );
                alvo.replace(licenca, "")
            };
            assert!(
                !fora.contains("sculpt3d") && !fora.contains("gfx") && !fora.contains("self."),
                "`{nome}` (a rota de `{porta}`) passou a alcançar o `AppGfx` por outro caminho: \
                 `{fora}`"
            );
        }
    }

    // ⛔ Controlo positivo da LENTE nos dois sentidos: ela vê a palavra quando é código, e não a
    // vê quando é prosa — o cabeçalho do host explica esta propriedade com `gfx.sculpt3d`.
    let host_code = code_only(SCULPT_HOST);
    let emprestimo =
        squeeze(fn_body(&host_code, "com_a_cena_emprestada").expect("o empréstimo mudou de nome"));
    assert!(
        emprestimo.contains("sculpt3d.take()"),
        "controlo positivo: a lente deixou de ver código — todo `!contains` acima seria vácuo"
    );
    assert!(
        SCULPT_HOST.contains("lê `gfx.sculpt3d`") && !host_code.contains("lê `gfx.sculpt3d`"),
        "controlo: a prosa do cabeçalho do host já não se lê como antes, ou a lente passou a \
         deixar comentários no texto"
    );
}

/// **Os invólucros que devolvem `bool`** — o que o dispatch lê como *«a escultura tomou isto»*.
const DEVOLVEM_BOOL: [&str; 5] = [
    "sculpt3d_wheel",
    "sculpt3d_pointer_down",
    "com_a_cena_emprestada",
    "sculpt3d_key",
    "sculpt3d_quad_key",
];

/// ⭐⭐ **SEM CENA, A SHELL RECUSA** — todo invólucro que diz ao dispatch se tomou o evento
/// responde `false` quando não há escultura, e o empréstimo devolve a cena sem caminho de fuga.
///
/// É a metade da inércia do módulo que mudou de dono na W2/L3-B: as portas da família passaram
/// a RECEBER a cena (inertes pelo tipo), e a guarda de runtime subiu para estes invólucros. Um
/// `else { return true; }` num deles faria uma sessão SEM escultura engolir o clique, a roda ou
/// a tecla do app inteiro — e o `every_3d_port_is_inert_without_a_scene` ficaria verde, porque
/// ele mede a assinatura da família.
///
/// ⚠️ **E o gémeo neutro (`sculpt3d_absent.rs`) é a mesma lei sem a feature**, lido aqui pelo
/// mesmo censo: *as duas metades da inércia são uma lei escrita nos dois lados do `cfg`.*
///
/// **Mutações que sangram:** `return false` → `return true` num `else` · um `return` entre o
/// `take` e a devolução · o gémeo a responder `true`.
#[test]
fn the_sculpt_host_refuses_without_a_scene() {
    let host = code_only(SCULPT_HOST);
    let bools: BTreeSet<String> = fn_names(&host)
        .into_iter()
        .filter(|n| signature(&host, n).is_some_and(|s| squeeze(s).ends_with("->bool")))
        .collect();
    assert_eq!(
        bools,
        DEVOLVEM_BOOL.iter().map(|p| (*p).to_string()).collect(),
        "os invólucros que devolvem `bool` mudaram — um novo tem de dizer o que responde sem cena"
    );
    for nome in DEVOLVEM_BOOL {
        let corpo = squeeze(fn_body(&host, nome).expect("listado acima"));
        assert!(
            !corpo.contains("returntrue"),
            "`{nome}` passou a responder `true` por conta própria — sem escultura ele engoliria o \
             evento do app"
        );
        let elses = else_blocks(&corpo);
        for e in &elses {
            assert_eq!(
                e, "returnfalse;",
                "um `else` de `{nome}` não é a recusa neutra"
            );
        }
        assert!(
            corpo.contains("self.com_a_cena_emprestada(") || !elses.is_empty(),
            "`{nome}` devolve `bool` sem guarda de cena e sem passar pelo empréstimo"
        );
    }

    let emp = squeeze(fn_body(&host, "com_a_cena_emprestada").expect("listado acima"));
    let take = "sculpt3d.take())else{returnfalse;};";
    let devolve = ".sculpt3d=Some(scene);";
    let depois_do_take = emp
        .find(take)
        .map(|i| i + take.len())
        .expect("o empréstimo deixou de recusar a `false` sem cena");
    let devolucao = emp
        .find(devolve)
        .expect("o empréstimo deixou de devolver a cena");
    assert!(
        depois_do_take <= devolucao,
        "a devolução veio antes do `take`"
    );
    let entre = &emp[depois_do_take..devolucao];
    assert!(
        entre.contains("gesto("),
        "o gesto deixou de correr ENTRE o `take` e a devolução"
    );
    assert!(
        !entre.contains("return") && !entre.contains('?'),
        "há um caminho de fuga entre o `take` e a devolução — a escultura some no primeiro \
         deles: `{entre}`"
    );

    let cursor = squeeze(fn_body(&host, "sculpt3d_seam_cursor").expect("o cursor da costura"));
    assert!(
        cursor.contains(".sculpt3d.as_ref()?"),
        "o cursor da costura deixou de responder `None` sem cena"
    );

    let ausente = code_only(SCULPT_ABSENT);
    assert_eq!(
        fn_names(&ausente).into_iter().collect::<BTreeSet<_>>(),
        ["sculpt3d_keys_live", "sculpt3d_seam_cursor"]
            .iter()
            .map(|p| (*p).to_string())
            .collect(),
        "o gémeo neutro mudou de forma"
    );
    for (nome, neutro) in [
        ("sculpt3d_keys_live", "false"),
        ("sculpt3d_seam_cursor", "None"),
    ] {
        assert_eq!(
            squeeze(fn_body(&ausente, nome).expect("listado acima")),
            neutro,
            "sem a feature, `{nome}` deixou de ser a resposta neutra"
        );
    }
}
