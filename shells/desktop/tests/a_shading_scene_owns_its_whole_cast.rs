//! **Uma cena de sombreamento é dona do ELENCO INTEIRO.**
//!
//! O `smoke_mesh()` devolve a peça **primária** e o `scene_objects()` só acrescenta **extras**. Uma
//! cena que declarasse todo o elenco em `scene_objects` abre, portanto, com a esfera lisa de 96×144
//! por cima — não convidada, e no meio da composição.
//!
//! ⚠️ **Isto aconteceu, e a aritmética é o que o mostrou:** na `=19` a bola de raio `0,45` mora em
//! `x = 0,35`, e `0,35 + 0,45 = 0,80 < 1,0` — ela ficava **inteiramente enterrada** dentro da
//! primária acidental; a de raio `1,0` em `x = −1,7` se fundia com ela. O roteiro chamava a escada
//! de *"o oráculo"* e um dos três degraus era invisível. Nada reclama: a cena abre, desenha, e o
//! artista julga um canal sobre uma composição que ninguém desenhou.
//!
//! O invariante que fecha a classe: **as duas portas respondem juntas**. Se uma cena tem elenco
//! próprio (`scene_objects` diz `Some`), ela também nomeia a peça com que abre (`primary_mesh` diz
//! `Some`) — e vice-versa.

use std::fs;

const SHADING: &str = "src/sculpt3d_scenes_shading.rs";

/// As cenas que o módulo de sombreamento declara, pelo literal que cada uma compara.
fn scenes(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        let Some(rest) = line.split("PH2D_SCULPT3D_SMOKE").nth(1) else {
            continue;
        };
        if let Some(after) = rest.split("Some(\"").nth(1) {
            let n: String = after.chars().take_while(char::is_ascii_digit).collect();
            if !n.is_empty() {
                out.push(n);
            }
        }
    }
    out
}

/// As cenas que a porta `name` ATENDE, lidas como TEXTO.
///
/// ⚠️ **É texto porque a ENTRADA destas portas é uma variável de ambiente**, e um
/// teste que a setasse mexeria em estado global de um binário com milhares de
/// testes em paralelo. O que sobra é ler o fonte — e ler o CORPO da porta é
/// afirmar a propriedade (*ela consulta o predicado desta cena*), não um
/// endereço.
fn door(src: &str, all: &[String], name: &str) -> Vec<String> {
    let at = src
        .find(&format!("pub(crate) fn {name}("))
        .unwrap_or_else(|| panic!("a porta `{name}` tem de existir"));
    let body = &src[at..];
    let end = body.find("\n}\n").expect("ela fecha");
    let body = &body[..end];
    all.iter()
        .filter(|n| body.contains(&format!("{}()", predicate_of(src, n))))
        .cloned()
        .collect()
}

#[test]
fn a_shading_scene_owns_its_whole_cast() {
    let src = fs::read_to_string(SHADING).expect("o modulo de cenas de sombreamento existe");
    let all = scenes(&src);
    // **Controle positivo:** sem cenas a varredura passaria por vácuo.
    assert!(
        all.len() >= 3,
        "achei {} cenas de sombreamento — a forma de declara-las mudou e o gate mede nada",
        all.len()
    );

    let primary = door(&src, &all, "primary_mesh");
    let objects = door(&src, &all, "scene_objects");
    assert!(
        !primary.is_empty(),
        "controle: alguma cena tem de nomear a propria peca primaria"
    );
    assert_eq!(
        primary, objects,
        "estas portas discordam sobre QUEM tem elenco proprio — uma cena que declara pecas no \
         `scene_objects` sem nomear a primaria abre com a esfera padrao por cima, nao convidada, \
         e foi assim que a `=19` enterrou uma das tres bolas da escada"
    );
}

/// **E O CONSUMIDOR TEM DE PERGUNTAR**, senão a porta existe e ninguém a atravessa.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO que sobreviveu ao irmão acima:** aquele prova que as duas
/// portas do módulo de sombreamento concordam sobre quem tem elenco próprio — e fica VERDE se o
/// `smoke_mesh` parar de chamar a `primary_mesh`, porque ele não lê o consumidor. É a falha de
/// *capacidade sem porta*: a peça certa é construída, ninguém a pede, e a cena volta a abrir com a
/// esfera padrão por cima.
///
/// ⚠️ **E a ORDEM é load-bearing:** a pergunta tem de vir ANTES do fallback de 96×144. Depois dele
/// ela é inalcançável, e o modo de falha é exactamente o que esta wave consertou.
#[test]
fn the_primary_mesh_door_is_actually_asked_before_the_fallback() {
    let src = fs::read_to_string("src/sculpt3d_scenes.rs").expect("o modulo de cenas existe");
    let at = src
        .find("pub(crate) fn smoke_mesh(")
        .expect("controle: a porta da peca primaria existe");
    let body = &src[at..];
    let end = body.find("\n}\n").expect("controle: ela fecha");
    let body = &body[..end];

    let asks = body
        .find("shading::primary_mesh()")
        .expect("`smoke_mesh` tem de PERGUNTAR pela peca primaria da cena de sombreamento");
    let fallback = body
        .find("uv_sphere(96, 144")
        .expect("controle: o fallback padrao existe");
    assert!(
        asks < fallback,
        "a pergunta pela peca primaria vem DEPOIS do fallback de 96x144, entao ela e' \
         inalcancavel — a cena volta a abrir com a esfera padrao por cima do elenco"
    );
}

