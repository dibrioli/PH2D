//! ⭐ **O que o painel PEDE e a cena FAZ** — o outro lado da ponte.
//!
//! ⚠️ **Módulo-filho de [`super`]**, e o corte é por **assunto**, o mesmo do irmão
//! [`panel`](super::panel): aquele responde *"o que o painel mostra e o que ele oferece"*, este
//! responde *"o que acontece quando se clica"*. Juntos são a costura inteira, e é por isso que a
//! lei da W34 (`field3d_reach_tests`) mede os **dois** de uma vez — publicar e fazer têm de ser a
//! mesma pergunta.
//!
//! ⚠️ A drenagem acontece **antes** de `publish_snapshot`, e a ordem é load-bearing: se o retrato
//! saísse primeiro, o painel pintaria o valor antigo por um quadro e o controle daria um salto para
//! trás debaixo do dedo.

use super::*;

/// Aplica todas as intenções pendentes do painel ao mundo.
///
/// Devolve **o que passou a estar selecionado**: `created` é o nó que um gesto fez nascer (entra
/// com o que a importação de escultura já tenha criado neste quadro), e `cleared` diz que a seleção
/// aponta para algo que deixou de existir.
///
/// ⚠️ **Ela deixou de receber a RAIZ e a CÂMERA na W100**, e a perda é a notícia: os dois só
/// serviam para *nascer uma forma*, que passou a entrar por [`add_shape`] — o painel já não escolhe
/// forma nenhuma, ele abre a paleta. *Um parâmetro que ninguém lê é uma pergunta que o leitor
/// seguinte tenta responder.*
pub(super) fn apply(
    world: &mut bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
    mut created: Option<u64>,
) -> (Option<u64>, bool) {
    let mut cleared = false;
    // As edições do painel escrevem no COMPONENTE do nó, que é a peça de verdade.
    for intent in ph2d_panel_model3d::drain_intents() {
        match intent {
            // ⭐ O verbo do gizmo é estado de VISTA: ele não entra no mundo, entra no smoke.
            ph2d_panel_model3d::ModelIntent::SetGizmoMode { slot } => {
                // ⛔⛔ **Era `Mode::ALL.get(slot)`, e o painel publica a lista OFERECIDA.** Enquanto
                // a oferta era `Mode::ALL` inteiro as posições casavam por construção; desde que um
                // verbo pode não ser oferecido (`docs/Render3d/05` §28) elas só casariam por
                // ACIDENTE. *Duas listas para a mesma pergunta divergem no dia em que uma delas
                // encolhe* — e aqui ninguém acusaria: o clique poria o verbo errado, em silêncio.
                if let Some(mode) = crate::scene::offered_verbs(world, selection)
                    .get(slot)
                    .copied()
                {
                    with_smoke(|s| {
                        s.gizmo_mode = mode;
                        // Trocar de verbo com uma alça agarrada deixaria um arrasto órfão.
                        s.drag = None;
                        s.gizmo_hot = None;
                    });
                }
            }
            // ⭐⭐⭐ **CRIAR abre a PALETA** (W100) — e ela é a mesma do `Ctrl+K` e do `+` do
            // Inspector.
            //
            // ⚠️ **Só ANOTA**, como a exportação e as esculturas: a paleta vive no `HeroScreen`, e
            // esta função recebe o **mundo**. Mesma divisão, mesma razão — e o pick volta noutro
            // quadro, por [`add_shape`], que é a porta única de nascer uma forma.
            ph2d_panel_model3d::ModelIntent::OpenShapes => {
                crate::smoke::ask_shape_palette();
            }
            // ⭐⭐⭐ **ACENDE-SE UMA LUZ** (ordem do dono, 14/09).
            //
            // ⚠️ **Ela nasce onde a câmera OLHA e não na origem**, e à distância e na direcção que o
            // rig que ela substitui tinha — ver [`crate::lights::opening_place`]. *Uma lâmpada que
            // nasce fora do enquadramento é uma lâmpada que o artista tem de ir procurar antes de a
            // poder mover.*
            //
            // ⛔ **E ela não consulta o `where_to_add`:** uma luz não tem pai. Ela nasce RAIZ, ao
            // lado da peça — é onde uma luz vive em todo aplicativo 3D, e é o que faz apagar a peça
            // não apagar a luz.
            ph2d_panel_model3d::ModelIntent::AddLight => {
                let cam = crate::smoke::with_smoke(|s| s.vp().cam).unwrap_or_default();
                // ⭐ **A MESMA lei da primeira luz** — uma lâmpada acrescentada depois é tão útil
                // como a que a cena trouxe. *Duas leis dariam duas lâmpadas diferentes com o mesmo
                // nome.*
                let (onde, lampada) = crate::lights::opening_light(&cam);
                let e = ph2d_field_ecs::add_light(world, onde, lampada);
                // ⭐ A luz nova fica ESCOLHIDA — é o que põe o gizmo em cima dela e abre a secção
                // dela no painel, sem ninguém a ter de a achar na Hierarquia. É a mesma lei da forma
                // que nasce da paleta.
                created = Some(e.to_bits());
            }
            // ⭐ **As ações sobre o objeto escolhido** — e o `slot` resolve-se em **chave**.
            //
            // ⚠️ **Nunca por número.** A fileira deixou de ser fixa na W57 (largar e ligar só
            // aparecem às vezes), então o índice de um verbo passou a depender do que foi
            // publicado. Casar `0`/`1` aqui faria um botão executar o verbo do vizinho **sem erro
            // nenhum** — a mesma lista, uma porta só (`panel::acts_for`).
            ph2d_panel_model3d::ModelIntent::Act { slot } => {
                if let Some(&one) = selection.first() {
                    match super::acts::acts_for(world, selection).get(slot).copied() {
                        Some(super::acts::ACT_DUPLICATE) => created = duplicate_node(world, one),
                        // ⚠️ O que foi apagado não pode continuar selecionado: o gizmo ficaria
                        // aceso sobre uma entidade que já não existe.
                        Some(super::acts::ACT_DELETE) if ph2d_field_ecs::remove(world, one) => {
                            cleared = true;
                        }
                        // ⭐ **ISOLAR** (W38) — estado de VISTA, não do documento: não muda o
                        // mundo, não entra no undo, e por isso não mexe em `created`/`cleared`.
                        Some(super::acts::ACT_ISOLATE) => {
                            let on = crate::smoke::toggle_isolate(Some(one.to_bits()));
                            crate::notice::say(if on {
                                ph2d_i18n::tr(
                                    "app.field3d.scene_intents.isolated_showing_only_this_object",
                                )
                                .into()
                            } else {
                                ph2d_i18n::tr("app.field3d.scene_intents.isolation_off_the_whole_part_is_back").into()
                            });
                        }
                        // ⭐⭐ **LARGAR o desenho** (W57) — a forma FICA com a última que teve.
                        //
                        // ⚠️ **Tirar o componente é tudo o que é preciso**, e é o que o torna
                        // desfazível de graça: o `FieldProfileSource` viaja no retrato do mundo, e
                        // o undo regista por DIFF. Uma cópia da geometria «para não perder» seria
                        // um segundo dono da forma.
                        Some(super::acts::ACT_UNLINK) => {
                            world
                                .entity_mut(one)
                                .remove::<ph2d_field_ecs::FieldProfileSource>();
                            crate::notice::say(
                                ph2d_i18n::tr("app.field3d.scene_intents.unlinked_this_shape_no_longer_follows_the_drawing").into(),
                            );
                        }
                        // ⭐⭐ **LIGAR ao contorno escolhido** (W57).
                        //
                        // ⚠️ **A resolução recomeça no default de propósito.** Herdar o nível do
                        // vínculo antigo faria um desenho novo nascer com a finura de outro, e o
                        // número que o artista vê no painel deixaria de dizer o que ele escolheu
                        // para aquele desenho.
                        Some(super::acts::ACT_LINK) => {
                            if let Some(path) = crate::smoke::profile_pick() {
                                world
                                    .entity_mut(one)
                                    .insert(ph2d_field_ecs::FieldProfileSource {
                                        path,
                                        level: ph2d_field::DEFAULT_PROFILE_RESOLUTION,
                                    });
                                crate::notice::say(
                                    ph2d_i18n::tr("app.field3d.scene_intents.linked_this_shape_now_follows_the_selected_drawi").into(),
                                );
                            }
                        }
                        // ⭐⭐⭐ **RELIGAR a escultura que perdeu o arquivo** (W76) — aqui só se
                        // **pede**: escolher o arquivo é um diálogo, e um diálogo não corre com o
                        // mundo emprestado.
                        Some(super::acts::ACT_RELINK) => {
                            crate::smoke::ask_relink_sculpt(one.to_bits());
                        }
                        _ => {}
                    }
                }
            }
            // ⭐⭐ **UMA VISTA NOMEADA** (W47) — estado de VISTA: não muda o mundo, não entra no
            // undo. ⚠️ Ela põe a orientação **e enquadra** (W46): uma vista de frente que deixasse a
            // peça fora do quadro seria a mesma tela vazia que a W45/W46 acabaram de fechar.
            ph2d_panel_model3d::ModelIntent::SetView { slot } => {
                if let Some(v) = crate::views::Standard::ALL.get(slot).copied() {
                    crate::smoke::with_smoke(|s| {
                        crate::input::fly_to_view(s, v);
                        // A mão mandou: o prato não recomeça a girar por cima da vista escolhida.
                        s.vp_mut().manual = true;
                    });
                }
            }
            // ⭐ **A LENTE e o ENQUADRAR** — as duas portas que só existiam como tecla.
            ph2d_panel_model3d::ModelIntent::Camera { slot } => {
                crate::smoke::with_smoke(|s| {
                    if slot == super::panel::ORTHO_SLOT {
                        s.vp_mut().cam.lens = crate::input::law::other_lens(s.vp_mut().cam.lens);
                    } else if slot == super::panel::QUAD_SLOT {
                        // ⭐⭐⭐ **A DIVISÃO** (W90) — ver `smoke::toggle_split`.
                        crate::smoke::toggle_split(s);
                    } else if slot == super::panel::FRAME_SLOT {
                        let mut to = s.vp().cam;
                        crate::input::law::home(&mut to);
                        crate::input::frame_into(s, &mut to);
                        crate::smoke::fly_to(s, to);
                    }
                    s.vp_mut().manual = true;
                });
            }
            // ⭐⭐⭐ **O SOMBREAMENTO** (`docs/Render3d/05`) — estado de VISTA: não muda o mundo, não
            // entra no undo. ⚠️ Mas muda o QUADRO, e por isso as duas portas largam o pedido
            // guardado: um olhar novo sobre o traçado velho seria o congelador que o doc do
            // `requested` já pagou.
            ph2d_panel_model3d::ModelIntent::SetShading { slot } => {
                if let Some(shading) = crate::shading::Shading::ALL.get(slot).copied() {
                    crate::smoke::with_smoke(|s| s.set_shading(shading));
                }
            }
            ph2d_panel_model3d::ModelIntent::SetLook { slot } => {
                crate::smoke::with_smoke(|s| s.set_look(crate::shading::with_view(s.look, slot)));
            }
            ph2d_panel_model3d::ModelIntent::SetExposure { slot } => {
                crate::smoke::with_smoke(|s| {
                    s.set_look(crate::shading::with_exposure(s.look, slot));
                });
            }
            // ⭐ **Sair para um arquivo.** ⚠️ O pedido só é ANOTADO aqui: escrever um arquivo é
            // assunto do app (diálogo, toast) e esta função recebe o **mundo**. Ele atravessa pelo
            // mesmo caminho que o pedido de abrir o painel já usava.
            ph2d_panel_model3d::ModelIntent::Export { slot } => {
                if let Some(level) = crate::export::ExportLevel::ALL.get(slot).copied() {
                    crate::smoke::ask_export(level);
                }
            }
            // ⭐ **Ligar ou desligar um modificador** — a casca e o afastamento.
            //
            // ⚠️ **Tirar primeiro, e só acrescentar se não tirou**: o botão é um interruptor, e uma
            // ordem ao contrário acrescentaria um segundo e tiraria o primeiro no mesmo clique —
            // que da tela lê como *"não aconteceu nada"*.
            ph2d_panel_model3d::ModelIntent::ToggleMod { slot } => {
                if let (Some(&one), Some(kind)) = (
                    selection.first(),
                    ph2d_field::UnaryKind::ALL.get(slot).copied(),
                ) && !ph2d_field_ecs::remove_mod(world, one, kind)
                    && !ph2d_field_ecs::add_mod(world, one, kind)
                {
                    // ⚠️ **A porta recusou** (uma escultura, W25) — e uma recusa muda seria a metade
                    // errada da cura: o painel já não oferece a fileira, mas um clique que chegue
                    // aqui por outro caminho tem de dizer porquê em vez de não fazer nada.
                    crate::notice::say(crate::notice::explain(
                        &ph2d_field::FieldError::ModsOnSampled { node: 0 },
                    ));
                }
            }
            // ⭐ **Combinar** — trocar a operação de uma, ou embrulhar as escolhidas numa nova.
            ph2d_panel_model3d::ModelIntent::ApplyOp { slot } => {
                if let Some(op) = op_at(slot) {
                    match selection {
                        // ⭐ **Uma OPERAÇÃO escolhida sozinha troca de operação** — o gesto de sempre.
                        [one]
                            if matches!(
                                world.get::<FieldNode>(*one).map(|n| &n.shape),
                                Some(NodeShape::Combine(_))
                            ) =>
                        {
                            let _ = ph2d_field_ecs::set_op(world, *one, op);
                        }
                        // ⭐ **Uma FORMA escolhida sozinha vira um GRUPO** (W31) — e era isto que
                        // faltava: Enio, 2026-08-22, *"ainda não temos como criar novos grupos"*.
                        //
                        // ⚠️ O braço anterior tinha de ganhar a guarda: sem ela, um `set_op` numa
                        // folha era recusado em silêncio e o clique não fazia nada. *Um gesto que só
                        // funciona com dois selecionados não é um gesto de criar grupo.*
                        many => {
                            if let Some(group) = ph2d_field_ecs::wrap_in_op(world, many, op) {
                                created = Some(group.to_bits());
                                // ⭐ **E ALGUÉM DIZ QUE ELE NASCEU** (W38). A W31 fez o gesto e
                                // deixou-o mudo: a Hierarquia ganha uma linha nova, o objeto
                                // escolhido passa a estar um nível abaixo, e nada na tela explica
                                // porquê. ⚠️ Diz **quantos** entraram, que é o que distingue
                                // *"criei um grupo com esta forma"* de *"embrulhei as três"*.
                                crate::notice::say(ph2d_i18n::tr_with(
                                    "app.field3d.scene_intents.group_created_with_object_s_inside",
                                    &[("many", &(many.len()))],
                                ));
                            }
                        }
                    }
                }
            }
            // ⭐⭐⭐ **O VERBO DESTA FORMA** (W97) — com que operação ela dobra sobre as anteriores.
            //
            // ⚠️ **A MISTURA em vigor é lida ANTES e escrita junto** — ver [`verb_at`]. Uma forma
            // que herdava a subtração de um grupo com filete `0,12` e passasse a subtrair com
            // aresta viva mudaria de forma ao clique, sem ninguém ter tocado num raio.
            //
            // ⚠️ O sujeito é o **primeiro** da seleção, que é exactamente o que a fileira **nomeia**
            // no painel ([`verbs_for`]) — as duas metades da costura leem a mesma entidade, e é isso
            // que impede o clique de escrever noutra forma que não a nomeada.
            ph2d_panel_model3d::ModelIntent::SetVerb { slot } => {
                if let Some(&e) = selection.first()
                    && let Some(role) = ph2d_field_ecs::verb_role(world, e)
                    && let Some(current) = role.op()
                    && let Some(verb) = verb_at(slot, current.blend())
                {
                    let _ = ph2d_field_ecs::set_verb(world, e, verb);
                }
            }
            // ⭐⭐⭐ **O CARÁTER da mistura** (W99) — e o **número não se mexe**.
            //
            // ⚠️ Numa FORMA, isto **materializa o verbo**, como o raio de junção e pelo mesmo
            // motivo: escolher o carácter da própria junta *é* pronunciar-se. Sem isso, o chip
            // escreveria no grupo e mudaria as outras formas caladas com ele.
            ph2d_panel_model3d::ModelIntent::SetCharacter { slot } => {
                if let Some(&e) = selection.first()
                    && let Some(c) = ph2d_field::Character::ALL.get(slot).copied()
                {
                    let _ = ph2d_field_ecs::set_character(world, e, c);
                }
            }
            // O referencial dos eixos é estado de VISTA, como o verbo.
            ph2d_panel_model3d::ModelIntent::SetGizmoFrame { slot } => {
                if let Some(frame) = crate::gizmo::Frame::ALL.get(slot).copied() {
                    with_smoke(|s| s.gizmo_frame = frame);
                }
            }
            // ⭐⭐ **O modo do LAÇO** (W112), estado de VISTA pela mesma lei. ⚠️ Um `slot` fora dos
            // dois é ignorado — a mesma resposta que os irmãos dão, e a razão é a mesma: a família
            // de ids tem `MAX_MODES` posições registadas sempre.
            ph2d_panel_model3d::ModelIntent::SetLassoMode { slot } => {
                if slot < 2 {
                    with_smoke(|s| s.lasso_subtracts = slot == 1);
                }
            }
            // ⭐⭐⭐ **O ESTILO É DA CENA, e o sujeito sai da FAMÍLIA** (`docs/Render3d/03`, a `W8`).
            //
            // ⚠️⚠️ **O `entity` da linha NÃO é lido, e isso é a lei e não um esquecimento:** o estilo
            // não é de entidade nenhuma. O precedente é o material de um GRUPO, cuja linha já viaja
            // com o id do primeiro alvo enquanto quem decide o alcance é este dreno — *um índice sem
            // família é um sujeito por adivinhar, e a adivinha é silenciosa.*
            //
            // ⚠️ **Estes dois braços vêm ANTES dos do material**, e a ordem é load-bearing: os de
            // baixo casam por `Param::Material(_)` e o genérico casa por TUDO. Um braço de estilo
            // depois do genérico seria um controlo pintado, registado e **morto sob o dedo**.
            ph2d_panel_model3d::ModelIntent::SetParam {
                param: ph2d_field::Param::Style(slot),
                value,
                ..
            } => {
                crate::smoke::with_smoke(|s| {
                    s.set_style(crate::estilo::with_number(s.style, slot, value));
                });
            }
            ph2d_panel_model3d::ModelIntent::SetColor {
                anchor: ph2d_field::Param::Style(slot),
                srgb,
                ..
            } => {
                crate::smoke::with_smoke(|s| {
                    s.set_style(crate::estilo::with_colour(s.style, slot, srgb));
                });
            }
            // ⭐⭐⭐ **O BRILHO** (`docs/Render3d/12`) — a mesma forma do estilo, e pelas mesmas duas
            // razões: o `entity` não é lido (o sujeito é a cena) e o braço vem ANTES do genérico.
            //
            // ⭐ **E há braço de COR desde 19/09** — a TINTA do halo, que o nosso modelo tem e o do
            // Godot não tinha. ⚠️ Ele vem ANTES do genérico pela mesma razão que o do estilo.
            ph2d_panel_model3d::ModelIntent::SetColor {
                anchor: ph2d_field::Param::Bloom(slot),
                srgb,
                ..
            } => {
                crate::smoke::with_smoke(|s| {
                    s.set_bloom(crate::brilho_painel::with_colour(s.bloom, slot, srgb));
                });
            }
            ph2d_panel_model3d::ModelIntent::SetParam {
                param: ph2d_field::Param::Bloom(slot),
                value,
                ..
            } => {
                crate::smoke::with_smoke(|s| {
                    s.set_bloom(crate::brilho_painel::with_number(s.bloom, slot, value));
                });
            }
            // ⭐⭐⭐ **UMA COR** (Enio, 2026-09-14) — a travessia sRGB → linear, e três escritas.
            //
            // ⚠️ **As três correm no MESMO quadro, e é isso que as torna UM passo de undo**: o
            // registo é por **diff** uma vez por quadro (`App::post_frame_undo`), então o que
            // importa não é serem uma chamada, é não haver quadro entre elas. Uma cor escolhida em
            // três passos de desfazer seria exactamente o que o artista não espera.
            //
            // ⚠️ **A recusa de um canal não aborta os outros**, como em todo `set_param` deste
            // dreno: o retrato publicado logo abaixo devolve a amostra à cor que ficou.
            //
            // ⚠️ **`field` é a ÂNCORA e os canais são `field + k`** — ver
            // [`ph2d_panel_model3d::ModelIntent::SetColor`]. ⛔ Um `0` escrito aqui à mão faria a cor
            // da emissão aterrar na cor base: *a mesma escrita, o sujeito errado, e sem erro nenhum.*
            ph2d_panel_model3d::ModelIntent::SetColor {
                entity,
                anchor,
                srgb,
            } => {
                let cor = crate::materials::colour_from_srgb8(srgb);
                // ⚠️ **Os três canais saem da PORTA**, a mesma que o painel usa para os dobrar na
                // amostra — ver [`ph2d_field::Param::colour_channels`].
                if let Some(canais) = anchor.colour_channels() {
                    // ⭐ **A cor de uma LUZ é daquela luz**, e não se espalha pela selecção: o alcance do
                    // material existe porque *«um material é a única coisa que um artista atribui a
                    // muitos objectos de uma vez»* — uma lâmpada não é.
                    let alvos: Vec<bevy_ecs::entity::Entity> =
                        if matches!(anchor, ph2d_field::Param::Light(_)) {
                            vec![bevy_ecs::entity::Entity::from_bits(entity)]
                        } else {
                            crate::scene::panel::material_reach(world, selection, entity)
                        };
                    for alvo in alvos {
                        for (k, v) in cor.into_iter().enumerate() {
                            let _ = ph2d_field_ecs::set_param(world, alvo, canais[k], v);
                        }
                    }
                }
            }
            // ⭐⭐⭐ **UM NÚMERO DE MATERIAL ESPALHA-SE PELA SELECÇÃO**, como a cor — ver
            // [`crate::scene::panel::material_reach`].
            //
            // ⚠️ **E uma DIMENSÃO não**, no braço logo abaixo: largura, raio e posição são **daquela
            // forma**, e espalhá-los por uma selecção seria esmagar o desenho. *A distinção não é
            // arbitrária — um material é a única coisa que um artista atribui a muitos objectos de
            // uma vez.*
            ph2d_panel_model3d::ModelIntent::SetParam {
                entity,
                param: param @ ph2d_field::Param::Material(_),
                value,
            } => {
                for alvo in crate::scene::panel::material_reach(world, selection, entity) {
                    let _ = ph2d_field_ecs::set_param(world, alvo, param, value);
                }
            }
            ph2d_panel_model3d::ModelIntent::SetParam {
                entity,
                param,
                value,
            } => {
                // Uma recusa é informação, não erro: o nó diz que aquele número não cabe, e o
                // retrato publicado logo abaixo devolve o controle ao valor que ficou.
                let _ = ph2d_field_ecs::set_param(
                    world,
                    bevy_ecs::entity::Entity::from_bits(entity),
                    param,
                    value,
                );
            }
        }
    }
    (created, cleared)
}

