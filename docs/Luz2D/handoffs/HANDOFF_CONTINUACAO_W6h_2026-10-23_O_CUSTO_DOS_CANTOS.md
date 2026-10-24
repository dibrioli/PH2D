# Handoff de continuação — Luz 2D, W6h: o custo dos cantos nas 7 câmeras (2026-10-23)

> Linha `line/Luz2D` sobre o `main` `972f411a6f` (reaberta depois da integração W6b–W6g, 0 commits próprios à
> partida). Ponto de partida: [integração W6b–W6g §9](HANDOFF_INTEGRACAO_line_Luz2D_2026-10-22_W6b_a_W6g.md) e
> [W6g §56](HANDOFF_CONTINUACAO_W6g_2026-10-19_O_CAMPO_QUE_GUARDA_O_VERTICE.md). Regras R1–R9 as da W5f.
>
> ⚠️ **Os instrumentos da W6g que viviam em `target/w6g/` PERDERAM-SE** (régua de registos, `distantes_custo_regua.rs`,
> `dump_head/`, `diag_*.py`): nenhuma cópia na máquina em 23/10. As réguas desta onda são VERSIONADAS (§2).

## 1. O elo do teto dos cantos até à tela (o §9.4 da integração) — feito, `b10cb7c544`

- `o_aviso_dos_cantos_chega_ao_inspector::o_teto_dos_cantos_chega_a_frase_do_cartao_luz`: a cena 5 (FHD, vista de
  fábrica) pela `LuzDoQuadro` com o teto mínimo (`com_teto_dos_cantos(u32::MAX)`, novo) — a placa recusa 29 blocos →
  a ficha da `publicar` (a da `fase_snapshots_publish`) leva `AvisoDaLuz::CantosArredondados { blocos: 29 }` → o
  Inspector pinta `panel.inspector.luz.corners_rounded_n` (lido pelo `censo_dos_avisos`). Controlo: o teto do padrão
  não diz nada, com os tempos lidos ≥ 2× (senão a régua é cega).
- O teto entra pela `LuzDoQuadro`, NÃO pelo `Degrau`: a `preferir` troca o degrau pelo `degrau_do(nível)` a cada
  quadro. `config_do_solver` passou ao irmão `quadro_ambiente.rs` (`quadro.rs` 700 → 698). Dev-deps novas:
  `ph2d-panel-inspector`, `ph2d-ui-testkit` (família → painel, aresta permitida).
- **Os tempos leem-se sem esperar**: o teste espera a placa entre quadros (`poll(wait)`), como o `a_nitidez_da_luz`.
- **Prova de mutação** (com a placa, `load` 107–122, sem relógio no teste): a cópia dos tempos (`quadro.rs`) · o
  `push` do aviso (`quadro_avisos.rs`) · a ficha (`inspector.rs`, `avisos: vec![]`) · a frase (`sections/luz.rs`,
  `String::new()`) — as quatro VERMELHAS no salto certo; repostas, verde.
- ⚠️ `cargo-test-narrow.sh` deu-o VERDE sem placa (o «sem GPU — saltado»): o portão só vale pelo `ph2d-run.sh`.
- Fica de fora: a linha da shell `set_current_inspector_luz(info)` (o teste repete-a, não a corre).

## 2. As réguas (versionadas, `f951410516`)

- `medida_luz2d_cantos_custo` (`#[ignore]`, à mão — cabeçalho): as 7 câmeras do §55 b (`Camera2d::new(centro, h)` com
  `h` = altura do mundo à vista; a de fábrica 10 m) × lei W6d · cunha · cunha + V2 · cunha + canto · padrão · padrão sem
  a luz do canto · lei W6d de controlo; 9 rodadas intercaladas, o MÍNIMO da C3 (preenche + visibilidade + fina) e as
  parcelas. As câmeras uma de cada vez (7 solvers vivos; 49 não cabem em 16 GB).
  **Validada contra o §55:** a lei W6d a ±0,01 ms nas 7 (2,345/2,351 · 2,609/2,614 · 3,641/3,630 · 3,036/3,047 ·
  1,932/1,931 · 1,541/1,534 · 2,090/2,090); controlo ±0,5 %.
- `medida_luz2d_cantos_bits` (`#[ignore]`): a prova de MESMO BIT das curas — os 32 casos (cenas 1–8 × miolo com/sem ×
  {lei W6d, padrão}), fluência em bruto. `PH2D_CANTOS_BITS=<pasta>` grava; `+ PH2D_CANTOS_BITS_BASE=<base>` compara e
  reprova com um byte diferente. **A base AINDA NÃO FOI GRAVADA** — grave-a sobre o HEAD antes de mexer no produto.

## 3. O padrão sobre a lei W6d, as 7 câmeras (4 rodadas, `load` 7–32; só relativos do mesmo processo)

