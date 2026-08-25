//! **A CENA DA RETOPOLOGIA** (`=35`) — a Q5 do ADR-0160, e a primeira vez que o
//! quad remesh é alcançável por um gesto.
//!
//! ⚠️ **A malha abre RUGOSA de propósito.** Sobre uma esfera lisa as duas
//! reconstruções — o voxel remesh e esta — devolvem coisas parecidas, e o smoke
//! não teria como as separar. O que distingue uma retopologia é a grade correr
//! **ao longo da forma**: é preciso haver forma.

/// `=35` — a cena da **RETOPOLOGIA**.
pub(crate) fn quad_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("35")
}

/// O roteiro da `=35`.
pub(crate) fn announce() {
    if !quad_scene() {
        return;
    }
    // ⭐⭐⭐ **O MESMO BOTÃO, OUTRO MOTOR ⇒ OUTRO ROTEIRO.**
    //
    // ⛔⛔ **O roteiro de baixo MENTE sobre o caminho novo, e mente no passo que
    // manda parar.** O passo (5) dele diz *"a peça tem de continuar fechada nas
    // duas pontas; se esburacar, PARE — é o defeito de 19/08 a voltar"*. Com
    // `PH2D_RETOPO_EXTRACT=1` a peça **vai** esburacar, por um defeito **medido e
    // registado** (o handoff de 24/08 §8-bis): a costura abre uma célula inteira
    // porque o solver a **pesa** em vez de a eliminar.
    //
    // ⇒ *Um smoke que manda reportar como regressão aquilo que já está medido
    // gasta o Enio duas vezes: uma a olhar, outra a escrever o report.* E o que
    // ele tem para julgar aqui é **outra** pergunta — a troca entre forma e casca.
    if crate::sculpt3d::history::retopo_extract::extract_requested() {
        announce_extract();
        return;
    }
    eprintln!(
        "[sculpt3d] =35 A RETOPOLOGIA (o quad remesh por campo cruzado, ADR-0160).\n\
         [sculpt3d]    Ela NAO substitui o Remesh: aquele re-amostra um campo de voxels e os\n\
         [sculpt3d]    quads dele seguem os EIXOS da grade; esta preserva a topologia e poe a\n\
         [sculpt3d]    grade a correr AO LONGO da forma.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e ache a secao Topology.\n\
         [sculpt3d]    (1) O BOTAO. Ele chama-se `Quad Retopology` e fica logo abaixo do\n\
         [sculpt3d]        `Remesh`. Se nao estiver la', PARE -- o resto nao diz nada.\n\
         [sculpt3d]    (2) CLIQUE. O terminal imprime quantos vertices, quantos quads e quantos\n\
         [sculpt3d]        nao-quads sairam, e em quantos ms. Medido nesta cena: a esmagadora\n\
         [sculpt3d]        maioria das faces sai com QUATRO lados.\n\
         [sculpt3d]    (3) OLHE A GRADE. As linhas tem de seguir os vincos da peca, e nao os\n\
         [sculpt3d]        eixos do mundo. Compare com o `Remesh` (desfaca antes): la' as linhas\n\
         [sculpt3d]        sobem e descem em ESCADA sobre uma feicao diagonal.\n\
         [sculpt3d]    (4) O Ctrl+Z DESFAZ, e devolve a malha inteira de antes.\n\
         [sculpt3d]    (5) O `Detail` vai de 0 (a grade mais GROSSA que ainda descreve a peca) a\n\
         [sculpt3d]        1 (a mais FINA que esta malha consegue resolver). ⚠️ ARRASTE-O DE UMA\n\
         [sculpt3d]        PONTA A OUTRA e clique em cada ponta: a peca tem de continuar fechada\n\
         [sculpt3d]        e inteira nas DUAS. Se em alguma ponta ela sumir, esburacar ou ficar\n\
         [sculpt3d]        espetada, PARE -- e' o defeito de 19/08 a voltar.\n\
         [sculpt3d]    (6) O `Follow Curvature` em 1,0 poe quadrados MENORES onde a forma aperta\n\
         [sculpt3d]        -- compare com ele em 0,0.\n\
         [sculpt3d]    (7) ⚠️ Com a pilha de multires montada ela RECUSA e diz para achatar antes."
    );
}

