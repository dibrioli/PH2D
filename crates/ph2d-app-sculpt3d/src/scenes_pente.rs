//! **A CENA DO PENTE DE TOPOLOGIA** (`=49`) — o fluxo das arestas segue o traço.
//!
//! # ⚠️ Ela abre com a topologia dinâmica ARMADA, e isso é a lição
//!
//! A pré-condição do pente é o interruptor (espec §2.1: desarmado, os dois lados
//! do controlo dão a MESMA malha, byte a byte) — logo uma cena que abrisse com
//! ele desligado punha o artista a arrastar uma ferramenta inerte no passo (1) e
//! a concluir que ela está partida. *Uma cena que ensina o contrário do que
//! acontece é pior que uma cena ausente*, e é o preço que o `Density` já pagou
//! com o report *«não vejo efeito»*.
//!
//! ⭐ **A cerca é exercitada na mesma, no fim:** o passo que DESLIGA o
//! interruptor é onde o artista vê a pista continuar na tela e o painel dizer
//! que ela dorme — a metade da cerca que o `show` de uma fileira não consegue
//! exprimir, porque o interruptor é um FACTO do retrato e não estado autorado.
//!
//! # ⚠️ E ela abre com o ARAME à vista
//!
//! O que este controlo muda é a **malha**, não a forma: com o arame desligado a
//! peça fica *exactamente igual* nos dois lados do slider, e o smoke mediria a
//! sombra em vez do produto.
//!
//! # ⛔⛔⛔ A peça é REMALHADA, e a peça de sempre não serve
//!
//! O que este pincel faz é pôr o fluxo das arestas a correr com o traço — logo
//! *ele é invisível onde a malha já corre nessa direcção*. Medido
//! (`diag_o_grao_das_pecas_candidatas`), `Q` na faixa **antes de ele tocar em
//! nada**, com a barra do corpus em `+0,0465`:
//!
//! | peça | ao longo do equador | ao longo de um meridiano |
//! |---|---|---|
//! | a esfera UV desta casa | `+0,5498` | `+0,5116` |
//! | **a de FÁBRICA do módulo** | **`+0,3241`** | **`+0,3241`** |
//! | **remalhada isotropicamente** | **`−0,0350`** | **`−0,0165`** |
//!
//! ⇒ *uma esfera UV JÁ É uma grade* — sete a doze vezes a barra —, e sobre ela
//! metade dos riscos que o artista der cai **ao longo** do grão e não mostra
//! nada. ⛔ **A peça de fábrica do módulo é uma delas**, logo abrir na peça de
//! sempre era o defeito.
//!
//! ⭐ **A única sem grão é a do remalhador isotrópico** — e não por acaso:
//! *isotrópico quer dizer exactamente «sem direcção preferida»*. É a fase zero
//! que o botão de retopologia já corre, e custa `~270 ms` **uma vez**, ao abrir.
//!
//! # ⭐⭐ E sobre ela o pente trabalha em QUALQUER rumo
//!
//! Medido no **regime que o app dá** (raio de fábrica pela câmara, `Detail` que
//! esta cena arma), `Q` desligado → no tecto, e quantos vértices o pente desloca
//! num traço:
//!
//! | rumo do traço | `Q` desligado → no tecto | vértices movidos | lascas |
//! |---|---|---|---|
//! | ao longo de `x` | `−0,0008 → +0,0985` | `2 786` | `0` |
//! | `30°` | `−0,0665 → −0,0001` | `2 775` | `0` |
//! | `45°` | `−0,0807 → +0,0775` | `2 791` | `1` |
//! | atravessado | `+0,0516 → +0,1285` | `2 803` | `0` |
//!
//! ⇒ **a malha deixa de CRUZAR o traço nos quatro** e o Δ passa a barra do
//! corpus em todos. ⚠️ **E o que ele NÃO faz está aqui também:** a `30°` um traço
//! só aterra em `−0,0001` — *ali ele neutraliza o cruzamento e não constrói
//! grade* —, logo o gate mede o **Δ** e *«já não cruza»*, nunca um `Q` absoluto.
//!
//! ⛔⛔ **E esta escolha REFUTOU uma recusa minha, medida horas antes.** A 1.ª
//! redacção desta cena abria na esfera UV e declarava a remalhada recusada com
//! *«Δ = +0,028, não se vê»*. Esse número saiu de **uma** direcção de traço e de
//! **outra** densidade; varrido o leque, a remalhada ganha em todas as colunas e
//! a UV é que deixa `3` lascas a `45°` e acaba com `Q = −0,008`, ou seja **sem
//! grade nenhuma**. *Uma recusa medida responde UMA pergunta, e esta respondeu à
//! errada.* ⚠️ Quem a matou foi uma **mutação SOBREVIVENTE**: trocar a peça pela
//! recusada deixava o gate da cena VERDE.
//!
//! ⚠️ **A densidade sai do corpus do ORÁCULO** (`6 146` vértices com um pincel de
//! raio `0,35` — `≈ 8` arestas por raio): mais fina e o arame vira um borrão
//! cinzento onde não se lê direcção nenhuma; mais grossa e não há arestas que
//! cheguem para uma grade se formar.
//!
//! # ⛔⛔⛔ E o `Detail` é ARMADO no prólogo, por um report: *«não percebi diferença»*
//!
//! A 1.ª redacção desta cena abria com o `Detail` de fábrica (`0,50`) e o gate
//! dela ficava **VERDE** — porque ele corria com um raio e um alvo de refino que
//! **eu escolhi**, e não com os que o app dá. Medidos
//! (`diag_o_regime_do_app_contra_o_do_gate`):
//!
//! | | o gate | **o APP de fábrica** |
//! |---|---|---|
//! | raio do pincel | `0,35` | **`0,1634`** (`50 px` pela câmara) |
//! | aresta alvo do refino | `0,035` | **`0,0805`** |
//! | **arestas por raio** | **`10,0`** | **`2,0`** |
//!
//! ⚠️ A `Detail 0,50` o alvo do passe (`0,0805`) é mais GROSSO que a aresta da
//! peça (`0,0527`) — *o passe engrossa em vez de refinar*, e o pincel acaba com
//! **duas** arestas de raio.
//!
//! ⛔⛔ **E a primeira hipótese — «a lei é fraca nesse regime» — foi REFUTADA:**
//! ali o `Q` até sobe MAIS (`+0,1926` contra `+0,1560`). O que muda é a
//! MAGNITUDE:
//!
//! | regime | arestas/raio | `ΔQ` | **vértices que MEXEM** |
//! |---|---|---|---|
//! | o app de fábrica | `2,0` | `+0,1926` | **`121`** |
//! | `Detail 0,75` | `4,1` | `+0,1839` | `603` |
//! | **`Detail 1,00`** | `9,3` | `+0,1588` | **`2 715`** |
//!
//! ⇒ *o `Q` é uma MÉDIA e sobe com um punhado de arestas alinhadas; o que o olho
//! lê é a CONTAGEM.* Num traço inteiro mexiam **121** vértices — a mesma forma de
//! defeito que o `Density` custou (*«a colheita é 1 %, invisível»*, com o gate
//! verde a afirmar só o SINAL). ⇒ o prólogo arma o [`DETALHE_DA_CENA`], e o gate
//! passou a correr no regime do app **com uma metade de magnitude**.

