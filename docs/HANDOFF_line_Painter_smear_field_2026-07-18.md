# HANDOFF — o Smear virou um campo (2026-07-18)

> Sucessor do [`HANDOFF_line_Painter_TAKEOVER_2026-07-18.md`](HANDOFF_line_Painter_TAKEOVER_2026-07-18.md).
> A tarefa #1 dele (o bug do Smear) foi atacada: **a lei do transporte está consertada e gateada**, e
> **o sintoma que o Enio fotografou ainda aparece no pincel PADRÃO** — por um motivo diferente, medido e
> renderizado. Leia o §4 antes de tocar em qualquer coisa: é onde está a parte que sobrou, e ela é uma
> **decisão de produto**, não um bug à espera de conserto.

## 1. Estado da linha

| | |
|---|---|
| Branch | `line/Painter`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| HEAD | `14deb079` |
| Ahead of `main` | **4 commits** |
| Árvore | limpa · `check --workspace --all-targets` 0 · `clippy --workspace --all-targets` 0 · `fmt` aplicado |
| Suítes | **workspace 7656 passed / 0 failed** (tool-painter 721 · painter-brush 256) |
| Perf | knife **4,57 ms/move @2048² · 5,50 @4096²** (kill 8) — gate novo `smear_perf_kill_criterion` |

⚠️ **Modo L.** Todo path absoluto com `/Worktrees/line-Painter/`; todo comando que muta abre com
`cd <worktree> &&`. **Integração e ship são ordem EXPLÍCITA do Enio.** Esta linha fecha, entrega e PARA.

---

## 2. O que estava errado, e a lei que fechou

`plow_dab_height` / `smear_dab` faziam, por dab:

```rust
dst += (fonte_um_passo_atrás − dst) · w
```

O espaçamento do Smear é ~1 px, então um arrasto de 170 px são ~170 passos e o que sobrevive é `h·wⁿ` —
um **PRODUTO** sobre a lista de dabs. No eixo `t = 0 ⇒ w = 1` exato e nada decai; 6 px fora, `0,8¹⁵⁰ ≈ 0`.
Terceira vez que esta linha encontra a mesma doença (a mordida do bow wave e a cápsula do relevo).

### 2.1 ⛔ Somar deslocamento NÃO é a cura — foi medido e reprovado

A leitura óbvia do handoff anterior (*"o deslocamento tem de ser acumulado"*) leva a `disp[i] += step·w`.
**Está errado, e erra de um jeito que passa num arrasto curto.** Um texel só acumula enquanto o pincel
está por cima dele, então o deslocamento total que ele alcança é limitado por ~*diâmetro × peso médio* —
~20 px numa faca de 32 px, **por mais longe que você arraste**. Passado isso o render amostra a fonte
congelada num ponto que nunca foi pintado e **a trilha simplesmente PARA**.

Medido, com essa versão instalada: cor **e** relevo caíam a zero ~35 px depois da crista, num arrasto de
160 px. Pior que o filamento.

### 2.2 A lei certa: o mapa é COMPOSTO, porque uma esfregada é um REVEZAMENTO

O dab *k* entrega o conteúdo ao *k+1*, que entrega adiante: o que está perto do eixo continua debaixo do
pincel em movimento e viaja o **traço inteiro**; o que está fora é ultrapassado uma vez e fica para trás.
Isso é **composição de mapas**, não soma de offsets:

```text
φ_novo(p) = φ_velho(p − v(p))          v(p) = step · w(p)
```

— backtracking semi-Lagrangiano. Em forma de deslocamento (`φ(p) = p − disp(p)`):

```text
disp_novo(p) = v(p) + disp_velho(p − v(p))
```

**Continua sendo "acumule e aplique UMA vez sobre a fonte congelada", e é esse o ponto.** O que é
reamostrado repetidamente é o **MAPA** — campo de coordenadas suave, localmente quase afim, que o bilinear
reproduz quase exato. A **IMAGEM** é reamostrada uma única vez, no fim. Reamostrar coordenada e reamostrar
figura não são a mesma operação, e a diferença entre elas é o módulo inteiro.

⚠️ Isto **não** é a tentativa (2) que o handoff anterior proibiu. Aquela reamostrava a **imagem** a cada
passo (`bilinear(src, p − step·w)` com `src` = resultado do passo anterior) e saía bit-idêntica ao lerp.
Esta reamostra o **mapa**. O handoff anterior já dizia a diferença; vale reler o corolário dele.

---

## 3. O que mudou (arquitetura)