| câmera | padrão (r1 · r2 · r3 · r4) | blocos com cunha |
|---|---|---|
| 5 · [−6, −1] h 2,5 | +6,6 · +5,7 · +6,4 · +6,4 % | 18 |
| 5 · [6, −1] h 2,5 | +12,8 · +11,1 · +10,8 · +11,5 % | 409 |
| 5 · zoom 1,6 | +5,3 · +5,9 · +4,9 · +5,9 % | 256 |
| 5 · zoom 1 | +4,6 · +4,2 · +4,0 · +4,4 % ✓ | 152 |
| 6 · [0, −3] h 1,5 | +7,0 · +7,3 · +7,1 · +7,5 % | 6 |
| 6 · zoom 4 | +7,7 · +7,8 · +6,9 · +7,3 % | 20 |
| 6 · zoom 1 | +3,0 · +2,1 · +2,4 % ✓ (r4: a lei W6d saiu alta, 2,252 — ruído) | 20 |

A borda: +0,03 ms pela cunha, +0,006…0,024 pela `V2` (quase fixa). **O grosso é a FINA.**

## 4. A bisseção da fina (diagnóstico FORA do git, reposto): o custo da `V2` é TRABALHO, não marcha mais longa

Override temporário `DIAG_V2` no `meio` (`c2_rc.wgsl`), pela `Config::diag_v2` — cada S decide `d < 0` (a decisão da
«cunha», com o MESMO campo `d`), logo a imagem e as marchas são as da cunha e a diferença é só trabalho:
S1 = lê a tabela · S3 = + o atlas · S4 = + a conta da cunha sem trigonometria · S2 = + a conta inteira (descartada) ·
S5 = a tabela num endereço fixo (sempre em cache). Fina, ms (`load` 7–9, a rodada mais limpa):

| câmera | cunha | S5 | S1 | S3 | S4 | S2 | cunha + V2 (imagem) |
|---|---|---|---|---|---|---|---|
| 5 · [−6, −1] | 1,518 | 1,535 | 1,558 | 1,603 | 1,712 | 1,721 | 1,739 |
| 5 · [6, −1] | 1,610 | 1,614 | 1,702 | 1,806 | 1,907 | 1,932 | 1,933 |
| 5 · zoom 1,6 | 2,259 | 2,269 | 2,351 | 2,418 | 2,474 | 2,512 | 2,471 |
| 5 · zoom 1 | 1,943 | 1,898 | 1,961 | 1,993 | 1,974 | 2,000 | 1,982 |
| 6 · [0, −3] | 1,057 | 1,049 | 1,081 | 1,111 | 1,205 | 1,225 | 1,204 |
| 6 · zoom 4 | 0,930 | 0,854 | 0,925 | 0,954 | 0,997 | 1,025 | 1,013 |

- **S2 ≈ cunha + V2** em todas: a `V2` não alonga a marcha; o custo é a avaliação dela em cada amostra `|d| < 1`
  dentro do laço da marcha ⇒ há espaço para uma cura de MESMO BIT, do tamanho do que falta em toda câmera.
- **A tabela (S1 − cunha, +0,02…+0,09) são FALHAS DE CACHE:** S5 ≈ cunha. A tabela é `u32` por bloco 8×8 da captura
  (129 600 entradas a 1080p, ~0,5 MB) e é lida em TODA amostra perto de qualquer borda, não só junto aos cantos.
- **O atlas (S3 − S1) +0,03…+0,10; a conta (S2 − S3) +0,05…+0,13, quase toda SEM a trigonometria (S4 ≈ S2).**
- **A luz do canto BAIXA a fina** (o padrão < cunha + V2 em 5 de 7): o ramo `CANTO_LUZ` poupa marchas.

## ⛔ Recusas MEDIDAS (W6h)

| Recusa | Porquê |
|---|---|
| Guardar seno e cosseno da cunha no atlas (tirar a trigonometria do `meio`) | S4 ≈ S2: a trigonometria custa 0,01–0,04 ms; o peso é a conta estar no laço. |
| O teto dos cantos no `Degrau` | A `preferir` troca o degrau pelo do nível a cada quadro: o teto perdia-se no caminho do produto. |

## 5. ABERTO (por ordem)

