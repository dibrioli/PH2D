//! **A SIMETRIA VIVA na shell** — o cozimento das cópias do [`ph2d_ecs::VecSymmetry`].
//!
//! Espelho do [`crate::offset_live`] / [`crate::contour_live`]: o componente guarda a relação (que
//! espelho, onde, quantas cópias) e a aparência é uma **função pura** dela, re-cozida aqui e
//! desenhada por [`ph2d_vec_render::dispatch`] **no z da forma**.
//!
//! # A simetria é da SESSÃO DE DESENHO, e não da forma seleccionada
//!
//! *"A linha de simetria deve aparecer logo que se aperta o botão e não quando se inicia o
//! desenho. A simetria funciona apenas para formas que serão desenhadas com a tool ligada e não
//! deve fazer simetria de formas que já existem previamente. Com o botão checado pode-se fazer
//! quantos desenhos desejar que a linha de simetria permanece no lugar"* (Enio, 2026-08-01).
//!
//! São três exigências e elas decidem o modelo inteiro:
//!
//! 1. **A linha existe sem forma nenhuma.** Logo ela não pode ser propriedade de uma forma: há um
//!    **eixo de SESSÃO**, em coordenadas de MUNDO, semeado no centro do ECRÃ no instante em que o
//!    botão liga. É ele que aparece, com a cena vazia e nada seleccionado.
//! 2. **Só o que for DESENHADO com o modo ligado espelha.** Logo armar **nunca** toca a selecção —
//!    a adopção acontece no nascimento, e o oráculo de *"o artista está a desenhar isto agora"* é
//!    o [`crate::vec_transform::gesture_paths`], a mesma porta que o `settle_origins` usa. Um
//!    diff de ids apanharia colar, o restore do undo e o próprio *Apply*, e espelharia coisas que
//!    o artista não desenhou.
//! 3. **O eixo fica no lugar entre desenhos.** Logo ele não segue a câmera depois de semeado nem
//!    salta para cada forma nova: uma semeadura por LIGAÇÃO, e ela vale para a sessão toda.
//!
//! E a promessa original continua de pé — *"uma vez que o desenho é feito, a referência para a
//! linha passa a ser o próprio desenho"*: cada forma **captura** o eixo de sessão no espaço LOCAL
//! dela, e a partir daí move-se com ela.
//!
//! # A captura só sela quando o pivô assenta
//!
//! ⚠️ Enquanto a forma está em gesto o [`crate::vec_transform::settle_origins`] pula-a, e no frame
//! em que o gesto acaba ele **translada a geometria e compensa no `Transform`** para pôr o pivô no
//! centro. Um eixo capturado antes disso ficaria deslocado exactamente por essa translação — e em
//! silêncio, porque nada falha. Por isso a re-derivação corre enquanto a forma está em gesto **e
//! mais uma vez no frame seguinte** (o `prev_drawing`), que é justamente o frame em que o pivô
//! assenta. Depois disso o eixo é DELA e ninguém lhe toca.
//!
//! # Desarmar esconde, não destrói
//!
//! *"Se a simetria for desmarcada antes do apply, as cópias somem mas não são destruídas"*. O
//! interruptor gateia o **COZIMENTO**, não o componente: as cópias desaparecem porque deixam de
//! ser cozidas, e voltam inteiras ao religar. Elas nunca estiveram no documento, então não há
//! nada a destruir — e nada a reconstruir.
//!
//! # O cozimento é LOCAL, e a ORDEM é a espinha
//!
//! O eixo vive no espaço da geometria autorada (ver o cabeçalho do componente), então a reflexão
//! corre **antes** de a pose entrar: `reflectir → assar a pose`. A ordem inversa só daria o mesmo
//! resultado se a pose fosse uma semelhança — sob escala não-uniforme ou skew as duas divergem, e
//! a que corresponde ao que o artista vê é esta (o espelho é uma propriedade da FORMA, e a pose
//! leva a forma inteira, cópias incluídas).
//!
//! ⚠️ É por isso que este módulo **não** pode copiar o `offset_live` verbatim: lá a distância é de
//! MUNDO, então ele assa primeiro e offseta depois. Aqui é o contrário, e a chave do memo tem de
//! carregar a pose junto (o resultado depende dela).
//!
//! # Quem lê a DERIVADA e quem lê a FONTE
//!
//! Lêem a derivada os que respondem *"o que está na tela aqui?"* — o [`ph2d_vec_render::dispatch`]
//! e o hit-test de canvas. Lêem a FONTE os que respondem *"o que o artista escreveu?"*: o modo
//! **Node** (as âncoras que ele arrasta são as autoradas — a mesma lei do raio de quina vivo,
//! ADR-0121) e a **caixa do gizmo**. Nenhum dos dois é esquecimento.
//!
//! # O memo
//!
//! O cozimento roda por frame, então sem memo uma cena parada pagaria a reflexão para sempre. A
//! chave é o que de facto determina o resultado — a geometria LOCAL que entra, a pose, e a spec —
//! e não um contador de versão que alguém esqueceria de bumpar.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecSymmetry};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::symmetry::{SymmetrySpec, symmetry_paths};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, Xform, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a geometria local + a pose + a spec) e o que SAIU.
struct Memo {
    spec: SymmetrySpec,
    xform: Xform,
    local: VecPath,
    out: Vec<VecPath>,
}

