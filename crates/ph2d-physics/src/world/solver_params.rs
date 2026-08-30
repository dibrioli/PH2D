//! **Os números com que o solver corre, e por que cada um é o que é.**
//!
//! ⚠️ Este módulo nasceu do teto de 700 LOC do `world.rs` — pela **quinta** vez nesta jornada, e
//! sempre pelo mesmo motivo: as tabelas de medição e as recusas escritas ao lado de cada constante.
//! ⭐ Elas são o valor deste ficheiro, não o seu peso: *uma constante sem a tabela ao lado é um
//! palpite à espera de um smoke*, e a subida `rapier2d` 0.28 → 0.35 cobrou isso três vezes num dia.
//!
//! O corte é por **responsabilidade**, nunca por linhas: aqui vive o que a `rapier` faz, e no
//! [`super`] vive quem o usa.

use crate::PhysicsWorld;
use rapier2d::dynamics::IntegrationParameters;

/// Os parâmetros de integração com que um [`PhysicsWorld`] nasce.
#[must_use]
pub(super) fn integration_parameters() -> IntegrationParameters {
    IntegrationParameters {
        dt: PhysicsWorld::DEFAULT_DT / PhysicsWorld::DEFAULT_SUBSTEPS as f32,
        // ⚠️ **O `damping_ratio` tem de vir escrito, e não do `..default()`.** Os dois
        // coeficientes viraram UM struct na rapier 0.31, então preencher só a frequência
        // não é possível — e o `5.0` aqui é exactamente o que o default da 0.28 dava, que
        // é o número que o doc acima diz ter sido MEDIDO (subir para 20 esticou a
        // recuperação de 9 para 30 quadros). *Uma constante que era implícita passou a ser
        // nossa; escrevê-la é o que impede o dia em que o upstream a muda por baixo.*
        contact_softness: rapier2d::dynamics::SpringCoefficients {
            natural_frequency: PhysicsWorld::DEFAULT_CONTACT_HZ,
            damping_ratio: 5.0,
        },
        // ⛔ **Um padrão da rapier mudou na 0.31 e ele MUDA O TATO: as iterações
        // internas de estabilização caíram de `2` para `1`.** Fica fixado no valor de
        // sempre, pelo critério da casa: *um padrão que muda o TATO é conteúdo autorado
        // — há cenas ajustadas contra ele —, enquanto um que muda a ARQUITECTURA do
        // solver se aceita, porque não há como o fixar.* Este é uma constante, e fixá-lo
        // custa uma linha.
        //
        // ⭐ **O que elas compram está MEDIDO, e não é o que se supõe.** A leitura
        // óbvia é *«mais estabilização = pilha em repouso mais quieta»*. O que o gate
        // `the_same_gesture_hangs_only_the_one_with_a_reach` mostra é outra coisa: ele
        // põe um personagem contra uma parede **sem beirada ao alcance**, e ele tem de
        // **escorregar**. Com `1` iteração ele deixou de escorregar de todo
        // (`1,727 → 1,750` em dois segundos, contra os `0,15 m` que a barra mede); com
        // `2` o escorregar volta e o gate fica verde. ⇒ estas iterações são
        // **load-bearing para o ATRITO**, não só para o tremor de uma pilha.
        // *Uma parede que segura quem não se agarrou não é economia de cálculo: é outra
        // regra de jogo.*
        //
        // ⛔⛔ **E uma hipótese REFUTADA, registada para não voltar:** a 0.29 introduziu
        // um `friction_model` configurável cujo padrão (`Simplified`) resolve uma
        // restrição de atrito por grupo de **4 contatos** em vez de uma por contacto — o
        // candidato óbvio para explicar a mudança no escorregar contra a parede. **Ele é
        // `#[cfg(feature = "dim3")]`**: não existe em 2D, e nunca nos alcançou. *O
        // compilador matou a hipótese antes da medição — e é por isso que se escreve o
        // palpite em código em vez de o assumir.*
        num_internal_stabilization_iterations: 2,
        // ⛔⛔ **SETE constantes da 0.35 mudam o TATO, e ficam fixadas no valor de sempre.**
        // O critério é o mesmo do bloco: *um padrão que muda o tato é conteúdo autorado — há
        // cenas ajustadas contra ele —, e um que muda a ARQUITECTURA do solver aceita-se,
        // porque não há como o fixar.* Estas são constantes; fixá-las custa uma linha cada.
        //
        // ⛔⛔⛔ **A `rapier` 0.35 DOBROU o `damping_ratio` dela (5 → 10) e nós ficamos no 5 —
        // por MEDIÇÃO, contra a recomendação escrita deles.** O doc do upstream justifica o 10
        // com *«softer contacts settle deeper under load … wedging and creeping instead of
        // resting»*, que é exactamente o sintoma que o smoke de 2026-08-29 mostrou. A auditoria
        // marcou-o SUSPEITO. Medido na cena de smoke `=4` (12 corpos, 15 s):
        //
        // | afinação | pior ângulo em repouso | afundamento máx |
        // |---|---|---|
        // | ⭐ **a nossa: `120 Hz` / `ζ 5`** | **`0,04455°`** | **`0,000264`** |
        // | só o `ζ` deles: `120 Hz` / `ζ 10` | ⛔ **`24,28°`** | `0,000201` |
        // | o par INTEIRO deles: `30 Hz` / `ζ 10` | `0,27936°` | ⛔ `0,003780` |
        //
        // ⭐⭐ **A frequência e o amortecimento são UMA MOLA, e adoptar metade de um par de
        // afinação mistura duas afinações.** Com o nosso `f = 120` (4× o deles), o `ζ = 10`
        // deles dá `24°` — **545× pior** — e ainda parte o gate
        // `a_landing_body_is_never_visibly_inside_the_floor_for_more_than_a_frame`. Com o par
        // inteiro deles a pilha assenta, mas afunda **14×** mais que a nossa.
        // ⇒ *a nossa afinação ganha nos DOIS eixos; a recomendação deles é sobre a mola deles.*
        //
        // ⚠️ **Reconferir se — e só se — o `DEFAULT_CONTACT_HZ` mudar.** A recusa é sobre o
        // PAR; mexer num lado re-abre a pergunta do outro.
        //
        // ⚠️ **`static_contact_softness` é um CONCEITO NOVO, e é o mais perigoso da lista.**
        // Até aqui **todo** contacto usava os nossos `DEFAULT_CONTACT_HZ`. A 0.35 partiu-os em
        // dois grupos: dinâmico↔dinâmico (`contact_softness`) e **contra corpos fixos**
        // (`static_contact_softness`, cujo padrão é 60 Hz / 10). O chão e as paredes são
        // fixos ⇒ ajustar só o primeiro deixaria **o contacto que mais importa** a metade da
        // rigidez que o slider «Contact Hz» do painel promete. *Uma lei escrita num sítio
        // quando o modelo tem dois não é uma lei.*
        static_contact_softness: rapier2d::dynamics::SpringCoefficients {
            natural_frequency: PhysicsWorld::DEFAULT_CONTACT_HZ,
            damping_ratio: 5.0,
        },
        // 0.001 → 0.005: os corpos afundariam **5×** mais uns nos outros em repouso.
        normalized_allowed_linear_error: 0.001,
        // 10 → 3: uma sobreposição funda sairia **3,3× mais devagar**.
        normalized_max_corrective_velocity: 10.0,
        // 0,002 → 0,02: contatos detectados **10× mais cedo**; corpos parecendo flutuar um
        // fio acima da superfície.
        normalized_prediction_distance: 0.002,
        // ⚠️ **Sem tecto → 400 u/s.** A 0.35 passou a limitar a velocidade linear. Em queda
        // livre a `9,81` isso levaria ~41 s a morder, mas o `blast::explode` divide impulso
        // por massa: um corpo leve com uma explosão forte era ilimitado e passaria a bater no
        // tecto **sem aviso**. `Real::MAX` desarma-o, que é o que sempre valeu aqui.
        normalized_max_linear_velocity: rapier2d::math::Real::MAX,
        // Duas reduções de trabalho no narrow phase que a 0.35 liga por omissão.
        //
        // ⛔ **A afirmação original — «elas mudam a trajectória e portanto o hash
        // `physics_ecs_c9`» — é FALSA para o agrupamento, e a fonte diz-o.** Em
        // `narrow_phase/pair_update.rs:350-357` da rapier 0.35, sob `not(dim3)` o `use_clusters`
        // é `false` **por construção** e o parâmetro é explicitamente descartado
        // (`let _ = contact_clustering;`), com o comentário deles: *«contact clustering isn't
        // implemented in 2D»*. ⇒ escrevê-lo é **no-op estrito** nesta crate, e desligá-lo não
        // custa nem compra nada. Fica escrito para que a linha não se leia como uma decisão.
        contact_clustering: false,
        // O reciclar é real em 2D: ele troca **custo por frescura** dos dados de contacto, não
        // afina resposta. Fica desligado porque o custo que ele poupa não foi medido.
        contact_recycling: false,
        // ⛔⛔ **DOIS campos novos ficam no padrão da 0.35, e a recusa é MEDIDA.**
        //
        // `friction_in_bias_pass: false` é a regra nova *«sem atrito ao aplicar o viés»*, e o
        // doc da rapier diz o que ela compra: *«load-bearing for tall stacks: friction
        // reacting to bias velocities pumps their coherent lean mode until they topple»*.
        // ⚠️ Ligá-lo a `true` **cura um gate** desta crate (`compound::a_part_moved_while_the
        // _clock_runs_takes_its_collider_along`) — e o gate é sobre uma peça composta a levar
        // o collider, não sobre atrito. ⇒ *seria comprar de volta um defeito que eles
        // corrigiram, para curar um sintoma que não é o dele.* Bissectado: dos dois campos,
        // este é o único que muda alguma coisa aqui.
        //
        // `warmstart_joints: false` fica pela razão oposta — medido, ligá-lo **não muda um
        // único gate**. Ele é capacidade nova (*«melhora a convergência de conjuntos de
        // juntas rígidas»*), e ligar o que não se mediu a precisar não é afinação, é ruído.
        //
        // ⚠️ *A régua desta decisão não é «o teste fica verde»: é «o que a plataforma
        // corrigiu de propósito, aceita-se».*
        ..IntegrationParameters::default()
    }
}
