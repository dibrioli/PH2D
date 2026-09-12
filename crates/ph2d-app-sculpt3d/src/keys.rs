//! **O TECLADO** — que tecla escolhe o quê na cena 3D.
//!
//! Módulo FILHO de [`super`] (`#[path]`), irmão do [`super::input`]: lá *o que a mão faz com o
//! PONTEIRO* (o traço, a órbita, a roda), aqui *o que ela ESCOLHE com o teclado* (o verbo, o nível,
//! a luz, o espelho). São dois assuntos, e a tabela de teclas cresce uma linha por wave — foi ela
//! que levou o arquivo do ponteiro ao teto de LOC.
//!
//! ⚠️ A porta é a mesma de sempre: sem cena armada ela devolve `false` no primeiro `if`, e o
//! teclado do app segue para o `store` como se este módulo não existisse.

use super::{LIGHT_STEP_DEG, MaskOp, RADIUS_STEP, Verb};

/// ⭐ **Os verbos da LISTA** — o que o `Shift` arma, cortado daqui na integração de 2026-09-10,
/// quando este ficheiro ficou vermelho no teto de LOC por ACUMULAÇÃO de duas linhas.
///
/// ⚠️ **Ele é declarado AQUI e não no `mod.rs`, e a razão é uma conta:** o pai estava a `599`
/// de `600`, e pôr a declaração lá levava-o a `604` — *uma extracção que cura um ficheiro pode
/// estourar o do lado*. Declarar um filho no módulo de que ele foi cortado é também onde ele
/// pertence: quem o lê está a ler o teclado, não a raiz do módulo.
#[path = "keys_scene.rs"]
mod keys_scene;

/// ⭐⭐⭐ **A escultura reivindica este `Delete`?** — a lei pura.
///
/// ⚠️ **Declarado AQUI e não na raiz do módulo, pela mesma conta do irmão acima:** a
/// `line/quadextract` levou o `mod.rs` de `596` a `604` contra um teto de `600` só com estas
/// duas declarações, e o `Delete` é uma pergunta do TECLADO — quem a lê está a ler este ficheiro,
/// e o único chamador dela sempre foi este.
#[path = "keys_delete.rs"]
mod keys_delete;

