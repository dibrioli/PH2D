//! **O MODO DE PREVIEW** (plano UI/UX W7, a metade de RUNTIME) — a UI que o artista desenhou
//! passa a **responder ao rato**, e só aqui.
//!
//! # Por que um MODO, e não simplesmente ligar o rato
//!
//! A ponte dos estados escreveu, no dia em que nasceu, as duas razões pelas quais o rato não
//! dirigia nada — e as duas continuam verdadeiras:
//!
//! 1. **Um hover que animasse a forma enquanto o artista trabalha tornaria o editor inutilizável.**
//!    É por isso que o Figma põe a interação num *modo de apresentação* separado.
//! 2. **O undo deste editor é por DIFF do mundo**, então uma pose escrita por hover viraria um
//!    passo de undo a cada vez que o rato passasse por cima de um botão.
//!
//! ⇒ o modo resolve as duas de uma vez: **enquanto ele está ligado, o gesto de edição não existe e
//! o undo não regista**; quando ele desliga, **o mundo volta exactamente ao que era**.
//!
//! # ⭐ Sair RESTAURA, e a restauração é do MUNDO — nunca do estado Default
//!
//! A tentação barata é *"ao sair, vá para o Default"*. Ela está errada, e o modo de falha é
//! silencioso: o **Default é uma pose GRAVADA**, e o artista pode ter movido a forma depois de a
//! gravar. Sair para o Default **moveria** o desenho dele — o documento mudaria por ele ter olhado.
//!
//! O que se restaura é a pose que o mundo tinha **no instante em que a preview foi ligada**. Ela é
//! capturada pela MESMA porta que o botão *Rec* usa ([`crate::vec_ui_state_edit::capture`]): uma
//! segunda leitura ao lado seria a que esquece um canal no dia em que a pose ganhar um.
//!
//! # O conjunto capturado é EXACTAMENTE o que a preview pode escrever
//!
//! A [`ph2d_ui_state::Machine`] só emite poses cujos ids aparecem nos estados autorados (o
//! `overlay` dela escreve o que a `Transition` produziu, e a `Transition` casa as duas listas
//! autoradas). ⇒ capturar todo id mencionado por qualquer estado de qualquer hospedeiro é
//! **completo por construção**, e não uma lista que envelhece. Há gate a medi-lo.
//!
//! # O rato diz *o que aconteceu*; o papel é DERIVADO
//!
//! `hot` (sobre que hospedeiro o cursor está) + `pressed` (o botão está em baixo) são os dois
//! fatos que a shell conhece. O papel sai deles por uma função, e não por uma tabela de gatilhos:
//! uma tabela teria de ser mantida em dia com a lista de papéis, e a lista já é um catálogo fixo.

use ph2d_ecs::SimWorld;
use ph2d_ui_state::{ObjectPose, StateRole, StateSets};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

use super::ui_state_bridge::{UiMachines, request};

/// O modo de preview: se está ligado, o que restaurar, e onde o rato está.
#[derive(Default)]
pub(crate) struct UiPreview {
    on: bool,
    /// A pose do MUNDO no instante em que a preview ligou — tudo o que ela pode escrever.
    ///
    /// ⚠️ Ela **não** é serializada e não pode ser: é *onde a cena estava*, um fato sobre uma
    /// sessão. O documento guarda *onde as poses são*.
    restore: Vec<ObjectPose>,
    /// **A CADEIA de hospedeiros sob o cursor, do mais INTERNO para fora.**
    ///
    /// ⚠️ Ela é uma lista e não um id porque hospedeiros **ANINHAM** (um menu que contém itens),
    /// e um menu não pode fechar quando o cursor entra num item dele. Guardar só o interno
    /// obrigaria a re-derivar os ancestrais no `point`, que não tem ECS — e a versão que não os
    /// derivava mandava o menu para o `Default`.
    hot: Vec<VecPathId>,
    /// O botão primário está em baixo **sobre o `hot`**.
    pressed: bool,
}

impl UiPreview {
    #[must_use]
    pub(crate) fn is_on(&self) -> bool {
        self.on
    }

