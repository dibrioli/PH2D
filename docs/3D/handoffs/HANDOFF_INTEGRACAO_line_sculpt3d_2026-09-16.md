# HANDOFF de INTEGRAÇÃO — `line/sculpt3d`, 2026-09-16

> **Continuação de
> [`HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-15.md`](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-15.md)**
> (que fechou no §47, o Box Trim aprovado). A numeração **continua** (§48 em diante).
>
> **A jornada:** o dono aprovou o Box Trim e pediu o pincel que *«faz o trim esfregando o pincel
> como massinha»*; o smoke dele devolveu três relatos — *«sem undo/redo»*, *«o radius máximo
> permitido é pouco»* e *«resultado inferior ao blender para produzir superfícies planas»* — e a
> dica *«o melhor jeito de ver o efeito é com o pincel bem grande, do tamanho da peça»*.
>
> ⛔ **Nada integrado, nada enviado** (`CLAUDE.md` §0.7). Zero contadores partilhados: nem
> `PROJECT_SCHEMA`, nem `VEC_SCENE_SCHEMA`, nem `FLIP_SCHEMA`, nem `FIELD_DOC_VERSION`, nem os
> registos de componentes; zero contrato congelado; zero ADR; zero pacote externo.
> ⚠️ **UMA mudança foundational, aditiva:** o `ph2d-editor-core` ganha a **ligação curva**
> pista↔número (§50.3) — nenhuma assinatura pública muda, e a identidade é byte-idêntica.

---

## §48 — ⭐⭐⭐ O PINCEL DE PLANO (`Verb::Plane`, 34.º verbo) — clean-room do alvo, cena `=47`

**O que é:** esfregar apara o relevo contra o plano que a própria pegada ajusta — as cristas
descem, os vales ficam (tectos `1/0`), ou o contrário. Espec atestada:
[`SPEC_pincel_de_plano.md`](../cleanroom/SPEC_pincel_de_plano.md) (R-pré, 4.ª passagem), com as
cinco erratas curadas pela janela I antes da primeira linha de código (`9939f100d`).

### §48.1 — A lei, e o que a paridade mediu

| população | barra | pior medido |
|---|---|---|
| **19** fixturas de um dab efectivo (G-1) | `1e-6` | **`8,941e-08`** |
| **23** traços inteiros (G-1b — a dívida que a espec nomeava, **paga**) | `1e-6` | **`5,960e-08`** |

⭐ As nossas normais de vértice desviam `0,034°` na crista e `0,445°` nas bossas da malha do
oráculo e a saída **ainda** bate — a lei da §2.1 é robusta a isso por construção (média pesada).

### §48.2 — Quatro leis que só o corpus deu

- **A consulta tem de cobrir os raios de AMOSTRAGEM**, não só a pegada: `lei_area20` desviava
  `1,703e-1` até o factor passar a `max(1 + 0,75 + |deslocamento|, frac_normal, frac_área)`
  (`tectos_query_factor`).
- **O `from_live` fica PREGADO a `true`** (`brush_verb_defaults.rs`): nesta lei o interruptor de
  acumular decide **de que superfície o plano é lido** (porta `le_a_superficie_viva`), e mais
  nada. Preso à coluna, as cadeias de 4 e 8 dabs desviavam `1,634e-1`/`3,022e-1` com a de 2 certa
  — *o primeiro dab estava certo e o segundo já não*.
- **A força NÃO passa pelo perfil `S`** — o `RefMode::declares` é uma **lista negra**, e um verbo
  novo sem perfil caía no slider cru (força linear: `2×` a meio slider). O `Plane` entrou nela, e
  o censo novo `o_censo_das_referencias_que_ninguem_declara` lista os que ficam de fora com o
  nome: **Sharpen**, Cloth, Pose, Boundary, Density, Erase/Smear Displacement, Scene Project, Box
  Trim — ⚠️ o **Sharpen** é dívida nova nomeada (a força dele é linear).
- **O primeiro dab é INERTE** (falta a direcção do traço) e um bit trancado mantém um traço parado
  vivo; o arnês de fecho dava **um** dab a cada verbo e lia o pincel morto ⇒ porta
  `Verb::exige_esfregar` + `arma_o_traco` nos dois arneses.

