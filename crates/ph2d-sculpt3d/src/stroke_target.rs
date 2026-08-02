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
    point: [f32; 3],
    normal: [f32; 3],
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
        let inv_r = 1.0 / dab.radius;
        let mut acc_p = [0.0f64; 3];
        let mut acc_n = [0.0f64; 3];
        let mut sum = 0.0f64;
        for &v in &self.footprint {
            let s = self.slot[v as usize] as usize;
            let p = self.base_pos[s];
            let n = self.base_nrm[s];
            if front_only && n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2] > 0.0 {
                continue;
            }
            let d = [
                p[0] - dab.center[0],
                p[1] - dab.center[1],
                p[2] - dab.center[2],
            ];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            // A ponderação é só o FALLOFF: o plano descreve a superfície sob o
            // pincel, e força/pressão/máscara dizem o quanto agir sobre ela, não
            // que forma ela tem.
            let w = f64::from(brush.falloff.weight(dist * inv_r));
            if w <= 0.0 {
                continue;
            }
            sum += w;
            for k in 0..3 {
                acc_p[k] += f64::from(p[k]) * w;
                acc_n[k] += f64::from(n[k]) * w;
            }
        }
        if sum <= 0.0 {
            // Pegada inteira na borda do falloff (Sharper com raio grande, por
            // exemplo) — ou, com o filtro ligado, nenhum vértice frontal. Quem
            // chama decide o que fazer com o `None`.
            return None;
        }
        let inv = 1.0 / sum;
        let mut point = [0.0f32; 3];
        let mut normal = [0.0f32; 3];
        for k in 0..3 {
            point[k] = (acc_p[k] * inv) as f32;
            normal[k] = (acc_n[k] * inv) as f32;
        }
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if len > 1e-12 {
            for n in &mut normal {
                *n /= len;
            }
        } else {
            // Normais que se cancelam (uma dobra fechada sob o pincel): sem
            // direção defensável, o plano vira o do próprio dab.
            normal = [0.0, 1.0, 0.0];
        }
        // O offset move o PLANO, não os vértices — é o knob que faz do Flatten
        // um Clay sem um segundo verbo.
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
        // até um ulp**. Só o `Verb::SnakeHook` o lê — os outros treze compõem o
        // alvo sem peso, e é o `accum` que os atenua.
        w: f32,
        v: u32,
        s: usize,
    ) -> [f32; 3] {
        let base = self.base_pos[s];
        let n_area = plane.normal;
        match brush.verb {
            Verb::Draw => add(base, n_area, reach),
            Verb::Inflate => add(base, self.base_nrm[s], reach),
            Verb::Smooth => self.neighbour_average(mesh, v, base),
            Verb::Sharpen => {
                let avg = self.neighbour_average(mesh, v, base);
                // Reflete a média através do próprio vértice: o oposto exato do
                // Smooth, com a mesma magnitude.
                [
                    base[0] * 2.0 - avg[0],
                    base[1] * 2.0 - avg[1],
                    base[2] * 2.0 - avg[2],
                ]
            }
            Verb::Flatten => project(base, plane),
            Verb::Fill => {
                if signed_distance(base, plane) < 0.0 {
                    project(base, plane)
                } else {
                    base
                }
            }
            Verb::Scrape => {
                if signed_distance(base, plane) > 0.0 {
                    project(base, plane)
                } else {
                    base
                }
            }
            // Achata E acrescenta: o barro que se adiciona, sem uma constante
            // escondida — o `reach` é o mesmo knob de todo verbo aditivo.
            Verb::Clay => add(project(base, plane), n_area, reach),
            Verb::Pinch => add_vec(base, tangential(base, dab.center, n_area), 1.0),
            Verb::Magnify => add_vec(base, tangential(base, dab.center, n_area), -1.0),
            Verb::Crease => {
                let t = tangential(base, dab.center, n_area);
                // Aperta lateralmente E cava: o `-reach` é o que faz um vinco
                // ser um vinco. Com `invert`, `reach` já chega negativo e o
                // mesmo verbo levanta uma crista.
                //
                // ⚠️ **O `shape⁴` é o que faz o vinco ser FINO** (`Crease.js:68`,
                // `Math.pow(fallOff, 5)`). Quatro e não cinco porque o aplicador
                // já multiplica `(alvo − base)` pelo `accum`: o deslocamento
                // normal sai `shape⁵ · intensity · reach`, que é a estrutura
                // exata da referência — o expoente cai **só** no coeficiente da
                // normal, e o termo do pinch fica LINEAR.
                //
                // ⚠️ **Sem ele o Crease é um Draw invertido AO BIT** — medido em
                // `uv_sphere(256,512)`: pico 0,06000 e largura 0,536 R nos dois,
                // razão de afiação **1,000**. A metade que "cava" não era um
                // vinco, era a marca do Draw com o sinal trocado.
                //
                // ⚠️ **E a cura de UMA LINHA existe, compila e está errada:**
                // `self.accum[s].powi(4)` não toca a assinatura e fica verde,
                // porque toda fixture desta crate usa `strength: 1.0`, onde
                // `accum == shape` ao bit. Com `strength 0.5` ela entrega
                // **3,1%** do reach em vez de 50% — o expoente comeria a
                // intensidade quatro vezes a mais.
                //
                // ⚠️ **O `keep` vai DENTRO do expoente** (`Crease.js:67` roda
                // antes do `:68`): um vértice meio-mascarado leva `0,5⁵ = 3%` do
                // empurrão normal e `0,5` do pinch. A assimetria é da referência.
                add(
                    add_vec(base, t, brush.pinch),
                    n_area,
                    -reach * shape.powi(4),
                )
            }
            // O alvo de posição de um verbo de máscara é o próprio lugar: ele
            // não move geometria, e `apply_mask` é quem escreve o canal dele.
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
        }
    }

    /// A média das posições **congeladas** do anel de `v`.
    ///
    /// Ler o `pre` e não o vivo é o que torna o Smooth idempotente: um traço que
    /// passa duas vezes no mesmo lugar suaviza uma vez, e a superfície não
    /// derrete enquanto o artista segura o botão parado.
    /// A média do anel — o alvo do Smooth, e o **oposto** do Sharpen.
    ///
    /// ⚠️ **Duas regras de BORDA, e as duas existem porque uma malha aberta tem
    /// beira** (o `vertOnEdge` do `Mesh.js`):
    ///
    /// 1. **Valência ≤ 2 CONGELA.** Com dois vizinhos a média é o ponto médio da
    ///    corda entre eles, então suavizar a ponta de uma tira a escorrega para
    ///    dentro da corda — a geometria some, e é um caminho só de ida.
    /// 2. **Um vértice de borda medeia só com vizinhos TAMBÉM de borda.** Com o
    ///    anel inteiro, a média inclui os vizinhos do anel de DENTRO e a boca é
    ///    sugada para o miolo: medido no `open_tube3`, a altura cai de **2 para
    ///    1,3597** em seis passes — a peça encolhe pelas duas pontas e nada na
    ///    ferramenta diz por quê. Restrita à borda, os vizinhos estão no MESMO
    ///    anel, logo na mesma altura, e a beira alisa ao longo dela mesma.
    ///
    /// ⚠️ **Fora de uma malha aberta as duas são inertes**: numa `uv_sphere` não
    /// há vértice de borda nem valência < 3 (medido, zero de ambos). Foi por isso
    /// que a classe inteira ficou invisível até existir uma fixture que a
    /// contivesse.
    fn neighbour_average(&self, mesh: &Mesh, v: u32, base: [f32; 3]) -> [f32; 3] {
        let vi = v as usize;
        let adj = mesh.adjacency();
        let ring = adj.vert_verts.neighbours(vi);
        if adj.valence(vi) <= 2 {
            return base;
        }
        let border = adj.is_border(vi);
        let mut acc = [0.0f32; 3];
        let mut n = 0u32;
        for &nb in ring {
            // Um vértice de borda só ouve a própria borda; um interior ouve tudo.
            if border && !adj.is_border(nb as usize) {
                continue;
            }
            let p = self.base_pos_of(mesh, nb);
            for k in 0..3 {
                acc[k] += p[k];
            }
            n += 1;
        }
        // ⚠️ `< 2` e não `== 0`: um único vizinho de borda dá uma "média" que é a
        // posição dele, e o vértice saltaria para cima do vizinho.
        //
        // ⚠️ **DEFESA EM CAMADAS, e ela é inalcançável em malha manifold —
        // MEDIDO, não suposto.** A curva de borda de uma superfície manifold é um
        // LOOP FECHADO, então todo vértice nela tem exatamente dois vizinhos de
        // borda: no `open_tube3` são 12 de 12. Só entrada não-manifold (que
        // apenas o `from_obj` pode trazer, e ele não tem chamador de produção)
        // alcança este ramo, então ele fica documentado em vez de gateado —
        // fabricar uma quinta fixture para ele seria construir a classe antes do
        // consumidor.
        if n < 2 {
            return base;
        }
        let inv = 1.0 / n as f32;
        [acc[0] * inv, acc[1] * inv, acc[2] * inv]
    }
}

fn add(p: [f32; 3], dir: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + dir[0] * k, p[1] + dir[1] * k, p[2] + dir[2] * k]
}

fn add_vec(p: [f32; 3], v: [f32; 3], k: f32) -> [f32; 3] {
    [p[0] + v[0] * k, p[1] + v[1] * k, p[2] + v[2] * k]
}

fn signed_distance(p: [f32; 3], plane: &PlaneFit) -> f32 {
    (p[0] - plane.point[0]) * plane.normal[0]
        + (p[1] - plane.point[1]) * plane.normal[1]
        + (p[2] - plane.point[2]) * plane.normal[2]
}

fn project(p: [f32; 3], plane: &PlaneFit) -> [f32; 3] {
    let d = signed_distance(p, plane);
    add(p, plane.normal, -d)
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
