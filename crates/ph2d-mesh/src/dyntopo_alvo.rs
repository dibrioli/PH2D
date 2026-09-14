//! ⭐⭐ **QUANTO DEVE MEDIR UMA ARESTA** — as duas leis do alvo, e a razão de
//! elas viverem fora do [`super::dyntopo`].
//!
//! ⚠️ **O corte é de RESPONSABILIDADE, e ele nomeia-se sozinho:** lá mora *como
//! se parte e se funde uma malha* (as filas, a recursão, o re-triangular, o
//! `rebuild`), aqui *qual é o tamanho que se pede*. São perguntas independentes
//! — a segunda mudou de âncora inteira em 2026-09-14 sem a primeira se mexer —
//! e foi a segunda que levou o ficheiro ao tecto de 700 LOC.
//!
//! ⛔ **Curado por CORTE e nunca por uma entrada no `FILE_OVERAGE_OK`.**

/// **O alvo de comprimento de aresta**, em unidades de OBJETO.
///
/// `detail` anda em `[0, 1]`: `0` é o mais grosso, `1` o mais fino. A fórmula é
/// a do `SculptBase.js` (`d2Max = radius² × (1.1 − sub) × 0.2`), tirada da raiz
/// para virar comprimento — a razão de a raiz vir aqui e não no chamador é que
/// **o quadrado é detalhe do laço**, e um alvo em comprimento é o que se pode
/// comparar com uma aresta num log ou num gate.
///
/// Os extremos, para quem for reafinar: `detail = 0` dá `0,469 × raio` e
/// `detail = 1` dá `0,141 × raio` — o mais fino que a referência oferece é
/// cerca de **um sétimo do pincel**.
///
/// ⛔ **O PRODUTO já não a chama** (ver o cabeçalho do módulo): ela carrega o
/// zoom para dentro do alvo, e o dono recusou isso. Ela fica porque é a lei da
/// referência e porque é ela que as bancadas de paridade medem.
#[must_use]
pub fn edge_target(radius: f32, detail: f32) -> f32 {
    let d = detail.clamp(0.0, 1.0);
    radius * ((1.1 - d) * 0.2).sqrt()
}

/// **O LADO DE UM TRIÂNGULO EQUILÁTERO que ladrilha `area` com `tris` peças.**
///
/// `area = tris × (√3/4) × a²` ⇒ `a = √(4·area / (√3·tris))`.
///
/// ⚠️ **É a metade PURA da lei, e existe separada de propósito:** a contagem
/// entra como número e não como posição de slider, e é isso que deixa a
/// varredura de calibração medir a faixa sem passar pela escada que a escolhe.
#[must_use]
pub fn edge_for_tri_count(area: f32, tris: f32) -> f32 {
    // ⚠️ **A guarda é escrita pela POSITIVA de propósito:** a negação de um `>`
    // sobre `f32` não cobre o `NaN` da maneira que se lê, e é o `NaN` que este
    // ramo existe para apanhar — uma área degenerada devolveria um alvo `NaN`,
    // que o motor lê como *«nenhuma aresta está fora da faixa»* e cala-se.
    if !(area.is_finite() && area > 0.0 && tris.is_finite() && tris >= 1.0) {
        return 0.0;
    }
    (4.0 * area / (3.0f32.sqrt() * tris)).sqrt()
}

/// **O MENOR NÚMERO DE TRIÂNGULOS que ainda descreve uma forma** — o extremo
/// grosso, MEDIDO.
///
/// ⭐ **Ele é o [`crate::MIN_QUADS`] do botão de retopologia, em triângulos.**
/// Aquele número saiu de uma varredura de volume na esfera amassada (volume da
/// entrada `3,78`): `96` faces guardam **`3,45`** (91 %) · `40` guardam `2,57`
/// (68 %) · `23` guardam `1,90` (50 %) ⇒ o joelho está em ~`100` faces. Um quad
/// são dois triângulos, logo o mesmo joelho aqui é **`200`**.
///
/// ⚠️ **A pergunta é a MESMA nos dois sítios** — *«a partir de que contagem a
/// superfície deixa de sobreviver?»* —, e a resposta é da SUPERFÍCIE, não da
/// espécie de face. Escrever um segundo número aqui seria duas medições da
/// mesma grandeza.
pub const MIN_TRIS: f32 = 200.0;

