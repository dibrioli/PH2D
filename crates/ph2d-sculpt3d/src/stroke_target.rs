//! **O ALVO de cada verbo** — de onde vem o ponto para o qual um vértice
//! caminha, e o plano que quatro deles ajustam.
//!
//! ⚠️ **Filho (`#[path]`) de [`super`], e não um módulo irmão:** estes métodos
//! leem os campos privados do [`SculptStroke`] (o `pre` congelado, os slots), e
//! um irmão os obrigaria a virar `pub(crate)` — a visibilidade viraria função do
//! TAMANHO do arquivo, que é o oposto do que o teto de LOC existe para fazer.
//!
//! ⚠️ O corte ainda cobra **duas** aberturas, e elas são o mínimo: `fit_plane` e
//! `compute_target` viram `pub(super)` porque o laço do dab (que ficou no pai) as
//! chama. Um filho VÊ o privado do pai, mas o pai não vê o do filho — e
//! `pub(super)` é estritamente mais estreito que o `pub(crate)` que um irmão
//! pediria — e o [`PlaneFit`] vai junto, porque ele aparece na assinatura das
//! duas. O resto (`fit_plane_over`, `neighbour_average`) só é chamado aqui e
//! segue privado.
//!
//! O corte é o que os próprios gates já usam: a **LEI do traço** (o envelope, a
//! captura, o ciclo de um dab) fica no pai; **para onde cada verbo aponta** — a
//! parte que difere entre os treze — fica aqui.

use super::*;
// ⚠️ **O `PlaneFit` vem do IRMÃO, e é a única abertura que o corte cobrou:** ele
// aparece na assinatura do `compute_target` porque quatro verbos consomem o
// plano que o outro arquivo ajusta.
use super::plane::PlaneFit;

/// **A ARITMÉTICA** com que um alvo é escrito — ver [`aim`].
#[path = "stroke_aim.rs"]
mod aim;
use aim::{add_vec, remove_along, rotate_about, signed_distance, to_plane};
// ⚠️ **O `cross` é re-exportado ao IRMÃO [`super::plane`]**, que monta a
// dobradiça da lâmina em V. A alternativa era uma segunda cópia de três linhas
// de aritmética lá — e o teto de LOC deste arquivo é exactamente o que não pode
// decidir onde uma operação vetorial mora.
// ⚠️ **O `add` vai junto, e pelo IRMÃO [`super::mesh_filter`]**: o filtro
// desloca `base + n·f`, que é literalmente esta operação — e uma segunda cópia
// dela lá seria a segunda resposta a *"como se soma um vetor a um ponto"*.
pub(super) use aim::{add, cross};

