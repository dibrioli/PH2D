//! **A CENA DO APAGADOR DE DESLOCAMENTO** (`=43`) — desfazer a escultura fina
//! sem tocar na forma grande.
//!
//! # ⚠️⚠️ Ela abre SEM pilha de propósito, e o 1.º passo é a RECUSA
//!
//! Este pincel só existe onde há **multiresolução**: sem um nível abaixo não há
//! deslocamento a apagar, e o dado de entrada **não existe** (espec §4.3). ⇒ a
//! cena podia abrir com a pilha pronta — e escondia do artista a metade que ele
//! mais precisa de aprender: *que este pincel pertence a uma família, e qual*.
//!
//! ⭐ O roteiro começa por pedir-lhe que o use **sem** pilha, porque o produto
//! **diz-lhe porquê** numa linha. É a mesma lei que a densidade pagou em 14/09:
//! *um verbo cujo efeito depende de um estado parece partido enquanto esse
//! estado não existir, e a cura é ele falar.*
//!
//! ⛔ **E o irmão-filtro do alvo ESTOIROU publicamente por não fazer esta
//! verificação** — é por isso que o gate desta fronteira é a **recusa**, não o
//! resultado.

/// `=43` — a cena do **APAGADOR DE DESLOCAMENTO**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `42`).
pub(crate) fn erase_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("43")
}

/// O roteiro da `=43`.
pub(crate) fn announce() {
    if !erase_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =43 O APAGADOR DE DESLOCAMENTO -- tirar a escultura fina, e a forma fica\n\
         [sculpt3d]    Este pincel so' faz sentido com MULTIRESOLUCAO: uma peca com niveis,\n\
         [sculpt3d]    onde a forma grande vive em baixo e a pele fina vive em cima. Ele\n\
         [sculpt3d]    apaga a PELE e nao toca na FORMA.\n\
         [sculpt3d]\n\
         [sculpt3d]    Abra o painel com a CRASE (`). O pincel chama-se `Erase Displacement`\n\
         [sculpt3d]    e esta' no FIM da fileira.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha-o AGORA, antes de mais nada, e carregue na bola.\n\
         [sculpt3d]        -> Nada acontece, E O LOG DIZ PORQUE: esta peca tem um nivel so',\n\
         [sculpt3d]           logo nao ha' deslocamento nenhum a apagar. Isso e' o certo.\n\
         [sculpt3d]    (2) Aperte K duas vezes (subdividir). O log diz quantos vertices ficaram.\n\
         [sculpt3d]    (3) Escolha `Draw` (tecla 1) e esculpa umas bossas bem visiveis num\n\
         [sculpt3d]        lado da bola. Gire e olhe: a silhueta agora e' irregular.\n\
         [sculpt3d]    (4) Volte a `Erase Displacement` e passe POR CIMA das bossas, devagar.\n\
         [sculpt3d]        -> Elas DERRETEM de volta para a superficie lisa, so' onde voce\n\
         [sculpt3d]           passa. A bola continua do mesmo tamanho.\n\
         [sculpt3d]        -> Passe de novo no mesmo sitio: ele nao vai ALEM do liso. Nao ha'\n\
         [sculpt3d]           `apagar demais`.\n\
         [sculpt3d]    (5) Baixe o `Strength` para metade e passe numa bossa nova.\n\
         [sculpt3d]        -> Ela so' derrete um QUARTO do caminho, nao metade. E' de\n\
         [sculpt3d]           proposito: a metade de baixo do slider ganha resolucao.\n\
         [sculpt3d]    (6) Segure Ctrl e passe.\n\
         [sculpt3d]        -> Nao muda NADA, e e' a lei: apagar tem um so' sentido.\n\
         [sculpt3d]    (7) Ctrl+Z desfaz cada passagem.\n\
         [sculpt3d]\n\
         [sculpt3d]    E A PROVA DE QUE ELE NAO COME A FORMA: com as bossas apagadas, aperte\n\
         [sculpt3d]    a virgula (,) para descer ao nivel de baixo e depois o ponto (.) para\n\
         [sculpt3d]    subir. A bola tem de ficar do mesmo tamanho -- ele repoe a pele na\n\
         [sculpt3d]    superficie que a subdivisao de facto descreve, e nao numa mais\n\
         [sculpt3d]    apertada. (Se ele encolhesse a peca a cada passagem, seria outro\n\
         [sculpt3d]    pincel: o erro vale 11 % da forma na quina de um cubo.)\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: no (1) nao aparecer a linha a dizer porque; se no (4)\n\
         [sculpt3d]    a bola ENCOLHER ou a forma grande mudar; se o pincel passar do liso e\n\
         [sculpt3d]    cavar; ou se o Ctrl mudar alguma coisa."
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
                crate::scenes::CENAS >= 43,
                "o tecto do roteador tem de conter esta cena (=43)"
            );
        }
    }
}
