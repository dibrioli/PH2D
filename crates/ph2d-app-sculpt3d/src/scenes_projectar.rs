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
//! # ⭐⭐ A placa é PLANA de propósito, e a bola está POR CIMA dela
//!
//! *Projectar é a ferramenta de encostar uma coisa na outra* — apertar um rosto
//! contra uma parede, sentar uma peça numa mesa. Uma placa plana torna o
//! resultado **legível sem medir**: ou o barro ficou chato ao nível dela, ou não
//! ficou.
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

/// Quanto a placa fica ABAIXO da bola, em unidades de mundo.
///
/// ⭐ **Escolhida para o vão ser MAIOR que o raio da bola:** com a placa a
/// `−1,6` e a bola de raio `1`, o barro tem de viajar `0,6` até encostar — um
/// deslocamento que se vê de longe, e que a folga de `1,0` do slider consegue
/// ultrapassar (é isso que torna a armadilha da §6.4 demonstrável no passo (6)).
pub(crate) const ALTURA_DA_PLACA: f32 = -1.25;

/// Meia-largura da placa — ver a varredura no gate.
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
         [sculpt3d]    de encostar uma coisa na outra -- sentar uma peca numa mesa,\n\
         [sculpt3d]    apertar um rosto contra uma parede.\n\
         [sculpt3d]\n\
         [sculpt3d]    A cena tem DUAS pecas: a bola (que voce esculpe) e uma PLACA\n\
         [sculpt3d]    plana por baixo dela.\n\
         [sculpt3d]\n\
         [sculpt3d]    Abra o painel com a CRASE (`). O pincel chama-se `Scene Project`\n\
         [sculpt3d]    e esta' no FIM da fileira.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha-o e arraste na parte de BAIXO da bola (o lado virado\n\
         [sculpt3d]        para a placa).\n\
         [sculpt3d]        -> O barro desce e fica CHATO, ao nivel da placa. A bola ganha\n\
         [sculpt3d]           uma base plana, como se estivesse pousada.\n\
         [sculpt3d]    (2) Rode a camera e arraste no TOPO da bola.\n\
         [sculpt3d]        -> Nada acontece. E' o certo: dali o raio vai para cima e nao\n\
         [sculpt3d]           encontra a placa. Este pincel nunca empurra para o infinito.\n\
         [sculpt3d]    (3) Ainda no topo, ligue `Search Both Ways` no painel e arraste.\n\
         [sculpt3d]        -> Agora acontece: ele passa a procurar tambem PARA TRAS, e o\n\
         [sculpt3d]           barro do topo desce ate' a placa.\n\
         [sculpt3d]    (4) Desligue-o outra vez e volte a esculpir por baixo.\n\
         [sculpt3d]    (5) Baixe o `Strength` para metade: o efeito fica um QUARTO do\n\
         [sculpt3d]        caminho, nao metade. E' de proposito, em toda esta familia.\n\
         [sculpt3d]    (6) SUBA o `Gap` para 0,20 e passe outra vez.\n\
         [sculpt3d]        -> O barro passa a parar ANTES da placa, deixando uma folga.\n\
         [sculpt3d]        ATENCAO -- suba o `Gap` ate' ao MAXIMO e passe:\n\
         [sculpt3d]        -> Agora o barro AFASTA-SE da placa em vez de se aproximar.\n\
         [sculpt3d]           Isto NAO e' um defeito: e' o comportamento da ferramenta de\n\
         [sculpt3d]           referencia, e esta' aqui para voce decidir se o quer assim.\n\
         [sculpt3d]           Se preferir que ele apenas PARE (nunca inverta), diga.\n\
         [sculpt3d]    (7) Troque `Ray Direction` de `View` para `Surface` e passe numa\n\
         [sculpt3d]        parte inclinada da bola.\n\
         [sculpt3d]        -> `View` empurra para dentro do ecra (segue os seus olhos);\n\
         [sculpt3d]           `Surface` empurra para dentro da peca (segue a forma dela).\n\
         [sculpt3d]           Rode a camera e repita: o `View` muda com a camera, o\n\
         [sculpt3d]           `Surface` nao.\n\
         [sculpt3d]    (8) Segure Ctrl e arraste.\n\
         [sculpt3d]        -> Ele procura do OUTRO LADO. Nao e' «afastar»: e' virar o raio.\n\
         [sculpt3d]    (9) Ctrl+Z desfaz cada passagem.\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: no (1) a bola inchar, encolher ou o barro atravessar\n\
         [sculpt3d]    a placa; se no (2) alguma coisa se mexer; se no (3) nao acontecer\n\
         [sculpt3d]    nada; ou se o app fechar sozinho."
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

    /// ⭐⭐ **A CENA CONTÉM O FENÓMENO: a placa está ABAIXO da bola, e o vão é
    /// maior que o raio dela.**
    ///
    /// ⚠️ **Sem esta afirmação o roteiro ensinaria o contrário do que acontece:**
    /// com a placa a atravessar a bola, o passo (1) mostraria barro a subir; com
    /// ela longe de mais, o passo (6) não conseguiria demonstrar a armadilha da
    /// folga, porque o slider pára em `1,0`.
    #[test]
    fn a_placa_esta_abaixo_da_bola_e_ao_alcance_da_folga() {
        let vao = -super::ALTURA_DA_PLACA - 1.0;
        assert!(
            vao > 0.0,
            "a placa atravessa a bola: o passo (1) mostraria o contrário do que diz"
        );
        assert!(
            vao < 1.0,
            "o vão é {vao} e o slider da folga pára em 1,0 — o passo (6) não \
             conseguiria demonstrar a armadilha da §6.4"
        );
    }
}

/// **O pincel no GESTO e na CENA** — ver [`tests`].
#[cfg(test)]
#[path = "projectar_tests.rs"]
mod gesto_tests;
