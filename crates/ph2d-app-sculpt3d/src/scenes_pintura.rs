//! **A CENA DA PINTURA** (`=51`) — pintar a peça, esbater e esfregar a tinta.
//!
//! # ⚠️ A peça é DENSA de propósito, e a densidade é o assunto
//!
//! A cor vive **por vértice**, logo *a resolução da tinta é a resolução da
//! MALHA* — é essa frase que esta cena existe para ensinar, e é dela que sai a
//! ordem do dono (*«permita que o dynamic topology funcione para os 3
//! pincéis»*): ligar o passe faz a malha adensar debaixo do pincel e a mesma
//! marca passa a ter a borda nítida.
//!
//! ⛔⛔ **E ela NÃO abre com a topologia dinâmica armada**, ao contrário da
//! `=49`: o degrau `(3) → (4)` do roteiro **É** a lição, e uma cena que já abre
//! adensada mostra o resultado sem a comparação. *Sem o antes, o depois não
//! afirma nada.*
//!
//! ⚠️ **A peça é a esfera de `96×144`** — a mesma que a `=16` usa, e pela razão
//! gémea: ali era o tamanho de feature que a malha comporta, aqui é o tamanho de
//! marca. Ela custa `~14 k` vértices, muito abaixo do default do módulo
//! (`98 306`), e ⛔ isso é deliberado: a §24 desta linha registou uma cena que
//! **fabricava** a peça pesada e fazia o dono reportar a ferramenta como lenta.
//!
//! # ⭐ E ela abre com o pincel JÁ ESCOLHIDO
//!
//! O `Paint` é o 36.º chip de uma grelha de 38 — um roteiro que começasse por
//! *«escolha o pincel de pintura»* mandaria o dono procurar. O prólogo arma-o,
//! como a `=49` arma o interruptor dela.

use ph2d_sculpt3d::Verb;

/// `=51` — a cena da **PINTURA**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `50`).
pub(crate) fn pintura_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("51")
}

/// A esfera densa — ver o cabeçalho.
pub(crate) fn peca() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::uv_sphere(LATITUDES, LONGITUDES, 1.0)
}

/// As duas contagens da peça, nomeadas para o gate as poder ler.
pub(crate) const LATITUDES: usize = 96;
/// Ver [`LATITUDES`].
pub(crate) const LONGITUDES: usize = 144;

/// **O que esta cena ARMA** — o pincel, e nada mais.
///
/// ⛔ **A topologia dinâmica fica DESLIGADA de propósito** (ver o cabeçalho), e
/// a cor da peça fica por pintar: semeá-la daria ao dono uma peça já colorida e
/// o passo (2) deixaria de mostrar a tinta a chegar.
///
/// ⛔⛔ **E ela troca de ferramenta pela PORTA, nunca escrevendo o campo.**
/// `cena.brush.verb = v` põe o RÓTULO do pincel de pintura num pincel que
/// continua afinado como o `Draw` — e o que se perde tem número: a força fica
/// em `0,50` (o default do `Draw`) onde o `Paint` declara **`0,75`**, um terço
/// menos de tinta em cada passagem. ⚠️ **E era irreversível dentro da cena:** o
/// [`ph2d_panel_sculpt3d::state::switch_verb_parts`] devolve cedo quando o verbo
/// que entra é o que já está em mãos, logo clicar no chip `Paint` é um no-op e o
/// `0,75` fica inalcançável — *a cena que arma pelo campo cru tranca a afinação
/// que ela própria quis escolher*. É a lei que a irmã `=49` já escreve ao lado
/// do `toggle_dyntopo`: **uma cena arma pela porta do produto, ou é a segunda
/// resposta à mesma pergunta.**
pub(crate) fn arma(cena: &mut crate::Sculpt3dScene) {
    if let Some(v) = verbo_da_cena(pintura_scene()) {
        ph2d_panel_sculpt3d::state::switch_verb_parts(
            &mut cena.verb_slots,
            &mut cena.brush,
            &mut cena.radius_px,
            v,
        );
    }
}

/// **O VERBO COM QUE ESTA CENA ABRE** — a lei, sem cena e sem dispositivo.
///
/// ⚠️ **Ela existe para poder ser GATEADA:** a [`arma`] recebe uma
/// [`crate::Sculpt3dScene`], que pede um `wgpu::Device` — um gate sobre ela
/// nasceria `#[ignore]` e **o CI nunca o correria**. *Quando um gate precisa de
/// um device para medir uma decisão que não tem pixel nenhum, a lei está no
/// sítio errado* (a frase que a §33 desta linha pagou).
///
/// ⚠️ **E ela recebe o veredito em vez de o LER:** a env é lida no `arma`, e uma
/// leitura aqui obrigaria o gate a escrever numa variável de ambiente — que
/// nesta crate custa um `unsafe` (proibido) e, pior, é estado GLOBAL a meio de
/// uma suíte que corre em paralelo.
pub(crate) fn verbo_da_cena(e_a_cena: bool) -> Option<Verb> {
    e_a_cena.then_some(Verb::Paint)
}