/// **O MAIOR NÚMERO DE TRIÂNGULOS que o passe pede** — o extremo fino, e o
/// recurso é o **QUADRO**.
///
/// ⛔⛔ **De que recurso é este número: do RELÓGIO DO DAB, e de mais nada.**
/// Cada passe do dyntopo termina num [`crate::Mesh::rebuild`] **inteiro** —
/// anéis, octree e normais —, e a tabela do cabeçalho deste módulo mede-o:
///
/// | vértices | `rebuild` | cabe no dab (8 ms)? |
/// |---|---|---|
/// | 6 050 | 0,59 ms | sim |
/// | 24 386 | 1,55 ms | sim |
/// | 97 922 | **5,50 ms** | sim, **apertado** (69 % do orçamento) |
///
/// Numa malha fechada os triângulos são ~`2×` os vértices, logo `97 922`
/// vértices são ~`196 000` triângulos — o ponto onde o rebuild come dois terços
/// do dab. ⇒ o tecto fica em **`100 000`** (≈ `50 000` vértices, `~2,8 ms`,
/// **35 %** do orçamento), que é o ponto mais fino ainda **confortável** e não
/// aquele em que o relógio ainda se aguenta.
///
/// ⚠️ **Quem o subir mede o RELÓGIO DO DAB**, não o da corrida inteira — e paga
/// a estrutura mutável que o cabeçalho deste módulo nomeia como a wave seguinte.
pub const MAX_TRIS: f32 = 100_000.0;

/// **QUANTOS TRIÂNGULOS um `detail` de `0..1` pede** — absoluto, ancorado na
/// ÁREA.
///
/// `0` é [`MIN_TRIS`] e `1` é [`MAX_TRIS`], com interpolação **GEOMÉTRICA**: a
/// faixa atravessa `500×`, e num slider linear todo o curso útil ficaria
/// espremido no primeiro décimo. *Um knob de TAMANHO tem de andar em razão
/// constante* — a mesma lei do `quads_for_detail` do botão.
///
/// ⚠️ **`clamp` NÃO fecha o `NaN`**, e ele cai no extremo GROSSO de propósito:
/// é o único lado que não pede infinitos vértices.
#[must_use]
pub fn tris_for_detail(detail: f32) -> f32 {
    let d = if detail.is_nan() {
        0.0
    } else {
        detail.clamp(0.0, 1.0)
    };
    MIN_TRIS * (MAX_TRIS / MIN_TRIS).powf(d)
}

/// ⭐⭐⭐ **O ALVO DE ARESTA DO PASSE, INDEPENDENTE DO ZOOM** — ver
/// [`MAX_TRIS`].
///
/// ⛔⛔ **ORDEM DO DONO (2026-09-14): *«a densidade da malha deve ser
/// independente do zoom»*.** E ele tinha um defeito medido do lado dele: o alvo
/// saía do raio do pincel **em unidades de MUNDO**, e esse raio é derivado do
/// raio em PIXELS através da câmera a cada dab. Medido na cena `=14`, com o
/// mesmo pincel (`160 px`), o mesmo slider (`0,5`) e a mesma peça:
///
/// | distância da câmera | raio em mundo | alvo de aresta |
/// |---|---|---|
/// | 1,5 | 0,1198 | **0,0415** |
/// | 3,0 | 0,2752 | 0,0953 |
/// | 6,0 | 0,5858 | **0,2029** |
///
/// ⇒ **`4,9×` de alvo só por aproximar ou afastar.** *O número não era da
/// malha* — e é literalmente a mesma frase que o botão de retopologia já tinha
/// escrita quando o `Quad Size` absoluto foi refutado com foto pelo dono.
///
/// ⭐ **A cura é a mesma daquele botão, e por isso ela não é uma invenção:**
/// ancorar na **ÁREA DA SUPERFÍCIE** ([`crate::Mesh::surface_area`]), que é
/// propriedade da forma e não da tesselação nem da vista. O slider passa a
/// pedir uma **contagem**, que é o que as três referências oferecem (ZBrush
/// *Target Polygons Count* · QuadriFlow *Number of Faces* · Instant Meshes).
///
/// ⚠️⚠️ **E o pincel deixa de mandar na densidade — de propósito.** O doc
/// antigo dizia *«um pincel pequeno detalha fino e um grande detalha grosso, que
/// é o que a mão espera»*, e isso é a **fracção do pincel** da referência
/// (espec §3.4, *detalhe PELO PINCEL*). O que fica é a *detalhe CONSTANTE* dela
/// — `1/(resolução · escala_do_objecto)` —, que é a que não muda com a vista.
/// ⇒ **o pincel diz ONDE, o slider diz QUÃO FINO.** As duas perguntas deixam de
/// partilhar um número.
#[must_use]
pub fn edge_target_for_mesh(mesh: &crate::Mesh, detail: f32) -> f32 {
    edge_for_tri_count(mesh.surface_area(), tris_for_detail(detail))
}
