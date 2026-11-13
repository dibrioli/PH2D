# Gabaritos de junta — os números de fábrica, medidos (12/11/2026)

Leitor: quem afina um gabarito. Fonte: a sonda `probe_factory_numbers` em
[`joint_assembly_tests.rs`](../../crates/ph2d-app-physics/src/joint_assembly_tests.rs) (ignorada; corre-se com
`--ignored --nocapture`). Cada gabarito monta as **próprias peças de fábrica** (as formas-guia do catálogo, atrito 1,0,
densidade 1) sobre um chão estático, e a física corre 5 s (300 ticks). Mesmo resultado com as peças de cenário feitas de
objetos e com elas por conta do mundo. Código: [`crates/ph2d-joint-templates`](../../crates/ph2d-joint-templates/src/catalog.rs).

## Deslocamento em 5 s

| Gabarito | Peça | dx (m) | dy (m) | giro (rad) | Leitura |
|---|---|---|---|---|---|
| Wheel · Car | Body | +9,092 | −0,077 | 0,000 | anda 1,8 m/s (≈ 2π · 0,30 m por volta); a suspensão afunda 7,7 cm, dentro do curso de 10 cm |
| Wheel · Motorbike | Body | +9,776 | −0,048 | +0,018 | anda; o garfo inclinado (25°) não tomba a moto |
| Pin · Door | Door | 0 | 0 | 0 | nasce pendurada, em repouso |
| Pin · Seesaw | Plank | 0 | 0 | 0 | equilibrada no pino central |
| Pin · Pendulum | Bob | 0 | 0 | 0 | nasce em repouso, a prumo |
| Pin · Flipper | Paddle | −0,062 | −0,245 | −0,500 | o servo leva a palheta ao repouso de −0,5 rad e segura |
| Rope · Chain | Links 1–6 | 0 | 0 | 0 | pendurada esticada |
| Rope · Bridge | Plank 4/5 | 0 | −0,083 | ±0,014 | flecha de 8,3 cm no meio com 2 cm de folga entre tábuas |
| Rope · Hanging | Load | 0 | 0 | 0 | pendurada |
| Slider · Elevator | Platform | 0 | +3,000 | 0 | sobe um andar e para no batente |
| Slider · Piston | Rod | +0,800 | 0 | 0 | faz o curso inteiro |
| Slider · Sliding door | Door | 0 | 0 | 0 | sem motor: parada até alguém empurrar |
| Spring · Trampoline | Bed | 0 | −0,003 | 0 | puxada pelos dois lados, assenta 3 mm |
| Spring · Soft platform | Platform | 0 | −0,008 | 0 | assenta 8 mm |
| Weld · Glue / Breakable | Part A/B | 0 | 0 | 0 | coladas no chão |
| Rod · Rigid pendulum | Bob | 0 | 0 | 0 | a prumo |
| Pulley · Lift | Load / Counterweight | 0 | 0 | 0 | pesos iguais, em equilíbrio |

## Provas que mordem (gates)

| Prova | O que reprova |
|---|---|
| `the_car_drives_forward_on_its_suspension` | carro anda < 3 m em 5 s, chassi > 10 cm fora da altura, ou tomba. Mutação `SPIN = 0` ⇒ vermelho (12/11). |
| `the_door_swings_and_stops_at_its_limit` | porta empurrada a 12 rad/s passa de π/2 + 0,1 |
| `the_bridge_holds_itself_and_a_load` | com 10 kg no meio, o tabuleiro desce abaixo de 0,5 m ou a caixa atravessa |
| `the_elevator_rises_one_storey_and_stops` | sobe ≠ 3,0 ± 0,1 m ou sai do trilho |
| `every_template_assembles_its_factory_parts_and_holds` | qualquer gabarito: número de juntas errado, peça em NaN, abaixo do chão ou a mais de 30 m |

## A ponte sob carga (smoke do dono, 12/11)

O dono viu a ponte esmagar mais quanto MENOR a densidade. Sondas `#[ignore]` em
`crates/ph2d-app-physics/src/physics_smoke_joint_templates_tests.rs` (`probe_*`), cena 123, caixa 0,5 m, 300 ticks.
`Plank 5` em repouso a `y = 1,00`; o topo do chão é `y = 0`.

**A densidade está certa** — pelo Inspector (`apply_physics_edit`, a caixa selecionada, a forma filha recebe):

| caixa (densidade) | Plank 5, fim | mais baixo |
|---|---|---|
| 0,1 | +0,915 | +0,872 |
| 1 | +0,903 | +0,797 |
| 10 | +0,905 | +0,382 |
| 40 (10 kg) | +0,750 | +0,046 |
| 400 | +0,027 | −0,037 |

**O que esmaga é a RAZÃO de massas** (caixa 10 kg; tábua 0,05 m² × densidade). Pousada sem queda dá o mesmo:

