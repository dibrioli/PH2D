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
| Pin · Bridge | Plank 4/5 | 0 | −0,083 | ±0,014 | flecha de 8,3 cm no meio com 2 cm de folga entre tábuas |
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

⚠️ **Corrigido 15/11 (`51051dde4f`):** a linha da cena 123 que aqui dizia «caixa **1,288**» lia uma caixa ATIRADA
pela emenda que voltou a cair por sorte — a ponte nunca cedeu. A emenda (a única junta de impulso entre as duas
árvores) com a rigidez de fábrica do rapier devolvia num sub-passo o erro do impacto. Medido (cena 123, 18 quedas
perturbadas, ressalto máximo da caixa · quedas que ficam na ponte):

| emenda | ressalto | ficam |
|---|---|---|
| rigidez de fábrica (1e6) | 2,0 m | 14/18 |
| mola 240 Hz | 1,04 m | — |
| mola 60 Hz | 0,30 m | — |
| **mola 15 Hz, amortecimento crítico (`SEAM_SOFTNESS`)** | **0,028 m** | **18/18** |
| emenda num poste | 1,92 m | 11/18 |

⛔ Recusa medida: todas as juntas a 60 Hz (moveu a porta). Gate `the_bridge_catches_a_dropped_crate_without_throwing_it`
(`ph2d-app-physics`, 9 quedas ±4 mm).

**Escolhido: o gabarito Bridge passa a Pins `one_chain` no meio de cada folga.** Gates: `ph2d-physics one_chain::`
(segura 200:1 contra o controlo de Pins sem corrente, rebobina bit-exato, apagar uma junta refaz as árvores, só o Pin
inquebrável e sem motor entra, a árvore nasce com as poses reais), `the_bridge_holds_itself_and_a_load` (tábua do
meio > 0,9) e a cena (caixa > 1,2).

⛔ Recusas MEDIDAS: árvore única de poste a poste (explode a 200:1 e 2000:1 com 4 sub-passos); fecho por Rope de
impulso (desaba); mais sub-passos/iterações sem multibody (a tabela do solver acima); Rope dentro da árvore (o rapier
não tem Rope em coordenadas reduzidas).

Limites declarados: o gémeo reduzido não tem reação medida (o Inspector não lê carga de uma junta na corrente); refazer a
árvore a meio do Play zera a velocidade dos elos; Pin com motor ou quebrável fica de impulso (não medido).

### A corrente em trechos e o tecto do «Planks» (16/11, smoke do dono: «Make the parts» travou)

O rapier resolve cada árvore multibody com a matriz de massa inteira: o passo custa ~n³ nos elos. Medido (perfil
smoke, esta estação, a ponte do Joint Editor por «Make the parts», um `PhysicsBridge::dispatch`):

| tábuas | numa árvore por poste: passo | 1.º passo | as mesmas juntas quebráveis (impulso) |
|---|---|---|---|
| 64 | 5,5 ms | 6,5 ms | 0,29 ms |
| 128 | 32 ms | 54 ms | 0,52 ms |
| 256 | 302 ms | 286 ms | 1,0 ms |
| 384 | 1 366 ms | 934 ms | 1,6 ms |
| 1 024 | 46 ms (já NaN) | **29 s** | — |

O campo aceitava até 65 535: um número grande parava o app. **Cura:** cada árvore cresce no máximo `SEGMENT_LINKS`
elos; a peça seguinte é a raiz livre de outra árvore, presa por uma emenda (`SEAM_SOFTNESS`). Sonda
`long_bridge(n, k)` (`ph2d-physics` `tests/it/one_chain.rs`, rapier puro, caixa 10 kg, 400 ticks):

| tábuas · K | passo (mín.) | caixa no fim | finito |
|---|---|---|---|
| 8 · qualquer K | 3,5–4,5 µs | 1,252 (todos iguais) | sim |
| 64 · 8 | 0,82 ms | 0,347 (desaba ao chão) | sim |
| 64 · 16 | 1,5 ms | 0,354 (desaba) | sim |
| **64 · 32** | **4,0 ms** | **1,039 = sem trechos** | sim |
| 64 · sem trechos | 4,1 ms | 1,039 | sim |
| 256 · 8 / 16 / **32** | 3,6 / 6,8 / **17,8 ms** | 0,347 (a ponte de 133 m pousa no chão) | sim |
| 256 · 64 / sem trechos | — | — | **não (explode)** |

