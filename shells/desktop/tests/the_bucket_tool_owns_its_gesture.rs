//! **Arch-gate da costura do BALDE** (plano 40) — o gesto inteiro é dele.
//!
//! ## O que este gate protege
//!
//! Cinco maneiras de partir a ferramenta deixam **todos os unit tests verdes**, porque nenhum deles
//! alcança o corpo do `input_dispatch` nem o dreno do quadro:
//!
//! 1. **o press cai na cadeia de baixo** — sem o `return`, um clique no vazio com o Balde na mão
//!    começa a desenhar uma forma (o defeito que o Lápis, o Width e o Trim já pagaram);
//! 2. **o realce não é limpo ao trocar de ferramenta** — uma região fica acesa a prometer um
//!    preenchimento que nenhum clique faz;
//! 3. **o clique RECALCULA em vez de usar a face do quadro** — o cursor pode ter andado um pixel
//!    entre o desenho e o gesto, e o artista depositaria uma região que nunca viu;
//! 4. **a forma nasce ao TOPO** — ela tapa as linhas que a cercam, e o desenho desaparece sob a
//!    própria tinta;
//! 5. **a rede é montada por QUADRO** — medido, `3,8 ms` a 20 traços e `188 ms` a 80, contra um
//!    orçamento de `16,7`.
//!
//! As asserções afirmam RELAÇÃO, nunca distância no fonte.

const DISPATCH: &str = include_str!("../src/input_dispatch.rs");
const LOOP: &str = include_str!("../src/render_loop/mod.rs");
const BUCKET: &str = include_str!("../src/vec_bucket.rs");

fn at(src: &str, needle: &str, onde: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "{onde} nao contem `{needle}` — se foi renomeado, actualize este gate (e confira que o \
             Balde ainda funciona: `PH2D_BUILD_SMOKE=82`)"
        )
    })
}

/// **Controle positivo:** as âncoras existem. Um scanner que não acha nada passaria em silêncio.
#[test]
fn the_scanner_finds_what_it_scans_for() {
    at(DISPATCH, "ph2d_tool_vector::DrawMode::Bucket", "o dispatch");
    at(DISPATCH, "self.apply_bucket()", "o dispatch");
    at(LOOP, "self.refresh_bucket_hover(pointer);", "o render_loop");
    at(LOOP, "ph2d_vec_render::draw_bucket_face(", "o render_loop");
    at(BUCKET, "fn refresh_bucket_hover(", "o vec_bucket");
}

/// **O press do Balde CONSOME o clique** — o `return` vem depois do bloco dele e antes da cadeia
/// que desenha formas.
#[test]
fn the_press_is_consumed_so_it_never_falls_into_drawing() {
    let modo = at(
        DISPATCH,
        "if self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Bucket {",
        "o dispatch",
    );
    let aplica = at(DISPATCH, "self.apply_bucket()", "o dispatch");
    let ret = DISPATCH[aplica..]
        .find("return;")
        .map(|i| i + aplica)
        .expect("o bloco do Balde tem de terminar em `return;`");
    assert!(
        modo < aplica && aplica < ret,
        "o `return` nao fecha o bloco do Balde"
    );
    // ⚠️ **E ele está FORA da guarda do estado.** Um `return` lá dentro deixaria um clique **sem
    // região acesa** cair na cadeia de baixo e começar a desenhar uma forma.
    //
    // A régua é a INDENTAÇÃO: o `return;` tem de estar ao mesmo nível do `let antes` (o corpo do
    // bloco do modo), e não mais fundo. ⛔ Contar chavetas foi a 1.ª redacção e reprovou sobre
    // produto correcto — *uma heurística de chavetas não sabe ler Rust*.
    let indent = |i: usize| {
        let linha = DISPATCH[..i].rfind('\n').map_or(0, |n| n + 1);
        i - linha
    };
    let antes = at(DISPATCH, "let antes = self", "o dispatch");
    assert_eq!(
        indent(ret),
        indent(antes),
        "o `return` do Balde esta' mais fundo que o corpo do bloco — ele parece estar DENTRO da \
         guarda do estado, e um clique sem regiao acesa cairia na cadeia de baixo"
    );
}

/// **O clique usa a face do QUADRO** — `apply_bucket` lê `vec_bucket_face` e não recalcula.
#[test]
fn the_click_deposits_what_the_highlight_showed() {
    let f = at(BUCKET, "fn apply_bucket(", "o vec_bucket");
    let corpo = &BUCKET[f..];
    let fim = corpo.find("\n    /// ").unwrap_or(corpo.len());
    let corpo = &corpo[..fim];
    assert!(
        corpo.contains("self.vec_bucket_face.clone()"),
        "o `apply_bucket` nao le a face do quadro"
    );
    assert!(
        !corpo.contains("ph2d_vec_fill::rede(") && !corpo.contains("face_em("),
        "o clique RECALCULA a face — o artista depositaria uma regiao que nunca viu:\n{corpo}"
    );
}

