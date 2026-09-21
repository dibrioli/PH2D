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
    /// Quantas **outras** peças há na cena **que o pincel pode ver** — ver
    /// [`super::Sculpt3dScene::alvos_visiveis`].
    pub outras_pecas: usize,
    /// ⭐⭐ **Quantas há que ele NÃO pode ver** (o olho fechado na Hierarquia, ou
    /// o isolamento).
    ///
    /// ⚠️ **Ela existe para a recusa poder dizer a CURA e não só o facto.**
    /// *«Não há outra peça»* e *«a outra peça está escondida»* levam o artista a
    /// gestos opostos — um vai criar geometria, o outro vai abrir um olho —, e
    /// as duas leem-se exactamente igual num contador só.
    pub outras_escondidas: usize,
    /// ⭐⭐⭐ **A TINTA FINA está armada nesta peça?** — ver
    /// [`crate::tinta_da_peca`].
    ///
    /// ⚠️⚠️ **Ela é a PRIMEIRA entrada desta família que não é uma AUSÊNCIA:**
    /// as três acima dizem *«falta-te uma coisa»*, e esta diz *«o que vais
    /// fazer vai CUSTAR uma que tu tens»*. É por isso que ela não impede o
    /// gesto — ela põe o preço à vista antes de ele ser pago, que é a única
    /// coisa que separa *perder detalhe* de *perder detalhe sem saber porquê*.
    pub tinta_fina_armada: bool,
    /// O passe de topologia está armado?
    pub dyntopo_armado: bool,
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
            return Some(ph2d_i18n::tr_with(
                "app.sculpt3d.recusa.precisa_de_uma_pilha",
                &[("nome", &nome)],
            ));
        }
        if verbo.precisa_de_bordo_aberto() && !tem_bordo_aberto(self.mesh) {
            return Some(ph2d_i18n::tr_with(
                "app.sculpt3d.recusa.trabalha_a_beira_de_uma_peca_aberta",
                &[("nome", &nome)],
            ));
        }
        if self.brush.precisa_das_pecas_da_cena() && self.outras_pecas == 0 {
            // ⭐ A CURA, e não só o facto: o artista que ouve *«nao ha' outra
            // peca»* vai criar geometria; quem ouve *«esta' escondida»* vai
            // abrir um olho. As duas leem-se igual num contador só.
            if self.outras_escondidas > 0 {
                let plural = if self.outras_escondidas == 1 { "" } else { "s" };
                return Some(ph2d_i18n::tr_with(
                    "app.sculpt3d.recusa.precisa_de_outra_peca_a_vista",
                    &[("nome", &nome), ("plural", &plural)],
                ));
            }
            return Some(ph2d_i18n::tr_with(
                "app.sculpt3d.recusa.precisa_de_outra_peca_na_cena",
                &[("nome", &nome)],
            ));
        }
        // ⭐⭐⭐ **O AVISO, e ele vem por ÚLTIMO de propósito:** as três acima
        // são gestos que não vão fazer nada, e este é um gesto que vai fazer
        // mais do que o artista pediu. Dizê-lo antes seria pôr um preço à
        // frente de uma impossibilidade.
        //
        // ⚠️ **A pergunta é sobre o VERBO e não sobre o modo** — o passe corre
        // a pedido de quem refina OU colapsa, e quem não faz nem uma coisa nem
        // outra deixa a topologia em paz mesmo com o interruptor ligado. *Uma
        // lente mais larga que a do consumidor põe este aviso em todo traço, e
        // um aviso que soa sempre é ruído que o artista aprende a ignorar —
        // exactamente quando ele passar a ser verdade.*
        if self.tinta_fina_armada
            && self.dyntopo_armado
            && (verbo.refina_no_dyntopo() || verbo.colapsa_no_dyntopo())
        {
            return Some(ph2d_i18n::tr_with(
                "app.sculpt3d.recusa.a_tinta_fina_perde_detalhe_com_topologia",
                &[("nome", &nome)],
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
            // ⚠️⚠️ **A MESMA porta que enche a lista** — ver
            // [`super::Sculpt3dScene::alvos_visiveis`]. Contar `objects.len() −
            // 1` aqui e filtrar lá faria um traço cujo único alvo está
            // ESCONDIDO mover zero vértices **e ficar calado**, que é
            // precisamente o defeito que este módulo existe para não ter.
            outras_pecas: self.alvos_visiveis().count(),
            outras_escondidas: self.objects.len().saturating_sub(1) - self.alvos_visiveis().count(),
            // ⚠️ **A pergunta é sobre a PEÇA e não sobre o knob da cena:** o
            // plano nasce na peça activa e sobrevive nela, logo um knob armado
            // com a peça ainda sem plano não custa detalhe nenhum — e o inverso
            // também é verdade (desarmar o knob não apaga o plano até o quadro
            // seguinte reconciliar). *O que se perde é o que existe.*
            tinta_fina_armada: o.tinta.is_some(),
            dyntopo_armado: self.dyntopo.armed,
        };
        if let Some(motivo) = entradas.recusa() {
            eprintln!("[sculpt3d] {motivo}");
        }
    }
}

#[cfg(test)]
#[path = "recusa_tests.rs"]
mod tests;
