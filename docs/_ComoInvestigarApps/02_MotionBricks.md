# MotionBricks — o oráculo de MOVIMENTO, e a porta está **ABERTA**

> **Alvo:** NVIDIA **MotionBricks** (NVlabs/GR00T-WholeBodyControl, sub-projecto `motionbricks/`),
> SIGGRAPH 2026, lançado 2026-04-27. Geração de movimento humanóide **em tempo real** a partir de
> primitivas de autoria, com o modelo a decidir **quanto tempo** cada transição demora.
>
> ⚠️ **Este doc descreve o mundo em 2026-09-15.** O estado vivo é o [`CLAUDE.md §5`](../../CLAUDE.md).

---

## §1 — A triagem de licença (passo 1, sempre — §0.9)

⚠️ **A unidade é o ARTEFACTO, nunca o nome do projecto.** O repositório-pai declara-se
`NOASSERTION` no GitHub — o que um levantamento pelo nome leria como *fechado*. O `LICENSE` na
raiz diz outra coisa, e diz-a por partes:

| artefacto | licença | o que isso autoriza |
|---|---|---|
| **o código** (71 ficheiros `.py`, 10 575 linhas) | ⭐ **Apache-2.0** | **ler, portar e distribuir**, com atribuição. **Sem parede, sem clean-room, sem subagentes** |
| **os pesos** (3 `.ckpt`, 2,33 GB) | **NVIDIA Open Model License** | uso comercial **permitido com atribuição**, sujeito aos requisitos de *trustworthy AI* |
| **o corpus** BONES-SEED (142 220 movimentos, 288 h @ 120 fps) | **licença própria, com portão** | ⛔ **NÃO medido** — exige aceitar termos em `bones.studio`. *Não o toque antes de alguém ler a licença* |

⭐ **É a quarta porta permissiva desta máquina** (as outras três: Godot MIT · OpenToonz BSD-3 ·
`libmypaint` ISC). ⇒ **aqui LÊ-SE o fonte**, e a §2 do [`00_o_metodo`](00_o_metodo.md) — *"ler o
fonte é o método pior"* — continua a valer como **preferência de método**, não como parede: a
saída ainda ensina *o quê* e o fonte ainda ensina só *como*.

⚠️ **As dependências instaladas trazem a licença delas.** `pynput` e `python-xlib` são **LGPL** e
só servem o teclado do demo com janela — o caminho headless não os chama. Tudo o resto
(`torch` BSD-3 · `mujoco` Apache-2.0 · `transformers` Apache-2.0 · `pytorch-lightning` Apache-2.0)
é permissivo.

---

## §2 — O que ele É, lido no CÓDIGO (não no comunicado)

O comunicado diz *"15 000 FPS, 2 ms, 350 000 clips"*. O código diz o que isso significa:

**Ele é um IN-BETWEENER com duração variável.** A chamada de inferência
(`motion_inference.predict` (`motionbricks/motion_backbone/inference/motion_inference.py`))
recebe **uma janela de 8 quadros de restrição** — 4 de **passado** (o contexto) e 4 de **futuro**
(o keyframe-alvo) — cada um com uma **máscara booleana** (`has_global_root_values`,
`has_local_root_values`, `has_local_poses`), e devolve o miolo. O número de quadros do miolo
(`num_tokens`) é **opcional**: passado como `MASKED`, **o modelo prediz quanto tempo a transição
demora**.

⭐⭐ **É essa a ideia, e ela não tem nada de específico de robôs:** *um keyframe é uma MÁSCARA
sobre uma janela de largura fixa, e a duração entre keyframes é uma SAÍDA, não uma entrada.*

### As três peças, e o que cada uma faz

| peça | ficheiro | o que faz | tamanho |
|---|---|---|---|
| **VQ-VAE** (tokenizador) | `vqvae/models/motion_vqvae.py` | comprime **4 quadros ⇒ 1 token**; quantizador **multi-cabeça** (8 cabeças), `code_dim 256` | 273 MB |
| **modelo de POSE** | `motion_backbone/models/pose_model.py` | transformador `n_embd 1024 · n_head 16 · n_layers 16`; prediz os tokens de pose | **1,6 GB** |
| **modelo de RAIZ** | `motion_backbone/models/root_model.py` | prediz **quantos tokens** (a duração) **e** a trajectória da raiz | 391 MB |

`min_tokens = 6`, `max_tokens = 16` ⇒ **24 a 64 quadros** (0,8 a 2,13 s a 30 fps) por geração.
O controlador **regenera a cada 8 quadros** (`_CONTROLLER_DT = 8/30 ≈ 0,267 s`).

