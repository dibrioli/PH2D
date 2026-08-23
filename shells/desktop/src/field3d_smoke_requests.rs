//! ⭐ **As CAIXAS DE CORREIO entre o painel e o app** — um pedido por gesto.
//!
//! # Por que elas existem
//!
//! A ponte com a cena (`field3d_scene::ecs_bridge`) recebe o **mundo**, e três gestos do painel não
//! são edições do mundo: escrever um arquivo, abrir um diálogo, abrir um painel. Esses são assunto
//! do **app** (janela, toast, `rfd`). O intent é drenado onde o mundo está; o pedido atravessa por
//! aqui — *uma porta, vários pedintes*.
//!
//! ⚠️ **Cada pedido é tirado UMA vez** (`Cell::take` / `replace`): um pedido que ficasse pousado
//! reabriria o diálogo em todo quadro seguinte, que é o modo de falha clássico de um sinal
//! guardado como estado. A lei irmã está no `ph2d-runtime` (o produtor publica, o consumidor drena).
//!
//! ⚠️ **Módulo-filho de [`super`]**, cortado da `field3d_smoke` na W34 pelo teto de LOC — o corte é
//! por **assunto**: o pai possui o *estado* do smoke (câmera, traçado, arrasto) e este possui os
//! *pedidos* que saem dele.

use super::*;

/// ⭐ **O pedido de EXPORTAR, tirado uma vez.**
///
/// ⚠️ Ele existe porque a ponte com a cena recebe o **mundo**, e escrever um arquivo é assunto do
/// **app** (diálogo, toast). O intent do painel é drenado lá dentro; o gesto atravessa por aqui, que
/// é o mesmo caminho que o pedido de abrir o painel já usa — *uma porta, dois pedintes*.
pub(crate) fn take_export_request() -> Option<crate::field3d_export::ExportLevel> {
    EXPORT.with(std::cell::Cell::take)
}

pub(crate) fn ask_export(level: crate::field3d_export::ExportLevel) {
    EXPORT.with(|c| c.set(Some(level)));
}

thread_local! {
    static EXPORT: std::cell::Cell<Option<crate::field3d_export::ExportLevel>> =
        const { std::cell::Cell::new(None) };
}

/// ⭐ **O pedido de IMPORTAR uma escultura**, pelo mesmo caminho e pelo mesmo motivo do de exportar:
/// abrir um diálogo é assunto do app, e o intent do painel é drenado dentro da ponte com o mundo.
pub(crate) fn take_import_request() -> bool {
    IMPORT.with(std::cell::Cell::take)
}

pub(crate) fn ask_import() {
    IMPORT.with(|c| c.set(true));
}

