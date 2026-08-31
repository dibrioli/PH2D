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
use ph2d_vec_scene::{VecPath, VecVertex};

use super::motion_lsystem_leaves::{
    Job, anchors_of, say_if_a_wire_drives_an_inert_param, say_if_the_leaf_cannot_go_in_front,
    say_if_the_letter_is_missing, say_if_the_level_hid_every_leaf,
};
use super::motion_lsystem_rows::plant_and_leaves;
use crate::motion_state::MotionState;

// ⛔⛔ **O TECTO `MAX_RIBBONS = 4096` FOI REMOVIDO — ele não era de recurso nenhum.**
//
// Report do Enio (2026-08-30): *"[lsystem] 9841 ramos passam do tecto de 4096 — a planta sai
// cortada"*. Ele shipou com o doc a dizer **«este número está por MEDIR»**, que é exactamente o
// que o `CLAUDE.md` §0.0 proíbe: *um limite que só diz «por segurança» é um palpite à espera de
// um smoke*. E a justificação que ele DAVA (*«cada fita é um `VecPath` e paga uma tesselação»*)
// **dissolveu** na mesma jornada, quando a planta passou a ser UMA geometria composta — quem
// move o número que tornava algo inalcançável tem de reconferir a nota.
//
// A medição, com os trilhos analíticos (release, caminho FRIO repetido, gramática
// `F -> F[+F]F[-F]F`):
//
// | gerações | ramos | publicar | por ramo |
// |---|---|---|---|
// | 4 | 624 | 0,113 ms | 0,181 µs |
// | 5 | 3 124 | 0,513 ms | 0,164 µs |
// | 6 | 15 624 | 1,377 ms | 0,088 µs |
// | **7** | **78 124** | **7,17 ms** | 0,092 µs |
// | 8 | 78 124 | 6,75 ms | 0,086 µs |
//
// ⭐ **A contagem de ramos JÁ É LIMITADA a montante**, pelo `MAX_MODULES = 262 144` do próprio
// nó — a `g = 8` a cadeia satura e os ramos param em `78 124`. E nesse ponto a publicação
// INTEIRA (derivar + decompor + construir as fitas) custa `7,17 ms`, a mesma ordem que o nó já
// declara para si (*"38,8 % de um quadro"* para a derivação no seu tecto). *O segundo tecto não
// protegia nada: só cortava a planta.*

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
pub(crate) fn v2(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Uma coluna escalar do esqueleto, ou vazia.
pub(crate) fn v1(s: &Stream, name: &str) -> Vec<f32> {
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
///
/// ⛔⛔ **E os trilhos são construídos AQUI, não pelo `power_stroke` — a 2.ª queda de fps.**
/// Aquele motor **densifica cada contorno em `RIBBON_SAMPLES = 128` amostras** e corre um
/// varrimento booleano (`Region::of`) sobre o resultado. É a coisa certa para uma curva com um
/// perfil liso; para um ramo de **três pontos** com uma parada de largura em cada um, as 128
/// amostras não acrescentam informação nenhuma e o varrimento custa ~3 µs por ramo — que numa
/// planta de 9 841 ramos é o quadro inteiro.
///
/// ⚠️ **Não é uma segunda lei de traço variável, e a diferença é o PRODUTO:** o `power_stroke`
/// devolve um contorno REGULARIZADO (auto-intersecções resolvidas, pronto para outra booleana);
/// aqui a saída é um contorno de PREENCHIMENTO que entra na planta composta sob `NonZero` — a
/// regra de preenchimento já resolve sobreposição, que é a única coisa que o varrimento
/// comprava. *A geometria dos trilhos é a mesma: `±w/2` na normal do vértice.*
fn ribbon(b: &ls::branch::Branch, origin: [f32; 2]) -> Option<ph2d_vec_scene::Contour> {
    RIBBONS_BUILT.with(|c| c.set(c.get() + 1));
    let n = b.points.len();
    if n < 2 || b.widths.len() != n {
        return None;
    }
    let pt = |i: usize| {
        [
            f64::from(b.points[i][0] - origin[0]),
            f64::from(b.points[i][1] - origin[1]),
        ]
    };
    // A direcção de cada SEGMENTO, já normalizada. Um segmento de comprimento zero herda a
    // direcção do anterior — ele não tem normal própria, e inventar uma poria um pico na fita.
    let mut dir = Vec::with_capacity(n - 1);
    let mut last = [1.0f64, 0.0];
    for i in 0..n - 1 {
        let (a, c) = (pt(i), pt(i + 1));
        let (dx, dy) = (c[0] - a[0], c[1] - a[1]);
        let len = dx.hypot(dy);
        last = if len > 1e-9 {
            [dx / len, dy / len]
        } else {
            last
        };
        dir.push(last);
    }

    // ⭐ Os DOIS TRILHOS, a `±w/2` da linha de centro na normal do vértice.
    let mut left = Vec::with_capacity(n);
    let mut right = Vec::with_capacity(n);
    for i in 0..n {
        let d_in = dir[i.saturating_sub(1)];
        let d_out = dir[i.min(n - 2)];
        // A normal do vértice é a bissectriz das duas normais vizinhas; o factor de esquadria
        // (`1/cos(θ/2)`) devolve a largura pedida MEDIDA PERPENDICULARMENTE ao segmento.
        let (mx, my) = (d_in[0] + d_out[0], d_in[1] + d_out[1]);
        let m = mx.hypot(my);
        let (tx, ty) = if m > 1e-9 {
            (mx / m, my / m)
        } else {
            // Meia-volta exacta: as duas direcções cancelam-se e não há bissectriz. Usa a
            // normal de entrada — a fita dobra sobre si, e o `NonZero` resolve a sobreposição.
            (d_in[0], d_in[1])
        };
        let (nx, ny) = (-ty, tx);
        // ⚠️ **A esquadria é CLAMPADA, e o recurso tem nome:** numa quina fechada `1/cos(θ/2)`
        // vai a infinito, e um vértice no infinito parte a tesselação. `4` é o limite de
        // esquadria de facto do SVG e do Illustrator (`stroke-miterlimit` nasce em `4`), e
        // acima dele a ponta é cortada — a sobreposição do `NonZero` fecha o resto.
        const MITER_MAX: f64 = 4.0;
        let cos_half = (nx * -d_out[1] + ny * d_out[0]).abs().max(1.0 / MITER_MAX);
        let h = f64::from(b.widths[i]) * 0.5 / cos_half;
        let (px, py) = (pt(i)[0], pt(i)[1]);
        left.push([px + nx * h, py + ny * h]);
        right.push([px - nx * h, py - ny * h]);
    }

    // O contorno: um trilho para a frente, o outro para trás. As pontas fecham-se sozinhas
    // (topo recto) — e uma ponta afinada a zero fecha num PONTO, que é o que se quer.
    let mut verts: Vec<VecVertex> = Vec::with_capacity(n * 2);
    verts.extend(left.iter().map(|p| VecVertex::corner(*p)));
    verts.extend(right.iter().rev().map(|p| VecVertex::corner(*p)));
    Some(ph2d_vec_scene::Contour {
        verts,
        closed: true,
    })
}

/// **A geometria de uma PLANTA INTEIRA** — um `VecPath` composto com um contorno por ramo.
///
/// ⚠️ **Uma tesselação por planta, não uma por ramo.** Ver [`ribbon`] para o número que obrigou
/// a isto.
///
/// ⚠️ **`FillRule::NonZero`**: os ramos SOBREPÕEM-SE na junção de propósito (é o colar que fecha
/// a forquilha), e com par-ímpar a sobreposição viraria um BURACO — exactamente o defeito que o
/// colar veio curar, de volta por outra porta.
/// **A JUNTA REDONDA** de um ramo que nasce noutro — o disco que fecha a cunha.
///
/// ⛔⛔ **Report do Enio (2026-08-30), 4.ª planta: *"pequenas fendas"*.** Duas fitas que se
/// encontram no MESMO ponto e com a MESMA largura ainda deixam um vão: as pontas delas são
/// perpendiculares a **direcções diferentes**, e entre as duas sobra uma cunha de ângulo `θ` e
/// raio `w/2` por cobrir. *O colar curou a LARGURA da junção; a ORIENTAÇÃO das duas pontas
/// ficou por curar, e é ela que se vê como um risco fino.*
///
/// ⇒ um disco de raio `w/2` centrado no ponto de junção cobre a cunha **qualquer que seja o
/// ângulo** — é a *round join* que todo desenhador de traço usa, e a única que não precisa de
/// saber quanto a curva vira.
///
/// ⚠️ **`JOIN_SIDES = 12`, e o recurso é o RAIO EM PIXELS.** Um dodecágono difere de um círculo
/// em `1 − cos(π/12) = 3,4 %` do raio; nas juntas mais grossas desta cena isso é fracção de
/// pixel, e nas finas é invisível. Mais lados é geometria paga em toda junta de toda planta.
fn join_disc(centre: [f64; 2], radius: f64) -> Option<ph2d_vec_scene::Contour> {
    const JOIN_SIDES: usize = 12;
    if !(radius.is_finite() && radius > 0.0) {
        return None;
    }
    let verts = (0..JOIN_SIDES)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let a = std::f64::consts::TAU * k as f64 / JOIN_SIDES as f64;
            VecVertex::corner([centre[0] + radius * a.cos(), centre[1] + radius * a.sin()])
        })
        .collect();
    Some(ph2d_vec_scene::Contour {
        verts,
        closed: true,
    })
}

