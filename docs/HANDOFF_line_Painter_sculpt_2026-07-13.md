# HANDOFF — `line/Painter`: o SCULPT do relevo (2026-07-13)

> **Para o agente NOVO que vai tocar esta linha.** Leia inteiro antes da primeira linha de código.
> É longo de propósito — a alternativa é você redescobrir na marra o que já custou caro.
>
> **O plano é [`docs/Painter/18_plano_sculpt_relevo.md`](Painter/18_plano_sculpt_relevo.md).** Ele
> **decide**; este documento te dá o modo de trabalho, o estado real e as armadilhas.
>
> **Ordem do Enio: comece pelo SMOOTH (W1).**

---

## PARTE I — O MODO DE TRABALHAR (Modo L)

### 1. O que é, em uma frase

`workstation` ⇒ **Modo L** ([ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)):
**N linhas paralelas, cada uma numa `git worktree` própria, SEM coordenador.** Você é uma linha.

Docs que mandam (não duplique, consulte): [`DIRETRIZ.md §1.5`](IntegracaoMultiAgente/DIRETRIZ.md) ·
[`GUIA_JORNADA_MODO_L.md`](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) ·
[`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (**a cada passo**) ·
`CLAUDE.md` (os 7 inegociáveis).

### 2. As 5 regras que, se você quebrar, quebrou o protocolo

1. **🔴 VOCÊ NÃO INTEGRA. NÃO FAZ `git push`. NÃO RODA `./scripts/ship.sh`.** Integração e ship são
   **ordem EXPLÍCITA do Enio**, por um agente integrador dedicado. Você **fecha a linha, escreve o
   handoff (DIRETRIZ §1.5.9) e PARA.** Os três por conta própria = **violação** (CLAUDE.md §0.7).
2. **Você PODE tocar foundational** (ADR-0107) — e vai (esta linha mexe em `ph2d-painter-brush`,
   `ph2d-editor-core/ids`, `ph2d-render` se for pra GPU). **Ao CRIAR foundational novo, projete pra
   ISOLAMENTO** (módulo irmão, ponto de extensão append-only). **PARE e reporte ao Enio** só em 2 casos:
   **contrato congelado** (CLAUDE.md §6) ou **rebase conflitando fora dos seus arquivos**.
3. **Fast mode o dia inteiro:** `git commit --no-verify`, **zero push, zero CI**. O gate pesado roda
   **1× no fechamento**, nunca por task.
4. **Inner loop = `cargo check -p <crate>`.** Nada de `--workspace`, clippy ou teste por task.
5. **`cd` em TODO comando, e mutação SEMPRE por caminho absoluto.** O `cwd` volta pro repo primário a
   cada turno. **Isto me pegou nesta jornada:** um `python3` com caminho relativo escreveu um `mod` na
   `main` em vez da worktree. Conferi o diff, revertí, reapliquei. Memória:
   [[feedback_sed_relative_path_hits_primary_cwd]].

### 3. Abrir a linha

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
git fetch --all && git rebase main     # a worktree JÁ existe e o target/ está quente
git log --oneline -1                   # confira
cargo check -p ph2d-tool-painter       # ← o loop. Só isto.
```

### 4. Como se prova uma coisa aqui (NÃO é negociável)

**Um gate verde que você não sabe derrubar não é um gate.** Toda afirmação vira teste, e todo teste tem
um **VERMELHO provado por MUTAÇÃO**: você quebra o código de propósito, roda, e o gate cai. Se não cair,
**ou o gate é frouxo, ou o seu COMENTÁRIO está errado** — e nesta jornada foi o comentário, duas vezes.

**Desfaça a mutação com `cp` de um backup, NUNCA com `git checkout -- <arquivo>`** — o checkout apaga a
sua feature não-commitada junto, o gate "passa" (porque a feature sumiu), e você lê isso como sucesso.
[[feedback_mutation_undo_with_cp_never_git_checkout]].

---

## PARTE II — O ESTADO REAL (verificado, não de memória)

### 5. Onde a linha está

**7 commits locais** sobre a `main` (`c315e18e`), **não integrados, não pushados**:

| | |
|---|---|
| `d611e34e` | **fix:** a UI do rig de luzes estava MORTA sob o mouse (os 13 ids nunca foram registrados) |
| `1c1f7e2a` | docs(memory): a lição do rig morto |
| `c6f49a63` | **feat:** o MATERIAL da tinta — núcleo + LUT 2D |
| `1f00c6b5` | **feat:** o material vira **per-pixel**; o Shine muda de dono |
| `27c81370` | **feat:** a luz que atravessa a tinta volta com a **cor** dela + toggle "Adjust Last Stroke" |
| `b0cf07a6` | **feat:** seletor de cor pro Wax (um **filtro**) |
| `6329828b` | **fix:** o undo esquecia o material |

**Gates no fechamento:** `cargo test --workspace` → **6613 passed, 0 failed** · clippy `--all-targets`
→ **0** · perf impasto **3,99 ms/movimento @2048²** (alvo ≤4, kill 8).

**Smoke do Enio: OK** (rig, material, Wax colorido — tudo aprovado).

### 6. O que existe hoje, e que você vai usar

Leia [`18_plano_sculpt_relevo.md`](Painter/18_plano_sculpt_relevo.md) §2 — mas em uma linha: **o kernel
do Smooth já existe** (`impasto_settle::box_blur`), **o motor de warp já existe** (`PaintMode::Deform`),
e o **molde de um modo novo** está pronto pra copiar (o Deform faz tudo que você precisa: state
mode-exclusive, sub-modo `u8`, rail, seção de painel, setters como fonte única de clamp, sessão no
snapshot do undo).

**Os 3 planos por camada** (lazy — uma camada nunca esculpida não paga nada):
`heights` (f32) · `covers` (u8) · `mats` (`MaterialBytes` = 7 B) = **12 B/px**, e há um gate que **conta**
isso (`the_impasto_planes_cost_twelve_bytes_per_pixel`).

---

## PARTE III — A FILA

**Ordem do Enio: W1 (Smooth) primeiro.** As ondas estão no plano §8. Não reordene sem falar com ele.

### O que ler antes de escrever a primeira linha
1. O plano **§4** (o padrão por-traço). **É a decisão que define todo o resto**, e a implementação
   ingênua (borrar `h` no lugar, por dab) **parece funcionar** e está errada. O gate de §4 nasce
   vermelho nela.
2. O plano **§3** ⚠️ (registrar o widget, e o gate que **CLICA** nele).
3. O comentário do `box_blur` em `impasto_settle.rs` — ele explica por que a soma corrida (a
   otimização óbvia) foi **escrita e rejeitada**: acumula erro de float *ao longo da linha* e quebra a
   byte-identidade do crop. **Não a reintroduza.**

---

## PARTE IV — ⚠️ AS ARMADILHAS (todas custaram caro nesta linha, TODAS nesta semana)

| Armadilha | O que acontece |
|---|---|
| **Pintar ≠ estar vivo** | Um widget que **pinta**, registra hit-rect, é encaminhado pelo `event.rs` e roteado pelo tool ainda está **MORTO** se não estiver no `WidgetStore`: `is_focusable` faz `None => false`, o Down não ativa, o `Click` **nunca existe**. Foi assim que os 13 controles do Impasto embarcaram. **E a armadilha de 2ª ordem:** registrar como `Checkbox` emite `Toggled`, que o `event.rs` **não encaminha** — *registrado e ainda morto*. Use `Button`. Gate: `MockPanelHost::click_at`. |
| **`nextest-impacted` não vê os gates de contagem de registry** | Registrar componente/id muda contagens afirmadas em gates que o impacted **não toca**. **Use `cargo test --workspace` no fechamento.** |
| **Pipe mascara o exit code** | `./x.sh \| grep foo` faz `$?` virar o do `grep`. **Verifique o ESTADO, não o código de saída.** |
| **Crase na msg de commit** | `fish`/`zsh` **executa** o conteúdo e a palavra some. Use `git commit -F <arquivo>` e **releia o log**. |
| **Limiar escolhido no chute** | Errei **duas vezes** nesta jornada (88% contra uma barra de 95%; 454 px contra uma de 500) e nas duas o **código estava certo e o gate errado**. Ou você **mede os dois lados** (certo e mutado) e põe a barra no meio, ou você escreve um gate que **não precisa de número** (byte-identidade contra um controle). |
| **Um gate só enxerga o termo que ele LIGA** | A mutação "divisor sem o filtro do Wax" passou em **todos** os gates existentes, porque todos usavam filtro **branco** e `albedo × branco = albedo`. |
| **Comentário que promete uma mutação que ele não pega** | Escrevi que o gate de tinta plana pegaria um divisor quebrado. **Não pega** — pixel plano sai pelo *early-out* antes de tocar o divisor. O gate era verdadeiro e o **comentário mentia**. Se a mutação não sangra, **o comentário é suspeito antes do gate**. |
| **Plano novo ⇒ snapshot do undo, no MESMO commit** | O `mats` ficou de fora e o buraco **se escondia**: em tela vazia a cobertura zera e a luz pesa o material obsoleto por zero. Só falou em **tinta-sobre-tinta**. |

---

## PARTE V — Como você FECHA a linha

1. Gate batched **1×**: `cargo test --workspace` (⚠️ **não** o impacted) + `cargo clippy --all-targets`.
2. Perf, se tocou o relevo:
   ```bash
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
     cargo test --release -p ph2d-tool-painter --lib -- impasto_perf_kill_criterion --ignored --nocapture
   # alvo <=4 ms/movimento, kill 8
   ```
3. **Smoke** — feature nova **ship com o exemplo que a demonstra**; não peça pro Enio montar:
   ```bash
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
     PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
   ```
   *(O smoke arma o impasto **em código** — o que é útil e é uma faca de dois gumes: foi ele que
   escondeu que o Enable mestre estava morto. Se você arma estado por baixo do pano, **o seam que ele
   pula é o seam que você não testou**.)*
4. **Escreva o handoff de integração** (DIRETRIZ §1.5.9): base, nº de commits, superfície foundational
   tocada, contratos, gates, **as armadilhas**, e o que ficou aberto.
5. **PARE.** Reporte *"linha pronta + handoff"* e **espere a ordem do Enio.**

---

## PARTE VI — Aberto (não é desta onda; não comece por eles)

- **Passe de luz na GPU** — o Enio **adiou** ("não é hora de GPU"). Quando voltar: o compositor GPU
  (`ph2d-render/src/layer_compositor/`, **foundational**) tem `LayerOp::Adjustment` e 8 slots livres em
  `AdjustmentKind ≤ 32`. O gate de paridade da casa é **o limite mais apertado que a matemática admite**
  (S/H bateu diff 0; Bloom ≤5B), **não** um "bit-a-bit" cego. Para o passe de luz **diff 0 é plausível**:
  o único transcendental (`pow`) **já é LUT** e `sqrt` é corretamente-arredondado por IEEE-754. E há um
  dado novo: o fold do material é **bandwidth-bound**, não ALU-bound.
- **Persistência** — `SpriteSource::Individual` e `CookedTexture` ainda não persistem os pixels (gap
  herdado, não do material: `mats` **já** vai pro disco, `PROJECT_SCHEMA = 9`).
- **A TINTA EMPURRADA (o Push)** — o Enio: *"ainda não resolveu. Fim da fila."* A mecânica está certa; o
  **desenho** da tinta deslocada não convence, e **não foi diagnosticado**. Quando chegar a hora: comece
  **renderizando e olhando**, não pela teoria. O candidato que a pesquisa aponta e que **não**
  implementamos é o **bow wave** (doc 17 §3, mecanismo 6) — e repare que o §6 do plano de sculpt encosta
  exatamente nele: **o Conserve do Scrape e o Push são o mesmo problema pelos dois lados.**
- **Cor de Wax independente da tinta** (o caso "azul que brilha laranja") — hoje é **impossível por
  construção**: o filtro multiplica, e multiplicação só **remove** espectro. Só com ordem do Enio.
- Bug #11 (Per-Layer Color, listras intermitentes) e o handoff de perf de camadas-como-brush —
  **herdados**, dormentes.
