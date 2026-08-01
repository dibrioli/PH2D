//! **O ALINHAMENTO do traço, vivo** — Inner/Outer desenhados no z da forma.
//!
//! Espelho dos cinco irmãos (`offset_live`/`pattern_live`/`contour_live`/`symmetry_live`/
//! `profile_live`): o documento guarda o que o artista autorou e o que se vê é derivado, re-cozido
//! aqui e desenhado por [`ph2d_vec_render::dispatch`] no z da fonte.
//!
//! # ⚠️ Este roda por ÚLTIMO, e não é ordem por gosto
//!
//! O `render_loop` funde os cinco mapas com `extend` e o comentário de lá diz por que isso é
//! seguro: *"uma forma é offset OU pattern OU contour (nunca dois — cada um é um componente
//! próprio e o painel oferece um de cada vez)"*. **O alinhamento não é membro dessa família.** Ele
//! é um campo do `StrokeSpec`, então **compõe** com qualquer um deles: uma forma pode ter offset
//! vivo *e* traço interno. Fundido por `extend` ele apagaria o offset (ou seria apagado por ele),
//! e a forma perderia metade do que o artista pediu, em silêncio.
//!
//! Então ele **não produz um mapa** — ele **TRANSFORMA** o mapa já montado, como o
//! `fx_silhouette`, que roda depois pela mesma razão (*"a união é do que se DESENHA"*). A entrada
//! de um caminho é o que estiver no mapa; se não houver nada, é a fonte assada em mundo.
//!
//! # O que uma entrada vira
//!
//! Um `VecPath` com traço alinhado é **substituído por dois ou mais**: ele próprio sem traço (o
//! preenchimento fica onde estava) e as peças da faixa recortada, que já nascem preenchidas com a
//! cor do traço (ver [`ph2d_vec_boolean::aligned_stroke`]). Um item recusado pelo motor — caminho
//! aberto, Centre, largura zero — passa **verbatim**; e se NENHUM item alinhou, o mapa não é
//! tocado, que é o que torna o mundo pré-alinhamento byte-idêntico (uma entrada, mesmo igual à
//! fonte, já muda a rota de desenho).
//!
//! # O memo é MEDIDO, e o número decidiu contra ele
//!
//! `aligned_stroke` custa **0,064-0,069 ms** numa rosquinha, **0,18** numa estrela de 5 pontas e
//! **1,46** na patológica de 24, contra 16,6 ms de um quadro. Isso é vivo com folga, e o memo
//! existe pelo caso da CENA CHEIA (dez formas traçadas × por frame), não pelo caso de uma. A
//! chave é o que de facto determina a saída — a geometria de MUNDO que entra — e não um contador
//! de versão que alguém esqueceria de bumpar.

use std::collections::BTreeMap;

use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

/// Uma entrada do memo: o que ENTROU (a lista de mundo) e o que SAIU.
struct Memo {
    input: Vec<VecPath>,
    out: Vec<VecPath>,
}

/// O alinhamento vivo de toda a cena, com memo por caminho.
#[derive(Default)]
pub(crate) struct AlignLive {
    memo: BTreeMap<VecPathId, Memo>,
}

impl AlignLive {
    /// **Aplica o alinhamento sobre o mapa vivo do frame.** Chamado DEPOIS dos cinco produtores
    /// e ANTES do `fx_silhouette` — a silhueta é do que se desenha, e o que se desenha já é isto.
    pub(crate) fn recook(&mut self, scene: &VecScene, xforms: &VecXforms, live: &mut LiveGeometry) {
        let mut touched: Vec<VecPathId> = Vec::new();
        for path in scene.paths() {
            // A entrada: o que o mapa já diz, ou a fonte assada em MUNDO (a mesma conversão do
            // `offset_live` — o `dispatch` sobe a derivada pela CÂMERA, não pelo afim do path,
            // e aplicar a pose duas vezes foi bug real desta linha).
            let input: Vec<VecPath> = match live.get(&path.id) {
                Some(items) => items.clone(),
                None => {
                    let mut world = path.cooked().into_owned();
                    bake_xform(&mut world, &xform_of(xforms, path.id));
                    vec![world]
                }
            };
            // Recusa BARATA: sem nenhum traço alinhado não há sequer o que perguntar ao motor.
            // ⚠️ Ela é um superconjunto, **nunca a decisão** — `is_aligned()` não sabe se o
            // caminho é FECHADO, e essa metade vive no kernel. Duas portas para *"isto vai fazer
            // alguma coisa?"* divergiriam, e a divergência foi medida: uma linha ABERTA com
            // Outer passava aqui e entrava no mapa com a própria fonte.
            if !input
                .iter()
                .any(|p| p.stroke.is_some_and(|s| s.is_aligned()))
            {
                continue;
            }
            let hit = self.memo.get(&path.id).is_some_and(|m| m.input == input);
            if !hit {
                // Quem decide é o MOTOR: `None` = nada foi alinhado, e aí o mapa fica intocado.
                let Some(out) = align_items(&input) else {
                    continue;
                };
                self.memo.insert(
                    path.id,
                    Memo {
                        input: input.clone(),
                        out,
                    },
                );
            }
            if let Some(m) = self.memo.get(&path.id) {
                touched.push(path.id);
                live.insert(path.id, m.out.clone());
            }
        }
        // O memo não sobrevive ao alinhamento: uma forma que voltou a Centre manteria a resposta
        // velha e a re-serviria se alguém a escolhesse de novo com outra geometria.
        self.memo.retain(|id, _| touched.contains(id));
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do
    /// memo, e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
    }
}

/// A lista de entrada com cada traço alinhado trocado por *o resto* + *a faixa*, ou **`None`** se
/// o motor não alinhou nenhum item.
///
/// ⚠️ **`None` e uma lista igual à entrada não são a mesma coisa**, e é por isso que a distinção
/// existe: uma entrada no mapa — mesmo idêntica à fonte — faz o `dispatch` desenhar pela rota da
/// DERIVADA (que já tem a pose assada) em vez da rota da fonte (que a aplica pela câmera). Aplicar
/// a pose duas vezes foi bug real desta linha.
///
/// ⚠️ **O item sem traço fica na lista**, e não é higiene: ele carrega o PREENCHIMENTO, que o
/// alinhamento não toca. Descartá-lo esvaziaria o miolo da forma no instante em que o artista
/// escolhe Inner.
fn align_items(input: &[VecPath]) -> Option<Vec<VecPath>> {
    let mut out = Vec::with_capacity(input.len() + 1);
    let mut any = false;
    for item in input {
        match ph2d_vec_boolean::aligned_stroke(item) {
            Some(band) => {
                any = true;
                out.push(VecPath {
                    stroke: None,
                    ..item.clone()
                });
                out.extend(band);
            }
            // Recusado pelo motor (aberto / Centre / largura zero) ⇒ verbatim.
            None => out.push(item.clone()),
        }
    }
    any.then_some(out)
}

#[cfg(test)]
#[path = "align_live_tests.rs"]
mod tests;
