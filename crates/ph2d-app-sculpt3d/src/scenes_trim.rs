//! **A CENA DO BOX TRIM** (`=46`) — cortar a peça com uma forma desenhada.
//!
//! # ⚠️ Ela abre com uma peça MAIS LEVE que o resto do módulo, e o número é o
//! argumento
//!
//! Um corte é uma operação sobre a peça **INTEIRA** e corre uma vez por gesto,
//! no largar. Medido nesta casa (`--release`, o mesmo anel de ecrã, a cadeia
//! completa — lâmina mais booleana):
//!
//! | peça | triângulos | o corte custa |
//! |---|---|---|
//! | cubo subdividido `3×` | `768` | `1,3 ms` |
//! | esfera `20 k` | `19 800` | `23,8 ms` |
//! | **esfera `50 k`** (esta cena) | **`49 612`** | **`58,4 ms`** |
//! | `sculpt_sphere` (o default do módulo) | `196 608` | **`380,6 ms`** |
//!
//! ⇒ no default do módulo o artista larga o rato e espera **mais de um terço de
//! segundo**, que é exactamente o report que esta família já pagou uma vez
//! (*«meio travado»*, 14/09, e a causa foi a cena a fabricar a peça pesada).
//! ⚠️ **E o extremo barato também não serve:** com `768` triângulos a face
//! cortada sai com uma dúzia deles e o artista não consegue ver se ela tem
//! malha — que é precisamente o que esta cena existe para mostrar.
//!
//! # ⛔ O que o dono viu em 2026-09-15, e o que esta cena tem de provar
//!
//! Report com foto: *«o remesh da face que você cortou fica ruim demais»*. A
//! face que um corte deixa é a **parede do prisma** recortada pela peça, e a
//! lâmina tinha **duas faces** ⇒ a face cortada saía com dois triângulos
//! gigantes, `34 ×` mais grossos que a peça à volta — e os cantos dela
//! partilham a normal da esfera, logo uma face **plana** era sombreada como se
//! fosse **curva**. *É por isso que o passo (3) do roteiro manda olhar para a
//! face cortada, e não só para a silhueta.*

/// `=46` — a cena do **BOX TRIM**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `45`) — o
/// gate `no_two_sculpt3d_scenes_claim_the_same_level` já apanhou uma cena a
/// reclamar um número tomado, e a segunda fica **inalcançável e muda**.
pub(crate) fn box_trim_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("46")
}

/// **Quantos triângulos a peça desta cena tem** — ver a tabela do cabeçalho.
///
/// ⚠️ **Ele nomeia o recurso, que é o relógio do CORTE**, e não um gosto: a
/// cadeia é `O(peça)` e o gesto acaba com ela a correr uma vez.
pub(crate) const TRIANGULOS_DA_PECA: usize = 50_000;

/// A peça com que a `=46` abre.
pub(crate) fn peca() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::sphere_with_triangles(TRIANGULOS_DA_PECA, 1.0)
}