1. **A cura do custo da `V2` na fina, MESMO BIT** — kill-criterion (escrito ANTES de construir):
   **PASSA** (todos): (i) `medida_luz2d_cantos_bits` 0 bytes diferentes nos 32 casos contra a base gravada no HEAD de
   antes; (ii) C3 do padrão ≤ +5 % sobre a lei W6d em CADA uma das 7 câmeras, 3 processos, o mínimo de 9 intercaladas
   (`medida_luz2d_cantos_custo`), `preenche + borda` ≤ +0,17 ms; (iii) `os_cantos_custam_ate_5_por_cento_da_memoria`,
   `o_teto_dos_cantos_recusa_e_diz` e `o_teto_dos_cantos_chega_a_frase_do_cartao_luz` verdes COM placa.
   **MORRE**: qualquer um; duas reconstruções sem passar ⇒ PARE e prove o modelo.
   Candidatas, pela bisseção do §4:
   - (a) **a tabela atrás de uma máscara de bits** (1 bit por bloco da captura, ~16 KB a 1080p, cabe na cache) — só
     o bit ligado lê a `u32`; mira o S1 − S5 (+0,02…+0,09). Exata por construção.
   - (b) **a conta da cunha fora do caminho comum da marcha** — o S3 → S4 (+0,04…+0,11) tem cara de registos /
     divergência; medir pela régua de registos (§20.1 da W6g) ANTES de escolher a forma. ⚠️ A régua perdeu-se: refazê-la
     VERSIONADA (crate fora do workspace com `Cargo.lock` próprio, wgpu-hal 29, `VK_KHR_pipeline_executable_properties`).
   - (c) o atlas (S3 − S1): por medir a forma.
2. A borda nos níveis 11–12 a 2160p Alta: +3,62 ms (W6g §54.1).
3. A anomalia da r5b (W6g §12); a família (b) (W5g §37–38).

Logs (fora do git, `target/`): `cantos_custo_1.log` (r1), `cantos_custo_diag.log` (S1/S2), `_diag2` (+ S3), `_diag3`
(+ S4/S5) — as tabelas acima são o que vale deles.

## 6. A rodada do §5.1 (23/10): o custo da `V2` NÃO é a marcha, é o `num_corpo` — (a) morre

- **A base do mesmo bit** gravada no HEAD `244c5511f7` (`target/cantos_bits/base/`, 32 casos); controlos: 2.ª corrida 0 bytes
  diferentes, lei W6d ≠ padrão nas cenas 2, 5, 6.
- **A régua de registos, versionada** (`9a381c7990`): `spikes/regua-registos` + `medida_luz2d_regua_registos`; as formas
  por `spikes/regua-registos/formas_w6h.py <pasta do v4> <saida>` — a mesma pasta entra na régua do custo como
  variantes do mesmo processo (`PH2D_CANTOS_CUSTO_MODULOS`, `Config::modulo_rc`). Uma rodada de 7 câmeras ≈ 18 s.
- **Registos: todas as variantes da fina em 128**, pilha 0 ⇒ o custo da `V2` não é registo. O ramo da cunha no `meio`
  põe **+101 KB** de binário (221 568 → 322 688): ~10 KB nos 3 sítios da marcha, **91 KB nos 17 de fora** (62 KB só
  no `num_corpo`).
- **O relógio** (`load` 3–13; controlos lei W6d ±0,9 %; `x0` = padrão), C3 sobre a lei W6d:

| câmera | padrão | ramo só na marcha (`x4`) | sem ramo (`x9`) | sem o ramo no `num_corpo` (`ga`) |
|---|---|---|---|---|
| 5 · [−6, −1] | +5,4…7,2 % | +2,0 | +0,6…1,4 | +1,7…3,2 |
| 5 · [6, −1] | +10,5…13,9 | +4,0 | +1,8…3,4 | +3,2…4,1 |
| 5 · zoom 1,6 | +5,3…7,5 | −0,8 | −0,9…0,0 | −1,0…0,0 |
| 5 · zoom 1 | +4,1…5,1 | +1,7 | +1,9…2,1 | +0,9…1,7 |
| 6 · [0, −3] | +7,2…8,9 | +2,9 | +2,3…2,4 | +3,0…3,7 |
| 6 · zoom 4 | +7,2…10,6 | −4,3 | −4,9…−5,3 | −3,8…−4,1 |
| 6 · zoom 1 | +2,0…4,2 | +1,6 | +1,5…3,1 | +2,1…3,1 |

  ⇒ a `V2` na MARCHA custa ~0; **todo o excesso é o ramo no `num_corpo`** (o predicado geométrico, que amostra EM CIMA da
  borda, `|d| < 1` sempre). Por chamador (`h_*`, `load` 28–30): nenhum domina — o pé (`c3_pe.wgsl`) ~40–50 %, a
  `na_fronteira` ~25 %, a `na_fronteira_fina` ~20 %. Pelas peças no `num_corpo` (`load` 38–44, só relativos): só a
  leitura da tabela ~metade; o atlas e a conta a outra; a trigonometria nada.
- **(a) a máscara de bits** (1 bit por bloco depois das entradas, no mesmo buffer; o leitor lê o bit e só então a
  entrada; exata por construção) — construída e medida: **PIOR** que sem ela em 7/7 (+0,00…+0,04 ms), também com
  saída única (`p3`). **A saída única** do `meio` sem máscara (`p2`) = o padrão. ⇒ o modelo do W6g §25 confirma-se no
  `num_corpo`: o custo é a ESTRUTURA da leitura condicional, não as falhas de cache. **1.ª reconstrução sem passar**;
  o produto reposto pelo git.
