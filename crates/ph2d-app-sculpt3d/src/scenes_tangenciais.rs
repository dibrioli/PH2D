//! **A CENA DOS DOIS GESTOS TANGENCIAIS** (`=40`) — o barro ESPALMA em vez de
//! levantar, e o outro VARRE matéria ao longo do traço.
//!
//! # ⚠️ Ela precisa de RELEVO, e é por isso que não é uma esfera lisa
//!
//! Os dois verbos movem o barro **no plano da superfície**. Numa esfera lisa
//! isso quase não se vê: a silhueta não muda, a luz mal muda, e o artista não
//! consegue separar *o gesto funcionou* de *o gesto não fez nada* — que é a
//! forma de cena que este módulo já recusou uma vez (a `=36`, onde o dono
//! respondeu *«do modo como o objecto é não é possível testar»*).
//!
//! ⇒ ela abre na esfera **enrugada**, a mesma da `=34`: as cristas são a marca
//! que o gesto arrasta, e o que se julga passa a ser binário — *as cristas
//! escorregaram para o lado, sim ou não?*
//!
//! # ⚠️ O que a cena tem de deixar o dono COMPARAR
//!
//! O polegar só significa alguma coisa **contra** o `Draw` (que levanta) e
//! contra o `Move / Grab` (que leva o gesto inteiro, inclinando-se para fora da
//! superfície). O roteiro põe os três no mesmo sítio, na mesma ordem, porque
//! *uma ferramenta nova julga-se pela diferença, não pela aparência*.
//!
//! ⚠️ **E os dois novos julgam-se um contra o outro:** a mesma conta, com a
//! pegada presa num e a viajar no outro. Voltar pelo mesmo caminho devolve o
//! barro num deles e **não** devolve no outro — é a pergunta mais barata que
//! separa os dois, e ela cabe num gesto.

/// `=40` — a cena dos **DOIS GESTOS TANGENCIAIS**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `39`), e
/// não deduzido de um `grep` de uma forma de declaração — a vizinha `=39`
/// registou o preço de o fazer ao contrário.
pub(crate) fn tangenciais_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("40")
}

/// O roteiro da `=40`.
pub(crate) fn announce() {
    if !tangenciais_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =40 OS DOIS GESTOS NOVOS -- espalmar e varrer\n\
         [sculpt3d]    Na tela esta' uma bola com cristas. As cristas sao a marca: e' por\n\
         [sculpt3d]    elas que voce ve para onde o barro foi.\n\
         [sculpt3d]\n\
         [sculpt3d]    Abra o painel com a CRASE (`). A fileira de pinceis esta' no topo; os\n\
         [sculpt3d]    dois novos chamam-se `Thumb` e `Nudge`, e estao no FIM da fileira.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha `Draw` e arraste uma vez sobre as cristas.\n\
         [sculpt3d]        -> O barro SOBE: aparece um vinco levantado por cima delas.\n\
         [sculpt3d]        E' o que voce ja' conhece, e serve de termo de comparacao.\n\
         [sculpt3d]    (2) Ctrl+Z. Escolha `Thumb` e faca o MESMO arrasto, no mesmo sitio.\n\
         [sculpt3d]        -> As cristas ESCORREGAM para o lado do arrasto, como se um polegar\n\
         [sculpt3d]           as tivesse espalmado. O barro NAO sobe: a silhueta da bola\n\
         [sculpt3d]           quase nao muda.\n\
         [sculpt3d]    (3) Ainda com o `Thumb`: carregue, arraste para a direita e, SEM largar,\n\
         [sculpt3d]        volte para o ponto de partida.\n\
         [sculpt3d]        -> O barro tem de voltar ao lugar. Este gesto e' reversivel enquanto\n\
         [sculpt3d]           o dedo nao larga.\n\
         [sculpt3d]    (4) Ctrl+Z. Escolha `Nudge` e repita o gesto de ida e volta.\n\
         [sculpt3d]        -> Agora o barro NAO volta: ele foi TRANSPORTADO, e fica amontoado\n\
         [sculpt3d]           no caminho. E' essa a diferenca entre os dois, e ela e'\n\
         [sculpt3d]           deliberada.\n\
         [sculpt3d]    (5) Ctrl+Z. Escolha `Move / Grab` e arraste como no passo (2).\n\
         [sculpt3d]        -> Aqui o barro vem ATRAS do dedo e sai da superficie: forma-se um\n\
         [sculpt3d]           bico. E' o gesto inteiro; o `Thumb` e' so' a parte dele que\n\
         [sculpt3d]           corre paralela a' pele.\n\
         [sculpt3d]    (6) Com o `Thumb` na mao, leve o `Strength` para metade e repita.\n\
         [sculpt3d]        -> O efeito tem de cair para cerca de um QUARTO, nao para metade:\n\
         [sculpt3d]           neste pincel o numero do slider entra ao quadrado, como na\n\
         [sculpt3d]           referencia.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: o `Thumb` levantar barro (fazer bico) em vez de o\n\
         [sculpt3d]    espalmar; se o gesto de ida e volta NAO devolver o barro no `Thumb`; se\n\
         [sculpt3d]    ele DEVOLVER no `Nudge`; ou se algum dos dois nao mexer nada.\n\
         [sculpt3d]\n\
         [sculpt3d]    (O knob `Normal radius`, no painel, so' aparece com estes dois na mao:\n\
         [sculpt3d]     ele afina de que pedaco da superficie o pincel tira o plano em que\n\
         [sculpt3d]     espalma. Mexer nele muda o resultado em superficie curva.)"
    );
}

#[cfg(test)]
mod tests {
    /// ⛔ **O gate que a vizinha `=39` pagou para existir:** duas cenas a
    /// reclamar o mesmo número deixam a segunda **inalcançável e muda**. Este
    /// teste não substitui o censo do roteador (que varre a crate inteira) — ele
    /// é a metade local, e falha primeiro.
    #[test]
    fn a_cena_reclama_o_nivel_que_o_roteador_declara() {
        // ⚠️⚠️ **`>=` e não `==`, e a diferença custou um vermelho.** A primeira
        // redacção deste gate dizia *«o tecto tem de CONTER esta cena»* e
        // assertava uma **igualdade** — ele passou enquanto esta era a última, e
        // reprovou no dia em que a seguinte nasceu, sobre produto correcto.
        // *Quando a mensagem de um gate e a asserção dele discordam, é a
        // asserção que está errada: a mensagem é o que alguém quis dizer.*
        assert!(
            crate::scenes::CENAS >= 40,
            "o tecto do roteador ({}) tem de conter esta cena (=40)",
            crate::scenes::CENAS
        );
    }
}