/// ⛔⛔ **A forma nasce ATRÁS de tudo, e quem manda nisso é a ENTIDADE.**
///
/// Report do Enio (2026-09-01): *"o preenchimento está acima do stroke, mas deveria estar abaixo"*.
/// O `insert_path(0, …)` põe a forma no início da CENA e não muda nada no desenho: a ordem que o
/// olho vê é o `RootOrder`, e o `vec_entities::sync` dá a toda entidade nova **o maior**.
#[test]
fn the_filled_shape_is_born_behind_the_lines() {
    let f = at(BUCKET, "fn arm_new_fills(", "o vec_bucket");
    let corpo = &BUCKET[f..];
    assert!(
        corpo.contains("ZOrder::ToBack"),
        "a forma do Balde nao e' mandada para o fundo — ela taparia o desenho"
    );
    // …e a receita é presa na MESMA passagem: a entidade só existe aqui.
    //
    // ⚠️ **A receita são as ÂNCORAS, e a semente é a rede** (plano 40 §11, 2026-09-02). Até aí ela
    // era só o ponto do clique; o que a substituiu são os pedaços de linha que cercavam a região.
    assert!(
        corpo.contains("VecBucketFill::new(seed, ancoras)"),
        "a receita (as ancoras + a semente) nao e' presa a' entidade — o preenchimento nao seria vivo"
    );
    let sync = at(LOOP, "crate::vec_entities::sync(", "o render_loop");
    let arma = at(LOOP, "crate::vec_bucket::arm_new_fills(", "o render_loop");
    assert!(
        sync < arma,
        "o `arm_new_fills` corre ANTES do `sync` — a entidade ainda nao existe"
    );
}

/// ⭐⭐⭐ **O UPKEEP corre em QUALQUER ferramenta** — é ele que mantém o preenchimento vivo.
///
/// Report do Enio (2026-09-01): *"se movo os nós da linha, o preenchimento não acompanha"*. O
/// artista arrasta um nó com a seta BRANCA; se o re-cozimento vivesse dentro do modo Balde, ele
/// nunca correria.
#[test]
fn the_upkeep_runs_in_every_tool_not_only_in_the_bucket() {
    let up = at(LOOP, "self.bucket_upkeep();", "o render_loop");
    let hover = at(LOOP, "self.refresh_bucket_hover(pointer);", "o render_loop");
    assert!(
        up < hover,
        "o upkeep tem de correr ANTES do realce, que le a rede dele"
    );
    // A guarda de MODO vive no realce, nunca no upkeep.
    let f = at(BUCKET, "fn bucket_upkeep(", "o vec_bucket");
    let fim = BUCKET[f..]
        .find("fn refresh_bucket_hover(")
        .unwrap_or(BUCKET.len() - f)
        + f;
    assert!(
        !BUCKET[f..fim].contains("!= ph2d_tool_vector::DrawMode::Bucket"),
        "o upkeep saiu cedo fora do modo Balde — o preenchimento deixaria de acompanhar as linhas"
    );
    // E ele RE-COZE: as faces que cada preenchimento GANHA viram a geometria do caminho.
    //
    // ⚠️ **Esta lista de literais mudou em 2026-09-02, e a mudança é a lição**: até aí o gate
    // exigia `rede.face_em(*seed)` — *uma* face por semente —, que é exactamente o defeito que o
    // report daquele dia nomeia (*"quando atravessamos uma linha com um nó, os preenchimentos se
    // quebram"*). ⛔ Um gate textual afirma sobre a REDACÇÃO, não sobre o comportamento: o que a
    // votação faz está medido em `vec_bucket_claim_tests`, e o que aqui se prova é só que o
    // `bucket_upkeep` **chama** a lei e **escreve** o que ela devolve.
    assert!(
        BUCKET[f..fim].contains("crate::vec_bucket_claim::donos("),
        "o upkeep nao pergunta de quem e' cada face — os preenchimentos nao sobreviveriam a uma \
         mudanca de topologia"
    );
    assert!(
        BUCKET[f..fim].contains("p.verts = primeiro;")
            && BUCKET[f..fim].contains("p.subpaths = subs;"),
        "o upkeep calcula a area nova e nao a ESCREVE inteira — uma regiao que partiu perderia \
         metade"
    );
    // ⚠️ **E ela DESCE ao espaço do caminho antes de ser escrita.** A rede fala MUNDO; um
    // `VecPath` já assentado tem pose própria, e escrever mundo nele desloca-o pelo centro dele —
    // o report de 2026-09-01 (*"nascendo deslocado para fora do stroke"*).
    assert!(
        BUCKET.contains("para_local(g, xfp)"),
        "a area re-cozida e' escrita em MUNDO num caminho que tem pose — ela sai deslocada"
    );
    // ⭐⭐⭐ **E o dono de cada face sai das ÂNCORAS, não de comparar com o quadro anterior.**
    //
    // ⚠️⚠️ **Esta lista de literais mudou pela SEGUNDA vez, e a mudança é a lição.** Ela exigiu,
    // sucessivamente, `rede.face_em(*seed)` (uma face por semente), depois `donos(…)` com regiões
    // do quadro anterior, e agora a resolução por âncora — três modelos em três dias. ⛔ Um gate
    // textual afirma sobre a REDACÇÃO, nunca sobre o comportamento: o que a lei faz está medido em
    // `vec_bucket_claim_tests` e nos ficheiros que o Enio exportou; o que aqui se prova é só que o
    // `bucket_upkeep` **chama** a lei e **escreve** o que ela devolve.
    assert!(
        BUCKET[f..fim].contains("crate::vec_bucket_claim::donos(&rede, &faces, &tags, &receitas)"),
        "o upkeep nao resolve as ancoras — a tinta voltaria a depender do quadro anterior"
    );
    assert!(
        BUCKET[f..fim].contains("ancoras: &f.ancoras"),
        "as receitas nao levam as ancoras do proprio preenchimento"
    );
    // ⛔⛔ **E a semente NÃO se re-semeia.** Reescrevê-la a cada quadro é escrita derivada, e foi
    // essa escrita — generalizada para uma região inteira — que fez a tinta derivar e trocar de
    // área nos quatro reports. *O gate exige a AUSÊNCIA porque a presença é o defeito.*
    assert!(
        !BUCKET[f..fim].contains("VecBucketFill::new("),
        "o upkeep reescreve a receita — a deriva volta por aqui"
    );
    // ⚠️ E a exclusão passa pela porta ÚNICA, com os DOIS termos: um fecho escrito à mão aqui foi
    // o que deixou a mutação `o-fill-entra-na-rede` sobreviver.
    assert!(
        BUCKET[f..fim].contains("fora_da_rede(vista.is_hidden(id), so_fill.contains(&id))"),
        "a exclusao nao passa pela porta unica, ou perdeu um dos dois termos"
    );
}

