//! **A CENA DE PROJECTAR NA CENA** (`=45`) — moldar uma peça **contra** outra.
//!
//! # ⚠️⚠️ Ela abre com DUAS peças, e sem a segunda o pincel é inerte por LEI
//!
//! Este pincel não tem alvo dentro de si: ele mede a distância até **outra
//! peça** e leva o barro até lá (espec §6.3.3 — *nenhum acerto, o vértice não se
//! move*). ⇒ uma cena de uma peça só mostraria uma ferramenta que parece
//! partida, que é a espécie que o `CLAUDE.md` §5.0 chama de **pior que uma cena
//! ausente**. A `=45` põe uma **bola** por cima de uma **placa**, e o roteiro
//! começa por apagar a placa para o artista ver a recusa.
//!
//! # ⭐⭐ A placa é PLANA e a bola está MEIO ENTERRADA nela — e isso é MEDIDO
//!
//! *Projectar é a ferramenta de encostar uma coisa na outra* — apertar um rosto
//! contra uma parede, sentar uma peça numa mesa. Uma placa plana torna o
//! resultado **legível sem medir**: ou o barro ficou chato ao nível dela, ou não
//! ficou.
//!
//! ⛔⛔⛔ **E a ALTURA dela não é estética: ela é a única grandeza que decide se
//! esta cena ensina o pincel ou o difama** (report do dono, 2026-09-14:
//! *«resultado bem bizarro»*). O deslocamento de um dab é `d · peso · força²`, e
//! o `d` é uma distância da **CENA** — nada na lei o compara com o raio do
//! pincel. Com a placa **por baixo** da bola o raio da vista atravessa a peça
//! inteira antes de a alcançar, e a régua
//! [`diag_a_regua_da_cena`](super::gesto_tests) leu, na coluna do meio do
//! canvas:
//!
//! | a placa | `d / R` no sítio onde o artista arrasta |
//! |---|---|
//! | `−1,25` (por baixo) | **`5,70`–`7,85`**, e **nenhum acerto** na metade de baixo |
//! | **`+0,40`** (a cortar a bola) | **`0,0`–`2,2`** |
//!
//! ⇒ *com a placa por baixo NÃO EXISTE ponto do ecrã em que este pincel seja
//! legível*: ou ele arrasta o barro seis raios através da peça, ou não faz nada.
//! Hoje a bola está **meio enterrada** na mesa e o que sobressai é uma **calota**
//! — carimbar nela deixa-a **rente à mesa**, que é a ferramenta a fazer o que ela
//! diz.
//!
//! ⚠️ **E a folga é o knob que mais surpreende**, por isso ela tem passo próprio:
//! ela só é «distância mínima» no sentido de avanço (espec §6.4, as duas
//! medidas), e com um valor maior que o vão **a peça AFASTA-SE**. *O artista tem
//! de ver isso uma vez, ou vai lê-lo como um defeito.*

use ph2d_mesh::{Face, Mesh, Pose};

/// `=45` — a cena de **PROJECTAR NA CENA**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `44`) — o
/// gate `no_two_sculpt3d_scenes_claim_the_same_level` já apanhou uma cena a
/// reclamar um número tomado, e a segunda fica **inalcançável e muda**.
pub(crate) fn projectar_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("45")
}

/// **A ALTURA da placa, e ela é DERIVADA de uma medição** — ver o cabeçalho.
///
/// ⭐⭐ **O número sai de uma conta com três entradas, nenhuma escolhida:** o
/// ponto da bola mais perto da câmara de fábrica está em `z ≈ 0,775` (o raio
/// vezes o cosseno de elevação do enquadramento), o raio do pincel de fábrica
/// mede `0,345` em espaço de objecto, e o raio da vista desce `0,775` por
/// unidade que anda ⇒ para o barro viajar **um raio de pincel** dali, a placa
/// tem de estar em `0,775 − 0,345 × 0,775 ≈ 0,51`. ⇒ `0,40` põe a coluna do
/// meio do canvas em `d/R ≈ 1,4` e o topo da calota em `2,2`.
///
/// ⚠️ **Ela CORTA a bola, e isso não é um acidente da escolha — é o que a
/// geometria obriga:** o raio da vista entra pela superfície virada ao artista e
/// tem de atravessar a peça para chegar a um alvo do outro lado, logo *um alvo
/// externo a uma peça convexa custa SEMPRE, no mínimo, a espessura dela*. Com a
/// bola a medir `2` de diâmetro e o pincel `0,345`, nenhuma posição de uma placa
/// **externa** dá um deslocamento comparável ao carimbo.
///
/// ⛔ **E ela tem de ficar abaixo de `0,775`**, senão a placa passa a estar
/// entre a câmara e a calota: ali o `aim` do pen-down escolheria a PLACA como
/// peça activa e o artista esculpiria a mesa.
pub(crate) const ALTURA_DA_PLACA: f32 = 0.40;

