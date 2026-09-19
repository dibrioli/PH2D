//! ⭐⭐⭐ **UMA COLUNA DEVOLVE EXACTAMENTE O QUE LEVOU** — o interruptor de fechar/reabrir.
//!
//! # O report que pagou este ficheiro
//!
//! > *«a retração dos painéis laterais ainda está ruim. O inspector explodiu, soltou vários
//! > painéis no meio do canvas.»* — Enio, 2026-09-07, **terceiro** report sobre o mesmo gesto.
//!
//! As waves 29 e 30 escreveram as duas metades do interruptor a perguntar a **factos diferentes**,
//! e a wave 30 declarou isso como lei: *«as duas metades de um interruptor podem precisar de
//! fontes de verdade diferentes, e é o estado que elas atravessam que decide qual»*. A frase é
//! verdadeira; a conclusão que faltava é esta:
//!
//! ⛔⛔ **Fontes diferentes não devolvem só respostas diferentes — devolvem CONJUNTOS diferentes.**
//!
//! | metade | perguntava | população medida no registo de 2026-09-07 |
//! |---|---:|---|
//! | fechar | *que rect foi PUBLICADO nesta coluna?* | **1** — só o painel da frente; a fila de abas esconde os outros e o [`super::panel_walk`] limpa-lhes o rect |
//! | reabrir | *quem DECLARA morar deste lado?* | **22** |
//!
//! Dos 22, **6 declaram `CAN_FLOAT`** e nascem centrados no viewport ⇒ *«soltou vários painéis no
//! meio do canvas»*, literalmente seis. Os outros 16 disputam uma fila de abas que pinta 3.
//!
//! # A lei
//!
//! ⭐⭐⭐ **Reabrir uma coluna repõe exactamente os painéis que AQUELE fecho escondeu, e mais
//! nenhum.** É uma **involução**, e por isso é uma **memória** — não uma re-derivação.
//!
//! ⚠️ **Nenhuma condição escrita sobre o registo consegue estreitar o superconjunto**, e não é por
//! falta de engenho: a informação que decide (*«o dono tinha isto aberto?»*) **não está no
//! registo**. Ela existe num instante só — o do fecho — e quem não a guardar ali não a tem depois.
//!
//! # As três perguntas que esta porta deixou de fazer errado
//!
//! 1. **Quem é inquilino?** — hoje é [`super::slot_tabs::occupants`], a **mesma** porta que a fila
//!    de abas usa. Ela já exclui os `CAN_FLOAT` (*«um painel que FLUTUA não ocupa encaixe nenhum»*)
//!    e já honra o encaixe **arrumado** por cima do declarado. ⛔ Uma segunda definição de
//!    *«inquilino»* divergiria da primeira no dia em que só uma fosse afinada — e foi exactamente
//!    o que aconteceu entre o `close_column` (rect publicado) e o `occupants` (encaixe).
//! 2. **Quantos inquilinos?** — os dois encaixes do lado, não um. As metades de baixo estão vazias
//!    hoje, e escrever a varredura sobre [`Slot::dock_side`] custa o mesmo e não caduca.
//! 3. **Que largura?** — a que a coluna tinha. ⚠️ O doc do gesto prometia *«um toque reabre a
//!    coluna na largura que ela tinha»* e nenhuma linha o cumpria: `open_column` não tocava na
//!    largura, e a última escrita antes de todo fecho é o `DOCK_W_MIN` clampado pela porta do
//!    store ⇒ **toda reabertura devolvia 220 px**, fosse qual fosse a largura do artista.
//!
//! # ⛔ Por que isto vive no `ph2d-editor-core` e não na shell
//!
//! Enquanto `close_column`/`open_column` foram `fn` privadas de um `impl App` do **binário**,
//! nenhum teste as alcançava — e o gate que as cobria (`the_border_gesture_reaches_the_panel.rs`)
//! era `fs::read_to_string` + `contains`, isto é, *ele leu a linha do defeito e chamou-lhe
//! correcta*. **A lei mudou de sítio para poder ter régua**; o gate vive em
//! `ph2d-panel-registry-init/tests/`, que é onde o registo real existe.

