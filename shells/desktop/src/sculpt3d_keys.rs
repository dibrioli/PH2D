//! **O TECLADO** — que tecla escolhe o quê na cena 3D.
//!
//! Módulo FILHO de [`super`] (`#[path]`), irmão do [`super::input`]: lá *o que a mão faz com o
//! PONTEIRO* (o traço, a órbita, a roda), aqui *o que ela ESCOLHE com o teclado* (o verbo, o nível,
//! a luz, o espelho). São dois assuntos, e a tabela de teclas cresce uma linha por wave — foi ela
//! que levou o arquivo do ponteiro ao teto de LOC.
//!
//! ⚠️ A porta é a mesma de sempre: sem cena armada ela devolve `false` no primeiro `if`, e o
//! teclado do app segue para o `store` como se este módulo não existisse.

use super::{LIGHT_STEP_DEG, MaskOp, Merge, Primitive, RADIUS_STEP, RemeshRefusal, Verb};
use crate::app_state::App;

impl App {
    /// As teclas da cena 3D. Devolve `true` se consumiu.
    pub(crate) fn sculpt3d_key(
        &mut self,
        code: winit::keyboard::KeyCode,
        ctrl: bool,
        shift: bool,
    ) -> bool {
        use winit::keyboard::KeyCode as K;
        // **ASSAR A FORMA NUM SPRITE** (`docs/3D/02.2`) — o objetivo 2.
        //
        // ⚠️ A tecla **ARMA e sai**, em vez de fazer: o bake precisa do mundo, do renderizador, do
        // `AssetDb` e do mapa de atlas, e os quatro só existem dentro do laço de frame. É o mesmo
        // desenho da tela de smoke da doação, e é por isso que este braço vem ANTES do empréstimo
        // da cena — escrever no `self` com ela emprestada não compila.
        //
        // ⚠️ E ele vem antes do `ctrl` de propósito: sem o `!ctrl` um `Ctrl+Shift+B` armaria um
        // bake a caminho de um atalho que não é este.
        if shift && !ctrl && code == K::KeyB {
            if self.sculpt3d_scene_mut().is_none() {
                return false;
            }
            self.sculpt3d_bake_request = true;
            return true;
        }
        // **O PAINEL** (W12) — alternar a UI da cena 3D.
        //
        // ⚠️ **No acento grave, e a escolha é por ELIMINAÇÃO, não por gosto:**
        // com uma cena armada este teclado consome quase toda letra (os dez
        // dígitos são verbos, `G`/`H`/`T`/`S`/`A` são verbos, `C`/`I`/`B`/`N` são
        // máscara, `K`/`J`/`V`/`O`/`P`/`U` são topologia, `X`/`Y`/`Z` o espelho,
        // `Q`/`E`/`R`/`F` a luz, `D` a doação), e o que sobra livre no app inteiro
        // é a crase — que é também onde consoles e sidebars costumam morar.
        //
        // ⚠️ E ela vive AQUI e não no teclado global: sem cena 3D não há painel a
        // alternar, e uma tecla global seria um atalho morto em todo documento
        // 2D. Antes do empréstimo da cena pelo motivo do `Shift+B` acima —
        // escrever no `self` com ela emprestada não compila.
        if code == K::Backquote && !ctrl && !shift {
            if self.sculpt3d_scene_mut().is_none() {
                return false;
            }
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                let on = hero.is_panel_visible("sculpt3d");
                hero.panel_visibility.insert("sculpt3d", !on);
                eprintln!(
                    "[sculpt3d] painel: {}",
                    if on { "FECHADO" } else { "ABERTO" }
                );
            }
            return true;
        }
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        if ctrl {
            if code != K::KeyZ {
                return false;
            }
            // ⚠️ **O `shift` é o que separa desfazer de refazer, e enquanto ele
            // não chegava aqui o atalho de REFAZER desfazia mais um passo** —
            // a forma de *"o redo não funciona"* que **destrói** trabalho em vez
            // de não fazer nada. Ele é o mesmo par do resto do app (Ctrl+Z /
            // Ctrl+Shift+Z): um terceiro atalho só para esta cena seria uma
            // segunda gramática a aprender.
            return if shift {
                scene.redo_stroke()
            } else {
                scene.undo_stroke()
            };
        }
        // Os dez primeiros verbos por número; o Mask fica no `M` porque ele não
        // é uma escultura a mais, é o canal que todos os outros respeitam.
        const BY_NUMBER: [Verb; 10] = [
            Verb::Draw,
            Verb::Inflate,
            Verb::Smooth,
            Verb::Sharpen,
            Verb::Flatten,
            Verb::Fill,
            Verb::Scrape,
            Verb::Clay,
            Verb::Pinch,
            Verb::Crease,
        ];
        // ⚠️ O **Move** fica no `G` (de *grab*) e não num número: os dez números
        // já estão tomados, e `G` é a tecla que Blender e SculptGL usam para o
        // mesmo gesto — um artista a tenta antes de procurar.
        // ⚠️ O **Move** fica no `G` (de *grab*), o **Snake Hook** no `H` (de
        // *hook*), o **Twist** no `T` e o **Local Scale** no `S` (de *scale*):
        // os dez números já estão tomados, e o `G` é a tecla que Blender e
        // SculptGL usam para o mesmo gesto — um artista a tenta antes de
        // procurar. Os quatro saem pela MESMA porta que os numerados usam,
        // senão só eles perderiam o default de força.
        // ⚠️ **E o `A` é o MAGNIFY, que não tinha tecla nenhuma.** Onze verbos
        // de carimbo queriam dez dígitos, e ele foi o que transbordou — sem uma
        // linha dizendo isso: existia no enum, tinha alvo, era varrido por todo
        // gate, e o artista **não conseguia pegá-lo**. Não era cerca de
        // Chesterton, era capacidade. O `A` é de *amplify*, a mesma família da
        // palavra que o rótulo mostra; o `P`, que seria o mnemônico do PAR,
        // levaria a mão ao oposto do que ela procura.
        // ⚠️ **OS VERBOS DA LISTA vêm ANTES dos verbos do PINCEL**, e a ordem é
        // o que os torna alcançáveis: `Shift+1..4` compartilham o código com os
        // dígitos que escolhem ferramenta, e o `match` de baixo não olha o
        // `shift`. Perguntar depois seria a mesma classe do item de menu que
        // nasce morto porque outro consumidor pegou o evento primeiro.
        if shift {
            let primitive = match code {
                K::Digit1 => Some(Primitive::Sphere),
                K::Digit2 => Some(Primitive::Cube),
                K::Digit3 => Some(Primitive::Cylinder),
                K::Digit4 => Some(Primitive::Torus),
                _ => None,
            };
            if let Some(kind) = primitive {
                let i = scene.add_primitive(kind);
                eprintln!(
                    "[sculpt3d] + {} (peca {i}, a cena tem {}) -- Ctrl+Z a tira",
                    kind.label(),
                    scene.objects.len()
                );
                return true;
            }
            if code == K::KeyD {
                scene.duplicate_active();
                eprintln!(
                    "[sculpt3d] DUPLICOU: a cena tem {} pecas -- a copia nasce AO LADO na tela",
                    scene.objects.len()
                );
                return true;
            }
            // **FUNDIR.** ⚠️ No `Shift+J` porque *juntar* é o verbo, e porque o
            // `Shift` é onde os verbos da LISTA moram neste teclado (o `J` sozinho
            // é des-subdividir, que age numa peça). O log traz o número dos TRÊS
            // desfechos: a fusão não muda a silhueta da cena — as peças ficam
            // onde estavam —, então sem a contagem o artista vê a mesma imagem e
            // não tem como saber se a tecla fez alguma coisa.
            if code == K::KeyJ {
                match scene.merge_visible() {
                    Merge::Done {
                        pieces,
                        verts,
                        faces,
                    } => eprintln!(
                        "[sculpt3d] FUNDIDAS {pieces} pecas numa so' -- {verts} vertices / {faces} faces \
                         (elas nao ficam SOLDADAS: use V para reconstruir a casca) -- Ctrl+Z as separa"
                    ),
                    Merge::Nothing => eprintln!(
                        "[sculpt3d] nao ha' o que fundir: e' preciso mais de UMA peca a' vista \
                         (Shift+I devolve a cena inteira)"
                    ),
                    Merge::Stack => eprintln!(
                        "[sculpt3d] nao' funde com a pilha montada: a fusao troca a BASE, e todo nivel \
                         acima e' subdivisao dela -- reverta os niveis antes"
                    ),
                }
                return true;
            }
            // **ISOLAR.** ⚠️ A resposta visual é a cena SUMIR menos uma peça — é
            // o *local view* do Blender, e é por isso que o log diz o que voltou
            // ou o que ficou: uma tela que perde quatro objetos sem uma linha
            // explicando é indistinguível de um crash de render.
            // **A CAVIDADE** — o canal que faz a escultura ser LIDA
            // (`docs/3D/05.1` §4, W10.1).
            //
            // ⚠️ **`Shift+C` e não `C`, e a única coisa que isto tira é um alias
            // acidental:** `C` sozinho limpa a máscara, e como o bloco do `shift`
            // cai adiante quando nada casa, hoje `Shift+C` também limpa. Nenhum
            // atalho DOCUMENTADO se move — e o mnemônico do artista é **C**avity,
            // que é o único que ele vai tentar antes de procurar.
            if code == K::KeyC {
                let amount = scene.cycle_cavity();
                if amount == 0.0 {
                    eprintln!(
                        "[sculpt3d] cavidade: DESLIGADA -- o barro liso da W3, ao byte                          (Shift+C liga)"
                    );
                } else {
                    eprintln!(
                        "[sculpt3d] cavidade: {amount:.2} -- a fresta ESCURECE e a crista CLAREIA                          (Shift+C avanca; volta a zero depois de 1.00)"
                    );
                }
                return true;
            }
            // **O ESPALHAMENTO SUB-SUPERFICIAL** (`docs/3D/05.1` §2a, W10.5) —
            // o terceiro dos três canais que o Enio nomeou, ao lado do AO e da
            // cavidade.
            //
            // ⚠️ `Shift+S` pelo mnemônico do artista (**S**kin / **S**cattering),
            // e ele estava livre no bloco de shift.
            if code == K::KeyS {
                let amount = scene.cycle_sss();
                if amount == 0.0 {
                    eprintln!(
                        "[sculpt3d] espalhamento: DESLIGADO -- o barro de sempre, ao byte                          (Shift+S liga)"
                    );
                } else {
                    eprintln!(
                        "[sculpt3d] espalhamento: {amount:.2} -- a luz ATRAVESSA a borda da sombra,                          e o VERMELHO vai mais longe que o azul.                          O painel tem as duas pistas: 'Subsurface' e 'Scatter' (o alcance)."
                    );
                }
                return true;
            }
            if code == K::KeyI {
                let on = scene.toggle_isolate();
                if on {
                    eprintln!(
                        "[sculpt3d] ISOLADA: as outras {} pecas sairam da vista (Shift+I devolve) \
                         -- o pincel nao alcanca o que nao se ve",
                        scene.objects.len().saturating_sub(1)
                    );
                } else {
                    eprintln!(
                        "[sculpt3d] a cena inteira voltou: {} pecas a' vista",
                        scene.objects.len()
                    );
                }
                return true;
            }
        }
        if code == K::Delete {
            // ⚠️ A recusa é REPORTADA. Um Delete que não faz nada e não diz nada
            // é indistinguível de uma tecla que não chegou.
            if scene.delete_active() {
                eprintln!(
                    "[sculpt3d] APAGOU: sobram {} pecas -- Ctrl+Z a devolve INTEIRA",
                    scene.objects.len()
                );
            } else {
                // ⚠️ **Esta linha MENTIA.** Ela dizia *"a ULTIMA peca nao e
                // apagavel: a cena ficaria vazia"*, que era verdade até o Enio
                // derrubar a cerca no smoke (*"não consigo deletar todos os
                // objetos"*) — a última passou a ser apagável, e a única recusa
                // que sobrou é a cena **já** vazia. Uma mensagem que descreve a
                // regra anterior é pior que nenhuma: ela ensina ao artista um
                // limite que o produto não tem.
                eprintln!("[sculpt3d] a cena ja' esta' VAZIA: nao ha' peca a apagar");
            }
            return true;
        }
        let held = match code {
            K::KeyG => Some(Verb::Move),
            K::KeyH => Some(Verb::SnakeHook),
            K::KeyT => Some(Verb::Twist),
            K::KeyS => Some(Verb::LocalScale),
            K::KeyA => Some(Verb::Magnify),
            _ => None,
        };
        let verb = held.or(match code {
            K::Digit1 => Some(BY_NUMBER[0]),
            K::Digit2 => Some(BY_NUMBER[1]),
            K::Digit3 => Some(BY_NUMBER[2]),
            K::Digit4 => Some(BY_NUMBER[3]),
            K::Digit5 => Some(BY_NUMBER[4]),
            K::Digit6 => Some(BY_NUMBER[5]),
            K::Digit7 => Some(BY_NUMBER[6]),
            K::Digit8 => Some(BY_NUMBER[7]),
            K::Digit9 => Some(BY_NUMBER[8]),
            K::Digit0 => Some(BY_NUMBER[9]),
            K::KeyM => Some(Verb::Mask),
            _ => None,
        });
        // As QUATRO operações de máscara. ⚠️ Elas não são verbos: um verbo pinta
        // *onde a mão passou* e estas respondem a *o que já está pintado*, então
        // elas não podem entrar na lista de números (escolher uma não é pegar
        // uma ferramenta — é executar um gesto e acabar).
        let mask_op = match code {
            K::KeyC => Some(MaskOp::Clear),
            K::KeyI => Some(MaskOp::Invert),
            K::KeyB => Some(MaskOp::Blur),
            K::KeyN => Some(MaskOp::Sharpen),
            _ => None,
        };
        if let Some(op) = mask_op {
            scene.mask_op(op);
            eprintln!("[sculpt3d] mascara: {}", op.label());
            return true;
        }
        // **SUBDIVIDIR.** ⚠️ O log imprime a contagem NOVA porque o preço desta
        // tecla é exponencial e invisível: quatro faces onde havia uma, a cada
        // toque. Um botão que quadruplica a malha sem dizer quanto ela ficou é
        // um botão que o artista aperta uma vez a mais.
        if code == K::KeyK {
            if scene.subdivide() {
                eprintln!(
                    "[sculpt3d] subdividida: nivel {} de {} -- {} vertices / {} faces / {} triangulos",
                    scene.level(),
                    scene.level_count().saturating_sub(1),
                    scene.mesh().vert_count(),
                    scene.mesh().face_count(),
                    scene.mesh().triangle_count()
                );
            } else {
                eprintln!("[sculpt3d] so' do TOPO: suba (.) antes de subdividir");
            }
            return true;
        }
        // **TAPAR BURACO.** ⚠️ O log diz o número dos DOIS desfechos, e o segundo
        // é o que importa: uma beira que não fecha deixa a malha aberta ali, e
        // *deixar em silêncio* é como o artista conclui que a tecla não funciona.
        if code == K::KeyO {
            match scene.close_holes() {
                Some(r) if r.is_noop() => eprintln!(
                    "[sculpt3d] nenhum buraco: a malha ja' e' fechada ({} arestas de beira sobrando)",
                    r.left_open()
                ),
                Some(r) => eprintln!(
                    "[sculpt3d] tapados {} buraco(s) -- {} vertices / {} faces ({} arestas de beira sobrando)",
                    r.filled(),
                    scene.mesh().vert_count(),
                    scene.mesh().face_count(),
                    r.left_open()
                ),
                None => eprintln!(
                    "[sculpt3d] nao' tapa com a pilha montada: tapar muda a TOPOLOGIA, e todo nivel acima e' subdivisao dela -- tape ANTES de subdividir"
                ),
            }
            return true;
        }
        // **RECONSTRUIR (voxel remesh).** ⚠️ O log traz o ANTES e o DEPOIS na
        // mesma linha porque este botão não muda a forma — ele muda a MALHA, e
        // sem os dois números o artista vê a mesma escultura e não tem como
        // saber se a tecla fez alguma coisa. O número de células explica o
        // tempo: ele é o cubo da resolução (medido em `measure_remesh`).
        if code == K::KeyV {
            match scene.remesh(ph2d_sdf::DEFAULT_RESOLUTION) {
                Ok(r) => eprintln!(
                    "[sculpt3d] reconstruida: {} -> {} vertices / {} -> {} faces ({} celulas, {} buraco(s) tapado(s))",
                    r.verts.0, r.verts.1, r.faces.0, r.faces.1, r.cells, r.holes_filled
                ),
                Err(RemeshRefusal::MultiresStack) => eprintln!(
                    "[sculpt3d] nao' reconstroi com a pilha montada: o remesh troca a TOPOLOGIA, e todo nivel acima e' subdivisao dela -- reverta os niveis antes"
                ),
                Err(RemeshRefusal::EmptyScene) => {
                    eprintln!("[sculpt3d] nao' reconstroi: nao ha' peca na cena")
                }
                // ⚠️ A escultura CONTINUA na tela — é isto que a recusa compra.
                // Antes daqui o campo vazado devolvia uma malha vazia que o
                // shell instalava, e a peça sumia com log de sucesso.
                Err(RemeshRefusal::Engine(e)) => eprintln!(
                    "[sculpt3d] nao' reconstroi, e a escultura fica como esta': {e} -- tente outra resolucao"
                ),
            }
            return true;
        }
        // **DES-SUBDIVIDIR.** ⚠️ Fica no `J` porque é o vizinho do `K`, e o par
        // diz o que faz: `K` acrescenta um nível ACIMA, `J` reconstrói um
        // ABAIXO. O log diz a contagem NOVA pela razão inversa à do `K` — aqui a
        // malha que o artista vê não muda de forma nenhuma, e sem o número ele
        // não tem como saber se o gesto fez alguma coisa.
        if code == K::KeyJ {
            if scene.reverse_level() {
                let base = scene.obj().and_then(|o| o.stack.level_mesh(0));
                eprintln!(
                    "[sculpt3d] revertida: nivel {} de {} -- a base nova tem {} vertices / {} faces",
                    scene.level(),
                    scene.level_count().saturating_sub(1),
                    base.map_or(0, ph2d_mesh::Mesh::vert_count),
                    base.map_or(0, ph2d_mesh::Mesh::face_count)
                );
            } else {
                eprintln!(
                    "[sculpt3d] nao' reverte: esta malha nao e' uma subdivisao (ou desca ao nivel 0 antes)"
                );
            }
            return true;
        }
        // ⚠️ **Descer e subir NÃO é uma edição** — ver `change_level`. O log diz o
        // nível porque a malha de baixo se PARECE com a de cima alisada: sem o
        // número, o artista não sabe em qual está.
        // **A TOPOLOGIA DINÂMICA.** ⚠️ No `P` porque as letras da coisa estão
        // todas tomadas (`D` mostra o sprite, `T` torce) e o `P` é o vizinho
        // livre do cacho de topologia (`K` subdivide, `J` reverte, `V` remalha,
        // `O` fecha buraco). O log diz as DUAS consequências de ligar — o modo
        // e a triangulação —, porque triangular MUDA a malha e uma mudança
        // calada é a que o artista descobre no save.
        if code == K::KeyP {
            let (on, tris) = scene.toggle_dyntopo();
            if !on {
                eprintln!("[sculpt3d] topologia dinamica DESLIGADA");
            } else if scene.level_count() > 1 {
                eprintln!(
                    "[sculpt3d] topologia dinamica ARMADA -- mas a pilha de multires esta' montada                      e ela RECUSA: refinar a base deixaria cada nivel descrevendo outra malha                      (reverta com J)"
                );
            } else {
                let d = scene.detail_label();
                eprintln!(
                    "[sculpt3d] topologia dinamica LIGADA (detalhe {d}, U cicla) --                      {tris} faces trianguladas; o traco passa a ADENSAR onde a aresta e' longa                      demais e AFINAR onde ela e' curta demais, so' onde o pincel toca,                      e o Ctrl+Z dele devolve a malha inteira"
                );
            }
            return true;
        }
        // O DETALHE — três degraus com nome. Ver `DETAIL_STEPS`.
        if code == K::KeyU {
            let d = scene.cycle_detail();
            // ⚠️ **A contagem entra aqui porque este é o gesto que a MUDA nos
            // dois sentidos**: baixar o detalhe e voltar a passar o pincel faz o
            // colapso retirar o que o refino pôs, e sem o número de antes o
            // artista não tem contra o que comparar.
            eprintln!(
                "[sculpt3d] detalhe: {d} -- a aresta alvo e' uma fracao do PINCEL,                  entao pincel pequeno detalha fino ({} vertices / {} faces agora)",
                scene.mesh().vert_count(),
                scene.mesh().face_count()
            );
            return true;
        }
        if code == K::Comma || code == K::Period {
            let up = code == K::Period;
            if scene.change_level(up) {
                eprintln!(
                    "[sculpt3d] nivel {} de {} -- {} vertices",
                    scene.level(),
                    scene.level_count().saturating_sub(1),
                    scene.mesh().vert_count()
                );
            } else {
                eprintln!(
                    "[sculpt3d] ja' esta' no {}",
                    if up { "TOPO" } else { "nivel 0" }
                );
            }
            return true;
        }
        if let Some(v) = verb {
            // ⚠️ **Arma o default do verbo, e só se o artista ainda não mexeu.**
            // O precedente é o `arm_inflate_defaults` do Painter: um verbo pode
            // querer nascer diferente (a máscara nasce em força cheia, senão ela
            // protege pela metade e o barro se move por baixo), e nenhum verbo
            // pode APAGAR uma escolha deliberada. "Não mexeu" é a força ser
            // exatamente o default do verbo que está saindo.
            let old = scene.brush.verb;
            if (scene.brush.strength - old.default_strength()).abs() < 1e-6 {
                scene.brush.strength = v.default_strength();
            }
            scene.brush.verb = v;
            eprintln!(
                "[sculpt3d] verbo: {} (forca {:.2})",
                v.label(),
                scene.brush.strength
            );
            return true;
        }
        match code {
            K::BracketLeft | K::BracketRight => {
                let f = if code == K::BracketRight {
                    RADIUS_STEP
                } else {
                    1.0 / RADIUS_STEP
                };
                scene.radius_px *= f;
                // O clamp mora na porta, então o LOG mostra o número que o dab
                // vai de fato usar — imprimir o cru faria a tecla parecer viva
                // depois de o teto ter sido alcançado.
                scene.radius_px = scene.radius_px();
                eprintln!("[sculpt3d] raio: {:.0} px de tela", scene.radius_px);
                true
            }
            // **O INTERRUPTOR DA DOAÇÃO** — barro ⇄ luz ⇄ desligada.
            //
            // ⚠️ Gesto de SMOKE, como o `Q`/`E`/`R`/`F` da luz: a UI final é o
            // toggle *"iluminada pela forma abaixo"* na pilha de camadas
            // (`docs/3D/05.2`), e ele espera a escultura ser uma CAMADA do
            // documento. Enquanto ela é um viewport solto, uma tecla é a única
            // porta honesta — um checkbox num painel diria que a forma pertence
            // a um documento a que ela ainda não pertence.
            K::KeyD => {
                let label = scene.cycle_role();
                eprintln!("[sculpt3d] a forma agora e: {label}");
                true
            }
            K::KeyX | K::KeyY | K::KeyZ => {
                let axis = match code {
                    K::KeyX => &mut scene.symmetry.x,
                    K::KeyY => &mut scene.symmetry.y,
                    _ => &mut scene.symmetry.z,
                };
                *axis = !*axis;
                eprintln!("[sculpt3d] espelho: {:?}", scene.symmetry);
                true
            }
            // **A LUZ.** Girar a lâmpada principal em torno da cena e subi-la.
            //
            // ⚠️ Isto é o gesto do SMOKE, não a UI final: o card de Lighting do
            // Painter já é o lugar onde este rig se autora, e é ele que a M4
            // conecta. Um segundo card aqui seria a segunda porta para o mesmo
            // número. Estas teclas existem para o Enio poder ver a forma reacender
            // sem abrir um documento de pintura.
            K::KeyQ | K::KeyE => {
                let d = if code == K::KeyE {
                    LIGHT_STEP_DEG
                } else {
                    360 - LIGHT_STEP_DEG
                };
                let l = scene.rig.current_mut();
                l.angle_deg = (l.angle_deg + d) % 360;
                eprintln!(
                    "[sculpt3d] luz: azimute {}deg elevacao {}deg",
                    l.angle_deg, l.elev_deg
                );
                true
            }
            K::KeyR | K::KeyF => {
                let l = scene.rig.current_mut();
                let up = code == K::KeyR;
                // Clampado no piso do resolvedor, e não em 0: abaixo dele a
                // resposta plana vai a zero e o modelo relativo dividiria por ~0.
                l.elev_deg = if up {
                    (l.elev_deg + LIGHT_STEP_DEG).min(90)
                } else {
                    l.elev_deg
                        .saturating_sub(LIGHT_STEP_DEG)
                        .max(ph2d_light::MIN_ELEV_DEG)
                };
                eprintln!(
                    "[sculpt3d] luz: azimute {}deg elevacao {}deg",
                    l.angle_deg, l.elev_deg
                );
                true
            }
            _ => false,
        }
    }
}
