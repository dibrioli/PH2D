//! **A metade da SHELL do modo `Branches`** — o esqueleto vira FITA.
//!
//! Report do Enio (2026-08-30): *"as formas crescem sempre separadas e não crescem como um
//! objeto só. O tronco deve ter uma estrutura única e não vários retângulos soltos
//! sobrepostos."*
//!
//! O nó decide *quais pontos formam um ramo* ([`ph2d_node_source_lsystem::branch`]); aqui a
//! polilinha de cada ramo vira **contorno preenchido**, com a largura a seguir a espessura da
//! tartaruga. É a segunda metade da lei que as quatro referências partilham — *um ramo é uma
//! curva com uma função de raio, varrida* (estudo no
//! [doc 95](../../../../docs/Motion%20Nodes/95_estudo_ramificacao_continua_e_instancias.md)).
//!
//! ⚠️ **Está aqui, e não no nó, por causa da cerca do ADR-0154:** um nó não alcança a
//! biblioteca vetorial nem a GPU, e é essa propriedade que deixa o cook memoizar e repetir ao
//! bit. O molde é o `source.shape`: o nó descreve, a shell constrói, interna sob a chave de
//! CONTEÚDO e publica; o `eval` clona.
//!
//! ⚠️ **O varrimento não é nosso**: o `power_stroke` já é o motor clássico do
//! Inkscape/Illustrator (dois trilhos deslocados por `±w(s)/2` na normal, tampas nas pontas, e
//! o sweep a regularizar cúspides). Reimplementá-lo aqui daria duas leis de traço variável a
//! divergir na borda, que é o único sítio onde ninguém lê um número.

use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_vec_scene::{VecPath, VecVertex, WidthStop, WidthStops};

use crate::motion_state::MotionState;

/// **Quantas fitas um nó publica, no máximo.**
///
/// ⚠️ **O recurso é a TESSELAÇÃO, não a memória.** Cada fita é um `VecPath` próprio no store e
/// paga uma tesselação por quadro em que muda; o tecto de módulos do nó
/// (`MAX_MODULES = 262 144`) é sobre a CADEIA, e uma cadeia daquele tamanho numa gramática que
/// bifurca dá dezenas de milhares de ramos. ⏳ **Este número está por MEDIR** — fica no valor
/// que o nó já declara para instâncias vivas de vetor até alguém correr a varredura, e a linha
/// de diagnóstico abaixo diz quando ele morde.
const MAX_RIBBONS: usize = 4096;