/// Meia-largura da placa — ver a varredura no gate.
///
/// ⚠️ **Ela tem de ser MAIOR que o raio da bola no corte** (`√(1 − 0,40²) =
/// 0,917`), senão a placa fica **dentro** da peça e o artista não vê alvo
/// nenhum — *um pincel que mede a distância a uma coisa invisível parece
/// partido*.
pub(crate) const LADO_DA_PLACA: f32 = 1.2;

/// **A PLACA** — um quadrado grande e plano, virado para cima.
///
/// ⚠️ **Dois triângulos bastam, e isso não é preguiça:** o raio acerta numa
/// face, e o que decide o resultado é onde o PLANO está, não quantas faces ele
/// tem. Uma placa subdividida seria mais cara sem mudar um vértice da resposta.
pub(crate) fn placa() -> Mesh {
    let l = LADO_DA_PLACA;
    Mesh::from_parts(
        vec![[-l, -l, 0.0], [l, -l, 0.0], [l, l, 0.0], [-l, l, 0.0]],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("a placa da cena =45")
}

/// As peças EXTRA desta cena — a placa, por baixo da bola.
pub(crate) fn scene_objects() -> Option<Vec<(Mesh, Pose)>> {
    projectar_scene().then(|| vec![(placa(), Pose::new([0.0, 0.0, ALTURA_DA_PLACA], 1.0))])
}

/// O roteiro da `=45`.
pub(crate) fn announce() {
    if !projectar_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =45 PROJECTAR NA CENA -- moldar uma peca CONTRA outra\n\
         [sculpt3d]    Este pincel nao empurra pela normal nem pelo gesto: ele mede a\n\
         [sculpt3d]    distancia ate' OUTRA PECA e leva o barro ate' la'. E' a ferramenta\n\
         [sculpt3d]    de encostar uma coisa na outra -- apertar um rosto contra uma\n\
         [sculpt3d]    parede, deixar uma peca rente a uma mesa.\n\
         [sculpt3d]\n\
         [sculpt3d]    A cena tem DUAS pecas: uma MESA quadrada e uma bola MEIO ENTERRADA\n\
         [sculpt3d]    nela. O que voce ve' da bola e' a CALOTA que sobressai.\n\
         [sculpt3d]\n\
         [sculpt3d]    Abra o painel com a CRASE (`). O pincel chama-se `Scene Project`\n\
         [sculpt3d]    e esta' no FIM da fileira.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha-o e arraste EM CIMA DA CALOTA (a parte redonda que\n\
         [sculpt3d]        sobressai da mesa -- nao na mesa).\n\
         [sculpt3d]        -> O barro desce e fica RENTE a' mesa: onde voce passou, a\n\
         [sculpt3d]           calota fica CHATA, ao nivel dela.\n\
         [sculpt3d]    (2) Segure Ctrl e arraste na calota outra vez.\n\
         [sculpt3d]        -> Agora o barro SOBE em vez de descer: o Ctrl inverte o\n\
         [sculpt3d]           movimento, e a calota AFASTA-SE da mesa em vez de encostar\n\
         [sculpt3d]           nela. Solte o Ctrl a seguir.\n\
         [sculpt3d]    (3) Rode a camera ate' ver a peca POR BAIXO da mesa e arraste na\n\
         [sculpt3d]        calota.\n\
         [sculpt3d]        -> Nada acontece, e e' o certo: o raio passou a apontar para\n\
         [sculpt3d]           longe da mesa, e este pincel nunca empurra para o infinito\n\
         [sculpt3d]           -- sem alvo, o barro fica quieto.\n\
         [sculpt3d]        Agora ligue `Search Both Ways` no painel e arraste outra vez.\n\
         [sculpt3d]        -> Volta a funcionar: ele passa a procurar tambem PARA TRAS e\n\
         [sculpt3d]           encontra a mesa.\n\
         [sculpt3d]    (4) Desligue `Search Both Ways` e volte a ver a peca por cima.\n\
         [sculpt3d]    (5) Baixe o `Strength` para metade: o efeito fica um QUARTO do\n\
         [sculpt3d]        caminho, nao metade. E' de proposito, em toda esta familia.\n\
         [sculpt3d]    (6) SUBA o `Gap` para 0,20 e passe outra vez.\n\
         [sculpt3d]        -> O barro passa a parar ANTES da mesa, deixando uma folga.\n\
         [sculpt3d]        ATENCAO -- suba o `Gap` ate' ao MAXIMO e passe:\n\
         [sculpt3d]        -> Agora o barro AFASTA-SE da mesa em vez de se aproximar.\n\
         [sculpt3d]           Isto NAO e' um defeito: e' o comportamento da ferramenta de\n\
         [sculpt3d]           referencia, e esta' aqui para voce decidir se o quer assim.\n\
         [sculpt3d]           Se preferir que ele apenas PARE (nunca inverta), diga.\n\
         [sculpt3d]    (7) Ponha o `Gap` de volta a zero, troque `Ray Direction` de `View`\n\
         [sculpt3d]        para `Surface` e passe perto da BORDA da calota.\n\
         [sculpt3d]        -> `View` empurra para dentro do ecra (segue os seus olhos);\n\
         [sculpt3d]           `Surface` empurra para dentro da peca (segue a forma dela).\n\
         [sculpt3d]           Rode a camera e repita: o `View` muda com a camera, o\n\
         [sculpt3d]           `Surface` nao.\n\
         [sculpt3d]    (8) Ctrl+Z desfaz cada passagem.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: no (1) a calota RASGAR (riscos escuros a atravessar\n\
         [sculpt3d]    a peca), inchar, encolher ou o barro atravessar a mesa; se no (2)\n\
         [sculpt3d]    o barro DESCER outra vez em vez de subir; se no (3) alguma coisa se\n\
         [sculpt3d]    mexer antes de ligar `Search Both Ways`, ou nada se mexer depois;\n\
         [sculpt3d]    ou se o app fechar sozinho."
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
                crate::scenes::CENAS >= 45,
                "o tecto do roteador tem de conter esta cena (=45)"
            );
        }
    }

    /// ⭐⭐⭐ **A CENA CONTÉM O FENÓMENO, e a afirmação é GEOMÉTRICA: a placa
    /// corta a bola, e o que sobra dela é uma CALOTA que o artista vê.**
    ///
    /// ⚠️ **Sem esta afirmação o roteiro ensinaria o contrário do que acontece**
    /// (a espécie que o `CLAUDE.md` §5.0 chama de pior que uma cena ausente), e
    /// em **três** maneiras diferentes:
    ///
    /// * com a placa **abaixo** da bola, o raio da vista atravessa a peça
    ///   inteira e o barro viaja `5,7`–`7,9` raios de pincel ⇒ o passo (1)
    ///   mostraria a peça a RASGAR (o report do dono);
    /// * com ela **acima** do ponto da bola mais perto da câmara, ela fica
    ///   entre o artista e a calota ⇒ o pen-down escolheria a **placa** como
    ///   peça activa;
    /// * mais estreita que o raio do corte, ela fica **dentro** da bola ⇒ não há
    ///   alvo à vista.
    ///
    /// ⚠️ **A régua da profundidade vive na sonda** (`diag_a_regua_da_cena`, que
    /// mede o `d/R` no caminho do produto com a câmara de fábrica): esta aqui
    /// afirma as três cercas que são função só de `ALTURA_DA_PLACA` e
    /// `LADO_DA_PLACA`, e por isso corre **sempre**, sem GPU.
    #[test]
    fn a_placa_corta_a_bola_e_deixa_uma_calota_a_vista() {
        let h = super::ALTURA_DA_PLACA;
        assert!(
            (-1.0..1.0).contains(&h),
            "a placa em {h} não corta a bola de raio 1 — com ela FORA da peça o \
             raio da vista atravessa a peça inteira e o passo (1) mostra um rasgo"
        );
        // ⚠️ **`0,775` é o `z` do ponto da bola mais perto da câmara de
        // fábrica** — não um número escolhido: é o cosseno de elevação do
        // enquadramento, e a sonda imprime-o (`olho (mundo)`).
        assert!(
            h < 0.775,
            "a placa em {h} fica entre a câmara e a calota: o pen-down passaria \
             a escolher a MESA como peça activa"
        );
        let raio_do_corte = (1.0 - h * h).sqrt();
        assert!(
            super::LADO_DA_PLACA > raio_do_corte,
            "a placa ({}) cabe dentro da bola no corte ({raio_do_corte:.3}) — o \
             artista não veria alvo nenhum",
            super::LADO_DA_PLACA
        );
        // ⭐ E a folga tem de conseguir ULTRAPASSAR a viagem, senão o passo (6)
        // não demonstra a armadilha da §6.4: o slider pára em `1,0` e a viagem
        // mais longa é a do topo da calota.
        let viagem_maxima = (1.0 - h) / 0.775;
        assert!(
            viagem_maxima < 1.0,
            "a viagem mais longa é {viagem_maxima:.3} e o slider da folga pára \
             em 1,0 — o passo (6) não conseguiria demonstrar a inversão"
        );
    }
}

/// **O pincel no GESTO e na CENA** — ver [`tests`].
#[cfg(test)]
#[path = "projectar_tests.rs"]
mod gesto_tests;
