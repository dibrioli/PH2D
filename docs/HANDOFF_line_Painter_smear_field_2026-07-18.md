# HANDOFF — o Smear virou um campo (2026-07-18)

> Sucessor do [`HANDOFF_line_Painter_TAKEOVER_2026-07-18.md`](HANDOFF_line_Painter_TAKEOVER_2026-07-18.md).
> **A fila dele fechou inteira** — as três primeiras tarefas, duas delas por MEDIÇÃO em vez de código:
>
> * **#1 o bug do Smear** — a lei do transporte estava errada (produto sobre a lista de dabs) e foi
>   trocada por composição de mapas (§2). **Smoke aprovado pelo Enio.** A agulha que sobrava no pincel
>   padrão é o **Hardness**, e fica: dureza É o controle de largura da esfregada (§4).
> * **#2 o engasgo de montagem** — era o fork SERIAL do plano canvas-inteiro; agora paralelo (§9.3).
> * **#3 Sculpt na GPU** — **dissolvido por medição**: o número que o justificava era a luz da CPU sendo
>   cobrada do sculpt por um gate desatualizado. Todo kernel já está sob o alvo (§9).
>
> Se você só vai ler uma seção, leia o **§9** — é onde uma ordem virou "não construa isto", e por quê.

## 1. Estado da linha

| | |
|---|---|
| Branch | `line/Painter`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| HEAD | `git log --oneline main..HEAD` (um commit não pode citar o próprio hash) |
| Ahead of `main` | 3 herdados do takeover + os desta jornada (§9 lista o que são) |
| Árvore | limpa · `check --workspace --all-targets` 0 · `clippy --workspace --all-targets` 0 · `fmt` aplicado |
| Suítes | **workspace 7658 passed / 0 failed** (tool-painter 724 · painter-brush 256) |
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

## 4. ✅ RESOLVIDO no smoke (2026-07-18): a agulha é o Hardness, e fica

**Enio smokou e aprovou.** A saída escolhida foi a terceira das três abaixo: **Hardness É o controle de
largura da esfregada**, nenhum default muda. O texto abaixo fica como o registro da medição, porque ele é
a razão de a decisão ter sido essa — e porque quem mexer no peso do dab no futuro precisa dela.

**Com o pincel PADRÃO da faca o produto desenha uma AGULHA.** Renderizado, não deduzido:
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
que o `walk_dab` responde para três consumidores. **O Enio escolheu a (3)** — nada a fazer no código.

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
| ~~1~~ | ~~A agulha residual do Smear~~ | **FECHADO** no smoke de 2026-07-18: Hardness é o controle de largura (§4) |
| 2 | Engasgo de montagem em tela grande | Medido, não investigado: 8,8 ms @2048 vs 17–21 @4096 na montagem da sessão de sculpt |
| ~~3~~ | ~~Sculpt na GPU~~ | **DISSOLVIDO por medição** — ver §9. Todo kernel já está sob o alvo |
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

---

## 9. O Sculpt na GPU foi DISSOLVIDO por medição — e o que apareceu no lugar

Ordem do Enio: *"#3 — Sculpt na GPU"*. Medi antes de desenhar (DIRETIVA §5: kill-criterion antes do build)
e **o número que justificava o port não existia.**

### 9.1 O gate cobrava do sculpt a luz da CPU

`sculpt_perf_kill_criterion` cronometrava `on_canvas_pointer` + `take_preview_arc()` num `Instant` só,
comentado *"what a frame really costs"*. Era verdade quando foi escrito e deixou de ser em **2026-07-18**,
quando a luz foi para a GPU: neste harness headless não há dispositivo, então o `take_preview_arc`
**compõe e ilumina na CPU — um caminho que o produto não roda**.

Separando os dois (`3352e39f`):

| verbo | KERNEL @2048 / @4096 | PREVIEW (CPU; GPU no produto) | SET-UP antes → depois |
|---|---|---|---|
| SMOOTH | **1,20 / 1,18** | 2,19 / 2,16 | 18,9 → 10,5 → **5,5** (§9.5) |
| SCRAPE | **0,67 / 0,67** | 2,30 / 2,31 | 12,9 → **4,8** |
| INFLATE | **3,54 / 3,70** | 2,29 / 2,39 | 15,1 → **5,9** |

