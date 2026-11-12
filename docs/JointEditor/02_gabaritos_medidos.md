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
cura é resolver a corrente inteira de uma vez (multibody, coordenadas reduzidas). A abrir na W2.

## O que esta tabela NÃO mede

- Carga: a força do elevador e da polia com peso em cima, e o torque do carro numa rampa. Só o carro vazio no plano.
- A ruptura do Weld · Breakable (o número é o `DEFAULT_BREAK_FORCE` do tipo, já medido em `measure_joint_break.rs`).
- O que acontece com peças do artista de outro tamanho: as âncoras seguem a caixa da peça, mas o curso e a força não.
