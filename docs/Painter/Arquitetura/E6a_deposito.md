# E6a — o depósito no filme por somas (31/10)

Plano: `F4_2_plano_wet_paint.md` §4 (linha «depósito»), §7 E6a, R9/R13/R19 do §4.1. Com `Sim::tinta == Pigmento` a
massa que entra no `susp`/`sett` vinda do pincel entra também em `Grid::tinta[i].filme`/`.aberto` como
`Soma::de(Tinta::de_srgb(ponta/255, t), Δmassa/M_UM)`; o que a massa perde, a soma perde pela mesma fracção
(`transfere`). Invariante: `filme.v·M_UM == susp`, `aberto.v·M_UM == sett`.

## O que mudou

| sítio | o quê |
|---|---|
| `wp/grid/tinta.rs:51` | `M_UM = 1374` (massa por µm), `soma_da_massa` `:55`, `srgb_0_255` (espelho), `migra` `:76` |
| `wp/grid/tinta.rs:93 garante` | aloca MIGRANDO a tinta velha da grade (filme ← `susp`/`susp_rgb`, aberto ← `sett`/`sett_rgb`, `t` = Transparency do material da célula, senão 0) — trocar de lei a meio não apaga a tela |
| `wp/grid/tinta.rs:108 restaura_janela` | recorte de uma grade sem o campo sobre uma com ele: a janela é migrada dos planos velhos já restaurados (antes voltava a NEUTRA, quebrando o invariante); campo que nasce aqui nasce migrado |
| `wp/trail/transfer.rs:132` | `pigmento = (p.tinta == Pigmento).then(...)`: `garante` + os parâmetros do transfer |
| `wp/trail/transfer.rs:~207` | a porta da célula: `if let Some(d) = &pigmento { d.pousa(...) } else if seco { <velha intacta> } else { <velha intacta> }` |
| `wp/trail/transfer.rs:416` | o arrasto (`Drag`): `deposito_tinta::arrasta` — as somas fazem o mesmo `lerp` que a massa |
| `wp/trail/transfer/deposito_tinta.rs` (novo) | `Pigmento::pousa`, `arrasta`, `tira`, `corta_sett`, `espelha`; `pousa_seco` (lei VELHA, movida verbatim de `transfer.rs` para caber no tecto) |
| `wp/trail/deposit.rs:133,248` | a cor por célula da janela (`pousa_a_tinta`, R9): `DepositLaw::mix = None` com Pigmento ⇒ Oklab pesado pela massa (`Soma::junta`) em vez de `mix.mix` |

`Pigmento::pousa` segue a MASSA da lei velha ao bit (ponta seca → `sett`; re-molhar `Dissolve` → a fracção que saiu do
`sett` move aberto→filme; Glaze → `dm` do suspenso assenta = `transfere(filme, aberto, dm/susp)`; tecto das 3000 →
`tira` a mesma fracção; célula virgem → `filme = nova`; a água do filme igual). **Não correm** com Pigmento:
`mix.mix` + `velatura` (R9: misturar é somar), `cobre_a_outra` (`WetCover`) e `cobre_o_assentado` (`DryCover`) (R19:
a cobertura é a da óptica), `settle_composite` (só cor), `g.vela` (`vela_aqui`, composição da lei velha; morre na E5).
`susp_rgb`/`sett_rgb` passam a ESPELHO das somas (Oklab → sRGB) nas células que o depósito toca: o bico (E6b) e o
ecrã antes da E5 lêem a cor misturada; nenhum leitor da lei nova os lê.

**R13 (`if p.oleo > 0` de `transfer.rs:~153`, `espalha.primeiro_toque`/`piso`):** NÃO é da lei velha da tinta — é a
espessura do relevo do óleo (`grid::espessura`), lida pela altura/emboss sobre a massa, que continua em massa. Fica a
correr nas duas leis; morre quando a altura vier de `h = w + v/φ` (E7/E8). Registado, não tocado.
**`trail.rs:565` / `deposit.rs` `pig`/`water`:** acumulam MASSA na janela do rasto, não no `susp`; a soma só nasce no
transfer. Nada a mudar. **`wp/trail/filme.rs`** (o `StrokeFilm`, máximo por célula em unidades de depósito): é
quantidade, não cor — sem mudança.

## Calibração de `M_UM`

Alvo: `alpha_of_mass(600) = 0,69917`. Cobertura de um dab de massa 600 sobre branco, água (`m = 0`, molhado).
Duas réguas: α = o α de `ecra_sobre_branco` (o briefing); C = a cobertura DERIVADA eq. (3), média das bandas (o
plano §2.3). `M_UM` = 600 ÷ o `v` (µm) que dá o alvo. Medido por `calibracao_de_m_um`:

