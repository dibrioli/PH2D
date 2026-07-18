# HANDOFF — assumindo a `line/Painter` (2026-07-18)

> Para o **agente que assume a linha**. Contém: o estado, **o bug aberto com duas tentativas falhas
> documentadas** (não as repita — a segunda é matematicamente impossível e eu perdi um ciclo nela), e a
> fila de tarefas com gatilho.
>
> Leia o §2 inteiro antes de tocar no Smear. É a parte cara.

## 1. Estado da linha

| | |
|---|---|
| Branch | `line/Painter`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| HEAD | `89ae4cc9` |
| Ahead of `main` | **3 commits** (a linha integrou hoje; estes são posteriores) |
| Árvore | limpa · `check --workspace --all-targets` 0 · clippy 0 |
| Suítes | tool-painter 718 · painter-brush 255 · render 153 · shell 776 · editor-core 756 |

```
89ae4cc9 probe(smear): a cena 13 mostra que o rastro do plow tem UM TEXEL de altura
e1fa546b fix(impasto): a faca leva a MASSA -- Plow nasce em 1.0
38700140 docs(session): line/Painter integrou -- sai da lista de nao-integradas
```

⚠️ **Modo L.** Todo path absoluto e com `/Worktrees/line-Painter/`; todo comando que muta abre com
`cd <worktree> &&`. Um `cd` para o primário (ex.: `git worktree add`) **envenena o cwd pelo resto do
turno** — me pegou hoje, escrevi no `main` por engano. Vide `feedback_sed_relative_path_hits_primary_cwd`.

⚠️ **Integração e ship são ordem EXPLÍCITA do Enio, nunca autônomos.** Feche, escreva handoff, PARE.

---

## 2. O BUG ABERTO — o Smear não leva massa além da fronteira

### 2.1 O sintoma (Enio, 2026-07-18)

> *"Operações como smear não conseguem levar o relevo para além das fronteiras do traço original."*
>
> e depois do meu primeiro conserto: *"as fronteiras não são vencidas. o relevo não é levado além. **nada
> resolvido**"*

Foto dele: um traço grosso com um lóbulo empurrado para fora, que tem **cor** mas lê **chato** — sem corpo.

### 2.2 A evidência (números, não teoria)

Rode a sonda — é ela que transformou "nada resolvido" em dois números:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
PH2D_PUSH_LOOK_DIR=/tmp/look cargo test -p ph2d-host-desktop --release --bins \
  -- --ignored probe_push_render_and_look --nocapture
