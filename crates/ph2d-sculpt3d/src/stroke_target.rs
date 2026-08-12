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

/// O plano ajustado à pegada de um dab.
///
/// ⚠️ **Inclinado, nunca horizontal** — um ajuste horizontal *cava uma cratera
/// na encosta* em vez de achatá-la (lição paga no `plane.rs` do Painter 2D).
/// O estimador é a média ponderada pelo falloff das posições e das normais da
/// pegada, que é o `calc_area_normal_and_center` do Blender; ele difere de um
/// ajuste por mínimos quadrados de verdade numa sela, e a divergência está
/// registrada aqui em vez de escondida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PlaneFit {
    pub(super) point: [f32; 3],
    pub(super) normal: [f32; 3],
}

impl SculptStroke {
    pub(super) fn fit_plane(&self, brush: &Brush, dab: &Dab) -> PlaneFit {
        // ⚠️ **O conjunto FRONTAL, e é a metade que o original faz
        // INCONDICIONALMENTE.** O `getFrontVertices` (`SculptBase.js:206-221`)
        // filtra por `n · eyeDir <= 0`, e o `Brush.js:32-34` / `Flatten.js:25-27`
        // o consomem sem perguntar a ninguém — é ele que decide a DIREÇÃO do
        // Draw e o PLANO do Flatten.
        //
        // ⚠️ **A outra metade do culling — filtrar o que se MOVE — NÃO entra.**
        // Ela é um checkbox do usuário, `_culling = false` por default em dez
        // tools (`GuiSculptingTools.js:62`), e portá-la ligada seria divergir
        // com a ferramenta em silêncio (livro-razão §A).
        //
        // Sem isto, um dab perto da silhueta ajusta o plano com vértices que
        // olham para o OUTRO lado, e o Draw empurra numa direção que o artista
        // não vê.
        let mut fit = self.fit_plane_over(brush, dab, true);
        if fit.is_none() {
            // Pegada inteiramente de costas (um dab que pegou só o outro lado da
            // silhueta): sem frontais não há o que cullar, e recusar aqui seria
            // devolver um plano NaN. A pegada inteira é a melhor resposta que
            // existe, e é a que havia antes desta fatia.
            fit = self.fit_plane_over(brush, dab, false);
        }
        fit.unwrap_or(PlaneFit {
            point: dab.center,
            normal: [0.0, 1.0, 0.0],
        })
    }

