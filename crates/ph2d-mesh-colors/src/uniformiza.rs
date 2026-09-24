//! ⭐⭐⭐⭐ **O PLANO GRADUADO DE UM FICHEIRO ANTIGO, levado a UM nível só.**
//!
//! ⚠️ **Porque existe:** entre 23/09 e 24/09 o `Even Detail` criava planos com
//! um nível POR FACE, e o ficheiro guardou-os. Ele saiu por ordem do dono, e em
//! 24/09 saiu também o registo de `19` palavras que deixava a placa desenhá-los
//! (ordem do dono: *liberar a memória que ele deixou reservada*). Um ficheiro
//! desse dia continua a abrir — **o carregador passa por aqui** — e o plano
//! chega à placa UNIFORME, no degrau que o artista tinha pedido.
//!
//! ⭐⭐ **LER e não re-semear, e a diferença é a tinta do artista.** Re-semear da
//! cor por vértice devolve a tinta à resolução da malha (é o sintoma que o
//! dono já reportou três vezes). Aqui cada amostra do plano novo é LIDA do
//! plano graduado no ponto dela — pela [`Tinta::cor_tri`]/[`Tinta::cor_quad`],
//! que são a lei que a placa e o oráculo partilham.
//!
//! ⭐⭐⭐ **E ela é EXACTA onde a face está no degrau pedido ou acima**, que é o
//! caso de todo plano gravado depois da cura do PISO (handoff §30): os lados
//! são potências de dois, logo o ponto `i/L` do plano novo cai num nó da
//! retícula da face, e a leitura ali tem peso `1` num canto e `0` nos outros.
//! Onde a face está ABAIXO do pedido (os planos da âncora da mediana, que viveu
//! horas) a leitura interpola — é o melhor que existe, porque essas amostras
//! nunca foram pintadas.
//!
//! ⚠️⚠️ **A ORDEM das faces é load-bearing:** uma amostra de ARESTA é escrita
//! pelas duas faces que a partilham, e só a mais FINA tem as amostras
//! verdadeiras da aresta (a grossa lê um subconjunto, e entre os nós dele
//! interpola). ⇒ as faces correm da mais grossa para a mais fina e **a última
//! escrita ganha** — determinístico, e com a amostra verdadeira no fim.

use crate::Tinta;
use crate::enderecos::{indice, sitio_quad, sitio_tri};

impl Tinta {
    /// **Este plano, com UM nível só — o que ele diz ter sido pedido.**
    ///
    /// Devolve `None` se `faces` não descreve este plano — outra contagem de
    /// faces, ou uma face com outro número de cantos. A razão é a de sempre:
    /// *um plano com o tamanho errado instalado numa malha é tinta no sítio
    /// errado*. ⚠️ Os VÉRTICES não entram na pergunta porque o plano novo nasce
    /// com os deste — compará-los com eles próprios seria uma metade que não
    /// prova nada.
    ///
    /// ⚠️ Um plano JÁ uniforme devolve uma cópia ao bit — ela é idempotente, e
    /// quem a chama não tem de perguntar antes.
    #[must_use]
    pub fn uniformizada<'a>(&self, faces: impl Iterator<Item = &'a [u32]> + Clone) -> Option<Self> {
        let topo = self.topologia();
        let lista: Vec<&[u32]> = faces.clone().collect();
        if lista.len() != topo.faces() {
            return None;
        }
        if lista
            .iter()
            .enumerate()
            .any(|(f, c)| crate::cantos(c) != topo.cantos_de(f))
        {
            return None;
        }
        let mut novo = Self::nova(topo.verts(), faces, self.nivel());
        let l = 1u32 << self.nivel();
        let lf = l as f32;

        // Da mais GROSSA para a mais FINA — ver o cabeçalho.
        let mut ordem: Vec<usize> = (0..lista.len()).collect();
        ordem.sort_by_key(|&f| (topo.nivel_de(f), f));

        for fi in ordem {
            let cantos = &lista[fi][..crate::cantos(lista[fi])];
            if cantos.len() == 3 {
                for i in 0..=l {
                    for j in 0..=(l - i) {
                        let k = l - i - j;
                        let bar = [i as f32 / lf, j as f32 / lf, k as f32 / lf];
                        let idx = indice(novo.topologia(), fi, sitio_tri(l, i, j, k), cantos);
                        novo.amostras_mut()[idx as usize] = self.cor_tri(fi, cantos, bar);
                    }
                }
            } else {
                for j in 0..=l {
                    for i in 0..=l {
                        let uv = [i as f32 / lf, j as f32 / lf];
                        let idx = indice(novo.topologia(), fi, sitio_quad(l, i, j), cantos);
                        novo.amostras_mut()[idx as usize] = self.cor_quad(fi, cantos, uv);
                    }
                }
            }
        }
        Some(novo)
    }
}
