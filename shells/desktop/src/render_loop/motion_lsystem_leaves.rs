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

/// **A aparência de um objecto**, na ordem
/// `(size, tint, uv_rect, texture_id, premultiplied, geometry_id)`.
///
/// ⚠️ O 5.º campo entrou pelo report de 2026-08-30 (*"o Alpha escurece as bordas da pintura"*):
/// ele é da TEXTURA, e sem ele o lowering pré-multiplicava outra vez ⇒ `RGB·α²`.
pub(crate) type Look = ([f32; 2], [f32; 4], [f32; 4], f32, f32, f32);

/// **O trabalho de uma planta**, já resolvido: `(chave, ramos, âncoras, os três nomes)`.
pub(crate) type Job = (
    String,
    Vec<ls::branch::Branch>,
    Vec<Anchor>,
    [String; 3],
    // O 1.º nível com folha, e o resto do aspecto delas.
    f32,
    LeafLook,
);

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
    let first1 = |c: &str| match st.get(c) {
        Some(Column::Scalar(v)) => v.first().copied(),
        _ => None,
    };
    let size = match st.get("size") {
        Some(Column::Vec2(v)) => v.first().copied().unwrap_or([1.0, 1.0]),
        _ => [1.0, 1.0],
    };
    let tint = first4("tint").unwrap_or([1.0, 1.0, 1.0, 1.0]);
    // ⭐⭐ **UMA FORMA DESENHADA TAMBÉM É UMA FOLHA** — e até 2026-08-30 não era: esta função
    // exigia `uv_rect`, que só uma sprite publica, então nomear uma forma do documento não
    // plantava nada e nem dizia porquê.
    //
    // ⚠️ **E é ela que torna o *Leaves In Front* possível.** A ordem de desenho da casa é
    // *sprites primeiro, vector depois* (declarada em `mod.rs`: «Fase 1: vector over sprite»),
    // então uma folha-sprite fica SEMPRE atrás dos galhos. Uma folha-vector vive na mesma
    // passagem que a planta, e ali quem manda é a ORDEM DAS LINHAS.
    if let Some(gid) = first1("geometry_id").filter(|g| *g > 0.0) {
        return Some((size, tint, [0.0, 0.0, 1.0, 1.0], 0.0, 0.0, gid));
    }
    Some((
        size,
        tint,
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
        // Uma sprite não tem geometria vectorial — é a mesma convenção do `source.object`.
        0.0,
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

/// **Este aviso já saiu?** — a metade que torna a decisão mensurável de um teste.
#[cfg(test)]
pub(crate) fn already_said(key: &str) -> bool {
    SAID.with(|s| s.borrow().contains(key))
}

/// **DIZ quando um FIO conduz um param que outro param mantém inerte.**
///
/// ⛔⛔ Report do Enio (2026-08-30): *"LFO não funciona animando Tropism Angle. corrija todos"*.
/// A medição ilibou a maquinaria — com `Tropism = 30` o LFO no `Tropism Angle` move a planta
/// (`0,541 → 0,528 → 0,578` de altura em três instantes) — e nomeou **duas** causas, as duas do
/// artista e nenhuma visível na tela:
///
/// 1. **`Tropism` nasce em `0`**, e o `Tropism Angle` é a DIRECÇÃO de uma força cuja
///    INTENSIDADE é zero. Mexer a direcção de nada continua a ser nada.
/// 2. **O `value.lfo` nasce com `amplitude = 1`**, e este param é em GRAUS: ±1° é invisível.
///
/// ⚠️ **Um `ParamGate` não exprime isto:** ele compara o valor de outro param com uma lista de
/// INTEIROS, e a condição aqui é *«diferente de zero»* num slider contínuo. E esconder a linha
/// seria pior — ela desapareceria no estado de fábrica, que é exactamente onde ele estava.
/// ⇒ o app **diz**, pelo mesmo canal das outras duas mensagens desta jornada.
///
/// ⚠️ **Só quando há FIO**, e é isso que a torna silenciosa no uso normal: um `Tropism Angle`
/// parado no default com `Tropism = 0` é o estado de fábrica de toda planta, e avisar sobre ele
/// seria ruído em cada quadro de cada cena.
pub(crate) fn say_if_a_wire_drives_an_inert_param(key: &str, driven: &[&str], tropism: f32) {
    if tropism != 0.0 || !driven.contains(&ls::param::TROPISM_ANGLE) {
        return;
    }
    let once = format!("{key} inert tropism");
    if SAID.with(|s| s.borrow_mut().insert(once)) {
        eprintln!(
            "[lsystem] ha' um fio a conduzir o «Tropism Angle», mas o «Tropism» esta' em 0 — o \
             angulo e' a DIRECCAO de uma forca, e uma forca de intensidade zero nao move nada. \
             Suba o «Tropism». (E um `value.lfo` nasce com amplitude 1, que neste param e' UM \
             GRAU: suba a amplitude tambem.)"
        );
    }
}

/// **DIZ quando o `First Level` apagou TODAS as folhas de uma letra.**
///
/// ⛔⛔ Report do Enio (2026-08-30, depois de eu shipar o knob): *"Keep own color não funciona,
/// as folhas não aparecem"* e *"Leaves in front não funciona, nada muda"* — **dois reports, e o
/// silêncio é metade da causa dos dois**. Uma gramática cujas marcas vivem todas abaixo do
/// nível mínimo fica sem folha nenhuma, e não há nada na tela que o diga.
///
/// ⚠️ **A cerca de fundo é o molde carregar o SEU número** (`Preset::leaf_first_level`, medido
/// por molde). Esta mensagem é para a gramática que o **artista** escreve, onde não há tabela
/// que o saiba por ele.
pub(crate) fn say_if_the_level_hid_every_leaf(
    key: &str,
    names: &[String; 3],
    anchors: &[Anchor],
    first_level: f32,
) {
    for (slot, name) in names.iter().enumerate() {
        let mine: Vec<&Anchor> = anchors.iter().filter(|a| a.slot == slot).collect();
        // Sem nome ou sem âncora, quem fala é o aviso da letra — este seria ruído por cima.
        if name.is_empty() || mine.is_empty() {
            continue;
        }
        if mine.iter().any(|a| a.grow > GROW_FLOOR) {
            continue;
        }
        let once = format!("{key} level {slot}");
        if SAID.with(|s| s.borrow_mut().insert(once)) {
            eprintln!(
                "[lsystem] «{name}» tem {} marca(s) na gramatica e NENHUMA se desenha: o «First \
                 Level» esta' em {first_level:.0} e todas elas nascem mais perto da raiz do que \
                 isso. Baixe o «First Level»",
                mine.len()
            );
        }
    }
}

/// **DIZ quando o artista pede folhas à frente e o objecto não pode ir lá.**
///
/// ⛔⛔ A casa desenha os **sprites antes do vector** (declarado em `mod.rs`: *«Fase 1: vector
/// over sprite»*), então uma folha que é uma IMAGEM fica sempre atrás dos galhos e nenhuma
/// ordem de linhas a move. Uma folha que é uma FORMA DESENHADA vive na mesma passagem que a
/// planta, e aí a fracção manda.
///
/// ⚠️ **Sem isto o `Leaves In Front` seria um knob morto no caso comum** — o artista mexe-o,
/// nada acontece, e não há nada na tela que explique porquê.
pub(crate) fn say_if_the_leaf_cannot_go_in_front(
    key: &str,
    names: &[String; 3],
    looks: &[Option<Look>; 3],
    front: f32,
) {
    if front <= 0.0 {
        return;
    }
    for (slot, look) in looks.iter().enumerate() {
        // Só acusa o que EXISTE e é sprite: um slot vazio já é dito pelo aviso da letra.
        let Some((.., gid)) = look else { continue };
        if *gid > 0.0 {
            continue;
        }
        let once = format!("{key} front {slot}");
        if SAID.with(|s| s.borrow_mut().insert(once)) {
            eprintln!(
                "[lsystem] «{}» e' uma IMAGEM, e as imagens desenham-se sempre ATRAS dos galhos \
                 — o «Leaves In Front» so' alcanca uma FORMA desenhada. Desenhe a folha com a \
                 caneta e nomeie-a, ou deixe o knob em 0",
                names[slot]
            );
        }
    }
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
    look_law: LeafLook,
) -> Stream {
    let (front, keep_own_colour) = (look_law.front, look_law.keep_own_colour);
    let looks: Vec<_> = names.iter().map(|n| named_appearance(cook, n)).collect();
    // **Uma linha**, antes de saber em que ordem ela entra.
    struct Row {
        p: [f32; 2],
        size: [f32; 2],
        rot: f32,
        geom: f32,
        tint: [f32; 4],
        uv: [f32; 4],
        tex: f32,
        premul: f32,
        tint_mask: f32,
    }
    let plant = Row {
        p: origin,
        size: [1.0, 1.0],
        rot: 0.0,
        geom: handle as f32,
        tint: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0, 1.0, 1.0],
        tex: 0.0,
        premul: 0.0,
        tint_mask: 1.0,
    };
    // ⭐⭐ **TRÊS BALDES, e a ordem entre eles É o z** — report do Enio (2026-08-30): *"não temos
    // a opção de escolher quantas folhas são desenhadas na frente ou atrás dos galhos"*.
    //
    // ⚠️ **A casa desenha os sprites ANTES do vector** (declarado em `mod.rs`: *«Fase 1: vector
    // over sprite»*), então uma folha-SPRITE fica sempre atrás da planta, e nenhuma ordem de
    // linhas a move. Uma folha-VECTOR vive na mesma passagem que a planta, e ali quem manda é a
    // ordem: as de trás vêm antes da linha da planta, as da frente depois.
    let (mut atras, mut frente, mut sprites) = (Vec::new(), Vec::new(), Vec::new());
    for (i, a) in anchors.iter().enumerate() {
        let Some((sz, tn, rect, tid, pm, gid)) = looks[a.slot] else {
            continue;
        };
        // ⚠️ **A marca fechada não vira linha nenhuma** — um quad de tamanho `0` custaria o
        // mesmo que um visível, e a árvore de fábrica traz `62` marcas para `31` pontas.
        if a.grow <= GROW_FLOOR {
            continue;
        }
        // ⭐ **O tamanho final e os dois sorteios** (report do Enio, 2026-08-30).
        let (scale, shove) = look_law.at(i);
        let sized = [sz[0] * a.grow * scale, sz[1] * a.grow * scale];
        let row = Row {
            // ⚠️ **O empurrão é em FRACÇÃO do tamanho da folha**, e não em unidades de mundo:
            // uma planta a `0,3` de passo e outra a `3` teriam de ser afinadas à mão, e o que
            // o artista quer dizer é *«desencostada do ramo por meia folha»*.
            p: [a.p[0] + shove[0] * sized[0], a.p[1] + shove[1] * sized[1]],
            size: sized,
            rot: a.rot,
            // ⚠️ `0` = SEM geometria vectorial ⇒ a linha vai pelo caminho do quad. É a mesma
            // convenção que o `source.object` usa para separar um vector vivo de uma sprite.
            geom: gid,
            tint: tn,
            uv: rect,
            tex: tid,
            premul: pm,
            // ⭐⭐ **A folha fora do TINT da árvore, e só do tint.**
            //
            // ⛔⛔ **A 1.ª cura escrevia `falloff` e PARTIU a planta** (report do Enio,
            // 2026-08-30: *"Keep own color não funciona, as folhas não aparecem"*): o
            // `falloff` é a máscara de TODOS os modificadores, e o `motion.move` faz
            // `P' = P + (dx, dy) · falloff` — as folhas ficavam PARADAS enquanto a planta se
            // movia, e a cena `=108` move cada coluna. *O canal que escolhi era muito mais
            // largo do que a pergunta que fiz.*
            tint_mask: f32::from(!keep_own_colour),
        };
        if gid <= 0.0 {
            sprites.push(row);
        } else if is_in_front(i, front) {
            frente.push(row);
        } else {
            atras.push(row);
        }
    }
    let n = 1 + atras.len() + frente.len() + sprites.len();
    let mut p = Vec::with_capacity(n);
    let (mut size, mut rot, mut geom) = (Vec::new(), Vec::new(), Vec::new());
    let (mut tint, mut uv, mut tex) = (Vec::new(), Vec::new(), Vec::new());
    let (mut premul, mut tint_mask) = (Vec::new(), Vec::new());
    for r in atras
        .into_iter()
        .chain(std::iter::once(plant))
        .chain(frente)
        .chain(sprites)
    {
        p.push(r.p);
        size.push(r.size);
        rot.push(r.rot);
        geom.push(r.geom);
        tint.push(r.tint);
        uv.push(r.uv);
        tex.push(r.tex);
        premul.push(r.premul);
        tint_mask.push(r.tint_mask);
    }
    let stream = Stream::new(n)
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
        .with("Count", Column::Scalar(vec![n as f32; n]));
    // ⚠️ **A coluna só nasce quando ela DIZ alguma coisa.** Uma coluna de uns responderia a uma
    // pergunta que ninguém fez e apagaria uma máscara que um nó a montante tivesse escrito —
    // ausente ⇒ `1` em toda a casa ⇒ byte-idêntico ao que havia antes deste param.
    if keep_own_colour {
        stream.with(
            ph2d_nodegraph::attr::TINT_MASK_COLUMN,
            Column::Scalar(tint_mask),
        )
    } else {
        stream
    }
}

/// **Esta folha vai à FRENTE?** — a fracção do painel, resolvida por marca e sem estado.
///
/// ⚠️ **Determinística e ESTÁVEL**: o sorteio é do índice da âncora, não de um contador de
/// linhas emitidas — senão uma folha que fechasse (peso `0`) reordenaria todas as outras entre
/// a frente e o fundo, e a árvore piscaria enquanto cresce.
///
/// ⚠️ `0` e `1` são exactos nas duas pontas: `hash ∈ [0, 1)`, logo `< 0` nunca e `< 1` sempre.
fn is_in_front(index: usize, front: f32) -> bool {
    hash01(index) < front
}

/// **O TAMANHO E O EMPURRÃO de cada folha** — o que o painel pede, resolvido por marca.
///
/// ⛔ Report do Enio (2026-08-30): *"não temos parâmetros para o tamanho final da folha nem
/// jitter de scale e posição"*.
///
/// ⚠️ **Os três são NEUTROS no default** (`1`, `0`, `0`), e o neutro é exacto: um `× 1.0` é a
/// identidade em `f32` e um sorteio de amplitude `0` nem é avaliado — o caminho de omissão é
/// byte a byte o que shipou antes deles.
#[derive(Clone, Copy)]
pub(crate) struct LeafLook {
    /// A fracção desenhada à frente dos galhos.
    ///
    /// ⚠️ **Ela mora aqui e não num argumento à parte** porque o clippy tem razão: sete
    /// argumentos posicionais já são um em que ninguém confia. Um `bool` e três `f32` seguidos
    /// numa chamada é uma troca à espera de acontecer.
    pub(crate) front: f32,
    /// `true` = as folhas mantêm a cor delas (os efeitos a jusante não as alcançam).
    pub(crate) keep_own_colour: bool,
    pub(crate) size: f32,
    pub(crate) size_jitter: f32,
    pub(crate) pos_jitter: f32,
}

impl LeafLook {
    /// `(multiplicador de tamanho, empurrão em fracções do tamanho)` para a marca `i`.
    ///
    /// ⚠️ **Três LANES do mesmo hash, e não três chamadas iguais:** com uma lane só, o tamanho
    /// e o empurrão de uma folha seriam o MESMO número — as maiores todas para o mesmo lado,
    /// que é um padrão visível e não um sorteio.
    pub(crate) fn at(self, i: usize) -> (f32, [f32; 2]) {
        let scale = if self.size_jitter == 0.0 {
            self.size
        } else {
            // `±jitter/2` em torno de `1`, logo `jitter = 1` dá de metade ao dobro.
            self.size * (1.0 + (hash01_lane(i, 1) - 0.5) * self.size_jitter)
        };
        let shove = if self.pos_jitter == 0.0 {
            [0.0, 0.0]
        } else {
            [
                (hash01_lane(i, 2) - 0.5) * self.pos_jitter,
                (hash01_lane(i, 3) - 0.5) * self.pos_jitter,
            ]
        };
        (scale, shove)
    }
}

/// `[0, 1)` a partir de um índice — o mesmo avalanche splitmix que o resto da casa usa.
fn hash01(i: usize) -> f32 {
    hash01_lane(i, 0)
}

/// O mesmo, numa LANE — sorteios distintos para perguntas distintas sobre a mesma marca.
fn hash01_lane(i: usize, lane: u32) -> f32 {
    let mut h = (i as u32)
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(lane.wrapping_mul(0xc2b2_ae35))
        .wrapping_add(0x1eaf_1eaf);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

#[cfg(test)]
#[path = "motion_lsystem_leaves_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_lsystem_says_tests.rs"]
mod says_tests;

#[cfg(test)]
#[path = "motion_lsystem_look_tests.rs"]
mod look_tests;