/// A direcção do eixo de SESSÃO, em mundo, antes de o `kind` opinar.
///
/// Hoje é sempre esta: não existe gesto para o artista desenhar a linha (item aberto, nomeado no
/// handoff), então `Custom` cai na vertical — o que a [`SymmetrySpec::mirror_dir`] já faz para uma
/// direcção degenerada. Os espelhos X/Y ignoram-na por construção.
const SESSION_DIR: [f64; 2] = [0.0, 1.0];

/// O cozimento vivo de todas as simetrias da cena, com memo por caminho.
#[derive(Default)]
pub(crate) struct SymmetryLive {
    memo: BTreeMap<VecPathId, Memo>,
    live: LiveGeometry,
    /// Quem estava em gesto no frame ANTERIOR — o frame em que o pivô assenta. Ver o cabeçalho:
    /// sem isto a captura fica deslocada pela translação do `settle_origins`, em silêncio.
    prev_drawing: Vec<VecPathId>,
}

impl SymmetryLive {
    /// A geometria derivada deste frame — o que o [`ph2d_vec_render::dispatch`] desenha no lugar
    /// da fonte. Vazia = nenhuma simetria viva na cena, e o desenho é o de sempre.
    pub(crate) fn live(&self) -> &LiveGeometry {
        &self.live
    }

    /// **Adopta** o que está a ser desenhado e re-afina o ESTILO do que já está armado.
    ///
    /// `origin` é o eixo de sessão em MUNDO. Duas metades, e a divisão é a lei do modelo:
    ///
    /// - o que está **em gesto** (e o que acabou de sair dele) tem o eixo re-derivado do de
    ///   sessão, porque a captura ainda não selou (ver o cabeçalho: o pivô assenta um frame
    ///   depois);
    /// - o que **já está armado** mantém o LUGAR dele e recebe só o estilo — é isto que faz
    ///   arrastar *Segments* actualizar a sessão inteira sem teleportar eixo nenhum.
    ///
    /// ⚠️ Uma forma que não tem o componente e não está em gesto **nunca** é tocada: é essa
    /// ausência que cumpre *"não deve fazer simetria de formas que já existem previamente"*.
    ///
    /// Devolve quantas formas estão armadas — o que o painel usa para oferecer o *Apply*.
    // Quatro dos oito argumentos são o MUNDO (`sim`/`map`/`scene`/`xforms`), a mesma forma
    // que o `recook` irmão já tem; agrupá-los num tipo só para calar a lint criaria uma
    // struct cujo único trabalho é existir. Precedente: `physics::body_desc`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn adopt(
        &mut self,
        sim: &mut SimWorld,
        map: &VecEntityMap,
        scene: &VecScene,
        xforms: &VecXforms,
        style: ph2d_vec_scene::symmetry::SymmetryStyle,
        origin: [f64; 2],
        drawing: &[VecPathId],
    ) -> usize {
        for id in drawing.iter().chain(self.prev_drawing.iter()) {
            // O eixo de sessão é de MUNDO; o componente guarda-o no LOCAL da forma. Uma pose
            // não-invertível (escala zero) não tem local nenhum — fica com o mundo, que é o que
            // ela desenha de qualquer maneira.
            let (center, dir) = match ph2d_vec_scene::xform_of(xforms, *id).inverse() {
                Some(inv) => (inv.apply(origin), inv.apply_vec(SESSION_DIR)),
                None => (origin, SESSION_DIR),
            };
            let spec = SymmetrySpec::from_style(style, center, dir);
            arm(sim, map, std::slice::from_ref(id), Some(spec));
        }
        self.prev_drawing.clear();
        self.prev_drawing.extend_from_slice(drawing);

        let mut live = 0;
        for path in scene.paths() {
            let Some(cur) = spec_of(sim, map, path.id) else {
                continue;
            };
            live += 1;
            let want = SymmetrySpec::from_style(style, cur.center, cur.dir);
            arm(sim, map, std::slice::from_ref(&path.id), Some(want));
        }
        live
    }

    /// Re-coze o que mudou. Chamado uma vez por frame, DEPOIS do `sync` (senão uma forma
    /// recém-criada ainda não tem entidade e o componente dela não seria encontrado).
    ///
    /// ⚠️ `on` gateia o COZIMENTO e não o componente — é o que faz desarmar esconder as cópias
    /// sem as destruir, e religar trazê-las de volta inteiras. O memo sobrevive de propósito: ele
    /// é chaveado por tudo o que determina a resposta, então religar não paga nada.
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        on: bool,
    ) {
        self.live.clear();
        if !on {
            return;
        }
        for path in scene.paths() {
            let Some(spec) = spec_of(sim, map, path.id) else {
                continue;
            };
            // A fonte é a curva COZIDA (raio de quina + pilha de efeitos já correram) mas ainda em
            // coordenadas LOCAIS — é nelas que o eixo autorado vive.
            let local = path.cooked().into_owned();
            let xform = xform_of(xforms, path.id);
            let hit = self
                .memo
                .get(&path.id)
                .is_some_and(|m| m.spec == spec && m.xform == xform && m.local == local);
            if !hit {
                let mut out = symmetry_paths(&local, &spec);
                for p in &mut out {
                    bake_xform(p, &xform);
                }
                self.memo.insert(
                    path.id,
                    Memo {
                        spec,
                        xform,
                        local: local.clone(),
                        out,
                    },
                );
            }
            if let Some(m) = self.memo.get(&path.id) {
                self.live.insert(path.id, m.out.clone());
            }
        }
        // ⚠️ Isto é HIGIENE, e não correcção — a distinção importa para quem vier depois. A chave
        // carrega tudo o que determina a resposta (spec + pose + geometria local), então uma
        // entrada velha só acerta quando as três coincidem, e aí ela está CERTA. O que o `retain`
        // impede é o memo crescer com formas que já não têm simetria. Não há gate porque não há
        // falha a produzir; se a chave um dia deixar de ser completa, esta linha passa a ser
        // correcção e o gate nasce com ela.
        self.memo.retain(|id, _| self.live.contains_key(id));
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do memo,
    /// e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
        self.live.clear();
        self.prev_drawing.clear();
    }
}

