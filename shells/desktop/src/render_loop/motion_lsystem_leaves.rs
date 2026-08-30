//! **AS TRÊS LETRAS QUE PLANTAM UM OBJECTO** — `J` · `K` · `M`, a metade do modo `Branches`
//! que não é geometria de ramo.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! RESPONSABILIDADE: o irmão [`super::motion_lsystem_gen`] responde *como um ramo vira uma
//! forma*, e este responde *onde um objecto nasce, virado para onde, de que tamanho e com que
//! alfa*.
//!
//! Report do Enio (2026-08-30): *"as folhas não crescem, elas aparecem e sem rotação [relativa]
//! ao galho. Elas não nascem e crescem na ponta dos galhos, elas aparecem em cada segmento. O
//! Alpha usado escurece as bordas da pintura (diferente da sprite)."* — três queixas, três
//! causas distintas, e o doc de cada função abaixo tem a sua
//! ([doc 95 §6](../../../../docs/Motion%20Nodes/95_estudo_ramificacao_continua_e_instancias.md)).

use super::motion_lsystem_gen::{v1, v2};
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::{Column, Stream};

/// **Uma âncora de instância** — onde uma letra `J`/`K`/`M` pousou.
///
/// ⚠️ Ela leva o ÂNGULO, e não só a posição: o doc da tartaruga diz porquê — *"uma marca não
/// tem osso, mas TEM direcção: ela aponta como o ramo em que pousou"*. Uma folha que ignorasse
/// isso ficaria toda virada para o mesmo lado numa planta que se abre em leque.
///
/// ⛔⛔ **E era EXACTAMENTE isso que acontecia** — report do Enio (2026-08-30): *"sem rotação
/// [relativa] ao galho"*. A 1.ª redacção lia o `wrot` do esqueleto e publicava-o numa coluna
/// chamada **`rotation`** — um nome que **ninguém lê**: a convenção de instâncias do Motion
/// chama-lhe **`rot`** (`ph2d-eval-motion`, em GRAUS). *Uma coluna com o nome errado não dá
/// erro nenhum: ela é ignorada, e o default é a identidade.*
///
/// ⚠️ E a fonte passa a ser a coluna **`rot`** do esqueleto, não o `wrot`: é ela que honra o
/// param `Orient` (mundo · local) que o artista já tem, e é ela que o modo `Segments` publica.
/// *Duas rotas a decidir a orientação de maneiras diferentes é o defeito, não a redundância.*
pub(crate) struct Anchor {
    pub(crate) p: [f32; 2],
    pub(crate) rot: f32,
    /// **Quanto esta marca já abriu**, `0..1` — a coluna `mark_grow` do esqueleto.
    ///
    /// ⭐ A lei é do NÓ (`ph2d_node_source_lsystem::turtle::mark_grow`), porque só ele tem o
    /// plano de gerações. Aqui ela é um multiplicador do tamanho: *nasce pequena na ponta e
    /// cresce*, e a marca da geração anterior encolhe pelo complemento.
    pub(crate) grow: f32,
    /// O índice em [`ls::LEAF_SYMBOLS`] — `0` = `J`, `1` = `K`, `2` = `M`.
    pub(crate) slot: usize,
}

/// As âncoras que as três letras plantaram, lidas do esqueleto.
pub(crate) fn anchors_of(sk: &Stream) -> Vec<Anchor> {
    let (p, sym, rot) = (v2(sk, "P"), v1(sk, "sym"), v1(sk, "rot"));
    let grow = v1(sk, "mark_grow");
    p.iter()
        .zip(sym.iter())
        .enumerate()
        .filter_map(|(i, (pos, s))| {
            let byte = *s as i32 as u8;
            let slot = ls::LEAF_SYMBOLS.iter().position(|k| *k == byte)?;
            Some(Anchor {
                p: *pos,
                rot: rot.get(i).copied().unwrap_or(0.0),
                // ⚠️ **Ausente ⇒ `1`**, o valor maduro: uma coluna que não existe não pode
                // apagar a folha de um esqueleto vindo de outra rota.
                grow: grow.get(i).copied().unwrap_or(1.0),
                slot,
            })
        })
        .collect()
}