/// `=49` — a cena do **PENTE DE TOPOLOGIA**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `48`), e
/// não lido de uma nota: o roteador desta família é DISPERSO, e quem o conta é
/// o gate `o_tecto_declarado_e_o_maior_nivel_reclamado`.
pub(crate) fn pente_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("49")
}

/// A bola de PARTIDA, antes do remalhador — ela é só a semente da forma.
///
/// ⚠️ **Ela não decide a densidade de saída**, e é por isso que o número aqui
/// não tem tabela: quem a decide é o [`ARESTA_ALVO`] logo abaixo.
const SEMENTE_DA_FORMA: usize = 12_000;

/// **A ARESTA QUE O REMALHADOR PERSEGUE**, em fracção da diagonal da caixa.
///
/// ⚠️ **O recurso que este número nomeia é a LEGIBILIDADE DO ARAME.** Medido
/// (`diag_o_relogio_da_peca_da_cena`): **`0,0142 → 5 276` vértices** ·
/// `0,0131 → 6 994` · `0,0125 → 7 593`, os três entre `230` e `290 ms`. O
/// escolhido é o mais grosso dos três — *o arame tem de se LER, e a `7 000`
/// vértices numa bola de raio `1` ele já é um borrão cinzento no ecrã do dono*.
const ARESTA_ALVO: f32 = 0.0142;

/// A peça com que a `=49` abre: uma bola **SEM GRÃO** — ver o cabeçalho.
///
/// ⛔ **O `triangulate` vem antes do remalhador de propósito:** ele recusa quads
/// por geometria, e sem ele a saída seria a entrada (a mesma armadilha em que o
/// arnês desta cena caiu: `288` arestas na faixa e **zero** triângulos).
pub(crate) fn peca() -> ph2d_mesh::Mesh {
    let mut m = ph2d_mesh::shapes::sphere_with_triangles(SEMENTE_DA_FORMA, 1.0);
    m.triangulate();
    let _ = ph2d_remesh_iso::remesh_isotropic(&mut m, ARESTA_ALVO);
    m
}