### ⭐⭐⭐ A peça que NÃO é uma rede: a mola

Entre *«o artista carregou em W»* e *«aqui está o alvo»* não há modelo nenhum — há uma **mola
criticamente amortecida em forma FECHADA**
(`full_agent._generate_spring_model_position_and_heading`):

```python
y  = (4.0 * ln2) / (halflife + eps) / 2.0        # halflife → taxa
j0 = x0 - alvo ;  j1 = v0 + j0 * y
x(t) = (j0 + j1 * t) * exp(-y * t) + alvo        # avaliada em 8 instantes de uma vez
```

com `exp(-x)` substituída por uma **aproximação racional** (`1/(1 + x + 0,48x² + 0,235x³)`).

⭐ **E o knob é um TEMPO:** `halflife = 0,80 s` para a **posição** e **`0,17 s`** para o **rumo**.
*A personagem vira ~4,7× mais depressa do que se desloca* — e isso lê-se de uma vez porque as duas
grandezas estão na mesma unidade.

⚠️ **O comentário do código contradiz a própria constante** (diz *"halflife = 0.4 for the
positions"* ao lado de `0.8`). *Leia a constante, não a prosa.*

### ⭐⭐ Um «tijolo» (estilo) é assustadoramente barato

[`demo/clips.py :: clip_holder_G1.CLIPS`] — **15 estilos**, e cada um é só:

```python
"walk_zombie": {"clip_id": "zombie_walk_180_R_003__A330",
                "start_frame": 10, "end_frame": 100,      # ⬅ o excerto de referência
                "avg_root_vel": 0.6 * 2,                  # ⬅ a velocidade que o estilo implica
                "allowed_pred_num_tokens": [1,1,1,1,1,1,0,0,0,0,0]}  # ⬅ que DURAÇÕES são legais
```

O `elbow_crawling` tem **5 quadros** de referência. O `walk_boxing` tem 10.
⭐⭐⭐ **`allowed_pred_num_tokens` é a lei transferível:** *um estilo DECLARA que durações admite* —
a maioria só aceita 6–11 tokens; `walk_left`/`walk_right` só 6–9 *"for robot deployment safety"*.

⛔ E há um cheiro a registar: o `* 2` em cada `avg_root_vel` está lá porque *"a velocidade real é
0,5 da `avg_root_vel` por causa do modelo de mola"* — **uma constante de dados a compensar o
comportamento de outro subsistema**, que é exactamente a forma do `ADAPT_RATIO` emprestado que o
quad remesh pagou (§0.0: *um limite legítimo diz de que recurso ele é*).

---

## §3 — A porta sem interface (MEDIDA)

⭐ **Há porta, e ela está no próprio demo:** `interactive_demo_g1.py` aceita **`--has_viewer 0`** e
**`--controller random`**, e nesse ramo não toca no teclado nem no X. ⇒ *nenhum `pynput`, nenhum
`DISPLAY`, nenhum MuJoCo viewer.* O harness está em
`~/Documentos/Projetos/ph2d-motionbricks/oracle_run.py` e grava CSV com cabeçalho (molde do §4).

```
cd ~/Documentos/Projetos/ph2d-motionbricks/oracle/motionbricks
../../.venv/bin/python ../../oracle_run.py --steps 600 --mode walk --out ../../corpus/walk.csv
```

### O que saiu (RTX 5060 Ti · 16 GiB · torch 2.14.0+cu130 · `load 3,8–4,0`)

| estilo | latência de uma chamada, p50 | deslocamento | altura da raiz (mediana) |
|---|---|---|---|
| `walk` | **16,28 ms** | 0,465 m/s | 0,758 m |
| `walk_zombie` | 15,53 ms | 0,238 m/s | 0,762 m |
| `stealth_walk` | 15,36 ms | 0,432 m/s | 0,729 m |
| `elbow_crawling` | 16,37 ms | 0,297 m/s | **0,186 m** |

⭐ **A `elbow_crawling` a `0,186 m` é o controlo que prova que os estilos são reais** — a pélvis
desce ao chão (`0,121..0,786`) e a amplitude média das 29 juntas sobe `0,748 → 1,173 rad`
(**+57 %**). *Um estilo que só mudasse a aparência não move a altura da raiz.*

### A decomposição (400 quadros, 23 chamadas em regime)

| | p50 | p90 |
|---|---|---|
| a chamada INTEIRA | **18,19 ms** | 31,02 ms |
| ⤷ só o **modelo** | **11,86 ms** (65 %) | 19,90 ms |
| ⤷ o resto (mola + MuJoCo + Python) | 6,33 ms | — |

- quadros por chamada: **32 a 44** (p50 44) ⇒ **2 418 quadros/s** gerados **num fluxo só**
- ciclo do controlador: **267 ms** (8 quadros a 30 fps) ⇒ **ocupação 7 %**

⚠️⚠️ **Os «15 000 FPS / 2 ms» do comunicado NÃO são estes números, e isso não os desmente.**
Aquilo é débito **em lote** e latência do **forward** noutra máquina; isto é `batch = 1`, sem
TensorRT, com Python no laço, num GPU de consumo. **A afirmação que se verifica aqui é a que
interessa ao produto:** o modelo ocupa **7 %** do relógio disponível entre duas replaneações.

### ⚠️ A janela (o smoke do dono) NÃO abre nesta sessão sem forçar X11

A sessão é **Wayland** (`XDG_SESSION_TYPE=wayland`, com XWayland em `DISPLAY=:0`). O visualizador
do MuJoCo morre no arranque:

```
GLFWError: (65548) Wayland: The platform does not provide the window position
WARNING: OpenGL error 0x502 in or before mjr_makeContext
```

⭐ **A cura é uma linha** — esconder o Wayland do GLFW, que então cai no XWayland:

```bash
env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 ../../.venv/bin/python scripts/interactive_demo_g1.py
```

⭐ E ela cura **duas** coisas: o `python-xlib` que rouba as teclas `wasd` ao MuJoCo **também só
existe em X11** (os *Known Issues* do alvo dizem-no). Medido: `300` quadros com visualizador
correram os `~10 s` inteiros. ⚠️ **O fecho às vezes dá `SIGSEGV`** (`exit 139`) *depois* de a
animação acabar — é a limpeza do GLFW, **não** a corrida. Uma corrida limpa saiu `exit 0`.
⛔ *Não leia o segfault de fecho como «não funciona».*

### ⛔ Mais quatro armadilhas medidas ao pôr isto de pé

1. ⛔ **`--allowed_mode` é comparado com `in`.** O controlador faz
   `[i for i in CLIPS if i in control_info["allowed_mode"]]`. Com uma **string** — que é o que o
   `argparse` do demo declara (`type=str`) — isso é **casamento por SUBSTRING**: pedir
   `walk_zombie` deixa passar **também** `walk`, e a corrida mistura dois estilos **em silêncio**.
   *Passe uma **LISTA**.* O harness já o faz (`a.mode.split(",")`).
2. ⛔ **`git lfs pull` encravou** (0 % de CPU, `.git/lfs` a zero) e o `git clone` normal **falha** se
   o `git-lfs` não estiver instalado (*«could not read greeting from subprocess git-lfs
   filter-process»*), porque o filtro está em `.gitattributes`. A rota que funcionou **sem root** é
   `media.githubusercontent.com/media/<owner>/<repo>/<ref>/<path>`, que serve o conteúdo real — e o
   **`sha256` bate com o do ponteiro LFS**, que é como se verifica.
3. ⛔ **O ramo headless do demo oficial ESTOURA com o controlador por omissão.** `--has_viewer 0`
   passa `viewer = None`, e o `WASD_controller` desreferencia-o para desenhar o alvo
   (`controllers.py:155`) ⇒ `Traceback` sempre. Só **`--has_viewer 0 --controller random`** corre.
   *A porta sem interface existe e a combinação por omissão não a abre.*
4. ⚠️ **Uma ligação HTTPS envelhecida lê-se como *throttling*.** A mesma transferência estava a
   `170 KiB/s` enquanto uma **segunda** ligação ao **mesmo ficheiro** dava `10 435 KiB/s` — 61×.
   *Antes de aceitar que a rede é lenta, abra uma segunda ligação e meça-a.* (8 pedaços em paralelo
   trouxeram os 390 MB em segundos.)

---

## §4 — O que ele ENSINA ao PH2D, por ordem de preço

### 4.1 ⭐⭐⭐ O knob de uma mola é um TEMPO — e o PH2D tem a lei certa com o rótulo errado

A [`ph2d-spring`](../../crates/ph2d-spring/src/lib.rs) integra `a = ω²(1−x) − 2ζω·v` por Euler
semi-implícito em fatias de `1/240 s`. Para `ζ = 1` — **que é o default dela** — essa equação tem
solução fechada, e é **exactamente** a que o MotionBricks avalia:

```
x(t) = (j0 + j1·t)·e^{−y·t} + alvo ,  j0 = x0 − alvo ,  j1 = v0 + j0·y ,  y = ω
```

⇒ a conversão é exacta: **`halflife = 2·ln2 / ω`**.

| | rigidez `ω` | meia-vida |
|---|---|---|
| `MIN_STIFFNESS` | 1 rad/s | **1 386 ms** |
| **`DEFAULT_STIFFNESS`** | **12 rad/s** | **115,5 ms** |
| `MAX_STIFFNESS` | 60 rad/s | **23,1 ms** |
| MotionBricks — **rumo** | (8,15) | **170 ms** |
| MotionBricks — **posição** | (1,73) | **800 ms** |

⭐ *«a UI fecha metade da distância em 116 ms»* é uma frase que o dono lê; *«ω = 12 rad/s»* não é.
⚠️ **A cura honesta é MOSTRAR o tempo ao lado da rigidez, não substituí-la:** a igualdade só vale
em `ζ = 1`, e o slider do PH2D oferece `ζ ∈ [0,1 · 2,0]`. Trocar o rótulo sem trocar a lei
mentiria em toda a faixa sub/sobre-amortecida.
⭐⭐ E a leitura de produto que o alvo traz de graça: **virar é ~4,7× mais rápido que andar**
(170 ms contra 800 ms). *Dois eixos de uma mesma pose não partilham meia-vida* — nota para quando
o PH2D animar a pose de uma câmera ou de um gizmo.
⛔ **A forma FECHADA não é a lição aqui.** Ela compra avaliação num `t` arbitrário (scrub), e a
`ph2d-spring` é uma mola de **UI**, que ninguém rebobina. Adoptá-la pelo relógio seria optimização
sem medição (§0.0).

### 4.2 ⭐⭐ Um keyframe é uma MÁSCARA sobre uma janela; a duração é uma SAÍDA

A porta única de inferência recebe `(janela de 8 quadros, 3 máscaras booleanas por quadro,
num_tokens opcional)`. Não há caso especial para *«só a posição»*, *«só o rumo»*, *«só a pose
final»* — **desligar um bit da máscara é o caso especial**, e o próprio produto usa isso:

```python
has_local_root_values[:, 3] = False          # «a última velocidade do contexto é inválida»
if not target_root_realignment:               # «não confies no alvo» — 3 máscaras de uma vez
    has_local_root_values[:, -4:] = False
```

⭐ A transferência barata é **a forma**, não o modelo: no PH2D, *«este canal tem key aqui»* e
*«este canal está livre»* são hoje perguntas espalhadas por `ph2d-anim` e pelo Flip. Uma janela
com máscara faz o caso geral cair de graça.
⏳ **A duração predita é a parte cara** e precisa de um modelo — não é proposta.

### 4.3 ⭐⭐ Um ESTILO é 5–90 quadros + uma velocidade + as durações LEGAIS

`allowed_pred_num_tokens` é uma máscara de 11 bits: *que durações este estilo admite*. A maioria
dos 15 estilos só aceita as 6 mais curtas; `walk_left`/`walk_right` só 4, com o motivo escrito ao
lado (*"for robot deployment safety"*).

⭐⭐ **Esta é a lição para a DINÂMICA DOS CICLOS** ([doc 103](../Motion%20Nodes/103_dinamica_dos_ciclos.md)):
hoje um ciclo do Motion **não declara** as durações que sabe fazer — e um param que declara a
própria faixa legal é a família do `ParamGateAbove`, que o repo já tem.

### 4.4 ⭐ O contacto do pé é um CANAL do documento, não uma derivação do consumidor

4 dimensões binárias (calcanhar e ponta de cada pé) viajam **dentro** do vector de features, e são
computadas **uma vez** na preparação dos dados (por limiar de posição/velocidade). O `ph2d-skeleton`
e o Flip não têm canal de contacto nenhum.
⇒ No dia em que alguém atacar o *foot-sliding* de um rig 2D, a lei é esta: **o contacto mede-se uma
vez e guarda-se**, em vez de cada quadro o re-derivar.

### 4.5 ⭐ NÃO canonicalizar — ALEATORIZAR

`docs/motion_representation.md`, textual: *«MotionBricks does not apply a fixed heading
canonicalization … each motion segment is randomly rotated at training time»*.
⭐ É o **mesmo veredito** que a `line/quadextract` mediu em 31/08 por outro caminho — canonicalizar
a pose **destrói** (`−77 %`, `−105 %`, CLAUDE.md §5 / 3D-Sculpt). Duas famílias sem relação, a
mesma conclusão. *Convergência, não prova.*

### 4.6 ⛔ O caminho ONNX está NOMEADO nos dois lados e CONSTRUÍDO em nenhum

| lado | o que existe | o que falta |
|---|---|---|
| **MotionBricks** | o código **evita** ops que não exportam (dois comentários explícitos em `mujoco_helper.py`), regista os clips como *parameters* *"so that it will be later used in onnx / trt model"* | `full_agent.set_prebaked_inference_engine()` ⇒ **`NotImplementedError`** |
| **PH2D** | `tract-onnx` **já está no `Cargo.lock`** (9 crates), vendorizado em [`ph2d-audio-ml`](../../crates/ph2d-audio-ml/), atrás da feature `audio-ml` **OFF por omissão**, com gate a impedi-lo de chegar ao mixer RT ([ADR-0123](../architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md)) | o modelo embutido que o repo aceita hoje tem **7,6 MB**; este tem **2,33 GB** — **306×** |

⛔ **Isto não é uma proposta.** É um endereço, para que a próxima janela que pergunte *«pode isto
correr dentro do PH2D?»* não responda *«não há runtime»*: o runtime existe, o exportador não shipou,
e a diferença de escala do modelo é de duas ordens e meia de grandeza.

---

## §5 — O que NÃO se porta (e o número que o diz)

| | porquê |
|---|---|
| **o modelo** | **2,33 GB** de pesos + torch + CUDA + Python. O maior modelo que este repo embute hoje é o DFN3 do denoise, com **7,6 MB** ([ADR-0123](../architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md)) — **306× mais pequeno** |
| **o esqueleto** | `G1Skeleton34` é um **robô**: 34 juntas, 29 dobradiças de 1 DOF, sem dedos, com 2 juntas de dedo do pé **fictícias** só para o contacto. Não é uma figura de animação, e muito menos um rig 2D |
| **o corpus** | BONES-SEED tem **licença própria com portão**. ⛔ não descarregar antes de alguém a ler |
| **o número do comunicado** | «15 000 FPS» é débito em **lote**. O que esta máquina faz num fluxo está no §3, e é outra grandeza |

## §6 — Onde está, e como se reinstala do zero

**Instalado em `~/Documentos/Projetos/ph2d-motionbricks/`** — *fora* da árvore do PH2D, como a
`ph2d-quadbench` (precedente [ADR-0162](../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md)).
⛔ **NÃO** em `~/Referencias/`, que é a **zona contaminada** e proíbe o papel I de entrar: aqui a
licença é Apache-2.0 e **ler é legal**.

```
ph2d-motionbricks/
  .venv/          # uv + CPython 3.12.14 (o sistema tem 3.14; ver aviso)
  oracle/         # clone esparso de NVlabs/GR00T-WholeBodyControl (só motionbricks/), 2,7 GB
  oracle_run.py   # O HARNESS: corre sem interface e grava CSV com cabeçalho
  corpus/         # a saída do alvo, sobre entradas nossas
```

```bash
# 1. uv, sem root
curl -LsSf https://astral.sh/uv/install.sh | sh

# 2. o clone (esparso, sem LFS — o git-lfs desta máquina veio de um tarball para ~/.local/bin)
mkdir -p ~/Documentos/Projetos/ph2d-motionbricks && cd $_
git clone --filter=blob:none --no-checkout --depth 1 \
    https://github.com/NVlabs/GR00T-WholeBodyControl.git oracle
cd oracle && git sparse-checkout init --cone && git sparse-checkout set motionbricks
GIT_LFS_SKIP_SMUDGE=1 git checkout

# 3. os pesos, por HTTPS directo (2,33 GB) — verifique o sha256 contra o ponteiro LFS
#    https://media.githubusercontent.com/media/NVlabs/GR00T-WholeBodyControl/main/<path>

# 4. o ambiente
cd ~/Documentos/Projetos/ph2d-motionbricks
uv venv --python 3.12 .venv
uv pip install --python .venv/bin/python torch numpy mujoco scipy hydra-core omegaconf \
    pytorch-lightning transformers pynput matplotlib vector-quantize-pytorch colorlog \
    adam-atan2-pytorch python-xlib
uv pip install --python .venv/bin/python -e oracle/motionbricks
```

⚠️ **O `conda` do README oficial não existe nesta máquina e não é preciso** — o `uv` faz o mesmo e
instala o CPython 3.12 sozinho.
⚠️ **O Python do sistema é 3.14 e o `torch` 2.14 tem roda `cp314`** — mas a pilha à volta
(`pytorch-lightning`, `transformers`) não foi verificada nele. **3.12 é o que está medido.**
⭐ Verificado: `torch 2.14.0+cu130` · `cuda disponível: True` · RTX 5060 Ti · **capability (12, 0)**
(Blackwell `sm_120`) · `mujoco 3.13.0`.