use super::HeroScreen;
use crate::screens::layout::DockSide;
use crate::screens::slot::Slot;

/// ⭐⭐ **O que um fecho levou consigo** — a memória que faz do par uma involução.
///
/// ⚠️ **A largura entra aqui, e não no store.** O store guarda *a escolha do artista*; esta é *o
/// estado de um gesto em curso*. Escrever a largura de volta no store ao fechar faria a coluna
/// fechada declarar uma escolha que ninguém fez, e a persistência gravá-la-ia.
#[derive(Clone, Debug, PartialEq)]
pub struct ClosedColumn {
    /// Os painéis que **este** fecho escondeu, na ordem do registo.
    pub panels: Vec<&'static str>,
    /// ⭐⭐ **A ESCOLHA de largura, não a largura.** `None` = ninguém arrastou aquela borda.
    ///
    /// ⛔ Guardar o número que [`crate::interaction::WidgetStore::dock_width`] devolve seria
    /// guardar o **default** de uma coluna que o artista nunca tocou — e repô-lo ao reabrir
    /// escreveria esse default como se fosse uma decisão dele. A persistência grava exactamente
    /// `dock_width_choice`, então um ciclo fechar→reabrir passaria a **prender a coluna no número
    /// velho para sempre**, no dia em que o default mudasse. O doc daquela porta já o dizia.
    pub width_choice: Option<f32>,
}

/// O índice deste lado na memória por-coluna.
const fn idx(side: DockSide) -> usize {
    match side {
        DockSide::Left => 0,
        DockSide::Right => 1,
    }
}

/// Os encaixes daquele lado — hoje um por lado com inquilinos, dois por construção.
fn slots_of(side: DockSide) -> impl Iterator<Item = Slot> {
    Slot::ALL
        .into_iter()
        .filter(move |s| s.dock_side() == Some(side))
}

/// ⭐ **Os INQUILINOS de uma coluna** — a mesma pergunta que a fila de abas faz, sobre os dois
/// encaixes do lado.
///
/// ⚠️ Ela responde sobre os **visíveis**: um painel fechado não é inquilino de ninguém. É por isso
/// que a metade que REABRE não pode usá-la, e tem de ler a memória.
#[must_use]
pub fn tenants(hero: &HeroScreen, side: DockSide) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for slot in slots_of(side) {
        out.extend(
            super::slot_tabs::occupants(hero, slot)
                .into_iter()
                .map(|o| o.id),
        );
    }
    out
}

/// ⭐⭐⭐ **Fecha a coluna** — esconde **todos** os inquilinos e guarda quem eles eram.
///
/// `false` quando não havia ninguém: nesse caso a coluna já estava vazia e o gesto não tem sujeito.
///
/// ⚠️ **Um fecho que não encontra ninguém NÃO apaga a memória de um fecho anterior.** Dois fechos
/// seguidos sem reabertura pelo meio são um gesto repetido, não dois gestos — e o segundo não pode
/// deitar fora o que o primeiro guardou.
///
/// ⛔⛔ **O `width_choice` vem de FORA, e não do store**, porque quando esta função corre o store
/// já não sabe a resposta: o gesto de fecho é um arrasto, e **cada pixel dele escreve a largura**
/// (clampada ao mínimo pela porta). Uma coluna de 400 px arrastada até fechar deixa `Some(220)`
/// gravado muito antes de o degrau disparar ⇒ ler o store aqui devolveria sempre o mínimo, que é
/// exactamente o defeito que esta wave está a curar. Quem sabe a largura de antes é **quem armou o
/// arrasto**.
pub fn close(hero: &mut HeroScreen, side: DockSide, width_choice: Option<f32>) -> bool {
    let panels = tenants(hero, side);
    if panels.is_empty() {
        return false;
    }
    for id in &panels {
        hero.panel_visibility.insert(id, false);
    }
    hero.dock_closed[idx(side)] = Some(ClosedColumn {
        panels,
        width_choice,
    });
    true
}