### §48.3 — ⚠️ Divergência DECLARADA e dívidas

- **Inversão `Afastar`** (o sinal troca): `1,007e-1` contra o oráculo, com gate a medir o número
  (`o_afastar_e_uma_divergencia_declarada_com_numero`). A `TrocarTectos` é **exacta**.
- ⏳ Por escrever: G-5 (o discriminante), G-6, G-12, G-13, G-14; a pegada projectada; a máscara
  no arnês de bancada. ⚠️ **G-12/G-14 dependem dos estabilizadores**, que são o §51.
- O verbo shipa **só com chip** (`CHIP_ONLY` no gate do teclado): `25` das `26` letras estão
  tomadas, medido.

**Smoke:** `PH2D_SCULPT3D_SMOKE=47` (bossas em `40 k` triângulos, roteiro de 7 passos).

---

## §49 — ⛔⛔ *«sem undo/redo»*: a ferramenta VECTORIAL em mãos matava as teclas da escultura

⭐ **Não era do pincel.** O `~/.ph2d/layout.txt` do dono tem `active=vector`; a ferramenta
vectorial é reactivada no **quadro 1** (o `move` de omissão é o do quadro 0) e, com o barro na
tela, o **ponteiro** ia para a escultura (esfregar funcionava) enquanto **todas as teclas**
morriam — o `sculpt3d_keys_dead_reason` respondia *«a ferramenta Motion/Vector está EM MÃOS»*.
⚠️ Os dois gates do traço (pelas funções e pelas portas de ponteiro/teclado da família) passavam
**verdes com o defeito vivo**: ele vivia entre o evento do `winit` e a porta da família.

### §49.1 — O instrumento, e o que ele mediu na app real

`PH2D_SCULPT3D_UNDO_PROBE=1` com a `=47` — o roteiro do smoke (crase, chip, esfregar, arrastar a
pista, digitar num campo, esfregar, `Ctrl+Z`×2, `Ctrl+Shift+Z`×2) pelo teclado e ponteiro reais:

| | `ferramenta` | `teclas` | `Ctrl+Z` |
|---|---|---|---|
| antes | `vector` do quadro 1 em diante | **mortas** | `undo=2` fica `2`, a malha não mexe |
| depois | `vector` até ao 1.º esfregão, `move` a seguir | vivas | `2→1→0`, a malha volta ao valor **original ao dígito**; o refazer devolve os dois |

### §49.2 — A cura, e onde ela mora

- **4.ª linha da tabela do dono do canvas** (`ph2d_app_field3d::mode`): *um gesto da escultura
  no canvas devolve a ferramenta em mãos à de omissão*, partilhando o re-baseline com o modelador
  (`larga_para_a_neutra`); três gates da lei.
- A shell chama-a **só depois** de a família tomar o aperto e **só com o barro na tela**.
- ⭐ **O `Ctrl+Z` recusado DIZ porquê** — `keys_delete::queixa_do_desfazer`, pura e gateada; o
  teclado da família passou a receber a **razão** das teclas mortas em vez do `bool`.

### §49.3 — ⛔ As duas catracas que a cura acordou, curadas por MOVER

- `the_shell_only_shrinks` (+168): a sonda inteira na shell custava 187 linhas ⇒ o roteiro e a
  leitura vivem em `ph2d_app_sculpt3d::sonda_undo` (com gate de botão/tecla nunca presos), e a
  shell fica com o **executor de 42 linhas**. Os três gates de fonte do elo de shell vivem em
  `host_contract_tests::a_sculpt_gesture_releases_the_tool_in_hand` — o cabeçalho daquele
  ficheiro já mandava que vivessem lá. ⚠️ **A catraca conta também `shells/desktop/tests/`**: o
  gate novo lá dentro, sozinho, deixava a shell `5` linhas acima do tecto.
- `undo_tests.rs` `817/700`: os dois gates do plano partiram-se para `undo_plano_tests.rs`.

Prova de mutação **7 de 7**. ⚠️ **O arnês de mutação mentia antes**: um teste que falha imprime
`0 passed`, e o detector de «não correu nada» lia isso como vazio ⇒ ele passou a SOMAR
`passed + failed`.

---