| onde | o quê |
|---|---|
| `ph2d-painter-brush/src/smear_field.rs` **(novo)** | `accumulate_dab_smear` — **terceiro passageiro do `walk_dab`**, ao lado da intensidade do sculpt e do ajuste de plano. É por isso que Tiling / Symmetry / shape editors / pressão / Jitter / **Shape** / **Grain** continuam de graça: o risco nº 1 do handoff anterior morre por construção. `SmearOut` empacota saída+scratch como o `PlaneOut` do irmão. |
| `tool/paint/warp/session.rs` **(novo)** | A **SESSÃO** (4 planos congelados + `disp` + `affect_relief` + `relief_disp_scale`) saiu do `DeformState` e virou tipo com **dois donos**. `warp_render_relief` segue a **porta ÚNICA** — cor e corpo não podem discordar sobre para onde a tinta foi. |
| `tool/paint/smear_warp.rs` **(novo)** | A rota do Smear: abre sessão, acumula pela lista de dabs (com os offsets de Tiling), re-renderiza do congelado. |
| `impasto_plow.rs`, `plow_dab_height`, `smear_blit_stamp`, `smear_grain.rs` | **DELETADOS.** O `plow_dabs` era um transporte PARALELO com cadeia própria — exatamente as "duas portas para *para onde a tinta foi*" que o handoff proibia. Os outros três ficaram órfãos e diriam ao próximo leitor que a esfregada ainda lift-blenda. `smear_dab` **fica** (o `watercolor_smudge` ainda o usa). |

**Ciclo de vida:** Deform = sessão por SESSÃO (Reconstruct precisa do histórico) · Smear = por **TRAÇO**
(não tem Apply nem Reset, e o resultado de um traço é a base do seguinte). Sair do modo encerra — gateado.

**O `Plow` virou ESCALA da porta única** (`relief_disp_scale`), não um segundo campo: `1.0` (o default de
`e1fa546b`) = pigmento e corpo como uma substância; `0.0` = a faca antiga. Há UM mapa de deslocamento.

**Fold:** o Smear nunca dobrou **Flow** (`coverage × strength`) e o `walk_dab` dobra `coverage × flow ×
strength` ⇒ o chamador entrega um spec com `flow = 1.0`. Tornar um slider inerte vivo não era deste fix.

---

## 4. ⚠️ O QUE NÃO FECHOU — e por que a próxima pessoa não deve "consertar" sozinha

**Com o pincel PADRÃO da faca o produto ainda desenha uma AGULHA.** Renderizado, não deduzido:
`scratchpad/look/13b_smear_after.png` mostra a cápsula com um fio saindo — a foto do Enio.

**E não é a lei do transporte.** O pincel padrão do slot Smear é, medido pela sonda:

```
[smear-probe] SMEAR brush: r=10.0 hardness=0.00 falloff=Smooth strength=1.00 flow=1.00 spacing=0.050
```

— **o mais macio que existe**. Varrendo só a dureza, com todo o resto igual (larguras da trilha em texels,
medidas ao longo do arrasto):

| pincel | larguras da trilha |
|---|---|
| r=10, **hardness 1.0** | `20, 20, 20, 20, 20, 20, 20, 20` ← largura CHEIA da faca |
| r=10, hardness 0.5 | `14, 12, 12, 12, 12, 12, 10, 10` |
| r=10, **hardness 0.0** (o default) | `10, 6, 4, 2, 2, 2, 2, 2` ← a agulha |
| r=40, hardness 0.0 | `34, 28, 26, 22, 20, 18, 18, 16` |

A leitura física: num peso maximamente macio, `w ≈ 1` só numa linha fininha, e **só quem tem `w ≈ 1`
acompanha o pincel em movimento** — todo o resto fica para trás e é depositado perto de onde estava. A
trilha afunilando até um fio é o que *qualquer* transporte advectivo correto produz com esse peso; o kernel
antigo tinha a mesma forma, com a decadência multiplicativa por cima.

**Por que eu parei aqui em vez de mexer.** As saídas plausíveis são todas decisões de produto:

1. **Mudar o default do slot Smear** (hardness > 0). Um número que o Enio vê e sente, como o Plow foi.
2. **Remapear o peso do Smear para transporte** (ex.: `w' = w^γ` ou um platô), para uma borda macia
   continuar carregando massa. Muda o desenho de *toda* esfregada já feita.
3. **Aceitar** que faca macia = fio, e documentar que dureza é o controle de largura da esfregada.

Nenhuma é "o conserto óbvio", e a (2) reabre a pergunta *"o que o peso do dab significa no transporte"*
que o `walk_dab` responde para três consumidores. **Peça a ordem ao Enio, com esta tabela na mão.**

### 4.1 Como reproduzir em 30 segundos

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
PH2D_PUSH_LOOK_DIR=/tmp/look cargo test -p ph2d-host-desktop --release --bins \
  -- --ignored probe_push_render_and_look --nocapture
