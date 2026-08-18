//! **A CENA DO FILTRO** (`=34`) — a W9 inteira, e a única cena desta linha que
//! leva uma pergunta que os gates **não conseguem** responder.
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
/// ⚠️ **O passo 4 é uma pergunta ABERTA, e está escrito como tal.** Os gates
/// desta wave provam que o kernel é o `calc_sharpen_filter` (paridade contra a
/// lei escrita à mão, `< 1e-6`), que o gesto é reversível ao byte e que ele não
/// depende da taxa de eventos do rato. O que eles **não** provam é que a lei
/// desenha o que se quer de um afiador: as duas réguas geométricas óbvias — o
/// degrau entre vizinhos e a largura da crista — **caem ou oscilam** sobre a lei
/// correcta, porque metade do mecanismo é achatar o pico e a outra metade é
/// puxar o terreno até ele. *Um gate escrito sobre uma delas não poderia falhar
/// pelo motivo que alegasse.* Este passo é o instrumento que decide.
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
         [sculpt3d]    (4) *** O SHARPEN -- E ESTE PASSO E' UMA PERGUNTA, NAO UMA CONFERENCIA.\n\
         [sculpt3d]        Pegue o SHARPEN e arraste. A lei e' a do Blender, conferida contra o\n\
         [sculpt3d]        fonte a menos de 1e-6 -- o que NAO sei dizer e' se ela desenha o que\n\
         [sculpt3d]        voce quer de um afiador. Ela nao desloca uma feicao: ela muda o\n\
         [sculpt3d]        CONTRASTE entre a crista e o terreno em volta (o topo achata, a volta\n\
         [sculpt3d]        e' puxada ate' ele). Olhe para as QUINAS das cristas e diga se elas\n\
         [sculpt3d]        ficam mais definidas ou apenas diferentes. As duas reguas geometricas\n\
         [sculpt3d]        que tentei nao separam as duas coisas -- por isso a pergunta e' sua.\n\
         [sculpt3d]    (5) O TETO DELE. O arrasto do Sharpen SATURA em 4,0 (medido: 4 / 8 / 16 /\n\
         [sculpt3d]        32 dao numeros identicos a todos os digitos -- e' o *stable state* que\n\
         [sculpt3d]        a referencia nomeia). Arraste MUITO: a peca tem de parar de mudar sem\n\
         [sculpt3d]        nunca explodir nem inverter. Se ela virar do avesso, reporte.\n\
         [sculpt3d]    (6) E O SHARPEN DEIXA A MALHA INTEIRA. Ele NAO e' o Enhance Details com\n\
         [sculpt3d]        outro nome: passe os dois no mesmo gesto e compare. Se sairem iguais,\n\
         [sculpt3d]        a bifurcacao do kernel nao chegou ao alvo: reporte."
    );
}
