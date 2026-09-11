//! **A CENA DO FILTRO** (`=34`) — a W9 inteira, e a cena que achou os dois
//! defeitos do Sharpen que nenhum gate desta linha tinha visto.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma lei nenhuma**, a mesma cicatriz que as `=28`..`=33`
//! herdaram do `impasto_smoke` do Painter 2D: a wave entrega chips num picker, e
//! uma cena que os escolhesse por baixo do pano pularia exactamente a costura
//! que ela existe para provar.
//!
//! ⚠️ **Ela abre com as CRISTAS, e a fixture é metade do smoke.** A sonda
//! `measure_sharpen_law.rs` mediu que sobre RUÍDO a lei do Sharpen degenera num
//! alisador — com a curvatura comparável em todo vértice o `f` fica alto em toda
//! parte, o gather é anulado por `(1 − f)` e só o termo médio sobrevive. *Ruído
//! não é feição.* Uma esfera lisa é pior ainda: sem contraste não há o que
//! contrastar, e o artista veria os dois chips novos a não fazer nada.

/// `=34` — a cena do **FILTRO**.
pub(crate) fn filter_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("34")
}

/// O roteiro da `=34`.
///
/// ⚠️ **Este doc dizia que o passo 4 era uma pergunta ABERTA, e o smoke fechou-a
/// pelo pior caminho: reprovando.** Eu tinha escrito que *"as duas réguas
/// geométricas óbvias caem ou oscilam sobre a lei correcta"* e concluído que só
/// o olho podia decidir — o Enio olhou, disse *"Sharpen não parece correto"*, e
/// a causa era minha: o `sharpen_factor` era recomputado a cada sub-passo
/// quando no fonte ele vive no `filter_cache`, construído **uma vez por gesto**.
/// *A régua não discriminava porque a lei alisava.* Corrigida, ela discrimina, e
/// os passos 4 e 5 passaram de perguntas a **conferências** — cada um com o
/// sintoma do defeito antigo escrito ao lado, para o smoke saber o que procurar.
///
/// ⚠️ **E o passo 5 guarda a segunda metade do mesmo report:** os polos desta
/// esfera têm valência 144 e o gather da referência não é normalizado pela
/// contagem, então a malha **explodia** (degrau a 1098×). A guarda de valência é
/// divergência declarada e é identidade em malha regular.
pub(crate) fn announce() {
    if !filter_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =34 O FILTRO (a W9 -- o verbo aplicado a` MALHA INTEIRA).\n\
         [sculpt3d]    Um filtro nao tem dab: e' um ARRASTO que aplica uma lei a toda a peca,\n\
         [sculpt3d]    a partir da pose congelada no pen-down -- entao arrastar de volta ao\n\
         [sculpt3d]    ponto de partida devolve a malha EXACTA.\n\
         [sculpt3d]    A cena abre com as CRISTAS de proposito: sobre uma esfera lisa as duas\n\
         [sculpt3d]    leis novas nao tem contraste com que trabalhar, e nao fariam nada.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e ache o picker de LEI do filtro.\n\
         [sculpt3d]    (1) OS NOVE CHIPS. Smooth . Relax . Surface Smooth . Inflate . Scale .\n\
         [sculpt3d]        Sphere . Random . Enhance Details . Sharpen. Se algum faltar, PARE --\n\
         [sculpt3d]        o resto nao diz nada.\n\
         [sculpt3d]    (2) O ARRASTO DE VOLTA. Pegue o Smooth, arraste ate' a peca alisar bem, e\n\
         [sculpt3d]        arraste de VOLTA ao ponto onde comecou sem soltar. As cristas tem de\n\
         [sculpt3d]        voltar INTEIRAS. Se sobrar residuo, a pose congelada nao esta' a ser\n\
         [sculpt3d]        reposta: reporte.\n\
         [sculpt3d]    (3) O TETO QUE A W9c-a REMOVEU -- e o CONTROLE vem primeiro. Pegue o\n\
         [sculpt3d]        SMOOTH e arraste para a ESQUERDA (forca negativa, que realca em vez\n\
         [sculpt3d]        de alisar). Passados ~1000 px ele PARA de responder: e' o clamp de\n\
         [sculpt3d]        (-1, 1) da referencia. Agora o mesmo gesto com ENHANCE DETAILS: ele\n\
         [sculpt3d]        tem de CONTINUAR a realcar muito depois de o Smooth ter parado.\n\
         [sculpt3d]        Medido: o Smooth prende em 0,0726 e a referencia alcanca 0,2179.\n\
         [sculpt3d]        Se os dois pararem juntos, o teto voltou: reporte.\n\
         [sculpt3d]    (4) *** O SHARPEN. Pegue-o e arraste. Ele NAO desloca uma crista: ele muda\n\
         [sculpt3d]        o CONTRASTE entre ela e o terreno em volta -- o topo achata um pouco e\n\
         [sculpt3d]        a volta e' puxada ate' ele, o que deixa a transicao mais CURTA. Olhe\n\
         [sculpt3d]        para as QUINAS das cristas: elas tem de ficar mais definidas.\n\
         [sculpt3d]        (No 1o smoke ele ALISAVA -- eu recomputava a curvatura a cada passo e\n\
         [sculpt3d]        a lei perseguia o proprio rasto. Agora ela e' medida uma vez, como no\n\
         [sculpt3d]        Blender. Se ainda parecer que ele alisa, reporte.)\n\
         [sculpt3d]    (5) A PECA NAO PODE EXPLODIR. Arraste MUITO: ela tem de parar de mudar e\n\
         [sculpt3d]        ficar parada (o arrasto satura). No 1o smoke a bola virava do avesso --\n\
         [sculpt3d]        os polos desta esfera tem 144 vizinhos e a conta do Blender supoe ~4.\n\
         [sculpt3d]        Se algo esticar para fora de quadro, reporte NA HORA.\n\
         [sculpt3d]    (6) E O SHARPEN DEIXA A MALHA INTEIRA. Ele NAO e' o Enhance Details com\n\
         [sculpt3d]        outro nome: passe os dois no mesmo gesto e compare. Se sairem iguais,\n\
         [sculpt3d]        a bifurcacao do kernel nao chegou ao alvo: reporte."
    );
}
