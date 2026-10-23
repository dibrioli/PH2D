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