| t | cor (sRGB) | α: v (µm) | α: M_UM | C(3): v (µm) | C(3): M_UM |
|---|---|---|---|---|---|
| 0 | azul (20,40,200) | 0,0043 | 139 432 | 0,0294 | 20 387 |
| 0 | amarelo (240,210,20) | 0,0043 | 139 431 | 0,2012 | 2 983 |
| 0 | vermelho (200,30,30) | 0,0082 | 73 565 | 0,0253 | 23 725 |
| 0 | branco (255,255,255) | 0,4367 | **1 374** | 0,4367 | **1 374** |
| 1 | azul | 0,4303 | 1 394 | 2,9431 | 204 |
| 1 | amarelo | 0,4303 | 1 394 | 20,1151 | 30 |
| 1 | vermelho | 0,8156 | 736 | 2,5290 | 237 |
| 1 | branco | 43,6669 | 14 | 43,6669 | 14 |

O α do ecrã é `max(1 − R_w)` por banda: numa cor saturada é a banda mais ESCURA, não o esconder — por isso varia
100× entre cores. Nenhum `M_UM` único iguala a lei velha (cuja cobertura não depende da cor) para todas as tintas.
**Critério escolhido:** a tinta OPACA (`t = 0`, a de fábrica) que MENOS esconde — o branco, onde α e C coincidem —
cobre o mesmo que `alpha_of_mass(600)`; toda outra opaca cobre ≥ (nenhuma perde cobertura). ⇒ `M_UM = 1374`. O teste
`calibracao_de_m_um` prova-o (branco `C(600) = alvo ± 1e-3`, cada opaca `C(600) ≥ alvo`). Preço: as cores escuras/
saturadas opacas ficam mais opacas que na lei velha (azul a 600: α ≈ 1). Alternativa medida (média geométrica das
quatro opacas em C, ~6 700): o branco a 600 cobriria ~0,2 — rejeitada. Os portões F0 (E9) dirão se o critério morde.

## Testes (por nome, `--profile` do script)

| teste | resultado |
|---|---|
| `o_deposito_conserva_volume_e_somas` (traço Pigmento 96×64) | Σfilme·M_UM = 245 726,1983 × Σsusp = 245 726,1990 (rel. 3e-9); sett 0; cor média vs pincel ΔOklab 1,2e-7 |
| `n_passadas_monotonas_no_motor` (6 passadas, linha central) | cobertura 0,4795 → 0,4868 → 0,4874 → 0,4875 → 0,4880 → 0,4892 (não desce) |
| `trocar_de_lei_a_meio_nao_apaga_a_tela` (Velha + passo + Velha, depois `garante`) | toda célula: `v·M_UM == massa` (rel. 1e-6), cor = `susp_rgb` (ΔOklab < 1e-5) |
| `calibracao_de_m_um` | verde (tabela acima) |
| lei velha ao bit: `trail::` (13), `sim_resumable` (2), `fingerprint` (3), `deposit` (13), `flow_bit` (2) | verdes |
| `o_interruptor_da_tinta_nao_muda_um_pixel_com_a_lei_velha` | verde depois do ajuste abaixo |

**Testes E3 ajustados (`wp/grid/tinta_tests.rs`):**
- `o_interruptor_da_tinta_nao_muda_um_pixel_com_a_lei_velha`: comparava os píxeis e `susp_rgb`/`sett_rgb` das sessões
  com `Pigmento` contra a velha — agora o depósito Pigmento escreve a cor por Oklab (sem `mix.mix`) de propósito.
  Passa a provar: Velha × Velha píxeis iguais; com Pigmento a MASSA e a água (`film, susp, sett, vel`) iguais ao bit
  (vermelho antes do ajuste: os píxeis mudaram).
- `os_campos_da_tinta_nova_nao_existem_na_lei_velha`: supunha o campo NEUTRO depois de Velha→Pigmento sobre tinta já
  pousada; agora nasce migrado — a asserção passa a «há tinta migrada».

## Em aberto

- O invariante só vale até ao primeiro passo da água: advecção/secagem (E7/E8) e o pick-up/re-molhar/smear/blend
  (E6b, inclusive `molhado.rs:642` do `WetOpen`) ainda movem a massa sem mover as somas.
- A cor da ponta é a de hoje (`tip_*`, suja pelo bico com `mix`): a ponta como `Soma` é da E6b.
- `agua_do_dab`/`peso` do traço fino (E4 aberto) continuam só na água — não é cor, não tocado.
- `M_UM`: confirmar o critério no portão-mãe F0 (E9).
