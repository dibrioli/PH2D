//! **O QUE O ANEL DE UM VÉRTICE DIZ** — a média das posições congeladas e a
//! normal que o [`Verb::SlideRelax`] remove.
//!
//! ⚠️ **O corte é por PERGUNTA, não por tamanho:** o irmão [`super::target`]
//! responde *para onde cada verbo aponta* (o `match` de vinte braços), e isto
//! responde *o que a vizinhança deste vértice é*. Os dois consumidores de aqui
//! — o Smooth e o Slide Relax — caminham para a MESMA média e diferem numa
//! subtração, então a média e a normal que a qualifica pertencem juntas.
//!
//! ⚠️ **Filho (`#[path]`) de [`super`] pelo mesmo motivo do irmão:** estes
//! métodos leem os campos privados do [`SculptStroke`] (o `pre` congelado), e um
//! módulo irmão os obrigaria a virar `pub(crate)` — a visibilidade viraria
//! função do TAMANHO do arquivo, que é o oposto do que o teto de LOC faz.
//!
//! ⚠️ **E um doc-comment PARTIDO AO MEIO veio junto na mudança:** o
//! `neighbour_average` carregava, empilhada sobre a própria descrição, a
//! primeira linha de um cabeçalho ANTIGO (*"a média do anel — o alvo do Smooth,
//! e o oposto do Sharpen"*), cujo corpo tinha sido substituído havia tempo. Não
//! era desta linha; foi corrigido aqui porque o bloco estava a ser reescrito de
//! qualquer maneira. *Um comentário que descreve o vizinho é pior que comentário
//! nenhum.*

use super::*;

impl SculptStroke {
    /// A média das posições **congeladas** do anel de `v`.
    ///
    /// Ler o `pre` e não o vivo é o que torna o Smooth idempotente: um traço que
    /// passa duas vezes no mesmo lugar suaviza uma vez, e a superfície não
    /// derrete enquanto o artista segura o botão parado.
    ///
    /// ⚠️ **A LEI mora em [`ph2d_mesh::ring_average`], e não aqui.** Ela tem as
    /// duas regras de borda que uma malha aberta exige, e ganhou um SEGUNDO
    /// consumidor quando o **extract** passou a relaxar a costura de uma peça
    /// recém-cortada: é o mesmo laplaciano, e duas cópias divergiriam exatamente
    /// no lugar que já custou uma medição (a boca do `open_tube3` sugada de 2
    /// para 1,3597 em seis passes).
    ///
    /// ⚠️ **O que fica AQUI é de onde a posição vem:** o traço lê o `pre`
    /// CONGELADO no pen-down, e é isso que o torna idempotente — um pincel
    /// parado suaviza uma vez, e a superfície não derrete enquanto o artista
    /// segura o botão. O extract entrega a posição viva pela mesma porta.
    pub(super) fn neighbour_average(&self, mesh: &Mesh, v: u32, at: [f32; 3]) -> [f32; 3] {
        ph2d_mesh::ring_average(mesh.adjacency(), v, at, |nb| mesh.positions()[nb as usize])
    }

    /// **A normal que o [`Verb::SlideRelax`] REMOVE** — a do vértice no miolo, a
    /// **bissetriz das arestas de beira** numa borda.
    ///
    /// ⚠️ **O ramo da borda é o caso NORMAL, e não a exceção que a leitura do
    /// `sculpt_smooth.cc:558` sugere:** a condição lá é *"borda **e** exactamente
    /// dois vizinhos sobreviveram ao filtro"*, e numa malha manifold a curva de
    /// borda é um LOOP FECHADO ⇒ **todo** vértice dela tem exactamente dois
    /// vizinhos de borda (medido: 12 de 12 no `open_tube3`, o mesmo número que o
    /// [`ph2d_mesh::ring_average`] regista do outro lado da mesma regra).
    ///
    /// ⚠️ **E ela é LOAD-BEARING, não cerimónia:** com a normal da SUPERFÍCIE, o
    /// plano tangente de um vértice de beira **contém** a corda entre os dois
    /// vizinhos, então a média (que é o ponto médio dessa corda) sobrevive
    /// inteira à subtração e a beira é puxada para dentro dela mesma, dab a dab.
    /// A bissetriz aponta para DENTRO da superfície; removê-la deixa só o
    /// movimento AO LONGO da beira, que é o que redistribuir significa ali.
    ///
    /// **`None` = não mexa neste vértice**, e são os três casos em que a
    /// referência devolve translação zero: anel de dois ou menos (o
    /// `neighbors.size() <= 2` de `:539`), nenhum vizinho de borda sobreviveu
    /// (`:559`), e a bissetriz DEGENERADA (`math::is_zero`, `:568`) — que
    /// geometricamente é a beira perfeitamente RETA, onde não existe *"para
    /// dentro"* a proteger.
    ///
    /// ⚠️ **Uma beira quase-reta NÃO cai no `None`, e isso é a diferença entre
    /// um limiar de degenerescência e um de comportamento:** a soma de dois
    /// unitários separados por `180° − δ` mede `≈ δ`, e a DIREÇÃO dela continua
    /// a ser a bissetriz exacta — pequena, não ruidosa. O `EPS` existe só para
    /// não dividir por zero.
    pub(super) fn relax_normal(&self, mesh: &Mesh, v: u32, at: [f32; 3]) -> Option<[f32; 3]> {
        const EPS: f32 = 1e-6;
        let adj = mesh.adjacency();
        let vi = v as usize;
        // ⚠️ **DEFESA EM CAMADAS, e a medição diz de que lado ela é redundante:**
        // o [`ph2d_mesh::ring_average`], uma chamada abaixo, guarda **o mesmo
        // predicado** (`valence <= 2 ⇒ devolve a base`), então com esta linha
        // apagada o delta continua zero e a suíte INTEIRA da crate fica verde
        // (249/249, medido). Ela fica porque este é um `fn` puro que **não sabe
        // responder** ali — um anel de dois não define plano nenhum — e porque a
        // camada tem gate próprio, que é o que impede as duas de caírem juntas.
        if adj.valence(vi) <= 2 {
            return None;
        }
        if !adj.is_border(vi) {
            return Some(mesh.normals()[vi]);
        }
        let mut acc = [0.0f32; 3];
        let mut n = 0u32;
        for &nb in adj.vert_verts.neighbours(vi) {
            if !adj.is_border(nb as usize) {
                continue;
            }
            let p = mesh.positions()[nb as usize];
            let d = [p[0] - at[0], p[1] - at[1], p[2] - at[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if len <= EPS {
                continue;
            }
            for k in 0..3 {
                acc[k] += d[k] / len;
            }
            n += 1;
        }
        if n == 0 {
            return None;
        }
        // ⚠️ **Fora de DOIS a referência volta à normal do vértice** (`:573`),
        // e não à bissetriz de três — inalcançável em malha manifold, mas o
        // `from_obj` pode trazer uma que não é.
        if n != 2 {
            return Some(mesh.normals()[vi]);
        }
        let len = (acc[0] * acc[0] + acc[1] * acc[1] + acc[2] * acc[2]).sqrt();
        if len <= EPS {
            return None;
        }
        Some([acc[0] / len, acc[1] / len, acc[2] / len])
    }
}