/// **Toda forma armada da cena** — o alvo do *Apply*.
///
/// ⚠️ A selecção não entra nisto, e a razão é o modelo: a simetria é um MODO, então *"consolidar a
/// forma e desativar a simetria"* vale para o que o modo produziu, não para o que estiver
/// seleccionado no momento do clique. Consolidar só o seleccionado deixaria metade das cópias na
/// tela com o modo já desligado — cópias que o cozimento gateado deixaria de desenhar, e o artista
/// veria o trabalho evaporar.
pub(crate) fn armed_paths(sim: &SimWorld, map: &VecEntityMap, scene: &VecScene) -> Vec<VecPathId> {
    scene
        .paths()
        .iter()
        .filter(|p| spec_of(sim, map, p.id).is_some())
        .map(|p| p.id)
        .collect()
}

/// **A linha de SESSÃO** — a que aparece no instante em que o botão liga, com a cena vazia.
///
/// ⚠️ A direcção sai da MESMA [`SymmetrySpec::mirror_dir`] que o kernel usa para reflectir: a
/// linha de sessão e o eixo que a próxima forma vai capturar são o mesmo fato, e derivá-lo duas
/// vezes desenharia a promessa num sítio e cumpri-la-ia noutro.
pub(crate) fn session_axis(
    style: ph2d_vec_scene::symmetry::SymmetryStyle,
    origin: [f64; 2],
) -> ph2d_vec_render::SymmetryAxis {
    use ph2d_vec_scene::symmetry::SymmetryKind;
    let spec = SymmetrySpec::from_style(style, origin, SESSION_DIR);
    ph2d_vec_render::SymmetryAxis {
        at: origin,
        dir: spec.mirror_dir(),
        segments: (spec.kind == SymmetryKind::Radial).then(|| spec.segments()),
    }
}

/// A simetria viva de `id`, se houver. Porta única: o cozimento, o **Apply**, o overlay das linhas
/// e o publish para o painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<SymmetrySpec> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecSymmetry>(Entity::from_bits(bits))
        .map(|s| s.spec)
}