/// **TODO CONTORNO SAI NO MESMO SENTIDO** — a lei sem a qual o `NonZero` faz buracos.
///
/// ⛔⛔ **Apanhado pelo gate da fenda, e é uma armadilha de primeira ordem.** Sob `NonZero` duas
/// voltas de SINAIS OPOSTOS **cancelam**: onde a junta cobria a fita o número de voltas dava
/// `0`, e a regra de preenchimento abria ali um buraco — *a cura da fenda a produzir uma fenda,
/// no mesmo sítio*. A fita nasce com o sentido que a normal do vértice lhe dá; o disco nascia
/// com o sentido do ângulo a crescer. Nada obrigava os dois a concordar.
///
/// ⇒ a área com sinal decide, e quem estiver ao contrário é invertido. É `O(n)` sobre pontos
/// que já estão na mão, e vale para todo contorno futuro sem ninguém ter de se lembrar disto.
fn face_the_same_way(c: &mut ph2d_vec_scene::Contour) {
    let n = c.verts.len();
    if n < 3 {
        return;
    }
    let twice_area: f64 = (0..n)
        .map(|i| {
            let a = c.verts[i].anchor;
            let b = c.verts[(i + 1) % n].anchor;
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    if twice_area < 0.0 {
        c.verts.reverse();
    }
}

fn plant_geometry(branches: &[ls::branch::Branch], origin: [f32; 2]) -> Option<VecPath> {
    let mut contours: Vec<ph2d_vec_scene::Contour> = Vec::with_capacity(branches.len() * 2);
    for b in branches {
        if let Some(c) = ribbon(b, origin) {
            contours.push(c);
        }
        // ⭐ A junta, e **só onde há junta**: uma raiz não nasce em ninguém, e um disco na base
        // arredondaria o pé do tronco — uma mudança de forma que ninguém pediu.
        if b.joins_parent
            && let (Some(p0), Some(w0)) = (b.points.first(), b.widths.first())
            && let Some(c) = join_disc(
                [f64::from(p0[0] - origin[0]), f64::from(p0[1] - origin[1])],
                f64::from(*w0) * 0.5,
            )
        {
            contours.push(c);
        }
    }
    for c in &mut contours {
        face_the_same_way(c);
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
    let mut jobs: Vec<Job> = Vec::new();
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
        // ⭐ As ÂNCORAS que as letras plantaram, e os três nomes que dizem o quê.
        let anchors = anchors_of(&sk);
        let names = [
            text(ls::LEAF_PARAMS[0]),
            text(ls::LEAF_PARAMS[1]),
            text(ls::LEAF_PARAMS[2]),
        ];
        // ⚠️ **Os nomes conduzidos por FIO** — a lista que separa «o artista pôs um LFO aqui»
        // de «este param está no default».
        let driven: Vec<&'static str> = ls::MANIFEST
            .params
            .iter()
            .map(|p| p.name)
            .filter(|n| {
                motion
                    .doc
                    .graph
                    .param_sources(id)
                    .is_some_and(|m| m.contains_key(*n))
            })
            .collect();
        say_if_a_wire_drives_an_inert_param(&key, &driven, get(ls::param::TROPISM));
        jobs.push((
            key,
            bs,
            anchors,
            names,
            get(ls::param::LEAF_FIRST_LEVEL),
            super::motion_lsystem_rows::LeafLook {
                front: get(ls::param::LEAF_FRONT),
                keep_own_colour: get(ls::param::LEAF_EFFECTS).round() as i32 == 0,
                size: get(ls::param::LEAF_SIZE),
                size_jitter: get(ls::param::LEAF_SIZE_JITTER),
                pos_jitter: get(ls::param::LEAF_POS_JITTER),
            },
        ));
    }

    for (key, bs, anchors, names, first_level, look_law) in jobs {
        let used = &bs[..];
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
            Some(h) => {
                say_if_the_letter_is_missing(&key, &names, &anchors);
                say_if_the_level_hid_every_leaf(&key, &names, &anchors, first_level);
                say_if_the_leaf_cannot_go_in_front(
                    &key,
                    &names,
                    &core::array::from_fn(|i| {
                        super::motion_lsystem_leaves::named_appearance(&motion.pump.cook, &names[i])
                    }),
                    look_law.front,
                );
                plant_and_leaves(origin, h, &anchors, &names, &motion.pump.cook, look_law)
            }
            None => Stream::new(0),
        };
        motion.pump.cook.set_external(key, stream);
    }
}

#[cfg(test)]
#[path = "motion_lsystem_gen_tests.rs"]
mod tests;