    /// **LIGA**, capturando o mundo. Sem estado nenhum autorado ela não liga — um modo de preview
    /// sobre uma cena sem poses é um modo que não faz nada, e o artista não teria como o saber.
    pub(crate) fn enter(
        &mut self,
        machines: &mut UiMachines,
        states: &StateSets,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &VecEntityMap,
    ) -> bool {
        if self.on || states.is_empty() {
            return false;
        }
        self.restore = touched(states)
            .into_iter()
            .map(|id| crate::vec_ui_state_edit::capture(sim, scene, map, id))
            .collect();
        self.on = true;
        self.hot.clear();
        self.pressed = false;

        // ⭐ **A cena abre em REPOUSO.** Entrar na preview põe cada hospedeiro no `Default`.
        //
        // ⚠️ Sem isto ela abria no que o MUNDO tivesse — que é o que o artista deixou depois da
        // última gravação, quase sempre a pose de `Hover`. A UI parecia já estar a ser tocada
        // antes de o rato chegar perto, e o primeiro gesto **saía** de um estado em vez de entrar
        // nele.
        //
        // ⚠️ **A captura vem ANTES, e a ordem é a lei da wave:** o que `leave` devolve é o MUNDO
        // que ela encontrou, nunca o `Default` — a tentação barata (*"ao sair, vá para o
        // Default"*) **moveria o desenho** de quem gravou o Default e depois mexeu na forma.
        //
        // ⚠️ E é INSTANTÂNEO, não animado: entrar num modo não é um gesto, e a pose de onde ela
        // partiria não é um estado. A máquina nasce parada no primeiro estado gravado (que é o
        // `Default`, porque os papéis são ordenados), então não há voo a construir — só a escrita.
        //
        // ⚠️ **Um hospedeiro SEM `Default` gravado fica onde está**, de propósito: não há para
        // onde o mandar, e escolher outro papel por ele mostraria um botão em `Hover` que ninguém
        // está a tocar.
        let resting: Vec<_> = states
            .hosts()
            .filter(|&h| states.role(h, StateRole::Default).is_some())
            .collect();
        for host in resting {
            request(machines, states, host, StateRole::Default);
        }
        for m in machines.values() {
            for p in m.pose() {
                crate::vec_ui_state_edit::install(sim, scene, map, p);
            }
        }
        true
    }

    /// **DESLIGA**, devolvendo o mundo ao que era. As máquinas morrem com o modo.
    ///
    /// ⚠️ **As máquinas são limpas, e não deixadas paradas**: uma máquina viva continuaria a
    /// afirmar que a cena mostra um papel (o readout do painel lê `live_role`), e o artista veria
    /// *"Showing: Hover"* sobre uma cena que voltou ao repouso.
    pub(crate) fn leave(
        &mut self,
        machines: &mut UiMachines,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &VecEntityMap,
    ) -> bool {
        if !self.on {
            return false;
        }
        for pose in &self.restore {
            crate::vec_ui_state_edit::install(sim, scene, map, pose);
        }
        self.restore.clear();
        machines.clear();
        self.on = false;
        self.hot.clear();
        self.pressed = false;
        true
    }

    /// **O rato mexeu-se.** `chain` é a cadeia de hospedeiros sob o cursor, **do mais interno
    /// para fora** (vazia = o vazio, ou uma forma sem estados); `pressed` é o botão primário.
    ///
    /// ⚠️ **Só pede quando MUDA**, a mesma lei do read-back do picker de tokens: o rato publica
    /// posição a cada quadro, e re-pedir o mesmo papel a cada um faria o `retarget` correr sobre
    /// a tabela inteira sessenta vezes por segundo para não mudar nada.
    ///
    /// ⚠️ **Quem se DEIXA é quem SAIU DA CADEIA**, e não *"o hospedeiro anterior"*. Um ancestral
    /// do novo *hot* continua sob o cursor — o menu não fecha porque o cursor desceu para um item
    /// dele. A versão anterior comparava um id com um id, e não tinha como saber disso.
    ///
    /// ⚠️ **E só o mais INTERNO responde ao aperto.** Um `Pressed` que subisse a cadeia acenderia
    /// o menu inteiro ao clicar num item; o ancestral segura o `Hover`, que é o que ele de facto
    /// é — o cursor está dentro dele.
    /// ⭐ **Devolve o hospedeiro que foi CLICADO**, se este evento fechou um clique — é a metade
    /// de PRODUTOR do laço sinal → ação (item 4 do estudo dos contêineres).
    ///
    /// ⚠️ **Um clique é apertar e SOLTAR sobre o mesmo hospedeiro**, e a segunda metade não é
    /// zelo: sem ela, apertar num botão, arrastar para fora e soltar dispararia — o gesto
    /// universal de *desistir* viraria o gesto de confirmar. É a mesma lei que todo botão desta
    /// casa já honra.
    ///
    /// ⚠️ **É o mais INTERNO que dispara**, o mesmo da regra do `Pressed`: clicar num item de menu
    /// não pode disparar também o menu que o contém.
    ///
    /// ⚠️ **E quem resolve o NOME é a shell**, não isto: o nome de um hospedeiro é o `Name` da
    /// entidade dele, e este módulo não alcança o ECS de propósito — foi essa ausência que o
    /// manteve testável de cabeça para baixo, sem janela e sem mundo.
    pub(crate) fn point(
        &mut self,
        machines: &mut UiMachines,
        states: &StateSets,
        chain: &[VecPathId],
        pressed: bool,
    ) -> Option<VecPathId> {
        if !self.on || (chain == self.hot && pressed == self.pressed) {
            return None;
        }
        // O clique fecha AQUI: o botão subiu, ele estava em baixo, e o alvo é o mesmo.
        let clicked = (self.pressed && !pressed)
            .then(|| {
                chain
                    .first()
                    .copied()
                    .filter(|c| self.hot.first() == Some(c))
            })
            .flatten();
        let left: Vec<VecPathId> = self
            .hot
            .iter()
            .copied()
            .filter(|h| !chain.contains(h))
            .collect();
        self.hot.clear();
        self.hot.extend_from_slice(chain);
        // Apertar no vazio não "prende" hospedeiro nenhum.
        self.pressed = pressed && !chain.is_empty();

        for h in left {
            request(machines, states, h, StateRole::Default);
        }
        for &h in chain {
            request(machines, states, h, self.role_for(h));
        }
        clicked
    }

