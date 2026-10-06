//! **A ferramenta de OSSO no canvas** (A14) — o press e o release do osso, movidos do ramo vetorial
//! (eram o `DrawMode::Bone` da ferramenta Vector, que desde a onda dos modos só está na mão no Edit
//! de uma forma). Os corpos são os de antes; o movimento (posar, pintar peso) não depende da
//! ferramenta e fica no `ramo_mover_vetor`.

use super::*;

impl crate::App {
    /// O osso no canvas, com a ferramenta de osso na mão: o press decide pela porta
    /// `bone_gesture::press` e o release faz nascer o osso do arrasto.
    pub(super) fn ramo_ferramenta_osso(
        &mut self,
        mapped_button: ph2d_host::PointerButton,
        kind: PointerKind,
        on_canvas: bool,
        evt: PointerEvent,
        menu_open_before: bool,
    ) -> bool {
        if !self.skeleton.tool_in_hand || menu_open_before {
            return false;
        }
        match (mapped_button, kind) {
            (ph2d_host::PointerButton::Primary, PointerKind::Down) if on_canvas => {
                // Um press de canvas com um campo de texto focado tem de o largar (o ramo consome
                // o press e salta o despacho do chrome que o faria) — como no ramo vetorial.
                if self.text_entry_focused() {
                    let _ = forward_to_hero(self.gfx.as_mut(), evt);
                }
                self.osso_premido();
                true
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Up) => self.osso_solto(),
            _ => false,
        }
    }

    // ⭐⭐⭐ **O OSSO** (estudo 42 item 5, doc 47 §2.6): apontar um osso
    // SELECCIONA-o (é assim que se ramifica); o vazio marca a ORIGEM, e o `release`
    // faz o osso dali até onde a mão soltou.
    //
    // ⛔ Consome o press SEMPRE que a ferramenta está na mão, pela razão do Trim e
    // do Balde: um clique no vazio não pode cair na cadeia de baixo e começar a
    // desenhar uma forma.
    // ⭐⭐⭐ **O OSSO** (estudo 42 item 5, doc 47 §2.6): a DECISÃO vive na porta
    // única `bone_gesture::press` — aqui ficam só os efeitos. ⚠️ Foi tê-la dentro
    // deste ficheiro que escondeu a metade que faltava (report do Enio,
    // 2026-09-06: *"o bind não funciona"* — apontar uma forma nunca a
    // seleccionava, e o botão só sabia recusar).
    //
    // ⛔ Consome o press SEMPRE que a ferramenta está na mão, pela razão do Trim e
    // do Balde: um clique no vazio não pode cair na cadeia de baixo e começar a
    // desenhar uma forma.
    fn osso_premido(&mut self) {
        if let Some(world) = self.vec_world_at(self.last_pointer) {
            let px = self.vec_px_to_world();
            let sel = self.selected_bone_bits();
            // ⭐ O VERBO do arrasto, que o grupo alternável da seção SKELETON diz.
            let acao = self.skeleton.tool.action;
            // ⭐ O PINCEL de peso — os dois números que o painel autora. Os outros dois verbos
            // não os lêem, e o `Pincel::INERTE` di-lo em voz alta lá dentro.
            let pincel = crate::bone_gesture::Pincel {
                ppm: self
                    .gfx
                    .as_ref()
                    .and_then(|g| g.hero_screen.as_ref())
                    .map_or(crate::EPS_PIXELS_PER_METER, |h| h.project.pixels_per_meter),
                // ⚠️ **O raio do painel e' de ECRA e a lei fala MUNDO** — a conversao e' aqui,
                // com o MESMO factor que o pick desta casa usa (`vec_px_to_world`). Sem ela o
                // `20.0` de fabrica valia 2 000 px e agarrava a peca inteira (report 19/09).
                raio: self.skeleton.tool.weight_radius * self.vec_px_to_world(),
            };
            let decisao = self.gfx.as_ref().map(|g| {
                crate::bone_gesture::press(
                    &g.sim,
                    &g.vec_scene,
                    &self.vec.pen,
                    world,
                    px,
                    sel,
                    acao,
                    pincel,
                )
            });
            match decisao {
                Some(crate::bone_gesture::BonePress::Grab { bone, part }) => {
                    // Agarrar o osso é o gesto de o POSAR (o gizmo de sprite não
                    // serve — ver `bone_pose::pose`), e também o que o
                    // selecciona: o pai do próximo osso é o que está aceso.
                    self.skeleton.bone_pose = Some((bone, part));
                    if let Some(gfx) = self.gfx.as_mut()
                        && let Some(hero) = gfx.hero_screen.as_mut()
                    {
                        hero.gizmo.selection = Some(bone);
                        hero.gizmo.extra_selection.clear();
                    }
                }
                // ⛔ Em *Transformar*, um press fora de osso NÃO aponta a forma (A14): o Bind é de
                // Object, e em Pose o cadeado não a deixa escolher — apontá-la na selecção da
                // caneta era um estado INVISÍVEL que o Bind de Object depois prendia (report de
                // 05/10). Fora de osso o press não faz nada.
                Some(crate::bone_gesture::BonePress::Pick { .. }) => {}
                // ⛔ Nem o press que começa um osso aponta a forma por baixo (A14, a mesma razão).
                Some(crate::bone_gesture::BonePress::Start { birth, .. }) => {
                    self.skeleton.bone_drag = Some(birth);
                }
                // ⭐⭐⭐ **O PINCEL DE PESO** — o traço fica preso à arte em que começou (o
                // `weight_drag`), e a 1.ª pincelada sai já neste press: *um pincel que só pinta
                // quando a mão se move faz um clique parecer um clique morto*.
                Some(crate::bone_gesture::BonePress::Weight { alvo }) => {
                    self.skeleton.weight_drag = alvo;
                    self.vec_pinta_peso(world);
                }
                None => {}
            }
        }
    }

