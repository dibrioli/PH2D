//! **A DISTÂNCIA ATÉ À OUTRA PEÇA** — a lei do [`crate::Verb::SceneProject`]
//! (`SPEC_unblocked_brushes.md` §6.3), e a única coisa que este pincel calcula
//! que os irmãos não sabem calcular.
//!
//! Tudo o resto dele é a cadeia de peso de sempre: o alvo de um vértice é
//! `p + direcção · d · peso · força²`, e o `d` é o que este módulo devolve.
//!
//! # ⚠️⚠️ O `d` é um PARÂMETRO DE RECTA, e a espec avisa-o por uma razão que
//! quase não nos toca
//!
//! A espec §6.3 deriva, para um afim **qualquer** `M`, que se `Q = P + d·N`
//! então `M·Q = M·P + d·(M·N)` — *o mesmo `d`* resolve a equação dos dois
//! lados, logo `d` é o parâmetro e **não** um comprimento. Ela fica com a
//! advertência de que converter `d` de volta dá um número plausível e errado.
//!
//! ⭐ **Na nossa casa essa armadilha COLAPSA num factor de escala, e é
//! medível:** a [`ph2d_mesh::Pose`] é uma **semelhança** — translação mais
//! escala **uniforme**, sem rotação e sem corte. ⇒ a norma de uma direcção
//! muda por um número só, e o parâmetro e o comprimento diferem por ele:
//!
//! ```text
//! d_no_activo  =  t_do_acerto  ×  escala_do_alvo / escala_do_activo
//! ```
//!
//! ⚠️ **O `t` que o motor devolve é um comprimento em unidades do ALVO**, e não
//! por descuido: o [`ph2d_mesh::Ray`] **normaliza a direcção na construção**, e
//! a [`ph2d_mesh::Pose::ray_to_local`] leva o raio ao espaço da malha consultada
//! — é o que o doc daquela porta já escreve. Comparar dois acertos de peças com
//! escalas diferentes pelo `t` cru seria a resposta errada; aqui **todos** os
//! candidatos são convertidos para o espaço do objecto ACTIVO antes de
//! competirem, e é por isso que o `min |d|` da espec fica honesto.
//!
//! ⛔ **O dia em que a `Pose` ganhar rotação, esta linha deixa de bastar** — e
//! deixa de bastar em SILÊNCIO, com um `d` plausível. É o que o gate
//! `a_distancia_atravessa_a_escala_das_duas_pecas` existe para acordar.

use ph2d_mesh::{Mesh, Pose, Ray};

/// O que o lançamento de raio devolve — ver [`distancia`].
///
/// ⚠️ **`None` e `Some(0.0)` são coisas DIFERENTES, e a espec separa-as:**
/// nenhum acerto quer dizer *o vértice não se move* (§6.3.3), e um acerto a
/// distância zero quer dizer *ele já está encostado* — mas o segundo ainda
/// **subtrai a folga** (§6.3.4) e pode acabar por o afastar. Colapsar os dois
/// num `f32` apagaria a segunda metade da §6.4.
pub(crate) type Distancia = Option<f32>;

/// **A DISTÂNCIA COM SINAL até à peça mais próxima**, em unidades do objecto
/// ACTIVO — ou `None` se nenhum raio acertou nada.
///
/// # As quatro regras, pela ordem da espec §6.3
///
/// 1. um raio por alvo, na direcção do dab; guarda-se o de **menor `|d|`**;
/// 2. com `nos_dois_sentidos`, lança-se **também** ao contrário, e esse
///    candidato entra com **`−d`** — na mesma comparação por valor absoluto;
/// 3. nenhum acerto em alvo nenhum ⇒ o vértice **não se move**;
/// 4. houve acerto ⇒ `d := d − folga`.
///
/// ⛔⛔ **A ORDEM de 1–2 contra 4 é load-bearing:** a folga entra **depois** de o
/// vencedor estar escolhido. Subtraí-la antes faria a competição ser entre
/// números já encolhidos, e um alvo longe podia ganhar de um perto por causa de
/// uma folga que é a mesma para os dois.
///
/// ⚠️ **A folga é subtraída de um `d` COM SINAL**, e daí saem os dois
/// comportamentos que ninguém prevê lendo o rótulo (espec §6.4, os dois
/// medidos): uma folga maior que o vão **afasta** a peça, e num acerto para trás
/// ela **cresce** a excursão em vez de a travar. *Isto é o alvo reproduzido de
/// propósito*, e a alternativa simétrica é decisão de produto — ver
/// [`folga_simetrica`].
pub(crate) fn distancia(
    ponto: [f32; 3],
    direccao: [f32; 3],
    activo: Pose,
    alvos: &[(Mesh, Pose)],
    nos_dois_sentidos: bool,
    folga: f32,
) -> Distancia {
    if alvos.is_empty() {
        return None;
    }
    let mundo = Ray::new(
        activo.point_to_world(ponto),
        activo.vector_to_world(direccao),
    );
    let ao_contrario = Ray::new(
        mundo.origin(),
        [-mundo.dir()[0], -mundo.dir()[1], -mundo.dir()[2]],
    );
    let mut melhor: Option<f32> = None;
    for (malha, pose) in alvos {
        // ⚠️ **O sinal é do SENTIDO em que o raio foi lançado**, e não do `t`:
        // o motor só devolve acertos para a frente, logo um acerto do raio
        // invertido está **atrás** do vértice e o `d` dele é negativo.
        for (raio, sinal) in [(&mundo, 1.0f32), (&ao_contrario, -1.0)] {
            if sinal < 0.0 && !nos_dois_sentidos {
                continue;
            }
            let local = pose.ray_to_local(raio);
            let Some(acerto) = malha.raycast(&local) else {
                continue;
            };
            let d = sinal * comprimento_no_activo(acerto.t, *pose, activo);
            if melhor.is_none_or(|m: f32| d.abs() < m.abs()) {
                melhor = Some(d);
            }
        }
    }
    melhor.map(|d| d - folga)
}

