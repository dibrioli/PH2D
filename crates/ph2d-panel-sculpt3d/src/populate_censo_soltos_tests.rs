//! ⛔⛔⛔ **O TERCEIRO CENSO — OS IDS SOLTOS**, irmão (`#[path]`) do
//! [`super`], cortado por ASSUNTO quando ele cruzou o teto de 600 LOC do
//! `architecture_panel_loc_cap`.
//!
//! ⚠️ **O corte é de RESPONSABILIDADE e não de tamanho.** Lá moram as
//! primitivas de extracção e os dois censos das FILEIRAS (o `[..]` e a
//! `TOGGLES`); aqui mora a pergunta que nenhum deles alcança — *e se um id
//! único for pintado à mão, fora de toda tabela?*
//!
//! ⚠️ Ele lê as fontes de PINTURA e as de DECLARAÇÃO de ids, que o irmão não
//! precisa de conhecer.

use super::{
    codigo, comandos, interruptores, nomeados_a_mao_no_populate, nomes_de_ids, registados,
};

// ───────────────────────────────────────────────────────────────────────────
// ⛔⛔⛔ O TERCEIRO CENSO — OS IDS SOLTOS, QUE OS DOIS DE CIMA NÃO VEEM
// ───────────────────────────────────────────────────────────────────────────
//
// Os censos acima cobrem **fileiras** (`&crate::ids::X[..]`) e a tabela
// `TOGGLES`. Um id **SOLTO** — um `NodeId` único pintado à mão, fora de toda
// tabela — é invisível aos dois: ele não é indexado, não tem braço de fileira,
// e o `populate` regista-o numa lista escrita à mão no fim do ficheiro. Hoje
// são **cinco**; o sexto que nascer sem registo é pintado, hit-indexado e
// **morto sob o dedo**, que é a oitava ocorrência que este ficheiro existe para
// tornar impossível.
//
// # Porque os gates da fundação NÃO fecham este buraco (medido 2026-09-15)
//
// O `architecture_panel_wiring_parity` tem dois gates para esta família
// (`hit_indexed_ids_are_registered` e `table_driven_chips_are_registered_too`)
// e **nenhum dos dois vê UM ÚNICO id deste painel**. Duas razões, ambas
// estruturais:
//
// 1. **Ele lê os ficheiros de pintura por NOME** — `file_name` que contenha
//    `"paint"`, ou caminho que contenha `"sections"`. Neste painel isso casa
//    `paint.rs` e `rows_sections.rs` e **salta os seis ficheiros de
//    `paint/`** (`body.rs`, `brush.rs`, `brush_fileiras.rs`, `mask_tools.rs`,
//    `tool.rs`, `widgets.rs`), que é onde os widgets são de facto desenhados.
//    *Um censo definido por prefixo de nome de ficheiro varre o que o nome
//    diz, nunca o assunto* — a §2.7 do HOWTO, outra vez.
// 2. **Ele só colhe `ids::LITERAL` como 1.º argumento de `.register(`.** Este
//    painel nunca escreve isso: os ids viajam como ARGUMENTO para helpers
//    (`paint_panel_close_button(rect, crate::ids::SCULPT3D_CLOSE, hit_index,
//    …)`, `toggle(ctx, id, …)`), e o hit-registo acontece dentro deles.
//
// ⇒ as duas extracções da fundação devolvem o **conjunto VAZIO** para
// `ph2d-panel-sculpt3d`. *Um gate que não colhe nada sobre um painel não o
// aprova: ele não fala dele.*
//
// # A régua
//
// `pintados() ∩ soltos_declarados − derivados_de_tabela` — os ids únicos que o
// código de pintura NOMEIA e que nenhuma das quatro tabelas percorridas pelo
// `populate` cobre. Sobre esse conjunto, três afirmações: ele é registado à
// mão (a metade do morto), tudo o que é registado à mão está nele (a metade do
// órfão), e ele é EXACTAMENTE a catraca abaixo (a metade da vigilância).
//
// ⚠️ **As tabelas de `rows*.rs` que NÃO são a das seções ficam de fora de
// propósito.** Medido: a pintura não nomeia nenhum id de slider/chip — eles
// chegam por `row.slider`/`row.chip` da travessia de `SECTIONS`. Se um dia
// alguém pintar um id de row à mão, este censo acusa-o — e isso está **certo**,
// porque é exactamente a *«row pintada à mão FORA daqui»* que o cabeçalho do
// `rows_sections.rs` proíbe por escrito.