## §50 — ⭐⭐⭐ *«o radius máximo permitido é pouco»*: o tecto é a DIAGONAL da vista

### §50.1 — O tecto antigo era uma opinião, e a medição do custo

O raio parava em `1/8` da altura da vista, e o doc dizia de que era: *«acima de meio modelo o
pincel é um deformador global, que é outra ferramenta»*. ⛔ **Não é um recurso.** Na vista de
omissão a peça ocupa `49 %` da altura ⇒ o tecto dava **metade do raio da peça** (`135` px a
1080 linhas contra `500` na pista do alvo). O custo, medido pela porta do produto
([`mede_o_raio_do_pincel`](../../../crates/ph2d-sculpt3d/tests/it/mede_o_raio_do_pincel.rs),
`--release`, `load 5`):

| triângulos | verbo | raio `0,25` | raio `1,0` | **peça inteira** (`2,5`) | `4,0` |
|---|---|---|---|---|---|
| `39 480` (a `=47`) | Draw | `0,16 ms` | `1,19` | **`2,76`** | `2,54` |
| `39 480` | Plane | `0,21` | `1,22` | **`1,40`** | `1,55` |
| `398 724` | Draw | `0,51` | `11,65` | **`27,8`** | `26,9` |
| `2 998 800` | Draw | `5,00` | `101,8` | **`256,8`** | `278,0` |

⭐ **~0,14 µs por vértice tocado, e SATURA na malha** (gate: a contagem acima do diâmetro não
cresce). E o espaçamento é proporcional ao raio ⇒ um pincel `8×` maior deposita `8×` menos dabs.
⇒ **o recurso que aperta é o ECRÃ**: com o cursor em qualquer ponto, um anel do tamanho da
diagonal contém a vista inteira, e acima dele só se acrescenta geometria que o artista não vê.
⚠️ O preço de uma malha pesada é o da **MALHA** (o filtro de malha inteira paga o mesmo), e a
saída dele é o K1 (`docs/3D/03.5`), não um tecto que esconda a peça.

### §50.2 — A cura

- `rulers::radius_ceiling_px(w, h)` = a diagonal (`RADIUS_MAX_FRAC_OF_DIAGONAL = 1`), com gate
  puro (`o_pincel_pode_ter_o_tamanho_da_peca`) e o elo na cena com GPU
  (`o_raio_da_cena_chega_a_diagonal_da_vista`).
- **O anel do cursor continua redondo:** os `48` segmentos fixos davam facetas de `~6 px` num
  anel de `2 779 px` ⇒ a contagem é derivada da **flecha** (`0,3 px` — a que os 48 davam no tecto
  antigo, o anel que o dono via redondo), entre `48` e `512`.
- A pista do painel vai a **`5 000` px** (≥ o digitável do alvo e ≥ a diagonal de um 4K).

### §50.3 — ⭐ A pista CÚBICA, e a extensão foundational que ela pediu

Uma pista **linear** até milhares punha o pincel de 50 px a `1 %` da trilha. ⇒ **ligação curva**
no `ph2d-editor-core` (`state/slider_curve.rs`): `número = offset + scale · pista^curva`.

- ⚠️ **Aditiva:** o `linked_slider_mapping` público continua `(scale, offset)` (sete crates o
  leem nos gates); a curva lê-se por `linked_slider_curve`, e `curva = 1` salta o `powf` —
  **byte-idêntico**. As projecções são UMA porta (`pista_para_fracao` / `fracao_para_pista`) que
  o store e o painel chamam, e a saturação do espelho continua a decidir-se na fracção LINEAR.
- **No painel a curva é DERIVADA da faixa** (`Row::curva`): piso positivo e três ordens de
  grandeza ⇒ cúbica; hoje só o raio (a razão seguinte é `200`). ⛔ Um campo por linha seria
  `58` literais a dizer «linear» e o `rows.rs` estava a `600/600`.
- Pincel de omissão a `~21 %` da pista, o tecto antigo a `~29 %`, a peça inteira a `~37 %`.

Prova de mutação **10 de 10** — ⚠️ a 1.ª ronda deu **9**: a curva sobre um mapa afim
**identidade** era descartada em silêncio, e o gate só usava a faixa do raio.
