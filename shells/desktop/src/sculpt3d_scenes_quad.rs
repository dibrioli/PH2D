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
         [sculpt3d]    (5) O `Quad Size` muda o lado do quadrado. O `Follow Curvature` em 1,0 poe\n\
         [sculpt3d]        quadrados MENORES onde a forma aperta -- compare com ele em 0,0.\n\
         [sculpt3d]    (6) ⚠️ Com a pilha de multires montada ela RECUSA e diz para achatar antes."
    );
}
