//! ⛔⛔⛔ **PORQUE É QUE ESTE GESTO NÃO VAI FAZER NADA** — a porta única das
//! recusas do pen-down.
//!
//! # O defeito que ela cura, e ele tem três reports do dono
//!
//! *«não vejo efeito com density»* · *«os outros 2 botões ainda não funcionam»* ·
//! *«resultado bem bizarro»*. Em dois dos três o pincel estava **certo** e o que
//! faltava era o app **dizer porquê**.
//!
//! ⚠️⚠️ **Um pincel que não faz nada e não diz porquê é indistinguível de um
//! pincel partido**, e o artista tira a conclusão cara: ele conclui que a
//! ferramenta não funciona, e não que falta a ENTRADA dela.
//!
//! # Porque é UMA porta e não três `if`s espalhados
//!
//! Antes desta, o motor tinha **duas** recusas em voz alta — a da pilha de
//! multiresolução (`history_referencia`) e a do passe de topologia
//! (`dyntopo::queixa_do_passe`) — e **duas** ausências: o pincel de contorno numa
//! peça fechada, e os dois que pedem outra peça na cena.
//!
//! ⭐ **A família é exactamente a dos predicados `precisa_d*` do motor**, e o
//! censo [`tests::toda_porta_de_entrada_ausente_tem_recusa`] deriva-a do ficheiro
//! que os declara: um predicado novo daquela família **reprova** até alguém lhe
//! dar voz. *É a diferença entre uma lista que alguém tem de se lembrar de
//! estender e uma que não fica verde sem a extensão.*
//!
//! ⛔ **A do `dyntopo` fica onde está, e não é inconsistência:** ela não é um
//! facto do PEN-DOWN — o passe de topologia só sabe que não mudou nada **depois**
//! de correr. *Uma recusa que se pode prever antes do gesto e uma que só se
//! descobre depois dele são duas perguntas.*
//!
//! # ⚠️ A lei é PURA, e a razão é um gate
//!
//! A primeira redacção pôs a lei num método da cena — e a cena pede um
//! `wgpu::Device`, logo o gate dela nasceria `#[ignore]` e **o CI nunca o
//! correria**. *Quando um gate precisa de um device para medir uma decisão que
//! não tem pixel nenhum, a lei está no sítio errado.*

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::Brush;

use super::Sculpt3dScene;

/// **O QUE O PEN-DOWN SABE**, do ponto de vista das entradas que podem faltar.
pub(crate) struct Entradas<'a> {
    pub brush: &'a Brush,
    /// Há um nível ABAIXO de onde tirar deslocamento?
    pub tem_referencia: bool,
    /// A peça que se vai esculpir.
    pub mesh: &'a Mesh,
    /// Quantas **outras** peças há na cena.
    pub outras_pecas: usize,
}

/// **A peça tem bordo aberto?**
///
/// ⚠️ **`O(V)` com saída antecipada, e só é chamada para UM verbo no pen-down** —
/// numa peça aberta ela pára no primeiro vértice de beira. O pior caso é a peça
/// **fechada**, que é precisamente o caso em que a resposta importa; ao lado, a
/// fotografia da superfície do pen-down do `SceneProject` já custa `0,70 ms` na
/// peça de fábrica.
///
/// ⛔ **Não usa o `manifold::border_edges`**, que constrói um mapa de TODAS as
/// arestas: aqui a pergunta é *«existe alguma?»*, e a adjacência já está
/// construída.
fn tem_bordo_aberto(mesh: &Mesh) -> bool {
    let adj = mesh.adjacency();
    (0..mesh.vert_count()).any(|v| adj.is_border(v))
}

impl Entradas<'_> {
    /// **A RECUSA** — `None` quando o gesto tem tudo o que pede.
    ///
    /// ⚠️ **Devolve o texto em vez de o imprimir**, e isso é o que a torna
    /// gateável: um `eprintln!` no meio da cadeia só se mede por captura de
    /// saída, e a metade que interessa — *a razão certa para o facto certo* —
    /// fica fora de qualquer teste.
    pub(crate) fn recusa(&self) -> Option<String> {
        let verbo = self.brush.verb;
        let nome = verbo.label();
        // ⭐ **A ordem é da mais ESPECÍFICA para a mais geral**, porque um gesto
        // pode falhar por mais de uma razão e o artista só lê a primeira: dizer
        // *«falta uma pilha»* a quem também não tem bordo seria mandá-lo
        // resolver a metade errada.
        if verbo.precisa_de_referencia() && !self.tem_referencia {
            return Some(format!(
                "{nome} precisa de uma pilha de multiresolucao -- sem um nivel \
                 ABAIXO nao ha' deslocamento nenhum (K subdivide, ',' desce)"
            ));
        }
        if verbo.precisa_de_bordo_aberto() && !tem_bordo_aberto(self.mesh) {
            return Some(format!(
                "{nome} trabalha a BEIRA de uma peca aberta -- esta peca e' \
                 fechada, e a regiao dele comeca na borda (experimente uma \
                 tigela, ou apague faces para abrir uma boca)"
            ));
        }
        if self.brush.precisa_das_pecas_da_cena() && self.outras_pecas == 0 {
            return Some(format!(
                "{nome} precisa de OUTRA peca na cena -- ele empurra o barro \
                 ate' encostar nela, e aqui so' ha' uma"
            ));
        }
        None
    }
}

impl Sculpt3dScene {
    /// Imprime a recusa, se houver. **O único consumidor é o pen-down.**
    pub(crate) fn diz_a_recusa_do_pen_down(&self) {
        let Some(o) = self.obj() else {
            return;
        };
        let entradas = Entradas {
            brush: &self.brush,
            // ⚠️ **A MESMA porta que fotografa a referência**, e não uma segunda
            // pergunta sobre a pilha: duas respostas divergiriam no dia em que
            // uma delas ganhasse uma cerca.
            tem_referencia: self.superficie_de_referencia().is_some(),
            mesh: o.stack.mesh(),
            outras_pecas: self.objects.len().saturating_sub(1),
        };
        if let Some(motivo) = entradas.recusa() {
            eprintln!("[sculpt3d] {motivo}");
        }
    }
}

#[cfg(test)]
#[path = "recusa_tests.rs"]
mod tests;
