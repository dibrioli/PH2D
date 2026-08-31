//! Gates da **membrana das fitas** — a metade da shell do modo `Branches`.
//!
//! ⚠️ **As quatro condições de UI não servem aqui**: isto não é um widget, é uma MEMBRANA. A
//! pergunta é a do `source.shape`: *a shell publica sob a chave que o nó lê?* Um par de chaves
//! divergentes não dá erro nenhum — dá uma planta invisível.

use super::publish;
use crate::render_loop::motion_lsystem_testkit::{key_of, plant, published};
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

/// ⭐⭐⭐ **UMA planta é UMA instância, com UMA geometria.**
///
/// ⚠️⚠️ **A lei apertou depois do *"ficamos com 4 fps"* (Enio, 2026-08-30).** Ela era *"menos
/// fitas que ossos"* — verdadeira e frouxa: a 1.ª redacção publicava **uma instância por RAMO**,
/// cada uma com geometria distinta, e o renderer tesselava as `3 124` **todo o quadro** (o cache
/// dele é por `geometry_id` e por quadro). Menos que os ossos, e na mesma inutilizável.
///
/// ⇒ a afirmação passa a ser o NÚMERO EXACTO: `1`. É também a leitura mais fiel do report que
/// abriu esta wave — *"não crescem como um objeto só"*.
#[test]
fn a_plant_in_branches_mode_publishes_fewer_ribbons_than_it_has_bones() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let ribbons = published(&state, &key).expect("a shell tem de publicar sob a chave do no'");
    assert_eq!(
        ribbons, 1,
        "uma planta tem de sair como UMA instância — {ribbons} seria uma tesselação por ramo, \
         todo o quadro"
    );

    // Quantos ossos a mesma planta tem, pela porta do próprio nó.
    let resolved =
        super::super::motion_externals::resolved_params(&mut state, n, 0.0, &ls::MANIFEST);
    let sk = ls::skeleton("F", "F -> F[+F]F[-F]F", |name: &str| {
        resolved.get(name).copied().unwrap_or(0.0)
    });
    assert!(
        ribbons < sk.count(),
        "{ribbons} fitas para {} ossos — isso é uma fita por retângulo",
        sk.count()
    );
}

/// ⭐⭐ **Cada fita leva uma GEOMETRIA de verdade.**
///
/// ⚠️ Um `geometry_id` de `0` é o «nada» do lowering: publicar contagem certa com ids vazios
/// desenharia coisa nenhuma e passaria no gate de cima.
#[test]
fn every_published_ribbon_carries_a_real_geometry_handle() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let ext = state.pump.cook.externals().get(&key).expect("publicado");
    let Some(Column::Scalar(ids)) = ext.value.get("geometry_id") else {
        panic!("a fita tem de carregar `geometry_id`");
    };
    assert!(!ids.is_empty());
    assert!(
        ids.iter().all(|h| *h > 0.0),
        "há fitas com handle vazio — elas não desenham: {ids:?}"
    );
}

/// ⭐ **O modo antigo continua intocado** — decisão do Enio (*"não quero eliminar o modo
/// atual"*).
///
/// A shell não publica nada, e o nó emite o esqueleto de sempre.
#[test]
fn segments_mode_publishes_nothing_and_keeps_the_old_skeleton() {
    let (mut state, n) = plant(ls::GEOMETRY_SEGMENTS);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    assert!(
        published(&state, &key).is_none(),
        "o modo Segments não pode publicar fitas"
    );
}

/// ⭐⭐⭐ **O default do nó É `Branches`** — a ordem do dono, medida no manifesto e não na
/// memória de ninguém.
#[test]
fn a_node_dropped_from_the_palette_is_born_in_branches_mode() {
    let d = ls::MANIFEST
        .params
        .iter()
        .find(|s| s.name == ls::param::GEOMETRY)
        .expect("o param existe")
        .default;
    assert_eq!(
        d.round() as i32,
        ls::GEOMETRY_BRANCHES,
        "o default tem de ser Branches (Enio, 2026-08-30)"
    );
    // ⚠️ E o VALOR de `Segments` continua a ser `0`: um documento salvo guarda o índice.
    assert_eq!(ls::GEOMETRY_SEGMENTS, 0);
}