const PAINT: &str = include_str!("paint.rs");
const PAINT_BODY: &str = include_str!("paint/body.rs");
const PAINT_BRUSH: &str = include_str!("paint/brush.rs");
const PAINT_BRUSH_FILEIRAS: &str = include_str!("paint/brush_fileiras.rs");
const PAINT_MASK_TOOLS: &str = include_str!("paint/mask_tools.rs");
const PAINT_TOOL: &str = include_str!("paint/tool.rs");
const PAINT_WIDGETS: &str = include_str!("paint/widgets.rs");

/// `(módulo, fonte)` de cada filho do [`crate::paint`].
///
/// ⚠️ **Esta lista é FECHADA contra o `paint.rs` por gate**
/// ([`a_lista_dos_pintores_e_fechada_sobre_os_modulos_do_paint`]) — é o que
/// impede a cegueira nº 1 da fundação de renascer aqui: um `mod` novo no
/// pintor reprova até entrar nesta tabela. *O `include_str!` falha alto quando
/// um ficheiro MUDA de sítio e é cego a um ficheiro que NASCE; o fecho sobre as
/// declarações de módulo é o que cobre a segunda metade.*
const PINTORES: &[(&str, &str)] = &[
    ("body", PAINT_BODY),
    ("brush", PAINT_BRUSH),
    ("brush_fileiras", PAINT_BRUSH_FILEIRAS),
    ("mask_tools", PAINT_MASK_TOOLS),
    ("tool", PAINT_TOOL),
    ("widgets", PAINT_WIDGETS),
];

const IDS: &str = include_str!("ids.rs");
const ID_INSPECTOR: &str = include_str!("ids/inspector.rs");
const ID_SCULPT3D: &str = include_str!("ids/sculpt3d.rs");
const ID_CLOTH: &str = include_str!("ids/sculpt3d_cloth.rs");
const ID_POSE: &str = include_str!("ids/sculpt3d_pose.rs");
const ID_BOUNDARY: &str = include_str!("ids/sculpt3d_boundary.rs");
const ID_SMEAR: &str = include_str!("ids/sculpt3d_smear.rs");
const ID_TRIM: &str = include_str!("ids/sculpt3d_trim.rs");
const ID_PLANO: &str = include_str!("ids/sculpt3d_plano.rs");
const ID_PROJECT: &str = include_str!("ids/sculpt3d_project.rs");
const ID_BRUSH: &str = include_str!("ids/sculpt3d_brush.rs");
const ID_SHADING: &str = include_str!("ids/sculpt3d_shading.rs");

/// `(módulo, fonte)` de cada filho do [`crate::ids`] — **fechada por gate**,
/// como a dos pintores. Ela é quem sabe separar um id SOLTO de uma FILEIRA, e
/// uma declaração que não fosse varrida leria-se como *«este id não existe»*,
/// que num censo por diferença é o mesmo byte que *«está coberto»*.
const ID_FONTES: &[(&str, &str)] = &[
    ("inspector", ID_INSPECTOR),
    ("sculpt3d", ID_SCULPT3D),
    ("sculpt3d_cloth", ID_CLOTH),
    ("sculpt3d_pose", ID_POSE),
    ("sculpt3d_boundary", ID_BOUNDARY),
    ("sculpt3d_smear", ID_SMEAR),
    ("sculpt3d_trim", ID_TRIM),
    ("sculpt3d_plano", ID_PLANO),
    ("sculpt3d_project", ID_PROJECT),
    ("sculpt3d_brush", ID_BRUSH),
    ("sculpt3d_shading", ID_SHADING),
];

