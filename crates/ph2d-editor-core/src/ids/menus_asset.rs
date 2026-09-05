//! ⭐⭐ **O menu de um CARTÃO da biblioteca de assets** (plano `docs/Components/07`, etapa C).
//!
//! Cortado de `menus.rs` (que passou o tecto de LOC do ficheiro) pelo mesmo critério que cortou o
//! `menus_timeline.rs`: eles são **um assunto**. Aqui o assunto é *«o que se faz a um asset da
//! biblioteca»* — usar, ver quem o usa, tirá-lo.
//!
//! # ⚠️ A tabela é PLANA, e é isso que obriga cada item a falar
//!
//! O menu não sabe se o cartão é um **Prefab** ou uma **Imagem** — o
//! [`crate::interaction::ContextMenuKind::AssetCard`] carrega a CÉLULA, e quem a converte em
//! endereço de asset é o painel (a única crate que conhece o `AssetRef`). ⇒ os três itens aparecem
//! sobre as duas famílias, e **três das seis células são recusas**. Quem as redige é o shell
//! (`shells/desktop/src/asset_card_verbs.rs`), porque o `PanelHostInternal` não dá `ToastQueue` e
//! uma recusa muda é pior que um item ausente.

use super::*;

/// ⭐ **Instanciar** o prefab do cartão. Numa Imagem responde com o motivo: pôr uma imagem na cena
/// é a **queda** num alvo, porque *qual* objecto a recebe é o que a queda responde e um item de
/// menu não.
/// ⭐⭐⭐ **EDITAR o componente do cartão** (report do Enio, 2026-09-05: *«não tem como editar o
/// componente»*).
///
/// # ⚠️ Ele SELECCIONA a receita, e é isso que a torna editável
///
/// A receita não está na cena — ela volta **enquanto está seleccionada** (`master_editing`, a marca
/// derivada `MasterEditing`). ⇒ o verbo não precisa de um modo novo nem de uma janela: ele põe a
/// selecção na raiz do mestre, e a maquinaria que já existe faz o resto — o canvas desenha-a, o
/// gizmo pega-a, o Inspector mostra-a, e mover uma peça dela chega a todas as cópias no mesmo
/// quadro.
///
/// ⛔ **A biblioteca era READ-ONLY para a FORMA.** Ela listava os componentes, instanciava,
/// respondia quem usa o quê — e não tinha como abrir um. *Um catálogo do qual não se pode editar o
/// conteúdo é uma vitrina.*
pub const CTX_MENU_ASSET_EDIT: NodeId = hash_node_id("ctx_menu_asset_edit");
pub const CTX_MENU_ASSET_INSTANTIATE: NodeId = hash_node_id("ctx_menu_asset_instantiate");
/// ⭐⭐ **Quem usa isto?** — a metade que o Godot chama *Owners* e que responde *«posso apagar?»*.
///
/// ⚠️ Ela **selecciona** os utilizadores na cena em vez de os listar: uma lista diz um número, uma
/// selecção põe o artista em cima deles, com o gizmo e o Inspector já apontados.
pub const CTX_MENU_ASSET_SELECT_USERS: NodeId = hash_node_id("ctx_menu_asset_select_users");
/// ⭐⭐ **O que este asset USA** — a metade *Dependencies* do plano 07 D9.
///
/// ⚠️ **Ela FILTRA a grade em vez de listar num diálogo**, e a razão é a irmã da do
/// [`CTX_MENU_ASSET_SELECT_USERS`]: uma lista diz nomes; a grade filtrada entrega **os cartões**,
/// sobre os quais todos os outros gestos (arrastar, o menu, o duplo-clique) continuam a valer.
/// *A resposta a «quem são?» tem de ser accionável, senão é um relatório.*
///
/// ⛔ **Não passa pelo barramento**, ao contrário dos três verbos vizinhos: ele muda **o que a
/// grade mostra**, que é vista do painel — como o chip de família e a linha de catálogo. Levá-lo
/// ao shell seria pedir ao mundo uma decisão que só o painel tem.
pub const CTX_MENU_ASSET_USES: NodeId = hash_node_id("ctx_menu_asset_uses");
/// ⭐⭐ **O que USA este asset** — a metade *Owners* de D9, em termos de BIBLIOTECA.
///
/// ⚠️⚠️ **Não é o [`CTX_MENU_ASSET_SELECT_USERS`], e a diferença decide um gesto.** Aquele responde
/// *que objectos da CENA usam isto* (e selecciona-os no canvas); este responde *que RECEITAS da
/// biblioteca usam isto* — e uma receita não está na cena, logo nenhuma selecção a alcança. Um
/// artista que vai mudar uma textura precisa das duas, e elas nunca têm a mesma resposta.
pub const CTX_MENU_ASSET_USED_BY: NodeId = hash_node_id("ctx_menu_asset_used_by");
/// ⭐⭐ **Tirar da biblioteca** (report do Enio, 2026-08-30: *«não dá para tirar um asset da
/// biblioteca. Ele entra e fica»*). A lei das duas metades — as cópias destacam-se e a receita
/// dissolve-se, **ou** ela volta à cena por ser a última cópia — vive em
/// `shells/desktop/src/instance_unmake.rs`.
pub const CTX_MENU_ASSET_REMOVE: NodeId = hash_node_id("ctx_menu_asset_remove");