/// ⭐⭐⭐ **Reabre a coluna** — repõe exactamente o que o fecho levou, e a largura que ela tinha.
///
/// Sem memória (a coluna já nasceu fechada, ou o app reiniciou com ela fechada) cai no
/// [`fallback`], que é o que a **tarefa activa** declara para aquele lado.
pub fn open(hero: &mut HeroScreen, side: DockSide) -> bool {
    let remembered = hero.dock_closed[idx(side)].take();
    let (panels, width_choice) = match remembered {
        Some(c) => (c.panels, c.width_choice),
        None => (fallback(hero, side), None),
    };
    if panels.is_empty() {
        return false;
    }
    for id in &panels {
        hero.panel_visibility.insert(id, true);
    }
    // ⚠️ `None` deixa a coluna no default, e é o correcto: repor um número que ninguém escolheu
    // transformá-lo-ia numa escolha, e a persistência grava exactamente as escolhas.
    if let Some(w) = width_choice {
        hero.store.set_dock_width(side, w);
    }
    true
}

/// ⭐⭐ **O que reabrir devolve quando não há memória** — a lista da **tarefa activa**, nunca o
/// registo inteiro.
///
/// ⚠️ **A `LayoutSpec::open` é a lista COMPLETA da tarefa** (*«tudo o que não está aqui fecha»*),
/// logo ela é exactamente a resposta a *«o que pertence a esta coluna neste modo de trabalho?»*.
/// ⛔ O registo responde a outra pergunta — *«quem PODE morar aqui?»* — e é essa troca que abria 22.
///
/// ⚠️ Um `CAN_FLOAT` nunca entra: ele não é inquilino de coluna nenhuma, e o gesto da borda de uma
/// coluna não tem por que abrir uma janela solta no meio do desenho.
///
/// O último recurso é o `DEFAULT_VISIBLE` do painel — numa tarefa que não declara nada para este
/// lado, puxar a borda dá o painel que o lado tem de fábrica, e não o vazio.
#[must_use]
pub fn fallback(hero: &HeroScreen, side: DockSide) -> Vec<&'static str> {
    let spec = hero.store.active_layout().spec();
    let mut declared: Vec<&'static str> = Vec::new();
    let mut by_default: Vec<&'static str> = Vec::new();
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            let m = &p.manifest;
            if m.can_float {
                continue;
            }
            let slot = super::slot_tabs::slot_of(hero, m);
            if slot.dock_side() != Some(side) {
                continue;
            }
            if spec.open.contains(&m.id) {
                declared.push(m.id);
            }
            if m.default_visible {
                by_default.push(m.id);
            }
        }
    });
    if declared.is_empty() {
        by_default
    } else {
        declared
    }
}

/// Há memória de um fecho deste lado? — a pergunta que a shell faz para saber se o gesto de
/// reabrir devolve a largura ou cai no fallback.
#[must_use]
pub fn has_memory(hero: &HeroScreen, side: DockSide) -> bool {
    hero.dock_closed[idx(side)].is_some()
}

/// ⭐⭐⭐ **Esta coluna está FECHADA?** — a pergunta que a marca do menu faz.
///
/// ⚠️ **É a MEMÓRIA do fecho e não a largura**, e a distinção é a que o header deste ficheiro
/// defende: uma coluna arrastada até ao `DOCK_W_MIN` continua **aberta** (desde 2026-09-09 a borda
/// nem a pode fechar), e ler a largura diria o contrário no dia em que o artista a apertasse.
///
/// ⛔ **Um segundo predicado escrito no menu divergiria deste no dia em que a memória mudasse de
/// forma** — foi exactamente o que aconteceu entre o `close_column` (que lia o rect publicado) e o
/// `occupants` (que lê o encaixe), e custou o report *«soltou vários painéis no meio do canvas»*.
#[must_use]
pub fn is_closed(hero: &HeroScreen, side: DockSide) -> bool {
    has_memory(hero, side)
}