/// ⭐⭐⭐ **A PORTA de nascer uma forma** (W100) — a paleta escolheu, e é aqui que a escolha vira
/// peça. Devolve o nó criado, quando ele nasce **neste** quadro.
///
/// # ⚠️ Por que é uma função, e não um braço do `match` acima
///
/// A escolha chega da **paleta**, que vive no `HeroScreen` e é drenada onde o mundo está
/// ([`super::ecs_bridge`]) — não é um pedido do painel, exatamente como a tecla do isolamento não
/// é. Deixá-la reentrar pela fila de intents obrigaria a shell a empurrar para uma fila que é do
/// painel; escrevê-la duas vezes seria a segunda porta. *Uma escolha, uma função.*
///
/// # ⚠️ Cada `Make` decide, e nenhum número decide
///
/// Três das cinco **só anotam**: cozer um contorno precisa da cena vetorial e abrir um arquivo
/// precisa de um diálogo, e nenhuma das duas coisas está aqui. É a mesma divisão da exportação.
/// ⛔ E é o [`Make`] que as separa, nunca a posição no catálogo — as constantes derivadas do fim da
/// lista (`SHAPES.len() - 4`…) morreram com a W100 porque a lista vai crescer.
pub(super) fn add_shape(
    world: &mut bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
    selection: &[bevy_ecs::entity::Entity],
    cam: ph2d_field_render::Orbit,
    slot: usize,
) -> Option<u64> {
    use crate::shapes::Make;
    let shape = crate::shapes::SHAPES.get(slot)?;
    // ⚠️ **A disponibilidade é reconferida AQUI**, e não só na paleta: entre abrir a paleta e
    // escolher um item o artista pode ter largado o contorno, e a paleta é um retrato do instante
    // em que abriu. *Uma affordance que envelheceu não pode virar um gesto que falha em silêncio.*
    let (live_sculpt, profile) = crate::smoke::palette_conditions();
    if !crate::shapes::available(shape, live_sculpt, profile) {
        crate::notice::say(ph2d_i18n::tr_with(
            "app.field3d.scene_intents.not_available_right_now",
            &[("shape", &(ph2d_i18n::tr(shape.key)))],
        ));
        return None;
    }
    match shape.make {
        Make::Formula(_) => {
            let prim = crate::shapes::shape_at(slot, new_shape_size(cam.half_extent))?;
            let parent = where_to_add(world, root, selection.first().map(|e| e.to_bits()));
            // ⭐ A forma nova fica SELECIONADA: é o que põe o gizmo em cima dela sem ninguém ter de
            // a procurar na Hierarquia.
            ph2d_field_ecs::add_leaf(world, parent, prim, cam.target)
                .ok()
                .map(|e| e.to_bits())
        }
        // ⭐⭐ **AS FORMAS DE PERFIL** (W53) — o desenho do editor vetorial vira peça.
        Make::Extrude => {
            crate::smoke::ask_profile_shape(crate::smoke::ProfileShape::Extrude);
            None
        }
        Make::Revolve => {
            crate::smoke::ask_profile_shape(crate::smoke::ProfileShape::Revolve);
            None
        }
        Make::Sculpt => {
            crate::smoke::ask_import();
            None
        }
        // ⭐ **A escultura da CENA** (W39) — o mesmo salto, sem o disco no meio.
        Make::SculptScene => {
            crate::smoke::ask_scene_sculpt();
            None
        }
    }
}