    /// O ajuste sobre a pegada, opcionalmente só nos vértices que olham para o
    /// olho. `None` = ninguém pesou (conjunto vazio, ou todo peso zero).
    fn fit_plane_over(&self, brush: &Brush, dab: &Dab, front_only: bool) -> Option<PlaneFit> {
        // ⚠️ **O PLANO É O DA REFERÊNCIA, e o produto CHAMA os kernels
        // portados** (`SculptBase.js:224-261`) em vez de os re-derivar. Até
        // 2026-08-11 esta função tinha uma soma própria, ponderada pelo
        // **FALLOFF** e lida do `pre` congelado, com o racional escrito ao lado:
        // *"o plano descreve a superfície sob o pincel, e força/pressão/máscara
        // dizem o quanto agir sobre ela, não que forma ela tem"*. O racional é
        // defensável e o **preço estava medido**: a NORMAL saía idêntica
        // (`cos 1,000000`) e o CENTRO ficava **0,029 fora, inteiramente ao
        // longo da normal** — `5,8 %` do raio.
        //
        // ⚠️ **E é por isso que o Draw não sentia e o Flatten sentia.** O Draw
        // consome só a direção (media `1,01×`); quem consome o PONTO entra pelo
        // `signed_distance`, que enxerga exatamente a componente ao longo da
        // normal — o Flatten media `0,54×` e o Clay `1,74×`.
        //
        // ⚠️ **A ponderação é a MÁSCARA, não o falloff** (`mAr[ind + 2]`), e a
        // leitura é a do estado congelado deste traço. A referência lê a malha
        // viva; nós lemos o `pre`, e a divergência é a mesma que o
        // [`crate::Grip::Stamp`] já carrega — o peso de um dab é função do
        // estado congelado, e um plano que subisse com a própria tinta faria o
        // Flatten perseguir a superfície que ele está achatando.
        let front = |v: u32| {
            let n = self.base_nrm[self.slot[v as usize] as usize];
            !front_only || n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2] <= 0.0
        };
        // O peso da referência: `1` é livre. O nosso `DEFAULT_MASK` é o oposto,
        // e o `free_weight` é a porta única dessa conversão.
        let free = |v: u32| {
            if front(v) {
                f64::from(crate::mask_ops::free_weight(
                    self.base_mask[self.slot[v as usize] as usize],
                ))
            } else {
                0.0
            }
        };
        let normal = crate::ref_kernels::area_normal_with(&self.footprint, |v| {
            let n = self.base_nrm[self.slot[v as usize] as usize];
            ([f64::from(n[0]), f64::from(n[1]), f64::from(n[2])], free(v))
        })?;
        let point = crate::ref_kernels::area_center_with(&self.footprint, |v| {
            let p = self.base_pos[self.slot[v as usize] as usize];
            ([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])], free(v))
        })?;
        let mut point = [point[0] as f32, point[1] as f32, point[2] as f32];
        let normal = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
        // O offset move o PLANO, não os vértices — é o knob que faz do Flatten
        // um Clay sem um segundo verbo.
        //
        // ⚠️ **Ele fica DEPOIS do kernel de propósito:** o `areaCenter` da
        // referência não o conhece (é nosso), e aplicá-lo por dentro faria a
        // soma que os gates de paridade comparam deixar de ser a dela.
        let off = brush.plane_offset * dab.radius;
        for k in 0..3 {
            point[k] += normal[k] * off;
        }
        Some(PlaneFit { point, normal })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn compute_target(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        plane: &PlaneFit,
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
        w: f32,
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
            // `Brush.js:57-91` — `deform = intensidade · raio · 0,1`, e o peso
            // inteiro (curva × intensidade × máscara × alpha) chega no `w`.
            Verb::Draw => add(live, n_area, reach * w),
            // `Inflate.js:36-76` — a MESMA constante do Draw, pela normal de
            // cada vértice em vez da de área.
            //
            // ⚠️ **A normal é a CONGELADA no pen-down, e a referência lê a
            // viva.** É divergência nossa e ela é deliberada: a normal viva sobe
            // junto com a tinta, e um traço parado passaria a inflar numa
            // direção que gira sozinha. Fica nomeada em vez de descoberta.
            Verb::Inflate => add(live, self.base_nrm[s], reach * w),
            // `Smooth.js` — o único tool de geometria da referência **sem
            // falloff** no kernel; aqui ele chega pelo `w`, que é o superconjunto
            // (o artista escolhe a curva, e `Falloff::Constant` reproduz a
            // referência). A forma é a dela: `pos·(1 − m) + média·m`.
            Verb::Smooth => {
                let avg = self.neighbour_average(mesh, v, live);
                let m = 1.0 - w;
                [
                    live[0] * m + avg[0] * w,
                    live[1] * m + avg[1] * w,
                    live[2] * m + avg[2] * w,
                ]
            }
            // NOSSO, e a referência não tem: o laplaciano com o sinal trocado.
            // Reflete a média através do próprio vértice, com a mesma magnitude
            // que o Smooth teria.
            Verb::Sharpen => {
                let avg = self.neighbour_average(mesh, v, live);
                [
                    live[0] + (live[0] - avg[0]) * w,
                    live[1] + (live[1] - avg[1]) * w,
                    live[2] + (live[2] - avg[2]) * w,
                ]
            }
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
            Verb::Flatten => to_plane(live, n_area, signed_distance(live, plane), w),
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
            // `Pinch.js:34-66` — `deform = intensidade · 0,05`.
            //
            // ⚠️ **O ganho não existia, e a ausência valia 20×**: o alvo era a
            // tangente inteira atenuada pelo peso, ou seja o vértice caminhava
            // até `w` da distância ao centro **num dab**. Medido: `16,88×`.
            Verb::Pinch => add_vec(
                live,
                tangential(live, dab.center, n_area),
                w * crate::PINCH_GAIN,
            ),
            Verb::Magnify => add_vec(
                live,
                tangential(live, dab.center, n_area),
                -w * crate::PINCH_GAIN,
            ),
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
            Verb::Crease => {
                let t = tangential(live, dab.center, n_area);
                let gain = w * crate::CREASE_FRACTION;
                add(
                    add_vec(live, t, gain * brush.pinch),
                    n_area,
                    -gain * shape.powi(4) * dab.radius * sign,
                )
            }
            // O alvo de posição de um verbo de máscara é o próprio lugar: ele
            // não move geometria ([`crate::Grip::Paint`]), e `apply_mask` é quem
            // escreve o canal dele.
            Verb::Mask => base,
            // **O GRAB.** O alvo é o `pre` deslocado pelo gesto INTEIRO: o
            // miolo acompanha o dedo e a borda fica, que é o que *"pego o barro
            // e ele vem comigo"* significa.
            //
            // ⚠️ **O peso NÃO entra aqui, e ele entrava** (`add_vec(base, pull,
            // shape)`, até 2026-08-01). O aplicador multiplica `(alvo − base)`
            // pelo `accum`, então um alvo já pesado aplica o falloff **duas
            // vezes** — medido em `tests/measure_pull_profile.rs`, a meio raio a
            // referência move `0,22500` e nós movíamos `0,12226`, que é
            // `pull·fall²` ao milésimo. O pincel saía pontudo: a borda da pegada
            // mal andava e o gesto lia como *"o Grab pega menos barro do que o
            // círculo promete"*. O `Move.js:120` aplica `fallOff` uma vez.
            //
            // ⚠️ **O gate do miolo não podia ver isto**: em `fall == 1` os dois
            // são o mesmo número, e é o miolo que ele mede. Quem vê é o PERFIL.
            // A máscara continua entrando uma vez, pelo `accum`.
            Verb::Move => add_vec(base, dab.pull, 1.0),
            // **O SNAKE HOOK** — o único alvo desta tabela que **não** parte do
            // `base`: ele parte de onde o vértice ESTÁ e soma o incremento deste
            // dab. É o revezamento ([`Grip::Hook`]), e é por isso que ele
            // precisa do `w` — o `accum` dele vale 1 e não atenua nada.
            //
            // ⚠️ **A leitura da posição viva é a MESMA que o `dab_core` usou
            // para medir a distância** (o `from` de lá): dois `mesh.positions()`
            // no mesmo dab devolvem o mesmo número, então não há duas verdades
            // — há uma verdade lida em dois lugares do mesmo instante. Se um dia
            // o aplicador passar a escrever DENTRO do laço, esta é a linha que
            // deixa de valer.
            Verb::SnakeHook => add_vec(mesh.positions()[v as usize], dab.pull, w),
            // **O TWIST** — gira o `pre` em torno da reta que passa pela âncora
            // na direção de quem olha. Ver [`Grip::Turn`].
            //
            // ⚠️ **O peso entra no ÂNGULO, e é essa linha inteira.** O aplicador
            // recebe `accum = 1`, então o que sai daqui é a posição FINAL — e
            // tem de ser, porque interpolar entre `base` e a posição girada
            // cortaria pela corda do arco e encolheria o barro na direção da
            // âncora quanto maior o giro. Com `θ·w` o vértice anda **sobre** a
            // circunferência, e a distância dele ao eixo é a mesma no fim.
            //
            // ⚠️ **É por isso que o peso é constante ao longo do gesto:** a
            // distância sai do `pre` congelado e uma rotação em torno de um eixo
            // que passa pela âncora **preserva** a distância à âncora — as duas
            // metades concordam por construção, e não por sorte.
            // ⚠️ **O eixo é MENOS o olho, e o sinal é a ferramenta inteira.** O
            // [`Dab::eye`] aponta *do olho para a superfície* — para DENTRO da
            // tela —, e pela regra da mão direita um giro positivo em torno dele
            // sai **horário** para quem olha. Varrer o dedo no anti-horário
            // torceria o barro ao contrário: manipulação direta invertida, que é
            // o mesmo erro que o smoke pegou nos dois sinais da órbita. O
            // original nega exatamente aqui (`Twist.js:41`,
            // `vec3.negate(twistData.normal, picking.getEyeDirection())`).
            Verb::Twist => rotate_about(
                base,
                dab.center,
                [-dab.eye[0], -dab.eye[1], -dab.eye[2]],
                dab.amount * w,
            ),
            // **O LOCAL SCALE** — afasta (ou aproxima) o `pre` da âncora.
            //
            // ⚠️ **O fator é clampado em ZERO, e o limite não é um palpite:** é
            // onde a operação deixa de estar definida. Um fator negativo não
            // encolhe mais — ele **reflete** a pegada através da âncora, virando
            // aquele pedaço da malha do avesso (normais para dentro, faces
            // invertidas). Colapsar na âncora é o fim honesto do gesto.
            Verb::LocalScale => {
                let f = (1.0 + dab.amount * w).max(0.0);
                let d = [
                    base[0] - dab.center[0],
                    base[1] - dab.center[1],
                    base[2] - dab.center[2],
                ];
                add_vec(dab.center, d, f)
            }
        }
    }

    /// A média das posições **congeladas** do anel de `v`.
    ///
    /// Ler o `pre` e não o vivo é o que torna o Smooth idempotente: um traço que
    /// passa duas vezes no mesmo lugar suaviza uma vez, e a superfície não
    /// derrete enquanto o artista segura o botão parado.
    /// A média do anel — o alvo do Smooth, e o **oposto** do Sharpen.
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
    fn neighbour_average(&self, mesh: &Mesh, v: u32, at: [f32; 3]) -> [f32; 3] {
        ph2d_mesh::ring_average(mesh.adjacency(), v, at, |nb| mesh.positions()[nb as usize])
    }
}

