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

## O que esta tabela NÃO mede

- Carga: a força do elevador e da polia com peso em cima, e o torque do carro numa rampa. Só o carro vazio no plano.
- A ruptura do Weld · Breakable (o número é o `DEFAULT_BREAK_FORCE` do tipo, já medido em `measure_joint_break.rs`).
- O que acontece com peças do artista de outro tamanho: as âncoras seguem a caixa da peça, mas o curso e a força não.
