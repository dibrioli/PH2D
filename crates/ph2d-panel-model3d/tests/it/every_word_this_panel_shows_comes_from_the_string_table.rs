//! ⭐⭐⭐ **NENHUMA PALAVRA DO PAINEL MODEL 3D É ESCRITA NO FONTE** — o HR-15, por crate.
//!
//! ⛔⛔ **Ele faltava, e a ausência não se lê:** medido em 2026-09-19, este era um de **10 painéis
//! de 29** sem este gate. O censo de literais lia `0` — *o painel estava limpo* —, e é exactamente
//! esse o estado em que um gate de censo custa nada e vale tudo: sem ele, o primeiro texto escrito
//! no fonte entra sem uma linha vermelha, e o segundo já é uma migração.
//!
//! ⚠️ **O corpo do gate é o [`ph2d_label_census::gate`]** — aqui ficam só a LISTA desta crate
//! (prefixo, tabelas, excepções) e as mensagens. *A régua não pode ter N cópias.*

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.model3d.";

/// ⚠️ **QUATRO tabelas e não uma:** o vocabulário do painel foi cortado por ASSUNTO quando o
/// `model3d.rs` passou o tecto de LOC — os nomes das coisas, as RAZÕES de uma fileira travada, a
/// APRESENTAÇÃO da cena (o olhar, a exposição, o estilo) e o BRILHO. *Um censo que leia só a
/// primeira acusa as chaves das outras como «sem tradução».*
///
/// ⛔⛔ **E ela deixou de ser escrita à MÃO em 2026-09-20, porque a quarta apanhou-a.** A lista
/// nomeava três ficheiros, a `W7` do brilho criou o `model3d_bloom.rs`, o `lib.rs` do `ph2d-i18n`
/// **declarou-o e encadeou-o no `tr`** — e este censo continuou a ler três: `174` declaradas contra
/// `182` usadas, com **`19`** chaves do brilho acusadas de não ter tradução **tendo-a**.
/// *Uma lista escrita à mão ao lado de uma realidade derivada são duas respostas à mesma pergunta,
/// e a que envelhece é sempre a escrita à mão.*
///
/// ⇒ a lista sai do **`lib.rs` que as declara**, que é o mesmo sítio de onde o `tr` as encadeia:
/// uma tabela nova entra neste censo **no commit em que nasce**, sem ninguém se lembrar dela.
/// ⚠️ Com **piso de população** — uma varredura partida devolve zero ficheiros, e um censo sobre
/// zero tabelas lê **toda** chave como «sem tradução», que se lê exactamente como esta falha e não
/// é ela.
fn tabelas(repo: &std::path::Path) -> Vec<String> {
    let lib = std::fs::read_to_string(repo.join("crates/ph2d-i18n/src/lib.rs"))
        .expect("o `lib.rs` do ph2d-i18n — é ele que declara as tabelas");
    let achadas: Vec<String> = lib
        .lines()
        .filter_map(|l| l.trim().strip_prefix("mod ")?.strip_suffix(';'))
        .filter(|m| *m == "model3d" || m.starts_with("model3d_"))
        .map(|m| format!("crates/ph2d-i18n/src/{m}.rs"))
        .collect();
    assert!(
        achadas.len() >= 4,
        "a colheita das tabelas devolveu {} — ela partiu-se, e um censo sobre zero tabelas acusa \
         TODA chave de não ter tradução",
        achadas.len()
    );
    for t in &achadas {
        assert!(
            repo.join(t).is_file(),
            "o `lib.rs` declara `{t}` e o ficheiro não existe"
        );
    }
    achadas
}

