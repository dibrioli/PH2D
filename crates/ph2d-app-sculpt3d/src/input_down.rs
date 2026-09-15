//! ⭐⭐ **QUEM TOMA O GESTO** — o pen-down da cena 3D.
//!
//! Irmão (`#[path]`) do [`super`] pelo tecto de LOC, e o corte é por
//! **responsabilidade**: este ficheiro é ARBITRAGEM — de quem é este clique? do
//! chrome, de uma costura, de um quadrante, do gizmo de navegação, de um arm, ou
//! do barro? — e o irmão é EXECUÇÃO: o que o gesto já tomado faz enquanto o dedo
//! anda e quando ele larga.
//!
//! ⚠️ **A ORDEM deste ficheiro é a lei**, e é por isso que ele é um só: cada
//! pergunta só é honesta depois de a anterior ter dito «não é minha». Espalhá-la
//! por dois sítios poria metade da arbitragem a correr depois da outra metade
//! ter já respondido.

use super::{Drag, Sculpt3dScene, Verb};

/// O botão apertou. Devolve `true` se a cena 3D tomou o gesto.
pub fn pointer_down(
    host: &mut impl ph2d_app_host::AppHost,
    scene: &mut Sculpt3dScene,
    button: winit::event::MouseButton,
) -> bool {
    let pos = host.pointer();
    // ⚠️ Um clique SOBRE A MOLDURA não é da cena. A pergunta era *"está sobre
    // um PAINEL?"*, e painel é só uma espécie de UI: a faixa do topo e o rail
    // não publicam `panel_rect`, então com o barro na tela a cena engolia o
    // clique em TODO pill do topo — inclusive no que existe para SAIR daqui
    // (Enio, 2026-08-09: *"a pill entra mas não sai do modo sculpt"*). A porta
    // nova cobre painéis E os fundos que a moldura pinta.
    //
    // O `Move` e o `Up` NÃO a fazem de propósito: um arrasto em curso continua
    // sendo do gesto que o abriu, mesmo que o cursor passeie por cima de um
    // painel (a regra de captura que todo gizmo deste shell segue).
    if host.pointer_over_chrome(pos.0, pos.1) {
        return false;
    }
    let mods = host.mods();
    let (ctrl, shift) = (mods.control, mods.shift);
    // ⚠️ **Com o barro fora da tela, o ponteiro NÃO é da cena.** Sem esta
    // pergunta a doação seria inalcançável pelo motivo mais bobo possível:
    // o artista troca para o modo LUZ, vai pintar, e cada clique orbita um
    // modelo invisível. É a mesma classe do clique-sobre-painel que o smoke
    // da W2 pegou — quem não está na tela não recebe o gesto.
    if !scene.shows_clay() {
        return false;
    }
    // ⚠️⚠️ **UMA CENA VAZIA NÃO RECEBE GESTO** — e sem esta pergunta o app CRASHA.
    //
    // Enio, 2026-08-22, a esculpir depois de apagar a única peça:
    // ```text
    // [sculpt3d] APAGOU: sobram 0 pecas -- Ctrl+Z a devolve INTEIRA
    // PH2D PANIC ... sculpt3d_input.rs:173 "index out of bounds: the len is 0 but the index is 0"
    // ```
    //
    // ⭐ **A cena vazia é um estado LEGÍTIMO** — o próprio `delete_active` a produz e promete o
    // Ctrl+Z de volta. O que faltava era a outra metade: os caminhos de gesto indexam
    // `objects[active]` **direto**, e com a lista vazia o `active` (que o delete prende em 0)
    // aponta para nada. *Um estado que o módulo declara legal e um caminho que o supõe
    // impossível é um pânico à espera do primeiro clique.*
    //
    // ⛔ **A cura completa é MAIOR do que este sítio, e é da `line/sculpt3d`:** medido
    // 2026-08-22, há **42** indexações `objects[…]` sem guarda em 9 arquivos deste módulo
    // (`filter`, `dyntopo`, `pull`, `transform`, `input`, `space`, `objects`, `import`), e a
    // porta segura que elas deviam usar (`obj()` / `obj_mut()`) **já existe**. A `line/3DModeling`
    // fecha aqui a porta que o artista bateu e nomeia o resto no handoff — reescrever 42 sítios
    // de um módulo alheio não é dela.
    //
    // ⚠️ E a recusa é **reportada**, que é a lei que este módulo já segue no `Delete`: um gesto
    // que não faz nada e não diz nada é indistinguível de um app partido.
    if scene.objects.is_empty() {
        eprintln!("[sculpt3d] a cena esta' VAZIA -- nao ha' o que esculpir (Ctrl+Z devolve)");
        return false;
    }
    // ⭐⭐⭐ **UM MENU ABERTO GANHA DE TUDO, E ATÉ DA COSTURA** (2026-09-08).
    //
    // ⛔⛔ **A precedência é do vizinho, e ele já a pagou:** o cabeçalho do
    // quadrante de baixo-direita nasce encostado ao cruzamento das costuras,
    // então o menu que ele abre cai **por cima da banda de agarrar o
    // divisor** — e metade das linhas dele seria inalcançável, com o ponteiro
    // a virar seta de redimensionar por cima de um menu. *Uma precedência
    // escrita por analogia («a costura ganha de tudo») deixa de valer quando
    // nasce algo que é modal.*
    //
    // ⚠️ **Ele consome o clique caia ele onde cair** — dentro escolhe, fora
    // fecha. Deixar o de fora passar orbitaria a peça no mesmo gesto em que
    // o artista só queria desistir do menu.
    if button == winit::event::MouseButton::Left && scene.view_menu_open().is_some() {
        scene.view_menu_click(pos.0, pos.1);
        scene.last = pos;
        return true;
    }
    // ⭐⭐⭐ **A COSTURA DA DIVISÃO VEM ANTES DE TUDO** (2026-09-08) — ela é
    // uma linha de chrome desenhada por cima das quatro vistas, e um clique
    // sobre ela nunca é da peça.
    if button == winit::event::MouseButton::Left && scene.seam_grab(pos.0, pos.1) {
        scene.last = pos;
        return true;
    }
    // ⭐⭐ **O CHIP DO CABEÇALHO ABRE O MENU DAQUELA VISTA** — depois da
    // costura, porque ele vive **dentro** de um viewport e ela vive entre
    // eles.
    //
    // ⚠️ **Ele acerta o ACTIVO antes de abrir**, e é isso que faz a escolha
    // do menu cair no quadrante certo: o `aim_view` aponta a câmera do
    // activo, e este é o único caminho que abre o menu.
    if button == winit::event::MouseButton::Left
        && let Some(i) = scene.chip_at(pos.0, pos.1)
    {
        scene.set_active_vp(i);
        scene.open_view_menu(i);
        scene.last = pos;
        return true;
    }
    // ⭐⭐⭐ **DE QUEM É ESTE CLIQUE** (2026-09-08).
    //
    // ⚠️⚠️ **E o `None` é uma RECUSA, não um caso a ignorar:** desde que a
    // peça passou a ser desenhada na ÁREA (e não na janela), um ponto fora
    // dela não tem barro por baixo — antes desta wave a cena engolia cliques
    // sobre as réguas e sobre a faixa da esquerda, porque a malha estava lá
    // desenhada por baixo do chrome.
    let Some(vp) = scene.vp_at(pos.0, pos.1) else {
        return false;
    };
    // ⚠️ **Tocar num quadrante torna-o ACTIVO**, e é o que faz o gesto
    // seguinte (o traço, o filtro, o transform) correr na vista em que a mão
    // está. A troca guarda a câmera do que sai — ver `set_active_vp`.
    scene.set_active_vp(vp);
    // ⭐⭐⭐ **O GIZMO DE NAVEGAÇÃO VEM PRIMEIRO** (2026-09-08) — ele está POR
    // CIMA da peça, e um clique tem de ser de quem está por cima.
    //
    // ⚠️ **Antes da guarda de cena vazia**, e é deliberado: girar a câmera
    // não precisa de barro nenhum, e o widget existe exactamente para dizer
    // de que lado se está a olhar. *A recusa que protege o `objects[active]`
    // não tem nada a ver com navegar.*
    //
    // ⚠️ **Só o esquerdo**: o direito já é a órbita livre e o meio o pan, e
    // um widget que engolisse os três roubaria dois gestos que a peça
    // inteira oferece.
    if button == winit::event::MouseButton::Left {
        let (px, py) = pos;
        if scene.nav_pointer_down(px, py) {
            scene.last = pos;
            return true;
        }
    }
    match button {
        winit::event::MouseButton::Left => {
            // ⚠️ **Com o transform ARMADO o esquerdo transforma.** Não há
            // fallback para órbita aqui, e é deliberado: o arm é um estado
            // que o painel MOSTRA, e um botão que às vezes transforma e às
            // vezes gira a câmera — conforme o que estava sob o cursor —
            // seria o mesmo gesto significando duas coisas. A órbita
            // continua inteira no botão direito.
            // ⚠️ **Com o FILTRO armado o esquerdo filtra**, pelo mesmo
            // argumento do transform logo abaixo — e os dois nunca estão
            // armados juntos (as portas de armar se excluem). A ordem aqui
            // não escolhe um vencedor: ela é a rede que torna a exclusão
            // observável se algum dia falhar.
            if scene.filter_arm() {
                // Mirar antes de começar, pelo motivo dos dois vizinhos: o
                // `begin_filter` congela a foto da malha ATIVA.
                scene.aim(pos.0, pos.1);
                if scene.begin_filter(pos.0, pos.1) {
                    scene.drag = Some(Drag::Filter);
                } else {
                    // ⚠️ **Este ramo é uma CONTRADIÇÃO, não um caso.** A
                    // única recusa do `begin_filter` é o arm apagado, e ele
                    // está aceso três linhas acima — ao contrário do
                    // transform logo abaixo, cuja recusa (peça toda
                    // mascarada) é um estado real que o artista alcança.
                    //
                    // ⚠️ A mensagem que vivia aqui nomeava QUATRO verbos e
                    // mentia desde a W9b: o picker desacoplou a lei do
                    // verbo em mãos, então *"o verbo em mãos não filtra"*
                    // deixou de ser uma frase verdadeira sobre este app —
                    // e ela nunca teve como ser impressa para alguém a
                    // desmentir. A rede de release fica (o gesto vira
                    // órbita em vez de um botão morto); quem grita é o
                    // debug.
                    debug_assert!(
                        false,
                        "o begin_filter recusou com o arm ACESO: a exclusão dos dois arms \
                         deixou de valer, ou ele ganhou uma segunda recusa sem chamador"
                    );
                    scene.drag = Some(Drag::Orbit);
                }
                scene.last = pos;
                return true;
            }
            if scene.transform_arm().is_some() {
                // ⭐⭐⭐ **A ALÇA É AGARRADA ANTES DE A SESSÃO COMEÇAR**
                // (2026-09-08), e a ordem é a lei: a projecção das alças sai
                // do pivô da malha ATUAL, e agarrar depois de o
                // `begin_transform` congelar a foto perguntaria a uma peça e
                // responderia sobre outra. ⚠️ `false` NÃO é recusa — sem alça
                // o transform corre livre, que é o gesto modal de sempre.
                let na_alca = scene.gizmo_grab(pos.0, pos.1);
                // ⚠️ **MIRAR VEM ANTES DE COMEÇAR**, a mesma ordem (e o
                // mesmo motivo) do traço logo abaixo: o `begin_transform`
                // congela a foto da malha ATIVA, e mirar depois faria a
                // sessão descrever a peça anterior.
                let no_barro = scene.aim(pos.0, pos.1);
                // ⛔⛔⛔ **REPORT DO ENIO, 2026-09-08:** *«com as ferramentas
                // de transformação ativadas anulo a rot do canvas. isso não
                // pode acontecer.»*
                //
                // ⚠️ **Ele está certo, e o defeito é PRÉ-EXISTENTE — a wave do
                // gizmo só o tornou visível.** Este braço tomava o botão
                // esquerdo SEM PERGUNTAR se o raio acertou alguma coisa,
                // então com o transform armado o gesto mais comum do mundo —
                // *arrastar no vazio para girar a peça* — deixava de existir.
                //
                // ⚠️⚠️ **O `aim` já declarava esta lei no próprio doc**:
                // *«`false` se o raio não achou nada (e aí o botão vira
                // órbita, **como em todo gesto**)»*. Era «todo gesto» menos
                // este. *Uma porta que documenta a regra e um chamador que
                // não a honra é a forma mais barata de um defeito ficar anos
                // à vista de toda a gente.*
                //
                // ⚠️ **A alça vem PRIMEIRO na condição, e é o que torna o
                // gizmo alcançável fora da peça:** a ponta de uma seta
                // espeta-se no vazio, e ali não há barro nenhum a acertar.
                //
                // ⛔ **A cerca que a nota deste braço levantava dissolveu-se:**
                // ela dizia que um botão que *«às vezes transforma e às vezes
                // gira a câmera — conforme o que estava sob o cursor»* seria
                // o mesmo gesto com dois sentidos. Isso valia enquanto não
                // houvesse **nada na tela** a dizer o que está sob o cursor;
                // com as alças desenhadas, o que era ambiguidade passou a ser
                // uma coisa que se vê. ⇒ §0.0: *quem move o número que
                // tornava a nota verdadeira tem de reconferir a nota.*
                if !na_alca && !no_barro {
                    scene.drag = Some(Drag::Orbit);
                    scene.last = pos;
                    return true;
                }
                if scene.begin_transform(pos.0, pos.1) {
                    scene.drag = Some(Drag::Transform);
                } else {
                    // ⚠️ A recusa é REPORTADA: uma malha inteiramente
                    // mascarada não tem o que mover, e um gesto que não faz
                    // nada e não diz nada é indistinguível de um botão que
                    // não chegou.
                    eprintln!(
                        "[sculpt3d] transform: a peca esta' toda PROTEGIDA -- nao ha' o que mover (I inverte a mascara)"
                    );
                    scene.drag = Some(Drag::Orbit);
                }
                scene.last = pos;
                return true;
            }
            // ⚠️ Os modificadores são lidos UMA vez, no pen-down, e valem o
            // traço inteiro. Soltar o Shift no meio de uma pincelada faria
            // metade dela ser outra ferramenta — e nenhum app de escultura
            // faz isso, porque a lei do traço congela um `pre` só.
            scene.brush.invert = ctrl;
            let verb = scene.brush.verb;
            if shift {
                scene.brush.verb = Verb::Smooth;
            }
            // ⚠️ **MIRAR VEM ANTES DE COMEÇAR**, e a ordem é a wave inteira:
            // o `begin` dimensiona os planos por-vértice na malha ATIVA, e
            // se a peça sob o cursor for outra o traço passa a escrever
            // índices de uma malha noutra. Com a peça nova maior que a
            // velha, isso é um pânico no primeiro dab.
            scene.aim(pos.0, pos.1);
            scene.stroke.begin(scene.objects[scene.active].stack.mesh());
            // A queixa do passe de topologia é UMA POR TRAÇO — ver
            // [`super::cena::Sculpt3dScene::dyn_queixa_dita`].
            scene.dyn_queixa_dita = false;
            // ⭐⭐ **A LISTA DE COLISORES é montada AQUI, uma vez, na pose
            // deste instante** (espec §5.6 cláusula 1) — as OUTRAS peças da
            // cena. ⇒ *uma peça que se mova durante o traço não se move para
            // o pano, e uma que apareça a meio não entra.*
            //
            // ⚠️ **Só com a opção ligada NESTE instante**, e a fotografia é
            // pelo mesmo motivo: a simulação nasce e morre com o traço (§6.3).
            // ⛔ E a pose de cada peça entra na cópia — a lei recebe posições
            // em espaço do MUNDO, e o `Multires::mesh` está em espaço local.
            scene.stroke.pecas_da_cena.clear();
            // ⭐⭐⭐ **A pergunta é ao PINCEL, não a um verbo** — ver
            // [`ph2d_sculpt3d::Brush::precisa_das_pecas_da_cena`]. Os dois
            // consumidores (a colisão do tecido e a PROJECÇÃO) têm predicados
            // diferentes, e um `if` de duas pernas escrito aqui faria o
            // terceiro herdar a perna errada em silêncio.
            // ⚠️ **E o «OUTRAS» sai da porta e não de um `i != activo` aqui** —
            // ver [`super::Sculpt3dScene::alvos_visiveis`]: a espec §6.1 conta
            // as que **não estão escondidas**, e este laço estava escrito duas
            // vezes na crate.
            if scene.brush.precisa_das_pecas_da_cena() {
                let alvos: Vec<(ph2d_mesh::Mesh, ph2d_mesh::Pose)> = scene
                    .alvos_visiveis()
                    .map(|i| {
                        let o = &scene.objects[i];
                        (o.stack.mesh().clone(), o.pose)
                    })
                    .collect();
                scene.stroke.pecas_da_cena = alvos;
            }
            // ⚠️ **E a pose do ACTIVO com elas, no mesmo instante** — ela é a
            // régua em que os candidatos da projecção competem
            // (`ph2d_sculpt3d::projectar`). Fotografá-la noutro sítio abriria a
            // hipótese de a lista e a régua descreverem instantes diferentes.
            scene.stroke.pose_activa = scene.objects[scene.active].pose;
            // ⭐⭐⭐ **E A SUPERFÍCIE DO PRÓPRIO ACTIVO, pela MESMA lei e no
            // mesmo instante** — ver
            // [`super::Sculpt3dScene::fotografa_a_superficie_do_pen_down`], onde
            // o report do dono que a pagou está medido.
            scene.fotografa_a_superficie_do_pen_down();
            // ⚠️ **Depois do `aim`**: a foto é da peça que este traço vai
            // esculpir, e antes do `aim` ela seria a da peça anterior.
            scene.open_dyntopo_stroke();
            // ⭐ **E a superfície de REFERÊNCIA**, pela mesma razão e no mesmo
            // sítio: ela é função do nível de baixo, que o traço não toca.
            scene.open_reference_stroke();
            // ⛔⛔ **E O GESTO DIZ PORQUE NAO VAI FAZER NADA** — a porta única
            // das recusas previsíveis ([`crate::recusa`]). *Um pincel que não
            // faz nada e não diz porquê é indistinguível de um pincel partido*,
            // e o artista tira a conclusão cara.
            scene.diz_a_recusa_do_pen_down();
            // A âncora do espaçamento nasce no pen-down: o 1º dab é o que
            // está sob o dedo, e o resíduo passa a contar a partir dele.
            scene.stroke_anchor = [pos.0, pos.1];
            scene.grab = None;
            // ⚠️ **E o pendente morre com o gesto anterior.** O pen-up
            // drena, então em regime ele já está vazio — mas um arrasto que
            // termine por outra porta (a cena fechada, o botão trocado)
            // deixaria um puxão órfão para carimbar dentro do traço
            // SEGUINTE, com a âncora nova. É a mesma razão do `twist`
            // logo abaixo.
            scene.pending_grab = None;
            // O ângulo varrido é do GESTO, então ele morre com o gesto
            // anterior — deixá-lo vivo faria o traço seguinte começar já
            // torcido, no lugar onde o anterior parou.
            scene.twist = None;
            // ⚠️ **Quem tem ÂNCORA não carimba no pen-down: ele PEGA.** O
            // primeiro toque escolhe o ponto e não move nada; o barro vem
            // quando o dedo anda. Vale para os TRÊS grips com âncora — o
            // Grab porque o puxão ainda é zero, o Snake Hook porque o
            // incremento ainda é zero, o Twist e o Local Scale porque o
            // ângulo e a fração ainda são zero —, e é por isso que a
            // pergunta é `anchors()` e não o nome de um verbo.
            let took = if scene.brush.verb.anchors() {
                scene.take_hold(pos.0, pos.1)
            } else {
                scene.sculpt_at(pos.0, pos.1)
            };
            if took {
                scene.drag = Some(Drag::Sculpt);
            } else {
                // Errou o modelo: o botão vira ÓRBITA. É o que o SculptGL
                // faz, e é o que impede o gesto mais comum do mundo —
                // arrastar no vazio — de não fazer nada.
                scene.brush.verb = verb;
                scene.drag = Some(Drag::Orbit);
            }
        }
        winit::event::MouseButton::Right => scene.drag = Some(Drag::Orbit),
        winit::event::MouseButton::Middle => scene.drag = Some(Drag::Pan),
        _ => return false,
    }
    scene.last = pos;
    true
}