/// ⭐ **O ROTEIRO DO CAMINHO NOVO** (`PH2D_RETOPO_EXTRACT=1`).
///
/// ⚠️ **Ele pede uma coisa só, e não é «funciona?»** — é o **julgamento de uma
/// troca**: este motor dá quads melhores e uma casca que não fecha; o de sempre
/// fecha a casca e dá quads piores. Só o dono do produto decide qual vale mais, e
/// ele não pode decidir sem ver os dois. ⇒ o passo (4) é o roteiro **inteiro**
/// dessa comparação, e não uma nota de rodapé.
///
/// ⛔ **O que este roteiro NÃO faz:** mandar arrastar o `Detail` à procura de casca
/// fechada. Aquele passo é do caminho de sempre e aqui reprovaria por desenho.
fn announce_extract() {
    eprintln!(
        "[sculpt3d] =35 A RETOPOLOGIA -- CAMINHO NOVO (PH2D_RETOPO_EXTRACT esta' ligado).\n\
         [sculpt3d]    O botao e' o mesmo `Quad Retopology`; o motor por tras dele e' outro.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e ache a secao Topology.\n\
         [sculpt3d]    (1) CLIQUE em `Quad Retopology`. ⚠️ A janela FICA PARADA alguns\n\
         [sculpt3d]        segundos -- e' o solver a correr, nao e' um travamento. Se passar de\n\
         [sculpt3d]        um minuto, ai' sim PARE e diga.\n\
         [sculpt3d]    (2) NO TERMINAL, procure `casca FECHADA`. ⭐ Desde 24/08 ela FECHA: a\n\
         [sculpt3d]        costura deixou de ser negociada e passou a ser imposta. Se em vez\n\
         [sculpt3d]        disso aparecer `N BURACO(S) na casca`, ⚠️ ISSO E' REGRESSAO e eu\n\
         [sculpt3d]        preciso do report -- ao contrario do que este roteiro dizia ate' 24/08.\n\
         [sculpt3d]    (3) OLHE OS QUADRADOS. Eles tem de parecer QUADRADOS -- nao losangos,\n\
         [sculpt3d]        nao tiras compridas. No terminal, `enviesamento X/Y graus`: o X e' o\n\
         [sculpt3d]        tipico, e menor e' melhor. Abaixo de 7 e' o nivel do melhor programa\n\
         [sculpt3d]        que existe.\n\
         [sculpt3d]    (4) ⭐⭐ OLHE AS PONTAS, OS CHIFRES E OS VINCOS -- e' o que falta.\n\
         [sculpt3d]        O motor ainda NAO sabe que um vinco existe, entao a grelha atravessa\n\
         [sculpt3d]        a aresta em diagonal e, numa ponta, chega a fechar-se num ponto.\n\
         [sculpt3d]        ⛔ Isso E' conhecido e esta' medido (num octaedro sao 9% das faces);\n\
         [sculpt3d]        NAO precisa de report. O que eu preciso e' do passo (5).\n\
         [sculpt3d]    (5) ⭐ O JULGAMENTO, e e' a unica coisa que eu nao consigo medir sozinho:\n\
         [sculpt3d]        tirando as pontas e os vincos, a malha ja' serve para trabalhar?\n\
         [sculpt3d]        Se serve, o proximo passo e' ensinar o motor a ver os vincos; se nao\n\
         [sculpt3d]        serve, diga O QUE ficou pior que o de sempre e eu meco isso primeiro.\n\
         [sculpt3d]    (6) O Ctrl+Z DESFAZ, e devolve a malha inteira de antes.\n\
         [sculpt3d]    ⭐ E PARA ME MANDAR A PECA: Ctrl+Shift+E grava a cena num ficheiro.\n\
         [sculpt3d]        Escreva o nome com  .obj  no fim (a extensao e' que decide o\n\
         [sculpt3d]        formato; .stl perde a cor). Grave DUAS: uma ANTES de clicar no\n\
         [sculpt3d]        botao e outra DEPOIS -- com as duas eu reproduzo o defeito exacto\n\
         [sculpt3d]        em vez de o adivinhar a partir de uma peca parecida.\n\
         [sculpt3d]    (7) ⚠️ Aumentar o `Detail` NAO limpa os defeitos: a taxa deles por face e'\n\
         [sculpt3d]        praticamente a mesma, entao uma malha 6x maior mostra 6x mais. Medido."
    );
}

#[cfg(test)]
mod tests {
    /// ⭐⭐ **OS DOIS ROTEIROS SAO EXCLUSIVOS, e o novo nao herda a armadilha.**
    ///
    /// ⚠️ **O gate LE O FONTE**, como o irmao da bifurcacao: o passo que manda
    /// *"PARE"* diante de uma casca esburacada e' correcto no caminho de sempre e
    /// **falso** no novo, e nada no compilador impede alguem de o copiar para ca'.
    /// *Um roteiro de smoke e' a unica peca deste repo cujo defeito custa o tempo
    /// do dono do produto, e nao o de uma maquina.*
    #[test]
    fn o_roteiro_novo_nao_manda_parar_diante_do_defeito_ja_medido() {
        let src = include_str!("sculpt3d_scenes_quad.rs");
        let (_, novo) = src
            .split_once("fn announce_extract()")
            .expect("o roteiro do caminho novo tem de existir");
        let novo = novo
            .split_once("mod tests")
            .map_or(novo, |(antes, _)| antes);
        assert!(
            novo.contains("casca FECHADA") && novo.contains("ISSO E' REGRESSAO"),
            "⚠️ a premissa deste gate MUDOU em 2026-08-24: a casca passou a fechar. O \
             roteiro tem de mandar REPORTAR um buraco, e nao calar-se sobre ele — \
             *quem move o numero que tornava algo inalcancavel tem de reconferir a nota*"
        );
        assert!(
            novo.contains("PONTAS") && novo.contains("NAO precisa de report"),
            "o defeito que HOJE esta' medido e' o das feicoes; e' esse que o roteiro tem \
             de nomear como conhecido, senao o Enio gasta uma jornada a reporta-lo"
        );
        assert!(
            !novo.contains("PARE -- e'"),
            "o roteiro novo herdou a armadilha do de sempre: mandar PARAR diante do \
             defeito que esta' medido"
        );
    }

    /// ⭐ **E o roteiro de sempre continua a ser o que se le sem a env var.**
    #[test]
    fn sem_a_env_var_o_roteiro_e_o_de_sempre() {
        assert!(
            !crate::sculpt3d::history::retopo_extract::extract_from(None),
            "sem a env var, a `announce` nao pode desviar para o roteiro novo"
        );
    }
}