    /// O papel que `host` deve mostrar, dados os dois fatos que o rato conhece.
    ///
    /// ⚠️ O `Pressed` é do mais interno; um ancestral sob o cursor fica em `Hover`.
    fn role_for(&self, host: VecPathId) -> StateRole {
        match self.hot.first() {
            Some(&h) if h == host && self.pressed => StateRole::Pressed,
            _ if self.hot.contains(&host) => StateRole::Hover,
            _ => StateRole::Default,
        }
    }
}

/// **Todo id que a preview pode escrever** — a união dos objetos de todos os estados autorados.
///
/// ⚠️ Ordenado e sem repetições: a restauração é uma sequência de escritas, e uma lista cuja ordem
/// dependesse da iteração de um mapa faria dois `leave` logicamente iguais escreverem em ordens
/// diferentes — invisível hoje, e a forma exacta de um bug que só aparece quando duas poses
/// disputam o mesmo id.
#[must_use]
fn touched(states: &StateSets) -> Vec<VecPathId> {
    let mut ids: Vec<VecPathId> = states
        .hosts()
        .flat_map(|h| {
            states
                .get(h)
                .iter()
                .flat_map(|s| s.objects.iter().map(|o| o.id))
                .collect::<Vec<_>>()
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// **A CADEIA de hospedeiros sob este ponto de mundo** — do mais INTERNO para fora, vazia se
/// nenhum.
///
/// ⚠️ Um hospedeiro é um GRUPO (um botão é fundo + rótulo), então um toque no rótulo é um toque no
/// botão: a pergunta é *"o que foi tocado pertence à sub-árvore de algum hospedeiro?"*, e não
/// *"o que foi tocado é um hospedeiro?"*. A segunda faria o hover morrer sempre que o cursor
/// passasse por cima do texto.
///
/// ⚠️ A lista de picks vem em ordem de Z (a frente primeiro), então o **primeiro** que pertence a
/// um hospedeiro ganha — o de cima é o que o artista vê.
///
/// ⭐ **E ela devolve uma CADEIA, não um hospedeiro, porque hospedeiros ANINHAM.** A versão que
/// devolvia um só varria `states.hosts()` — um `BTreeMap` — e parava no primeiro que contivesse
/// o pick, então com um menu a conter um item o vencedor era decidido por **qual `VecPathId` era
/// menor**. Aqui a cadeia é **ordenada por PERTENÇA**: se `a` está na sub-árvore de `b`, então
/// `a` é mais interno — nenhuma ordem de mapa entra na resposta.
#[must_use]
pub(crate) fn host_under(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    states: &StateSets,
    picked: &[VecPathId],
) -> Vec<VecPathId> {
    for id in picked {
        let mut chain: Vec<VecPathId> = states
            .hosts()
            .filter(|&h| h == *id || members(sim, scene, map, h).contains(id))
            .collect();
        if chain.is_empty() {
            continue;
        }
        // Do mais interno para fora: `a` antes de `b` quando `a` está DENTRO de `b`.
        //
        // ⚠️ A relação é a pertença, e não a profundidade contada à parte: contar profundidade
        // seria uma segunda travessia da árvore, com a chance de discordar da primeira.
        chain.sort_by(|a, b| {
            let a_in_b = members(sim, scene, map, *b).contains(a);
            let b_in_a = members(sim, scene, map, *a).contains(b);
            match (a_in_b, b_in_a) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                // Irmãos (nenhum contém o outro) mantêm uma ordem estável, e ela não é
                // observável: `point` acende a cadeia inteira em `Hover` menos o primeiro.
                _ => a.cmp(b),
            }
        });
        return chain;
    }
    Vec::new()
}

use crate::vec_ui_state_edit::members;

#[cfg(test)]
#[path = "ui_preview_tests.rs"]
mod tests;