- ⚠️ **A alocação de registos tem degraus:** o `u0` (o `ga` com o leitor da máscara) com `DISTANTES 1` vai a **195**
  registos (o `ga` fica em 128) e o C3 dele +23 % nas três câmeras com distantes; o ramo UNIFORME no `num_corpo`
  (`if <uniforme> { meio } else { meio_sem }`) vai a 168 e custa +15…+81 %. Toda forma nova passa pela régua com
  `DISTANTES 0` E `1`.

## ⛔ Recusas MEDIDAS (§6)

| Recusa | Porquê |
|---|---|
| A máscara de bits antes da `cantos_tabela` | Pior em 7/7: a leitura condicional a mais custa mais do que as falhas que poupa (§6). |
| A saída única do `meio` (um `return` em vez de quatro) | = o padrão (`p2`). |
| O ramo uniforme `meio`/`meio_sem` no `num_corpo` no mesmo kernel | 168 registos, +15…+81 %. |
| Curar a `V2` na marcha | `x4` ≈ `x9`: a marcha não paga. |

## 7. ABERTO (por ordem — substitui o §5)

1. **A cura do custo da `V2`, mesmo bit — a ÚLTIMA reconstrução antes do two-strikes** (kill-criterion do §5.1 tal e
   qual). A única forma medida que chega ao alvo nas 7 câmeras é o `num_corpo` sem o ramo (`ga`: ≤ +4,1 %), e só é de
   mesmo bit onde nenhuma consulta do `num_corpo` cai num bloco com cunha. Desenho: **duas pipelines da fina** — a
   enxuta (o `num_corpo` sem o ramo; a marcha com ele) para os grupos longe de toda cunha, e a de hoje, por despacho
   indireto, sobre a lista dos grupos perto (o molde do `c3_fina_adia`/`c3_fina_cantos`). ANTES de construir: (i) o
   raio MÁXIMO das consultas do `num_corpo` à volta do texel da fina (pé na face, `na_fronteira*`, `c0_ao_longo_da_face`)
   — sem um limite, não é exata; (ii) a enxuta pela régua com `DISTANTES 0/1` (o degrau do §6); (iii) quantos grupos
   ficam «perto» na c5 [6, −1] (409 blocos com cunha), que decide se o ganho sobrevive.
2. A borda nos níveis 11–12 a 2160p Alta: +3,62 ms (W6g §54.1).
3. A anomalia da r5b (W6g §12); a família (b) (W5g §37–38).

Logs (`target/`): `cantos_custo_clock{1..6}.log`, `regua_registos/{head,head2,b,clock*,dist*}/`.

## 8. As três medições do §7.1 (23/10, por ordem do dono «siga») — a obra compensa

- **(i) O raio das consultas do `num_corpo`** (leitura do WGSL pelo explorador, com linhas): fora da marcha todo
  `num_corpo`/`meio` do caminho da `c3_fina` cai a ≤ **~16,4 texels da fina** do texel calculado (o máximo:
  `saida_na_direcao`, `c3_pe.wgsl:323`, via `normal_secante`; o pé, `na_fronteira*`, `c0_ao_longo_da_face` e o
  `peso_de_fora` abaixo dele) — **SALVO as sementes sem porta do `onde_calcular`** (`c3_direta.wgsl:742–750`: quando
  a janela 5×5 não tem texel de fora — interiores translúcidos, a borda da região), que podem ir à diagonal da grade.
  ⇒ a enxuta só é exata se ADIAR esses texels para a pipeline completa (o molde da lista do `c3_fina_adia`).
- **(ii) A enxuta** (`ga` + a saída por grupo `e1`: uma leitura uniforme por grupo antes do `fundo_do_grupo`): **128
  registos com `DISTANTES 0` e `1`**; no relógio (`load` 12–13) `e1` = `ga` + 0,00…0,03 ms, C3 **≤ +3,5 %** nas 7.
- **(iii) Os grupos 8×8 da fina PERTO de uma cunha** (`medida_luz2d_cantos_perto`, o retângulo do grupo na captura
  dilatado por R contra os blocos com cunha):

| câmera | R 4 | R 8 | R 16,4 |
|---|---|---|---|
| 5 · [−6, −1] | 0 % | 0 % | 0,1 % |
| 5 · [6, −1] | 1,2 % | 1,7 % | 2,7 % |
| 5 · zoom 1,6 | 2,7 % | 4,4 % | 8,4 % |
| 5 · zoom 1 | 3,1 % | 4,9 % | 9,6 % |
| 6 · [0, −3] | 0,0 % | 0,1 % | 0,1 % |
| 6 · zoom 4 | 0,2 % | 0,2 % | 0,6 % |
| 6 · zoom 1 | 0,5 % | 0,9 % | 2,1 % |

  Estimativa (NÃO medida): enxuta + fração × (padrão − enxuta) ⇒ [6, −1] ~+3,6 %, as outras ≤ ~+2,6 %; o despacho a
  mais e os texels adiados por medir.