fn add(p: [f32; 3], dir: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + dir[0] * k, p[1] + dir[1] * k, p[2] + dir[2] * k]
}

fn add_vec(p: [f32; 3], v: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + v[0] * k, p[1] + v[1] * k, p[2] + v[2] * k]
}

/// `p` girado de `angle` radianos em torno da reta que passa por `pivot` na
/// direção `axis` — **Rodrigues**, e o eixo tem de chegar UNITÁRIO
/// ([`Dab::turning`] é quem garante).
///
/// ⚠️ **Dois transcendentais por vértice por dab**, e é o único ponto deste
/// módulo que os paga (o falloff é polinomial e a raiz é instrução de hardware).
/// Um `sin_cos` é UMA chamada para os dois — pedir `sin` e `cos` separados
/// dobraria a redução de argumento. O preço está medido na sonda
/// `measure_brush_kernel`; ele não é aproximável por tabela, porque quantizar o
/// ângulo faria a torção sair em degraus e interpolar entre nós de uma tabela
/// devolveria uma rotação de norma menor que 1 — o encolhimento, de novo, por
/// outra porta.
fn rotate_about(p: [f32; 3], pivot: [f32; 3], axis: [f32; 3], angle: f32) -> [f32; 3] {
    let (s, c) = angle.sin_cos();
    let v = [p[0] - pivot[0], p[1] - pivot[1], p[2] - pivot[2]];
    let dot = axis[0] * v[0] + axis[1] * v[1] + axis[2] * v[2];
    let cross = [
        axis[1] * v[2] - axis[2] * v[1],
        axis[2] * v[0] - axis[0] * v[2],
        axis[0] * v[1] - axis[1] * v[0],
    ];
    let k = dot * (1.0 - c);
    [
        pivot[0] + v[0] * c + cross[0] * s + axis[0] * k,
        pivot[1] + v[1] * c + cross[1] * s + axis[1] * k,
        pivot[2] + v[2] * c + cross[2] * s + axis[2] * k,
    ]
}