/// ⛔⛔⛔ **UMA PLANTA QUE NÃO MUDOU NÃO CONSTRÓI NADA** — o gate que nasceu do *"ficamos com
/// 4 fps"* (Enio, 2026-08-30).
///
/// A membrana tinha o memo certo e **não o usava**: chamava o construtor da fita e só depois o
/// entregava ao `intern`, que não o teria chamado. Cada quadro re-corria o varrimento booleano
/// de todos os ramos de todas as plantas.
///
/// ⚠️ **A régua é uma CONTAGEM, não um relógio** — de propósito. Um gate de tempo entra na
/// família de flakes de recurso sob fan-out do `CLAUDE.md` §5.0; o número de geometrias
/// guardadas é determinístico e diz exactamente a mesma coisa: *se nada mudou, nada se
/// construiu*.
///
/// ⚠️ E corre a VARREDURA entre as duas publicações, que é a segunda metade: sem o
/// `handle_for` a marcar as chaves como vivas, o fim do quadro apagaria as geometrias que estão
/// a ser desenhadas e a reconstrução voltava por outra porta — com o memo intacto.
#[test]
fn republishing_an_unchanged_plant_builds_no_geometry_and_survives_the_sweep() {
    let (mut state, _n) = plant(ls::GEOMETRY_BRANCHES);

    let before = super::ribbons_built();
    publish(&mut state, 0.0);
    let built = state.shape_store.len();
    let first_pass = super::ribbons_built() - before;
    assert!(built > 0, "a 1.ª publicação tem de construir as fitas");
    assert!(first_pass > 0, "a 1.ª publicação tem de CONSTRUIR fitas");
    let dropped = state.shape_store.sweep();
    assert!(
        dropped.is_empty(),
        "a varredura do 1.º quadro apagou {} geometrias que acabaram de ser pedidas",
        dropped.len()
    );

    let before_second = super::ribbons_built();
    publish(&mut state, 0.0);
    let second_pass = super::ribbons_built() - before_second;
    assert_eq!(
        second_pass, 0,
        "a 2.ª publicação da MESMA planta CONSTRUIU {second_pass} fitas (a 1.ª construiu \
         {first_pass}) — o memo está lá e não está a ser usado"
    );
    assert_eq!(
        state.shape_store.len(),
        built,
        "e nada de novo foi guardado"
    );
    let dropped = state.shape_store.sweep();
    assert!(
        dropped.is_empty(),
        "a varredura do 2.º quadro apagou {} geometrias ainda em uso — falta marcá-las vivas",
        dropped.len()
    );
    assert_eq!(state.shape_store.len(), built, "nada se perdeu no caminho");
}

/// ⛔⛔ **UMA PLANTA GRANDE SAI INTEIRA** — o report do Enio de 2026-08-30 (*"9841 ramos passam
/// do tecto de 4096 — a planta sai cortada"*).
///
/// ⚠️ **O gate mede a AUSÊNCIA de um segundo tecto**, e a barra não é um número escolhido: é a
/// contagem que a decomposição devolve. Um corte a `N` ramos passaria despercebido em toda
/// planta pequena e mutilaria exactamente as grandes — que são as que alguém constrói para ver
/// se o motor aguenta.
///
/// ⚠️ E o limite a sério fica NOMEADO no lado do nó (`MAX_MODULES`), que é onde ele foi medido.
#[test]
fn a_big_plant_is_published_whole_and_no_second_ceiling_clips_it() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    // Seis gerações desta gramática dão ~15 k ramos — bem acima do tecto que foi removido.
    state.doc.graph.set_param(n, ls::param::GENERATIONS, 6.0);
    let before = super::ribbons_built();
    publish(&mut state, 0.0);
    let built = super::ribbons_built() - before;

    let resolved =
        super::super::motion_externals::resolved_params(&mut state, n, 0.0, &ls::MANIFEST);
    let sk = ls::skeleton("F", "F -> F[+F]F[-F]F", |name: &str| {
        resolved.get(name).copied().unwrap_or(0.0)
    });
    let want = ls::branch::branches(
        &super::v2(&sk, "P"),
        &super::v1(&sk, "parent"),
        &super::v2(&sk, "size"),
        &super::v1(&sk, "sym"),
        0.0,
    )
    .len();
    assert!(
        want > 4096,
        "a fixtura tem de ser MAIOR que o tecto removido: {want}"
    );
    assert_eq!(
        built, want,
        "a membrana construiu {built} fitas de {want} — alguma coisa está a cortar a planta"
    );
}