| tábuas | Rope, fim · mais baixo | pousada, fim | Pin entre as tábuas, fim · mais baixo |
|---|---|---|---|
| 0,1 (2000:1) | +0,027 · −0,036 | +0,026 | +0,095 · +0,013 |
| 1 (200:1, o padrão) | +0,750 · +0,046 | +0,568 | **+0,978** · +0,308 |
| 10 (20:1) | +0,842 · +0,619 | +0,840 | +1,033 · +0,653 |
| 40 (5:1) | +0,897 · +0,797 | +0,889 | — |

Solver, tábuas 1: `sub 4 it 4` +0,750 · `8/4` +0,771 · `12/4` +0,950 · `4/16` +1,045 · `12/16` +0,968 (mais baixo +0,675).
Tábuas 0,1: nenhuma combinação segura (o melhor, `12/16`, volta a +0,885 mas passa por +0,023).

Leitura: o `Rope` (só um limite de distância) cede sob 200:1; o `Pin` entre tábuas segura o peso parado (2 cm) mas
ainda afunda no impacto; 2000:1 nenhum ajuste de junta ou de solver segura — é o limite do solver iterativo, cuja
cura é resolver a corrente inteira de uma vez (multibody, coordenadas reduzidas).

### A cura (W2): a corrente resolvida a partir de cada poste

Sonda `crates/ph2d-physics/src/world/bridge_chain_probe.rs` (`sonda_da_ponte`, rapier puro, mesma geometria,
caixa 10 kg caindo de 3 m, **600 ticks**; caixa pousada = 1,30). «Tree» = uma árvore multibody de um poste ao
outro, fechada por uma junta de impulso; «TwoTrees» = uma árvore por poste, encontrando-se no meio por UM Pin de
impulso; «Product» = `spawn_joint` com Pins `one_chain` (`world/chains.rs`), que cresce em largura de TODO corpo fixo.

| tábuas · sub-passos | Rope (hoje) caixa · mais baixo | Pin de impulso | TreePin | TwoTrees = **Product** |
|---|---|---|---|---|
| 0,1 · 4 | 0,306 · −0,034 | 0,808 · −0,007 | explode (NaN) | 1,207 · +0,267 (Product 1,210 · +0,526) |
| **1 · 4 (o padrão)** | 0,944 · +0,013 | 1,017 · +0,312, caixa escorrega a x −0,63 | NaN | **1,283 · +0,740** |
| 10 · 4 | 1,156 · +0,616 | 1,149 · +0,653 | 1,297 · +0,908 | 1,296 · +0,949 |
| 1 · 8 | 1,041 · +0,071 | 1,093 · +0,535 | 1,294 · +0,854 | 1,297 · +0,900 |

TreeRope (árvore + Rope de impulso no fecho): a ponte desaba (caixa 0,35) em todas as linhas. O Product reproduz o
TwoTrees bit a bit em 5 das 6 linhas; a 2000:1 · 4 o encontro cai noutra emenda (o caso é caótico) e fica melhor.

Na cena 123 (ponte real, `probe_plank_density_against_bridge_sag`, 300 ticks): tábuas 1 → caixa **1,288**, tábua do
meio no fim 0,992, mais baixo 0,723 (era 0,750 · 0,046); tábuas 0,1 → mais baixo 0,465 (era −0,036).

**Escolhido: o gabarito Bridge passa a Pins `one_chain` no meio de cada folga.** Gates: `ph2d-physics one_chain::`
(segura 200:1 contra o controlo de Pins sem corrente, rebobina bit-exato, apagar uma junta refaz as árvores, só o Pin
inquebrável e sem motor entra, a árvore nasce com as poses reais), `the_bridge_holds_itself_and_a_load` (tábua do
meio > 0,9) e a cena (caixa > 1,2).

⛔ Recusas MEDIDAS: árvore única de poste a poste (explode a 200:1 e 2000:1 com 4 sub-passos); fecho por Rope de
impulso (desaba); mais sub-passos/iterações sem multibody (a tabela do solver acima); Rope dentro da árvore (o rapier
não tem Rope em coordenadas reduzidas).

Limites declarados: o gémeo reduzido não tem reação medida (o Inspector não lê carga de uma junta na corrente); refazer a
árvore a meio do Play zera a velocidade dos elos; Pin com motor ou quebrável fica de impulso (não medido).

## O que esta tabela NÃO mede

- Carga: a força do elevador e da polia com peso em cima, e o torque do carro numa rampa. Só o carro vazio no plano.
- A ruptura do Weld · Breakable (o número é o `DEFAULT_BREAK_FORCE` do tipo, já medido em `measure_joint_break.rs`).
- O que acontece com peças do artista de outro tamanho: as âncoras seguem a caixa da peça, mas o curso e a força não.
