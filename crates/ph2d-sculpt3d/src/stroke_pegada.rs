//! **A PEGADA CONGELADA DO PEN-DOWN** — o que ela é, e como ela atravessa uma
//! malha que muda de tamanho debaixo do traço.
//!
//! Filho (`#[path]`) do [`super`], irmão do [`super::growth`]. ⚠️ **O corte é de
//! ASSUNTO e não de tamanho**, e os dois assuntos são de facto dois: lá mora *o
//! `pre` de cada vértice a sobreviver à topologia*, aqui *o CONJUNTO de vértices
//! do gesto a sobreviver a ela*. São peças diferentes do estado de um traço, com
//! leis diferentes — o `pre` **herda-se** de pais, a pegada **acolhe** e
//! **larga**.
//!
//! ⚠️ O corte foi FORÇADO pelo tecto de LOC (o pai chegou a `739` de `700` no
//! dia em que o polegar passou a subdividir) e é melhor por isso: a pegada tinha
//! a declaração num ficheiro, a razão de existir noutro e as duas metades da
//! manutenção num terceiro. ⛔ *Subir o número em vez de cortar é o que o
//! `CLAUDE.md` §2 proíbe por escrito.*

use super::SculptStroke;
use ph2d_mesh::Birth;

/// **UMA PEGADA CONGELADA NO PEN-DOWN** — uma por passagem de simetria.
///
/// ⚠️⚠️ **Ela existe porque a pegada normal sai das posições VIVAS, e num gesto
/// que desloca o barro `0,38` num pincel de raio `0,35` isso deixa de ser
/// inócuo:** os vértices que o próprio gesto levou saem do raio da consulta, o
/// conjunto amostrado encolhe, e a normal da área — que é uma média sobre ele —
/// **muda com o comprimento do traço**. O efeito é invisível num plano (ali toda
/// normal é a mesma) e MEDIDO numa esfera: `8,5e-3` de desvio contra o oráculo
/// no traço inteiro, contra `1,5e-4` truncado a 8 eventos.
///
/// ⚠️ **Só o [`crate::Verb::Thumb`] a congela hoje**, e a cerca é deliberada: o
/// [`crate::Verb::Move`] tem a mesma forma e o mesmo defeito **provável**, mas é
/// um verbo que já shipa, com corpus próprio por correr — mudá-lo seria alterar
/// produto a partir de uma inferência. *A fixture que o decide existe* (o
/// oráculo gravou o agarrar), e a pergunta está nomeada.
///
/// ⛔⛔ **A CHAVE É O CENTRO, e ela nasceu de um gate que reprovou:** a primeira
/// versão guardava UMA pegada por traço, e a simetria corre o mesmo traço
/// espelhado — a segunda passagem reusava a pegada da primeira e só metade da
/// malha se mexia. O censo
/// `every_verb_inherits_symmetry_from_the_one_place_it_is_expanded` apanhou no
/// minuto seguinte. ⇒ uma entrada por PASSAGEM, e o centro espelhado é a
/// identidade natural dela: constante ao longo do gesto, distinto entre
/// passagens, sem ninguém ter de propagar um índice até aqui.
///
/// # ⚠️⚠️ Os dois vetores são o MESMO conjunto por duas ordens, e as duas são load-bearing
///
/// - `verts` está na **ordem da consulta**, que é a ordem em que o octree os
///   devolveu — e é ela que o dab clona. ⛔ Ordená-lo mudaria a ordem das somas
///   em `f32` da normal da área, que é precisamente a grandeza que esta pegada
///   existe para estabilizar: *uma cura que mexe nos bits da coisa que ela
///   protege não é uma cura.*
/// - `ordenada` é o **índice** — a mesma lista por valor, para o
///   [`SculptStroke::cresce_a_pegada_congelada`] perguntar *«este pai já estava
///   dentro?»* em `log n` em vez de varrer a pegada por cada vértice que nasce.
///
/// ⭐ **E o índice fica ordenado de graça no refino:** o motor APENDA, logo todo
/// vértice novo tem índice maior que qualquer um que já lá estava — empurrá-lo
/// para o fim mantém a ordem. Só o colapso a quebra, e é lá que ele é refeito.
#[derive(Clone, Debug)]
pub(super) struct PegadaCongelada {
    /// O centro da passagem — a chave, e o porquê está no cabeçalho.
    pub(super) centro: [f32; 3],
    /// Os vértices na ORDEM DA CONSULTA. É esta lista que o dab clona.
    pub(super) verts: Vec<u32>,
    /// Os mesmos vértices por VALOR — o índice de pertença. Ver o cabeçalho.
    pub(super) ordenada: Vec<u32>,
}