/// ⭐⭐⭐ **TROCAR o que está seleccionado por este componente** (ADR-0164 / plano F5, o último
/// critério) — **sem levar excepção nenhuma**, que é o `None` do `ObjectMatchMode` do Unity.
///
/// # ⚠️ São TRÊS itens, e é isso que cumpre *«nunca automático»*
///
/// O plano F5 escreve: *«trocar para mestre NÃO aparentado: só por gesto, com os 3 modos
/// (`Nenhum` default · `Por nome` · `Por hierarquia`) + relatório. ⛔ Nunca automático (HR-5)»*.
/// Sem parentesco não existe mapa derivado — só um **palpite** —, e um palpite que o app escolhe
/// sozinho é a heurística que esta linha recusa desde a F5.1. ⇒ o modo é escolhido **pelo item que
/// o artista aperta**, e este, o sem adjectivo, é o seguro: ele não adivinha nada.
///
/// ⚠️ **O sujeito é a SELECÇÃO, e o cartão é o objecto** — ao contrário dos cinco itens acima, cujo
/// sujeito é o próprio cartão. O rótulo di-lo em voz alta (*«Replace selection…»*), porque um menu
/// que age sobre outra coisa que a apontada tem de a nomear.
pub const CTX_MENU_ASSET_REPLACE: NodeId = hash_node_id("ctx_menu_asset_replace");
/// ⭐⭐ **Trocar levando as excepções pelo CAMINHO DE NOMES** — sobrevive a reordenar os irmãos,
/// perde-se com um renomear. Ver [`CTX_MENU_ASSET_REPLACE`] e `shells/desktop/src/instance_swap_match.rs`.
pub const CTX_MENU_ASSET_REPLACE_BY_NAME: NodeId = hash_node_id("ctx_menu_asset_replace_by_name");
/// ⭐⭐ **Trocar levando as excepções pelo CAMINHO DE POSIÇÕES** — o par simétrico do de cima:
/// sobrevive a renomear, perde-se ao reordenar. ⛔ Nenhum dos dois contém o outro, e é por isso que
/// os dois existem.
pub const CTX_MENU_ASSET_REPLACE_BY_TREE: NodeId = hash_node_id("ctx_menu_asset_replace_by_tree");

/// ⭐⭐ **Tirar da biblioteca, a partir da HIERARQUIA** — o mesmo verbo, o outro sujeito.
///
/// ⚠️ **Ele existe porque o `Verb::Unmake` já aceita os dois** (a receita e uma cópia dela), e sem
/// este item essa metade era **inalcançável**: o único produtor era o cartão do navegador, cujo
/// sujeito é sempre a raiz da receita. A auditoria de 2026-08-30 apanhou-o — *um gate que chama a
/// função directamente fabrica um alcance que a UI não tem*.
///
/// ⚠️ ⛔ Id PRÓPRIO, e não o do cartão: a tabela do menu da Hierarquia despacha por `row` e a do
/// cartão por `cell`. Partilhar o id faria o guarda de um painel consumir o pedido do outro.
pub const CTX_MENU_HIER_REMOVE_FROM_LIBRARY: NodeId =
    hash_node_id("ctx_menu_hier_remove_from_library");

// ── ⭐⭐ O menu de uma LINHA DE CATÁLOGO (wave A3) ───────────────────────────────────────────────
//
// ⚠️ **Ids próprios, e não os do cartão:** os dois menus despacham por sujeitos diferentes (uma
// célula · uma linha), e partilhar um id faria o guarda de um consumir o pedido do outro.

/// Renomear o catálogo — abre o campo in-place sobre a linha.
pub const CTX_MENU_CATALOG_RENAME: NodeId = hash_node_id("ctx_menu_catalog_rename");
/// Apagar o catálogo **e os descendentes**. ⛔ Nunca apaga um asset — eles voltam a *Unassigned*, e
/// a voz di-lo.
pub const CTX_MENU_CATALOG_DELETE: NodeId = hash_node_id("ctx_menu_catalog_delete");