// **QUANTAS FITAS FORAM DE FACTO CONSTRUÍDAS** — a sonda que o gate do memo precisava e não
// tinha.
//
// ⛔⛔ **A 1.ª régua deste gate media `VecPathStore::len()` e a mutação SOBREVIVEU:** o
// `intern` deduplica por chave, então uma fita construída à toa é **descartada em silêncio** e
// a contagem de guardadas não se mexe. *O `len` conta o que foi GUARDADO; o desperdício é o que
// foi CONSTRUÍDO e deitado fora, e são grandezas diferentes.*
//
// ⚠️ Mesmo desenho que o `MotionFx::dirt_rebinds` já ship pela mesma razão: um custo que só
// aparece quando alguém o CONTA.
//
// ⚠️⚠️ **POR THREAD, e não global — apanhado pelo próprio gate.** Com um `AtomicUsize` de
// processo o contador soma as construções dos OUTROS testes que correm em paralelo, e a
// segunda publicação media `62` onde a resposta é `0`. *Um contador global medido dentro de
// uma suíte paralela mede a suíte, não o caso.* Na shell o laço de desenho é uma thread só,
// então a contagem por thread é a mesma que a de processo — sem a corrida.
thread_local! {
    static RIBBONS_BUILT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Quantas fitas o processo já construiu (ver [`RIBBONS_BUILT`]).
///
/// ⚠️ `cfg(test)` no ACESSOR e não no contador: o `fetch_add` fica sempre (é um add relaxado,
/// e tirá-lo faria o produto medido divergir do produto que ship).
#[cfg(test)]
pub(crate) fn ribbons_built() -> usize {
    RIBBONS_BUILT.with(std::cell::Cell::get)
}

/// Uma coluna `Vec2` do esqueleto, ou vazia.
fn v2(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Uma coluna escalar do esqueleto, ou vazia.
fn v1(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **A fita de um ramo**, em coordenadas locais à PLANTA — os contornos preenchidos que ela
/// contribui para a geometria da planta inteira.
///
/// ⚠️⚠️ **A origem é a da PLANTA, e não a do ramo — e a diferença é o relógio.** A 1.ª redacção
/// dava um `VecPath` por ramo, cada um com origem própria, e a planta saía como N instâncias
/// com N geometrias DISTINTAS. O desenho tesselá-las-ia **todas, todo o quadro** (o cache do
/// renderer é por `geometry_id` e por quadro), e foi isso — mais o memo que não era usado — que
/// deu o *"ficamos com 4 fps"*. Uma planta é UM objecto: um `VecPath` composto, uma tesselação.
fn ribbon(b: &ls::branch::Branch, origin: [f32; 2]) -> Option<Vec<VecPath>> {
    RIBBONS_BUILT.with(|c| c.set(c.get() + 1));
    let base = origin;
    b.points.first()?;
    let mut centre = VecPath {
        verts: b
            .points
            .iter()
            .map(|p| VecVertex::corner([f64::from(p[0] - base[0]), f64::from(p[1] - base[1])]))
            .collect(),
        closed: false,
        ..VecPath::default()
    };

    // ⚠️ **O perfil é MULTIPLICADOR, e a largura de referência é a MAIOR do ramo.** O
    // `WidthStop::mult` escala a largura do traço, então o traço vale `w_max` e cada parada
    // vale `w_i / w_max` — assim a base fica em `1` e a ponta afina. Normalizar pela PRIMEIRA
    // daria multiplicadores acima de `1` num ramo que engrossa (um `!` que aumenta), e o
    // motor de traço não é obrigado a gostar disso.
    let w_max = b.widths.iter().copied().fold(0.0f32, f32::max);
    // ⚠️ `is_finite` **e** `> 0`: um ramo de largura zero não é uma fita, e um `NaN` vindo de um
    // param conduzido faria o `mult` de cada parada ser `NaN` — o varrimento devolveria lixo em
    // vez de vazio.
    if !w_max.is_finite() || w_max <= 0.0 {
        return None;
    }
    let frac = b.arc_fractions();
    let stops: Vec<WidthStop> = frac
        .iter()
        .zip(&b.widths)
        .map(|(t, w)| WidthStop {
            pos: f64::from(*t),
            mult: f64::from(*w / w_max),
        })
        .collect();

    // ⚠️ **A largura do traço é a MAIOR do ramo, e é uma LARGURA (não meia)** — o `power_stroke`
    // desloca por `±w/2`, então passar a espessura da tartaruga tal e qual dá um ramo com
    // exactamente a espessura que o `!` da gramática pediu.
    // ⚠️ A COR aqui é inerte: o `power_stroke` devolve a forma PREENCHIDA da fita, e quem a
    // pinta é o `tint` da instância a jusante. O branco é o neutro de multiplicação.
    centre.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        ph2d_vec_scene::Rgba8::new(255, 255, 255, 255),
        f64::from(w_max),
    ));

    // ⚠️⚠️ **DOIS motores, e a pergunta escolhe qual** — apanhado por gate, na 1.ª corrida.
    //
    // O `power_stroke` **devolve vazio de propósito** quando o perfil é UNIFORME: *"aí o comando
    // é o `outline_stroke`, e ter dois botões para a mesma saída seria pior que ter um"* (o doc
    // dele). E uma planta sem nenhum `!` na gramática tem largura constante em todo o ramo —
    // que é o caso COMUM, não a excepção. Chamar só o de largura variável fazia a planta inteira
    // desaparecer, com a membrana a publicar contagem `0` e nada vermelho em lado nenhum.
    let profile = WidthStops::new(stops);
    let out = if profile.is_uniform() {
        ph2d_vec_boolean::outline_stroke(&centre)
    } else {
        ph2d_vec_boolean::power_stroke(&centre, &profile)
    };
    if out.is_empty() {
        return None;
    }
    Some(out)
}

/// **A geometria de uma PLANTA INTEIRA** — um `VecPath` composto com um contorno por ramo.
///
/// ⚠️ **Uma tesselação por planta, não uma por ramo.** Ver [`ribbon`] para o número que obrigou
/// a isto.
///
/// ⚠️ **`FillRule::NonZero`**: os ramos SOBREPÕEM-SE na junção de propósito (é o colar que fecha
/// a forquilha), e com par-ímpar a sobreposição viraria um BURACO — exactamente o defeito que o
/// colar veio curar, de volta por outra porta.
fn plant_geometry(branches: &[ls::branch::Branch], origin: [f32; 2]) -> Option<VecPath> {
    let mut contours: Vec<ph2d_vec_scene::Contour> = Vec::new();
    for b in branches {
        for c in ribbon(b, origin).into_iter().flatten() {
            contours.push(ph2d_vec_scene::Contour {
                verts: c.verts,
                closed: c.closed,
            });
        }
    }
    if contours.is_empty() {
        return None;
    }
    let first = contours.remove(0);
    Some(VecPath {
        verts: first.verts,
        closed: first.closed,
        subpaths: contours,
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        ..VecPath::default()
    })
}

/// **Publica as fitas** de cada `source.lsystem` em modo `Branches`.
///
/// ⚠️ **Chamada de [`super::motion_externals::publish_all`]**, ao lado das outras quatro
/// membranas e **antes** da varredura do store — que é o que impede as geometrias deste quadro
/// de serem apagadas antes de alguém as pedir.
pub(crate) fn publish(motion: &mut MotionState, seconds: f64) {
    let ids: Vec<ph2d_nodegraph::graph::NodeId> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ls::MANIFEST.name)
        .map(|n| n.id)
        .collect();

    // Junta os trabalhos primeiro: o empréstimo do grafo tem de cair antes de mexer no store e
    // no cook (três campos disjuntos do `MotionState`).
    let mut jobs: Vec<(String, Vec<ls::branch::Branch>)> = Vec::new();
    for id in ids {
        let resolved = super::motion_externals::resolved_params(motion, id, seconds, &ls::MANIFEST);
        let get = |name: &str| resolved.get(name).copied().unwrap_or(0.0);
        if get(ls::param::GEOMETRY).round() as i32 != ls::GEOMETRY_BRANCHES {
            continue;
        }
        let texts = motion.doc.graph.node_text_param_overrides(id);
        let text = |k: &str| texts.and_then(|m| m.get(k)).cloned().unwrap_or_default();
        let (axiom, rules) = (text(ls::AXIOM_PARAM), text(ls::RULES_PARAM));
        // ⚠️ A chave sai da MESMA função que o `eval` chama — dois nomes divergiriam e a planta
        // desapareceria sem erro nenhum.
        let key = ls::ribbon_key(get, &axiom, &rules);
        let sk = ls::skeleton(&axiom, &rules, get);
        let bs = ls::branch::branches(
            &v2(&sk, "P"),
            &v1(&sk, "parent"),
            &v2(&sk, "size"),
            &v1(&sk, "sym"),
            // ⭐ O afinamento da ponta vem do PAINEL, e chega aqui pela mesma escada resolvida
            // que cunha a chave — senão a fita seria construída com um valor e memoizada com
            // outro.
            get(ls::param::TIP_TAPER),
        );
        jobs.push((key, bs));
    }

    for (key, bs) in jobs {
        let clipped = bs.len() > MAX_RIBBONS;
        if clipped {
            // ⚠️ **Um tecto que morde em silêncio lê-se como *«a planta está incompleta»***, e a
            // causa é indistinguível de uma gramática errada. Ver [`MAX_RIBBONS`].
            eprintln!(
                "[lsystem] {} ramos passam do tecto de {MAX_RIBBONS} — a planta sai cortada",
                bs.len()
            );
        }
        let used = &bs[..bs.len().min(MAX_RIBBONS)];
        // ⚠️ **A origem da planta é o primeiro ponto do primeiro ramo**, e a geometria inteira é
        // local a ela: a pose viaja na instância, como em toda a casa, e duas plantas iguais em
        // sítios diferentes partilham UMA geometria.
        let Some(origin) = used.first().and_then(|b| b.points.first().copied()) else {
            motion.pump.cook.set_external(key, Stream::new(0));
            continue;
        };
        // ⛔⛔ **PERGUNTAR ANTES DE CONSTRUIR — e a 1.ª redacção fazia o contrário.**
        //
        // Report do Enio (2026-08-30): *"ficamos com 4 fps"*. Ela chamava o construtor e só
        // depois entregava o resultado ao `intern`, que **não** o teria chamado com a chave já
        // internada. O memo estava lá, correcto, e **nunca era usado**: cada quadro re-corria o
        // varrimento booleano de todos os ramos de todas as plantas (medido: **3 124 fitas por
        // quadro** só na fixtura do gate).
        //
        // ⚠️ *Um `intern(chave, || construir())` só poupa se o `construir` for PREGUIÇOSO;
        // passar-lhe um valor já construído é escrever o memo e pagar na mesma.* O
        // `source.shape` sempre fez `intern(&key, || build_shape_path(&p))` — a diferença está
        // inteira no `||`.
        //
        // ⚠️ O `handle_for` é a metade de CONSULTA e **marca a chave como viva** (o doc dele diz
        // porquê): sem isso a varredura do fim do quadro apagaria exactamente as geometrias que
        // estão a ser desenhadas, e a reconstrução voltava por outra porta.
        let handle = match motion.shape_store.handle_for(&key) {
            Some(h) => Some(h),
            None => {
                plant_geometry(used, origin).map(|path| motion.shape_store.intern(&key, || path))
            }
        };
        let stream = match handle {
            Some(h) => Stream::new(1)
                .with("P", Column::Vec2(vec![origin]))
                .with("size", Column::Vec2(vec![[1.0f32, 1.0]]))
                .with("geometry_id", Column::Scalar(vec![h as f32]))
                .with("Index", Column::Scalar(vec![0.0]))
                .with("Count", Column::Scalar(vec![1.0])),
            None => Stream::new(0),
        };
        motion.pump.cook.set_external(key, stream);
    }
}

#[cfg(test)]
#[path = "motion_lsystem_gen_tests.rs"]
mod tests;
