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

/// **A fita de um ramo** — a linha de centro em coordenadas LOCAIS e o perfil de largura.
///
/// ⚠️ **Local, e a origem é o primeiro ponto do ramo.** A pose vive na instância (`P`), como em
/// toda a casa: assim um `motion.move` a jusante desloca o ramo INTEIRO como um objeto — que é
/// literalmente o que o report pediu (*"crescer como um objeto só"*) — e duas plantas iguais em
/// sítios diferentes partilham a geometria.
fn ribbon(b: &ls::branch::Branch) -> Option<VecPath> {
    let base = *b.points.first()?;
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
    let mut out = if profile.is_uniform() {
        ph2d_vec_boolean::outline_stroke(&centre)
    } else {
        ph2d_vec_boolean::power_stroke(&centre, &profile)
    };
    if out.is_empty() {
        return None;
    }
    // ⚠️ **Um `VecPath` COMPOSTO, não N instâncias.** O varrimento pode devolver mais de um
    // contorno (uma cúspide onde a curvatura aperta mais que a largura); publicá-los como
    // instâncias separadas faria um ramo ser dois objectos, que é o defeito que esta wave veio
    // curar, um nível abaixo.
    let mut first = out.remove(0);
    for extra in out {
        first.subpaths.push(ph2d_vec_scene::Contour {
            verts: extra.verts,
            closed: extra.closed,
        });
    }
    Some(first)
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
        let mut p = Vec::with_capacity(bs.len());
        let mut ids = Vec::with_capacity(bs.len());
        let mut sizes = Vec::with_capacity(bs.len());
        let clipped = bs.len() > MAX_RIBBONS;
        for (i, b) in bs.iter().take(MAX_RIBBONS).enumerate() {
            // A chave de CADA fita é a do nó mais o índice do ramo — o store guarda uma
            // geometria por ramo, e duas plantas idênticas partilham as duas.
            let bkey = format!("{key}\u{2}{i}");
            let Some(base) = b.points.first().copied() else {
                continue;
            };
            let handle = {
                let built = ribbon(b);
                match built {
                    Some(path) => motion.shape_store.intern(&bkey, || path),
                    None => continue,
                }
            };
            p.push(base);
            sizes.push([1.0f32, 1.0]);
            ids.push(handle as f32);
        }
        if clipped {
            // ⚠️ **Um tecto que morde em silêncio lê-se como *«a planta está incompleta»***, e a
            // causa é indistinguível de uma gramática errada. Ver [`MAX_RIBBONS`].
            eprintln!(
                "[lsystem] {} ramos passam do tecto de {MAX_RIBBONS} — a planta sai cortada",
                bs.len()
            );
        }
        let n = p.len();
        let stream = Stream::new(n)
            .with("P", Column::Vec2(p))
            .with("size", Column::Vec2(sizes))
            .with("geometry_id", Column::Scalar(ids))
            .with(
                "Index",
                Column::Scalar((0..n).map(|i| i as f32).collect::<Vec<_>>()),
            )
            .with("Count", Column::Scalar(vec![n as f32; n]));
        motion.pump.cook.set_external(key, stream);
    }
}

#[cfg(test)]
#[path = "motion_lsystem_gen_tests.rs"]
mod tests;