/// O número de voltas (`NonZero`) de um ponto na geometria composta — o mesmo critério com que
/// o renderer a preenche.
fn winding(path: &ph2d_vec_scene::VecPath, q: [f64; 2]) -> i32 {
    let mut w = 0i32;
    let contours = std::iter::once((path.verts.as_slice(), path.closed))
        .chain(path.subpaths.iter().map(|c| (c.verts.as_slice(), c.closed)));
    for (verts, _closed) in contours {
        let n = verts.len();
        for i in 0..n {
            let a = verts[i].anchor;
            let b = verts[(i + 1) % n].anchor;
            // Regra clássica da meia-recta para o número de voltas.
            if a[1] <= q[1] {
                if b[1] > q[1] {
                    let cross = (b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1]);
                    if cross > 0.0 {
                        w += 1;
                    }
                }
            } else if b[1] <= q[1] {
                let cross = (b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1]);
                if cross < 0.0 {
                    w -= 1;
                }
            }
        }
    }
    w
}

/// ⛔⛔⛔ **NENHUMA FENDA NA JUNÇÃO** — o report do Enio de 2026-08-30 (*"no quarto exemplo, com
/// Custom, pequenas fendas"*), medido.
///
/// ⚠️ **A régua é a COBERTURA, não a contagem de contornos.** Um gate que só contasse os discos
/// ficaria verde com o disco no sítio errado ou com raio zero. Este pergunta o que o olho
/// pergunta: *este ponto está pintado?* — pelo mesmo critério (`NonZero`) com que o renderer o
/// preenche.
///
/// A afirmação é a propriedade que o disco compra: **todo ponto a menos de `w/2` da junção está
/// coberto**. É exactamente o que uma cunha por cobrir viola, e a sonda varre um anel inteiro de
/// direcções para não depender de adivinhar de que lado a cunha caiu.
#[test]
fn no_wedge_is_left_uncovered_where_a_branch_meets_its_parent() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    // ⚠️ Quatro gerações, não cinco: a sonda é `O(sondas × vértices)` e a fixtura grande punha
    // o gate em 18 s. `624` ramos já dão `124` juntas, que é população de sobra para a lei.
    state.doc.graph.set_param(n, ls::param::GENERATIONS, 4.0);
    let resolved =
        super::super::motion_externals::resolved_params(&mut state, n, 0.0, &ls::MANIFEST);
    let sk = ls::skeleton("F", "F -> F[+F]F[-F]F", |name: &str| {
        resolved.get(name).copied().unwrap_or(0.0)
    });
    let bs = ls::branch::branches(
        &super::v2(&sk, "P"),
        &super::v1(&sk, "parent"),
        &super::v2(&sk, "size"),
        &super::v1(&sk, "sym"),
        0.0,
    );
    let origin = bs[0].points[0];
    let path = super::plant_geometry(&bs, origin).expect("a planta tem geometria");

    let joints: Vec<_> = bs.iter().filter(|b| b.joins_parent).collect();
    assert!(
        joints.len() > 100,
        "a fixtura tem de ter juntas: {}",
        joints.len()
    );
    let mut naked = 0usize;
    for b in &joints {
        let (p0, w0) = (b.points[0], b.widths[0]);
        let r = f64::from(w0) * 0.5 * 0.6;
        for k in 0..16 {
            let a = std::f64::consts::TAU * f64::from(k) / 16.0;
            let q = [
                f64::from(p0[0] - origin[0]) + r * a.cos(),
                f64::from(p0[1] - origin[1]) + r * a.sin(),
            ];
            if winding(&path, q) == 0 {
                naked += 1;
            }
        }
    }
    assert_eq!(
        naked,
        0,
        "{naked} sondas de {} caíram em cima de uma FENDA — a cunha entre as duas pontas não \
         está coberta",
        joints.len() * 16
    );
}