**Escolhido K = 32:** até 64 tábuas a ponte é a de antes bit a bit; as longas deixam de explodir; o custo cresce em
linha reta. ⛔ Recusa medida: K = 8 e 16 (mais baratos, mas a ponte de 64 desaba — as emendas cedem).

No app, com trechos (o mesmo `dispatch`): 64 → 3,6 ms · 128 7,5 · 192 11,8 · 224 13,0 · **256 15,8** · 320 20,2 · 512 32,4
(antes 302 ms a 256). **Tecto `MAX_ROW_PARTS = 128`** (era 256 até o 2.º smoke do dono, 16/11: «simulando com 256
peças travou o app»). O recurso é o quadro INTEIRO, não só o tique: quando tique + desenho passam de 16,7 ms, o passo
fixo (`FixedStep`, `DEFAULT_MAX_SUBSTEPS = 8`) fica a dever tiques e paga 2, 4… até 8 por quadro — a espiral; o app
deixa de responder. Medido no app (perfil smoke, Xvfb, cena 123 + `PH2D_JOINT_EDITOR_SMOKE=14`, «Planks» N, Make,
Play, `PH2D_FLUID_PROFILE=1`, janela de 120 quadros):

| tábuas | quadro em Play | simulação por quadro (média · pico) | aviso `dropped … (max_substeps cap)` |
|---|---|---|---|
| — (antes do Play) | 31,6–32,1 ms (a base do Xvfb) | 0,6 · 1,2 ms | — |
| **128** | **33,7–37,2 ms** | 13,2 · 37 ms (2 tiques por quadro) | não |
| 256 | **104,7 ms (~10 fps)** | 28 · **95,7 ms** (a espiral) | sim (0,25 s) |

Com 128 o quadro fica na base e a simulação cabe com folga de 2× (7,5 ms por tique). ⛔ Recusa medida: 256 (o tique
de 15,8 ms sozinho já enche o quadro de 60 Hz; somado ao desenho, a espiral). A espiral em si é do passo fixo do app
inteiro (toda cena pesada a tem); não é desta linha.

Para onde vai a CPU em Play com 128 tábuas (`perf record -F 499`, 6 s, binário smoke com `STRIP=none`, carga ~9 na
máquina; amostras na função folha): física 47 % (`Multibody::update_mass_matrix` e o `nalgebra` dela) · os canais de
componente da cena (`ph2d_app_components::canais_componente`, `serde_json`, `BTreeMap<FieldKey>`) 16 % · alocação
10 % · o resto espalhado; o Joint Editor < 1 %. Os canais custam por objecto e por quadro no app inteiro, e também não
são desta linha. Gates: `a_long_bridge_is_cut_into_trees_and_a_short_one_is_the_old_chain`,
`planks_para_no_tecto_medido_da_fila`. Quem mudar o K ou a física da corrente reconfere esta tabela.

O travamento do PC do dono (16/11 ~15:31) **não se reproduziu** com o gesto dele (8 tábuas, Breaks On, 500 N, Make,
Play: CPU e memória planas, placa < 26 %, no Xvfb com a mesma placa). O journal só mostra o compositor pendurado
e, no mesmo minuto, três outras linhas a compilar e a correr apps. O número grande no «Planks» é o único caminho
medido que pára o app; está fechado.

## O que esta tabela NÃO mede

- Carga: a força do elevador e da polia com peso em cima, e o torque do carro numa rampa. Só o carro vazio no plano.
- A ruptura do Weld · Breakable (o número é o `DEFAULT_BREAK_FORCE` do tipo, já medido em `measure_joint_break.rs`).
- O que acontece com peças do artista de outro tamanho: as âncoras seguem a caixa da peça, mas o curso e a força não.