## 9. ABERTO (por ordem — substitui o §7)

1. **Construir as duas pipelines da fina** (a 2.ª e ÚLTIMA reconstrução do two-strikes; kill-criterion do §5.1 tal e
   qual, mais: a enxuta pela régua com `DISTANTES 0/1` ≤ 128 registos). Peças: (a) um passe que marca os grupos da fina
   perto de uma cunha (o mapa dos blocos com cunha dilatado por R = 17 texels da fina, depois do `c3_cantos`) e os
   lista para despacho indireto; (b) a `c3_fina` compilada com o `num_corpo` sem o ramo (uma constante `override`,
   p. ex. `NUM_CORPO_SEM_CANTO`), a sair nos grupos marcados (`e1`) e a ADIAR para uma lista o texel que tome a semente
   sem porta do `onde_calcular`; (c) a `c3_fina` de hoje sobre os grupos listados e sobre os texels adiados. Exata por
   construção; a prova é o `medida_luz2d_cantos_bits` contra a base do §6.
2. A borda nos níveis 11–12 a 2160p Alta: +3,62 ms (W6g §54.1).
3. A anomalia da r5b (W6g §12); a família (b) (W5g §37–38).

## 10. As duas pipelines da fina — o censo e o desenho (escrito ANTES de mexer no produto; 2.ª e última reconstrução)

- **A base do mesmo bit confere no HEAD `e5725967e4`**: 32 casos, 0 bytes diferentes (`target/cantos_bits/hoje`).
- **O censo das sementes sem porta** (`medida_luz2d_sementes_sem_porta`, `0b2872cdd0`: o padrão com a sonda 7 por remendo
  CONTADO da `fonte_do_rc`, sem tocar no produto). ⚠️ O padrão corre com o miolo, que liga o `FUNDO_SEM_LUZ` (salta os
  grupos do fundo, onde mora a maioria destes texels); a sonda só chega à fluência sem o miolo ⇒ o salto emula-se à
  força. Com o fundo saltado (o caminho real):

| câmera | texels sem porta | distância máx. (texels da fina) | perto de uma cunha (texels / grupos ≈) |
|---|---|---|---|
| 5 · [−6, −1] | 0 | — | 0 |
| 5 · [6, −1] | 2 179 (0,30 %) | 77 | 2 179 / 110 |
| 5 · zoom 1,6 | 13 040 (1,3 %) | 39 | 5 154 / 359 |
| 5 · zoom 1 | 6 132 (0,9 %) | 19 | 4 314 / 259 |
| 6 · [0, −3] | 0 | — | 0 |
| 6 · zoom 4 | 7 076 (1,0 %) | 141 | 305 / 20 |
| 6 · zoom 1 | 3 313 (0,5 %) | 19 | 943 / 54 |
| cena 2 (fábrica) | 12 197 (1,7 %) | 19 | 9 525 / 878 |

  Sem o salto (a lei sem miolo) são até 15 % (c5 zoom 1,6). ⇒ **NÃO são raros**: a lista ADIADA por texel do §9.1 b
  teria 300–9 500 texels, e um passe por texel em lista custa ~0,5 ms de LATÊNCIA (W6g §12, recusado) — sozinho passa
  o alvo (~0,15 ms). **A lista por texel morre antes de construir.**
- **O desenho** — a marcação é por GRUPO e cobre as sementes:
  - **(a) `c3_fina_marca`**, um fio por grupo 8×8 da fina, depois do `c3_cantos` e do preenchimento: o grupo é PERTO se
    o retângulo dele dilatado por R = 17 texels da fina toca um bloco 8×8 da captura com cunha, OU se o retângulo que
    junta o grupo a alguma semente do `preenchido` que o `onde_calcular` pode ler (as células da janela 5×5 e as 3×3
    vizinhas de cada texel do grupo), dilatado por R, toca um. Os perto entram numa lista (cabeçalho = despacho
    indireto) e num mapa por grupo. Exata: todo `num_corpo` fora da marcha cai a ≤ ~16,4 texels do texel OU — sem
    porta — do segmento texel → semente (o `na_fronteira`, o `pe_na_borda` e o resto à volta do ponto remapeado);
    as sementes COM porta estão a < 6 texels.
  - **(b) a enxuta**: a `c3_fina` com `override NUM_CORPO_SEM_CANTO` (o `num_corpo` pelo `meio` sem o ramo da cunha; a
    marcha com ele — a forma `ga`) e a saída do grupo marcado ANTES do `fundo_do_grupo` (a forma `e1`, uma leitura
    uniforme). Régua de registos com `DISTANTES 0` e `1`: ≤ 128.
  - **(c) a fina de hoje** por despacho indireto sobre a lista (o grupo listado → o `gid` dele; o `fundo_do_grupo`
    corre igual).