```

Cena **13** (`13a_smear_before.png` / `13b_smear_after.png`) é o gesto do Enio, dirigido pela porta REAL do
app (`set_paint_tool_mode("smear")`, **não** poking em `paint.paint_mode` — meu teste unitário poked e
passou enquanto o produto falhava).

O que ela imprime:

```
ao longo de y=200:   x150 h3.98 · x210 h3.98 · x250 h3.73 · x310 h2.68   <- atravessa a fronteira
atravessando x=250:  y194 h0.00 · y200 h3.73 · y206 h0.00                <- UM TEXEL de altura
```

Pincel de smear com raio **10**; o rastro tem **1 texel**. E o PNG mostra uma **agulha**, não massa.

**Então a frase certa não é "o relevo não passa da fronteira" — é "passa um filamento e massa nenhuma".**

### 2.3 O mecanismo (confirmado)

`plow_dab_height` (`crates/ph2d-painter-brush/src/height.rs`) faz, por dab:

```rust
dst[di] += (sh - dst[di]) * w;      // sh = fonte um passo inteiro atrás
```

O espaçamento de dab do Smear é **1 pixel** (medido: `from=[150,200] to=[151,200]`), então uma passada de
170 px são ~170 passos. O transporte vira um **PRODUTO** sobre os passos: chega `h · wⁿ`.

- No eixo do arrasto `t = 0` ⇒ `w = 1` **exato** ⇒ nada decai ⇒ o filamento sobrevive.
- 6 px fora do eixo `t = 0.6` ⇒ `w ≈ 0.8` ⇒ `0.8¹⁵⁰ ≈ 0` ⇒ zero.

É a **mesma doença que esta linha já curou duas vezes** — a mordida do bow wave
(`feedback_a_sequential_accumulation_is_sampling_dependent`) e a cápsula do relevo. Terceiro kernel com
ela. ⚠️ **A COR tem a mesma estrutura** (`smear_dab`, `crates/ph2d-painter-brush/src/smear.rs`) — a agulha
existe nela também. O Enio decidiu explicitamente: **consertar os dois pelo MESMO campo.**

### 2.4 ⛔ Duas tentativas MEDIDAS E REPROVADAS — não repita

**(1) Subir o default do Plow de `0.0` para `1.0`** (commit `e1fa546b`). **Necessário, NÃO suficiente.**
Com `0.0` o relevo não se move nem um pixel (o pigmento ia a x=99 e o relevo parava em x=41, a borda exata
do traço). Com `1.0` ele se move — como filamento. **O default fica** (não reverta: com 0 não há nem o
filamento), mas ele nunca foi o bug do Enio, e eu o apresentei como se fosse.

**(2) Trocar o lerp por deslocamento fracionário** (`dst[p] = bilinear(src, p − step·w)`, esperando
transformar o produto `wⁿ` numa soma `n·w`). **Provavelmente a ideia que você também vai ter. É
matematicamente inútil, e eu implementei antes de perceber.**

Motivo: com passo de 1 px e `w ≈ 0.8`, o deslocamento é **sub-pixel**, e
`sample(x − 0.8) = 0.8·dst[x−1] + 0.2·dst[x]` — que **é** a mistura com o vizinho, e `dst[x]` fora do traço
vale zero. Mesma lei, outra escrita. **Os números da sonda saíram bit-idênticos** (foi assim que descobri).

**Corolário que fecha a porta:** *nenhum* conserto local no kernel por-dab resolve isto. Enquanto cada
passo reamostrar o **resultado do passo anterior**, a massa decai geometricamente com a distância. O
deslocamento tem de ser **acumulado** e aplicado **uma vez** sobre a fonte congelada.

### 2.5 A arquitetura que o conserto exige

O **Deform já tem a lei certa** — leia `crates/ph2d-tool-painter/src/tool/paint/warp/apply.rs:52-72`:

```rust
let src  = Arc::clone(&self.paint.deform.pre);      // fonte PRISTINA, congelada no pen-down
let disp = Arc::make_mut(&mut self.paint.deform.disp);
d[0] += a[0];  d[1] += a[1];                        // SOMA, não produto
let px = bilinear_clamped(&src, w, h, dx - d[0], dy - d[1]);   // UM resample da pristina
...
self.warp_render_relief(bbox);   // h/covers/mats pelo MESMO disp — a porta única
```

O Smear precisa disso: sessão com `pre` congelado (rgba + os 3 planos), `disp` acumulado por dab, e render
do congelado. É onde cor e relevo passam a compartilhar **literalmente** o mesmo campo.

### 2.6 ⚠️ Os quatro riscos — é uma WAVE, não um patch

1. **O Smear ganha Tiling, Symmetry, Shape e Grain DE GRAÇA** por cavalgar a lista de dabs
   (`stamp_dabs_inner`). Uma sessão de warp **não herda nada disso**. Se você trocar o motor sem plano,
   "Tiling não funciona no Smear" nasce aqui — exatamente o bug que o plano do Sculpt §10.1 previu para o
   outro caso.
2. **Ciclo de vida da sessão** (pen-down/up, undo, troca de camada, envelope molhado) é **código novo**,
   não kernel novo. É onde o tempo vai.
3. **`warp_render_relief` lê `self.paint.deform.*`.** Ou o Smear toma emprestado o estado do Deform — dois
   donos de uma sessão, cheiro ruim — ou a sessão de warp é **extraída** para algo que os dois usam. A
   segunda é a certa e é refactor de verdade. *Não* duplique o `warp_render_relief`: duas portas para "para
   onde a tinta foi" divergem, e este módulo já pagou por isso.
4. **Muda o desenho do Smear de COR**, que o Enio nunca reclamou. Ele aceitou explicitamente
   (*"os dois pelo mesmo campo"*), mas isso quer dizer **smoke obrigatório da cor**, não só do relevo.

### 2.7 ⚠️ O gate que existe está INSUFICIENTE — conserte-o primeiro

`the_knife_carries_the_body_as_far_as_it_carries_the_pigment`
(`crates/ph2d-tool-painter/src/tool/paint/tests.rs`) — **eu escrevi, está verde, e o produto está
vermelho.** Ele mede **ALCANCE ao longo do eixo do arrasto**, e o filamento tem alcance perfeito. Mede
exatamente a única coisa que já funcionava.

**Refaça-o como SECÇÃO TRANSVERSAL:** atravessando o rastro a meio caminho, a largura do relevo tem de ser
comparável à do pincel (e à do pigmento). Esse gate nasce **VERMELHO hoje** — comece por ele, é o
red-first que eu não tive.

O irmão antigo (`impasto_plow_drags_the_relief_with_the_paint`) amostra **um texel** logo depois da crista:
verdadeiro em qualquer plow > 0, e não diz nada sobre massa. Foi por isso que o bug viveu com dois gates
verdes ao lado.

---

## 3. Fila de tarefas (ordem recomendada)

| # | tarefa | gatilho / estado |
|---|---|---|
| **1** | **O bug do Smear** (§2) | **Ordem viva do Enio.** Comece pelo gate de secção transversal |
| 2 | Engasgo de montagem em tela grande | **Medido, não investigado:** a montagem da sessão de sculpt custa 8,8 ms @2048 mas **17–21 ms @4096** — ~20 ms de engasgo no início de *cada* traço em 4K. A sessão congela 4 planos por `Arc` (deveria ser refcount, não cópia): há algo a entender. Acorde se o Enio reclamar de engasgo ao começar traço |
| 3 | Sculpt na GPU | **Minha recomendação antes do bug aparecer.** O §0.0 novo do CLAUDE.md (*"o teto é o do HARDWARE, nunca o do caminho lento"*) aponta direto para cá: o Inflate roda a 5,9–6,2 ms/move contra alvo 4, em CPU, e os planos de relevo **já atravessam para a GPU** — foi o que a luz construiu (`ph2d-render::ImpastoLightPass`) |
| 4 | Cache com chave de versão dos planos da luz GPU | Deliberadamente adiado. Hoje materializa por frame sujo. Uma versão teria de rastrear TODA entrada do fold, e **o modo de falha de esquecer uma é uma luz velha que ninguém vê que é velha**. Acorde se aparecer em profile — e traga o gate que prova a invalidação em cada entrada |
| 5 | Conserve p/ Flatten/Fill | **Decisão de design, não implementação:** conservar quem *adiciona* exige decidir de onde o volume vem. Precisa do Enio |
| 6 | Relevo do papel | **BARREIRA:** acopla impasto↔aquarela; §2 do plano 16 exige **ordem nova do Enio**. Não faça sem ela |
| 7 | A cura do banco | Residual 0,0286, documentado como **invisível no render** e não-gateado de propósito. Baixo retorno |

---

## 4. Coisas que vão te economizar um ciclo

- **`sculpt_perf_kill_criterion` é confiável agora** (commit `66b57b63`, já em `main`): a janela de
  aquecimento é MEDIDA, não presumida — move 0 é grátis (a sessão nem abriu), move 1 paga a montagem,
  2..19 é regime. Dispersão caiu de 3,65 ms para 0,06. **A barra (kill 8) não foi tocada.**
- **Esta máquina degrada ~3× ao longo de uma sessão longa.** O kernel *inalterado* mediu 10 ms onde a doc
  registra 3. Prefira **gate contado** a wall-clock (`the_reach_bound_admits_only_the_offsets_that_could_contribute`
  é o padrão: conta offsets, igual em toda máquina).
- **Gates de GPU e perf são `#[ignore]`** e fazem `return` limpo sem dispositivo — numa máquina sem GPU a
  paridade CPU/GPU deixa de ser verificada **em silêncio**.
- **Mutação com `cp`, nunca `git checkout`** (apaga a feature e o gate "passa"). Backup antes, restaure
  depois, confirme com `diff -q`.
- **Busca negativa precisa de controle positivo.** Hoje grepei `EXTRAORDINARIO`/`extraordinário` e deu
  zero; o texto era `EXTRAORDINÁRIO` e eu quase reportei um merge corrompido que estava intacto.

## 5. Documentos

- [`HANDOFF_line_Painter_gpu_light_2026-07-18.md`](HANDOFF_line_Painter_gpu_light_2026-07-18.md) — a luz na
  GPU + reach bound do Inflate + o conserto do harness de perf.
- [`HANDOFF_INTEGRACAO_line_Painter_2026-07-18.md`](HANDOFF_INTEGRACAO_line_Painter_2026-07-18.md) — o que
  foi integrado hoje.
- [`HANDOFF_line_Painter_inflate_closing_2026-07-18.md`](HANDOFF_line_Painter_inflate_closing_2026-07-18.md)
  — o fechamento morfológico do Inflate.
- CLAUDE.md §5 (Painter) — estado por-módulo, já atualizado com tudo acima.