/// ⛔⛔⛔ **UMA PLANTA PARADA DERIVA UMA VEZ, E NO QUADRO SEGUINTE NÃO DERIVA NADA** — o achado
/// nº 1 da auditoria de seis lentes (2026-08-31, [doc 96](../../../../docs/Motion%20Nodes/96_auditoria_do_lsystem_2026-08-31.md) §2.1).
///
/// # O que o irmão acima não podia ver
///
/// `republishing_an_unchanged_plant_builds_no_geometry_and_survives_the_sweep` mede
/// [`super::ribbons_built`] — o **varrimento booleano** que faz a fita, que é o segundo passo e
/// o único que o memo do `shape_store` protegia. Antes dele corre a **derivação**: a reescrita
/// da gramática (`ls::skeleton`) mais o `ls::branch::branches` que a percorre.
///
/// ⚠️⚠️ **Essa metade corria incondicionalmente, todo quadro**, e o memo só era consultado 74
/// linhas depois. Medido com a máquina calma (load `0,26`, mediana, mesmo processo), na
/// gramática do Bush:
///
/// | gerações | elementos | `skeleton` | `branches` | **por quadro, deitado fora** |
/// |---|---|---|---|---|
/// | 4 | 782 | 0,024 | 0,028 | 0,052 ms |
/// | 5 | 3 907 | 0,148 | 0,162 | 0,310 ms |
/// | 6 | 19 532 | 0,594 | 0,650 | **1,244 ms** |
///
/// ⭐ O modo `Segments` é **plano em ~0,001 ms em qualquer tamanho** — é o memo do cook a
/// acertar, e o nó é `Effect::Pure` **exactamente para isso**. *A razão não tem tecto: ela é o
/// tamanho da planta.*
///
/// ⚠️ **A régua é uma CONTAGEM, pela mesma razão que a do irmão:** um gate de relógio entra na
/// família de flakes de recurso sob fan-out (`CLAUDE.md` §5.0). *Se nada mudou, nada se derivou.*
#[test]
fn a_static_plant_derives_once_and_the_next_frame_derives_nothing() {
    let (mut state, _n) = plant(ls::GEOMETRY_BRANCHES);

    let before = super::plants_derived();
    publish(&mut state, 0.0);
    let first = super::plants_derived() - before;
    assert!(
        first > 0,
        "a 1.ª publicação tem de DERIVAR a planta — senão este gate não mede nada"
    );
    let _ = state.shape_store.sweep();

    let before_second = super::plants_derived();
    publish(&mut state, 0.0);
    let second = super::plants_derived() - before_second;
    assert_eq!(
        second, 0,
        "a 2.ª publicação da MESMA planta DERIVOU-A outra vez ({second} contra {first} na 1.ª) \
         — o trabalho é feito e o memo só é consultado depois, com a resposta já lá"
    );
}