impl SculptStroke {
    /// **A PEGADA CONGELADA ACOLHE OS VÉRTICES QUE NASCERAM DENTRO DELA.**
    ///
    /// ⭐⭐⭐ **Ordem do dono (2026-09-14): *«Thumb se for possível, deveria
    /// subdividir»*** — e isto é o *«se for possível»*. O polegar é o único
    /// verbo que congela a pegada, e sem esta função ligá-lo à topologia
    /// dinâmica daria o pior dos dois mundos: o refino cria superfície no meio
    /// do carimbo e o carimbo **não a vê**, deixando os vértices novos parados
    /// entre vizinhos deslocados — uma cratera de agulhas com a forma da malha
    /// nova.
    ///
    /// # ⭐⭐ A regra é *OS DOIS PAIS*, e ela é EXACTA e não conservadora
    ///
    /// A pegada é o resultado de uma consulta por ESFERA, e uma esfera é
    /// **convexa**: o vértice novo nasce no ponto médio dos pais, logo se os
    /// dois estavam dentro da bola ele está dentro dela também. Não há palpite
    /// nenhum nesta linha — há uma propriedade da forma consultada.
    ///
    /// ⛔ **A alternativa — *«um pai basta»* — está errada por duas vias.** Ela
    /// admite pontos médios que caem fora da bola (o pai de fora foi excluído
    /// pela consulta, logo o meio pode estar fora), e sobretudo faz a pegada
    /// **CRESCER para fora ao longo do gesto**: cada refino empurraria a
    /// fronteira mais um anel, e o conjunto amostrado voltaria a depender de
    /// quantos eventos o traço teve — que é **exactamente** o defeito que
    /// congelar a pegada existe para não ter. *Uma cura que reabre o defeito que
    /// a motivou não é uma cura.*
    ///
    /// ⚠️ **A ORDEM dos nascimentos é load-bearing aqui também**, pelo mesmo
    /// motivo da herança do `pre`: um vértice novo pode ser pai de outro no
    /// mesmo passe, e percorrer para a frente é o que garante que ele já está no
    /// índice quando o filho pergunta.
    pub(super) fn cresce_a_pegada_congelada(&mut self, births: &[Birth]) {
        // ⚠️ **Vazia é o caminho de OMISSÃO de todo verbo menos um.** Sem esta
        // linha, cada refino de cada traço pagaria a varredura dos nascimentos
        // por uma lista que nunca tem nada.
        if self.pegada_ancorada.is_empty() {
            return;
        }
        for b in births {
            for p in &mut self.pegada_ancorada {
                if p.ordenada.binary_search(&b.a).is_err()
                    || p.ordenada.binary_search(&b.b).is_err()
                {
                    continue;
                }
                p.verts.push(b.vert);
                // ⭐ O refino APENDA, logo `b.vert` é maior que tudo o que já
                // está no índice — o `push` mantém a ordem sem ordenar nada.
                p.ordenada.push(b.vert);
            }
        }
    }

