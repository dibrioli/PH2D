//! **A CENA DO FILTRO DE TECIDO** (`=37`) — o solver do pano na peça INTEIRA.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma nada, e isso é metade do smoke** — a mesma cicatriz que
//! o `impasto_smoke` do Painter 2D pregou e que as `=28`..`=32` herdaram: a wave
//! entrega uma fileira NOVA no painel, e uma cena que a escolhesse por baixo do
//! pano pularia justamente a costura que ela existe para provar.
//!
//! ⚠️ **E ela abre com a esfera de sempre, de propósito.** Uma folha plana
//! mostraria a gravidade melhor — e mostraria SÓ a gravidade: o aperto, o
//! expandir e a escala precisam de uma peça com volume para se lerem. *Uma
//! fixtura que favorece um dos cinco não julga os outros quatro.*

use ph2d_sculpt3d::{ClothFilterKind, ClothFilterOrientation};

/// `=37` — a cena do **FILTRO DE TECIDO**.
pub(crate) fn cloth_filter_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("37")
}

/// **Os nomes que o roteiro cita saem do MOTOR**, nunca de prosa.
///
/// ⚠️ *Um roteiro com o rótulo escrito à mão manda o artista procurar um botão
/// que já mudou de nome, e ninguém fica sabendo* — a mesma lei da `=30`.
fn rotulos() -> (String, String) {
    let tipos = ClothFilterKind::ALL
        .iter()
        .map(|k| k.label())
        .collect::<Vec<_>>()
        .join(" · ");
    let orient = ClothFilterOrientation::offered()
        .iter()
        .map(|o| o.label())
        .collect::<Vec<_>>()
        .join(" · ");
    (tipos, orient)
}