```

⚠️ Se `/tmp/look` sair vazio, o sandbox redirecionou `/tmp` — use um path do scratchpad da sessão.

A cena **13** é o gesto do Enio. A sonda agora imprime **duas** coisas que faltavam:

* `SMEAR brush: …` — **qual faca o app de fato balança** (era invisível, e é a peça que faltava).
* `trail WIDTH` — a **largura** da trilha, contada. ⚠️ A linha `across x=250` amostra **de 6 em 6 px**, e
  por isso **não distingue uma agulha de 1 texel de uma faixa de 11**: as duas leem `h0.00 · h… · h0.00`.
  Foi ela que fez o filamento parecer total. **Prefira a linha de WIDTH.**

---

## 5. Gates novos (e o que cada mutação mata)

| gate | mutação que sangra |
|---|---|
| `the_knife_carries_the_body_across_the_frontier_as_mass_not_a_filament` (secção transversal) | soma em vez de composição · plow 0 |
| `the_smear_trail_is_a_fact_of_the_path_not_the_dab_spacing` | soma em vez de composição |
| `the_knives_warp_session_does_not_outlive_its_stroke` | tirar o teardown do pen-up · tirar o braço de troca de modo |
| `a_second_smear_stroke_builds_on_the_first` | (nenhuma — ver abaixo) |
| `smear_field` ×4 (kernel) | soma em vez de composição · lerp por passo |
| `smear_perf_kill_criterion` | — |

**Mutações rodadas:** soma-em-vez-de-composição mata **3** gates; sem teardown de pen-up mata 1; sem braço
de troca de modo mata 1; porta ignorando a escala do Plow mata `no_other_paint_mode_touches_the_relief`.

⚠️ **Um sobrevivente, e ele é honesto:** tirar o teardown do pen-up deixa
`a_second_smear_stroke_builds_on_the_first` **VERDE**. Motivo real, não acidente de fixture: **um mapa de
warp COMPÕE, e composição é associativa** — uma sessão que erradamente atravessa dois traços reconstrói
quase a mesma figura. O doc do gate agora diz isso em voz alta e limita o que ele afirma. Escrevi um gate
maior (tinta de outra ferramenta entre dois traços) e **medi que ele não discrimina**: correto e mutante
davam `448 → 158` idênticos, porque o teardown do pen-up já cobre o caso e o que sobra é a esfregada
legítima diluindo o azul no vermelho. **Deletei em vez de shipar um gate que afirma o que não prova**
(a defesa em camadas do `feedback_layered_defenses_need_per_layer_gates`: duas guardas, mutar uma não sangra).

---

## 6. Duas armadilhas de FIXTURE que custaram tempo — não as repita

O gate anterior (`…as_far_as_it_carries_the_pigment`) estava verde sobre um produto vermelho por **dois**
motivos independentes, e os dois são sobre o fixture, não sobre o código:

1. **Cravava `hardness = 1.0`.** Disco duro ⇒ `w = 1` em toda a pegada ⇒ o produto `wⁿ` nunca decai ⇒
   **o filamento não pode se formar**. O fixture excluía exatamente o fenômeno que existia para pegar, e
   reportava 24 px de corpo sob 24 px de pigmento: nota cheia, num canvas onde o bug era inalcançável.
2. **Media relevo contra PIGMENTO.** A cor tem a estrutura idêntica e adoece igual — medido:
   `relief_w == pigment_w` **ao texel** em toda estação da trilha. **Razão entre dois doentes é verde por
   construção.** O oráculo agora é o **PINCEL** (largura absoluta), que é o que o artista tem direito a.

E uma terceira, do handoff anterior, que continua valendo e me pegou de novo por outro caminho: **dirija
pela porta REAL** (`set_paint_tool_mode`), nunca por `paint.paint_mode`. O poke pula o `switch_brush_slot`,
que é o que carrega o pincel PRÓPRIO do Smear — e o pincel próprio do Smear é justamente onde o problema
que sobrou mora.

---

## 7. Fila (herdada, com o item 1 atualizado)

| # | tarefa | gatilho / estado |
|---|---|---|
| **1** | **A agulha residual do Smear** (§4) | **Precisa de ORDEM do Enio** — 3 saídas, todas decisões de produto. Tabela de dureza pronta |
| 2 | Engasgo de montagem em tela grande | Medido, não investigado: 8,8 ms @2048 vs 17–21 @4096 na montagem da sessão de sculpt |
| 3 | Sculpt na GPU | Recomendação do handoff anterior; §0.0 do CLAUDE.md aponta para cá |
| 4 | Cache com chave de versão pros planos da luz GPU | Adiado de propósito; acorde se aparecer em profile |
| 5 | Conserve p/ Flatten/Fill | Decisão de design, precisa do Enio |
| 6 | Relevo do papel | **BARREIRA:** exige ordem nova do Enio |
| 7 | A cura do banco | Residual 0,0286, invisível no render. Baixo retorno |

## 8. Coisas que vão te economizar um ciclo

- **Capture a sessão ANTES do pen-up.** `close_stroke` libera o `disp`; li o campo depois do Up e vi
  zeros, e quase caí na conclusão de que o transporte não rodava. (`feedback_capture_stroke_session_before_pen_up`.)
- **Oráculo com passo grosso mente.** A linha `across x=250` da sonda amostra de 6 em 6 e não distingue
  1 texel de 11. Conte a largura.
- **`u8 + 20` estoura em papel branco** (255). Um gate meu morreu com "attempt to add with overflow" antes
  de chegar na asserção; alargue para `u16` ao comparar canais.
- **`str.replace` de `"mod X;"` casa dentro de `"pub mod X;"`** e deixa um `pub ` órfão que gruda na linha
  seguinte (`pub pub mod spec;`). Ancore no prefixo inteiro.
- Esta máquina degrada ~3× numa sessão longa; prefira **gate contado** a wall-clock.