const SECOES: &str = include_str!("rows_sections.rs");

/// ⛔⛔ **A CATRACA dos ids soltos: um por linha, COM A RAZÃO e com o gate de
/// costura que o cobre.**
///
/// Um id aqui é uma exceção **declarada**, não uma permissão: ele é registado
/// à mão no fim do `populate.rs` e tem um gate de costura próprio que CLICA
/// nele. ⚠️ **Ela só encolhe** — a metade de obsolescência de
/// [`a_catraca_dos_ids_soltos_nao_cresce_nem_envelhece`] reprova uma linha que
/// já não descreve nada, porque *uma catraca sem censo de obsolescência não
/// desce: ela vira LICENÇA* (`CLAUDE.md` §5.0).
///
/// ⚠️ **Entrada nova exige DUAS coisas**, e a segunda é o ponto: a linha aqui e
/// o gate de costura que carrega no pixel. Sem o segundo, esta lista passa a
/// ser exactamente a lista escrita à mão que os dois censos acima existem para
/// matar.
const SOLTOS_COM_GATE_PROPRIO: &[(&str, &str)] = &[
    // Os TRÊS eixos do espelho: pintados em `paint/body.rs` numa tabela em
    // linha, despachados por um braço que compara os três nomes. Costura:
    // `each_mirror_axis_toggles_only_itself` + as três linhas «sym x/y/z» do
    // `every_painted_control_is_clickable_where_it_is_drawn`
    // (`tests/it/seam.rs`), que clicam no centro de cada um.
    (
        "SCULPT3D_SYM_X",
        "eixo do espelho · seam: each_mirror_axis_toggles_only_itself",
    ),
    (
        "SCULPT3D_SYM_Y",
        "eixo do espelho · seam: each_mirror_axis_toggles_only_itself",
    ),
    (
        "SCULPT3D_SYM_Z",
        "eixo do espelho · seam: each_mirror_axis_toggles_only_itself",
    ),
    // O comando «aplicar a referência a todos»: pintado em `paint/tool.rs` por
    // `command(..)`, braço próprio no `event.rs`. Costura: a linha «ref apply
    // to all» do `every_painted_control_is_clickable_where_it_is_drawn`.
    (
        "SCULPT3D_REF_MODE_ALL",
        "comando de um toque · seam: every_painted_control_is_clickable_where_it_is_drawn",
    ),
    // ⚠️ **O FECHO do painel é o único SEM gate de costura, e a ausência é
    // conhecida** (medido 2026-09-15: nenhuma ocorrência em `tests/`). Ele é
    // cromo de painel — pintado pelo `paint_panel_close_button` da
    // `ph2d-editor-core`, que é partilhado por todos os painéis —, logo a
    // costura dele é da fundação e não desta crate. *Uma isenção nomeada e uma
    // isenção silenciosa leem-se igual numa tabela; esta está nomeada.*
    (
        "SCULPT3D_CLOSE",
        "cromo de painel (paint_panel_close_button da fundação) · SEM seam próprio: dívida nomeada",
    ),
];