/// O roteiro da `=46`.
pub(crate) fn announce() {
    if !box_trim_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =46 BOX TRIM -- cortar a peca com uma forma desenhada\n\
         [sculpt3d]    Voce desenha uma forma por cima da peca e o que fica dentro dela\n\
         [sculpt3d]    e' cortado, de lado a lado. E' a ferramenta de aparar: tirar uma\n\
         [sculpt3d]    fatia, abrir um chanfro, cortar uma ponta fora.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel com a CRASE (`). Na fileira de ferramentas,\n\
         [sculpt3d]        no FIM, ha' um botao `Box Trim`. Carregue nele.\n\
         [sculpt3d]        -> Repare que os controlos de pincel (Radius, Strength) SOMEM:\n\
         [sculpt3d]           esta ferramenta nao tem raio nem forca, e no lugar deles\n\
         [sculpt3d]           aparece `Shape` com tres botoes -- Box, Circle e Lasso.\n\
         [sculpt3d]    (2) Com `Box` escolhido, arraste por cima da bola.\n\
         [sculpt3d]        -> Uma forma AMARELA acompanha a mao, e o barro NAO se mexe\n\
         [sculpt3d]           ainda. Voce esta' a desenhar a lamina, nao a cortar.\n\
         [sculpt3d]    (3) LARGUE.\n\
         [sculpt3d]        -> O que estava dentro da caixa desaparece, e fica uma face\n\
         [sculpt3d]           CHATA no lugar.\n\
         [sculpt3d]        OLHE PARA ESSA FACE DE PERTO (aproxime com a roda do rato):\n\
         [sculpt3d]        -> Ela tem de ter a MESMA malha fina do resto da bola, e o\n\
         [sculpt3d]           encontro dela com a parte redonda tem de ser uma aresta\n\
         [sculpt3d]           NITIDA. Se a face sair lisa como um espelho derretido, com\n\
         [sculpt3d]           o sombreado a escorrer de um canto ao outro, e' o defeito\n\
         [sculpt3d]           que voce reportou em 15/09 -- e ele voltou.\n\
         [sculpt3d]    (4) Ctrl+Z devolve a bola inteira.\n\
         [sculpt3d]    (5) Carregue em `Circle` e arraste a partir do MEIO do sitio que\n\
         [sculpt3d]        quer tirar: aqui o ponto onde voce comeca e' o CENTRO, e\n\
         [sculpt3d]        afastar a mao aumenta o raio.\n\
         [sculpt3d]    (6) Carregue em `Lasso` e desenhe a mao livre, inclusive uma forma\n\
         [sculpt3d]        em C. Repare que aparece uma pista nova, `Smooth Stroke`,\n\
         [sculpt3d]        que nao existe nas outras duas.\n\
         [sculpt3d]        -> Desenhe um laco a tremer com ela em ZERO: a forma amarela\n\
         [sculpt3d]           copia o tremor todo. Suba-a ate' ao fim e desenhe outra vez:\n\
         [sculpt3d]           o tremor sai e a forma que voce quis fica. Os CANTOS\n\
         [sculpt3d]           sobrevivem -- ela nao arredonda o desenho, so' o alisa.\n\
         [sculpt3d]    (7) A tecla L faz o mesmo sem o painel: pega na ferramenta e, nas\n\
         [sculpt3d]        vezes seguintes, roda entre Box, Circle e Lasso -- e depois da\n\
         [sculpt3d]        ultima devolve o pincel que voce tinha. Shift+L faz o corte\n\
         [sculpt3d]        entrar pela FORMA da peca em vez de pelo ecra.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: a bola INTEIRA desaparecer (em vez de perder so' o\n\
         [sculpt3d]    que voce cercou); se o corte acontecer DURANTE o arrasto; se a\n\
         [sculpt3d]    forma amarela nao aparecer; se a face cortada sair sem malha (veja\n\
         [sculpt3d]    o passo 3); se um dos tres botoes de forma nao fizer nada; se a\n\
         [sculpt3d]    pista `Smooth Stroke` aparecer com Box ou Circle escolhidos; ou se\n\
         [sculpt3d]    largar o rato travar o app por mais de um piscar de olhos."
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
                crate::scenes::CENAS >= 46,
                "o tecto do roteador tem de conter esta cena (=46)"
            );
        }
    }

    /// ⭐⭐ **A CENA CONTÉM O FENÓMENO — e as duas cercas são as duas maneiras de
    /// ela mentir.**
    ///
    /// ⚠️ *Uma cena de smoke que ensina o contrário do que acontece é pior que
    /// uma cena ausente* (`CLAUDE.md` §5.0), e aqui há **duas** formas disso:
    /// uma peça **pesada** faz o largar do rato parecer um travamento (o report
    /// de 14/09), e uma peça **leve demais** entrega uma face cortada com uma
    /// dúzia de triângulos — onde o passo (3), que manda olhar para a malha
    /// dela, não consegue afirmar nada.
    #[test]
    fn a_peca_da_cena_cabe_num_gesto_e_mostra_a_malha_do_corte() {
        let m = super::peca();
        let tris: usize = m
            .faces()
            .iter()
            .map(|f| f.verts().len().saturating_sub(2))
            .sum();
        // O tecto sai da tabela do cabeçalho: `196 608` triângulos custam
        // `380,6 ms` e `49 612` custam `58,4`, e o corte é ~linear na peça.
        assert!(
            tris <= 60_000,
            "a peça tem {tris} triângulos — acima daqui o corte passa dos ~70 ms \
             e o largar do rato lê-se como travamento"
        );
        // E o piso: a face cortada é uma secção da peça, logo a contagem dela
        // escala com `tris^(2/3)`. Abaixo daqui ela não tem malha para mostrar.
        assert!(
            tris >= 10_000,
            "a peça tem {tris} triângulos — a face cortada sairia com uma dúzia \
             deles e o passo (3) do roteiro não teria o que afirmar"
        );
    }
}