**Todo verbo já está sob o alvo de 4.** O *"Inflate a 6,4 contra alvo 4"* era ~2,3 ms de luz de CPU
somados ao kernel. Não era só generoso: gastava a maior parte do orçamento com trabalho de outro, então
uma regressão real de kernel tinha ~2 ms a menos de folga do que o número anunciava.

Os kernels também são **planos na tela** — limitados pela pegada do pincel (raio 20 custa 4-6× menos que
raio 100 no mesmo canvas). ⚠️ Eu afirmei o contrário num turno intermediário ("o Inflate escala, 3,77 →
7,32") e **estava errado**: era a deriva desta máquina. O handoff anterior avisa disso e eu caí mesmo
assim — **prefira gate CONTADO ou de RAZÃO a wall-clock**.

### 9.2 Três bloqueios que o port teria encontrado

Valem se alguém retomar a ideia:

1. **O caminho quente já é paralelo** — a bola do Inflate foi de 44 ms serial → 3 ms com rayon por linhas.
2. **`sculpt_close::PAR_MIN`** existe porque abaixo de ~262k texels o rayon deixava o Inflate **mais
   lento**; o `cr` de uma dab tem ~62k. Um dispatch de GPU tem overhead **maior** que um fork de rayon —
   a mesma medição que reprovou o rayon nessa escala reprova a GPU.
3. **O argmax da advecção depende de ordenação ESTÁVEL** (`sort_by`, `sculpt_offset.rs`): num platô
   uniforme toda fonte equidistante empata no float e a ordem de iteração *é* a resposta. Ordem instável
   = matéria de outra fonte = **outra cor**. Uma redução paralela de GPU não tem ordem estável nenhuma.

E o resto é **fold**, não óptica (`plane_sum`, `amount`, `bank`, o escalar da onda, `locked_dir` — fatos
sequenciais sobre a lista de dabs). Pelo padrão da luz isso fica na CPU e sobe pronto; sobraria portar a
parte que já roda em 3 ms.

### 9.3 O que apareceu no lugar: o fork do plano (`050c2d80`)

A única coisa que escalava com a tela era a **montagem**. Causa medida: a sessão congela `pre` por `Arc`
(refcount, não cópia) e a primeira escrita chama `Arc::make_mut`, que vê o segundo dono e copia o plano
inteiro — **10,88 ms a 4096²**, que era praticamente toda a montagem.

A cópia é necessária (snapshot + buffer mutável = uma cópia). Fazê-la numa thread só não era. Porta única
`plane_fork::fork_par`, **compartilhada de propósito** — sculpt, Reshape e Smear pagavam a mesma cópia.

⚠️ **4× o dado custa 20× o tempo** (16,8 MB → 0,54 ms; 67,1 MB → 10,88 ms): o custo não é banda, é a
alocação nova com as faltas de página no primeiro toque. É por isso que o paralelo ganha *mais* a 4K.

⚠️ **O gate do caminho rápido é de RAZÃO, e tem de ser.** Os gates de correção não conseguem ver o ramo
paralelo — um fork é uma cópia, então ele é semanticamente idêntico por construção e nenhum valor,
ponteiro ou refcount difere. Um gate comportamental seria o caminho serial medido contra ele mesmo, verde
para sempre. Mutação: neutralizar o ramo leva a razão de **3,2× para 1,0× → RED**, e os dois gates de
correção seguem verdes sob ela, como a doc diz que devem.

### 9.4 O PEN-UP do Inflate: DIAGNOSTICADO, não consertado

O número: `Up(commit)` custa **7,4–7,9 ms** no Inflate contra 1,6–3,2 nos irmãos; o preview do mesmo frame
é uniforme (~2,9) em todos. Está no **commit**, não no desenho.

**A causa, medida.** A primeira hipótese era a cauda do stabilizer (o Up descarrega ~1 raio de trajeto
segurado). **Refutada:** a razão `Up / move` é **constante em ~1,9–2,1×** para raio 100, 40 e 20. Se fosse
a cauda ela variaria com o raio (cauda ≈ raio, move = 40 px fixo). Constante em 2× é a assinatura de
renderizar **duas vezes**, e é isso mesmo — `stroke_lifecycle::paint_end` chama `stamp_dabs` duas vezes:

```rust
self.paint_extend(ev);              // stamp #1 → render_sculpt(rect_A)
if let Some(mut stroke) = self.paint.stroke.take() {
    stroke.finish(&mut dabs);
    self.stamp_dabs(&dabs);         // stamp #2 → render_sculpt(rect_B)
```

Os dabs são todos legítimos (o `finish` produz a cauda de verdade). O que é desperdício é o **render**:
`rect_A` e `rect_B` ficam ambos no fim do traço e se sobrepõem quase inteiramente, e para o Inflate o
render é a bola `O(ρ²)` — a parte cara.

**Por que consertar seria byte-idêntico** (o argumento, para quem for fazer): o render do sculpt é
**idempotente** — ele re-deriva `h = pre + k·Δ` do `pre` congelado sobre o rect, não acumula. Renderizar
uma vez sobre a UNIÃO dá exatamente o mesmo resultado, e o caso que parece perigoso não existe: um texel
só recebe `amount` do batch 2 se estiver no rect tocado do batch 2, logo está na união. Ganho esperado:
metade do pen-up do Inflate (~3,7 ms), uma vez por traço.

⚠️ **Não faça isso fundindo os dois `stamp_dabs` numa chamada só.** As fronteiras de batch são
significativas para outros sistemas (os grupos do `DabRng`, a emenda da cápsula de altura via
`last_height_center`, o `last_smear_pos`); fundir batches muda o que eles veem. O que precisa ser adiado é
só o **render** do sculpt — acumular o rect e descarregar uma vez por evento de ponteiro.

**Por que parei aqui:** o ganho é ~3,7 ms uma vez por traço, e a mudança mexe no *agendamento* do render,
que é onde as regressões deste módulo historicamente se escondem (o re-stamp por frame dos shape editors
passa pelo mesmo caminho). Vale uma passagem com cabeça fresca e gate próprio, não o fim de uma jornada
longa.

### 9.5 A montagem do SMOOTH — FECHADA (a hipótese anotada estava errada)

Esta seção dizia: *"a montagem do SMOOTH ainda é a mais cara (10,5 ms) porque ele aloca o memo do blur
além do `amount`"*. **A causa estava errada, e os próprios números a desmentiam:**

- `vec![0.0; n]` de `f32` cai no `alloc_zeroed` do std (páginas zeradas por mmap) — alocar é quase grátis.
  O que custa é o **toque**, e o toque é limitado pela pegada do pincel.
- A montagem do SMOOTH media **10,02 @2048 e 9,70 @4096**: **plana na tela**. Uma alocação canvas-inteira
  quadruplicaria de 2048 para 4096. Um custo que não escala com a tela não é um custo de tela.

O experimento que decidiu: **reordenar os verbos no gate.** Se o custo fosse aquecimento do processo (pool
do rayon, arena do alocador), ele ficaria com quem mede primeiro. Medindo o SCRAPE primeiro, ele custou
4,80 / 4,71 — **não herdou nada** — e o SMOOTH seguiu pagando 11,47 mesmo em segundo. Era o verbo.

**Causa real:** o SMOOTH é o único verbo com **memo de blur**, e a primeira batelada descobre a pegada
inteira de uma vez (pincel de raio 100 ⇒ ~16 tiles) enquanto cada move seguinte descobre um ou dois. Esses
16 tiles eram borrados **um de cada vez**, cada um através de uma janela crescida de `r` em cada lado.

**Fix:** os tiles rodam concorrentes. Isso não é uma afirmação nova a defender — é o **próprio argumento de
byte-identidade do módulo, lido para a frente**: cada tile é função pura do `pre` congelado e da própria
janela, nenhum lê a saída de outro, e as regiões de escrita são disjuntas. `blur_tile` virou função livre
sobre `&[f32]` justamente para que a *assinatura* seja o argumento de independência — não existe `&mut self`
por onde um tile pudesse ver outro. Abaixo de `PAR_MIN_TILES` (4) segue serial: o regime permanente
descobre 1-2 tiles por move e não pode ficar mais lento para o primeiro move ficar mais rápido.

| | antes | depois |
|---|---|---|
| SMOOTH SET-UP @2048 / @4096 | 10,02 / 9,70 | **5,87 / 5,37** · **5,61 / 4,97** (2 runs) |
| SCRAPE SET-UP (referência) | 4,80 / 4,71 | 5,69 / 4,53 · 5,13 / 4,57 |
| SMOOTH kernel (regime permanente) | 1,08 / 1,09 | 0,72-0,86 (não regrediu) |

O excesso *específico do SMOOTH* acabou: a montagem dele agora é indistinguível da do SCRAPE. Os ~5 ms que
sobram são a montagem que **todo** verbo paga (o fork do plano + primeiro toque), já atacada em `050c2d80`.

⚠️ **O gate cobria o caminho paralelo por acidente** (tela de 200 px = 16 tiles). Se alguém subisse o
limiar, ele cairia no serial e continuaria **verde, medindo o caminho antigo contra si mesmo** — a
armadilha do ADR-0120. Agora `the_tile_memo_is_byte_identical_to_a_whole_canvas_blur` roda os **dois**
lados do limiar contra o mesmo oráculo (um blur de canvas inteiro, que não sabe de tiles nem de threads) e
**afirma o cruzamento**. 3 mutações, 3 sangram: janela `r-1` (1023 texels) · um tile perdido no scatter
(4096 = um tile exato) · limiar para 32 (o guard de fixture dispara).

`sculpt_tests.rs` estourou o teto de LOC com isso ⇒ split `sculpt_tests/memo.rs` (não allowlist).

### 9.6 O pen-up do Inflate — MEDIDO DE NOVO E RECUSADO DE NOVO

Re-medido no fim (§9.4 tinha deixado em aberto): pen-up **10,9 / 10,3 ms** para o Inflate contra
**4,1-4,4** dos outros verbos. A causa é a do §9.4 e ela se confirma — `paint_end` carimba duas vezes
(`paint_extend`, depois os dabs de `finish`, que **é** só a cauda: `finish` abre com `out.clear()`), e cada
carimbada termina no seu próprio `render_sculpt`. Achado extra: o **mesmo padrão está no heartbeat** —
`on_tick` faz `stamp_dabs` e, com o pincel PARADO, `settle` + `stamp_dabs` de novo.

**Continua não valendo, e agora com o número na mão:** o ganho é ~3,7 ms *uma vez por traço*, e 10,9 ms
**não atravessa um frame de 60 fps** — o artista não sente. O custo do outro lado é construir contador no
PRODUTO (não há mecanismo de contagem compartilhado nesta crate) e mexer em *agendamento* de render, que é
onde as regressões deste módulo historicamente se escondem. Medir de novo confirmou a recusa em vez de
derrubá-la; se alguém retomar, o desenho está no §9.4 (adiar **só** o render — as bateladas de dabs têm de
continuar duas, pelo `DabRng`, pelo `last_height_center` e pelo `last_smear_pos`).

### 9.7 A perf do WARP — GATEADA (era o buraco do módulo)

`CLAUDE.md` carregava *"⚠️ perf do Deform não é gateada (nunca foi) — 3 amostragens/texel a mais no bbox
quando há relevo"* desde o W4. Aviso em prosa não é orçamento: nada media, então nada podia notar mudança.
E a superfície ficou **mais carregada** do que quando a nota foi escrita — o Smear desta linha foi
reconstruído sobre o mesmo campo e re-renderiza pela mesma porta (`warp_render_relief`), então uma
regressão ali atinge duas ferramentas.

`warp/perf_tests.rs::warp_perf_kill_criterion` (radius 100, o pior caso):

| | TOTAL @2048 / @4096 | da qual ADVECÇÃO do relevo | tela plana |
|---|---|---|---|
| DEFORM | 4,18 / 4,14 ms | **2,52 (60%)** | 1,66 |
| SMEAR | 3,28 / 3,30 ms | 1,71 (52%) | 1,57 |

Ambos sob o kill 8 do irmão sculpt e **planos na tela**. O número que não existia: **a advecção é a metade
MAIOR do Deform** — "3 amostragens/texel a mais" *mais que dobra* a ferramenta. Não é regressão, é o preço
honesto do corpo viajar com os pixels; mas agora é um preço medido.

**Duas barras, e a ORDEM é load-bearing.** Uma razão entre as duas telas (o caminho é limitado pela
pegada ⇒ quadruplicar a tela não pode mover o custo) e um kill de wall-clock. Escrevi o kill dentro do laço
de tamanho e a razão depois — e assim **a razão nunca conseguiria ficar vermelha**: qualquer regressão
canvas-proporcional grande o bastante para dobrar a razão estoura 8 ms na tela MAIOR *por aritmética*, então
o kill dispararia sempre antes. Verde por construção. Com a razão afirmada primeiro, cada barra tem uma
pergunta e sangra sozinha:

- plano inteiro em vez da janela ⇒ **razão 4,86×** (dispara, com o diagnóstico certo: *"algo começou a
  percorrer o plano"*);
- janela crescida 400 px **constantes** ⇒ razão **1,00×, passa**; o **kill** dispara em 47 ms.

### 9.8 Conserve p/ Flatten/Fill — LANDOU (ordem do Enio; era "decisão de design")

**A decisão:** a lei é UMA, e o Scrape sempre foi caso particular dela —

> **Conserve = a variação LÍQUIDA de volume do traço é zero; o aro liquida a diferença com o SINAL do
> ledger.**

Scrape/Chisel só removem ⇒ ledger sempre negativo ⇒ aro sobe (foi por isso que o 1º corte pôde cravar uma
crista sem perceber que estava supondo algo). Fill/Clay adicionam ⇒ ledger positivo ⇒ **o aro afunda: um
fosso**. Encher uma cova arrastando a tinta ao redor deixa fosso, exatamente como raspar um canal deixa
crista.

Custo estrutural: quase nada, e por bons motivos. A mordida parou de codificar *a expressão do Scrape* e
passou a **perguntar ao verbo** (`Travel::{Down,Up,Both}`, espelho exato do `match` do render);
`bank_dab_push` **já era transparente ao sinal** (`plane[i] += k·scale`); e o painel **já perguntava ao
tool** (`sculpt_conserves`) ⇒ **zero mudança de painel**. Estender `conserves()` bastou para o checkbox
nascer nos cards novos.

#### ⚠️ A medição mudou o que o flag SIGNIFICA no Flatten

Medido ANTES de construir (volume líquido de um traço, `loads·px²`):

| | offset 0 | +0,25 | +0,5 | −0,25 |
|---|---|---|---|---|
| FLATTEN | **+0,7 (+1,2%)** | +741,7 | +1482,7 | −740,3 |
| FILL | +59,7 | +741,7 | +1482,7 | +0,0 |

**No centro do Offset o Flatten já é conservativo, e não por sorte:** o ajuste de mínimos quadrados passa
pelo centroide ponderado, então `Σ w·(plano − h) = 0` **por construção** — o que ele tira dos picos já está
nos vales. O resíduo é só o descasamento entre os pesos do fit e o `k = min(amount,1)` do render.

Logo, no Flatten este flag **não é** *"pare de deletar tinta"*: é **o contrapeso do Offset**. O Offset é o
botão de volume (fora do centro cria ou destrói **12×** toda a redistribuição) e o Conserve é o que faz o
botão mover tinta em vez de conjurá-la. É por isso que vale oferecê-lo num verbo neutro em repouso: ele é
vivo exatamente onde o verbo deixa de ser neutro.

#### ⚠️ O bug que nenhum ledger pegaria

Com a lei implementada o ledger fechava em **±0,0 em todos os casos** — e estava mentindo. O aro
**prefere texels que o traço não trabalhou** (`1 − paint`, para não empilhar a crista no próprio canal), e
invertido o sinal esse mesmo fator guia o fosso **para fora da tinta**. Medido num depósito real:
**83% do saque caía em tela nua**, com o ledger em ±0,0. Relevo sob cobertura zero **não renderiza** ⇒
ledger verde sobre mentira visível, que é pior que ledger desequilibrado, porque nada reclama.

Cura: um saque é pesado pelo que está de fato lá — **`Supply { relief, cover }`**, a grandeza que a LUZ
integra (`altura × cobertura`). ⚠️ **Relevo sozinho não basta, e isso foi medido**: o depósito espalha
altura num halo mais largo que a tinta, e pesar por ele ainda deixava **56%** no invisível.
**83% → 56% → 0%**, com o ledger fechando o tempo todo.

O fixture teve de mudar junto: `sculpt_canvas` põe **cobertura 255 em toda parte**, então nele "fora da
tinta" não existe e um fosso no vazio é indistinguível de um na tinta — *um fixture que não contém o
fenômeno não pode gateá-lo*. Os gates novos usam um **depósito REAL** com bordas.

Gates (3 novos, 4 mutações, 4 sangram): ledger do Fill · **o fosso só toma tinta de pé** · o Offset como
botão de volume e o Conserve como contrapeso. Os 4 gates velhos de Scrape ficam verdes — o depósito é
byte-intocado (`supply` só é consultado quando `displaced < 0`).

**Aberto, herdado e inalterado:** o banco ainda normaliza por dab (cada dab divide o próprio aro), o
resíduo conhecido de 0,0286 do §7 da fila. O fosso herda exatamente a mesma imperfeição, pelo mesmo
motivo, e a cura é a mesma para os dois.

### 9.9 A cura do BANCO — FECHADA POR MEDIÇÃO (o item não existia)

O último item da fila dizia: *"cada dab ainda normaliza o próprio aro = um produto sobre a lista de dabs;
residual 0,0286"*. Fui curá-lo e a medição o desmontou em dois passos.

**1. Não é dependência de espaçamento.** A razão da ondulação entre 2 px e 1 px é **~1,0×**
(0,91–1,21 em todos os `lat`, nos dois falloffs). Uma corrugação por-dab quase dobraria ao dobrar o
espaçamento. **O aro já é função do caminho** — a propriedade que a "cura" ia comprar já está paga.

**2. O número vinha de ONDE foi medido.** A janela `80..110` atravessa o **fim** do traço (o traço acaba em
x=120), onde a crista rampa de altura cheia até zero — e `trench_ripple` é **pico-a-pico**, então sobre uma
rampa ele reporta a rampa, não uma corrugação.

| | janela 80..110 | **meio 82..98** | fim 100..118 |
|---|---|---|---|
| Sphere lat+12 | 0,0894 | **0,0180** | 0,1585 |
| Smooth lat+9 | 0,0170 | **0,0017** | 0,1084 |

No meio do traço o aro é essencialmente uniforme (Smooth: 0,9% da própria crista).

**O que ficou no lugar da cura:** `the_rim_is_a_fact_of_the_path_not_of_the_dab_spacing` — irmão do gate do
canal, medindo o **aro** (lat 9 e 12, fora do pincel) **mid-stroke**, com guard de presença (*"há pilha
aqui para medir?"*). Ele pina a propriedade verdadeira no lugar onde ela é honestamente verificável, para
o próximo leitor não re-derivar o item falso a partir da nota. A mutação que o faz sangrar é **o banco
compositar em vez de acumular** (`plane[i]*(1−k) + k·scale`) — que É exatamente a doença que o item
descrevia — e ela quebra a asserção de espaçamento (−0,2417 a 1px contra −0,3843 a 2px).

⚠️ **A lição, e ela é a mesma da §9.5:** um resíduo medido no lugar errado vira trabalho planejado, e ficou
planejado por um ano. Duas vezes nesta jornada uma nota de trabalho aberto descreveu uma *causa* que os
próprios números dela desmentiam.

### 9.10 Também sobra

- Nada medido em aberto neste eixo. Os kernels estão todos sob o alvo de 4 e planos na tela; a montagem
  restante é comum a todos os verbos e é o fork do plano.
