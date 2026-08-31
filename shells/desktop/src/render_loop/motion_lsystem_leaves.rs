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
    /// **A identidade ESTÁVEL da marca** — `(geração, ordinal dentro dela)`, dobrada num
    /// número. É dela que saem os sorteios e o lado (frente/trás), e **não** do índice na
    /// lista: ao crescer, a planta insere marcas no meio e o índice de uma folha antiga muda.
    pub(crate) seed: u32,
    /// O índice em [`ls::LEAF_SYMBOLS`] — `0` = `J`, `1` = `K`, `2` = `M`.
    pub(crate) slot: usize,
}

/// As âncoras que as três letras plantaram, lidas do esqueleto.
pub(crate) fn anchors_of(sk: &Stream) -> Vec<Anchor> {
    let (p, sym, rot) = (v2(sk, "P"), v1(sk, "sym"), v1(sk, "rot"));
    let (grow, born) = (v1(sk, "mark_grow"), v1(sk, "gen"));
    let mut por_geracao: std::collections::BTreeMap<i32, u32> = std::collections::BTreeMap::new();
    let mut out = Vec::new();
    for (i, (pos, s)) in p.iter().zip(sym.iter()).enumerate() {
        let byte = *s as i32 as u8;
        let Some(slot) = ls::LEAF_SYMBOLS.iter().position(|k| *k == byte) else {
            continue;
        };
        // ⛔⛔ **A IDENTIDADE DE UMA MARCA NÃO É O ÍNDICE DELA NA LISTA** — report do Enio
        // (2026-08-30): *"nem todas as folhas crescem, algumas aparecem já grandes"*.
        //
        // Ao crescer, a planta **insere marcas no MEIO** (a travessia é em profundidade), então
        // o índice de uma folha que já existia MUDA. Com o sorteio a sair do índice, a mesma
        // folha ganhava um tamanho novo — e a que estava pequena saltava para grande. O mesmo
        // valia para o `Leaves In Front`: as folhas trocavam de lado enquanto a planta crescia.
        //
        // ⭐ **O par `(geração, ordinal dentro dela)` é estável**, e é estável pela razão que
        // faz a planta crescer: as gerações velhas não se reescrevem, logo a ordem relativa das
        // marcas de uma geração não muda quando outra nasce.
        let g = born.get(i).copied().unwrap_or(0.0) as i32;
        let n = por_geracao.entry(g).or_insert(0);
        *n += 1;
        out.push(Anchor {
            p: *pos,
            rot: rot.get(i).copied().unwrap_or(0.0),
            // ⚠️ **Ausente ⇒ `1`**, o valor maduro: uma coluna que não existe não pode
            // apagar a folha de um esqueleto vindo de outra rota.
            grow: grow.get(i).copied().unwrap_or(1.0),
            seed: (g as u32).wrapping_mul(0x0001_0001).wrapping_add(*n),
            slot,
        });
    }
    out
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
    super::motion_lsystem_rows::LeafLook,
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
        if mine
            .iter()
            .any(|a| a.grow > super::motion_lsystem_rows::GROW_FLOOR)
        {
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

#[cfg(test)]
#[path = "motion_lsystem_leaves_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_lsystem_says_tests.rs"]
mod says_tests;

#[cfg(test)]
#[path = "motion_lsystem_look_tests.rs"]
mod look_tests;