/// Os `mod X;` declarados numa fonte.
fn modulos(fonte: &'static str) -> std::collections::BTreeSet<&'static str> {
    codigo(fonte)
        .map(|l| {
            l.trim_start_matches("pub(crate) ")
                .trim_start_matches("pub ")
        })
        .filter_map(|l| l.strip_prefix("mod ").and_then(|r| r.strip_suffix(';')))
        .collect()
}

/// `(ids SOLTOS, ids de FILEIRA)` declarados pelo [`crate::ids`].
type Declaracoes = (
    std::collections::BTreeSet<&'static str>,
    std::collections::BTreeSet<&'static str>,
);

fn declaracoes() -> Declaracoes {
    let (mut soltos, mut fileiras) = (
        std::collections::BTreeSet::new(),
        std::collections::BTreeSet::new(),
    );
    for &(_, fonte) in ID_FONTES {
        for linha in codigo(fonte) {
            let Some((_, direita)) = linha.split_once("pub const ") else {
                continue;
            };
            let fim = direita
                .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                .unwrap_or(direita.len());
            let (nome, cauda) = direita.split_at(fim);
            if nome.len() < 2 {
                continue;
            }
            if cauda.starts_with(": NodeId") {
                soltos.insert(nome);
            } else if cauda.starts_with(": [NodeId") {
                fileiras.insert(nome);
            }
        }
    }
    (soltos, fileiras)
}

/// Todo `ids::NOME` que o código de PINTURA nomeia.
fn pintados() -> std::collections::BTreeSet<&'static str> {
    let mut achados = nomes_de_ids(PAINT);
    for &(_, fonte) in PINTORES {
        achados.extend(nomes_de_ids(fonte));
    }
    achados
}

/// Os ids que o `populate` regista **percorrendo uma tabela** — as quatro
/// travessias dele: as fileiras, os cabeçalhos de secção, os comandos e os
/// interruptores.
fn derivados_de_tabela() -> std::collections::BTreeSet<&'static str> {
    let mut cobertos = comandos();
    cobertos.extend(interruptores());
    cobertos.extend(nomes_de_ids(SECOES));
    cobertos.extend(registados());
    cobertos
}

/// **OS IDS SOLTOS PINTADOS** — o sujeito dos três gates abaixo.
fn soltos_pintados() -> std::collections::BTreeSet<&'static str> {
    let (soltos_declarados, _) = declaracoes();
    let cobertos = derivados_de_tabela();
    pintados()
        .into_iter()
        .filter(|n| soltos_declarados.contains(n) && !cobertos.contains(n))
        .collect()
}

/// ⛔⛔⛔ **GATE — todo id SOLTO pintado é registado à mão pelo `populate`**,
/// que é a metade do controlo morto sob o dedo para a população que nem as
/// fileiras nem a `TOGGLES` cobrem.
#[test]
fn todo_id_solto_pintado_e_registado_a_mao_pelo_populate() {
    let a_mao = nomeados_a_mao_no_populate();
    let mortos: Vec<&str> = soltos_pintados().difference(&a_mao).copied().collect();
    assert!(
        mortos.is_empty(),
        "estes ids SOLTOS sao pintados pelo `paint/` e NAO sao registados pelo \
         `populate.rs`: {mortos:?} — eles sao hit-indexados e MORTOS sob o \
         ponteiro. ⛔ Os gates da fundacao NAO os veem (ver o cabecalho desta \
         seccao): a cura e' acrescenta'-los a' lista a' mao no fim do \
         `populate.rs` E escrever o gate de costura que clica neles"
    );
}

/// ⛔⛔ **GATE — todo id nomeado à mão no `populate` é um id SOLTO pintado**,
/// que é a metade do órfão, e a cura dela é a OPOSTA (apagar, ou derivar de uma
/// tabela, nunca ligar).
///
/// ⚠️ Ele apanha **duas** espécies de uma vez: o id registado que ninguém pinta
/// (lixo) e o id registado à mão que uma tabela **já** cobre (a segunda lista
/// ao lado de uma tabela — a doença que o gate dos interruptores acima curou
/// sobre 17 entradas).
#[test]
fn todo_id_nomeado_a_mao_no_populate_e_um_solto_pintado() {
    let soltos = soltos_pintados();
    let intrusos: Vec<&str> = nomeados_a_mao_no_populate()
        .difference(&soltos)
        .copied()
        .collect();
    assert!(
        intrusos.is_empty(),
        "o `populate.rs` nomeia a' mao ids que NAO sao soltos-pintados: \
         {intrusos:?} — ou ninguem os pinta (lixo, cura: APAGAR), ou ja' vem de \
         uma das quatro tabelas que ele percorre (cura: apagar a linha a' mao, \
         que e' a segunda resposta a' mesma pergunta)"
    );
}