/// **O comprimento do alvo, na régua do activo** — ver o cabeçalho.
///
/// ⚠️ Porta separada de propósito: ela é a única linha deste módulo que depende
/// de a [`Pose`] ser uma **semelhança**, e tê-la sozinha é o que deixa o gate
/// apontar-lhe o dedo quando essa premissa mudar.
fn comprimento_no_activo(t: f32, alvo: Pose, activo: Pose) -> f32 {
    t * alvo.scale() / activo.scale()
}

/// ⛔⛔ **A FOLGA SIMÉTRICA — a lei que NÃO shipa, escrita porque a decisão é do
/// dono e ela tem de estar medida dos dois lados.**
///
/// A espec §6.4 mede que a folga do alvo só é «distância mínima» no sentido de
/// avanço; a alternativa é `d := sign(d) · max(0, |d| − folga)`, que trava nos
/// **dois** sentidos e nunca inverte o sentido do movimento.
///
/// ⚠️ **Adoptá-la é uma DIVERGÊNCIA DELIBERADA a declarar**, e reproduzir o alvo
/// é reproduzir um defeito — *as duas frases que a põem, e sem terceira saída*.
/// Enquanto o dono não decide, shipa a do alvo (é ela que o corpus mede) e esta
/// fica aqui com o gate que a compara, para a troca ser **uma linha** e não uma
/// wave.
///
/// ⚠️ **`#[cfg(test)]` é a declaração honesta do estado dela:** ela está
/// **medida** e não **shipada**. Um `pub(crate)` sem chamador seria uma lei
/// órfã a fingir-se de produto — a espécie que o `CLAUDE.md` §5.0 nomeia (*uma
/// porta sem chamador e uma lei ausente produzem o mesmo app*). No dia em que o
/// dono decidir, ela ganha o chamador e o `cfg` sai no mesmo commit.
#[cfg(test)]
#[must_use]
pub(crate) fn folga_simetrica(d: f32, folga: f32) -> f32 {
    d.signum() * (d.abs() - folga).max(0.0)
}