/// ⛔⛔ **O MEMO DA DERIVAÇÃO NÃO CRESCE COM O RELÓGIO** — a segunda metade do achado nº 1.
///
/// A cura do irmão acima acrescentou uma **segunda tabela** endereçada pela mesma chave de
/// conteúdo. E a chave inclui o `Generations` **pelos bits**, então com o slider a ser arrastado
/// ela é nova todo quadro.
///
/// ⚠️⚠️ *Um cache cuja chave pode mudar a 60 Hz não é um cache — é uma fuga com memória.* A
/// frase é do doc do `VecPathStore`, e foi escrita sobre um `wgpu OOM` medido no **quadro
/// 19706** da cena `=76`. Uma tabela nova sob a mesma chave herda o mesmo risco no mesmo dia em
/// que nasce — e a única defesa é a varredura correr sobre as DUAS.
///
/// ⚠️ **A régua é o TAMANHO da tabela depois de N quadros**, não o tempo: determinística, e diz
/// exactamente o que se quer saber.
#[test]
fn the_derivation_memo_is_swept_and_does_not_grow_with_the_clock() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);

    // Vinte quadros, cada um com um `Generations` DIFERENTE — o que um arrasto faz.
    //
    // ⚠️⚠️ **Pela porta do PRODUTO (`publish_all`), e não chamando `sweep()` aqui.** A 1.ª
    // redacção deste gate corria `publish` e depois varria à mão as duas tabelas — e a mutação
    // que APAGA a varredura do produto **sobreviveu**, porque o teste fazia o trabalho que
    // devia estar a auditar. *A lei tinha gate; a ENTREGA não* — que é exactamente o defeito
    // que a auditoria de hoje achou no `tip_taper` (doc 96 §4.3), repetido por mim no mesmo dia.
    for k in 0..20 {
        state
            .doc
            .graph
            .set_param(n, ls::param::GENERATIONS, 3.0 + k as f32 * 0.05);
        crate::render_loop::motion_externals::publish_all(&mut state, 0.0);
        assert!(
            state.lsystem_memo.len() <= 1,
            "depois de {} quadro(s) com chaves diferentes o memo guarda {} derivações — a \
             varredura não o alcança, e ele cresce uma entrada por quadro para sempre",
            k + 1,
            state.lsystem_memo.len()
        );
    }
    // ⚠️ O controlo do próprio gate: uma tabela que nunca guardasse nada também passaria o
    // `<= 1` acima, e não estaria a memoizar coisa nenhuma.
    assert_eq!(
        state.lsystem_memo.len(),
        1,
        "e a derivação do último quadro TEM de estar lá — senão o memo não memoiza nada"
    );
}

/// ⛔⛔ **O `handle_for` É A AUTORIDADE — uma âncora órfã nunca é servida.**
///
/// A cura do achado nº 1 pôs DUAS tabelas sob a mesma chave de conteúdo: a geometria (no
/// `shape_store`) e a derivação (no `PlantMemo`). Elas nascem e são varridas juntas — mas *"são
/// varridas juntas"* é uma afirmação sobre código, e este gate é o que a torna falsificável.
///
/// # O caso que ele encena
///
/// O memo perde a entrada e o store fica com a geometria. Se a membrana acreditasse no memo, ela
/// saltava a derivação (o store TEM o handle), ia buscar a origem e as âncoras a uma tabela
/// vazia, e **publicava uma corrente vazia** — a planta desaparecia, com a geometria intacta a
/// dois passos de distância.
///
/// ⚠️ **A guarda certa é a mais barata**: perguntar às duas e re-derivar se discordarem. Sem
/// este gate ela é código defensivo que nenhuma mutação mata — *e código defensivo que ninguém
/// pode falsificar é indistinguível de código morto.*
#[test]
fn a_memo_that_lost_its_derivation_is_re_derived_and_not_served_empty() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    publish(&mut state, 0.0);
    let key = key_of(&mut state, n);
    let cheia = published(&state, &key).unwrap_or(0);
    assert!(cheia > 0, "a 1.ª publicação tem de dar uma planta");
    let geometrias = state.shape_store.len();

    // A dessincronia: o memo esquece, o store lembra. (Duas varreduras sem ninguém pedir a
    // chave esvaziam o memo; o store não é varrido.)
    // ⚠️ DUAS: a primeira PRESERVA (o `publish` marcou a chave viva) e limpa a lista de vivos;
    // a segunda é que a deita fora. É a mesma escada do `VecPathStore`.
    state.lsystem_memo.sweep();
    state.lsystem_memo.sweep();
    assert_eq!(state.lsystem_memo.len(), 0, "o memo tem de ficar vazio");
    assert_eq!(
        state.shape_store.len(),
        geometrias,
        "e o store tem de MANTER a geometria — senão não há dessincronia para medir"
    );

    publish(&mut state, 0.0);
    assert_eq!(
        published(&state, &key).unwrap_or(0),
        cheia,
        "a planta saiu VAZIA (ou diferente) depois de o memo perder a derivação — a membrana \
         acreditou numa tabela que o `handle_for` já não confirma"
    );
    assert_eq!(
        state.lsystem_memo.len(),
        1,
        "e a derivação tem de estar de volta"
    );
}