/// **Arma** (ou desarma) a simetria viva de cada caminho de `ids`.
///
/// `spec = None` **REMOVE** o componente em vez de guardar um espelho inerte — é a lei do
/// `VecOffset` com `d = 0`, e é ela que faz *"desmarcar antes do Apply"* devolver a forma ao
/// estado limpo em vez de a deixar com uma relação invisível pendurada.
///
/// Devolve quantas entidades mudaram.
pub(crate) fn arm(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    spec: Option<SymmetrySpec>,
) -> usize {
    let want = spec.map(VecSymmetry::new);
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let cur = sim.world().get::<VecSymmetry>(e).copied();
        if cur == want {
            continue;
        }
        let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        match want {
            Some(v) => {
                em.insert(v);
            }
            None => {
                em.remove::<VecSymmetry>();
            }
        }
        n += 1;
    }
    n
}

/// **MATERIALIZA** as cópias da seleção — o botão *Apply*.
///
/// *"Botão apply para consolidar a forma e desativar a simetria"* (Enio, 2026-08-01). É o único
/// momento em que as cópias passam a existir no documento; até aqui elas eram desenho, e é isso
/// que faz o desarmar não destruir nada.
///
/// ⚠️ **Uma porta, dois consumidores:** o `layers_for` daqui produz a MESMA lista que o
/// [`SymmetryLive::recook`] cozinha — `symmetry_paths` sobre a curva local, e só então a pose. Se
/// o Apply tivesse a sua própria produção, a forma SALTARIA no clique, que é o defeito que o
/// ADR-0128 pagou cinco vezes.
///
/// ⚠️ **O componente não precisa de ser removido:** a forma-fonte SAI da cena (remove+insere) e o
/// `vec_entities::sync` do frame seguinte despawna a entidade dela com o componente dentro. Quem
/// não produziu nada fica — e fica COM a simetria, porque apagar a arte do artista num clique de
/// *Apply* seria a pior resposta possível a *"não há nada aqui"*.
///
/// `false` = não havia simetria viva na seleção. **UM passo de undo** para o gesto inteiro.
pub(crate) fn materialise(
    scene: &mut VecScene,
    sim: &SimWorld,
    pen: &mut ph2d_vec_edit::PenTool,
    history: &mut ph2d_vec_edit::History,
    map: &VecEntityMap,
    xforms: &VecXforms,
    ids: &[VecPathId],
) -> bool {
    let live: Vec<(VecPathId, SymmetrySpec)> = ids
        .iter()
        .filter_map(|id| spec_of(sim, map, *id).map(|s| (*id, s)))
        .collect();
    if live.is_empty() {
        return false;
    }
    let pre = scene.clone();
    let touched =
        crate::vec_expand::materialise_selection(scene, pen, xforms, ids, |id, local, xf| {
            let Some((_, spec)) = live.iter().find(|(i, _)| *i == id) else {
                return Vec::new();
            };
            // A MESMA ordem do cozimento: reflectir no espaço da forma, e só então assar a pose.
            let mut out = symmetry_paths(&local, spec);
            for p in &mut out {
                bake_xform(p, xf);
            }
            out
        });
    if touched {
        history.push_undo(pre);
    }
    eprintln!("[ph2d-vec] simetria materializada: {} forma(s)", live.len());
    true
}

/// **Os eixos vivos da cena, em MUNDO** — o que o overlay desenha.
///
/// ⚠️ A direcção sai de [`SymmetrySpec::mirror_dir`], a MESMA porta que o kernel usa para
/// reflectir, e só depois sobe pela pose (`apply_vec`, não `apply`: levar uma direção como ponto a
/// transladaria). Uma segunda derivação aqui desenharia um eixo onde a geometria não espelha — e
/// ninguém lê um número numa screenshot, então a divergência apareceria como *"a linha está
/// torta"*, meses depois.
pub(crate) fn live_axes(
    scene: &VecScene,
    sim: &SimWorld,
    map: &VecEntityMap,
    xforms: &VecXforms,
) -> Vec<ph2d_vec_render::SymmetryAxis> {
    use ph2d_vec_scene::symmetry::SymmetryKind;
    scene
        .paths()
        .iter()
        .filter_map(|p| spec_of(sim, map, p.id).map(|s| (p.id, s)))
        .map(|(id, spec)| {
            let xf = xform_of(xforms, id);
            ph2d_vec_render::SymmetryAxis {
                at: xf.apply(spec.center),
                dir: xf.apply_vec(spec.mirror_dir()),
                segments: (spec.kind == SymmetryKind::Radial).then(|| spec.segments()),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "symmetry_live_tests.rs"]
mod tests;
