//! **O ROTEIRO de cada cena** — o que ela pede ao artista para julgar.
//!
//! Módulo FILHO de [`super`] (`#[path]`), e o corte é por ASSUNTO: lá mora *que malha esta cena
//! monta*, aqui *o que ela manda fazer com ela*. Os dois seguem sendo a mesma pergunta — e é por
//! isso que este é um filho e não um irmão de outro arquivo: o doc do [`super::announce`] argumenta
//! contra pôr o roteiro no arquivo do GESTO, não contra pô-lo ao lado da fixture. Como filho,
//! `use super::*` alcança os predicados de cena sem re-exportar nada.
//!
//! ⚠️ O que forçou o corte foi o teto de LOC (HR-18, 600 na shell), e a cena `=12` foi a que o
//! cruzou. Um arquivo que só cresce em roteiro ia cruzá-lo de novo na cena seguinte.

use super::*;

/// **O roteiro da cena armada** — nada, quando ela não tem passos próprios.
///
/// ⚠️ Recebe a MALHA porque dois roteiros medem a fixture antes de falar dela (a `=4` conta as
/// arestas de beira, a `=6` mede a maior aresta): um roteiro que afirmasse um número sem o medir
/// seria exatamente a cena mentindo, que é o defeito que o smoke do Colorize pagou.
pub(super) fn for_scene(mesh: &ph2d_mesh::Mesh) {
    if crate::sculpt3d::holes_scene() {
        // ⚠️ **A cena DECLARA o furo que montou.** Um smoke de fechar buraco
        // sobre uma malha sem buraco é indistinguível da feature quebrada —
        // a lição que o smoke do Colorize pagou, e aqui o número é a beira.
        let edges = mesh.edges();
        let border = (0..edges.len())
            .filter(|&e| edges.valence(u32::try_from(e).unwrap_or(u32::MAX)) == 1)
            .count();
        eprintln!(
            "[sculpt3d] =4 FECHAR BURACO: a malha abre com {border} arestas de BEIRA -- se este\n\
             [sculpt3d]    numero for zero, PARE: nao ha' buraco e o resto do smoke nao diz nada.\n\
             [sculpt3d]    Esta esfera CHEGOU QUEBRADA -- gire com o botao direito\n\
             [sculpt3d]    ate' o furo, e olhe POR DENTRO dela (nao ha' culling: o interior aparece).\n\
             [sculpt3d]    Aperte O: o log diz quantos buracos tapou, e o furo vira uma TAMPA.\n\
             [sculpt3d]    A tampa e' um leque a partir do centro do contorno, entao ela AFUNDA --\n\
             [sculpt3d]    passe o Smooth (3) nela e ela vira superficie. Ctrl+Z desfaz.\n\
             [sculpt3d]    Depois de tapada, K subdivide e o modelo fica solido de verdade."
        );
    }
    if crate::sculpt3d::cavity_scene() {
        // ⚠️ **A cena MEDE a escada antes de falar dela.** Um roteiro que
        // dissesse *"há sete sulcos de profundidades diferentes"* sem contar os
        // números seria a cena mentindo — o defeito que o smoke do Colorize
        // pagou —, e aqui o número é a CURVATURA, que é literalmente a grandeza
        // que o canal desenha.
        //
        // Cada sulco mora numa faixa de latitude própria, e o número é o máximo
        // dentro dela. Imprimir a ESCADA e não só um total é o que torna o
        // roteiro conferível pelo artista.
        let mut ladder = String::new();
        for k in 0..7usize {
            let v = -0.45 + 0.15 * k as f32;
            let mut mx = 0.0f32;
            for w in 0..mesh.vert_count() {
                let q = mesh.positions()[w];
                if (q[1] - v).abs() < 0.04 && q[2] > 0.3 {
                    mx = mx.max(mesh.curvatures()[w]);
                }
            }
            ladder.push_str(&format!(" {mx:.3}"));
        }
        eprintln!(
            "[sculpt3d] =15 CAVIDADE: sete sulcos PARALELOS, do mais FUNDO (embaixo) ao mais raso.\n\
             [sculpt3d]    curvatura de cada um, de baixo para cima:{ladder}\n\
             [sculpt3d]    -- se o primeiro nao passar de ~0,15, PARE: a escada nao foi cavada e\n\
             [sculpt3d]    o resto do smoke nao diz nada.\n\
             [sculpt3d]    Olhe a esfera COMO ELA ABRE: os sulcos fundos se veem, e os rasos\n\
             [sculpt3d]    quase nao. Essa e' a referencia -- e' o que a luz sozinha mostra.\n\
             [sculpt3d]    Aperte Shift+C: a cavidade vai a 0,35. Os sulcos rasos APARECEM, e o\n\
             [sculpt3d]    log diz o numero. Aperte de novo (0,70) e de novo (1,00); o quarto\n\
             [sculpt3d]    toque volta a ZERO, e a imagem tem de voltar EXATAMENTE a que abriu.\n\
             [sculpt3d]    O que julgar: a fresta ESCURECE e a crista ao lado dela CLAREIA. Se so'\n\
             [sculpt3d]    escurecer, metade do termo nao esta' chegando.\n\
             [sculpt3d]    Depois esculpa com a cavidade LIGADA (0 = Crease, 3 = Smooth): o canal\n\
             [sculpt3d]    acompanha o traco AO VIVO -- a fresta nova nasce escura sob o pincel, e\n\
             [sculpt3d]    o Smooth a apaga.\n\
             [sculpt3d]    E gire com o botao direito: a cavidade e' da FORMA, entao ela NAO nada\n\
             [sculpt3d]    com a camera -- ela fica onde o barro esta'.\n\
             [sculpt3d]    Q/E/R/F movem a luz: a sombra dos sulcos muda e a cavidade nao. Sao\n\
             [sculpt3d]    dois canais, e e' por isso que ela le forma onde a luz nao chega."
        );
    }
    if crate::sculpt3d::turn_scene() {
        // ⚠️ **A cena DECLARA que trouxe cristas.** Numa esfera LISA um
        // Twist em torno do eixo da vista é quase invisível — ela é
        // invariante por rotação —, e o smoke não teria como separar a
        // feature funcionando da feature morta.
        eprintln!(
            "[sculpt3d] =5 TORCER e INFLAR: esta esfera tem uma CRUZ de cristas, e ela existe\n\
             [sculpt3d]    porque numa esfera LISA um giro em torno do eixo da vista nao se ve.\n\
             [sculpt3d]    Aperte T, pegue o CRUZAMENTO das cristas e VARRA um circulo em volta\n\
             [sculpt3d]    dele: os bracos entortam em redemoinho. Varra de VOLTA ao comeco --\n\
             [sculpt3d]    a cruz tem de voltar reta (o gesto e' o TOTAL varrido, nao a soma dos passos).\n\
             [sculpt3d]    Perto do ponto que voce pegou ha' uma ZONA MORTA de 30 px: ali a direcao\n\
             [sculpt3d]    e' ruido, e nada gira ate' voce sair dela.\n\
             [sculpt3d]    Aperte S e arraste na HORIZONTAL: para a direita o cruzamento incha,\n\
             [sculpt3d]    para a esquerda ele encolhe -- e volta ao lugar no caminho de volta.\n\
             [sculpt3d]    Aperte X (espelho) e repita o T: as duas metades tem de girar para\n\
             [sculpt3d]    lados OPOSTOS (um redemoinho no espelho gira ao contrario); com o S\n\
             [sculpt3d]    as duas metades incham JUNTAS."
        );
    }
    if crate::sculpt3d::remesh_scene() {
        // ⚠️ **A cena DECLARA o esticamento que montou.** Um smoke de remesh
        // sobre uma malha saudável é indistinguível da feature quebrada: a
        // forma sobrevive nos dois casos, e é só a DENSIDADE que muda. O
        // número aqui é a maior aresta — a mesma lição da cena `=4`.
        let pos = mesh.positions();
        let mut longest = 0.0f32;
        let mut tris = Vec::new();
        mesh.triangle_indices(&mut tris);
        for t in &tris {
            for k in 0..3 {
                let a = pos[t[k] as usize];
                let b = pos[t[(k + 1) % 3] as usize];
                let d =
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                longest = longest.max(d);
            }
        }
        eprintln!(
            "[sculpt3d] =6 O REMESH: a maior aresta desta malha mede {longest:.3} -- se este numero\n\
             [sculpt3d]    nao passar de ~0.15, PARE: o bico nao esticou e o resto nao diz nada.\n\
             [sculpt3d]    Esta esfera foi PUXADA por um snake hook ate' o barro acabar: gire e olhe\n\
             [sculpt3d]    o bico -- ele esta' FACETADO, feito de poucos triangulos compridos.\n\
             [sculpt3d]    (1) Tente esculpir na PONTA dele (Draw, tecla 1): quase nada acontece,\n\
             [sculpt3d]        porque nao ha' vertices ali. Essa e' a doenca.\n\
             [sculpt3d]    (2) Aperte V: o log diz vertices ANTES -> DEPOIS. A FORMA tem de\n\
             [sculpt3d]        sobreviver -- se o bico sumir ou a esfera virar outra coisa, reprove.\n\
             [sculpt3d]    (3) Esculpa na MESMA ponta de novo: agora ela responde. Esse e' o botao.\n\
             [sculpt3d]    (4) Ctrl+Z devolve a malha esticada, inteira; Ctrl+Shift+Z refaz.\n\
             [sculpt3d]    (5) Aperte K (subdividir) e depois V: ele RECUSA, e o log diz por que --\n\
             [sculpt3d]        um remesh troca a topologia, e os niveis de cima sao subdivisao dela."
        );
    }
    if document_scene() {
        // ⚠️ **A cena DECLARA quantas peças montou e com que forma.** Um smoke
        // de persistência sobre uma esfera LISA é indistinguível da feature
        // quebrada — reabrir e ver uma esfera não diz se ela VOLTOU ou se
        // NASCEU. Por isso a peça central tem cristas, e por isso o número aqui
        // é a contagem de peças: se ele não for 3, PARE.
        eprintln!(
            "[sculpt3d] =8 O DOCUMENTO: 3 pecas na mesa -- a esfera com CRISTAS no centro,\n             [sculpt3d]    um CUBO grande a` esquerda e um OCTAEDRO pequeno a` direita, cada um\n             [sculpt3d]    com pose propria. Se voce nao ver TRES formas diferentes, PARE.\n             [sculpt3d]    1) Esculpa: marque a esfera com um traco que voce reconheca depois.\n             [sculpt3d]    2) Aperte K (subdividir), esculpa um detalhe FINO, e aperte , (descer ao 0).\n             [sculpt3d]       -- e' esta a janela em que o detalhe fino so' existe no documento.\n             [sculpt3d]    3) Ctrl+S. FECHE o app. Abra de novo com a MESMA variavel e Ctrl+O.\n             [sculpt3d]    A escultura tem de voltar INTEIRA: as tres pecas, nas mesmas poses,\n             [sculpt3d]    com o seu traco -- e no nivel 0, que e' onde voce estava.\n             [sculpt3d]    4) Aperte . (subir): o detalhe fino do passo 2 tem de estar la'.\n             [sculpt3d]    Ctrl+Z depois de abrir NAO pode desfazer nada da sessao anterior."
        );
    }
    if objects_scene() {
        eprintln!(
            "[sculpt3d] =7 A CENA E' UMA LISTA: tres pecas, cada uma no SEU lugar e no SEU tamanho.\n\
             [sculpt3d]    Uma esfera no meio, um CUBO grande a' esquerda, um OCTAEDRO pequeno a' direita.\n\
             [sculpt3d]    (1) Gire (botao direito) e olhe: as tres tem de estar la', separadas, e a\n\
             [sculpt3d]        perspectiva tem de ser coerente -- nenhuma pode nadar em relacao as outras.\n\
             [sculpt3d]    (2) Esculpa no CUBO (esquerdo). O barro tem de ceder EXATAMENTE sob o cursor,\n\
             [sculpt3d]        e a pegada tem de ter o mesmo tamanho APARENTE que na esfera do meio --\n\
             [sculpt3d]        e' isso que prova que o pincel atravessou a escala da peca.\n\
             [sculpt3d]    (3) Esculpa no OCTAEDRO (pequeno, a' direita): mesma coisa. Se a pegada dele\n\
             [sculpt3d]        parecer MAIOR ou MENOR que a das outras, reprove.\n\
             [sculpt3d]    (4) Esculpa uma peca, depois OUTRA, e aperte Ctrl+Z duas vezes: cada undo tem\n\
             [sculpt3d]        de desfazer NA PECA CERTA. Se a segunda peca 'consertar' a primeira, reprove.\n\
             [sculpt3d]    (5) Aproxime com a roda ate' o cubo ocupar a tela e esculpa: o pincel continua\n\
             [sculpt3d]        do mesmo tamanho em PIXELS, como sempre foi.\n\
             [sculpt3d]    (6) Onde as pecas se cruzam na tela, clicar tem de pegar a que esta' NA FRENTE.\n\
             [sculpt3d]    (7) OS VERBOS DA LISTA: Shift+1 esfera, Shift+2 cubo, Shift+3 cilindro,\n\
             [sculpt3d]        Shift+4 toro. A peca nova nasce ONDE VOCE ESTA' OLHANDO e ja' vem ativa --\n\
             [sculpt3d]        esculpa nela sem clicar em mais nada.\n\
             [sculpt3d]    (8) Shift+D DUPLICA a ativa: a copia nasce AO LADO na tela (gire e confira que\n\
             [sculpt3d]        ela continua ao lado do ponto de vista NOVO, nao presa a um eixo de mundo).\n\
             [sculpt3d]    (9) Delete APAGA a ativa, e Ctrl+Z tem de devolve-la INTEIRA -- com o que voce\n\
             [sculpt3d]        esculpiu nela. Tente apagar ate' sobrar UMA: a ultima o log RECUSA.\n\
             [sculpt3d]   (10) O teste duro do undo: esculpa a peca A, acrescente B, esculpa B, apague B,\n\
             [sculpt3d]        e va' desfazendo. Cada passo tem de voltar NA PECA CERTA, na ordem inversa."
        );
    }
    if crate::sculpt3d::reversion_scene() {
        eprintln!(
            "[sculpt3d] =3 A REVERSAO: esta malha densa CHEGOU PRONTA -- um nivel so', e por isso\n\
             [sculpt3d]    o ',' nao leva a lugar nenhum. Aperte J: a malha NAO muda de forma\n\
             [sculpt3d]    (e' esse o ponto), e nasce um nivel ABAIXO dela. Aperte J de novo.\n\
             [sculpt3d]    Agora ',' desce ate' a base grossa: mova UM vertice la' e suba com '.'\n\
             [sculpt3d]    -- a forma grande andou e a pele fina continua onde estava.\n\
             [sculpt3d]    Ctrl+Z desfaz cada J; Ctrl+Shift+Z refaz."
        );
    }
    if bake_scene() {
        // ⚠️ **Os passos (4) e (5) são a wave**, e nenhum dos dois é sobre o momento do bake: o que
        // separa isto de um carimbo é o objeto continuar RESPONDENDO à luz depois, e continuar
        // aceso depois de a escultura sair. Um roteiro que parasse no (3) deixaria o artista
        // aprovar um efeito que qualquer filtro de imagem entrega.
        eprintln!(
            "[sculpt3d] =11 O OBJETO MISTO: ha' um SPRITE branco na mesa (ja' SELECIONADO), e a\n\
             [sculpt3d]    forma da esfera vai acender ELE.\n\
             [sculpt3d]    (1) A esfera chega com CRISTAS -- e' delas que a luz toma a forma.\n\
             [sculpt3d]        Gire (botao direito) ate' a vista que voce quer assar: o bake usa a\n\
             [sculpt3d]        camera do ESCULTOR, entao o que voce ve' e' o que ele grava.\n\
             [sculpt3d]    (2) Shift+B ASSA. O log diz o tamanho.\n\
             [sculpt3d]    (3) Aperte D UMA vez: o barro sai da tela e o SPRITE aparece. Ele tem de\n\
             [sculpt3d]        estar com o RELEVO DA ESFERA desenhado em luz e sombra. Se ele so'\n\
             [sculpt3d]        escureceu por igual, reprove.\n\
             [sculpt3d]        (com o barro na tela o sprite fica ATRAS dele -- por isso o D.)\n\
             [sculpt3d]    (4) O TESTE DA WAVE: Q/E giram a lampada, R/F a sobem. O sprite tem de\n\
             [sculpt3d]        RE-ACENDER a cada toque -- as sombras ANDAM. Se ele ficar congelado,\n\
             [sculpt3d]        isto e' um carimbo e nao um objeto, e a wave falhou.\n\
             [sculpt3d]    (5) O SEGUNDO TESTE: aperte D ate' voltar ao BARRO e Delete ate' a\n\
             [sculpt3d]        escultura sumir; volte ao D. O sprite tem de continuar aceso E as\n\
             [sculpt3d]        teclas de luz tem de continuar movendo as sombras dele -- sem malha.\n\
             [sculpt3d]    (6) O assado NAO vai bater com o barro, e a diferenca esta' MEDIDA:\n\
             [sculpt3d]        a LEI da luz e' a mesma -- com o mesmo albedo dos dois lados as duas\n\
             [sculpt3d]        concordam a 0,0020 no ARO (e ha' gate). O que difere e' o ALBEDO:\n\
             [sculpt3d]        o passe leva a luz ate' 1,65x, e um sprite e' unorm8 -- entao sobre\n\
             [sculpt3d]        BRANCO 43,6% da esfera satura em (255,255,255) e a forma SOME ali.\n\
             [sculpt3d]        O barro vivo nunca satura porque e' HDR e a cor dele e' 0,74.\n\
             [sculpt3d]        Sobre arte de meio-tom (128) o estouro e' ZERO -- e' o albedo que\n\
             [sculpt3d]        decide quanto da forma sobrevive, nao a lampada.\n\
             [sculpt3d]    (7) Assar DE NOVO por outro angulo tem de substituir a luz, nao somar --\n\
             [sculpt3d]        gire, Shift+B, e o sprite nao pode ficar mais escuro a cada bake.\n\
             [sculpt3d]    (8) E ele SOBREVIVE a fechar o app -- mas isso e' a cena =12, que tem\n\
             [sculpt3d]        um passo destrutivo (fechar) e por isso mora separada."
        );
    }
    if reopen_scene() {
        // ⚠️ **O roteiro tem de dizer o CAMINHO do arquivo**, e a cena o fixa: sem `PH2D_PROJECT_PATH`
        // o save cai no CWD, e o artista que roda o segundo comando de outro diretório abre um
        // projeto vazio e reprova uma feature que funciona.
        //
        // ⚠️ E o passo (5) é o que separa esta cena de um teste de persistência qualquer: reabrir com
        // os pixels certos prova que os PIXELS viajaram; mover a lâmpada depois prova que os CANAIS
        // viajaram. Um roteiro que parasse no (4) aprovaria uma fotografia.
        eprintln!(
            "[sculpt3d] =12 O OBJETO ASSADO QUE VOLTA: a mesma mesa da =11 -- um SPRITE branco ja'\n\
             [sculpt3d]    selecionado e uma esfera com cristas para acende-lo.\n\
             [sculpt3d]    RODE ASSIM (o caminho importa -- o save grava onde voce mandar):\n\
             [sculpt3d]      env PH2D_SCULPT3D_SMOKE=12 PH2D_PROJECT_PATH=/tmp/ph2d_w87.postcard \\\n\
             [sculpt3d]          cargo run -p ph2d-host-desktop --release\n\
             [sculpt3d]    (1) Gire ate' a vista que voce quer (botao direito) e Shift+B ASSA.\n\
             [sculpt3d]    (2) Aperte D uma vez: o barro sai e o SPRITE aparece, com o relevo da\n\
             [sculpt3d]        esfera desenhado em luz e sombra. Confira que Q/E movem as sombras.\n\
             [sculpt3d]    (3) Ctrl+S. O log diz `[proj] salvo:` com o caminho e o tamanho.\n\
             [sculpt3d]        ⚠️ Ele tem de dizer ~8 MB: os canais de um sprite de 1024 sao 4 MiB de\n\
             [sculpt3d]        pixels + 4 MiB de forma (MEDIDO -- guardar a forma como f32 custaria\n\
             [sculpt3d]        16). Alguns KB significa que os canais NAO foram gravados.\n\
             [sculpt3d]    (4) FECHE O APP e rode o MESMO comando de novo. Ctrl+O.\n\
             [sculpt3d]        ⚠️ O TESTE DA WAVE: o sprite tem de voltar ACESO, com a MESMA luz --\n\
             [sculpt3d]        nao branco, nao chapado, e nao com a lampada default. Se ele voltar\n\
             [sculpt3d]        em branco os canais nao viajaram; se voltar aceso por OUTRO angulo,\n\
             [sculpt3d]        o rig nao viajou.\n\
             [sculpt3d]    (5) O SEGUNDO TESTE, e e' ele que separa isto de uma fotografia: aperte\n\
             [sculpt3d]        Q/E/R/F. As sombras do sprite reaberto tem de ANDAR. Se ficarem\n\
             [sculpt3d]        congeladas, o que voltou foi a imagem e nao o objeto.\n\
             [sculpt3d]    (6) O terceiro, opcional: rode o mesmo comando SEM a escultura na mesa\n\
             [sculpt3d]        (`env PH2D_PROJECT_PATH=/tmp/ph2d_w87.postcard cargo run ...`, sem o\n\
             [sculpt3d]        SCULPT3D_SMOKE) e Ctrl+O. O objeto continua aceso, e a luz do card do\n\
             [sculpt3d]        Painter continua movendo as sombras dele -- sem cena 3D nenhuma."
        );
    }
    if crate::sculpt3d::fuse_scene() {
        // ⚠️ **A cena DECLARA quantas peças montou**, e o número é o oráculo dos
        // dois verbos: fundir e isolar mudam a CONTAGEM, e o log de cada um diz
        // a nova. Sem a linha de abertura o artista não teria contra o que
        // comparar.
        eprintln!(
            "[sculpt3d] =13 A FUSAO e o ISOLAMENTO: QUATRO pecas de formas diferentes --\n\
             [sculpt3d]    a esfera (centro), um cubo grande (esquerda), um octaedro (direita)\n\
             [sculpt3d]    e um cubo pequeno (em cima). Se as quatro nao aparecerem, PARE.\n\
             [sculpt3d]    (1) Shift+I ISOLA a peca que voce clicou por ultimo: as outras tres\n\
             [sculpt3d]        SOMEM. Clique numa peca antes, para escolher qual fica.\n\
             [sculpt3d]        Esculpa: o pincel so' alcanca o que esta' a' vista.\n\
             [sculpt3d]        Shift+I de novo devolve a cena inteira -- e as tres que voltam tem\n\
             [sculpt3d]        de ser AS MESMAS FORMAS de antes, cada uma no seu lugar.\n\
             [sculpt3d]    (2) O TESTE DO SLOT (um bug que esta wave achou e curou): clique no\n\
             [sculpt3d]        cubo GRANDE e aperte Delete. As tres que sobram tem de continuar\n\
             [sculpt3d]        sendo ELAS MESMAS -- esfera, octaedro, cubo pequeno. Se alguma\n\
             [sculpt3d]        aparecer com a forma da que voce apagou, o device ficou com a\n\
             [sculpt3d]        geometria da peca morta. Ctrl+Z traz o cubo de volta.\n\
             [sculpt3d]    (3) Shift+J FUNDE tudo o que esta' a' vista numa peca so'. A imagem\n\
             [sculpt3d]        quase nao muda -- e' isso mesmo: as pecas ficam onde estavam. O\n\
             [sculpt3d]        log diz quantas entraram e o tamanho do que saiu.\n\
             [sculpt3d]        Agora esculpa ATRAVESSANDO duas delas: e' UMA malha, entao o\n\
             [sculpt3d]        pincel nao 'troca de peca' no meio do traco.\n\
             [sculpt3d]    (4) Elas NAO ficam soldadas (fundir nao e' soldar). Aperte V\n\
             [sculpt3d]        (reconstruir): agora sim elas viram uma casca so'.\n\
             [sculpt3d]    (5) Ctrl+Z desfaz a fusao INTEIRA num passo -- as quatro voltam, com\n\
             [sculpt3d]        as poses e as formas delas. Ctrl+Shift+Z funde de novo.\n\
             [sculpt3d]    (6) Com uma peca ISOLADA, Shift+J tem de RECUSAR e dizer por que:\n\
             [sculpt3d]        fundir age no que se ve, e o que se ve e' uma peca so'.\n\
             [sculpt3d]    (7) Aperte K (subdividir) numa peca e tente Shift+J: ele recusa com a\n\
             [sculpt3d]        pilha montada, e o log diz o conserto (J reverte)."
        );
    }
    if crate::sculpt3d::dyntopo_scene() {
        // ⚠️ **A cena imprime a contagem de partida**, e ela é o oráculo: o
        // modo existe para MUDAR esse número onde o pincel toca, e sem o valor
        // de antes o artista não tem contra o que comparar.
        eprintln!(
            "[sculpt3d] =14 A TOPOLOGIA DINAMICA: uma esfera GROSSA (as facetas tem de
             [sculpt3d]    ser visiveis a olho nu -- se ela vier lisa, PARE: e' a cena errada).
             [sculpt3d]    (1) O CONTROLE, primeiro: esculpa um dedo AGORA, com o modo
             [sculpt3d]        desligado. A forma sai FACETADA -- a malha nao tem triangulos
             [sculpt3d]        para descrever o que voce pediu. E' esse o problema.
             [sculpt3d]        Ctrl+Z desfaz.
             [sculpt3d]    (2) Aperte P. O log diz LIGADA e quantas faces a triangulacao
             [sculpt3d]        criou (a esfera nasce em quads, e o modo e' de triangulos).
             [sculpt3d]        A silhueta NAO pode mudar: triangular nao move um vertice.
             [sculpt3d]    (3) Esculpa o mesmo dedo. Agora a malha ADENSA onde o pincel passa,
             [sculpt3d]        e so' ali -- o resto da esfera continua grosso. Olhe a borda do
             [sculpt3d]        traco: ela tem de ser LISA. Um buraco fino, uma faceta preta ou
             [sculpt3d]        uma quina que pisca na luz e' rachadura, e e' reprovacao.
             [sculpt3d]    (4) A LEI DO TRACO, e este passo e' o que separa a wave de um port
             [sculpt3d]        ingenuo: passe DEVAGAR e depois RAPIDO sobre o mesmo caminho, com
             [sculpt3d]        a mesma forca. As duas passadas tem de deixar o MESMO relevo. Se
             [sculpt3d]        a lenta afundar mais, o traco voltou a compor.
             [sculpt3d]    (5) U cicla o detalhe (grosso / medio / fino) e o alvo e' uma FRACAO
             [sculpt3d]        DO PINCEL: no fino, um pincel pequeno ([ diminui) faz detalhe
             [sculpt3d]        muito mais fino que o mesmo fino num pincel grande.
             [sculpt3d]    (6) O CAMINHO DE VOLTA, e ele e' a metade nova: com o detalhe no FINO
             [sculpt3d]        esculpa ate' a malha adensar bem. Agora aperte U ate' GROSSO (o log
             [sculpt3d]        diz a contagem) e passe o MESMO pincel por cima. A contagem tem de
             [sculpt3d]        DESCER -- o alvo cresceu, entao o que era detalhe passou a ser
             [sculpt3d]        excesso e o traco o retira. Medido a 10 dabs por passada:
             [sculpt3d]        128 -> 621 vertices no fino, 255 no grosso, 698 de volta ao fino.
             [sculpt3d]        A SILHUETA tem de sobreviver: o vertice que fica desliza pela
             [sculpt3d]        superficie, nao afunda nela. Se a esfera murchar, PARE.
             [sculpt3d]    (7) E o par nao pode MOER: fique parado com o pincel apoiado, no mesmo
             [sculpt3d]        detalhe, e a contagem tem de ASSENTAR. Se ela subir e descer para
             [sculpt3d]        sempre, a histerese quebrou.
             [sculpt3d]    (8) Ctrl+Z desfaz o traco INTEIRO num passo -- a malha volta a ser a
             [sculpt3d]        grossa, com a contagem de antes. Ctrl+Shift+Z refaz.
             [sculpt3d]    (9) P de novo DESLIGA: o traco seguinte volta a facetar (a malha
             [sculpt3d]        adensada FICA -- desligar nao desfaz).
             [sculpt3d]   (10) A RECUSA: Ctrl+Z ate' voltar ao inicio, aperte K (subdividir) e
             [sculpt3d]        depois P. O log tem de dizer que RECUSA com a pilha montada, e
             [sculpt3d]        dizer o conserto (J reverte)."
        );
    }
    if crate::sculpt3d::donation_scene() {
        eprintln!(
            "[sculpt3d] =2 A DOACAO: ha uma TELA BRANCA embaixo, e a tecla D alterna\n\
             [sculpt3d]    BARRO (esculpir) -> LUZ (a forma acende a tinta) -> DESLIGADA (o A/B)\n\
             [sculpt3d]    esculpa, aperte D, pegue o Painter e pinte CHAPADO: a tinta tem de sair ACESA\n\
             [sculpt3d]    aperte D de novo e a MESMA tinta fica plana -- e essa diferenca e a wave"
        );
    }
}