/// O nome da função-predicado de uma cena (`= Some("19")` → `sss_scene`).
fn predicate_of(src: &str, level: &str) -> String {
    let needle = format!("== Some(\"{level}\")");
    let at = src.find(&needle).expect("a cena existe");
    // sobe até o `pub(crate) fn <nome>(` imediatamente anterior
    let head = &src[..at];
    let start = head
        .rfind("pub(crate) fn ")
        .expect("toda cena e' uma funcao");
    let after = &head[start + "pub(crate) fn ".len()..];
    after
        .split('(')
        .next()
        .expect("o nome termina no parentese")
        .trim()
        .to_string()
}

/// **A luz que a cena traz CHEGA à sessão.**
///
/// ⚠️ **Um gate de unidade é cego a isto, e a `line/anim` já o pagou:** os testes
/// do módulo de cenas provam que `scene_rig()` devolve o rig certo, e ficariam
/// *todos verdes* com o construtor da sessão ignorando a porta e escrevendo
/// `LightRig::default()` — a cena seria dona de uma luz que ninguém acende.
///
/// Ele afirma a PROPRIEDADE (*o campo `rig` nasce da porta*), nunca o endereço:
/// o construtor pode mudar de arquivo, de ordem ou de vizinhos sem envelhecer
/// esta asserção. O que ele **não** pode é voltar a semear a luz por conta.
#[test]
fn the_scene_rig_reaches_the_session() {
    const SESSION: &str = "src/sculpt3d.rs";
    let src = fs::read_to_string(SESSION).expect("o módulo da sessão existe");

    // ⚠️ **Um SEED é uma chamada; uma DECLARAÇÃO de campo não é** — e a 1ª
    // versão deste gate ancorou na declaração (`rig: LightRig,`) e reprovou
    // produto correto. O filtro é essa diferença, e não uma posição no arquivo.
    //
    // ⚠️ **E ele aceita o campo semeado por um BINDING** (`let rig = …;` +
    // `rig,`), que é como o construtor passou a fazer quando a testemunha do rig
    // nasceu: o carimbo e o rig TÊM de sair da mesma expressão, senão a
    // testemunha nasce discordando do que ela testemunha. A forma sintática do
    // seed é um endereço; o que este gate afirma é *de onde a luz vem*.
    let seeds: Vec<&str> = src
        .lines()
        .map(str::trim)
        .filter(|l| (l.starts_with("rig: ") || l.starts_with("let rig = ")) && l.contains('('))
        .collect();

    // **Controle positivo:** se o campo sumir ou for renomeado, isto falha ALTO
    // em vez de varrer o vazio — o modo de falha que o `project_tokens::install`
    // da `line/Vector` sofreu quando o dono se mudou.
    assert_eq!(
        seeds.len(),
        1,
        "o construtor da sessão semeia o campo `rig` exatamente uma vez; achei {:?}",
        seeds
    );
    let seed = seeds[0];

    assert!(
        seed.contains("scene_rig()"),
        "o campo `rig` tem de nascer da porta da cena; hoje ele diz `{}`",
        seed
    );
    assert!(
        seed.contains("unwrap_or_default"),
        "sem cena armada a luz é a de todo dia — a porta devolve `None` e o \
         default é quem responde; hoje: `{}`",
        seed
    );
}

/// **A cena que traz luz própria é uma cena, não um acidente.**
///
/// ⚠️ **Este é o gate que faltava, e a mutação o nomeou:** com `scene_rig`
/// devolvendo `None` para todo mundo, os dois gates de unidade do módulo ficam
/// VERDES — um testa o rig direto (que continua horizontal) e o outro testa o
/// caso *sem cena armada* (que continua `None`). Nenhum dos dois pergunta se a
/// porta de fato **atende** alguém.
#[test]
fn the_light_door_answers_the_scene_that_needs_it() {
    let src = fs::read_to_string(SHADING).expect("o modulo de cenas de sombreamento existe");
    let all = scenes(&src);
    let lit = door(&src, &all, "scene_rig");

    // **Controle positivo.** Uma porta que não atende ninguém é uma porta morta.
    assert!(
        !lit.is_empty(),
        "a porta `scene_rig` tem de atender alguma cena — hoje ela nao consulta \
         nenhum predicado, e a cena abre sob a luz de todo dia em silencio"
    );

    // Toda cena que traz luz própria também é dona do elenco: as duas metades
    // descrevem a MESMA composição, e uma cena que escolhesse a luz sem escolher
    // as peças estaria acendendo a esfera padrão de outra pessoa.
    let cast = door(&src, &all, "primary_mesh");
    for n in &lit {
        assert!(
            cast.contains(n),
            "a cena `={n}` traz luz propria mas nao nomeia a propria peca"
        );
    }
}