- **A fração marcada** (`medida_luz2d_cantos_perto`, linha `MARCA`): c5 [−6, −1] 0,1 % · [6, −1] **4,0 %** · zoom 1,6
  12,2 % · zoom 1 13,3 % · c6 [0, −3] 0,2 % · zoom 4 0,9 % · zoom 1 3,1 %. Estimativa (NÃO medida): enxuta + fração ×
  (padrão − enxuta) ⇒ [6, −1] ~+4,5 %, as outras ≤ ~+3,7 %, mais o passe a mais e o 2.º despacho — apertada na
  [6, −1].
- **Kill-criterion**: o do §5.1 tal e qual + a enxuta ≤ 128 registos com `DISTANTES 0/1`. Falhar ⇒ produto reposto pelo
  git e PROVAR O MODELO (não há 3.ª forma).
- **A regra EXATA** (a mesma na CPU e na placa; refinada depois de ler o `pe_na_borda` e o `outra_face`): o disco do
  `pe_na_borda` tem raio `d + 2h` à volta do texel e não o segmento; `d` ≤ a semente mais perto das 3×3 células, a
  outra face < `d + 2h` ⇒ o grupo dilatado por 17 + `d_max` + 4, OU o segmento até cada semente dilatado por 1 texel
  (a bissecção do `na_fronteira` só lê à volta dele). Fração marcada: c5 [−6, −1] 0,1 % · [6, −1] 4,2 % · zoom 1,6
  18,2 % · zoom 1 16,1 % · c6 [0, −3] 0,2 % · zoom 4 2,6 % · zoom 1 5,2 % (régua `medida_luz2d_cantos_perto`, linha `EXATA`).

## 11. A 2.ª reconstrução — construída, de mesmo bit, e MORRE pelo custo; o modelo provado; produto reposto

- **O produto** (`9070f679d8`, revertido no commit seguinte — `git show 9070f679d8` o reconstrói): a tabela das áreas
  dos blocos com cunha (`c3_fina_cunhas_linhas`/`_colunas`, 256 fios por linha/coluna), a `c3_fina_marca` (a regra
  exata), a `c3_fina_enxuta` (`FINA_ENXUTA`: o `num_corpo` pelo `meio_sem_cunha`, GERADO do `meio` na montagem do
  módulo, `shaders.rs`, por transformação contada) e a `c3_fina_listada` por despacho indireto; `Config::fina_dupla`.
  **Mesmo bit: 32 casos, 0 bytes** contra a base. **Registos: as cinco em 128** (enxuta 260 KB, listada 323 KB, com
  `DISTANTES 0/1`). Portões com placa: memória +3,0 %, os dois do teto verdes.
- **O custo reprova** (3 processos, `load` 12–16, controlo lei W6d ±1 %): o padrão a **+19,3…+28,9 %** sobre a lei
  W6d nas 7 câmeras (o de antes, no mesmo processo, +2,2…+12,2 %).
- **A bisseção** (`spikes/regua-registos/formas_w6h_dupla.py` sobre o `v4.wgsl` da régua de registos; `load` ~3,5;
  `d0_igual` = o padrão ±1 %) — a fina em ms:

| forma | c5 [−6, −1] | c5 [6, −1] | c5 z 1,6 | c5 z 1 | c6 [0, −3] | c6 z 4 | c6 z 1 |
|---|---|---|---|---|---|---|---|
| lei W6d | 1,381 | 1,607 | 2,257 | 1,951 | 1,060 | 0,924 | 1,113 |
| padrão de antes (uma fina) | 1,503 | 1,840 | 2,395 | 2,030 | 1,181 | 1,024 | 1,132 |
| só a enxuta, passes vazios (`d3`) | 1,500 | 1,757 | 2,265 | 1,946 | 1,143 | 0,925 | 1,132 |
| + a tabela (`d2`) | 1,528 | 1,808 | 2,313 | 1,995 | 1,182 | 0,970 | 1,160 |
| + a marcação, nada listado (`d1`) | 1,575 | 1,810 | 2,363 | 2,026 | 1,226 | 1,011 | 1,202 |
| as duas finas (`d0`) | 1,950 | 2,074 | 3,029 | 2,569 | 1,439 | 1,338 | 1,499 |

  e, noutra rodada (`load` ~5), sobre o padrão de UMA fina: a `ga` e a `e1` antigas da §8 e o módulo de hoje com o
  `num_corpo` sem cunha em todo o lado (`g1`) — os três iguais, C3 −3,9…+4,9 % (a enxuta EM SI está certa: os ~0,09 ms
  de `d3` acima de `g1` são os quatro despachos a mais e as barreiras).
