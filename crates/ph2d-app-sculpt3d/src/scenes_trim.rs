//! **A CENA DO BOX TRIM** (`=46`) — cortar a peça com uma forma desenhada.
//!
//! # ⛔⛔⛔ Ela abriu com uma peça PRÓPRIA até 2026-09-17, e isso foi o defeito
//!
//! O cabeçalho que estava aqui escolhia uma esfera de `50 k` triângulos e
//! defendia a escolha com **um** número — o relógio do corte:
//!
//! | peça | triângulos | o corte custa | **a borda desvia (mundo)** |
//! |---|---|---|---|
//! | esfera `12 k` | `11 900` | — | `0,0277` · p90 `0,0557` |
//! | **esfera `50 k`** (esta cena até 17/09) | `49 612` | `97,7 ms` | **`0,0103` · p90 `0,0201`** |
//! | esfera `80 k` | `79 000` | — | `0,00007` · p90 `0,0137` |
//! | **`sculpt_sphere` — a peça do MÓDULO** | `196 608` | `375,1 ms` | **`0,00000` · p90 `0,00007`** |
//!
//! ⚠️⚠️ **A coluna da direita não existia, e é a que o dono julga** (report de
//! 2026-09-17: *«o que não fica legal é a topologia das bordas do corte»*). A
//! borda de um corte é feita de pontos **na curva desenhada** e de **vértices da
//! própria peça** que o motor aproveita quando estão perto — e os segundos não
//! estão na curva, logo ela **serrilha com a amplitude da malha**. À densidade
//! do módulo isso cai para **`0,00007`**; a `50 k` vai a `1,2` arestas.
//!
//! ⇒ **a cena deixou de escolher peça**: ela abre com a do módulo, e o corte
//! custa os `375 ms` que o produto custa. ⚠️ **O precedente de 14/09 (*«meio
//! travado»*) media OUTRO recurso** — ali era o custo **por dab**, a 60 Hz,
//! contra um *kill* de `8 ms`; aqui é uma espera **uma vez por gesto**, numa
//! operação destrutiva que tem desfazer. *Comparar os dois foi o que deixou a
//! cena a ensinar uma borda que o artista não tem.*
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
         [sculpt3d]        Faca DUAS vezes, e a segunda e' a que importa:\n\
         [sculpt3d]        -> primeiro um corte todo DENTRO da bola;\n\
         [sculpt3d]        -> depois Ctrl+Z e outro que SAIA pela beira dela, mordendo\n\
         [sculpt3d]           a silhueta.\n\
         [sculpt3d]        OLHE PARA A SUPERFICIE REDONDA AO LADO DO CORTE nos dois casos:\n\
         [sculpt3d]        -> ela tem de estar lisa. Um pontinho claro ou escuro encostado\n\
         [sculpt3d]           a' borda -- um triangulo que apanha a luz de outra maneira\n\
         [sculpt3d]           que os vizinhos -- e' o defeito de 17/09 a voltar, e ele so'\n\
         [sculpt3d]           aparecia no corte que sai pela beira.\n\
         [sculpt3d]        E OLHE PARA A LINHA DA BORDA em si:\n\
         [sculpt3d]        -> ela tem de seguir o circulo que voce desenhou. Se ela\n\
         [sculpt3d]           serrilhar -- entrar e sair do circulo de meio triangulo em\n\
         [sculpt3d]           meio triangulo --, a peca esta' grossa de mais para esse\n\
         [sculpt3d]           corte. Nenhum pincel conserta isso: falta MALHA, nao falta\n\
         [sculpt3d]           alisar. Adense com o pincel `Density` por onde vai cortar e\n\
         [sculpt3d]           corte outra vez.\n\
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
         [sculpt3d]    pista `Smooth Stroke` aparecer com Box ou Circle escolhidos; se\n\
         [sculpt3d]    sobrar uma mancha de luz na superficie ao lado da borda do corte\n\
         [sculpt3d]    (veja o passo 5); ou se largar o rato travar o app por mais de um\n\
         [sculpt3d]    piscar de olhos."
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

    /// ⛔⛔⛔ **A PREMISSA DESTE GATE MORREU EM 2026-09-17, e a morte está no
    /// diff.**
    ///
    /// Ele chamava-se `a_peca_da_cena_cabe_num_gesto_e_mostra_a_malha_do_corte`
    /// e prendia a peça desta cena entre `10 000` e `60 000` triângulos, com o
    /// tecto derivado **do relógio do corte**. ⚠️ **Ele nomeava UM recurso e era
    /// cego ao segundo:** a `50 k` a borda do corte serrilha até `1,2` arestas
    /// da malha, e na peça do módulo ela cai **na curva desenhada**
    /// (`0,00007` de mundo). *A cena estava a trocar a coluna que o dono julga
    /// pela que ela media.*
    ///
    /// ⇒ o que fica é a lei, e ela é **derivada**: a cena não escolhe peça
    /// nenhuma. Quem lhe quiser dar uma outra vez tem de escrever o ramo, e é
    /// esse ramo que este gate proíbe.
    #[test]
    fn a_cena_do_corte_nao_escolhe_a_propria_peca() {
        let src = include_str!("scenes_mesh.rs");
        assert!(
            !src.contains("box_trim_scene"),
            "o roteador de peças voltou a ter um ramo para a `=46` — e uma cena \
             que escolhe a sua peça escolhe também os defeitos que o dono vê"
        );
        // ⭐ E o piso que a metade antiga tinha razão em guardar: a face cortada
        // é uma secção da peça, e abaixo de uma certa densidade o passo (3) do
        // roteiro não tem o que afirmar.
        let m = crate::scenes::mesh::peca_de_fabrica();
        let tris: usize = m
            .faces()
            .iter()
            .map(|f| f.verts().len().saturating_sub(2))
            .sum();
        assert!(
            tris >= 10_000,
            "a peça do módulo tem {tris} triângulos — a face cortada sairia com \
             uma dúzia deles"
        );
    }
}
