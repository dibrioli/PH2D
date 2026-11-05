# E11: a lei `Pigmento` na placa (05/11)

Plano: [F4_2 §6](../Arquitetura/F4_2_plano_wet_paint.md). Portão: `crates/ph2d-wet-gpu/tests/tinta.rs`,
`paridade_da_tinta_por_placa`. A régua é por placa ([memória 24/10](../../../project-memory/project_same_bit_law_is_per_device_2026_10_24.md)).

## O que a placa faz com a lei nova

- **Na placa:** o gather das somas com a massa (`t_transporte`, `t_sedimenta`, `t_bordas`), o meio pelo volume molhado, Bingham e
  mobilidade no campo (`campo_oleo`), a secagem pelas somas com retenção e evaporação do meio (`seca_tinta`), o espelho sRGB, o decaimento
  da abertura (`t_cura`). As passadas são módulos à parte (`advect_tinta.wgsl`, `seca_tinta.wgsl`): com a lei velha nada disto se aloca.
- **Na CPU:** o FECHO da cura (a placa conta quem o pede; o passo desce, fecha, sobe), a composição do ecrã. `r_g`, `r_g0`,
  `altura_fechada` e a aparência da célula (`suave` incluído) não sobem: nada na placa os escreve.
- **`placa_sabe` recusa, com a lei nova:** Pigment mix (lado espectral), chão como força, `Knob::Dissolve`, `Knob::EdgeSoftMass`, Real Oil na
  mão (`p.oleo`), tinta opaca. O motor fica na CPU e não diverge.
- FMA: o ecrã não migrou, logo `camada_banda` (a única conta que o F4-1 pedia sem contracção) não corre na placa; o que corre é linear mais
  `pow`/`exp` na retenção, na mobilidade e na cura. Determinismo medido abaixo, em vez de lutar contra o compilador.

## A medição (320×200, 120 passos, `--profile smoke`, 05/11, depois da auditoria)

`cor` = níveis sRGB 0–255 sobre branco, placa × CPU, **nas células com tinta à vista** (a cor a mais de 1/2 nível do branco num dos lados), já não
nas 64 000 da folha. `frac` = fracção dessas células acima de 0,5 nível. `controlo` = CPU × CPU a ±1 ulp de partida.

| cena | tinta | Radeon 610M: máx / p99.9 / frac | RTX 5060 Ti: máx / p99.9 / frac | controlo: máx / p99.9 |
|---|---|---|---|---|
| água | 33 221 | 1,15 / 0,0016 / 1,5e-4 | 0,002 / 0,0012 / 0 | 0,002 / 0,0010 |
| óleo diluído | 32 171 | 0,003 / 0,0019 / 0 | 6,18 / 0,0262 / 1,2e-4 | 0,222 / 0,0021 |
| pasta alta (`Dilution` 0,2, `Body` 0,5) | 27 622 | 21,6 / 0,321 / 8,0e-4 | 21,6 / 0,321 / 8,0e-4 | 0,002 / 0,0009 |
| óleo meio curado (`o` = 0,5) | 32 171 | 0,003 / 0,0019 / 0 | 6,18 / 0,0262 / 1,2e-4 | 0,222 / 0,0021 |
| óleo quase fechado (`o` = 0,02, `Rewet` 8, muito assente) | 32 196 | 0,057 / 0,0018 / 0 | 0,057 / 0,0143 / 0 | 0,010 / 0,0006 |
| água a sedimentar (contra o ramo mais perto, ver abaixo) | 33 302 | 0,004 / 0,0012 / 0 | 0,002 / 0,0012 / 0 | **15,9 / 1,48** |

A devolução (+12 passos na CPU) fica nos mesmos limites: pior p99.9 0,386 e frac 8,3e-4 (a pasta), pior máx 21,6. Bit entre corridas: 0 nas 6 cenas e nas duas
placas. Ciclo do worker × descida inteira, ao bit e fora as paredes: 0. Massa e volume, relativo: ≤ 7,5e-7.

Pior p99.9 por campo, nas duas placas, e o teto (≈ 5×):

| campo | pior | teto | | campo | pior | teto |
|---|---|---|---|---|---|---|
| film | 4,9e-5 | 2,5e-4 | | meio | 5,4e-7 | 3e-6 |
| susp | 2,35e-2 | 0,5 | | abertura | 6,2e-6 (máx 3,3e-4) | 3e-5 (máx 5,6e-4 = 2 curas) |
| sett | 9,5e-2 | 0,5 | | espelho sRGB | 6,2e-3 | 0,03 |
| somas (`filme.*`, `aberto.*`) | 6,9e-5 | 3,5e-4 | | massa relativa | 7,5e-7 | 1e-5 |

**Os tetos da cor, agora duros:** p99.9 ≤ 0,5 (o do degrau da lei velha; pior 0,386) · máx ≤ 32 (pior 21,6) · frac ≤ 0,1 % (pior 8,3e-4). ⛔ O controlo já não
sobe limite nenhum: `max(0,5; 1,5 × controlo)` era a porta por onde a sedimentar passava com 1,48.

**A sedimentar bifurca, e a lei é que se parte.** A CPU perturbada a ±1 ulp nas duas alternâncias (A e B) diverge entre si (p99.9 1,48; 15,9 níveis em 75 células), e a nominal cai no ramo B (p99.9 0,0010).
As duas placas caem no ramo A, a 0,004 dele. Não é placa: um arredondamento escolhe o ramo. O portão, quando a própria CPU se parte (`p99.9 > 0,5`), mede a placa
contra o ramo mais perto e pelos MESMOS limites, para os campos também. ⚠️ Achado aberto: uma lei em que 1 ulp move 15,9 níveis em 75 células.

