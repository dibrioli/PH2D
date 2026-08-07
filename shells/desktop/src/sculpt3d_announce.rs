//! **O QUE A CENA DIZ AO ARTISTA** — o manual que o smoke imprime.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e ele é o que ele parece:** o irmão diz *que
//! cena é esta e que malha ela abre*; aqui fica *o que o artista precisa saber
//! para julgá-la*. A separação existe porque esta metade **cresce a cada wave** —
//! todo verbo, todo canal e todo botão novo ganham uma linha aqui — enquanto a
//! outra só ganha uma cena.
//!
//! A regra que o texto obedece: **a cena IMPRIME o que montou**. Um smoke que
//! não se declara deixa o artista sem saber se está vendo a feature ou o app
//! vazio — a lição que o smoke do Colorize pagou.

use super::scenes::scripts;

/// **A cena DECLARA o que montou** — o banner e as instruções de cada uma.
///
/// ⚠️ Mora aqui, ao lado da fixture, e não no arquivo do gesto: *que malha esta
/// cena monta* e *o que ela pede ao artista para julgar* são a mesma pergunta,
/// e mantê-las separadas foi o que deixou uma cena declarar um número que a
/// outra metade não produzia. O gesto ficou com o gesto.
///
/// ⚠️ E declarar não é cortesia: um smoke que não diz o que montou é
/// indistinguível da feature quebrada — a lição que o smoke do Colorize pagou, e
/// que as cenas `=4` e `=6` pagam de novo com um NÚMERO (a beira, a aresta).
pub(crate) fn announce(mesh: &ph2d_mesh::Mesh) {
    // A cena IMPRIME o que montou. Um smoke que não se declara deixa o
    // artista sem saber se está vendo a feature ou o app vazio — a lição
    // que o smoke do Colorize pagou.
    eprintln!(
        "[sculpt3d] malha com {} vértices / {} faces / {} triângulos\n\
         [sculpt3d] ESQUERDO esculpe (fora do modelo, gira) · DIREITO gira · MEIO desloca · RODA aproxima\n\
         [sculpt3d] Shift = Smooth enquanto segurar · Ctrl inverte Draw/Inflate/Clay/Crease e limpa a mascara\n\
         [sculpt3d] 1..9,0 escolhem o verbo · A alarga (magnify) · M mascara · [ ] tamanho · X/Y/Z espelho · Ctrl+Z desfaz\n\
         [sculpt3d] o pincel mede PIXELS DE TELA: aproxime com a roda e ele continua do mesmo tamanho\n\
         [sculpt3d] a MASCARA (M) protege o que ela pinta e se VE (azul frio): C limpa · I inverte · B borra · N afia\n\
         [sculpt3d] K = SUBDIVIDIR: 4 faces onde havia 1, e a forma ALISA (Catmull-Clark/Loop)\n\
         [sculpt3d]     o log diz a contagem nova a cada toque -- ela quadruplica; Ctrl+Z desfaz\n\
         [sculpt3d] , e . DESCEM e SOBEM na pilha de niveis: esculpa fino em cima, volte ao 0\n\
         [sculpt3d]     para mover a FORMA GRANDE, e suba -- o detalhe fino continua la'\n\
         [sculpt3d] J = DES-SUBDIVIDIR: reconstroi um nivel ABAIXO da base (o par do K)\n\
         [sculpt3d]     so' funciona se a malha JA' for uma subdivisao -- o log diz quando nao e'\n\
         [sculpt3d] O = TAPAR BURACO: todo contorno aberto ganha uma tampa (e o log diz quantos)\n\
         [sculpt3d] V = RECONSTRUIR (voxel remesh): a malha vira um campo e volta com densidade\n\
         [sculpt3d]     UNIFORME -- e' o que devolve barro onde um estica'o o gastou; a forma fica\n\
         [sculpt3d] G = PEGAR o barro (grab): segure e arraste, e ele vem com o dedo\n\
         [sculpt3d] H = ESTICAR (snake hook): a pegada ANDA com o cursor e sai um espinho\n\
         [sculpt3d]     o G volta ao lugar quando voce volta; o H deixa a ponta la' -- essa e' a diferenca\n\
         [sculpt3d] T = TORCER (twist): segure e VARRA um circulo em volta do ponto que voce pegou\n\
         [sculpt3d] S = INFLAR/ENCOLHER (local scale): segure e arraste na HORIZONTAL\n\
         [sculpt3d]     os dois voltam ao lugar quando voce varre de volta -- o gesto e' o TOTAL, nao a soma\n\
         [sculpt3d] A LUZ e o rig do artista (o mesmo que acende a tinta): Q/E giram a lampada, R/F a sobem\n\
         [sculpt3d] o espelho nasce DESLIGADO; PH2D_SCULPT3D_DIAG=1 mede se o pincel cai sob o cursor\n\
         [sculpt3d] --- O PAINEL (W12) ---\n\
         [sculpt3d] ele abre com a cena, e a CRASE (`) o fecha e o reabre\n\
         [sculpt3d] TOOL (os 16 verbos) · BRUSH (raio, forca, falloff, mascara) · SYMMETRY\n\
         [sculpt3d] TOPOLOGY (dyntopo, detalhe, niveis, remesh, tapar) · SHADING · SCENE\n\
         [sculpt3d] a CAVIDADE e' o slider da secao SHADING: 0 e' o barro liso, 1 o teto\n\
         [sculpt3d] MATERIAL (SHADING): 'Rig' e' a luz do DOCUMENTO; os outros seis sao MATCAPS --\n\
         [sculpt3d]     luz do OLHO, que nao gira com o modelo. Sob um matcap as duas pistas de\n\
         [sculpt3d]     lampada SOMEM, porque ele nao le o rig -- e isso e' o certo, nao um bug\n\
         [sculpt3d] ACCUMULATE (BRUSH): desarmado, cruzar o proprio traco NAO intensifica --\n\
         [sculpt3d]     e' a lei do envelope, e uma pincelada deposita no maximo a forca do\n\
         [sculpt3d]     pincel. Armado, passar duas vezes soma duas vezes. Ele so' aparece nos\n\
         [sculpt3d]     verbos de CARIMBO: quem tem ancora (G/H/T/S) carrega o gesto TOTAL\n\
         [sculpt3d]     desde o pen-down, e somar totais nao significa nada\n\
         [sculpt3d]     ATENCAO: a PRIMEIRA passada acumulada e' mais FRACA (a lei entrega a\n\
         [sculpt3d]     media do falloff, nao o pico); e' da segunda em diante que ela paga\n\
         [sculpt3d] ALPHA (BRUSH, logo abaixo do Falloff): o PADRAO que decide onde, dentro da\n\
         [sculpt3d]     pegada, o verbo age -- None e' o pincel liso; os seis sao Noise, Pores,\n\
         [sculpt3d]     Scales, Cracks, Grain e Ridges. Ele esta' colado ao ESPACO, nao ao gesto:\n\
         [sculpt3d]     passar devagar ou rapido, de ida ou de volta, poe a textura no MESMO lugar\n\
         [sculpt3d]     ALPHA SCALE so' aparece com um padrao armado, e mede a feature em unidades\n\
         [sculpt3d]     de OBJETO -- ela precisa de ~10 arestas para ser textura em vez de\n\
         [sculpt3d]     chuvisco, entao SUBDIVIDA (K) antes de baixar a escala. A cena =16 abre\n\
         [sculpt3d]     densa de proposito e imprime a razao que ela conseguiu\n\
         [sculpt3d] WIREFRAME (SHADING): a malha por cima da forma -- e' o que mostra onde o remesh\n\
         [sculpt3d]     pos os aneis e ate' onde o refino chegou; ela some e volta sem custo com\n\
         [sculpt3d]     a caixa desmarcada (a lista de arestas so' existe com ela armada)\n\
         [sculpt3d] AMBIENT OCCLUSION (SHADING): o quanto do CEU cada vertice enxerga --\n\
         [sculpt3d]     a fresta funda escurece porque ela ve pouco ceu, nao porque a luz mudou.\n\
         [sculpt3d]     Ele e' ASSADO sob comando (botao 'Bake AO'), e nao acompanha o traco: o\n\
         [sculpt3d]     bake custa ~338 ms na malha da =16 (campo 301 + traco 37, MEDIDO), entao\n\
         [sculpt3d]     um passe por pincelada gastaria um terco de segundo para produzir um dado\n\
         [sculpt3d]     que a pincelada seguinte invalida\n\
         [sculpt3d]     ORDEM: aperte 'Bake AO' e SO' ENTAO suba o slider -- ele nasce em ZERO, e\n\
         [sculpt3d]     sem bake ele e' inerte AO BYTE (o canal ausente e' ceu aberto em todo lado)\n\
         [sculpt3d]     ⚠️ DEPOIS DE ESCULPIR o painel avisa 'AO describes the previous shape' --\n\
         [sculpt3d]     o numero fica VELHO e nao parece velho, entao ele e' DITO. Asse de novo\n\
         [sculpt3d]     A CENA =17 e' um TORO: o aro de DENTRO tem de escurecer e o de FORA nao\n\
         [sculpt3d] 'Screen Occlusion' e' o OUTRO AO, e ele nasce LIGADO: medido a cada frame\n\
         [sculpt3d]     a partir da PROFUNDIDADE e das NORMAIS da tela (GTAO), entao ele nunca\n\
         [sculpt3d]     fica velho -- esculpa uma cratera e ela escurece na hora, sem botao\n\
         [sculpt3d]     ⚠️ Ele NAO substitui o assado: so' ve o que esta' na TELA (o assado ve o\n\
         [sculpt3d]     corpo inteiro em qualquer direcao, viaja no arquivo e vai para a doacao\n\
         [sculpt3d]     ao 2D). Um ve a sombra ENTRE duas pecas, o outro ve dentro de uma so'.\n\
         [sculpt3d]     Ligados os dois, eles compoem pela MAIS ESCURA -- nunca multiplicando\n\
         [sculpt3d]     Custa 0,41 ms/frame a 1920x1080 (2,4% de um quadro de 60 fps), MEDIDO\n\
         [sculpt3d]     A CENA =18 sao duas esferas ENCOSTADAS: a fresta entre elas e' o que\n\
         [sculpt3d]     SO' este passe consegue medir\n\
         [sculpt3d] o ANEL do cursor e' desenhado NO PONTO DE ACERTO -- se ele nao estiver\n\
         [sculpt3d]     debaixo do mouse sobre o barro, o pick esta' errado e da' para VER",
        mesh.vert_count(),
        mesh.face_count(),
        mesh.triangle_count()
    );
    scripts::for_scene(mesh);
}