/// **A aparência de um objecto**, na ordem `(size, tint, uv_rect, texture_id, premultiplied)`.
///
/// ⚠️ O 5.º campo entrou pelo report de 2026-08-30 (*"o Alpha escurece as bordas da pintura"*):
/// ele é da TEXTURA, e sem ele o lowering pré-multiplicava outra vez ⇒ `RGB·α²`.
pub(crate) type Look = ([f32; 2], [f32; 4], [f32; 4], f32, f32);

/// **O trabalho de uma planta**, já resolvido: `(chave, ramos, âncoras, os três nomes)`.
pub(crate) type Job = (String, Vec<ls::branch::Branch>, Vec<Anchor>, [String; 3]);

/// A aparência que um objecto NOMEADO publicou — `(size, tint, uv_rect, texture_id, premul)`.
///
/// ⭐⭐ **Lida do canal externo, e não resolvida outra vez.** O `publish_objects` já pôs a
/// aparência de cada objecto da cena sob o NOME dele (`render_loop/mod.rs:7317`), e esta
/// membrana corre depois (`:7341`) — *a ordem é o que torna isto uma leitura em vez de uma
/// segunda resolução a divergir da primeira*.
///
/// `None` quando o nome está vazio, quando ninguém publicou aquele nome, ou quando o que ele
/// nomeia não é uma sprite. ⚠️ **Não adivinha e não falha**: a folha simplesmente não nasce, e
/// o quadro seguinte tenta de novo — um nome pode ser escrito antes de a forma existir.
pub(crate) fn named_appearance(cook: &ph2d_nodegraph::cook::Cook, name: &str) -> Option<Look> {
    // ⚠️ **Atalho, NÃO uma guarda de correcção** — e a mutação que o apagou SOBREVIVEU, o que
    // é a resposta certa: ninguém publica sob a chave vazia, então a busca abaixo já devolveria
    // `None`. Fica porque poupa uma busca no mapa por slot vazio por planta por quadro, e o
    // caso comum é os três estarem vazios. *Um `if` que a mutação não mata ou é redundante ou é
    // não-medido; este é o primeiro, e está escrito.*
    if name.is_empty() {
        return None;
    }
    let st = &cook.externals().get(name)?.value;
    let first4 = |c: &str| match st.get(c) {
        Some(Column::Vec4(v)) => v.first().copied(),
        _ => None,
    };
    Some((
        match st.get("size") {
            Some(Column::Vec2(v)) => v.first().copied().unwrap_or([1.0, 1.0]),
            _ => [1.0, 1.0],
        },
        first4("tint").unwrap_or([1.0, 1.0, 1.0, 1.0]),
        first4("uv_rect")?,
        match st.get("texture_id") {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        },
        // ⚠️ **Ausente ⇒ `0`**, que é o que este caminho fazia antes de a bandeira existir.
        match st.get("premultiplied") {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        },
    ))
}

// **O QUE JÁ FOI DITO** — para o aviso sair uma vez, e não sessenta vezes por segundo.
//
// ⚠️ Por thread e por `(chave, slot)`: a chave é de CONTEÚDO, então mudar a gramática dá uma
// chave nova e o aviso volta a poder sair — que é exactamente quando ele interessa.
thread_local! {
    static SAID: std::cell::RefCell<std::collections::BTreeSet<String>> =
        const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}

/// **A METADE PURA da pergunta** — quais slots têm nome e não têm onde nascer.
///
/// ⚠️ Ela existe separada porque a outra metade **escreve no `stderr`**, e um gate não lê o
/// `stderr` de outro processo: *a lei tem de ser alcançável de um teste, ou o que se prova é o
/// canal e não a decisão.*
pub(crate) fn unanswered_slots(names: &[String; 3], anchors: &[Anchor]) -> Vec<usize> {
    (0..names.len())
        .filter(|&s| !names[s].is_empty() && !anchors.iter().any(|a| a.slot == s))
        .collect()
}