    /// O Up apaga as guias de snap, e o osso nasce do arrasto: a mesma leitura que a pré-visualização desenhou,
    /// o pai que o press apontou, a emenda da corrente solta e a memória do revelar-ao-focar.
    fn osso_solto(&mut self) -> bool {
        // Fim de gesto: as guias de snap não sobrevivem ao Up.
        self.vec_clear_snap_guides();
        // ⭐⭐⭐ **O OSSO nasce aqui** (estudo 42 item 5): origem no press, comprimento e
        // ângulo no arrasto, PAI = o osso seleccionado — e o novo fica seleccionado, que
        // é o que faz arrasto-arrasto-arrasto ser uma cadeia.
        //
        // ⚠️ **Consome SÓ com o gesto VIVO** (a origem marcada), pela lei que o
        // `shape_up_consumes` documenta: soltar sobre um botão do painel neste modo não
        // pode engolir o clique.
        // ⭐ **O traço de PESO acaba aqui** — e a lei do fim é largar o alvo, não pintar mais um
        // dab: o último já saiu no movimento. ⛔ Ele não consome o `up`, porque nada mais no modo
        // Osso reage a um `up` sem `bone_drag`.
        self.skeleton.weight_drag = None;
        if let Some(nascimento) = self.skeleton.bone_drag.take() {
            let px = self.vec_px_to_world();
            let mut nasceu = None;
            if let Some(solto) = self.vec_world_at(self.last_pointer) {
                // ⭐⭐⭐ **A MESMA leitura que a pré-visualização desenhou**
                // ([`crate::bone_gesture::drag_now`]): a emenda, a ponta encaixada e o
                // limiar. O artista viu o osso saltar para aquela bolinha, e é
                // exactamente ali que ele nasce.
                //
                // ⚠️ **A EMENDA** (ordem do dono, 2026-09-09): se o arrasto acaba na
                // BASE de uma corrente solta, a ponta do osso novo encaixa nela e essa
                // corrente passa a pendurar-se nele — duas correntes viram uma.
                let Some(agora) = self
                    .gfx
                    .as_ref()
                    .map(|g| crate::bone_gesture::drag_now(&g.sim, nascimento, solto, px))
                else {
                    return true;
                };
                let (ponta, emenda) = (agora.tip, agora.splice);
                if agora.armed
                    && let Some(gfx) = self.gfx.as_mut()
                {
                    // ⭐⭐⭐ **O PAI é o que o PRESS apontou** (ordem do dono,
                    // 2026-09-09) — ⛔ nunca a selecção, que era a lei que ele mandou
                    // tirar. Ele ainda é filtrado porque um osso pode ter sido apagado
                    // entre o press e o release, e um pai morto não tem espaço local.
                    let pai = nascimento
                        .parent
                        .and_then(ph2d_ecs::Entity::try_from_bits)
                        .filter(|e| gfx.sim.world().get::<ph2d_skeleton_ecs::Bone>(*e).is_some())
                        // ⭐ Sem pai-osso, a raiz nasce FILHA do esqueleto em Edit (A14).
                        .or_else(|| {
                            self.skeleton
                                .target
                                .and_then(ph2d_ecs::Entity::try_from_bits)
                        });
                    nasceu =
                        crate::bone_gesture::create(&mut gfx.sim, pai, nascimento.origin, ponta);
                    // ⭐⭐⭐ **E a corrente solta passa a pendurar-se no osso novo.**
                    //
                    // ⚠️ **Depois do `create`, nunca antes:** o pai só existe agora, e
                    // adoptar antes dele nascer não tem onde pendurar. ⚠️ E o `connect`
                    // preserva a pose de MUNDO do adoptado — sem isso o esqueleto
                    // inteiro saltaria pela pose do osso novo.
                    if let (Some(novo), Some((alvo, _))) = (nasceu, emenda) {
                        crate::bone_gesture::connect(&mut gfx.sim, alvo, novo);
                    }
                    if let Some(bits) = nasceu
                        && let Some(hero) = gfx.hero_screen.as_mut()
                    {
                        hero.gizmo.selection = Some(bits);
                        hero.gizmo.extra_selection.clear();
                    }
                }
            }
            // ⭐⭐⭐ **UM OSSO ACABADO DE NASCER NÃO É UM OSSO ESCOLHIDO** (report do
            // dono, 2026-09-09: *«cada vez que se cria um osso o modo Transform é
            // selecionado»*).
            //
            // ⛔ O osso novo fica aceso — é assim que o artista vê qual é — e no quadro
            // seguinte a aresta do foco lia isso como *«o artista escolheu um osso»* e
            // armava *Transform*, arrancando-o do verbo em que ele estava. A memória
            // absorve-o AQUI, onde se sabe que ele nasceu de um arrasto e não de uma
            // escolha. *A aresta continua a valer; o que mudou é quem a alimenta.*
            if let Some(bits) = nasceu {
                crate::skeleton_reveal::on_birth(&mut self.skeleton.osso_revelado, bits);
            }
            return true;
        }
        false
    }
}
