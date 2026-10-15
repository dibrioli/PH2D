# Handoff de continuação — Luz 2D, W5v: o fundo dos opacos (2026-10-15) — PARCIAL, decisão do dono pendente

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6` (o main não andou), por cima da W5u ([handoff](HANDOFF_CONTINUACAO_W5u_2026-10-15_O_CUSTO_POR_FIO.md)),
> W5t ([handoff](HANDOFF_CONTINUACAO_W5t_2026-10-15_A_SUBIDA_PAGA_POR_FIO.md)), W5s
> ([handoff](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md)), W5r
> ([handoff](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md)) e W5q
> ([handoff](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md)), todas não integradas. Commits da W5v:
> `b7e2b92e7f` (perf: a fina salta o fundo dos opacos) · `e7dfcf2816` (test: a régua W5v). Nada integrado, nada
> enviado. Placa RTX 5060 Ti. `c2` = ms de GPU do passe holográfico; «por processo» = um processo por medida.

## 0. Estado

⏳ **DECISÃO DO DONO PENDENTE** (regra: bit não mantido ⇒ parar). O corte é exato ao bit contra a lei, mas o produto
muda contra o de hoje em **1 byte num caso de 42**: Medium, cena 3, câmera em x=2 alt 2, texel (1840, 834), canal R
0,80225 → 0,80176 (1 ulp f16, 0,06 %).

Mecanismo PROVADO: o `miolo_final` fazia `mix(k.rgb, difusão, suavidade=1)` e o driver arredonda pelo valor antigo de
`k` (a luz do fundo que a fina calculava); com o fundo saltado (`k=0`) o resultado é a difusão exata. Com um `select`
exato no `miolo_final`, antes × novo = 0 bytes. A chave `PH2D_LIGHT2D_FUNDO=0` (`Config::miolo_salta_o_fundo=false`)
devolve o antes AO BIT (salto e `select` juntos).

Se o dono recusar: padrão `false` (uma linha em `config.rs`) — o corte fica atrás da chave.

## 1. Medir: o que a régua do app mostrou

Instrumento `PH2D_LIGHT2D_CUSTO=1` ganhou a linha «onde». Janela 1920×1052, grade Low 640×384, fina ×4, captura ×8,
fluência 2560×1536, região 1172×642, F16. App, cena 5, ANTES (lote 1, placa só com o kwin), ms:

| Janela · nível | total | decomposição |
|---|---|---|
| 1920 Low | 7,15–7,65 | fina 3,18–3,21 · c2 1,25 · vis 0,75–0,79 · miolo 0,78 · rebote 0,6 · preenche 0,37 · entorno 0,47 |
| 1920 Medium | 7,37–7,83 | fina 2,13 |
| 1920 High | 15,9–18,7 | c2 10,3–12,6 (varia ENTRE processos, estável dentro: suspeita do regime de alocação) |
| 2560 Low | 9,0–9,1 | fina 3,80 |
| 2560 Medium | 8,6–8,9 | |
| 2560 High | 22,6–25,7 | |

O Inspector do dono dava 7,6 (AOC) e 9,0 (LG): REPRODUZIDO.

Régua nova `medida_luz2d_w5v_caso` (a configuração DO APP via `config_da_vista`, aritmética F16; as réguas antigas
usavam F32), um caso por processo: cena 5 FHD Low 6,99 (fina 2,81, c2 1,24, vis 0,59, miolo 0,78) · Medium 7,08 (fina
1,92) · High 15,54 (c2 9,33, fina 2,48); cena 1: fina Low 0,80 / High 1,05. A fina não é igual em todo nível (captura
×8/×4/×2).

Variantes da fina na cena 5 Low (só atribuição): sem borda exata 2,81→1,89 · passo×2 →2,04 · sem média no eixo →2,15 ·
sem face →2,23 · raios 2 →2,33 · bissecções 2 →2,34 · sem curvatura →2,33. No High só as do interior mexem (−0,1 a
−0,5). Etapas HRC High, cena 1 vs 5: traça 0,65/0,78 · sobe 1,24/1,67 · desce 4,84/5,22 · soma 1,39/1,46 (o conteúdo
soma ~1 ms; o resto do app é a placa partilhada/regime). A visibilidade não oscilou (0,75–0,92 no app; o 0,8–1,6 da
W5u não se reproduziu).

## 2. O corte

No fundo de um opaco (o texel e as 4 vizinhas dentro de um corpo, na região, semente do preenchimento na célula ⇒
todas remapeadas) com miolo e suavidade 1, o `miolo_classifica` dá peso 0 e o `miolo_final` troca a luz pela
difusão: a fina calculava a fronteira, a visibilidade medida e a c0 ao longo da face para nada.

Implementação: `fundo_do_grupo` no `c3_direta.wgsl` (override `FUNDO_SEM_LUZ`); o salto é do GRUPO 8×8 INTEIRO
(atómico + barreira); ladrilhos mistos calculam tudo. Também no `c3_fina_exata` (o High usa-o).

⛔ Achado: o salto POR TEXEL faz o driver parar a placa (Xid 109, CTX SWITCH TIMEOUT) — determinístico, dependente da
FORMA do código (15 experimentos por mutação do módulo: condição extra depois do teste corre, antes trava; tirar
qualquer peça grande do interior muda o resultado; teto em laços não cura). A impressão digital da imagem é igual em
todas as formas que correm ⇒ não é laço sem fim na semântica; é a mistura, no mesmo grupo de fios, de fios que saem
cedo com fios na marcha pesada.

## 3. Provas

Régua do bit `medida_luz2d_w5v_fundo_ao_bit` (mesmo processo, 80 quadros, casos da W5q + cena 8, Low/Medium/High = 42):
o corte sozinho 0 bytes em 42/42; controlos (fundo sem exigir vizinhas; salto forçado com suavidade 0,5) mudaram 27 e
30 casos; antes contra antes 0 bytes (a placa é determinística). O 1 byte do §0 é o produto contra o de hoje; com o
`select` exato no `miolo_final`, antes × novo = 0 bytes.

Tempos (lote 2; OUTRO `ph2d-host-desktop` na placa em todo o lote ⇒ só o A/B intercalado vale). Régua por processo,
chave off → on, ms: cena 5 FHD Low custo 7,04→6,64, fina 2,85→2,44 · Medium 7,13→6,75, fina 1,95→1,58 · High
15,82→15,67, fina 2,51→2,17; 1920×1052 Low 7,08→6,68 · 2560×1052 Low 8,43→8,01, fina 3,48→3,06; cena 6 Low fina
1,63→1,35; cena 3 0,88→0,75; cenas 1/4/8 iguais (sem fundo). App (par a par): fina 1920 Low 3,21→2,86 · Medium
2,16→1,83 · 2560 Low 3,83→3,45 · Medium 2,41→2,06; High ruidoso.

Portões (verificador, por nome, outra app na placa, load 13–44): `g3_` 8/8 (RMSE dentro da tolerância; o mais apertado:
céu e vidro Alto 0,0065/0,0072), `g4_` 4/4, `g5_` 2/2, holográfica 16/16, miolo 12/12, `g9_` 15/15, `g10_` 2/2,
`g12_` 2/2, `redimensionar_a_janela_nao_recompila_a_luz` 1/1, `a_nitidez_da_luz` 6/6, `o_interior_do_opaco` 5/5,
`o_miolo_das_formas` 1/1, `censo_wgsl_partilhado` 2/2, `smoke_cena_1` 1/1, `smoke_cenas_w4` 6/6;
`check --workspace --all-targets` limpo. (`o_topo_passa_a_grade` não existe: nome do briefing errado.) O gate batched
do fecho (clippy, nextest-impacted, censos) AINDA NÃO foi corrido.

## 4. Para o integrador

Campo público novo `Config::miolo_salta_o_fundo` (`true`); override `FUNDO_SEM_LUZ`; `UniformeNivel._q:[u32;2]` →
`exata: u32, _q: u32` (WGSL `_q0` → `exata`); `config_da_vista`/`config_do_entorno` públicas no `ph2d-app-light2d`
(`quadro_ambiente.rs`) — a única porta; `solver_miolo.rs` 697 LOC (teto 700). O texto dos shaders mudou ⇒ a 1.ª
abertura recompila a luz. Nenhum contrato congelado. Sem schema, sem registo novo.

## 5. ⛔ Recusas MEDIDAS

- O salto por texel (Xid 109, trava a placa).
- Teto de iterações no laço da abertura (não cura).
- Pôr a chave dentro de `Miolo` (muitos literais completos nos testes): ficou no `Config`, passada ao `DoMiolo`.
- As da W5u/W5t/W5s/W5r/W5q continuam (links acima).

## 6. ABERTO, por ordem

1. A decisão do dono (aceitar o 1 ulp, ou padrão `false`).
2. Fecho: gate batched (`PH2D_SUITE=fecho`), smoke do dono com os números, binário do smoke por último, cache
   aquecido, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.
3. Mais fina no Low: borda exata (−0,9), passo, média no eixo — mudam a imagem ⇒ produto.
4. `c2` do High no app varia 10,3–12,6 ENTRE processos (estável dentro): medir o regime de alocação no app
   (`hrc_nasce_duas_vezes`) antes de cortar.
5. O §6b da W5u (a cheia por DRAM; limiar 2^17).

Smoke do dono: ⏳ pendente (W5s, W5t, W5u, W5v).
