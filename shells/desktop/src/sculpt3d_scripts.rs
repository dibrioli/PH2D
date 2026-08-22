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
pub(crate) fn for_scene(mesh: &ph2d_mesh::Mesh) {
    if crate::sculpt3d::env_scene() {
        // ⚠️ **A cena MEDE o que ela promete**, como as vizinhas: o número aqui é
        // o CONTRASTE entre o piso que uma face virada para o topo da tela recebe
        // e o que uma virada para o fundo recebe. Sem ele o roteiro pediria ao
        // artista que julgasse uma propriedade sem dizer de que tamanho ela é.
        let up = ph2d_light::env_ambient([0.0, -1.0, 0.0]);
        let down = ph2d_light::env_ambient([0.0, 1.0, 0.0]);
        let l = |c: [f32; 3]| 0.2126f32.mul_add(c[0], 0.7152f32.mul_add(c[1], 0.0722 * c[2]));
        eprintln!(
            "[sculpt3d] =24 O AMBIENTE TEM DIRECAO -- uma esfera com rugas EM ESCADA.\n\
             [sculpt3d]    O piso da difusa (o que uma face virada para longe da luz devolve)\n\
             [sculpt3d]    deixou de ser UM numero para toda direcao: para o topo da tela ele\n\
             [sculpt3d]    vale {:.3} e para o fundo {:.3} -- {:.2}x de contraste, com a MEDIA\n\
             [sculpt3d]    sobre todas as normais inalterada (o termo redistribui, nao expoe).\n\
             [sculpt3d]    ⚠️ Esta cena traz LUZ PROPRIA: uma lampada RASANTE vinda da DIREITA,\n\
             [sculpt3d]       entao a sombra e' a METADE ESQUERDA inteira, de cima a baixo. O rig\n\
             [sculpt3d]       de todo dia vem de CIMA -- do mesmo lado em que este ambiente poe o\n\
             [sculpt3d]       ceu --, e sob ele apenas 11,5% da sombra visivel recebe a metade\n\
             [sculpt3d]       CLARA do termo: o artista veria so' escurecer.\n\
             [sculpt3d]    Abra o painel com a CRASE (`) -- o slider 'Environment' fica na\n\
             [sculpt3d]    secao Shading, entre Cavity e AO.\n\
             [sculpt3d]    (1) Olhe a METADE ESQUERDA da peca -- a que a lampada nao alcanca.\n\
             [sculpt3d]        Cada degrau tem um topo virado para o ceu e um beiral virado para\n\
             [sculpt3d]        o chao.\n\
             [sculpt3d]    (2) A PERGUNTA DA WAVE: o slider nasce em ZERO -- a sombra inteira e'\n\
             [sculpt3d]        UM cinza chapado, os degraus somem ali, e e' assim que o barro era\n\
             [sculpt3d]        ate' ontem. Arraste 'Environment' ate' 1: o ALTO da sombra CLAREIA\n\
             [sculpt3d]        e esfria, o BAIXO escurece e esquenta. Volte a 0 e a imagem tem de\n\
             [sculpt3d]        voltar EXATAMENTE a que abriu.\n\
             [sculpt3d]    (3) O SINAL, e ele e' de olho: o CLARO tem de ficar EM CIMA -- os dois\n\
             [sculpt3d]        lados do termo tem de aparecer, nao so' o escuro. O gate mede, na\n\
             [sculpt3d]        mesma geometria, 62,6/62,6 desligado contra 75,2 no alto e 50,4 no\n\
             [sculpt3d]        fundo. Se a sombra so' escurecer, ou clarear por BAIXO -- PARE.\n\
             [sculpt3d]    (4) Q/E giram a lampada. O ambiente NAO gira com ela: ele e' o\n\
             [sculpt3d]        estudio, e o estudio nao se move quando voce move uma luz.\n\
             [sculpt3d]    (5) Escolha um MATCAP na secao Shading: o slider 'Environment' tem de\n\
             [sculpt3d]        SUMIR. Um matcap ja' E' um ambiente -- somar o nosso em cima\n\
             [sculpt3d]        seriam dois, e o slider seria um controle que nao faz nada.",
            l(up),
            l(down),
            l(up) / l(down)
        );
    }
    if crate::sculpt3d::alpha_image_scene() {
        eprintln!(
            "[sculpt3d] =25 O ALPHA POR IMAGEM -- uma esfera SULCADA e um sprite na mesa.\n\
             [sculpt3d]    Ate' agora os padroes eram NOVE FORMULAS: o artista escolhia um nome.\n\
             [sculpt3d]    Agora ele pode APONTAR para uma imagem, e ela vira o padrao do pincel.\n\
             [sculpt3d]    A lei e' a do ZBrush e a do slot Shape do Painter -- BRANCO E' CHEIO\n\
             [sculpt3d]    (luminancia), e um texel TRANSPARENTE nao tem tinta nenhuma.\n\
             [sculpt3d]    (1) Clique o SPRITE no canvas para selecionar. Abra o painel com a\n\
             [sculpt3d]        CRASE (`) -- na secao Brush, logo abaixo da fileira de padroes,\n\
             [sculpt3d]        aparece 'Use Selected Sprite as Pattern'.\n\
             [sculpt3d]    ⚠️ Sem sprite selecionado o botao NAO EXISTE -- ele nao fica apagado.\n\
             [sculpt3d]       Um botao que so' pode falhar e' como se aprende que ele nao funciona.\n\
             [sculpt3d]    (2) Aperte-o. O log diz o TAMANHO que ele leu; se nao disser, PARE.\n\
             [sculpt3d]        O swatch do preview passa a mostrar a imagem, e a escala e'\n\
             [sculpt3d]        SEMEADA com o que este modelo comporta (sem isso os poros saem\n\
             [sculpt3d]        gigantescos -- e' um smoke que ja' foi reprovado assim).\n\
             [sculpt3d]    (3) ESCULPA. O relevo tem de sair com o desenho da imagem, LADRILHADO\n\
             [sculpt3d]        pela superficie -- nao um carimbo unico com borda. Ele e' um\n\
             [sculpt3d]        PADRAO, irmao dos nove, nao a ponta finita do slot Shape.\n\
             [sculpt3d]    (4) A imagem e' DIRECIONAL: arraste 'Pattern Angle' e ela tem de GIRAR\n\
             [sculpt3d]        na peca. As duas pistas de eixo aparecem por isso.\n\
             [sculpt3d]    (5) Volte ao chip 'None' da fileira: o pincel volta a ser liso, e o\n\
             [sculpt3d]        botao continua la' para apontar de novo."
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
    if crate::sculpt3d::alpha_scene() {
        // ⚠️ **A cena MEDE a razão antes de falar dela.** Um padrão é uma função
        // contínua e a malha o amostra nos VÉRTICES: se a célula tiver a ordem
        // da aresta, cada vértice vê um valor independente do vizinho e o que
        // chega ao barro é chuvisco. O número que decide é `célula ÷ aresta`, e
        // ele é da MALHA que esta cena de fato abriu — não de uma que eu
        // esperava que ela abrisse.
        let pos = mesh.positions();
        let ring = mesh.adjacency();
        let mut lens: Vec<f32> = Vec::new();
        for v in 0..pos.len() {
            for &n in ring.vert_verts.neighbours(v) {
                if n as usize > v {
                    let (a, b) = (pos[v], pos[n as usize]);
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    lens.push(d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt());
                }
            }
        }
        lens.sort_by(f32::total_cmp);
        let edge = lens[lens.len() / 2];
        // ⚠️ **O número que o roteiro imprime é o que o PRODUTO vai semear**, e
        // não a constante de fábrica: ela virou sentinela quando o 1º smoke desta
        // wave reprovou o default absoluto (*"os poros são gigantescos"*), e um
        // roteiro que ainda a citasse estaria descrevendo um build que não existe.
        let seed = ph2d_sculpt3d::recommended_scale(mesh);
        let ratio = seed / edge;
        let across = 2.0 / seed;
        eprintln!(
            "[sculpt3d] =16 ALPHA: aresta mediana {edge:.4} · a escala que esta malha comporta\n\
             [sculpt3d]    e' {seed:.4} ({ratio:.1} arestas por feature, {across:.0} features\n\
             [sculpt3d]    atravessando o modelo)\n\
             [sculpt3d]    -- se as features nao passarem de ~20, PARE: o padrao vai sair como\n\
             [sculpt3d]    CRATERA, nao como textura, e o resto do smoke nao diz nada.\n\
             [sculpt3d]    O ALPHA e' a fileira nova na secao BRUSH, logo abaixo do Falloff:\n\
             [sculpt3d]    None (o pincel liso) e NOVE padroes -- os seis ISOTROPICOS e os tres\n\
             [sculpt3d]    DIRECIONAIS, que tem eixo e sao julgados pela cena =21. A pista\n\
             [sculpt3d]    ALPHA SCALE aparece\n\
             [sculpt3d]    LOGO EMBAIXO dela, e SO' com um padrao armado -- sem padrao ela\n\
             [sculpt3d]    mediria uma feature que nao existe.\n\
             [sculpt3d]    Escolha 'Pores' e desenhe com o Draw (1) numa faixa larga: em vez de\n\
             [sculpt3d]    um monte liso tem de sair PELE -- pontos, nao uma rampa.\n\
             [sculpt3d]    Passe pelos seis isotropicos: Noise (rocha) · Pores (pele) ·\n\
             [sculpt3d]    Scales (reptil) · Cracks (terra seca) · Grain (chatter fino) ·\n\
             [sculpt3d]    Ridges (ruga, casca).\n\
             [sculpt3d]    ESCOLHER UM PADRAO SEMEIA A ESCALA que o modelo comporta -- e' por\n\
             [sculpt3d]    isso que ele ja' abre com tamanho de textura em vez de cratera. Uma\n\
             [sculpt3d]    escala e' absoluta (um poro tem o tamanho de um poro), mas QUAL numero\n\
             [sculpt3d]    depende do tamanho e da densidade da peca. Depois de voce arrastar a\n\
             [sculpt3d]    pista, ela e' SUA: trocar de padrao nao a pisa mais.\n\
             [sculpt3d]    O QUE JULGAR, e e' a wave inteira: o padrao esta' colado ao ESPACO,\n\
             [sculpt3d]    nao ao gesto. Passe DEVAGAR e depois RAPIDO pelo mesmo lugar -- tem de\n\
             [sculpt3d]    sair igual. Passe de VOLTA pelo caminho -- os poros caem nos MESMOS\n\
             [sculpt3d]    lugares. Se eles se mexerem com a mao, o padrao esta' preso ao dab.\n\
             [sculpt3d]    PARA IR MAIS FINO: subdivida (K) e ARRASTE a pista para a esquerda.\n\
             [sculpt3d]    Um padrao so' pode ser tao fino quanto a malha o amostra -- e' o mesmo\n\
             [sculpt3d]    fato que faz um escultor de ZBrush subdividir antes de pegar um alpha.\n\
             [sculpt3d]    ATENCAO ao Cracks e ao Pores: eles cobrem ~14% da superficie de\n\
             [sculpt3d]    proposito (uma trinca e' uma LINHA), entao o pincel parece mais fraco\n\
             [sculpt3d]    -- suba a Forca. O Ridges cobre ~70% e quase nao enfraquece.\n\
             [sculpt3d]    E pegue o Crease (0) com um padrao: o vinco AFIA o padrao (ele entra\n\
             [sculpt3d]    na quinta potencia), entao o mesmo alpha sai muito mais recortado."
        );
    }
    if crate::sculpt3d::directional_alpha_scene() {
        // ⚠️ **A MESMA medição da `=16`, e pelo mesmo motivo:** a lei das dez
        // arestas não muda porque o padrão ganhou eixo. Um estrato picado por
        // uma malha grossa lê como chuvisco, e girar o eixo de um chuvisco é
        // indistinguível de o eixo não fazer nada.
        let seed = ph2d_sculpt3d::recommended_scale(mesh);
        let across = 2.0 / seed;
        eprintln!(
            "[sculpt3d] =21 O EIXO: a familia DIRECIONAL do alpha (a escala que esta malha\n\
             [sculpt3d]    comporta e' {seed:.4}, {across:.0} features atravessando o modelo).\n\
             [sculpt3d]    -- se as features nao passarem de ~20, PARE, como na =16.\n\
             [sculpt3d]    Os seis primeiros padroes leem IGUAL de qualquer direcao. Os tres\n\
             [sculpt3d]    ultimos -- Strata, Scratches, Weave -- tem um EIXO, e ele e' a wave.\n\
             [sculpt3d]    1) Escolha STRATA na fileira Alpha. DUAS pistas novas aparecem logo\n\
             [sculpt3d]       abaixo da Alpha Scale: PATTERN ANGLE e PATTERN TILT. Elas so'\n\
             [sculpt3d]       existem com um padrao direcional armado -- volte para 'Pores' e\n\
             [sculpt3d]       elas SOMEM (sob um isotropico o eixo nao move um bit, ha' gate).\n\
             [sculpt3d]    1b) ⚠️ O BARRO INTEIRO FICA TINGIDO DE VIOLETA com o padrao, na hora\n\
             [sculpt3d]       em que voce escolhe o alpha -- e ESSE e' o preview que importa:\n\
             [sculpt3d]       ele mostra, na SUA forma, como o padrao se deita nela antes de\n\
             [sculpt3d]       voce tocar o barro. Ele nao e' a mascara (aquela e' AZUL-FRIA) nem\n\
             [sculpt3d]       o cursor (AMBAR): tres canais, tres cores. A caixa PREVIEW ON\n\
             [sculpt3d]       MODEL desliga o tinto quando voce quiser o barro limpo.\n\
             [sculpt3d]    1c) O quadrinho no painel e' o IRMAO dele e responde outra coisa:\n\
             [sculpt3d]       ele abrange 1/8 do SEU modelo, entao a densidade que voce ve e' a\n\
             [sculpt3d]       que o pincel vai depositar. Arraste a ALPHA SCALE e o padrao TEM\n\
             [sculpt3d]       de mudar NA HORA nos DOIS -- no quadro e no barro. Abaixo da\n\
             [sculpt3d]       escala que a malha resolve ele DIZ isso numa linha, em vez de\n\
             [sculpt3d]       mostrar um padrao lindo que sai como chuvisco.\n\
             [sculpt3d]    2) Desenhe uma faixa larga com o Draw (1). Tem de sair CAMADAS\n\
             [sculpt3d]       HORIZONTAIS -- e' o eixo de fabrica (+Y), a leitura que um estrato\n\
             [sculpt3d]       tem no mundo.\n\
             [sculpt3d]    3) ⚠️ O TESTE DA WAVE: arraste PATTERN ANGLE ate' 0. As camadas tem de\n\
             [sculpt3d]       ficar DE PE' nos TRES ao mesmo tempo -- no quadro do painel, no\n\
             [sculpt3d]       TINTO sobre o barro, e no relevo que voce ja' esculpiu. Se um\n\
             [sculpt3d]       deles girar sozinho, ha' duas respostas para a mesma pergunta e a\n\
             [sculpt3d]       que mente e' a que voce esta' olhando -- PARE. Suba PATTERN TILT\n\
             [sculpt3d]       e elas se inclinam.\n\
             [sculpt3d]    3b) E o tinto SEGUE o barro: esculpa uma cova funda com o Draw e olhe\n\
             [sculpt3d]       o padrao dentro dela. Ele e' lido na POSICAO do vertice, entao ele\n\
             [sculpt3d]       acompanha a forma nova em vez de ficar pintado onde ela estava.\n\
             [sculpt3d]    4) O padrao continua colado ao ESPACO, nao ao gesto: passe DEVAGAR e\n\
             [sculpt3d]       depois RAPIDO pelo mesmo lugar, e passe de VOLTA -- as camadas caem\n\
             [sculpt3d]       nos MESMOS lugares. (Medido: um traco de 27 dabs guarda contraste\n\
             [sculpt3d]       0,285 contra 0,295 de um carimbo unico. Uma coordenada presa ao DAB\n\
             [sculpt3d]       perderia 57% -- foi essa medicao que decidiu o desenho.)\n\
             [sculpt3d]    5) SCRATCHES: riscos ESPARSOS, perpendiculares ao eixo. Ele cobre ~10%\n\
             [sculpt3d]       da superficie de proposito (um risco e' uma LINHA), entao o pincel\n\
             [sculpt3d]       parece fraco -- suba a Forca, como no Cracks.\n\
             [sculpt3d]    6) WEAVE: a trama, e uma das duas familias de fios corre AO LONGO do\n\
             [sculpt3d]       eixo. Gire o Pattern Angle e a trama gira junto.\n\
             [sculpt3d]    7) E o de sempre: pegue o Crease (0) com o Strata -- o vinco AFIA as\n\
             [sculpt3d]       camadas, porque o alpha entra na quinta potencia."
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
    if crate::sculpt3d::transform_scene() {
        // ⚠️ **A cena DECLARA o tamanho da BANDA**, e é ele que a torna válida:
        // a lei que esta wave corrige só é visível onde a máscara é PARCIAL, e
        // uma cena com a banda vazia deixaria o artista julgando uma
        // propriedade que ele não tem como ver.
        let (band, total) = crate::sculpt3d::soft_masked_counts();
        eprintln!(
            "[sculpt3d] =23 O TRANSFORM -- a mascara MOVE. DUAS esferas:\n\
             [sculpt3d]    a da DIREITA vem com mascara MACIA -- o polo sul pregado, o norte\n\
             [sculpt3d]    livre, e {band} de {total} vertices na BANDA entre os dois;\n\
             [sculpt3d]    a da ESQUERDA vem NUA. Se as duas nao aparecerem, PARE.\n\
             [sculpt3d]    Abra o painel com a CRASE (`) -- os tres botoes 'Transform Free Part'\n\
             [sculpt3d]    ficam no fim da secao Brush, logo abaixo do Extract.\n\
             [sculpt3d]    (1) Clique na esfera da DIREITA, aperte 'Rotate' e ARRASTE com o botao\n\
             [sculpt3d]        ESQUERDO em volta dela. O topo gira, o fundo NAO se mexe, e o meio\n\
             [sculpt3d]        torce suave. O botao DIREITO continua orbitando a camera.\n\
             [sculpt3d]    (1b) O QUE O SMOKE ANTERIOR REPROVOU: arraste em CIRCULO em volta da\n\
             [sculpt3d]        esfera e confira que o barro segue a mao -- mesmo SENTIDO, e volta\n\
             [sculpt3d]        por volta (uma volta do dedo = uma volta da peca). Faca de novo com\n\
             [sculpt3d]        o pen-down LONGE do centro e depois PERTO: tem de dar o mesmo. Se\n\
             [sculpt3d]        girar ao contrario, ou menos que a mao, PARE.\n\
             [sculpt3d]    (2) A PERGUNTA DA WAVE, e ela e' de OLHO: gire MEIA VOLTA ou mais. A\n\
             [sculpt3d]        cintura tem de continuar REDONDA -- ela torce, mas nao AFINA. Se a\n\
             [sculpt3d]        esfera pinçar em direcao ao eixo, como uma ampulheta, PARE: essa e'\n\
             [sculpt3d]        exatamente a lei que esta wave substituiu.\n\
             [sculpt3d]    (3) Ctrl+Z devolve a esfera em UM passo. Repita com 'Move' (a metade\n\
             [sculpt3d]        livre desliza e a pregada fica) e com 'Scale' (arrastar para longe\n\
             [sculpt3d]        do centro cresce, para perto encolhe).\n\
             [sculpt3d]    (4) Clique de novo no botao ACESO: ele desarma, e o esquerdo volta a\n\
             [sculpt3d]        esculpir. Sem isso a ferramenta ficaria presa na mao.\n\
             [sculpt3d]    (5) A METADE QUE PROVA A COSTURA: clique na esfera da ESQUERDA, pegue\n\
             [sculpt3d]        o verbo Mask (tecla M), BAIXE a forca para ~0,3 e pinte uma mancha\n\
             [sculpt3d]        de bordas MACIAS. Arme 'Rotate' e gire: o que voce pintou nao se\n\
             [sculpt3d]        move, o resto sim, e a transicao acompanha a suavidade da mancha.\n\
             [sculpt3d]    (6) Na esfera da esquerda: tecla C (limpa) e depois I (inverte) deixam\n\
             [sculpt3d]        a peca TODA protegida. Arme e arraste: o log tem de RECUSAR com\n\
             [sculpt3d]        uma frase, e nada pode se mover."
        );
    }
    if crate::sculpt3d::extract_scene() {
        // ⚠️ **A cena DECLARA quantos vértices ela mascarou**, e é esse número
        // que a torna válida: sem ele, *"apertei e nada saiu"* e *"apertei e
        // saiu uma casca de dois triângulos"* seriam a mesma queixa.
        eprintln!(
            "[sculpt3d] =22 O EXTRACT -- a mascara vira uma PECA. DUAS esferas:\n\
             [sculpt3d]    a da DIREITA ja' vem com uma calota mascarada ({} de {} vertices);\n\
             [sculpt3d]    a da ESQUERDA vem NUA. Se as duas nao aparecerem, PARE.\n\
             [sculpt3d]    Abra o painel com a CRASE (`) -- o botao 'Extract Mask' e os dois\n\
             [sculpt3d]    numeros dele ficam no fim da secao Brush, logo abaixo das quatro\n\
             [sculpt3d]    operacoes de mascara.\n\
             [sculpt3d]    (1) Clique na esfera da DIREITA e aperte 'Extract Mask'. O log diz o\n\
             [sculpt3d]        tamanho da peca nova. Ela nasce EM CIMA da calota -- e' uma casca,\n\
             [sculpt3d]        entao a silhueta da cena quase nao muda. Arraste-a para o lado\n\
             [sculpt3d]        (gizmo) e olhe: e' uma calota com ESPESSURA, fechada, sem furo na\n\
             [sculpt3d]        beira. Se ela aparecer VAZADA ou com a luz pelo avesso, PARE.\n\
             [sculpt3d]    (2) Ctrl+Z tira a peca. Ponha 'Extract Thickness' em ZERO e extraia de\n\
             [sculpt3d]        novo: agora sai uma FOLHA, uma casca de uma camada so'. Gire a\n\
             [sculpt3d]        camera: ela tem de ficar visivel dos dois lados.\n\
             [sculpt3d]    (3) Ctrl+Z. Ponha a espessura NEGATIVA (-0,15) e extraia: a casca\n\
             [sculpt3d]        cresce para DENTRO da esfera, nao para fora.\n\
             [sculpt3d]    (4) A METADE QUE PROVA A COSTURA: clique na esfera da ESQUERDA, pegue\n\
             [sculpt3d]        o verbo Mask (tecla M) e pinte uma mancha IRREGULAR, bem\n\
             [sculpt3d]        serrilhada de proposito. Extraia com 'Extract Smooth' em ZERO e\n\
             [sculpt3d]        olhe a beira: ela segue o serrilhado. Ctrl+Z, ponha o Smooth em 8\n\
             [sculpt3d]        e extraia de novo -- a beira tem de ficar CALMA, e a peca NAO pode\n\
             [sculpt3d]        encolher para dentro da mancha que voce pintou.\n\
             [sculpt3d]    (5) Extraia com a mascara LIMPA (tecla C na esfera da esquerda): o log\n\
             [sculpt3d]        tem de RECUSAR com uma frase, e nao criar uma peca vazia.",
            crate::sculpt3d::masked_dome_counts().0,
            crate::sculpt3d::masked_dome_counts().1,
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
    topology::for_scene(mesh);
    crate::sculpt3d::scenes::elastic::announce();
    crate::sculpt3d::scenes::strip::announce();
    crate::sculpt3d::scenes::thumb::announce();
    crate::sculpt3d::scenes::scrape::announce();
    crate::sculpt3d::scenes::surface::announce();
    crate::sculpt3d::scenes::layer::announce();
    crate::sculpt3d::scenes::filter::announce();
    crate::sculpt3d::scenes::quad::announce();
    crate::sculpt3d::scenes::ear::announce();
    if crate::sculpt3d::donation_scene() {
        eprintln!(
            "[sculpt3d] =2 A DOACAO: ha uma TELA BRANCA embaixo, e a tecla D alterna\n\
             [sculpt3d]    BARRO (esculpir) -> LUZ (a forma acende a tinta) -> DESLIGADA (o A/B)\n\
             [sculpt3d]    esculpa, aperte D, pegue o Painter e pinte CHAPADO: a tinta tem de sair ACESA\n\
             [sculpt3d]    aperte D de novo e a MESMA tinta fica plana -- e essa diferenca e a wave"
        );
    }
}

/// **OS ROTEIROS DA TOPOLOGIA** — ver [`topology`]. Filho, e o corte é o mesmo
/// que trouxe este arquivo à existência.
#[path = "sculpt3d_scripts_topology.rs"]
mod topology;