impl SculptStroke {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn compute_target(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        plane: &PlaneFit,
        // **A NORMAL DO GESTO**, quando o verbo em mãos a pede — ver
        // `stroke_normal_do_gesto.rs`.
        //
        // ⚠️ **Ela é PASSADA e não computada aqui pela mesma razão que o `w`:**
        // é uma varredura da pegada, e esta função corre **por vértice**.
        // Derivá-la cá dentro faria um dab custar o quadrado do que custa.
        //
        // ⚠️ **`None` não é «não há normal»** — é *este verbo não pergunta*. Os
        // dois que perguntam caem na normal do plano do carimbo quando a
        // amostragem degenera, que é a única resposta finita.
        n_gesto: Option<[f32; 3]>,
        reach: f32,
        shape: f32,
        // O peso COMPLETO deste dab (`falloff × intensidade × máscara`) — o
        // mesmo `w` que o aplicador usaria como `accum`.
        //
        // ⚠️ **Ele é PASSADO e não recomputado**, e o motivo está no comentário
        // do `w` em `stroke.rs`: derivá-lo aqui (`shape × strength × pressure`)
        // re-associa o produto de três fatores, e **30,4% dos triplos divergem
        // até um ulp**. Leem-no os TRÊS verbos cujo alvo já é a posição final
        // (`SnakeHook`, `Twist`, `LocalScale` — os que carimbam `accum = 1`); os
        // outros treze compõem o alvo sem peso, e é o `accum` que os atenua.
        //
        // ⚠️ **E os verbos servidos por um [`crate::Field`] o leem também**,
        // porque com um campo a curva vira a INDICADORA do suporte: `w` já é *o
        // peso sem geometria*, ao bit. O parâmetro `flat` que existia para eles
        // morreu com essa troca — ver o sítio que constrói o `w`.
        w: f32,
        // **A demão JÁ acumulada depois deste dab** — o factor de deslocamento
        // do *Layer* da referência, depois de descontar o já depositado.
        //
        // ⚠️ **Só a família do `GripLaw::coat` o lê**, e ele é PASSADO em vez de
        // lido de `self.accum[s]` por uma razão de ORDEM: o `accum[s]` ainda
        // guarda o valor do dab ANTERIOR quando esta função corre (quem o
        // escreve é o `scatter_one`, depois), e a referência compõe a translação
        // com o valor NOVO. Ler o campo aqui atrasaria a demão em um dab — o
        // defeito que não aparece num traço longo e aparece num toque.
        disp: f32,
        v: u32,
        s: usize,
    ) -> [f32; 3] {
        let base = self.base_pos[s];
        // ⚠️ **A família do CARIMBO ancora no VIVO** (2026-08-11, a metade 2 do
        // plano de paridade): cada dab soma o próprio incremento sobre o que o
        // anterior deixou, que é a estrutura do kernel da referência
        // (`vAr[ind] = vx + anx * fallOff`). O `base` continua guardado e
        // intocado — ele é o undo, e é o `pre` que a distância lê quando o
        // Accumulate está desarmado.
        let live = mesh.positions()[v as usize];
        let n_area = plane.normal;
        // O SINAL do gesto invertido, que a referência dobra dentro do `deform`.
        let sign = if brush.invert && brush.verb.honours_invert() {
            -1.0
        } else {
            1.0
        };
        match brush.verb {
            // ⛔⛔ **INALCANÇÁVEL, e não é uma omissão:** o tecido não tem alvo
            // por-vértice — ele tem um SOLVER, e o `SculptStroke::dab` desvia
            // para o `stroke_cloth` antes de este laço existir. Devolver a
            // posição viva é a resposta correta para *«e se alguém chegar aqui
            // mesmo assim?»*: não mover nada. ⚠️ Um `unreachable!()` aqui
            // trocaria um no-op por um panic no caminho do ponteiro.
            // ⛔⛔ **INALCANÇÁVEL, e pela razão mais forte da família:** o
            // Box Trim não tem dab nenhum — ele intercepta o ARRASTO, e o que
            // muda a peça é uma booleana sobre a malha inteira, no pen-up. A
            // resposta é a dos irmãos sem alvo por-vértice: não mover nada.
            Verb::BoxTrim => live,
            Verb::Cloth => live,
            // ⛔⛔ **INALCANÇÁVEL pela MESMA razão, e a resposta é a mesma:** a
            // pose não tem alvo por-vértice — ela tem uma cadeia de mapas
            // afins, e o `SculptStroke::dab` desvia para o `stroke_pose` antes
            // deste laço existir. Não mover nada é a resposta correcta para
            // *«e se alguém chegar aqui mesmo assim?»*; um `unreachable!()`
            // trocaria um no-op por um panic no caminho do ponteiro.
            Verb::Pose | Verb::Boundary => live,
            // ⛔⛔ **INALCANÇÁVEL, e a razão é a mais forte das três:** a
            // densidade não tem alvo por-vértice porque **não tem lei
            // por-vértice** — o `SculptStroke::dab` devolve antes deste laço
            // existir ([`Verb::sem_lei_por_vertice`]). Não mover nada é a
            // resposta correcta para *«e se alguém chegar aqui mesmo assim?»*.
            Verb::Density => live,
            // ⭐⭐ **O APAGADOR — uma interpolação LINEAR pura em direcção à
            // superfície de referência** (espec §4.1): `p ← p + f·(R − p)`, sem
            // direcção privilegiada, sem normal e sem acumulador.
            //
            // ⭐⭐⭐ **A FORÇA ENTRA AO QUADRADO, e o quadrado é ESCRITO AQUI —
            // não sai da composição.** Espec §1.1, medida: com a curva
            // *Constant* e força `1` o vértice pousa **na** referência
            // (`1,000000`); com força `0,5` percorre **`0,250000`** do caminho,
            // e não `0,500`.
            //
            // ⛔⛔ **E o padrão do [`Verb::Thumb`] NÃO TRANSFERE para cá, apesar
            // de parecer o mesmo.** Lá o quadrado sai de graça da composição —
            // o alvo leva um peso e o aplicador multiplica pelo `accum`, que
            // leva o outro. Mas aquele verbo é [`Grip::Hold`], e a tabela do
            // [`crate::GripLaw`] dá `unit_accum = FALSE` a ele; o
            // [`Grip::Stamp`] — o nosso — dá **`true`**: *o alvo já traz o peso
            // e o `accum` vale 1*. Escrito à maneira do polegar, este pincel
            // media `0,500` a meio curso, que é exactamente o *«parecer o dobro
            // de forte»* que a §1.1 avisa.
            //
            // ⚠️ **`w × strength` é a conta certa e não `w × w`:** `w` é
            // `peso × força × pressão`, logo o produto dá
            // `peso × força² × pressão` — a **força** ao quadrado e a **pressão**
            // linear, que é a fórmula da espec letra por letra. `w × w`
            // elevaria a pressão e a curva de queda junto.
            //
            // ⛔ **E o tecto `min(f, 1)` sai de graça:** `w ≤ 1` por construção
            // (é um produto de factores em `[0, 1]`), logo ele **nunca
            // ultrapassa** a referência. *Não há «apagar demais».*
            //
            // ⚠️⚠️ **Sem referência fotografada ele devolve o VIVO — não move
            // nada.** É a resposta certa para um traço sem pilha de
            // multiresolução: ali o dado de entrada não existe, e *a lei não
            // inventa uma superfície*. A recusa em voz alta é da shell (§4.3);
            // esta linha é a rede que garante que contorná-la não inventa
            // geometria.
            Verb::EraseMultires => self
                .reference
                .get(v as usize)
                .map_or(live, |&r| toward3(live, r, w * brush.strength)),
            // ⭐⭐ **O ESFREGÃO — ele não move o vértice para onde a mão vai;
            // move o CAMPO DE DESLOCAMENTO sobre a superfície de referência**
            // (espec §5.2). A lei inteira vive no irmão
            // [`super::stroke_smear`], porque ela lê o campo dos VIZINHOS e não
            // cabe numa função pura por-vértice — a mesma razão do
            // [`Verb::SurfaceSmooth`].
            Verb::SmearMultires => self.smear_target(mesh, brush, dab, v, s, w),
            // ⭐⭐⭐ **PROJECTAR NA CENA** — o vértice viaja até **encostar
            // noutra peça**. A lei inteira vive no irmão [`crate::projectar`],
            // pela mesma razão do [`Verb::SmearMultires`]: ela não cabe numa
            // função pura por-vértice deste `match` — precisa das outras peças
            // da cena e da régua em que elas competem.
            Verb::SceneProject => {
                crate::projectar::alvo_do_vertice(self, brush, dab, n_area, live, w)
            }
            // `Brush.js:57-91` — `deform = intensidade · raio · 0,1`, e o peso
            // inteiro (curva × intensidade × máscara × alpha) chega no `w`.
            Verb::Draw => add(live, n_area, reach * w),
            // ⭐⭐⭐ **O AFIADO é o Draw com OUTRA NORMAL, e mais nada neste
            // `match`** (espec §2.3, decisão D-3): a direcção do deslocamento é
            // a **normal da área do alvo** — a mesma lei que os gestos
            // tangenciais e o pincel de plano já correm —, e não a normal do
            // estimador de plano da casa.
            //
            // ⚠️ **O que separa este pincel do Draw NÃO se vê aqui:** a
            // distância de onde a curva pesa (o `from_live` pregado), a curva,
            // a direcção de fábrica, o passo e a atenuação chegam todos DENTRO
            // do `w` e do `reach`. *Um segundo braço aritmético aqui seria a
            // segunda resposta a «para onde este dab empurra».*
            //
            // ⚠️ **O recuo é o do irmão:** sem amostra nenhuma dentro da fracção
            // do raio, a normal da área degenera e a única resposta finita é a
            // do plano do carimbo.
            Verb::DrawSharp => add(live, n_gesto.unwrap_or(n_area), reach * w),
            // ⚠️ **A LEI É A DO DRAW, ao bit — e isso É a wave.** O que separa
            // uma faixa de um domo não é para onde o barro vai (os dois sobem
            // pela normal da área, pelo mesmo `reach`), é **QUE BARRO** a
            // passada alcança: a silhueta em caixa e o portão parabólico da
            // profundidade vivem no `w`, pela [`crate::Footprint`].
            //
            // ⚠️ **Um segundo braço aritmético aqui seria a segunda resposta a
            // *"para onde este dab empurra"*** — e o dia em que o Draw ganhasse
            // um termo, a faixa não o ganharia, em silêncio.
            Verb::ClayStrips => add(live, n_area, reach * w),
            // `Inflate.js:36-76` — a MESMA constante do Draw, pela normal de
            // cada vértice em vez da de área.
            //
            // ⚠️ **A normal é a CONGELADA no pen-down, e as DUAS referências
            // leem a viva** (`Inflate.js:64-66` do SculptGL, MIT, e o *Inflate*
            // da referência). É divergência
            // nossa, ela é deliberada — a normal viva sobe junto com a tinta, e
            // um traço parado passaria a inflar numa direção que gira sozinha —
            // e desde 2026-08-12 ela tem **NÚMERO**, porque a frase acima era uma
            // afirmação sobre uma grandeza que ninguém tinha medido.
            //
            // ⚠️ **MEDIDO** (`tests/measure_inflate_normal_drift.rs`, traço
            // parado, raio 0,45): a normal de um vértice da pegada gira
            //
            // | força | 1 dab | 4 | 16 | 64 |
            // |---|---|---|---|---|
            // | 0,3 | 2,8° | 10,9° | 33,4° | **53,4°** |
            // | 1,0 | 9,2° | 30,2° | 52,2° | **58,4°** |
            //
            // (pior caso; a média da pegada acompanha, e o MIOLO gira menos —
            // 15-18° — porque ali a superfície sobe sem inclinar tanto).
            //
            // ⇒ **A cerca está CERTA, e o preço dela é paridade.** Meio ângulo
            // reto não é um detalhe numérico: um traço parado com a normal viva
            // arrastaria a direção do empurrão para longe do que o artista
            // apontou. Quem quiser a lei da referência ganha o ramo de MODO
            // (a `KernelLaw`), não uma troca do default.
            Verb::Inflate => add(live, self.base_nrm[s], reach * w),
            // `Smooth.js` — o único tool de geometria da referência **sem
            // falloff** no kernel; aqui ele chega pelo `w`, que é o superconjunto
            // (o artista escolhe a curva, e `Falloff::Constant` reproduz a
            // referência). A forma é a dela: `pos·(1 − m) + média·m`.
            // A família que lê o ANEL vive no irmão [`ring`].
            Verb::Smooth => self.target_smooth(mesh, brush, v, live, w),

            // NOSSO, e a referência não tem: o laplaciano com o sinal trocado.
            // Reflete a média através do próprio vértice, com a mesma magnitude
            // que o Smooth teria.
            // A família que lê o ANEL vive no irmão [`ring`].
            Verb::Sharpen => self.target_sharpen(mesh, brush, v, live, w),

            // `Flatten.js:41-81` — o deslocamento é **proporcional à distância ao
            // plano**, não a uma constante do pincel: longe ele anda muito, perto
            // anda pouco. É isso que faz o verbo CONVERGIR em vez de oscilar, e é
            // por isso que ele não tem `reach`.
            //
            // ⚠️ **O nosso Flatten é BILATERAL e o da referência não é** — lá o
            // `comp = ±1` escolhe um lado e o outro é `continue`, então o
            // `Flatten` deles é o nosso `Fill` ou o nosso `Scrape`. Os dois lados
            // ao mesmo tempo é a leitura do Blender, e é a que o artista espera
            // de um verbo chamado *achatar*.
            //
            // ⚠️ **E é aqui que o modo entra.** Em `S` o verbo morde UM lado —
            // o `comp = −1` que o `Flatten.js:11` traz de fábrica, ou seja o
            // lado que RASPA; quem quer o outro escolhe o `Fill`, que é o mesmo
            // kernel com o flag virado. Em `B` ele morde os dois, que é o
            // verbo de PLANO da referência (um controlo acima, outro abaixo).
            // ⭐⭐⭐ **O PINCEL DE PLANO — e o alvo dele é a projecção BILATERAL,
            // sem um único caso especial.** A espec §4 fecha-se em
            //
            // ```text
            // translação(v) = − n · força² · factor(v) · distância_com_sinal(v)
            // ```
            //
            // que é **literalmente** o que a [`to_plane`] escreve (`p − n·(d·w)`)
            // com `w = factor × intensidade`, e a intensidade já traz a força ao
            // quadrado pela porta [`crate::Brush::weight`].
            //
            // ⭐⭐ **Quem escolhe o LADO não é este `match` — é a PEGADA.** Nos
            // quatro verbos de plano da casa o lado é um `if d > 0.0` aqui; aqui
            // ele vive na [`crate::Footprint::Tectos`], que põe o vértice **fora
            // da silhueta** quando o tecto daquele lado é zero. ⇒ *é isso que
            // transforma três verbos num só*: `altura 1 / profundidade 0` dá o
            // raspar, `0 / 1` dá o encher, `1 / 1` dá o achatar — medido, `151`
            // e `114` vértices que somam exactamente os `265` do bilateral.
            //
            // ⚠️ **E é por isso que ele não lê o [`crate::PlaneReach`]:** aquele
            // enum responde *«este verbo toca um lado ou os dois?»*, e para este
            // a resposta é do artista, num par de números contínuos.
            Verb::Plane => {
                // ⚠️ **O plano é o DESTE pincel, não o do [`super::plane`]** — ver
                // [`super::SculptStroke::plano`], escrito pela construção da
                // pegada no mesmo dab. `None` é *sem amostra nenhuma*: não mover.
                let Some(p) = self.plano else { return live };
                let d = (live[0] - p.centro[0]) * p.normal[0]
                    + (live[1] - p.centro[1]) * p.normal[1]
                    + (live[2] - p.centro[2]) * p.normal[2];
                // ⭐ **O SINAL é da porta da inversão**, e só a lei *afastar* o
                // move: no modo *trocar os tectos* a força **não muda** (espec
                // §5), e trocar o sinal ali daria as duas leis ao mesmo tempo —
                // uma saída que não é nenhuma delas.
                let s = brush
                    .plano_inversao
                    .sinal(brush.invert && brush.verb.honours_invert());
                // ⭐ O factor do traço arrastado (espec §14.4) — `1` pela porta por script.
                to_plane(live, p.normal, d * s, w * p.factor)
            }
            Verb::Flatten => {
                let d = signed_distance(live, plane);
                match brush.mode.kernel_for(brush.verb).plane {
                    crate::PlaneReach::Bilateral => to_plane(live, n_area, d, w),
                    crate::PlaneReach::OneSided if d > 0.0 => to_plane(live, n_area, d, w),
                    crate::PlaneReach::OneSided => live,
                }
            }
            // A metade que SOBE (o `comp = +1` da referência): quem já passou do
            // plano **não é tocado**, e é esse `continue` que torna o verbo
            // auto-limitado.
            Verb::Fill => {
                let d = signed_distance(live, plane);
                if d < 0.0 {
                    to_plane(live, n_area, d, w)
                } else {
                    live
                }
            }
            // A metade que DESCE (`comp = −1`).
            Verb::Scrape => {
                let d = signed_distance(live, plane);
                if d > 0.0 {
                    to_plane(live, n_area, d, w)
                } else {
                    live
                }
            }
            // **O CLAY é o `Fill` contra um plano LEVANTADO**, e não um verbo
            // próprio: `Brush.js:52` empurra o centro de área por `raio · 0,1` e
            // chama o mesmo `Flatten.flatten`. O barro sobe até o plano e PARA —
            // que é o que faz um Clay parecer barro em vez de um Draw.
            //
            // ⚠️ **Ele deixou de ser `achata E acrescenta`** (`add(project(...),
            // n, reach)`), e a metade que se perdia era exatamente a auto-limitação:
            // aquela forma empurrava para sempre. Medido contra a referência
            // naquele desenho: **3,80×**.
            //
            // ⚠️ **O `plane_offset` do artista SOMA a este**, porque o `fit_plane`
            // já o aplicou: com o knob em `0` sai a referência exata, e girá-lo
            // levanta o plano a mais.
            //
            // ⚠️ **O Ctrl DESCE o plano e inverte o LADO**, que é o
            // `this._negative ? -off : off` do `Brush.js:47` seguido do
            // `distToPlane * comp > 0 → continue` do `Flatten.js:63`: o Clay
            // invertido é um **Scrape contra um plano rebaixado**, e não um
            // Clay mais fraco. O `sign` faz as duas metades de uma vez, e é por
            // isso que ele não pode ser um `if` só na altura do plano.
            //
            // ⚠️ **Ele passou a ser INERTE quando o verbo deixou de usar o
            // `reach`** — era por ali que o Ctrl entrava (`Brush::reach`
            // devolve o negativo), e o Clay novo não o consome. O gate
            // `invert_changes_the_result_of_exactly_the_verbs_that_have_an_opposite`
            // pegou na hora.
            Verb::Clay => {
                let d =
                    signed_distance(live, plane) - sign * dab.radius * crate::CLAY_PLANE_FRACTION;
                if d * sign < 0.0 {
                    to_plane(live, n_area, d, w)
                } else {
                    live
                }
            }
            // **O POLEGAR é o Flatten contra um plano INCLINADO** — o *Clay Thumb*.
            //
            // ⚠️ **A projeção é a MESMA do [`Verb::Flatten`], bilateral** (a
            // mesma projeção ao plano, sem compensação e sem teste de lado); o
            // que a ferramenta acrescenta é inteiramente a construção do plano,
            // e por isso ela não traz aritmética de alvo nova.
            //
            // ⚠️ **O plano passa pelo CENTRO DO DAB, não pelo centro de área** —
            // a referência ergue o plano inclinado sobre a posição do dab já
            // espelhada para a cópia de simetria. É a única diferença de ORIGEM
            // entre este verbo e os quatro que a [`super::plane`] serve, e ela é
            // load-bearing: ancorado no centro de área, o plano inclinado
            // deslizaria para trás junto com a média da pegada, e a inclinação
            // deixaria de morder onde a mão está.
            //
            // ⚠️ **Sem eixo não há depósito.** O [`stroke_axis`] responde `None`
            // pelos dois degenerados de uma vez (dab sem caminho · caminho que
            // mergulha na normal), e é ele que reproduz as DUAS saídas
            // antecipadas da referência — o primeiro dab do traço, adiado, e o
            // deslocamento do traço nulo.
            Verb::ClayThumb => match stroke_axis(n_area, dab.path) {
                // ⚠️ **O eixo de INCLINAÇÃO é o do *Clay Thumb* (o produto
                // vetorial da normal de área com o deslocamento do traço), e ele
                // sai do MESMO door
                // que devolve o `Y`** — a referência monta `y = n × x`, e num
                // frame ortonormal isso se inverte exatamente em `x = y × n`.
                // Derivá-lo assim é o que faz a pergunta *"este dab tem
                // direção?"* ser feita **uma vez**; um segundo `cross` com um
                // segundo piso seria a segunda resposta, e o dia em que um dos
                // dois pisos mudasse o verbo depositaria onde o outro recusa.
                Some(along) => {
                    let axis = cross(along, n_area);
                    // O sinal é o da referência (a inclinação entra negativa): a
                    // normal tomba para TRÁS, contra o caminho, e é isso que põe
                    // a borda dianteira do polegar a cortar mais fundo.
                    let tilted =
                        rotate_about(n_area, [0.0; 3], axis, -self.thumb_tilt_deg.to_radians());
                    // ⚠️ **Sem superfície local, e não é omissão:** este plano é
                    // CONSTRUÍDO (a normal de área INCLINADA em torno do eixo do
                    // traço), não o da pegada — uma superfície curva ajustada ao
                    // barro por baixo de uma dobradiça seria uma segunda lei a
                    // discutir com a primeira sobre onde o alvo fica.
                    let tilt_plane = PlaneFit {
                        point: dab.center,
                        normal: tilted,
                        surface: None,
                    };
                    let d = signed_distance(live, &tilt_plane);
                    to_plane(live, tilted, d, w)
                }
                None => live,
            },
            // **A LÂMINA EM V é o [`Verb::Scrape`] contra DOIS planos** —
            // *Multiplane Scrape*. A aritmética do alvo é a mesma projeção
            // dos outros cinco verbos de plano; o que a ferramenta acrescenta é
            // inteiramente *qual* plano serve *qual* vértice, e isso mora na
            // moldura que a [`super::plane::ScrapePlanes`] resolveu uma vez por
            // dab.
            //
            // ⚠️ **O culling é `d <= 0`, não `d < 0`** — na referência um
            // vértice do lado de trás do plano, ou sobre ele, tem factor zero. Um
            // vértice exactamente sobre o meio-plano dele não é matéria a
            // remover, e a diferença é o que impede o miolo do sulco de tremer
            // entre dois dabs.
            //
            // ⚠️ **`self.scrape` é `None` quando este dab não deposita**, e é a
            // mesma recusa do polegar por baixo: sem direção não há dobradiça, e
            // no modo dinâmico um lado sem vértices não tem normal para medir.
            Verb::MultiplaneScrape => match self.scrape {
                Some(s) => {
                    let n = s.normal_at(live);
                    let d = signed_distance(
                        live,
                        // ⚠️ **Sem superfície local, pela razão do polegar:** as
                        // duas facetas do V são planos construídos, e a
                        // dobradiça é a lei desta ferramenta.
                        &PlaneFit {
                            point: s.origin,
                            normal: n,
                            surface: None,
                        },
                    );
                    if s.cull && d <= 0.0 {
                        live
                    } else {
                        to_plane(live, n, d, w)
                    }
                }
                None => live,
            },
            // `Pinch.js:34-66` — `deform = intensidade · 0,05`.
            //
            // ⚠️ **O ganho não existia, e a ausência valia 20×**: o alvo era a
            // tangente inteira atenuada pelo peso, ou seja o vértice caminhava
            // até `w` da distância ao centro **num dab**. Medido: `16,88×`.
            //
            // ⛔ **O DOC QUE ESTAVA AQUI AFIRMAVA O CONTRÁRIO DO QUE O CÓDIGO
            // FAZIA, e ficou dois meses de pé.** Ele dizia: *"com campo ele
            // deixa de REMOVER VOLUME … a `F` tem traço zero, então o que sai de
            // lado sai pela normal: aperta E espirra, que é o que um material
            // faz"*. Medido pela porta do artista
            // (`tests/measure_pinch_family_modes`), o aperto com campo removia
            // **4,8× MAIS** volume que o sem (`−4,43` contra `−0,92`, em
            // 10⁻⁴ de `V`), e dentro do anel o deslocamento normal era
            // **NEGATIVO** — ele afundava.
            //
            // ⚠️ **A álgebra estava certa e a GEOMETRIA do consumidor não:** o
            // traço zero reparte `+s` na normal e `−s/2` no plano, mas os
            // vértices de uma MALHA vivem na superfície (`r · n ≈ 0`), então o
            // termo normal é ~zero. *Uma casca não tem material fora do plano
            // para receber o que sai de lado.* O campo saiu do verbo em
            // 2026-08-15 — ver [`crate::Verb::elastic_field`].
            //
            // ⚠️ **A LEI LATERAL passou a ser função do MODO E DO VERBO**, e é
            // ela que fecha a outra metade do report (*"Pinch em B e S bons mas
            // idênticos ou quase idênticos"*): o `B` daqui é o *Pinch*, que
            // remove a componente ao longo do traço — ver
            // [`crate::RefMode::lateral_for`].
            Verb::Pinch => add_vec(
                live,
                lateral_pull(brush.mode, brush.verb, live, dab.center, n_area, dab.path),
                w * crate::PINCH_GAIN,
            ),
            // ⚠️ **O l-mode do Magnify é a ESCALA, não o aperto invertido.** O
            // `s-mode` é o Pinch com o sinal trocado — um empurrão lateral para
            // fora, que **acrescenta** volume no plano e nada na normal. A
            // família *scale* do paper dilata a vizinhança **radialmente**, que
            // é o que "magnificar" quer dizer, e o perfil decide onde a
            // dilatação acaba.
            Verb::Magnify => match brush.mode.field(Verb::Magnify) {
                Some(_) => {
                    let d = [
                        live[0] - dab.center[0],
                        live[1] - dab.center[1],
                        live[2] - dab.center[2],
                    ];
                    let f = 1.0
                        + w * crate::PINCH_GAIN
                            * crate::kelvinlet::rigid_profile(d, dab.radius, brush.elastic_scales);
                    add_vec(dab.center, d, f.max(0.0))
                }
                _ => add_vec(
                    live,
                    lateral_pull(brush.mode, brush.verb, live, dab.center, n_area, dab.path),
                    -w * crate::PINCH_GAIN,
                ),
            },
            // `Crease.js:38-76` — aperta lateralmente **e** cava, com ganho
            // próprio (`intensidade · 0,07`).
            //
            // ⚠️ **O expoente cai SÓ no termo da normal, e é isso que faz um
            // vinco ser fino:** o puxão lateral é linear no falloff (largo) e o
            // afundamento é quíntico (estreito). O `shape⁴ · w` **é** o `f⁵ ·
            // intensidade` da referência, porque `w = shape · intensidade` — e
            // escrever `shape.powi(5) * intensity` exigiria passar a intensidade
            // por um parâmetro que já viaja dentro do `w`.
            //
            // ⚠️ **A máscara entra DENTRO do expoente** (`Crease.js:67` roda
            // antes do `:68`): um vértice meio-mascarado leva `0,5⁵ = 3%` do
            // empurrão normal e `0,5` do lateral. A assimetria é da referência.
            //
            // ⛔ **O `l-mode` COMPOSTO deste verbo MORREU em 2026-08-15.** Ele
            // era o único da tabela cujo deslocamento não vinha inteiro do
            // kernel, e a nota aqui media o que acontecia se a lei *"com campo, a
            // curva é o SUPORTE do campo"* alcançasse também a metade que não é
            // do campo (o vinco virava CRATERA: 82 % do bico a um raio e meio).
            // A cura de então foi tomar a estreiteza do perfil do próprio
            // Kelvinlet; a medição de agora diz que o problema era um degrau
            // acima — **43,7 % do gesto caía fora do anel do cursor**, e nenhuma
            // das três referências declara um aperto elástico. Ver
            // [`crate::Verb::elastic_field`].
            Verb::Crease => {
                let t = lateral_pull(brush.mode, brush.verb, live, dab.center, n_area, dab.path);
                let gain = w * crate::CREASE_FRACTION;
                add(
                    add_vec(live, t, gain * brush.pinch),
                    n_area,
                    -gain * shape.powi(4) * dab.radius * sign,
                )
            }
            // **O BLOB** (na referência é a MESMA lei do Crease com o booleano
            // de inversão ligado) — o Crease com o aperto lateral NEGADO e o depósito para
            // CIMA. Ver [`Verb::Blob`] para por que a direção é nossa.
            //
            // ⚠️ **Os DOIS sinais mudam, e não é simetria por gosto:** negar só
            // o lateral daria um monte que ainda CAVA (o `-` do Crease é o
            // `_negative` do `Crease.js`, herdado); negar só o normal daria um
            // Crease erguido, que é outro verbo — precisamente o que o `Ctrl` no
            // Crease já entrega. É a combinação que não é alcançável por nenhum
            // ajuste do vizinho, e é isso que o torna um verbo em vez de um flag.
            //
            // ⚠️ **O `shape⁴` FICA, e ele é o que separa os dois na TELA:** o
            // termo normal é quíntico no falloff (estreito) e o lateral é linear
            // (largo), então no Crease o resultado é um sulco fino dentro de um
            // aperto largo. No Blob a mesma assimetria vira um **domo estreito
            // dentro de um empurrão largo** — o barro sai de baixo e sobe no
            // meio, que é a forma que a palavra descreve.
            Verb::Blob => {
                let t = lateral_pull(brush.mode, brush.verb, live, dab.center, n_area, dab.path);
                let gain = w * crate::CREASE_FRACTION;
                add(
                    add_vec(live, t, -gain * brush.pinch),
                    n_area,
                    gain * shape.powi(4) * dab.radius * sign,
                )
            }
            // **O RELAX** — a mesma média do [`Verb::Smooth`] com **uma** linha a
            // mais, e essa linha é a ferramenta inteira: o que corre ao longo da
            // normal é REMOVIDO, então o que sobra desliza pela superfície e a
            // forma não se mexe (o deslocamento é projectado no plano tangente).
            //
            // ⚠️ **A normal é a VIVA (`mesh.normals()`), ao contrário do
            // [`Verb::Inflate`]** — e as duas escolhas estão certas porque a
            // grandeza tem papéis opostos nos dois: lá ela é a direção em que se
            // ANDA (congelar impede o empurrão de girar sob um traço parado),
            // aqui ela é o plano em que se FICA (congelar prenderia o vértice ao
            // plano tangente do pen-down, e ele sairia da superfície que os dabs
            // anteriores moveram). Ver o doc de [`Verb::SlideRelax`].
            //
            // ⚠️ **Numa BORDA a normal vira a bissetriz**, e isso é o que impede
            // a beira de encolher — ver [`Self::relax_normal`].
            // A família que lê o ANEL vive no irmão [`ring`].
            Verb::SlideRelax => self.target_slide_relax(mesh, brush, v, live, w),

            // A família que lê o ANEL vive no irmão [`ring`].
            Verb::SurfaceSmooth => self.target_surface_smooth(mesh, v, s, live, w, brush),
            // **A DEMÃO — o alvo é a camada CHEIA, e o `accum` é a fração dela
            // já depositada.** É a lei do *Layer* da referência, com o factor de
            // deslocamento a sair do nosso `accum` em vez de um plano próprio
            // ([`crate::GripLaw::coat`]).
            //
            // ⚠️ **`base` e `base_nrm`, os dois CONGELADOS** — a referência lê
            // as posições e normais congeladas do pen-down, e aqui isso é o que
            // torna a demão idempotente sob re-carimbo: um shape editor que
            // re-emite a lista inteira de dabs a cada quadro tem de chegar ao
            // mesmo lugar, e um alvo ancorado no VIVO subiria a cada passada.
            //
            // ⚠️ **O PESO ENTRA AQUI, e o alvo é a POSIÇÃO FINAL** — o
            // alvo do *Layer* da referência é `live + (meta − live) · factors`, com `meta =
            // base + normal_base · altura · disp`. Duas coisas que esta linha
            // corrige de uma vez, e as duas estavam escritas aqui como decisão:
            //
            // ⚠️ **(1) A translação sai do VIVO, não do `base`.** A frase antiga
            // dizia *"um alvo ancorado no VIVO subiria a cada passada"* — falso,
            // e o `disp` é o motivo: ele SATURA (`coat_step`), então a meta é
            // limitada por construção e ancorar no vivo não a deixa crescer. O
            // que o `base` congelado governa é a META e a DISTÂNCIA da curva
            // (as posições congeladas do pen-down, e isso continua), nunca de
            // onde se anda.
            //
            // ⚠️ **(2) O peso multiplica, e não é dobrar o perfil.** A frase
            // antiga dizia *"o aplicador multiplica pelo `accum`, um alvo já
            // pesado aplicaria o perfil duas vezes"* — e a referência de facto
            // o aplica nos DOIS lugares, porque eles respondem perguntas
            // diferentes: dentro da recorrência da demão o `factors` é
            // a TAXA com que aquele vértice enche a demão, e aqui ele é a
            // FRAÇÃO do caminho até a meta que este dab anda. Sem o segundo a
            // demão escreve a meta de forma ABSOLUTA — e aí ela sobrescreve o
            // que o Auto Smooth acabou de alisar (medido: `auto_smooth 0,5`
            // tirava **14,9×** do relevo da demão contra **4,2×** do Draw) e o
            // Hardness deixa de ter forma, porque a única coisa que ele muda é
            // a taxa. Era o report do Enio, nos dois eixos.
            //
            // ⚠️ **`factors` é o `shape` e NÃO o `w`** — a cadeia de fatores da
            // referência usa só a avaliação da curva de queda, ou seja **a
            // curva, sem a força**; a força do traço é o nosso `intensity` e já
            // entra na recorrência. Passar o `w` aqui aplicaria a força duas
            // vezes.
            Verb::Layer => {
                let n = self.base_nrm[s];
                let h = sign * brush.layer_height * disp;
                let goal = [base[0] + n[0] * h, base[1] + n[1] * h, base[2] + n[2] * h];
                [
                    live[0] + (goal[0] - live[0]) * shape,
                    live[1] + (goal[1] - live[1]) * shape,
                    live[2] + (goal[2] - live[2]) * shape,
                ]
            }
            // O alvo de posição de um verbo de máscara é o próprio lugar: ele
            // não move geometria ([`crate::Grip::Paint`]), e `apply_mask` é quem
            // escreve o canal dele.
            // ⚠️ **Os DOIS verbos de CANAL devolvem o `base`** — eles não movem
            // um vértice, e o alvo neutro é o que garante que o aplicador de
            // posições, se algum dia for chamado por engano, não move nada.
            Verb::Mask | Verb::Paint => base,
            // **OS QUATRO GESTOS COM ÂNCORA** vivem no irmão [`gripped`] — a
            // família que a [`Verb::anchors`] nomeia.
            // **OS QUATRO GESTOS COM ÂNCORA**, mais os **DOIS TANGENCIAIS** que
            // herdam o grip deles e trocam só o que o offset é.
            Verb::Move
            | Verb::SnakeHook
            | Verb::Twist
            | Verb::LocalScale
            | Verb::Thumb
            | Verb::Nudge => {
                self.target_gripped(brush, dab, w, base, live, n_gesto.unwrap_or(n_area))
            }
        }
    }
}

/// **AS LEIS QUE LEEM O ANEL** — a família que [`Verb::uses_neighbours`] nomeia.
/// Ver [`ring`].
/// **As peças de VECTOR da lei do alvo** — ver [`vetores`](self).
#[path = "stroke_target_vetores.rs"]
mod vetores;
pub(super) use vetores::{lateral_pull, stroke_axis, toward3};

#[path = "stroke_target_ring.rs"]
mod ring;

/// **OS QUATRO GESTOS COM ÂNCORA** — a família que [`Verb::anchors`] nomeia.
/// Ver [`gripped`].
#[path = "stroke_target_grip.rs"]
mod gripped;