/// **DIZ quando um nome está posto e a gramática não tem a letra.**
///
/// ⛔⛔ Report do Enio (2026-08-30): *"só apareceu em seu exemplo, ao trocar o tipo de árvore não
/// aparece mais"*. Os moldes de planta passaram a trazer o `J`, mas **uma gramática escrita pelo
/// artista pode não ter letra nenhuma** — e aí o campo fica cheio, nada desenha, e não há como
/// saber porquê. *Um controlo com valor lá dentro e efeito nenhum é a pior espécie de morto: ele
/// parece ligado.*
///
/// ⚠️ **Diz a CURA, não só o sintoma** — a letra que falta é a informação que resolve.
pub(crate) fn say_if_the_letter_is_missing(key: &str, names: &[String; 3], anchors: &[Anchor]) {
    for slot in unanswered_slots(names, anchors) {
        let name = &names[slot];
        let once = format!("{key} slot {slot}");
        let fresh = SAID.with(|s| s.borrow_mut().insert(once));
        if fresh {
            eprintln!(
                "[lsystem] «{name}» esta' no slot {letra}, mas a gramatica nao emite nenhum \
                 `{letra}` — acrescente um (ex.: `[{letra}]` no fim de um ramo) ou o objecto nao \
                 tem onde nascer",
                letra = ls::LEAF_SYMBOLS[slot] as char
            );
        }
    }
}

/// **A planta MAIS as folhas, num stream só.**
///
/// ⚠️ **Mídia MISTA na mesma corrente, e o lowering já a sabe rotear:** uma linha com
/// `geometry_id > 0` vai ao passe VECTORIAL (a planta), e as outras são quads amostrados do
/// atlas (as folhas). Publicá-las em correntes separadas obrigaria o artista a juntá-las com um
/// `motion.combine` para as mover como uma planta só.
/// **Abaixo disto a folha não se desenha.** Não é um limiar de gosto: `1/256` de um quad é
/// menos de um pixel em qualquer zoom que o editor oferece, e o custo de uma linha é o mesmo
/// visível ou não. ⚠️ Ele **não** é um degrau visível — o peso passa por aqui a subir, e a
/// folha que ele esconde tem `size` abaixo de meio pixel.
pub(crate) const GROW_FLOOR: f32 = 1.0 / 256.0;

pub(crate) fn plant_and_leaves(
    origin: [f32; 2],
    handle: u32,
    anchors: &[Anchor],
    names: &[String; 3],
    cook: &ph2d_nodegraph::cook::Cook,
) -> Stream {
    let looks: Vec<_> = names.iter().map(|n| named_appearance(cook, n)).collect();
    // A planta é a linha `0`; cada âncora com objecto RESOLVIDO acrescenta uma.
    let mut p = vec![origin];
    let mut size = vec![[1.0f32, 1.0]];
    let mut rot = vec![0.0f32];
    let mut geom = vec![handle as f32];
    let mut tint = vec![[1.0f32, 1.0, 1.0, 1.0]];
    let mut uv = vec![[0.0f32, 0.0, 1.0, 1.0]];
    let mut tex = vec![0.0f32];
    let mut premul = vec![0.0f32];
    for a in anchors {
        let Some((sz, tn, rect, tid, pm)) = looks[a.slot] else {
            continue;
        };
        // ⚠️ **A marca fechada não vira linha nenhuma** — um quad de tamanho `0` custaria o
        // mesmo que um visível, e a árvore de fábrica traz `62` marcas para `32` pontas.
        if a.grow <= GROW_FLOOR {
            continue;
        }
        p.push(a.p);
        size.push([sz[0] * a.grow, sz[1] * a.grow]);
        rot.push(a.rot);
        premul.push(pm);
        // ⚠️ `0` = SEM geometria vectorial ⇒ a linha vai pelo caminho do quad. É a mesma
        // convenção que o `source.object` usa para separar um vector vivo de uma sprite.
        geom.push(0.0);
        tint.push(tn);
        uv.push(rect);
        tex.push(tid);
    }
    let n = p.len();
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("size", Column::Vec2(size))
        // ⛔ **`rot`, e não `rotation`** — é este o nome que a convenção de instâncias lê
        // (`ph2d-eval-motion`, em GRAUS). O outro era ignorado em silêncio.
        .with("rot", Column::Scalar(rot))
        .with("premultiplied", Column::Scalar(premul))
        .with("geometry_id", Column::Scalar(geom))
        .with("tint", Column::Vec4(tint))
        .with("uv_rect", Column::Vec4(uv))
        .with("texture_id", Column::Scalar(tex))
        .with(
            "Index",
            Column::Scalar((0..n).map(|i| i as f32).collect::<Vec<_>>()),
        )
        .with("Count", Column::Scalar(vec![n as f32; n]))
}

#[cfg(test)]
#[path = "motion_lsystem_leaves_tests.rs"]
mod tests;