**A abertura só onde há aberto (`v > 1e-6`).** Abaixo disso o denominador media ruído: em `v ~ 1e-43` a placa zera o subnormal e o `o` fica como estava, e a CPU reabre-o pelo volume que lá chegou
(Δ até 0,98 no «quase fechado», sem efeito nenhum). Acima, Δ máx 3,3e-4 (a pasta, na franja de `v ~ 1e-5`).

**As paredes, fora do bit do ciclo.** `bordas` despacha a folha inteira e escreve nas duas colunas de cada parede `wet = 1` e `vel = ±max_v`; a descida suja não as traz fora da caixa.
É condição de parede derivada, reescrita a cada passo dos dois lados, e sem água naquelas linhas para a ler. O `corpo` entra no bit embora ninguém o escreva no passo (só o pouso).

⚠️ A pasta passa, mas o controlo a 1 ulp não a explica (0,002 contra 21,6): é a mesma pior célula nas duas placas, logo aritmética, e a tabela da franja abaixo é a pista. Folga pequena
na fracção (8,3e-4 de 1e-3).

## A cura fora da caixa (D2)

`t_cura` escreve a abertura na folha inteira, e `desce_suja` só trazia a caixa: a CPU ficava com o `o` velho e o `sobe_sujo` seguinte, de um traço noutra coluna, devolvia-o à placa
(200 passos, faixa seca: CPU 0,998612 · placa 1,000000). Portão: `a_cura_fora_da_faixa_suja_desce_e_um_traco_nao_a_apaga`, agora Δ 0.

**A cura:** a passada anota as linhas curadas em `fecham[1..=2]`, lidas no mesmo `read` da contagem do fecho, e a descida suja seguinte traz SÓ a abertura das linhas fora das faixas
dela, e refaz o hash só dessas.

| 4096×2560, meia folha com óleo seco fora da caixa, RTX, 3 × 3 corridas intercaladas | sem | com |
|---|---|---|
| `t_cura`, 1× por 40 passos | 0,707 ms | 0,76 ms |
| a descida depois da cura, 1× por 40 passos | 52–54 ms | 87–92 ms (+37 ms) |

A água não tem meio, não cura e não paga nada; o óleo com a caixa por cima também não (a abertura já desce com ela). Radeon: 45 → 80 ms.

⛔ Recusas: descer a grade inteira no quadro da cura (~100 B por célula contra 4, 1 GB a 4K) · `sobe` não escrever a abertura fora do sujo (a CPU continuava com o `o` velho, e um traço
que lhe muda o `o` tem de subir) · CPU e placa partilharem o relógio da cura (dois caminhos para a mesma conta, e a CPU só teria o `m` velho das linhas sujas).

## A franja: por que o máximo não é 1

A máscara `active` pergunta `film > 0`, e o passo a passo mostra a 1.ª divergência na máscara (5 células em `n = 12`), com film e massa
iguais a 8 algarismos. A CPU sozinha, zerando film e susp abaixo de ε a cada passo, contra a CPU sem isso:

| ε | água | óleo diluído | pasta alta | a sedimentar |
|---|---|---|---|---|
| 1e-38 | 0,00 (0) | 0,00 (0) | 13,2 (3) | 0,00 (0) |
| 1e-30 | 0,00 (0) | 1,37 (3) | 13,2 (15) | 0,00 (0) |
| 1e-20 | 14,1 (73) | 23,8 (14) | 112 (119) | 18,7 (92) |

(cor máx, em níveis; entre parêntesis as células acima de 0,5.) Isto é a sensibilidade da LEI a um resíduo que desaparece, e existia antes da placa. ⚠️ Medido só
como controlo: a mão de um resíduo de 1e-38 decidir um escoamento é um achado aberto no [BUGS #63](../BUGS_painter.md), não curado aqui.

## Custo (RTX 5060 Ti, 4096×2560, água, intercalado no mesmo processo, mínimo de 8 blocos de 6 passos)

Duas corridas: lei velha 5,74 e 5,69 ms/passo · `Pigmento` 9,21 e 9,23 ms/passo, com `load` ~60–120 na máquina (só o carimbo da placa é fiável sob esta carga; o relógio é o da espera).

## Mutação (18, shaders e descida; Radeon)

As 18 sangram. Duas precisaram de cura da régua: «re-molhar ignora a abertura» sobreviveu até a cena «óleo quase fechado» pôr muito assente debaixo da água; «ninguém pede o fecho» sangrou de início por invalidar o layout (removia o último uso do buffer), e refeita com `atomicAdd(.., 0u)` sangra pela lei (`fechadas CPU 800 placa 0`).

Depois da auditoria (14, Radeon; `*` também RTX), todas sangram: a descida não traz o curado* · a passada não anota a 1.ª linha · nem a última · o hash do curado não se actualiza (`a CPU não mudou nada e subiu`) · o passo ignora a faixa · `placa_sabe` recusa Pigmento* · recusa a pasta · aceita `Dissolve` · a descida suja sem as somas · a sedimenta das somas fora do plano · a sedimentação 2 % maior* (a que o controlo escondia) · re-molhar ignora a abertura · a cura 2× mais depressa · `t_bordas` fora do plano.