/// O roteiro da `=37`.
pub(crate) fn announce() {
    if !cloth_filter_scene() {
        return;
    }
    let (tipos, orient) = rotulos();
    eprintln!(
        "[sculpt3d] =37 O FILTRO DE TECIDO -- o pano aplicado a` PECA INTEIRA.\n\
         [sculpt3d]    O pincel de tecido amassa o barro onde voce passa. Este faz a MESMA\n\
         [sculpt3d]    fisica na peca toda de uma vez, sem encostar nela: voce arrasta na\n\
         [sculpt3d]    horizontal e a peca inteira responde, e continua a assentar enquanto\n\
         [sculpt3d]    voce segura -- e' uma simulacao a correr, nao um efeito aplicado.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel com a CRASE (`) e ligue o botao Filter.\n\
         [sculpt3d]    (2) Aparecem DUAS fileiras de leis. A de baixo chama-se Cloth:\n\
         [sculpt3d]        {tipos}. Escolha Gravity.\n\
         [sculpt3d]    (3) Aparece uma terceira fileira, Orientation ({orient}).\n\
         [sculpt3d]        Deixe como esta' por agora.\n\
         [sculpt3d]    (3b) LIGUE o modo Pro no painel. Aparecem os numeros do FILTRO --\n\
         [sculpt3d]        Filter Mass, Filter Damping, Filter Plasticity, Filter Quality --\n\
         [sculpt3d]        mais Filter Collisions. Eles sao do FILTRO, nao do pincel: aparecem\n\
         [sculpt3d]        com QUALQUER ferramenta na mao, que e' quando o filtro roda.\n\
         [sculpt3d]        Escolhendo Scale aparecem tambem tres travas Force Axis (X/Y/Z);\n\
         [sculpt3d]        nos outros quatro tipos elas somem, porque so' a Scale as le^.\n\
         [sculpt3d]    (3c) E aparecem DOIS controlos novos, no topo do bloco e sem precisar\n\
         [sculpt3d]        do modo Pro: Stretch Limit e Preserve Volume. Sao a resposta ao\n\
         [sculpt3d]        report de que o pano estica sem fim -- o passo (10) mede-os.\n\
         [sculpt3d]    (4) Clique na esfera e ARRASTE PARA A DIREITA, devagar, sem soltar.\n\
         [sculpt3d]        A peca INTEIRA tem de ceder e continuar a ceder enquanto a mao anda.\n\
         [sculpt3d]        Arraste de volta para a esquerda: ela e' empurrada para o outro lado.\n\
         [sculpt3d]        (Voltar ao ponto de partida NAO desfaz -- e' simulacao. Quem desfaz\n\
         [sculpt3d]         e' o Ctrl+Z, e ele desfaz o arrasto INTEIRO de uma vez so'.)\n\
         [sculpt3d]    (5) COMO SABER QUE DEU ERRADO: se o PRIMEIRO arrasto nao mover nada,\n\
         [sculpt3d]        ou se so' um circulo pequeno se mover em vez da peca toda, PARE\n\
         [sculpt3d]        e reporte.\n\
         [sculpt3d]    (6) Ctrl+Z. A peca tem de voltar inteira, num passo so'.\n\
         [sculpt3d]    (6b) A QUALIDADE -- o controlo que o Blender nao da'. Escolha Pinch e\n\
         [sculpt3d]        aperte com forca (arrasto longo). Com Filter Quality em 5 (o valor\n\
         [sculpt3d]        de fabrica, que e' o do Blender) o pano estica muito onde voce\n\
         [sculpt3d]        aperta. Ctrl+Z, ponha a Quality em 32 e repita o MESMO arrasto:\n\
         [sculpt3d]        o pano tem de esticar visivelmente MENOS. Medido: 4,9x menos.\n\
         [sculpt3d]        Custa tempo -- numa peca densa 32 fica lento, e a saida e' reduzir\n\
         [sculpt3d]        a malha (o botao de retopologia), nao baixar a qualidade.\n\
         [sculpt3d]    (7) A ORIENTACAO. Escolha View na terceira fileira, gire a camera com o\n\
         [sculpt3d]        botao DIREITO ate' olhar a peca de outro angulo, e arraste de novo:\n\
         [sculpt3d]        agora o 'baixo' e' o baixo do ECRA, seja qual for o angulo. Com\n\
         [sculpt3d]        Local, o baixo e' o da peca e gira com ela. Se os dois derem a mesma\n\
         [sculpt3d]        coisa depois de girar, reporte.\n\
         [sculpt3d]    (8) OS OUTROS QUATRO, o mesmo arrasto em cada um:\n\
         [sculpt3d]        Inflate -- a peca incha, e o pano resiste (nao e' uma esfera maior).\n\
         [sculpt3d]        Expand  -- o pano CRESCE e sobra: ele enruga em vez de inchar liso.\n\
         [sculpt3d]        Pinch   -- tudo e' puxado para o ponto ONDE VOCE CLICOU. Clique noutro\n\
         [sculpt3d]                   sitio e arraste: o aperto muda de lugar. O ponto NAO segue\n\
         [sculpt3d]                   o cursor durante o arrasto -- ele fica onde voce apertou.\n\
         [sculpt3d]        Scale   -- a peca cresce a partir do centro dela, com o pano a segurar.\n\
         [sculpt3d]    (9) A MISTURA. Ligue tambem os numeros do tecido no painel (Mass,\n\
         [sculpt3d]        Damping, Plasticity) e repita a gravidade: mais massa cede menos,\n\
         [sculpt3d]        mais amortecimento pa'ra mais depressa. Se mexer neles nao mudar\n\
         [sculpt3d]        nada, reporte.\n\
         [sculpt3d]    (10) O PANO DEIXA DE SER UM ELASTICO -- e' o report de 08/09.\n\
         [sculpt3d]        Monte a cena da foto: pegue a Mask e pinte TRES manchas pequenas\n\
         [sculpt3d]        perto do topo da esfera (elas ficam presas). Escolha Cloth >\n\
         [sculpt3d]        Gravity e arraste ate' ao fim, varias vezes.\n\
         [sculpt3d]        (a) Ponha Stretch Limit em 2,00 (o extremo elastico) e arraste: a\n\
         [sculpt3d]            esfera esvazia-se em tres tubos compridos, como na foto.\n\
         [sculpt3d]        (b) Ctrl+Z ate' voltar. Ponha Stretch Limit em 1,10 (o de fabrica)\n\
         [sculpt3d]            e repita o MESMO arrasto: o pano tem de TRANCAR -- os tubos\n\
         [sculpt3d]            ficam curtos e a superficie deixa de vir suja de manchas\n\
         [sculpt3d]            escuras. Medido: 4x menos esticao.\n\
         [sculpt3d]        (c) Ctrl+Z. Suba Preserve Volume de 0 para 1 e repita: agora a\n\
         [sculpt3d]            esfera nao se esvazia -- ela deforma mantendo o tamanho.\n\
         [sculpt3d]            Medido: sem ela sobram 44% do volume; com ela, 98%.\n\
         [sculpt3d]        (d) Onde ele muda MAIS: escolha Pinch e aperte com forca. Com\n\
         [sculpt3d]            Preserve Volume em 0 a peca IMPLODE (sobra 3% dela e a\n\
         [sculpt3d]            superficie fica rasgada); com 1, ela aperta e continua peca.\n\
         [sculpt3d]        (e) COMO SABER QUE DEU ERRADO: se 1,10 e 2,00 derem a mesma coisa,\n\
         [sculpt3d]            ou se o Preserve Volume nao mudar nada na Gravidade, reporte.\n\
         [sculpt3d]        ⚠️ ESPERADO, nao e' defeito: com Preserve Volume ligado a Scale e o\n\
         [sculpt3d]            Inflate quase nao fazem nada -- os dois existem para mudar o\n\
         [sculpt3d]            volume, e conservar o volume anula-os. Por isso ele nasce em 0.\n\
         [sculpt3d]        ⚠️ Ele so' funciona em peca FECHADA (sem buracos).\n\
         [sculpt3d]    ⚠️ A fileira de cima (Filter) sao as leis de MALHA, que ja' existiam.\n\
         [sculpt3d]        So' UMA das duas fileiras tem chip aceso de cada vez -- se as duas\n\
         [sculpt3d]        acenderem juntas, reporte."
    );
}
