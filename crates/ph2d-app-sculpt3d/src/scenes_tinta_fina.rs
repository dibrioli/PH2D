//! **A CENA DA TINTA FINA** (`=52`) — a tinta deixa de ter a resolução da malha.
//!
//! # ⚠️ A peça é GROSSA de propósito, e é o OPOSTO da irmã
//!
//! A [`crate::scenes::pintura`] (`=51`) abre **densa** para ensinar a frase *«a
//! cor mora nos VÉRTICES, logo a resolução da tinta é a da malha»*, e a cura
//! que ela mostra é **adensar a malha** (a tecla `P`). Esta abre com uma esfera
//! de `24 × 32` — `~770` vértices, `18×` menos que a irmã — porque aqui a cura
//! é outra: **a tinta ganha resolução PRÓPRIA e a malha fica onde está.**
//!
//! ⛔⛔ **Sem a peça grossa a cena não tem o fenómeno.** Numa malha densa a
//! marca por vértice já sai quase limpa, e o degrau `Mesh → 8×` seria invisível
//! — *a cena mostraria a cura a não fazer nada*, que é o defeito que a `=50`
//! desta linha pagou por carimbar onde a lei antiga já cortava.
//!
//! # ⭐ E o ARAME é o que prova a metade que importa
//!
//! O passo que separa esta wave da irmã não é *«a borda ficou limpa»* — é *«a
//! borda ficou limpa E A MALHA NÃO MUDOU»*. Com a topologia dinâmica o arame
//! adensa debaixo do pincel; aqui ele fica **exactamente** como estava, e o
//! roteiro manda ligá-lo antes de qualquer coisa.
//!
//! # ⚠️ O PREÇO está no roteiro, e não numa nota
//!
//! Voltar a `Mesh` **não devolve** o detalhe fino: o plano é largado e o que
//! fica é a cor por vértice, que é a mesma tinta vista com a resolução da
//! malha. *Uma cena que escondesse isso ensinaria que o controlo é grátis.*

use ph2d_sculpt3d::Verb;

/// `=52` — a cena da **TINTA FINA**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `51`).
pub(crate) fn tinta_fina_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("52")
}

/// As duas contagens da peça, nomeadas para o gate as poder ler.
///
/// ⚠️ **Elas são a metade GROSSA do par** — ver o cabeçalho; há gate a exigir
/// que esta peça tenha **menos** vértices que a da `=51`, senão as duas cenas
/// ensinam a mesma coisa e uma delas é ruído.
pub(crate) const LATITUDES: usize = 24;
/// Ver [`LATITUDES`].
pub(crate) const LONGITUDES: usize = 32;

/// A esfera grossa — ver o cabeçalho.
pub(crate) fn peca() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::uv_sphere(LATITUDES, LONGITUDES, 1.0)
}

/// **O que esta cena ARMA** — o pincel de pintura e o ARAME, e nada mais.
///
/// ⛔ **A resolução da tinta fica em `Mesh` de propósito:** o degrau `(3) →
/// (4)` do roteiro **É** a lição, e uma cena que já abre com o plano armado
/// mostra o resultado sem a comparação — *sem o antes, o depois não afirma
/// nada* (a frase que a irmã `=51` já escreve).
///
/// ⭐ **E o arame nasce LIGADO**, ao contrário de tudo o resto: ele é o
/// CONTROLO desta cena (a malha não muda), e um controlo que o dono tem de se
/// lembrar de ligar não está na comparação.
///
/// ⛔⛔ **A troca de ferramenta é pela PORTA**, nunca escrevendo o campo — a lei
/// que a `=51` pagou: `cena.brush.verb = v` põe o rótulo do pincel de pintura
/// num pincel afinado como o `Draw` (força `0,50` contra os `0,75` do `Paint`) e
/// **tranca** a afinação, porque o `switch_verb_parts` devolve cedo no verbo que
/// já está em mãos.
pub(crate) fn arma(cena: &mut crate::Sculpt3dScene) {
    if let Some(v) = verbo_da_cena(tinta_fina_scene()) {
        ph2d_panel_sculpt3d::state::switch_verb_parts(
            &mut cena.verb_slots,
            &mut cena.brush,
            &mut cena.radius_px,
            v,
        );
        cena.wireframe = true;
    }
}