/// O roteiro da `=51`.
///
/// ⛔⛔ **O texto fica DENTRO do `eprintln!`, e isso é uma decisão.** Ele esteve
/// numa `const` para o censo dos nomes o poder ler, e o censo de TEXTO desta
/// família reprovou-o na hora: a isenção do HR-15 aqui é *«sai por `eprintln!`,
/// logo é terminal e não ecrã»*, e uma `const` solta perde-a — *a cura teria
/// sido uma linha nova de dívida para tornar um gate mais fácil de escrever*.
/// ⇒ o censo colhe as linhas do roteiro do próprio fonte (as que trazem o
/// prefixo do módulo), que é o molde que a `=49` já usa.
///
/// ⛔ **E não há uma crase SOLTA aqui dentro**, nem para nomear a própria tecla:
/// o censo empareja as crases para saber o que o roteiro NOMEIA, e uma solta
/// fá-lo colher meia frase como se fosse o rótulo de um controlo. Há gate sobre
/// a paridade delas, e ele diz isto.
pub(crate) fn announce() {
    if !pintura_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =51 PINTAR A PECA -- a tinta, o esbater e o esfregar\n\
         [sculpt3d]    O pincel de PINTURA ja' esta' na sua mao.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel (tecla CRASE, acima do TAB) e escolha uma cor\n\
         [sculpt3d]        forte nas tres pistas `Color R` / `Color G` / `Color B` --\n\
         [sculpt3d]        por exemplo R=1, G=0, B=0 para vermelho.\n\
         [sculpt3d]    (2) Arraste sobre a peca.\n\
         [sculpt3d]        -> A peca ganha a cor onde voce passou. A forca da pista\n\
         [sculpt3d]           `Strength` diz quanto de cor cada passagem deposita.\n\
         [sculpt3d]    (3) APROXIME (roda do rato) e olhe a BORDA da marca.\n\
         [sculpt3d]        -> Ela e' serrilhada, e isso nao e' defeito: a cor mora nos\n\
         [sculpt3d]           VERTICES, logo ela e' tao fina quanto a malha e' densa.\n\
         [sculpt3d]    (4) Carregue `P` (topologia dinamica) e pinte AO LADO da 1.a marca.\n\
         [sculpt3d]        -> A malha adensa debaixo do pincel e esta marca sai com a\n\
         [sculpt3d]           borda LIMPA. Compare as duas lado a lado.\n\
         [sculpt3d]\n\
         [sculpt3d]    (4-bis) COM QUE LUZ: no painel, a fileira `Material` (secao\n\
         [sculpt3d]        Shading, role a roda) tem agora tres familias -- `Flat`, `Rig`\n\
         [sculpt3d]        e os materiais. Escolha `Flat`.\n\
         [sculpt3d]        -> A peca fica SEM SOMBRA e a cor que voce escolheu e' a cor que\n\
         [sculpt3d]           voce ve'. E' o modo de JULGAR a cor: com luz, a mesma tinta\n\
         [sculpt3d]           parece mais escura onde a luz e' escura.\n\
         [sculpt3d]        -> Volte a um material e veja a cor acender com a forma.\n\
         [sculpt3d]\n\
         [sculpt3d]    (5) No painel, na fileira de ferramentas, escolha `Blur` e arraste\n\
         [sculpt3d]        POR CIMA da fronteira entre a cor e o barro.\n\
         [sculpt3d]        -> Ela ESBATE: a cor de cada ponto vai ficando a media da\n\
         [sculpt3d]           vizinhanca dele.\n\
         [sculpt3d]    (6) Escolha `Smear Color` e arraste NUM SENTIDO SO', do lado pintado\n\
         [sculpt3d]        para o lado limpo.\n\
         [sculpt3d]        -> A tinta VIAJA com a mao, como quem arrasta tinta fresca com\n\
         [sculpt3d]           o dedo. E' a diferenca para o (5): aquele esbate no mesmo\n\
         [sculpt3d]           sitio, este LEVA.\n\
         [sculpt3d]        -> Com o cursor PARADO ele nao faz nada, e isso e' a lei dele.\n\
         [sculpt3d]    (7) `Ctrl+Z` algumas vezes.\n\
         [sculpt3d]        -> A tinta volta atras, passo a passo.\n\
         [sculpt3d]\n\
         [sculpt3d]    COMO SABER QUE DEU ERRADO: se no (2) a peca nao mudar de cor\n\
         [sculpt3d]    nenhuma, confirme que as pistas de cor nao estao todas em 1 (branco\n\
         [sculpt3d]    sobre barro claro nao se ve). Se o `Ctrl+Z` do (7) nao devolver a\n\
         [sculpt3d]    tinta, ou se o (4) nao mudar nada na borda, PARE e reporte.\n\
         [sculpt3d]    Se a cor SUMIR ao trocar de luz no (4-bis), PARE: ate' 20/09 o\n\
         [sculpt3d]    material de matcap descartava a tinta, e era esse o defeito."
    );
}

#[cfg(test)]
#[path = "scenes_pintura_tests.rs"]
mod tests;
