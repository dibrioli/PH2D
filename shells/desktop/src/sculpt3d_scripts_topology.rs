//! **OS ROTEIROS DAS CENAS QUE MUDAM A MALHA** — tapar, reconstruir, reverter,
//! adensar, achatar.
//!
//! Módulo FILHO de [`super`] (`#[path]`), e o corte é por ASSUNTO, dentro do
//! corte que o irmão já tinha feito: lá mora *o que a cena manda fazer com o
//! barro* (pincel, padrão, sombreamento, doação), aqui *o que ela manda fazer
//! com a TOPOLOGIA*.
//!
//! ⚠️ **E o irmão previu este corte:** o cabeçalho dele diz que o teto de LOC o
//! forçou uma vez e que *"um arquivo que só cresce em roteiro ia cruzá-lo de
//! novo na cena seguinte"*. Cruzou — na cena do achatar. Cortar por número de
//! cena teria sido arbitrário; a família da topologia é um assunto, e a cena
//! nova dela nasce aqui.

/// Os roteiros das cenas de topologia. Chamada pelo [`super::for_scene`].
pub(crate) fn for_scene(mesh: &ph2d_mesh::Mesh) {
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

    if crate::sculpt3d::flatten_scene() {
        // ⚠️ A cena DECLARA quantos vértices a máscara cobre, e o número sai da
        // MESMA peça que ela monta: um roteiro que anuncia um número que a peça
        // não tem faz o artista procurar um defeito que não existe.
        let (masked, total) = crate::sculpt3d::flatten_scene_counts();
        eprintln!(
            "[sculpt3d] =26 O ACHATAR, e a MASCARA que ATRAVESSA a reconstrucao.\n\
             [sculpt3d]    Uma esfera GROSSA com metade mascarada: {masked} de {total} vertices,\n\
             [sculpt3d]    com a fronteira RETA no meio. Se a casca da mascara nao aparecer, PARE.\n\
             [sculpt3d]    Abra o painel com a CRASE (`), secao Topology.\n\
             [sculpt3d]    (1) Aperte V (Remesh). Ele RECONSTROI, a mascara continua la', e a\n\
             [sculpt3d]        fronteira continua RETA -- e' a metade B da wave. Ctrl+Z devolve.\n\
             [sculpt3d]    (2) Agora suba a pilha: 'Subdivide' DUAS vezes (o readout Level tem de\n\
             [sculpt3d]        dizer 2 / 2). Esculpa alguma coisa no topo, para haver detalhe.\n\
             [sculpt3d]    (3) Aperte V de novo. Ele RECUSA, e o log tem de dizer ACHATE a pilha\n\
             [sculpt3d]        antes -- NAO 'reverta'. Reverter deixa a pilha mais ALTA, e era\n\
             [sculpt3d]        isso que as cinco recusas mandavam fazer.\n\
             [sculpt3d]    (4) O BOTAO DA WAVE: 'Flatten Levels' aparece logo abaixo de Subdivide,\n\
             [sculpt3d]        e SO' com a pilha montada. Aperte. O Level volta a 0 / 0 e o barro\n\
             [sculpt3d]        na tela tem de continuar com O DETALHE que voce esculpiu la' em\n\
             [sculpt3d]        cima -- se ele voltar liso, o achatar ficou com a malha errada.\n\
             [sculpt3d]    (5) Aperte V. Agora ele reconstroi, e a mascara sobrevive de novo.\n\
             [sculpt3d]    (6) Ctrl+Z. A PILHA INTEIRA volta (Level 2 / 2, com o detalhe), e o\n\
             [sculpt3d]        botao Flatten reaparece. Se o Level voltar mas o detalhe nao,\n\
             [sculpt3d]        a entrada de desfazer perdeu um nivel."
        );
    }
}
