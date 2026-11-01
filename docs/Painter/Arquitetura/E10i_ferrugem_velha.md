# E10i — a ferrugem da lei Velha depois da E10g (janela 6, 01/11)

Defeito: na grelha E9 (`sonda_varredura_oleo`), a lei **Velha** passou a mostrar «ferrugem» (amarelo sobre azul
vira laranja) em P600 × W0,5 (Dilution ≈ 0,127, líquido livre ≈ 3 % do dab) depois de `f8a006c60b` (E10g). Antes: 0.

## 1. O mecanismo (medido)

A E10g fez o diluente do óleo líquido: acima de Dilution 0,1 o dab pousa filme (antes `m = 1` pousava água 0 e a
célula tinha `film = 0` ⇒ `lost = 1` ⇒ tudo assentava num passe só). Agora o filme fino (0,0004–0,003) do óleo, que
seca devagar (`meio::retencao/evaporacao`), assenta em **muitos passes parciais** em `drying.rs::dry_cell`, cada um pelo
`assenta_memo_f32` → [`assenta::assenta`](../../../crates/ph2d-wet-paint/src/assenta.rs) com a lei da célula
`Ecra::Cobre { opaca: 1 }` (célula de pincel de óleo: `com_oleo` ⇒ `com_opaca`).

O `assenta` é o INVERSO do ecrã (Newton que mantém a cor vista) e não tem caixa — e a cobertura não soma na massa
(`alpha_of_mass` não é linear), logo o inverso EXTRAPOLA fora das duas tintas. Amostras da sonda (contador temporário
em `dry_cell`, removido): **8 % dos passes de assentar** (32 511 de 400 000) saem > 0,5 nível da caixa
`[min(sc,uc), max(sc,uc)]`:

| sett (sc) | susp (uc) | a_s → a_s2 · a_f → a_f2 | resultado |
|---|---|---|---|
| azul `[40,90,220]` | o MESMO azul | 0,088 → 0,099 · 0,045 → 0,035 | `[43,7, 90, 220]` (a mesma tinta muda de cor) |
| azul | o MESMO azul | 0,117 → 0,131 · 0,014 → 0 | `[42,9, 92,2, 220,5]` |
| `[57,5, 63,3, 199,8]` | amarelo `[245,205,25]` | 0,180 → 0,190 · 0,158 → 0,150 | **`[112,7, 68,5, 7,7]`** — B abaixo das duas (25), R > G > B: laranja |
| azul | amarelo | 0,035 → 0,045 · 0,045 → 0,035 | `[72,0, 108,8, 0,0]` |

Cada passe guarda o seu resto e o seguinte parte dele: o canal azul vai ao chão (0) e o cruzamento ganha matiz
quente. Com a água (`medium = 0`) o filme evapora em poucos passes ⇒ 8,31 de ferrugem; com o óleo, 52,08.

A/B no MESMO binário (`PH2D_E10I` temporário, removido), Flat Bristle P600 W0,5 pr0,2:

| interruptor | ferrugem |
|---|---|
| nenhum | 52,08 |
| caixa depois do assentar em `dry_cell` | **0,00** |
| caixa na sujidade da ponta (`trail/transfer.rs:184-188`) | 52,08 |
| `dry_gate = 0` (`painter/doors.rs:35`) | 52,08 |
| óleo assenta tudo num passe (`lost = 1` com `m > 0`) | 7,19 |

Hipóteses da sujidade da ponta e do dry gate: refutadas.

## 2. A cura

A lei OPACA promete o ponto no SEGMENTO entre as duas tintas (`Ecra::com_opaca`); o inverso dela tem de respeitar a
mesma caixa — a lei E10e2 §E3.1 (`v.max(min(d,s)).min(max(d,s))`) aplicada no sítio que vaza:

- `wp/assenta/ponto.rs` — `na_caixa_opaca(opaca, a_s, sc, uc, x)`: prende cada canal à caixa na fracção `opaca`;
  `opaca = 0` ou `a_s ≤ 0` devolve `x` ao bit.
- `wp/assenta.rs` (`assenta`, ramo `Cobre`) — o resultado do `resolve` passa pela `na_caixa_opaca`.

Fica de fora, de propósito, o tapar extra (`opaca = 0`, a lei do Rebelle/aguarela): lá a tinta clara guarda-se mais
clara que a escura POR LEI (o docstring do `assenta`: com o tecto o cruzamento seco ficava malva, `05 §4b`).

## 3. Testes

| teste | antes | depois |
|---|---|---|
| wp `assentar_a_tinta_opaca_fica_na_caixa_das_duas_tintas` (novo, os estados medidos acima) | ✗ (com a cura sabotada) | ✓ |
| wp `advect_jacobi_` 4 · `bit` 31 · `tinta` 72 · `o_invariante_soma_massa` 1 · `oleo_sobre_oleo` 5 · `flow_bit` 2 · `deposit` 14 · `assenta` 27 | ✓ | ✓ (hashes ao bit) |
| tp `remolha_tests` 3 · `controlos_e10` 6 · `pastel_seco` 2 · `opaca` 3 de 4 | ✓ | ✓ |
| tp it `familias::guache::a_guache_opaca_cobre_o_azul_seco` (`guache.rs:518`) | ✗ | ✗ igual — vermelho ANTIGO (falha na mesma linha com a cura desligada) |

## 4. As quatro piores células (P600 W0,5 n1, Velha, papel padrão)

| pincel | ferrugem antes → depois | degraus antes → depois |
|---|---|---|
| Flat Bristle pr0,2 | 52,08 → **0,00** | 0 → 0 |
| Tilted Shade pr0,2 | 50,25 → **0,00** | 0 → 0 |
| Filbert pr0,2 | 25,34 → **0,00** | 2 → 0 |
| Filbert Dry pr0,6 | 7,75 → **3,48** | 19 → 16 |

Montagem (esquerda antes, direita depois, linhas na ordem da tabela; não versionada):
`/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/b0e24821-70eb-48fe-8f81-fba2b019044a/scratchpad/e10i_antes_depois.png`.

## 5. Aberto

1. Filbert Dry pr0,6: resta 3,48 de ferrugem — outra porta (candidatas: `lift_settled`/re-molhar, o pouso do dab no
   filme); medir com a mesma régua.
2. A mesma extrapolação existe com `opaca = 0` na água (8,31 com `medium = 0` era quase toda opaca = 1 nesta célula;
   a água pura não foi medida aqui) — a caixa lá foi RECUSADA por medição (malva); se houver ferrugem na água, a cura
   é outra (o inverso exacto, ou somar a massa antes do ecrã), nunca o tecto.
3. GPU (`ph2d-wet-gpu`): o assentar da placa não foi auditado (NVIDIA proibida nesta janela).
4. `wp/assenta.rs` está em 700 linhas (o tecto): a próxima mudança lá tem de partir o ficheiro.
5. Re-correr a grelha E9 inteira para confirmar 0 de ferrugem fora das quatro células.