thread_local! {
    static IMPORT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// ⭐ **A escultura carregada, à espera de virar nó** — a volta do pedido.
///
/// ⚠️ **São três saltos e não dois, e o motivo é o mundo**: quem tem o `&mut World` (a ponte com a
/// cena) não pode abrir um diálogo, e quem abre o diálogo (o app) não tem o mundo. O arquivo é lido
/// e o campo registado no meio; o que fica pendurado aqui é só o **nome**, que o próximo quadro
/// transforma em nó. O atraso é de um quadro e ninguém o vê.
pub(crate) fn take_pending_sculpt() -> Option<String> {
    PENDING_SCULPT.with(|c| c.borrow_mut().take())
}

pub(crate) fn ask_spawn_sculpt(key: String) {
    PENDING_SCULPT.with(|c| *c.borrow_mut() = Some(key));
}

thread_local! {
    static PENDING_SCULPT: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// A extensão do arquivo importado, para o nó nascer no tamanho do enquadramento.
///
/// ⚠️ Ela viaja separada do nome porque é **do arquivo**, não do campo: o campo é construído nas
/// unidades do autor de propósito (é isso que faz a célula da grade ser a resolução real dele), e o
/// tamanho de convivência mora na **pose**, onde um clique o desfaz.
pub(crate) fn take_sculpt_extent() -> Option<f32> {
    SCULPT_EXTENT.with(std::cell::Cell::take)
}

pub(crate) fn ask_sculpt_extent(extent: f32) {
    SCULPT_EXTENT.with(|c| c.set(Some(extent)));
}

thread_local! {
    static SCULPT_EXTENT: std::cell::Cell<Option<f32>> = const { std::cell::Cell::new(None) };
}

// ⚠️ **O canal de avisos VIVEU AQUI** (W23: as esculturas que não voltaram do arquivo) e mudou-se
// para [`crate::field3d_notice`] na W25, quando apareceu o segundo produtor — a peça que não
// cozinha. Dois canais paralelos teriam duas leis de repetição, dois drenos, e dois sítios onde
// alguém se esquece de drenar. *O módulo fala por uma boca.*

/// **O painel abre sozinho na primeira vez** que o smoke desenha, e só nessa.
///
/// ⭐ *Feature nova = auto-play* é a lei da casa, e um painel que exigisse aprender uma tecla para
/// aparecer é uma feature que ninguém alcança. Abrir **uma vez** é o que reconcilia isso com o botão
/// de fechar: reabri-lo todo quadro faria o X não funcionar, que é a forma mais irritante de duas
/// portas discordarem.
pub(crate) fn take_open_panel_request() -> bool {
    thread_local! {
        static PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    }
    // Só pede se o smoke está de facto armado — senão o painel de modelagem abriria em toda sessão
    // do app, ocupando o encaixe da direita para não mostrar nada.
    if with_smoke(|_| ()).is_none() {
        return false;
    }
    PENDING.with(|p| p.replace(false))
}

thread_local! {
    /// O pill do topo pediu o módulo ligado.
    static PILL_ARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// **A porta de ARMAR**, escrita pelo shell a partir da visibilidade do painel.
///
/// ⭐ O módulo passa a ter **duas** entradas: a variável de ambiente (a do smoke dirigido) e o
/// **pill do topo** — que é a que um artista encontra. Enquanto a única porta era a `env`, o módulo
/// não existia para quem abre o app (Enio, 2026-08-19: *"não temos um Pill no topo"*).
///
/// ⚠️ Uma porta que só quem já sabe consegue abrir é o mesmo que não existir — a lição que o pill do
/// SCULPT já tinha registado ao lado, e que este módulo repetiu.
pub(crate) fn set_armed_by_panel(open: bool) {
    // ⭐⭐ **SEGUE O PAINEL NOS DOIS SENTIDOS** (W42) — e até 2026-08-22 ele **travava ligado**, com
    // uma razão escrita ao lado:
    //
    // > *"Fechar o painel fecha o PAINEL; a peça continua na cena… Fazer o X do painel apagar o
    // > modelo da tela seria um segundo significado para o mesmo gesto, e o artista perderia a peça
    // > sem a ter apagado."*
    //
    // ⚠️ **A metade protegida estava certa; a conclusão não.** O medo era perder a **peça**, e a
    // peça não vive aqui: desde a W5 ela é uma **árvore de entidades ECS** — está na Hierarquia, é
    // salva, é desfeita, e o `Smoke` é só o cache do quadro para a thread do traçado. Largá-lo
    // perde o **cache e a câmera**, nada mais; ao rearmar, o cozimento reencontra a raiz e a
    // semente é ignorada (é a lei que o gate `deleting_the_part_does_not_replant_it_next_frame`
    // já prende, e o gate `rearming_does_not_replant_the_demo_over_the_artists_piece` prova
    // para este caminho).
    //
    // ⛔ **O preço da trava era o app inteiro.** Enio, 2026-08-22: *"o modo Modelagem nunca é
    // desativado e não consigo usar nenhum outro modo do app"* e, depois da W40, *"ainda não
    // consigo usar outros modos como vector"*. Com a bandeira presa, fechar o painel deixava
    // **todo gancho de entrada** a consumir o gesto — e, pela ordem do `input_dispatch`, quem vem
    // depois da modelagem (o Vector, o gizmo, a seleção) nunca via o clique.
    //
    // *Uma cerca de Chesterton cuja razão dissolveu continua a cobrar o preço dela.* A razão
    // dissolveu na W5, quando a hierarquia passou a ser o documento — e ninguém reconferiu a nota.
    PILL_ARMED.with(|c| c.set(open));
}

/// ⭐ **O pedido de trazer a escultura DA CENA** (W39) — tirado uma vez, como os irmãos.
///
/// ⚠️ Ele atravessa por aqui e não é servido na hora pela razão de sempre: quem tem a escultura
/// viva é o `AppGfx`, e a ponte com a cena recebe o **mundo**.
pub(crate) fn take_scene_sculpt_request() -> bool {
    SCENE_SCULPT.with(std::cell::Cell::take)
}

pub(crate) fn ask_scene_sculpt() {
    SCENE_SCULPT.with(|c| c.set(true));
}

thread_local! {
    static SCENE_SCULPT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn armed_scene() -> Option<u32> {
    if let Ok(v) = std::env::var("PH2D_FIELD_SMOKE") {
        return Some(v.parse().unwrap_or(1));
    }
    // Aberto pelo pill: a cena 1 é a que mostra os dois arredondamentos de uma vez.
    PILL_ARMED.with(std::cell::Cell::get).then_some(1)
}