/// O passo em direção ao plano: `p − n · (d · w)`.
///
/// ⚠️ **Porta única dos QUATRO verbos de plano** (`Flatten`, `Fill`, `Scrape`,
/// `Clay`), e é o que os mantém a mesma lei com gates diferentes. Escrita quatro
/// vezes, o dia em que um deles ganhasse um caso especial seria o dia em que os
/// outros três parariam de concordar com ele sem ninguém notar.
fn to_plane(p: [f32; 3], normal: [f32; 3], d: f32, w: f32) -> [f32; 3] {
    add(p, normal, -d * w)
}

fn signed_distance(p: [f32; 3], plane: &PlaneFit) -> f32 {
    (p[0] - plane.point[0]) * plane.normal[0]
        + (p[1] - plane.point[1]) * plane.normal[1]
        + (p[2] - plane.point[2]) * plane.normal[2]
}

/// A parte de `centro − p` que corre ao longo da superfície.
///
/// ⚠️ **Divergência deliberada do Blender**, que faz o Pinch mover para o centro
/// em 3D (`co += (center − co) * fade`). Aquele vetor tem componente ao longo da
/// normal, então o Pinch dele também ACHATA um pouco — dois efeitos num knob. Ao
/// remover a componente normal, apertar é apertar; quem quer achatar tem quatro
/// verbos para isso.
fn tangential(p: [f32; 3], center: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let d = [center[0] - p[0], center[1] - p[1], center[2] - p[2]];
    let along = d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2];
    [
        d[0] - normal[0] * along,
        d[1] - normal[1] * along,
        d[2] - normal[2] * along,
    ]
}