/// ⭐ As excepções, **com o mecanismo**.
///
/// ✅ **A do `Panel::TITLE` MORREU na integração de 2026-09-20, e foi a metade da obsolescência que
/// a apanhou.** Ela foi escrita contra a árvore desta linha, onde aquela `const` ainda era um
/// `&'static str` com `"Model 3D"` dentro; no `main` de hoje ela é um `TextKey`
/// (`panel.model3d.title`) e quem pinta a aba traduz — logo o literal saiu do binário, não só do
/// alcance deste censo. *O gate irmão do painel autorado registou exactamente esta remoção em
/// 2026-09-17, três dias antes de esta linha a herdar.*
///
/// ⚠️ **Ela não foi apagada por incómodo: foi o `excecoes_mortas` a pedi-lo pelo nome** — uma
/// isenção que já não abriga literal nenhum é a licença de que o §5.0 avisa, e mantê-la deixaria
/// este censo cego ao dia em que alguém escrevesse `"Model 3D"` outra vez no fonte.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte do painel e nunca chegam à tabela \
         de strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<secção>.<nome>` numa das tabelas \
         ({:?}) e um `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`).",
        intrusos.join("\n  "),
        tabelas(&gate::raizes(env!("CARGO_MANIFEST_DIR")).1)
    );
}

#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "excepções mortas:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐⭐ **AS CHAVES EXISTEM DOS DOIS LADOS** — e é aqui que a convenção do BALÃO se paga.
///
/// ⚠️⚠️ **O sufixo `.tip` não entra neste censo, e a razão é uma PROPRIEDADE do mecanismo:** a
/// [`ph2d_panel_model3d`] não escreve nenhuma chave `*.tip` no fonte — ela **compõe-a** a partir da
/// chave da fileira (ver o módulo `dica`). ⇒ para o extractor de chaves usadas, uma dica é
/// **invisível**, e uma dica declarada na tabela e ainda sem fileira apareceria aqui como **órfã**.
///
/// ⛔ **É por isso que a metade das órfãs tolera o sufixo**, com a lista a dizer porquê: *uma chave
/// composta em runtime não é uma chave usada no fonte, e apagá-la por «ninguém a usa» apagaria a
/// frase que o dono pediu.*
#[test]
fn every_key_of_this_panel_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let tabelas = tabelas(&repo);
    let refs: Vec<&str> = tabelas.iter().map(String::as_str).collect();
    let c = gate::chaves(&repo, PREFIX, &refs);
    println!("  declaradas: {} · usadas: {}", c.declaradas, c.usadas);
    // ⛔ Controlo de vacuidade: o vocabulário medido em 2026-09-19, menos folga para encolher.
    assert!(
        c.declaradas >= 60 && c.usadas >= 40,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas — o `tr` pinta o identificador cru (e vaza uma `String` por \
         chamada, pelo `leak_key`):\n  {}",
        c.sem_traducao.join("\n  ")
    );
    let orfas: Vec<&String> = c
        .orfas
        .iter()
        .filter(|k| !k.ends_with(ph2d_panel_model3d::DICA_SUFIXO))
        .filter(|k| !ORFAS_HERDADAS.iter().any(|o| orfa(o) == **k))
        .collect();
    assert!(
        orfas.is_empty(),
        "declaradas e ninguém as usa — apague-as:\n  {orfas:?}"
    );
}

/// ⛔⛔ **A CATRACA DAS ÓRFÃS HERDADAS — ela só ENCOLHE, e tem censo de obsolescência.**
///
/// Estas **sete** chaves foram achadas pela PRIMEIRA corrida deste gate (2026-09-19) e são
/// anteriores a ele: `grep` sobre `crates/` e `shells/` devolve **zero** consumidores para cada uma.
///
/// | chave | o que aconteceu |
/// |---|---|
/// | `panel.model3d.radius` | o painel deixou de ter uma linha «Radius» genérica quando cada dimensão passou a trazer a chave dela (`field.dim.*`) |
/// | `panel.model3d.kind.{box,cylinder,extrude}` | os nomes das formas passaram a sair do CATÁLOGO (`panel.model3d.add.*`), que é a fonte da contagem |
/// | `panel.model3d.kind.{union,intersection,difference}` | idem, pela fileira de operações |
///
/// ⛔ **A cura é APAGÁ-LAS da tabela, e ela não é desta linha:** `crates/ph2d-i18n/**` tem dono
/// nesta jornada, e *uma órfã é onde alguém escreve, um dia, uma frase sobre um controlo que já não
/// existe* — logo a dívida fica NOMEADA aqui em vez de silenciada.
///
/// ⚠️ **Sem a metade de obsolescência isto seria uma LICENÇA** (`CLAUDE.md` §5.0): o gate abaixo
/// reprova quando uma destas deixar de ser órfã — porque alguém a apagou (a cura) **ou** porque
/// alguém lhe arranjou um consumidor (e aí a linha aqui passou a mentir).
/// ⛔⛔ **Elas são guardadas SEM o prefixo, e isso não é arrumação — é a régua a não se ler a si
/// própria.** O censo varre o repo INTEIRO à procura de `panel.model3d.*`, e **este ficheiro é parte
/// do repo**: escritas por extenso, as sete entradas da catraca passavam a contar como
/// *consumidoras*, as sete deixavam de ser órfãs, e a metade de obsolescência acusava-as todas como
/// curadas. *A 1.ª redacção fez exactamente isso, e o gate reprovou sobre a própria lista.* ⚠️ É a
/// mesma armadilha que o doc do `looks_like_a_key` já regista, uma grafia depois.
const ORFAS_HERDADAS: &[&str] = &[
    // ⭐⭐⭐ **VAZIA desde 2026-09-19, e uma catraca vazia é a mais apertada que existe:** já não há
    // linha onde escrever uma chave morta em silêncio. As sete que aqui estavam
    // (`kind.{box,cylinder,extrude,union,intersection,difference}` e `radius`) foram **APAGADAS** da
    // tabela — que é a cura que a própria entrada prescrevia —, e foi a metade de obsolescência
    // deste gate que mandou tirá-las. *Uma catraca sem censo de obsolescência não desce: ela vira
    // LICENÇA* (`CLAUDE.md` §5.0).
];

/// A chave inteira de uma entrada da catraca — ver [`ORFAS_HERDADAS`].
fn orfa(sufixo: &str) -> String {
    format!("{PREFIX}{sufixo}")
}

/// ⭐⭐ **A METADE JUSTA da catraca** — uma entrada que já não descreve nada tem de sair.
#[test]
fn every_inherited_orphan_is_still_an_orphan() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let tabelas = tabelas(&repo);
    let refs: Vec<&str> = tabelas.iter().map(String::as_str).collect();
    let c = gate::chaves(&repo, PREFIX, &refs);
    let curadas: Vec<String> = ORFAS_HERDADAS
        .iter()
        .map(|s| orfa(s))
        .filter(|k| !c.orfas.contains(k))
        .collect();
    assert!(
        curadas.is_empty(),
        "estas chaves deixaram de ser órfãs (ou foram apagadas, que é a cura, ou ganharam um \
         consumidor) — tire-as da catraca:\n  {curadas:?}"
    );
}