/// **O VERBO COM QUE ESTA CENA ABRE** — a lei, sem cena e sem dispositivo.
///
/// ⚠️ **Ela existe para poder ser GATEADA** e **recebe o veredito em vez de o
/// ler**, pelas duas razões que a irmã `=51` escreve: um gate sobre a [`arma`]
/// pediria um `wgpu::Device` e nasceria `#[ignore]`, e escrever uma variável de
/// ambiente numa suíte paralela é estado global.
pub(crate) fn verbo_da_cena(e_a_cena: bool) -> Option<Verb> {
    e_a_cena.then_some(Verb::Paint)
}

/// O roteiro da `=52`.
///
/// ⛔⛔ **O texto fica DENTRO do `eprintln!`** — a mesma decisão da `=51`: numa
/// `const` ele perde a isenção do HR-15 (*«sai por `eprintln!`, logo é terminal
/// e não ecrã»*) e o censo de texto reprova-o.
///
/// ⛔ **E não há uma crase SOLTA aqui dentro:** o censo empareja as crases para
/// saber o que o roteiro NOMEIA, e uma solta fá-lo colher meia frase como se
/// fosse o rótulo de um controlo.
pub(crate) fn announce() {
    if !tinta_fina_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =52 A TINTA DEIXA DE TER A RESOLUCAO DA MALHA\n\
         [sculpt3d]    O pincel de PINTURA ja' esta' na sua mao, e a peca e' GROSSA\n\
         [sculpt3d]    de proposito: o arame ja' esta' ligado e voce consegue contar\n\
         [sculpt3d]    os quadrados dela.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel (tecla CRASE, acima do TAB) e carregue na\n\
         [sculpt3d]        CAIXA DE COR da fileira `Color`. Escolha uma cor forte\n\
         [sculpt3d]        (vermelho) e carregue FORA do seletor para o fechar.\n\
         [sculpt3d]    (2) Arraste uma marca CURTA sobre a peca.\n\
         [sculpt3d]        -> A borda dela e' um serrilhado GROSSO, do tamanho dos\n\
         [sculpt3d]           quadrados do arame. E' a tinta a ter a resolucao da malha.\n\
         [sculpt3d]    (3) No painel, logo ABAIXO da caixa de cor, esta' a fileira\n\
         [sculpt3d]        `Paint Detail`, com quatro botoes. Ela esta' em `Mesh`.\n\
         [sculpt3d]        Carregue no ultimo (o `8x`).\n\
         [sculpt3d]    (4) Pinte uma segunda marca AO LADO da primeira.\n\
         [sculpt3d]        -> Esta sai com a borda LIMPA, e o arame NAO MUDOU: conte os\n\
         [sculpt3d]           quadrados outra vez, sao os mesmos. E' essa a diferenca\n\
         [sculpt3d]           para a cena =51, onde a borda so' limpava adensando a malha.\n\
         [sculpt3d]        -> Compare as duas marcas lado a lado.\n\
         [sculpt3d]    (5) Volte a fileira para `Mesh`.\n\
         [sculpt3d]        -> A segunda marca ENGROSSA e fica como a primeira. E' o preco,\n\
         [sculpt3d]           e nao um defeito: sem o plano, a cor volta a morar nos\n\
         [sculpt3d]           vertices. Carregue `8x` outra vez e pinte -- o detalhe novo\n\
         [sculpt3d]           volta a sair fino.\n\
         [sculpt3d]    (6) Com o `8x` escolhido, carregue `P` (topologia dinamica) e pinte.\n\
         [sculpt3d]        -> O TERMINAL tem de dizer que a tinta fina perde detalhe: um\n\
         [sculpt3d]           pincel que muda a malha muda o chao debaixo do plano, e o app\n\
         [sculpt3d]           avisa ANTES em vez de apagar o seu trabalho em silencio.\n\
         [sculpt3d]    (7) `Ctrl+Z` algumas vezes.\n\
         [sculpt3d]        -> A tinta volta atras, passo a passo, com plano ou sem ele.\n\
         [sculpt3d]\n\
         [sculpt3d]    COMO SABER QUE DEU ERRADO: se no (4) a borda sair igual a' do (2),\n\
         [sculpt3d]    ou se o arame ADENSAR quando voce so' trocou a fileira, PARE e\n\
         [sculpt3d]    reporte. Se a peca ficar PRETA, ou aparecerem faces pretas de\n\
         [sculpt3d]    aresta dura no meio da marca, PARE e reporte."
    );
}

#[cfg(test)]
#[path = "scenes_tinta_fina_tests.rs"]
mod tests;