- ⇒ **O MODELO**: o 2.º despacho da fina pesada custa a LATÊNCIA da cadeia mais longa dos grupos dele — **0,26–0,42 ms
  com 0,1 % ou com 5 % dos grupos listados** (0,72 com 18 %) —, e os grupos perto de uma cunha são os mais lentos. Numa
  fina só eles sobrepõem-se aos outros 11 mil; num despacho próprio correm com a placa vazia. É o MESMO mecanismo do
  passe dos cantos da W6g §12 (~0,45–0,5 ms quase igual com 6 367 ou 2 548 texels). A tabela (0,03 ms) e a marcação
  (0,05–0,08) somam pouco; o `d_max` (576 leituras por fio) é a maior parte da marcação.
- **Two-strikes atingido** (§5.1): (a) a máscara de bits e (b) as duas pipelines. Produto reposto pelo git (o mesmo bit
  contra a base conferido depois: 32 casos, 0 bytes). Instrumentos que ficam: o censo `medida_luz2d_sementes_sem_porta`,
  a linha `MARCA`/`EXATA` do `medida_luz2d_cantos_perto`, `PH2D_CANTOS_CUSTO_SO` e `PH2D_CANTOS_CUSTO_MODULOS_BASE` na
  régua do custo, a régua de registos com `_d1`, e o `formas_w6h_dupla.py`.

## ⛔ Recusas MEDIDAS (§10–§11)

| Recusa | Porquê |
|---|---|
| A lista ADIADA por texel (as sementes sem porta numa lista, um passe por texel) | 300–9 500 texels; um passe por texel em lista custa ~0,5 ms de latência (W6g §12). |
| As duas pipelines da fina EM SÉRIE (marca → enxuta → listada indireta) | O 2.º despacho custa a latência dos grupos lentos, 0,26–0,42 ms fixos: +19…+29 % (§11). |
| A regra pelo SEGMENTO até à semente com dilatação 17 (sem o disco) | Inexata: o `pe_na_borda` consulta um disco de raio `d + 2h` (§10). |

## 12. ABERTO (por ordem — substitui o §9) — é o DONO que decide se o item 1 continua

1. **A cura do custo da `V2`: o two-strikes chegou.** O que o modelo deixa (cada uma seria uma 3.ª forma, só com ordem):
   (a) a listada SEM barreira com a enxuta — a escrever num buffer seu (o grupo 2 sem as saídas, o molde do
   `c3_fina_cantos`) e uma junta no fim, para a placa as sobrepor (⚠️ a anomalia r5b: pré-lista ‖ fina sem barreira saiu
   MAIS cara que a soma — medi-la primeiro); (b) um despacho só com o ramo UNIFORME por grupo no TOPO (as duas finas
   inteiras num kernel; ⚠️ o ramo uniforme no `num_corpo` foi a 168 registos — régua antes); (c) aceitar o custo de hoje
   (+2…+12 % na C3 com os cantos).
2. A borda nos níveis 11–12 a 2160p Alta: +3,62 ms (W6g §54.1).
3. A anomalia da r5b (W6g §12) — agora com a mesma família de (1a): a latência de um despacho pequeno da fina pesada; a
   família (b) (W5g §37–38).

> **O item 1 do §12 FECHOU por ordem do dono (opção c: o custo de hoje fica).**

## 13. A borda nos níveis de cima (§12.2) — bissectada; a leitura SEM desvio (−3,0 ms a 2160p Alta, o mesmo bit)

Régua VERSIONADA `medida_luz2d_borda_topo` (`2d1a819ea5`, `72476f0a40`): cena 5, `[-4.2, -1.8]` h 3,8, a fina que
cabe (k 1 a 2160p, grade 4992×2944), a vista do produto `P` (céu · borda · borda de controlo) e UMA forma do
`de_fora` por processo (`PH2D_BORDA_TOPO_FORMAS`; a 2160p Alta só cabem duas vistas de ~4,1 GB), 9 rodadas, o
mínimo; depois as fases por nível; `PH2D_BORDA_TOPO_REGISTOS` escreve os módulos para `spikes/regua-registos`
(o instrumento `Solver::constantes_da_hrc`). Logs em `target/borda_topo/`.

**(a) A medição de hoje** (`load` 15–45): a borda +4,2…+5,1 ms de `c2` sobre o céu (A×A ≤ ±0,8); n 11 + 12 =
+3,51 ms (era +3,62), dos quais a CONTA do `de_fora` ~2,84 e a leitura ~0,66.