/// As teclas da cena 3D. Devolve `true` se consumiu.
pub fn key(
    scene: &mut crate::Sculpt3dScene,
    req: &mut crate::Sculpt3dRequests,
    // ⚠️ **O painel chega por PARÂMETRO, e não por pedido nem por porta** — o `HeroScreen` é um
    // tipo da `ph2d-editor`, uma crate-módulo, logo ele atravessa a fronteira sem que nada novo
    // seja inventado. Ele e a cena vivem os dois no `AppGfx`, e é a shell — a dona dele — quem os
    // desmonta e empresta os dois campos de uma vez.
    hero: Option<&mut ph2d_editor::screens::hero::HeroScreen>,
    code: winit::keyboard::KeyCode,
    ctrl: bool,
    shift: bool,
    // ⚠️⚠️ **Os factos chegam COLHIDOS, e isso corrige um defeito latente:** o
    // `text_entry_focused` era perguntado **duas** vezes neste corpo (a guarda geral e o
    // `Delete`), e nada obrigava as duas leituras a concordar. Agora é uma leitura só, e a
    // igualdade é por construção.
    factos: &keys_delete::DeleteFacts,
    // `App::sculpt3d_keys_live()` — *o barro está na tela?*, a guarda da camada de baixo.
    keys_live: bool,
) -> bool {
    use winit::keyboard::KeyCode as K;
    // ⚠️ **UM CAMPO FOCADO É DONO DO TECLADO — e esta é a metade GERAL da cura.**
    //
    // Vale para as DUAS camadas abaixo, e por isso mora aqui em cima: enquanto ela
    // não existia, um chip numérico focado em QUALQUER painel do app não recebia um
    // dígito, porque os dez são verbos deste teclado e ele corre antes do store
    // (Enio, 2026-08-17: *"nem mesmo digitar um número no painel motion"*). É a
    // mesma pergunta que o `motion_keys_live`/`vector_keys_live` já compunham — a
    // porta é `text_entry_focused`, e ela não é de módulo nenhum.
    if factos.text_focused {
        return false;
    }
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
        req.bake_request = true;
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
        if let Some(hero) = hero {
            let on = hero.is_panel_visible("sculpt3d");
            hero.panel_visibility.insert("sculpt3d", !on);
            eprintln!(
                "[sculpt3d] painel: {}",
                if on { "FECHADO" } else { "ABERTO" }
            );
        }
        return true;
    }
    // **O INTERRUPTOR DA DOAÇÃO** — barro ⇄ luz ⇄ desligada.
    //
    // ⚠️ **Ele sobe para a camada do MÓDULO junto com o bake e o painel, e não é
    // arrumação:** ele é o único caminho até `FormRole::Off`, e o pill é binário
    // por desenho (de qualquer papel ele ENTRA no barro). Deixá-lo debaixo da
    // guarda do barro faria `Clay --D--> Light` e ali parava — `Off` viraria
    // inalcançável, uma regressão que a cura não pede.
    //
    // ⚠️ **O hoist preserva o comportamento ao bit:** o `ctrl` já saiu no braço
    // logo abaixo (só o `Ctrl+Z` é desta cena) e o `Shift+D` é capturado mais
    // acima (duplicar peça), então o braço que este código substitui só era
    // alcançado com `!ctrl && !shift` — que é exatamente o que a guarda diz.
    //
    // ⚠️ E ele custa ZERO ao Motion, medido: o `D` do grafo exige **cmd**
    // (`KEY_KEY_D if cmd`), então o `D` nu não colide com atalho nenhum de lá.
    if code == K::KeyD && !ctrl && !shift {
        let label = scene.cycle_role();
        eprintln!("[sculpt3d] a forma agora e: {label}");
        return true;
    }
    // ⚠️ **DAQUI PARA BAIXO O TECLADO É DA ESCULTURA — e ele exige o barro NA TELA.**
    //
    // A guarda pergunta o que o PONTEIRO desta cena já perguntava
    // (`FormRole::draws_clay`), e a assimetria entre as duas portas ERA o bug: o
    // clique cedia ao sair do modo, a tecla não. Ver [`App::sculpt3d_keys_live`]
    // para o mecanismo e os números.
    // ⭐⭐⭐ **O `Delete` decide-se numa PORTA própria, e RESOLVE-SE aqui** — ver
    // [`keys_delete`]. ⛔ Ele **não** pode cair no guarda geral abaixo: o report de
    // 2026-09-04, com a linha impressa como prova, foi *«a ferramenta Motion/Vector está EM
    // MÃOS e reivindica as teclas nuas»* — e o `Delete` não é uma tecla nua. A lei que
    // decide está no irmão, com os quatro factos e a ordem das explicações.
    if code == K::Delete {
        if let keys_delete::DeleteClaim::NotOurs(porque) = keys_delete::claim_delete(factos) {
            eprintln!("[sculpt3d] o Delete NAO foi para a escultura: {porque}");
            return false;
        }
        // ⚠️ A recusa é REPORTADA. Um Delete que não faz nada e não diz nada é
        // indistinguível de uma tecla que não chegou.
        if scene.delete_active() {
            eprintln!(
                "[sculpt3d] APAGOU: sobram {} pecas -- Ctrl+Z a devolve INTEIRA",
                scene.objects.len()
            );
        } else {
            eprintln!("[sculpt3d] a cena ja' esta' VAZIA: nao ha' peca a apagar");
        }
        return true;
    }
    if !keys_live {
        return false;
    }
    // ⭐⭐ **O TECLADO DA CÂMERA** — a divisão em quatro e as seis vistas
    // nomeadas. Irmão (`#[path]`) pelo tecto de LOC, e o corte é o que a
    // nota deste módulo já desenhava: aqui *o que a mão escolhe sobre o
    // BARRO* (o verbo, o nível, a máscara, o espelho), ali *o que ela
    // escolhe sobre a VISTA*.
    if super::keys_view::camera_key(scene, code, ctrl) {
        return true;
    }
    // ⛔⛔ **O `if ctrl` ABAIXO É UM CATCH-ALL, e é por isso que a câmera vem
    // ANTES dele** (report do Enio, 2026-09-08: *«o atalho das 4 viewports
    // não funciona»*). Ele devolve `false` para todo `Ctrl+` que não seja o
    // desfazer — o que protege os verbos sem modificador de serem
    // disparados por um `Ctrl+1` — e, na primeira redacção desta wave, matou
    // o `Ctrl+Numpad1` (a vista OPOSTA) junto com a tecla da divisão.
    //
    // ⚠️ *Um bloco de modificador que devolve `false` é dono de todo o espaço
    // dele a partir daquela linha; quem quiser um atalho ali tem de vir
    // acima, e nenhum warning o diz.*
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
    if shift && keys_scene::sculpt3d_shift_verbs(scene, code) {
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
        // ⚠️ A MESMA resolução autorada que o botão do painel usa. Duas
        // portas para este número divergiriam no dia em que só uma
        // aprendesse o slider — e o artista teria dois remeshes diferentes
        // para o mesmo gesto.
        match scene.remesh(scene.remesh_res) {
            Ok(r) => eprintln!(
                "[sculpt3d] reconstruida: {} -> {} vertices / {} -> {} faces ({} celulas, {} buraco(s) tapado(s))",
                r.verts.0, r.verts.1, r.faces.0, r.faces.1, r.cells, r.holes_filled
            ),
            // ⚠️ **UMA frase, e ela mora com o tipo** — ver
            // `remesh_refusal.rs`. Antes eram cinco braços aqui e
            // outros cinco no painel, com textos que nada obrigava a
            // concordar. ⭐ A escultura CONTINUA na tela: é isto que a recusa
            // compra, e antes daqui o campo vazado devolvia uma malha vazia
            // que o shell instalava — a peça sumia com log de sucesso.
            Err(e) => {
                debug_assert!(
                    e.reaches_voxel_remesh(),
                    "o voxel remesh devolveu uma recusa que nao e' dele: {e:?}"
                );
                eprintln!("[sculpt3d] {}", e.explain());
            }
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
                "[sculpt3d] topologia dinamica ARMADA -- mas a pilha de multires esta' montada                      e ela RECUSA: refinar a base deixaria cada nivel descrevendo outra malha                      (ACHATE a pilha antes)"
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
        // ⚠️ **O ATALHO E O CHIP PASSAM PELA MESMA PORTA.** Trocar de
        // ferramenta é guardar o pincel vivo no slot do verbo que sai e
        // carregar o do que entra — a lei inteira, e ela mora na
        // `ph2d_panel_sculpt3d::state`, ao lado da tabela que ela move.
        //
        // ⚠️ **A divergência que isto FECHA estava escrita aqui:** a rota
        // do teclado chamava `Brush::arm_verb_defaults`, que armava DOIS
        // campos, e o comentário confessava que *"o falloff, a referência e
        // o raio seguem armados só pela rota do painel"* — trocar de verbo
        // pelo atalho e pelo chip davam pincéis diferentes.
        ph2d_panel_sculpt3d::state::switch_verb_parts(
            &mut scene.verb_slots,
            &mut scene.brush,
            &mut scene.radius_px,
            v,
        );
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
        // ⚠️ **O `D` (o interruptor da doação) MUDOU-SE para a camada do módulo**,
        // no topo desta função, junto com o bake e o painel — ele é o único caminho
        // até `FormRole::Off` e por isso não pode viver debaixo da guarda do barro.
        // Gesto de SMOKE, como o `Q`/`E`/`R`/`F` da luz: a UI final é o toggle
        // *"iluminada pela forma abaixo"* na pilha de camadas (`docs/3D/05.2`), e
        // ele espera a escultura ser uma CAMADA do documento.
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
