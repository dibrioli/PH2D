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
//! # ⭐⭐ E sobre ela o pente CONSTRÓI a grade, em qualquer rumo
//!
//! Medido (`diag_a_escada_por_rumo`), `Q` desligado → no tecto, e quantos
//! triângulos da faixa ficam abaixo de `5°`:
//!
//! | rumo do traço | `Q` desligado → no tecto | lascas |
//! |---|---|---|
//! | ao longo de `x` | `−0,0034 → +0,0961` | `0` de `2 196` |
//! | `30°` | `−0,0537 → +0,0202` | `0` de `2 192` |
//! | `45°` | `−0,0612 → +0,0857` | `0` de `2 260` |
//! | `60°` | `−0,0279 → +0,0592` | `0` de `2 230` |
//!
//! ⇒ **o `Q` TROCA DE SINAL nos quatro** — de uma malha a cruzar o traço para
//! uma a correr com ele — e não fica **uma única lasca** em rumo nenhum. *O
//! roteiro não tem de mandar o artista adivinhar a direcção.*
//!
//! ⚠️ **E o que ele NÃO faz está aqui também:** um traço só não leva o `Q` acima
//! da barra em todos os rumos (a `30°` ele chega a `+0,0202`), logo o gate mede
//! o **Δ** e o **sinal**, que é o que o artista vê. *Uma barra absoluta ali
//! reprovaria sobre produto correcto.*
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
}

/// O roteiro da `=49`.
pub(crate) fn announce() {
    if !pente_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =49 EDGE FLOW -- as linhas da malha viram-se para seguir o traco\n\
         [sculpt3d]    A bola abre com o ARAME a' vista (as linhas finas da malha) porque e'\n\
         [sculpt3d]    a MALHA que este ajuste muda, nao a forma. Repare que os triangulos\n\
         [sculpt3d]    dela estao virados para todo o lado, sem direccao -- e' assim que uma\n\
         [sculpt3d]    peca esculpida fica. O que este pincel faz e' PENTEA'-LOS para onde a\n\
         [sculpt3d]    sua mao for: e' o que se quer ao longo de um braco, de uma prega, de\n\
         [sculpt3d]    um musculo.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Abra o painel com a CRASE (`). Na seccao `Topology` o\n\
         [sculpt3d]        `Dynamic Topology` ja' vem LIGADO, e logo abaixo dele esta' o\n\
         [sculpt3d]        `Edge Flow`, em zero.\n\
         [sculpt3d]    (2) Sem lhe tocar, arraste um risco comprido por cima da bola.\n\
         [sculpt3d]        -> A malha fica mais FINA debaixo do pincel e os triangulos dela\n\
         [sculpt3d]           continuam desencontrados, sem direccao nenhuma. E' o estado de\n\
         [sculpt3d]           sempre, e e' com ele que voce compara o que vem a seguir.\n\
         [sculpt3d]    (3) Puxe o `Edge Flow` ate' ao MAXIMO e risque OUTRA vez, ao lado e\n\
         [sculpt3d]        paralelo ao primeiro.\n\
         [sculpt3d]        -> Agora as linhas da malha alinham-se e correm AO LONGO do risco,\n\
         [sculpt3d]           como um pente passado no cabelo. Ponha os dois riscos lado a\n\
         [sculpt3d]           lado: e' essa a diferenca.\n\
         [sculpt3d]    (4) Risque numa direccao qualquer OUTRA, noutro sitio da bola.\n\
         [sculpt3d]        -> Elas alinham-se outra vez, e seguem a NOVA direccao. Ele nao tem\n\
         [sculpt3d]           direccao propria: ele segue a mao. Experimente varios rumos --\n\
         [sculpt3d]           todos funcionam, nao ha' um bom e um mau.\n\
         [sculpt3d]    (5) Ponha o `Edge Flow` a meio (perto de 0,50) e risque noutro sitio.\n\
         [sculpt3d]        -> Vira menos e ARRUMA mais: os triangulos ficam mais parecidos uns\n\
         [sculpt3d]           com os outros. Meio curso e' onde ele limpa a malha.\n\
         [sculpt3d]    (6) DESLIGUE o `Dynamic Topology` e risque outra vez.\n\
         [sculpt3d]        -> O `Edge Flow` fica na tela e aparece por baixo dele uma linha a\n\
         [sculpt3d]           dizer que ele esta' a dormir; a malha NAO se reorganiza. Volte\n\
         [sculpt3d]           a ligar o interruptor e ele volta ao trabalho.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: as linhas nao virarem no passo (3); se elas seguirem\n\
         [sculpt3d]    sempre a mesma direccao seja qual for o risco; se a bola ficar com\n\
         [sculpt3d]    ESTRIAS de sombra (uma fileira de riscos escuros, que sao triangulos\n\
         [sculpt3d]    finos de mais para terem luz); ou se, com o `Dynamic Topology`\n\
         [sculpt3d]    DESLIGADO, o risco ainda reorganizar a malha.\n\
"
    );
}

/// **OS GATES DESTA CENA** — ver o módulo irmão.
#[cfg(test)]
#[path = "scenes_pente_tests.rs"]
mod tests;