/// ⛔⛔⛔ **GATE — a catraca dos soltos não CRESCE nem ENVELHECE.**
///
/// A metade que cresce: um id solto novo reprova mesmo estando registado —
/// porque registar não basta, ele precisa do gate de costura que carrega nele.
/// A metade que envelhece: uma linha da catraca que já não descreve nada
/// reprova, senão *a catraca vira LICENÇA* (`CLAUDE.md` §5.0).
#[test]
fn a_catraca_dos_ids_soltos_nao_cresce_nem_envelhece() {
    let soltos = soltos_pintados();
    let catraca: std::collections::BTreeSet<&str> =
        SOLTOS_COM_GATE_PROPRIO.iter().map(|&(n, _)| n).collect();
    let novos: Vec<&str> = soltos.difference(&catraca).copied().collect();
    assert!(
        novos.is_empty(),
        "ids SOLTOS novos, fora da catraca: {novos:?} — um id solto e' a forma \
         que NENHUM censo deste repo ve' sozinho. => escreva o gate de costura \
         que CLICA nele e acrescente-o ao `SOLTOS_COM_GATE_PROPRIO` com a razao \
         e o nome desse gate. ⛔ Nunca acrescente a linha sem o gate: ai' esta \
         lista vira a lista a' mao que os censos acima existem para matar"
    );
    let obsoletos: Vec<&str> = catraca.difference(&soltos).copied().collect();
    assert!(
        obsoletos.is_empty(),
        "estas linhas do `SOLTOS_COM_GATE_PROPRIO` ja' nao descrevem nada: \
         {obsoletos:?} — o id deixou de ser pintado, ou passou a vir de uma \
         tabela. A catraca tem de DESCER, senao a proxima pessoa le'-a como se \
         a excepcao ainda existisse"
    );
}

/// ⭐ **O PISO DE POPULAÇÃO E OS CONTROLOS** dos três gates acima — sem eles,
/// uma extracção partida devolve conjuntos vazios e as três diferenças ficam
/// **trivialmente** verdes.
///
/// ⚠️ **O controlo NEGATIVO é metade do valor:** um censo por diferença fica
/// verde tanto quando não acha nada como quando a cobertura engole tudo, e é a
/// segunda forma que morde aqui (ler o `event.rs` inteiro em vez do corpo da
/// tabela dá `0` soltos sobre um painel que tem `5`).
#[test]
fn o_censo_dos_ids_soltos_varre_a_populacao_toda() {
    let (soltos_declarados, fileiras_declaradas) = declaracoes();
    assert!(
        soltos_declarados.len() >= 120 && fileiras_declaradas.len() >= 20,
        "as declarações de ids leem {} soltos e {} fileiras (medido 2026-09-15: \
         155 e 23) — a extracção de `pub const NOME: NodeId` quebrou, não o \
         produto",
        soltos_declarados.len(),
        fileiras_declaradas.len()
    );
    let pintados = pintados();
    assert!(
        pintados.len() >= 50 && comandos().len() >= 15 && nomes_de_ids(SECOES).len() >= 6,
        "a varredura leu {} ids pintados, {} comandos e {} cabeçalhos de secção \
         (medido: 69 / 19 / 7) — uma delas deixou de casar, e os gates irmãos \
         passariam a medir o vácuo",
        pintados.len(),
        comandos().len(),
        nomes_de_ids(SECOES).len()
    );
    // ⚠️ **Controlo POSITIVO da extracção:** um solto que TEM de estar lá, e
    // uma fileira que tem de ser reconhecida COMO fileira.
    assert!(
        pintados.contains("SCULPT3D_SYM_X") && soltos_declarados.contains("SCULPT3D_SYM_X"),
        "a extracção perdeu o `SCULPT3D_SYM_X` — ela deixou de ler o `paint/` ou \
         o `ids/` como eles estão escritos"
    );
    assert!(
        fileiras_declaradas.contains("SCULPT3D_VERB"),
        "o `SCULPT3D_VERB` deixou de ser lido como FILEIRA — a separação \
         solto/fileira quebrou, e os soltos passariam a incluir toda a tabela"
    );
    // ⚠️ **Controlo NEGATIVO da SUBTRACÇÃO:** um representante de cada uma das
    // quatro travessias do `populate` NÃO pode aparecer como solto. Sem isto,
    // uma cobertura vazia leria-se como «não há mortos».
    let soltos = soltos_pintados();
    for (nome, de_onde) in [
        ("SCULPT3D_SUBDIVIDE", "a tabela COMMANDS"),
        ("SCULPT3D_ACCUMULATE", "a tabela TOGGLES"),
        ("SCULPT3D_SEC_SYMMETRY", "os cabeçalhos de secção"),
        ("SCULPT3D_VERB", "as fileiras `[..]`"),
    ] {
        assert!(
            !soltos.contains(nome),
            "`{nome}` apareceu como id SOLTO e ele vem de {de_onde} — a \
             cobertura deixou de o ver, e este censo passou a acusar controlos \
             vivos"
        );
    }
}