/// **O que esta cena ARMA depois de a cena nascer** — o interruptor da topologia
/// dinâmica e o arame.
///
/// ⚠️ **Ela é chamada pela porta do prólogo** (`crate::scenes::prologo`) e não
/// pelo construtor: armar a topologia dinâmica TRIANGULA a malha, logo é um acto
/// sobre uma cena que já existe, e o `toggle_dyntopo` é a única resposta a
/// *«ligar o passe»* que este módulo tem. *Escrever `dyntopo.armed = true` aqui
/// seria a segunda, e a que não tritura os quads.*
pub(crate) fn arma(cena: &mut crate::Sculpt3dScene) {
    if !pente_scene() {
        return;
    }
    let (ligado, _) = cena.toggle_dyntopo();
    debug_assert!(ligado, "a =49 tem de abrir com a topologia dinamica ARMADA");
    cena.wireframe = true;
    cena.dyntopo.detail = DETALHE_DA_CENA;
}

/// **O `Detail` COM QUE ESTA CENA ABRE** — ver o cabeçalho.
///
/// ⛔ **Ele NÃO é o de fábrica, e o número é MEDIDO:** a `0,50` o alvo do passe
/// (`0,0805`) é mais grosso que a aresta da peça (`0,0527`), logo o passe
/// **engrossa** — o pincel fica com `2,0` arestas de raio e um traço inteiro
/// mexe **`121`** vértices, que é invisível. A `1,00` são `9,3` arestas de raio
/// e **`2 715`** vértices.
///
/// ⚠️ **O recurso do topo está medido noutro sítio** (`MAX_TRIS = 100 000`, o
/// relógio do dab: `~2,8 ms` de um orçamento de `8`), logo isto não é uma cena a
/// pedir mais do que o produto dá — é uma cena a abrir onde a ferramenta tem o
/// que fazer.
pub(crate) const DETALHE_DA_CENA: f32 = 1.0;

/// O roteiro da `=49`.
///
/// ⛔⛔⛔ **ELE MUDOU DUAS VEZES, e as duas por MEDIÇÃO.**
///
/// A 1.ª redacção prometia *«as linhas viram-se e passam a correr ao longo do
/// risco»* e o dono reprovou-a com foto (*«não sei o que é para esperar. não
/// vejo diferença»*, 18/09) — o arame desenhado mostrava os dois lados do
/// controlo **indistinguíveis**. A 2.ª passou a dizer o contrário: *«o efeito é
/// medível e quase invisível»*.
///
/// ⭐⭐⭐ **E essa segunda MORREU no mesmo dia, quando a lei ganhou a terceira
/// metade** (a troca de diagonal — ver [`ph2d_mesh::alinha_arestas`]). Medido na
/// peça desta cena, nos quatro rumos do traço, o `ΔQ` passou de `+0,028`–`+0,064`
/// para **`+0,10`–`+0,24`**, contra os `+0,074` da lei que ela substituiu — e o
/// arame, ampliado na faixa do risco, mostra agora arestas visivelmente mais
/// longas e alinhadas com ele.
///
/// ⚠️ **O que ele NÃO volta a prometer é uma GRADE de livro.** O que se vê é a
/// malha debaixo do risco a ficar direccional; quem quiser conferir o desenho
/// corre a sonda [`super::tests::sondas::diag_desenha_o_arame`] e compara os dois
/// `.ppm`. *Uma cena que promete mais do que o desenho mostra é a espécie que o
/// `CLAUDE.md` §5.0 chama de pior que uma cena ausente.*
pub(crate) fn announce() {
    if !pente_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =49 EDGE FLOW -- a malha debaixo do risco passa a seguir o risco\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel com a CRASE (`). Na seccao `Topology` o\n\
         [sculpt3d]        `Dynamic Topology` ja' vem LIGADO e o `Detail` no TOPO. Logo\n\
         [sculpt3d]        abaixo esta' o `Edge Flow`, em zero.\n\
         [sculpt3d]    (2) Risque uma vez com ele em ZERO.\n\
         [sculpt3d]    (3) Ponha-o no MAXIMO e risque outra vez, ao lado do primeiro.\n\
         [sculpt3d]    (4) Compare os dois riscos de PERTO (aproxime com a roda): no\n\
         [sculpt3d]        segundo os triangulos ficam mais COMPRIDOS e apontam na\n\
         [sculpt3d]        direccao em que a sua mao andou; no primeiro eles apontam para\n\
         [sculpt3d]        todos os lados.\n\
         [sculpt3d]\n\
         [sculpt3d]    COMO SABER QUE DEU ERRADO: se os dois riscos ficarem iguais, ou se\n\
         [sculpt3d]    a malha ficar com triangulos finos como lascas de vidro, diga-me --\n\
         [sculpt3d]    as duas coisas estao medidas e nenhuma devia acontecer.\n\
         [sculpt3d]\n\
         [sculpt3d]    (medido nesta peca: o alinhamento da faixa sobe 4x mais do que subia\n\
         [sculpt3d]    antes de hoje, e sem um unico triangulo fino)\n"
    );
}

/// **OS GATES DESTA CENA** — ver o módulo irmão.
#[cfg(test)]
#[path = "scenes_pente_tests.rs"]
mod tests;
