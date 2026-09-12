//! **A CENA DAS TRÊS FEATURES DO MODELADOR** (`=38`) — o gizmo de canto, as
//! quatro janelas e as alças de transformar, que o módulo de modelagem 3D já
//! tinha e a escultura não (ordem do Enio, 2026-09-08).
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO abre a divisão, e isso é metade do smoke** — a mesma cicatriz
//! que as `=28`..`=32` e a `=37` herdaram: a wave entrega um GESTO novo, e uma
//! cena que o executasse por baixo do pano pularia justamente a costura que ela
//! existe para provar. O artista abre com a mão e vê acontecer.
//!
//! ⚠️ **E ela abre com a esfera de sempre.** Uma peça assimétrica leria melhor
//! *«esta é a vista de cima»* — e leria PIOR o resto: com quatro imagens
//! diferentes na tela é a **moldura do quadrante activo** que se tem de ver, e
//! ela é a coisa que este smoke julga.

/// `=38` — a cena dos **QUATRO VIEWPORTS**.
pub(crate) fn viewports_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("38")
}

/// **Os nomes que o roteiro cita saem do MOTOR**, nunca de prosa.
///
/// ⚠️ *Um roteiro com o rótulo escrito à mão manda o artista procurar um botão
/// que já mudou de nome, e ninguém fica sabendo* — a mesma lei da `=30` e da
/// `=37`.
fn vistas() -> String {
    ph2d_viewport3d::views::Standard::ALL
        .iter()
        .map(|v| ph2d_i18n::tr(v.key()))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Os nomes dos três verbos do transform, do MOTOR.
fn verbos() -> String {
    ph2d_sculpt3d::TransformKind::ALL
        .iter()
        .map(|k| k.label())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// O roteiro da `=38`.
pub(crate) fn announce() {
    if !viewports_scene() {
        return;
    }
    let (vistas, verbos) = (vistas(), verbos());
    eprintln!(
        "[sculpt3d] =38 AS TRES COISAS QUE O MODELADOR 3D JA' TINHA -- agora na escultura.\n\
         [sculpt3d]    (a) o gizmo de canto que diz de que lado voce olha, (b) quatro janelas\n\
         [sculpt3d]    ao mesmo tempo, e (c) as alcas de mover/girar/escalar sobre a peca.\n\
         [sculpt3d]\n\
         [sculpt3d]    -- (a) O GIZMO DE CANTO --\n\
         [sculpt3d]    (1) No canto de cima a` direita ha' seis bolinhas coloridas. Passe o rato\n\
         [sculpt3d]        por cima: a que esta' sob o cursor acende.\n\
         [sculpt3d]    (2) CLIQUE numa. A peca gira e passa a ser vista daquele lado.\n\
         [sculpt3d]    (3) Agora SEGURE em cima do gizmo e mexa: ele gira a peca como se voce\n\
         [sculpt3d]        a rodasse com a mao.\n\
         [sculpt3d]\n\
         [sculpt3d]    -- (b) AS QUATRO JANELAS --\n\
         [sculpt3d]    (4) Aperte Ctrl+Alt+Q (a mesma tecla do modelador 3D). A area parte-se\n\
         [sculpt3d]        em QUATRO. Tres delas tem um rotulo no canto ({vistas}).\n\
         [sculpt3d]    (5) Clique dentro de uma: ela ganha uma MOLDURA -- e' a activa. Pinte nela.\n\
         [sculpt3d]        So' ela recebe o pincel; as outras mostram a mesma peca de outro angulo.\n\
         [sculpt3d]    (6) Leve o rato ate' a LINHA que separa as janelas. A seta muda. Arraste:\n\
         [sculpt3d]        elas mudam de tamanho. Ctrl+Alt+Q outra vez fecha, e fica a que voce\n\
         [sculpt3d]        estava a usar.\n\
         [sculpt3d]\n\
         [sculpt3d]    -- (c) AS ALCAS DE TRANSFORMAR --\n\
         [sculpt3d]    (7) Pinte uma MASCARA numa parte da peca (a tecla C pinta, I inverte):\n\
         [sculpt3d]        o que fica protegido nao se mexe, e as alcas movem o RESTO.\n\
         [sculpt3d]    (8) Abra o painel com a CRASE (`) e ligue um dos tres: {verbos}.\n\
         [sculpt3d]        Aparecem alcas sobre a peca -- setas coloridas (mover), aneis (girar)\n\
         [sculpt3d]        ou um punho (tamanho).\n\
         [sculpt3d]    (9) ARRASTE UMA SETA: a peca anda SO' naquela direccao. Arraste um dos\n\
         [sculpt3d]        quadradinhos entre duas setas: ela anda so' naquele plano. Arraste um\n\
         [sculpt3d]        anel: ela gira so' em torno daquele eixo.\n\
         [sculpt3d]   (10) Arraste SOBRE a peca, longe das alcas: continua a valer o de sempre\n\
         [sculpt3d]        (mover no plano da tela). E arraste no VAZIO, fora da peca: a\n\
         [sculpt3d]        camera gira, como sem ferramenta nenhuma na mao.\n\
         [sculpt3d]\n\
         [sculpt3d]    Teclado: Numpad1 frente, Numpad3 direita, Numpad7 topo -- com Ctrl, o\n\
         [sculpt3d]    lado oposto de cada uma.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO se: o pincel cair LONGE do cursor dentro de um quadrante; se\n\
         [sculpt3d]    pintar numa janela nao mudar a peca nas outras; se a moldura ficar numa\n\
         [sculpt3d]    janela e o pincel noutra; se uma alca agarrar ao lado de onde ela esta'\n\
         [sculpt3d]    desenhada; se um anel do lado de tras girar ao contrario do seu dedo;\n\
         [sculpt3d]    se armar uma ferramenta de transformar tirar a rotacao da camera;\n\
         [sculpt3d]    ou se a peca aparecer por baixo dos paineis / das reguas em vez de\n\
         [sculpt3d]    dentro da area do canvas."
    );
}