**(b) ⚠️ A vista `P` da régua tem REGIME:** a soma (`hrc_soma` n 0) 12,3–13,3 ms contra 5,5–5,6 da `identidade`
(o MESMO módulo por `modulo_rc`, 27 registos nas duas), e as subidas −0,3 ms por nível; não é a ordem de nascimento
(vista antes/depois do entorno, renascida), nem a troca céu ↔ borda (`PH2D_BORDA_TOPO_SEM_CEU=1`). A descida de `P`
cai também, em 2 de ~10 processos, a 3,2 ms por nível (contra 2,2). ⇒ forma × forma (a `identidade` como controlo),
nunca forma × `P`. Se a app real cai neste regime NÃO foi medido (o W6g §51 sugere que não).

**(c) A bisseção (registos da `hrc_desce`, o resto igual):**

| forma | registos | descida n 0 | n 11 | n 12 |
|---|---|---|---|---|
| produto / identidade | 46 | 2,24–2,38 | 3,83–3,89 | 2,62–2,64 |
| `fora_base` por um escalar | 46 | 2,24 | 3,78 | 2,56 |
| sem a escrita do arquivo / e só-leitura | 43 / 43 | — | — | — |
| sem ler o arquivo | 40 | 1,93 | 3,47 | 2,39 |
| só o ponto `b` / até à coluna | 38 / 38 | 1,96 | 2,09 / 2,27 | 1,19 / 1,35 |
| **a leitura sem desvio** | **38** | **1,93** | **3,55–3,57** | **2,48–2,49** |

A leitura do arquivo DENTRO do `if` levava a descida a 46 registos e custava ~0,25–0,3 ms em TODO nível, céu incluído
(a mesma pipeline). Ler sempre (`select(0u, i, ok)`) e escolher depois (`select(ceu, lido, ok)`) é o mesmo valor.

**(d) A cura (`6a93d54d04`, `c2_hrc.wgsl`):** `c2` mínimo, forma × forma (2160p Alta, 2 × 2 processos alternados;
os outros tetos no mesmo processo, `load` 165–254):

| teto | sem desvio (o produto) | com desvio (antes) | |
|---|---|---|---|
| 2160p Alta | 46,24 · 46,30 | 49,41 · 49,14 | −3,0 ms (−6,1 %) |
| 1080p Alta | 10,345 | 11,024 | −6,2 % |
| 1440p Média | 4,193 | 4,436 | −5,5 % |
| 2160p Baixa | 5,296 | 5,593 | −5,3 % |
| 1080p Baixa | 1,373 | 1,394 | −1,5 % |

**Gate:** `a_leitura_da_borda_sem_desvio_nao_muda_um_bit` (ficheiro próprio; as 4 vistas do portão do T nulo + 2160p
Alta, o produto × a leitura de antes por `modulo_rc`): 0 texels nas 5. Mutação (`PH2D_SEM_DESVIO_MUTA`): M1 sem o
`select` do céu 164 931 texels · M2 só a 1.ª sonda 1 690 432; controlo a corrida verde. Também: `medida_luz2d_cantos_bits`
32 casos 0 bytes contra a base; `a_luz_de_fora_nao_atravessa` 8/8. ⚠️ Provado só na NVIDIA (bug 7).

## ⛔ Recusas MEDIDAS (§13)

| Recusa | Porquê |
|---|---|
| O índice `fora_base[m/4][m%4]` por um escalar do uniforme | 46 registos e a mesma descida: não é o índice dinâmico. |
| Declarar o arquivo só-leitura / tirar a escrita (o módulo por papel) | 43 registos; a leitura sem desvio sozinha dá 38 com a escrita. |
| Comparar uma forma com a vista `P` da régua (soma, subidas) | `P` tem regime: a `identidade` com o mesmo módulo dá a soma em 5,5 contra 12,6 ms. |
| Contar bits entre `P` e uma forma depois de quadros diferentes | O estado acumulado difere: a `identidade` dá os mesmos 409 891 texels que qualquer forma. |

## 14. ABERTO (por ordem — substitui o §12)

1. **O bug 7 (`BUGS_luz2d.md`): 7 testes reprovam na Radeon integrada** — o mesmo bit por dispositivo (T nulo, um
   cone), o enchimento do `hrc_r0` (524 288 contra 3 072), precisão/lei de custo da holográfica e do miolo; a cura do
   §13 também se prova lá. A reprodução sem a NVIDIA está no bug.
2. A conta do `de_fora` em n 11–12 a 2160p Alta: ~2,8 ms, sem ideia de mesmo bit (tabela recusada W6g §43 c; laço
   único W6g §53).
3. O regime da soma da vista `P` (§13 b): medir se a app real (o `LuzDoQuadro`, com o renascimento) cai nele.
4. A anomalia da r5b (W6g §12); a família (b) (W5g §37–38).
5. A suíte no fecho da linha (não corre desde o W6g §56).
