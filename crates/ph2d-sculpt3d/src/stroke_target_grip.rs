//! **O ALVO DOS QUATRO GESTOS COM ÂNCORA** — a família que a
//! [`Verb::anchors`] nomeia, cortada por ASSUNTO do irmão [`super::target`].
//!
//! ⚠️ **A linha de corte é uma porta que já existia, não uma inventada para
//! caber no teto de LOC:** `anchors()` responde *este verbo escolhe um ponto
//! no pen-down e só anda quando o dedo anda?*, e é exatamente esse o conjunto
//! que vive aqui. Os doze verbos que ficaram no pai são carimbos — o alvo deles
//! é função do dab, não de um gesto acumulado desde o toque.
//!
//! ⚠️ **Filho (`#[path]`) e não um módulo irmão**, pela mesma conta que o pai
//! paga: estes braços leem os planos congelados do [`SculptStroke`], e um irmão
//! obrigaria `base_pos`/`base_nrm`/`grip` a virar `pub(crate)` — a visibilidade
//! viraria função do TAMANHO do arquivo, que é o oposto do que o teto existe
//! para fazer.

use super::*;

impl SculptStroke {
    /// O alvo dos quatro verbos de âncora — ver o cabeçalho.
    ///
    /// ⚠️ **Ele recebe MENOS que o irmão do carimbo, e a lista é a medida da
    /// diferença:** nenhum destes quatro lê o `reach`, o `shape`, a normal do plano ou o índice do
    /// vértice — o alvo de um gesto com âncora é função do GESTO (`dab.pull`,
    /// `dab.center`) e da posição, nunca do perfil do dab, que entra depois pelo
    /// `accum`. Passar os três só para simetria com o pai seria três argumentos
    /// que o próximo leitor tentaria usar.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn target_gripped(
        &self,
        brush: &Brush,
        dab: &Dab,
        w: f32,
        base: [f32; 3],
        live: [f32; 3],
    ) -> [f32; 3] {
        match brush.verb {
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
            // ⚠️ **E é aqui que o `l-mode` entra**, com a lei de de Goes & James
            // 2017: o gesto deixa de ser um vetor multiplicado por um escalar e
            // passa a ser a FORÇA aplicada ao bico de um sólido elástico. O que
            // muda de visível é que o barro **à frente** do puxão acompanha mais
            // que o barro **ao lado** dele (medido `1,33×` a um ε do centro) —
            // uma curva de falloff não tem como exprimir isto, porque ela devolve
            // um escalar e um escalar não tem para onde apontar.
            //
            // ⚠️ **O `ε` É O RAIO DO PINCEL**, e até 2026-08-13 ele era o raio
            // DIVIDIDO pelo [`crate::KELVINLET_REACH`] — a inversão que fez o
            // report *"o l-mode do grab está bizarro"*. Espremendo o campo dentro
            // da pegada, a largura característica dele virava um terço do
            // círculo do cursor e o agarre saía uma AGULHA: medido no arrasto do
            // produto, o barro a meio raio acompanhava `0,03` do puxão contra os
            // `0,55` do `s-mode`, e o modo inteiro movia **um terço** do barro.
            // Hoje o raio do pincel é o `ε` — a ESCALA da resposta elástica — e é
            // a PEGADA que cresce para conter o campo ([`Brush::query_radius`]).
            Verb::Move => match brush.mode.field(Verb::Move) {
                Some(_) => {
                    let f = [dab.pull[0] * w, dab.pull[1] * w, dab.pull[2] * w];
                    let r = [
                        base[0] - dab.center[0],
                        base[1] - dab.center[1],
                        base[2] - dab.center[2],
                    ];
                    let eps = dab.radius;
                    add_vec(
                        base,
                        crate::kelvinlet::grab(r, eps, f, brush.elastic_scales),
                        1.0,
                    )
                }
                // ⚠️ **`Some(_)` e não o variante nomeado, desde que o *qual*
                // passou a ser do VERBO** ([`crate::Verb::elastic_field`]): este
                // braço só pode ser alcançado pelo campo deste verbo, então
                // nomeá-lo seria repetir num segundo sítio um fato que já tem
                // dono — e o segundo sítio é exatamente onde um par discorda.
                None => add_vec(base, dab.pull, 1.0),
            },
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
            //
            // ⚠️ **COM CAMPO ele continua partindo do vivo, e é isso que o
            // separa do [`Verb::Move`] sem lhe dar um campo próprio:** o
            // [`crate::Field::Grab`] é o MESMO, e o gancho é a âncora que anda.
            // O `r` sai da posição VIVA porque é dela que este verbo parte —
            // medi-lo do `pre` daria um perfil ancorado onde o vértice já não
            // está, e o revezamento comporia dois erros por dab.
            Verb::SnakeHook => {
                // ⚠️ **É o `live` que o pai já leu**, e não uma segunda leitura
                // de `mesh.positions()[v]`: a expressão é a mesma e o instante
                // é o mesmo, então usar a que já está na mão faz UMA leitura
                // onde havia duas — o que só APERTA o invariante que o
                // parágrafo acima descreve.
                let from = live;
                match brush.mode.field(Verb::SnakeHook) {
                    Some(_) => {
                        let f = [dab.pull[0] * w, dab.pull[1] * w, dab.pull[2] * w];
                        let r = [
                            from[0] - dab.center[0],
                            from[1] - dab.center[1],
                            from[2] - dab.center[2],
                        ];
                        add_vec(
                            from,
                            crate::kelvinlet::grab(r, dab.radius, f, brush.elastic_scales),
                            1.0,
                        )
                    }
                    _ => add_vec(from, dab.pull, w),
                }
            }
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
            //
            // ⚠️ **COM CAMPO só o ÂNGULO muda, e a geometria fica a mesma** — o
            // campo elástico entra como o ESCALAR do
            // [`crate::kelvinlet::rigid_profile`], não como deslocamento. Somar
            // `perfil · (ω × r)` à posição seria linearizar a rotação e o barro
            // **INCHARIA** com o ângulo varrido — medido a meio raio,
            // **1,0408 a meio radiano e 1,5271 a dois**, contra **1,0000** por
            // esta rota (a tabela vive no [`crate::kelvinlet::rigid_profile`]).
            // É a mesma objeção do parágrafo acima vista do outro lado: pela
            // corda a forma encolhe, pelo deslocamento ela cresce.
            // Girar `θ·w·perfil` mantém todo vértice **sobre** a própria
            // circunferência e deixa o campo decidir só quanto cada um
            // acompanha.
            Verb::Twist => {
                let turn = match brush.mode.field(Verb::Twist) {
                    Some(_) => {
                        let r = [
                            base[0] - dab.center[0],
                            base[1] - dab.center[1],
                            base[2] - dab.center[2],
                        ];
                        w * crate::kelvinlet::rigid_profile(r, dab.radius, brush.elastic_scales)
                    }
                    _ => w,
                };
                rotate_about(
                    base,
                    dab.center,
                    [-dab.eye[0], -dab.eye[1], -dab.eye[2]],
                    dab.amount * turn,
                )
            }
            // **O LOCAL SCALE** — afasta (ou aproxima) o `pre` da âncora.
            //
            // ⚠️ **O fator é clampado em ZERO, e o limite não é um palpite:** é
            // onde a operação deixa de estar definida. Um fator negativo não
            // encolhe mais — ele **reflete** a pegada através da âncora, virando
            // aquele pedaço da malha do avesso (normais para dentro, faces
            // invertidas). Colapsar na âncora é o fim honesto do gesto.
            //
            // ⚠️ **COM CAMPO é a FRAÇÃO que o perfil pesa, pelo mesmo motivo do
            // Twist:** `s·r` é radial, então escalar o raio por
            // `1 + s·perfil` é exato onde somar `perfil·(s·r)` seria a mesma
            // conta — mas só enquanto o clamp em zero não morde. Passando pelo
            // fator, o *fim honesto do gesto* que o parágrafo acima descreve
            // continua sendo o colapso na âncora, e não uma reflexão através
            // dela num vértice qualquer do meio da pegada.
            Verb::LocalScale => {
                let d = [
                    base[0] - dab.center[0],
                    base[1] - dab.center[1],
                    base[2] - dab.center[2],
                ];
                let grow = match brush.mode.field(Verb::LocalScale) {
                    Some(_) => {
                        w * crate::kelvinlet::rigid_profile(d, dab.radius, brush.elastic_scales)
                    }
                    _ => w,
                };
                let f = (1.0 + dab.amount * grow).max(0.0);
                add_vec(dab.center, d, f)
            }
            // Os doze carimbos vivem no pai.
            _ => base,
        }
    }
}
