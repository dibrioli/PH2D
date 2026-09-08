//! **A CENA DOS QUATRO VIEWPORTS** (`=38`) — ver a peça de quatro lados ao mesmo
//! tempo, e o gizmo que diz de que lado se está a olhar.
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
    crate::field3d_views::Standard::ALL
        .iter()
        .map(|v| ph2d_i18n::tr(v.key()))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// O roteiro da `=38`.
pub(crate) fn announce() {
    if !viewports_scene() {
        return;
    }
    let vistas = vistas();
    eprintln!(
        "[sculpt3d] =38 QUATRO VIEWPORTS -- ver a peca de quatro lados ao mesmo tempo.\n\
         [sculpt3d]    Ate' agora havia UMA janela 3D. Agora a area pode partir-se em quatro,\n\
         [sculpt3d]    cada uma com a sua camera: de cima, do lado, de frente, e a sua.\n\
         [sculpt3d]    Voce esculpe naquela em que clicar -- as outras tre^s so' mostram.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) No canto de cima a` direita ha' agora um GIZMO com seis bolinhas\n\
         [sculpt3d]        coloridas. Passe o rato por cima: a que esta' sob o cursor acende.\n\
         [sculpt3d]    (2) CLIQUE numa bolinha. A peca gira e passa a ser vista daquele lado.\n\
         [sculpt3d]        Agora ARRASTE a partir do gizmo (segure e mexa): ele gira a peca\n\
         [sculpt3d]        como se voce a estivesse a rodar com a mao.\n\
         [sculpt3d]    (3) Aperte Ctrl + a tecla da CRASE (`, a` esquerda do 1). A area parte-se\n\
         [sculpt3d]        em QUATRO. Tre^s delas tem um rotulo no canto ({vistas}).\n\
         [sculpt3d]    (4) Clique dentro de uma das quatro: ela ganha uma MOLDURA -- e' a activa.\n\
         [sculpt3d]        Pinte nela. So' ela recebe o pincel; as outras mostram a mesma peca\n\
         [sculpt3d]        a mudar, de outro angulo.\n\
         [sculpt3d]    (5) Leve o rato ate' a LINHA que separa as quatro. A seta muda. Arraste:\n\
         [sculpt3d]        as janelas mudam de tamanho.\n\
         [sculpt3d]    (6) Ctrl+crase outra vez fecha, e fica a janela em que voce estava.\n\
         [sculpt3d]\n\
         [sculpt3d]    Teclado: Numpad1 frente, Numpad3 direita, Numpad7 topo -- e com Ctrl\n\
         [sculpt3d]    cada uma da' o lado oposto.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO se: o pincel cair LONGE do cursor dentro de um quadrante;\n\
         [sculpt3d]    se pintar numa janela nao mudar a peca nas outras; se a moldura ficar\n\
         [sculpt3d]    numa janela e o pincel noutra; ou se a peca aparecer por baixo dos\n\
         [sculpt3d]    paineis / das reguas em vez de dentro da area do canvas."
    );
}
