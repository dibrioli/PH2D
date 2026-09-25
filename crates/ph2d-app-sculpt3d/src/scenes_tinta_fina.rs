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

/// ⭐⭐⭐ **O DEGRAU QUE O ROTEIRO MANDA CARREGAR** — a lição desta cena.
///
/// ⚠️ **Ele é uma CONST e não o último da fileira desde 2026-09-20**, quando o
/// dono mandou acrescentar o `16×`: até aí *«o degrau da lição»* e *«o topo da
/// fileira»* eram a mesma coisa **por acidente**, e o gate que media a cena
/// afirmava o topo. *Duas grandezas que coincidem hoje não são uma lei*, e o
/// dia em que deixaram de coincidir foi o dia seguinte.
///
/// ⭐ Ela tem **dois** consumidores — o gate da densidade e o da cena no
/// produto —, e o censo que exige que o roteiro a NOMEIE é o que impede a const
/// e o texto de se separarem.
///
/// ⚠️ **Ela é `#[cfg(test)]` e isso é a resposta certa, não uma cerca** (o
/// idioma do `CUSTO_POR_VERTICE_NO_TECTO`): o produto não a lê — quem manda
/// carregar no chip é o ROTEIRO, que é texto — e o que ela faz é impedir que os
/// gates e o texto envelheçam cada um para o seu lado. *Pôr um consumidor
/// artificial no produto para calar o `dead_code` seria escrever código para o
/// linter.*
#[cfg(test)]
pub(crate) const DEGRAU_DA_LICAO: ph2d_panel_sculpt3d::state::DetalheDaTinta =
    ph2d_panel_sculpt3d::state::DetalheDaTinta::Oito;

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
         [sculpt3d]        `Paint Detail`. Ela esta' em `Mesh`. Carregue no `8x`.\n\
         [sculpt3d]    (4) Pinte uma segunda marca AO LADO da primeira.\n\
         [sculpt3d]        -> Esta sai com a borda LIMPA, e o arame NAO MUDOU: conte os\n\
         [sculpt3d]           quadrados outra vez, sao os mesmos. E' essa a diferenca\n\
         [sculpt3d]           para a cena =51, onde a borda so' limpava adensando a malha.\n\
         [sculpt3d]        -> Compare as duas marcas lado a lado.\n\
         [sculpt3d]    (4-bis) Carregue `256x`, o ultimo da fileira, e pinte outra vez.\n\
         [sculpt3d]        -> E' o mais fino que o app oferece: menos de um pixel de ecra\n\
         [sculpt3d]           por amostra, que e' o que o traco FINO do Painter pede.\n\
         [sculpt3d]           Numa peca DENSA a placa pode nao o comportar: a fileira\n\
         [sculpt3d]           desce sozinha ao maior que cabe, e um aviso aparece no topo.\n\
         [sculpt3d]    (5) Volte a fileira para `Mesh` e depois carregue `8x` outra vez.\n\
         [sculpt3d]        -> Em `Mesh` as marcas ENGROSSAM (sem plano, a cor mora nos\n\
         [sculpt3d]           vertices) e ao voltar ao `8x` elas ficam FINAS outra vez,\n\
         [sculpt3d]           como estavam. Ate' 23/09 nao voltavam: largar o degrau\n\
         [sculpt3d]           APAGAVA o detalhe fino para sempre.\n\
         [sculpt3d]        ⚠️ O que volta e' o ULTIMO plano que saiu. Se entre sair e\n\
         [sculpt3d]           voltar voce passar por DOIS degraus diferentes, ou pintar\n\
         [sculpt3d]           com a fileira em `Mesh`, ou esculpir, o detalhe fino daquele\n\
         [sculpt3d]           traco deixa de poder voltar.\n\
         [sculpt3d]    (6) Com o `8x` escolhido, carregue `P` (topologia dinamica) e pinte\n\
         [sculpt3d]        VARIAS marcas, umas por cima das outras.\n\
         [sculpt3d]        -> As marcas anteriores CONTINUAM finas e o arame NAO adensa:\n\
         [sculpt3d]           com a tinta fina armada, um pincel de COR deixa de mexer na\n\
         [sculpt3d]           malha -- ele nao precisa dela para ter resolucao, e o\n\
         [sculpt3d]           TERMINAL diz isso no primeiro traco.\n\
         [sculpt3d]        -> Agora escolha o `Draw` na fileira de cima e desenhe: o arame\n\
         [sculpt3d]           adensa, o TERMINAL avisa, e a tinta volta a ser grossa. E'\n\
         [sculpt3d]           o preco de mudar a malha, e ele so' se paga onde e' pedido.\n\
         [sculpt3d]        -> E o app NAO FECHA. Ate' 21/09 este gesto exacto -- um pincel\n\
         [sculpt3d]           de FORMA com o plano armado -- matava a sessao no primeiro\n\
         [sculpt3d]           traco, porque o plano ficava a descrever a malha de antes.\n\
         [sculpt3d]    (6-quinquies) Escolha `Move / Grab`, PEGUE na bola e fique um\n\
         [sculpt3d]        instante SEM arrastar, antes de puxar.\n\
         [sculpt3d]        -> Tambem nao fecha, e a tinta nao pisca. Este gesto tem um\n\
         [sculpt3d]           caminho so' dele: ele PEGA em vez de carimbar, logo ha' um\n\
         [sculpt3d]           instante em que a malha ja' mudou e nada a marcou como suja.\n\
         [sculpt3d]    (6-bis) Clique UMA VEZ no VAZIO, fora da bola, e depois\n\
         [sculpt3d]        pinte outra marca.\n\
         [sculpt3d]        -> As marcas de antes continuam finas. Ate' 21/09 um clique\n\
         [sculpt3d]           fora da peca levava o plano com ele, e a tinta inteira\n\
         [sculpt3d]           voltava a' resolucao da malha.\n\
         [sculpt3d]    (6-ter) Agora COMECE o arrasto FORA da bola, no vazio a' esquerda,\n\
         [sculpt3d]        e entre nela sem largar o botao.\n\
         [sculpt3d]        -> A marca SAI, e sai fina. Com um pincel de COR na mao, o\n\
         [sculpt3d]           traco nao precisa de comecar em cima da peca -- ate' 21/09\n\
         [sculpt3d]           este gesto nao pintava NADA, e lia-se como a tinta a sumir.\n\
         [sculpt3d]        -> O PRECO: com um pincel de cor, arrastar no vazio ja' NAO\n\
         [sculpt3d]           roda a camara. Para rodar, use o botao DIREITO.\n\
         [sculpt3d]    (6-quater) Escolha um pincel BEM FINO (a pista `Radius`) e\n\
         [sculpt3d]        pinte entre dois quadrados do arame, longe de um canto.\n\
         [sculpt3d]        -> A marca SAI. Ate' 21/09 a tinta so' caia quando o pincel\n\
         [sculpt3d]           apanhava um CANTO do arame: o deposito pedia um vertice,\n\
         [sculpt3d]           e a tinta fina existe precisamente para nao o pedir.\n\
         [sculpt3d]    (7) `Ctrl+Z` algumas vezes, e depois `Ctrl+Shift+Z`.\n\
         [sculpt3d]        -> A tinta volta atras traco a traco, e volta a aparecer --\n\
         [sculpt3d]           COM o plano armado (`2x`..`256x`) e sem ele.\n\
         [sculpt3d]        -> Ate' 21/09 o Ctrl+Z nao desfazia a tinta FINA (medido:\n\
         [sculpt3d]           1010 amostras pintadas, 1010 depois do desfazer). O\n\
         [sculpt3d]           desfazer olhava para os VERTICES tocados, e esta tinta\n\
         [sculpt3d]           escreve AMOSTRAS.\n\
         [sculpt3d]        ⚠️ Sair da fileira e VOLTAR ao mesmo degrau nao tira o Ctrl+Z\n\
         [sculpt3d]           ao traco -- o plano volta inteiro (passo 5). O que o tira\n\
         [sculpt3d]           e' passar por DOIS degraus diferentes pelo meio: ai' o plano\n\
         [sculpt3d]           de amostras foi refeito e o de antes ja' nao existe.\n\
         [sculpt3d]\n\
         [sculpt3d]    (8) GRAVAR E REABRIR. Com a fileira em `8x` e uma marca FINA na\n\
         [sculpt3d]        peca, carregue `Ctrl+Shift+S`, escolha um nome e grave.\n\
         [sculpt3d]        FECHE o app, abra-o outra vez com o MESMO comando, e carregue\n\
         [sculpt3d]        `Ctrl+O` para escolher o ficheiro que gravou.\n\
         [sculpt3d]        -> A marca volta FINA, e a fileira `Paint Detail` volta sozinha\n\
         [sculpt3d]           para `8x`.\n\
         [sculpt3d]        ⚠️ FECHAR o app e' o passo, nao um capricho: reabrir o ficheiro\n\
         [sculpt3d]           na MESMA sessao deixa a fileira onde voce a poz, e ai' este\n\
         [sculpt3d]           passo passa mesmo com o defeito vivo.\n\
         [sculpt3d]        -> Ate' 21/09 a tinta voltava e o DETALHE nao: a fileira abria\n\
         [sculpt3d]           em `Mesh` e o PRIMEIRO quadro refazia o plano a' resolucao\n\
         [sculpt3d]           da malha. O ficheiro estava certo; quem a deitava fora era\n\
         [sculpt3d]           o quadro seguinte.\n\
         [sculpt3d]\n\
         [sculpt3d]    (9) LEVAR A TINTA PARA FORA. Com a fileira em `8x` e a marca FINA\n\
         [sculpt3d]        na peca, carregue `Ctrl+Shift+E`, escreva teste.obj e grave.\n\
         [sculpt3d]        -> Aparecem DUAS mensagens no topo, e a de baixo diz que a\n\
         [sculpt3d]           tinta fina foi assada.\n\
         [sculpt3d]        -> Na pasta onde gravou ficam TRES ficheiros: teste.obj,\n\
         [sculpt3d]           teste.mtl e teste_0.png. Abra a imagem: e' a tinta, um\n\
         [sculpt3d]           quadrado por face da peca.\n\
         [sculpt3d]        -> Abra o teste.obj noutro programa (Blender, Windows 3D\n\
         [sculpt3d]           Viewer): a peca chega COM a pintura fina, e nao com a\n\
         [sculpt3d]           versao grossa que a malha teria.\n\
         [sculpt3d]        ⚠️ Se gravar como .ply ou .stl a tinta fina NAO sai (os dois\n\
         [sculpt3d]           formatos so' sabem cor por vertice), e a mensagem de baixo\n\
         [sculpt3d]           avisa que ela ficou para tras.\n\
         [sculpt3d]\n\
         [sculpt3d]    (10) PINTAR TUDO DE UMA VEZ. Com a marca FINA na peca, escolha outra\n\
         [sculpt3d]        cor na caixa e carregue `Fill Piece`, logo abaixo dela.\n\
         [sculpt3d]        -> A peca INTEIRA fica com a cor nova.\n\
         [sculpt3d]        Carregue `Ctrl+Z` UMA vez.\n\
         [sculpt3d]        -> A marca volta, e volta FINA -- nao na resolucao da malha.\n\
         [sculpt3d]\n\
         [sculpt3d]    COMO SABER QUE DEU ERRADO: se no (4) a borda sair igual a' do (2),\n\
         [sculpt3d]    ou se o arame ADENSAR quando voce so' trocou a fileira, PARE e\n\
         [sculpt3d]    reporte. Se a peca ficar PRETA, ou aparecerem faces pretas de\n\
         [sculpt3d]    aresta dura no meio da marca, PARE e reporte. Se no (6-ter) o\n\
         [sculpt3d]    arrasto RODAR a camara em vez de pintar, PARE e reporte. Se no\n\
         [sculpt3d]    (7) o Ctrl+Z deixar a marca onde estava, PARE e reporte. Se no\n\
         [sculpt3d]    (8) a marca voltar GROSSA, ou a fileira abrir em `Mesh` com uma\n\
         [sculpt3d]    peca que voce gravou fina, PARE e reporte. Se no (9) sair so'\n\
         [sculpt3d]    UM ficheiro em vez de tres, ou se a peca chegar ao outro programa\n\
         [sculpt3d]    sem cor nenhuma, PARE e reporte. Se no (10) o `Ctrl+Z` trouxer a\n\
         [sculpt3d]    marca GROSSA, PARE e reporte. E se o\n\
         [sculpt3d]    app FECHAR sozinho em qualquer passo, PARE e mande a linha do\n\
         [sculpt3d]    terminal que comeca por PH2D PANIC -- ela diz onde."
    );
}

#[cfg(test)]
#[path = "scenes_tinta_fina_tests.rs"]
mod tests;