/// ⭐⭐⭐ **PROJECTAR NA CENA** — o vértice viaja até **encostar noutra
/// peça**, e a distância é medida por um raio (espec §6.3). A lei do
/// raio vive no irmão [`crate::projectar`]; aqui fica o que ela tem
/// em comum com todos os outros: a cadeia de peso.
///
/// ⛔⛔ **O `w × strength` DO APAGADOR NÃO TRANSFERE PARA CÁ, e a
/// linha que os separa é INVISÍVEL a olho: quem declara um perfil de
/// referência já recebe a força ao quadrado no `w`.** Medido, com o
/// slider a `0,5`:
///
/// | verbo | modo `B` | modo `S` |
/// |---|---|---|
/// | `Scene Project` | **`0,25`** | `0,5` |
/// | `Erase Displacement` | `0,5` | `0,5` |
///
/// O apagador **não tem perfil** ([`crate::ref_profiles`] devolve
/// `None` para ele), logo o `Brush::weight` cai no slider cru e o
/// quadrado tem de ser escrito no braço dele. Este verbo **tem**, e
/// o funil já o aplicou ⇒ multiplicar outra vez daria a força ao
/// **CUBO**. ⚠️ E não é teoria: escrito à maneira do apagador ele
/// media `0,0625` onde o oráculo mede `0,125`, e quem o apanhou foi
/// a fixtura `projectar_forca05_constante_1passo`.
///
/// ⇒ aqui o `w` chega pronto, como chega ao `Draw` e a todos os
/// outros verbos da referência restrita. *O que é excepção são os
/// dois de multirresolução, não este.*
///
/// ⚠️⚠️ **A DIREÇÃO é do DAB e não do vértice** ([`crate::ProjectMode`]):
/// uma direcção por vértice foi construída e rejeitada pelo autor do
/// alvo como inutilizável (espec §9.2) — *recusa medida por
/// terceiros*.
///
/// ⛔ **Nenhum acerto ⇒ o VIVO**, que é *não mover nada* (§6.3.3). É
/// a mesma forma do apagador sem referência: quando o dado de
/// entrada não existe, **a lei não inventa** — aqui ela não projecta
/// para o infinito.
/// ⛔⛔⛔ **A INVERSÃO SAIU DO PRODUTO POR ORDEM DO DONO (15/09) — e
/// isto é a RECUSA REGISTADA, não uma ausência.** Palavras dele
/// depois do smoke da `=45`: *«não vi utilidade na feature Scene
/// Project + CTRL. Melhor retirá-la e documentá-la como
/// indesejada.»*
///
/// ⚠️ **Retirar o gesto retira a CAPACIDADE, e isso foi medido antes
/// de se cortar:** o `Brush::invert` tem **um** escritor no produto
/// inteiro (`scene.brush.invert = ctrl`, no pen-down) e **nenhum**
/// controlo de painel o oferece ⇒ o `Ctrl` era a única porta. Com o
/// verbo fora do [`crate::Verb::honours_invert`] o `sign` do
/// `stroke_target` fica preso em `+1` para sempre, e um parâmetro que
/// só pode valer uma coisa é um ÓRFÃO — *a cura de um órfão é
/// apagar* (`CLAUDE.md` §5.0), e não deixá-lo vivo e inalcançável.
///
/// ⭐⭐ **O que fica registado é a MEDIÇÃO, para ninguém a repetir.** A
/// lei estava CERTA e provada no dia em que saiu: `inverter` **nega a
/// translação** e o raio **não vira** (espec §1.2: *«o sinal entra no
/// factor»*), com as duas fixturas do corpus a fechar a **`2,384e-7`**
/// contra uma barra de `2e-6`. ⛔ A redacção anterior dizia o
/// contrário e ela e a cena que a sustentava estavam erradas JUNTAS,
/// cada uma a confirmar a outra: a bancada tinha reconstruído o alvo
/// da invertida **ACIMA** (lido do deslocamento máximo, que é para
/// cima) e, com o alvo em cima e os dois sentidos desligados, a
/// negação escrita no fim media **`0` vértices movidos contra `301`**.
/// O que as separou foi o **CABEÇALHO** (a base e a invertida são
/// idênticas campo a campo, `sentido: ADD` nas duas) e o número que a
/// espec §6.6 publica para a assimetria, `1,415e-1`, que era
/// **exactamente** o nosso desvio. *Ler o próprio desvio no número
/// que a espec dá é o diagnóstico inteiro.*
///
/// ⚠️ **E o mecanismo que explicava porque ela não explodia fica
/// aqui também:** a negação empurra o vértice para LONGE do alvo,
/// logo o dab seguinte mede uma distância MAIOR — mas o peso cai,
/// porque a pegada é uma esfera em torno de um centro PARADO e o
/// vértice sai dela; a excursão travava em exactamente `0,500000`,
/// cravado no corpus.
///
/// ⛔ **Duas fixturas do oráculo saíram do corpus VIVO com esta
/// decisão** (`projectar_invertido` e `projectar_subtrair`) — ver a
/// lista nomeada na banca, que reprova se alguém reintroduzir o
/// gesto sem as repor. *É uma divergência DECLARADA da referência:
/// ela tem a capacidade, e nós não a queremos.*
///
/// ⚠️ **Ela vive AQUI e não no `match` dos alvos por uma razão de TECTO e de
/// ASSUNTO:** o `stroke_target.rs` responde *para onde cada verbo aponta* com
/// uma linha por verbo, e este precisava de cinquenta — ele lê as OUTRAS peças
/// da cena, que nenhum irmão dele lê. *O corte é o mesmo que o esfregão já
/// pagou.*
pub(crate) fn alvo_do_vertice(
    stroke: &crate::SculptStroke,
    brush: &crate::Brush,
    dab: &crate::Dab,
    n_area: [f32; 3],
    live: [f32; 3],
    w: f32,
) -> [f32; 3] {
    let direccao = brush.project_mode.direccao(dab.eye, n_area);
    distancia(
        live,
        direccao,
        stroke.pose_activa,
        &stroke.pecas_da_cena,
        brush.project_bidirectional,
        brush.project_min_distance,
    )
    .map_or(live, |d| {
        let f = d * w;
        [
            live[0] + direccao[0] * f,
            live[1] + direccao[1] * f,
            live[2] + direccao[2] * f,
        ]
    })
}

#[cfg(test)]
#[path = "projectar_tests.rs"]
mod tests;