/// ⭐⭐ **GATE — a lista de PINTORES é fechada sobre os `mod` do `paint.rs`.**
///
/// ⚠️ *O `include_str!` falha alto quando um ficheiro MUDA de sítio e é cego a
/// um que NASCE* — um `mod` novo no pintor entraria em produção com os ids dele
/// invisíveis a este censo, que é exactamente a cegueira nº 1 da fundação. Este
/// fecho torna o nascimento um erro de teste.
#[test]
fn a_lista_dos_pintores_e_fechada_sobre_os_modulos_do_paint() {
    let declarados = modulos(PAINT);
    let lidos: std::collections::BTreeSet<&str> = PINTORES.iter().map(|&(n, _)| n).collect();
    assert!(
        declarados.len() >= 6,
        "o `paint.rs` declara só {} módulos (medido: 6) — a extracção de \
         `mod X;` quebrou",
        declarados.len()
    );
    assert_eq!(
        declarados, lidos,
        "os módulos de `paint.rs` e a tabela `PINTORES` divergiram — um `mod` \
         novo tem de ser acrescentado à tabela (senão os ids dele são invisíveis \
         a este censo), e um `mod` apagado tem de sair dela"
    );
}

/// ⭐⭐ **GATE — a lista de fontes de IDS é fechada sobre os `mod` do
/// `ids.rs`.** Mesma lei do irmão acima: uma declaração de id não varrida
/// lê-se como *«este id não existe»*, e num censo por diferença isso é o mesmo
/// byte que *«está coberto»*.
#[test]
fn a_lista_das_fontes_de_ids_e_fechada_sobre_os_modulos_do_ids() {
    let declarados = modulos(IDS);
    let lidos: std::collections::BTreeSet<&str> = ID_FONTES.iter().map(|&(n, _)| n).collect();
    assert!(
        declarados.len() >= 9,
        "o `ids.rs` declara só {} módulos (medido: 9) — a extracção de `mod X;` \
         quebrou",
        declarados.len()
    );
    assert_eq!(
        declarados, lidos,
        "os módulos de `ids.rs` e a tabela `ID_FONTES` divergiram — um ficheiro \
         de ids novo tem de entrar na tabela, senão os `pub const` dele não são \
         classificados e o censo dos soltos mede menos do que existe"
    );
}
