//! **A CENA DO ESFREGÃO DE DESLOCAMENTO** (`=44`) — levar o relevo com a mão,
//! sem deformar a forma que está por baixo.
//!
//! # ⚠️⚠️ Ela abre SEM pilha, e o 1.º passo é a RECUSA — irmã da `=43`
//!
//! Este pincel só existe onde há **multiresolução**, pela mesma razão do
//! apagador: sem um nível abaixo não há deslocamento nenhum para transportar
//! (espec §5.6). ⇒ o roteiro começa por pedir que o use **sem** pilha, porque o
//! produto **diz-lhe porquê** numa linha — e é onde ele aprende de que família
//! o pincel é.
//!
//! # ⭐⭐ E o roteiro tem de ENSINAR a diferença que dá nome à ferramenta
//!
//! *Esfregar não é empurrar.* Um artista que nunca viu este pincel espera que
//! ele arraste a **superfície** — e ele arrasta a **pele**: a forma grande fica
//! exactamente onde estava, e o relevo viaja por cima dela. ⇒ o passo que o
//! prova é **descer de nível**: a forma de baixo não se mexeu.
//!
//! ⚠️ **E o passo do cursor PARADO não é uma curiosidade — é a lei** (§5.3):
//! com a mão parada o arrasto fica **inerte ao bit**, e o artista precisa de o
//! ver uma vez para não o ler como uma ferramenta partida.

/// `=44` — a cena do **ESFREGÃO DE DESLOCAMENTO**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `43`).
pub(crate) fn smear_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("44")
}

/// O roteiro da `=44`.
pub(crate) fn announce() {
    if !smear_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =44 O ESFREGAO DE DESLOCAMENTO -- levar o relevo com a mao\n\
         [sculpt3d]    Este pincel NAO empurra a superficie: ele empurra a PELE por cima\n\
         [sculpt3d]    dela. A forma grande fica onde esta'; o relevo e' que viaja.\n\
         [sculpt3d]    Como o apagador, ele so' existe com MULTIRESOLUCAO.\n\
         [sculpt3d]\n\
         [sculpt3d]    Abra o painel com a CRASE (`). O pincel chama-se `Smear Displacement`\n\
         [sculpt3d]    e esta' no FIM da fileira.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha-o AGORA, antes de mais nada, e arraste sobre a bola.\n\
         [sculpt3d]        -> Nada acontece, E O LOG DIZ PORQUE: esta peca tem um nivel so'.\n\
         [sculpt3d]           Isso e' o certo.\n\
         [sculpt3d]    (2) Aperte K duas vezes (subdividir).\n\
         [sculpt3d]    (3) Escolha `Draw` (tecla 1) e faca DUAS OU TRES BOSSAS bem visiveis,\n\
         [sculpt3d]        separadas, num lado da bola.\n\
         [sculpt3d]    (4) Volte a `Smear Displacement` e ARRASTE sobre as bossas, de um lado\n\
         [sculpt3d]        para o outro, como quem espalha tinta com o dedo.\n\
         [sculpt3d]        -> As bossas ANDAM na direcao em que voce arrasta, e deixam um\n\
         [sculpt3d]           rasto. A bola continua do mesmo tamanho.\n\
         [sculpt3d]    (5) Pare o dedo em cima de uma bossa e mantenha o botao carregado\n\
         [sculpt3d]        SEM MEXER.\n\
         [sculpt3d]        -> Nao acontece NADA, e e' a lei: sem movimento nao ha' direcao\n\
         [sculpt3d]           para onde empurrar.\n\
         [sculpt3d]    (6) No painel, troque `Deformation` de `Drag` para `Pinch` e carregue\n\
         [sculpt3d]        (mesmo parado) sobre uma bossa.\n\
         [sculpt3d]        -> Agora acontece: o relevo ADENSA para o centro do circulo, sem\n\
         [sculpt3d]           apertar a malha. Troque para `Expand` e ele ESPALHA para fora.\n\
         [sculpt3d]           Estes dois nao precisam de movimento -- a direcao deles vem da\n\
         [sculpt3d]           geometria.\n\
         [sculpt3d]    (7) Baixe o `Strength` para metade: o efeito fica um QUARTO, nao\n\
         [sculpt3d]        metade. E' de proposito.\n\
         [sculpt3d]    (8) Segure Ctrl e arraste -> nao muda NADA. Para esfregar ao contrario,\n\
         [sculpt3d]        arraste para o outro lado.\n\
         [sculpt3d]    (9) Ctrl+Z desfaz cada passagem.\n\
         [sculpt3d]\n\
         [sculpt3d]    E A PROVA DE QUE ELE NAO DEFORMA A FORMA: com o relevo espalhado,\n\
         [sculpt3d]    aperte a virgula (,) para descer ao nivel de baixo. A forma la' em\n\
         [sculpt3d]    baixo tem de estar INTACTA -- ele nunca lhe tocou. O ponto (.) sobe\n\
         [sculpt3d]    outra vez.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: no (1) nao aparecer a linha a dizer porque; se no (4)\n\
         [sculpt3d]    a bola ENCOLHER, inchar ou a silhueta grande mudar; se no (5) alguma\n\
         [sculpt3d]    coisa se mexer com o dedo parado; ou se o Ctrl mudar alguma coisa.\n\
         [sculpt3d]\n\
         [sculpt3d]    NA BORDA DO CIRCULO o relevo esmorece um pouco -- isso e' CONHECIDO e\n\
         [sculpt3d]    e' o mesmo comportamento da ferramenta de referencia, nao um defeito."
    );
}

#[cfg(test)]
mod tests {
    /// ⛔ **O gate que as vizinhas `=39`/`=41` pagaram para existir:** duas cenas
    /// a reclamar o mesmo número deixam a segunda **inalcançável e muda**.
    #[test]
    fn a_cena_reclama_o_nivel_que_o_roteador_declara() {
        const {
            assert!(
                crate::scenes::CENAS >= 44,
                "o tecto do roteador tem de conter esta cena (=44)"
            );
        }
    }
}