    /// **A PEGADA CONGELADA ATRAVESSA UM COLAPSO** — a irmã exacta da
    /// [`Self::cresce_a_pegada_congelada`], e o que muda é que aqui não há nada
    /// a herdar: há uma **renumeração** a aplicar e mortos a largar.
    ///
    /// ⚠️⚠️ **Sem ela o polegar esculpiria com a pegada de OUTROS VÉRTICES.** Um
    /// índice guardado no pen-down não é um vértice: quando o colapso enche um
    /// buraco com um sobrevivente da cauda, aquele índice passa a nomear barro
    /// noutro sítio da peça. *Um índice sobrevive a uma renumeração; o que ele
    /// nomeava, não.*
    ///
    /// # ⛔⛔ A tradução é uma CADEIA, e a primeira redacção desta função supôs que não era
    ///
    /// O plano do colapso ([`ph2d_mesh::Remap`]) é uma **sequência de trocas**, e
    /// ela aplica-se por ordem: `(11 → 10)`, `(10 → 9)`, `(9 → 8)` quer dizer que
    /// o vértice que começou em `11` acaba em **`8`**, passando por dois
    /// endereços que não são o dele. ⚠️ *Um índice que é destino de uma troca
    /// pode ser origem da seguinte* — logo **não existe** a tabela plana
    /// `origem → destino` que eu escrevi primeiro, e com ela um índice do meio de
    /// uma cadeia ficava guardado como se fosse final.
    ///
    /// ⛔ **O modo de falha foi o bom:** um `index out of bounds` no primeiro dab
    /// a seguir a um colapso, dentro da máscara de alcance
    /// ([`crate::dab_alcance`]) — *a pegada guardava um índice maior que a
    /// malha*. Se a cadeia tivesse acabado **dentro** do novo tamanho, o carimbo
    /// teria simplesmente pegado no barro errado, em silêncio.
    ///
    /// # Porque a tradução é do tamanho do COLAPSO e nunca da malha
    ///
    /// Um vetor de tradução por VÉRTICE seria `O(malha)` por dab — numa peça de
    /// um milhão de vértices, megabytes a limpar por carimbo para traduzir umas
    /// centenas. A lista de movimentos já é do tamanho do que se mexeu, e dela
    /// saem **duas** tabelas, cada uma com a sua pergunta:
    ///
    /// | tabela | responde |
    /// |---|---|
    /// | `pegada_traducao` | *o que está NESTE índice muda-se para onde?* (segue-se em cadeia) |
    /// | `pegada_mortos` | *este índice foi SOBRESCRITO?* — quem lá estava morreu |
    ///
    /// ⚠️ **E quem não está em tabela nenhuma ainda pode ter morrido:** um
    /// vértice acima do novo tamanho que ninguém foi buscar é cauda truncada. É a
    /// mesma varredura que a irmã do `pre` faz sobre os slots, pela mesma razão.
    pub(super) fn encolhe_a_pegada_congelada(&mut self, remap: &ph2d_mesh::Remap) {
        if self.pegada_ancorada.is_empty() {
            return;
        }
        let de_para = &mut self.pegada_traducao;
        de_para.clear();
        de_para.extend_from_slice(&remap.vert_moves);
        de_para.sort_unstable_by_key(|&(de, _)| de);
        let mortos = &mut self.pegada_mortos;
        mortos.clear();
        mortos.extend(remap.vert_moves.iter().map(|&(_, para)| para));
        mortos.sort_unstable();
        let verts = u32::try_from(remap.verts).unwrap_or(u32::MAX);
        for p in &mut self.pegada_ancorada {
            p.verts
                .retain_mut(|v| match onde_parou(*v, de_para, mortos, verts) {
                    Some(novo) => {
                        *v = novo;
                        true
                    }
                    None => false,
                });
            // ⛔ **O índice REFAZ-SE, e não se remenda:** a renumeração manda
            // sobreviventes da cauda para buracos no meio, logo a ordem por valor
            // quebra-se sempre. `verts` é que mantém a ordem da consulta, que é a
            // que o dab lê.
            p.ordenada.clear();
            p.ordenada.extend_from_slice(&p.verts);
            p.ordenada.sort_unstable();
        }
    }
}

/// **ONDE UM VÉRTICE FOI PARAR DEPOIS DE UM COLAPSO** — `None` se ele morreu.
///
/// A metade pensante da [`SculptStroke::encolhe_a_pegada_congelada`], onde está
/// o porquê de isto ser uma **cadeia** e não uma consulta. Função livre porque a
/// lei não precisa de nada do traço: dadas as duas tabelas, a resposta é do plano
/// do colapso e de mais nada — e é isso que a torna gateável de frente.
///
/// ⭐ **A cadeia é finita e não tem ciclos por construção:** as origens das
/// trocas descem estritamente (o plano varre os mortos do maior para o menor) e
/// todo destino é menor que a sua origem, logo segui-la é um passeio para baixo
/// que acaba sempre.
///
/// ⚠️ **A morte é consultada PRIMEIRO, e a ordem é uma propriedade medida:** um
/// índice que é sobrescrito **e** é origem de uma troca é sempre sobrescrito
/// antes de se mudar (o contrário obrigaria uma origem a crescer ao longo do
/// plano, e elas só descem) ⇒ *quem lá estava morreu, e quem se muda a seguir é o
/// inquilino novo.*
pub(super) fn onde_parou(
    v: u32,
    de_para: &[(u32, u32)],
    mortos: &[u32],
    verts: u32,
) -> Option<u32> {
    if mortos.binary_search(&v).is_ok() {
        return None;
    }
    let mut actual = v;
    while let Ok(i) = de_para.binary_search_by_key(&actual, |&(de, _)| de) {
        actual = de_para[i].1;
    }
    (actual < verts).then_some(actual)
}