/// **O realce é LIMPO fora do modo**, e a rede guardada morre com ele.
#[test]
fn leaving_the_tool_clears_the_highlight_and_the_cache() {
    let f = at(BUCKET, "fn refresh_bucket_hover(", "o vec_bucket");
    let guarda = at(
        &BUCKET[f..],
        "!= ph2d_tool_vector::DrawMode::Bucket",
        "o refresher",
    ) + f;
    let corpo = &BUCKET[guarda..];
    let fim = corpo.find("return;").expect("a guarda tem de sair cedo");
    assert!(
        corpo[..fim].contains("self.vec_bucket_face = None;"),
        "sair do Balde nao apaga o realce"
    );
    // ⚠️ A rede guardada NÃO morre aqui: ela serve os preenchimentos vivos em toda ferramenta. O
    // que a apaga é o upkeep, quando não há preenchimento nenhum **e** o balde não está na mão.
    let up = at(BUCKET, "fn bucket_upkeep(", "o vec_bucket");
    assert!(
        BUCKET[up..].contains("if fills.is_empty() && !armado {"),
        "o upkeep nao sai de graca quando nao ha' nada a fazer — quem nao usa o balde pagaria"
    );
}

/// ⛔ **A rede é GUARDADA, não montada por quadro** — o `rede(` só corre atrás da comparação de
/// chave.
#[test]
fn the_network_is_cached_not_rebuilt_every_frame() {
    let chave = at(
        BUCKET,
        "if self.vec_bucket_cache.as_ref().is_some_and(|c| c.chave == k) {",
        "o vec_bucket",
    );
    let monta = at(BUCKET, "ph2d_vec_fill::rede(&contornos)", "o vec_bucket");
    assert!(
        chave < monta,
        "a rede e' montada ANTES da comparacao de chave — isso e' montar por quadro, e custa \
         3,8 ms a 20 tracos"
    );
    assert_eq!(
        BUCKET.matches("ph2d_vec_fill::rede(").count(),
        1,
        "ha' mais de um sitio a montar a rede — um deles nao passa pelo cache"
    );
}
