# 02 — As tarefas

> Cada tarefa tem **ID**, **onde**, **fazer**, **verificar** e **recuo**. O ID é o endereço: cite-o
> no commit (`chore(stack): C7 — get_current_texture`), no `04_registro.md` e num handoff.
>
> ⚠️ **Regra que atravessa todas:** o inner loop é `bash scripts/cargo-check-narrow.sh <crate>`.
> Teste só onde a tarefa **diz** para testar. Razão medida: §2 do `CLAUDE.md`.
>
> ⚠️ **Tier desta máquina: `workstation`** (123 GiB, 32 cores, mold). 10 `cargo check` simultâneos,
> 5 builds. Não serialize o que não depende.

---

# Bloco T — o terreno (5 tarefas) · 🟢 sem risco

## T1 — `nodatacow` nos 7 `target/`, **agora que estão vazios**

**Por quê:** `chattr +C` só vale para arquivos **criados depois**. Num `target/` cheio ele é inerte.
O disco acabou de ser trocado e os 7 targets estão vazios — **esta é a única hora em que sai de graça.**

**Onde:** o `target/` do primário e os 6 das worktrees.

**Fazer:**
```
cd /home/enio/Documentos/Projetos/PH2D
mkdir -p target && chattr +C target
for w in Worktrees/*/; do mkdir -p "$w/target" && chattr +C "$w/target"; done
```

**Verificar:** `bash scripts/btrfs-health.sh` — a linha `! N target(s) de worktree SEM nodatacow`
tem de desaparecer.

**Recuo:** `chattr -C <dir>`. Sem efeito sobre arquivos já escritos.

## T2 — o vermelho FALSO do `btrfs-health.sh`

**Por quê:** ver `01_inventario.md` §7. A regra da linha 116 reprova por metadata apertada **sem
olhar o não-alocado**, e no disco novo isso grita todo dia sobre um sistema saudável. Um alarme que
sempre toca deixa de ser lido — e o dia em que o perigo for real, ninguém vai olhar.

**Onde:** [`scripts/btrfs-health.sh`](../../scripts/btrfs-health.sh) linhas 113-118.

**Fazer:** a metadata só é vermelha quando **as duas** condições valem. Manter o amarelo como está.
```
meta_free < 1 GiB  E  unalloc < 8 GiB   → VERMELHO (a metadata não tem de onde crescer)
meta_free < 1 GiB  E  unalloc >= 8 GiB  → verde, com a nota «o btrfs aloca outro pedaço quando precisar»
```

**Verificar:** `bash scripts/btrfs-health.sh` fica verde neste disco **e** — prova de mutação —
com `unalloc` forçado a 0 volta a ficar vermelho. ⛔ Sem o segundo, a correção é indistinguível de
apagar o teste.

**Recuo:** `git checkout scripts/btrfs-health.sh`.

## T3 — a fotografia do ANTES

**Por quê:** metade das tarefas deste plano acabam em *«ficou mais lento?»* ou *«esse teste já
falhava?»*. Sem o antes, cada uma dessas perguntas custa uma reconstrução.

**Fazer, e colar no [`04_registro.md`](04_registro.md) §1:**
```
cd /home/enio/Documentos/Projetos/PH2D
rustc -V; cargo -V
bash scripts/stack-audit.sh | sed 's/\x1b\[[0-9;]*m//g' > "docs/Atualizar Stack/_antes_audit.txt"
git rev-parse HEAD
/usr/bin/time -v cargo build --workspace --release 2>&1 | tail -20   # o relógio da build limpa
CARGO_INCREMENTAL=0 cargo nextest run --workspace --no-fail-fast 2>&1 | tail -30
```

⚠️ **`--no-fail-fast` é obrigatório.** O nextest cancela no primeiro ✗ e esconde o resto da suíte —
uma corrida já parou em 11 240 com 1 007 por correr (§5.0 do `CLAUDE.md`). Sem ele a fotografia mente.

⚠️ **Registre também os vermelhos PRÉ-EXISTENTES pelo nome.** Se um deles reaparecer no bloco C,
saber que já estava lá economiza meio dia.

**Verificar:** o `_antes_audit.txt` existe e o `04_registro.md` §1 tem as 5 linhas preenchidas.

## T4 — a rede

**Fazer:**
```
git tag stack-upgrade-base
git switch -c chore/stack-upgrade-2026-08
```

**Por quê a tag e não só a branch:** as 6 worktrees continuam a apontar para `main`. A tag é o ponto
para o qual qualquer uma delas volta, e sobrevive a um `reset` da branch.

⛔ **Não pushe nada.** §0.7 — o ship é do Enio.

**Verificar:** `git tag | grep stack-upgrade-base` e `git branch --show-current`.

## T5 — o `stack-audit.sh` entra no roteador

**Por quê:** medido em 2026-08-18 — o `cargo-check-narrow.sh` está no `CLAUDE.md` há semanas, faz a
coisa certa, e foi invocado **5 vezes em 101 sessões** contra 13 791 `cargo check` à mão. *Os quatro
scripts realmente vivos têm em comum uma coisa só: um passo obrigatório de protocolo os nomeia.*

**Onde:** [`CLAUDE.md`](../../CLAUDE.md) §1 (tabela do roteador).

**Fazer:** uma linha na tabela — *«Atualizar dependência / «dá para subir X?»» → `bash scripts/stack-audit.sh` + [`docs/Atualizar Stack/`](.)*.

⛔ **Uma linha.** O §5 foi de 1,7 KB a 917 KB por acréscimos que pareciam pequenos, e este arquivo é
injetado inteiro em todo agente.

**Verificar:** `grep -c 'stack-audit' CLAUDE.md` ≥ 1.

---

# Bloco A — Rust 1.95 → 1.98 (10 tarefas) · 🟢 se compilar, está igual

⚠️ **Este bloco não muda um pixel.** Toda diferença que ele pode causar é erro de compilação ou
reclamação do clippy. É por isso que ele vai primeiro: quando o bloco C ficar vermelho, o Rust já
não é suspeito.

## A1 — instalar a toolchain antes de pinar

**Fazer:** `rustup toolchain install 1.98 -c rustfmt -c clippy -c rust-src --profile default`

⚠️ **Um `-c` por componente.** O `--component a b c` faz o rustup ler `b` e `c` como **nomes de
toolchain** e falhar com `invalid toolchain name: 'clippy'`. Custou uma tentativa em 2026-08-29.

**Verificar:** os três, não só o primeiro — um componente que não veio só aparece no `ship.sh`:
```
rustup run 1.98 rustc -V && rustup run 1.98 cargo clippy -V && rustup run 1.98 cargo fmt --version
```
**Feito em 2026-08-29:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` · `clippy 0.1.98` · `rustfmt 1.9.0`.

**Por quê antes:** pinar primeiro faz o próximo comando de cargo baixar a toolchain no meio de outra
coisa, e o erro sai disfarçado de erro de build.

## A2 — o pin

**Onde:** [`rust-toolchain.toml`](../../rust-toolchain.toml) linha 2.
**Fazer:** `channel = "1.95"` → `channel = "1.98"`.
**Verificar:** `rustc -V` na raiz do repo diz 1.98 sem `rustup run`.

## A3 — a MSRV declarada

**Onde:** [`Cargo.toml`](../../Cargo.toml) linha 33.
**Fazer:** `rust-version = "1.95"` → `"1.98"`.

⚠️ **As duas movem-se JUNTAS, e há um gate que o exige:**
[`crates/ph2d-editor-core/tests/architecture_msrv_is_the_pinned_toolchain.rs`](../../crates/ph2d-editor-core/tests/architecture_msrv_is_the_pinned_toolchain.rs).
Ele lê o `channel` do `rust-toolchain.toml` e o `rust-version` do `Cargo.toml` e exige que sejam
iguais. Mexer num sem o outro **reprova**.

⚠️ **Leia o doc-comment dele antes de tocar em qualquer coisa de MSRV** — ele registra que o job de
CI que este gate substituiu **nunca testou a versão que o nome dele prometia** (o `rust-toolchain.toml`
vence o `rustup default`), e que o piso real da workspace **é** o pin, medido versão a versão.
*Um número de MSRV aqui não é uma promessa de compatibilidade: é a afirmação de que os dois campos
concordam.*

**Verificar:** `cargo metadata --no-deps --format-version 1 | grep -o '"rust_version":"[^"]*"' | head -1`

## A4 — os 4 pontos do CI

**Onde:** [`.github/workflows/spike.yml`](../../.github/workflows/spike.yml) linhas **49, 235, 461, 628**.
**Fazer:** `dtolnay/rust-toolchain@1.95` → `@1.98` nas quatro.

⚠️ **São quatro, e o comentário das linhas 42-48 avisa que elas movem em lockstep.** Um `@1.95`
esquecido não dá erro — dá um job medindo outra toolchain, e a cache key roda sozinha.

**Verificar:** `grep -c 'rust-toolchain@1.98' .github/workflows/spike.yml` = 4 e
`grep -c 'rust-toolchain@1.95' .github/workflows/spike.yml` = 0.

## A5 — compila?

**Fazer:** `cargo check --workspace --all-targets`
**Verificar:** exit 0. Se falhar, é código nosso que usava algo removido — raro entre 1.95 e 1.98.

## A6 — o clippy novo

**Por quê é o custo real do bloco:** três versões de regras novas, e o `ship.sh` roda
`clippy --workspace --all-targets -- -D warnings` sobre **297 crates**.

**Fazer:**
```
cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs --fix --allow-dirty
cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
```

⚠️ **`--fix` primeiro, revisão depois.** Ele arruma a maioria mecanicamente; o que sobra é o que
merece olho. ⛔ **Não silencie com `#[allow]` sem ler.** Uma regra nova do clippy achou defeito real
neste repo antes.

**Verificar:** o segundo comando sai 0.
**Recuo:** `git diff` do `--fix` é inteiramente revisável; `git checkout -- <ficheiro>` por arquivo.

## A7 — `linker_messages`: decidir agora, não descobrir no CI

**Por quê:** o Rust 1.97 deixou de esconder a saída do ligador e emite-a como aviso.
**Conferido:** esse lint fica **fora** do grupo `warnings` de propósito, então o nosso
`RUSTFLAGS: -D warnings` **não** o transforma em erro. Ele só polui o log.

**Fazer:** rodar uma build e **olhar**. Se o `mold` (Linux) ou o `ld` (macOS) falarem:
- se a mensagem apontar defeito real → **corrija-o** (foi para isso que a mudança existe);
- se for ruído conhecido → `[lints.rust] linker_messages = "allow"` no `Cargo.toml` do workspace,
  **com o texto da mensagem no comentário**. ⛔ Um `allow` sem a mensagem ao lado é uma cerca de
  Chesterton que ninguém vai poder derrubar depois.

**Verificar:** `cargo build --workspace 2>&1 | grep -c 'linker stderr'`.

## A8 — `build.warnings` (Cargo 1.97) — avaliar, não adotar às cegas

**Por quê:** o Cargo 1.97 estabilizou `build.warnings`, que faz o mesmo que o nosso
`RUSTFLAGS: -D warnings` **sem invalidar a cache de build**. Hoje o `RUSTFLAGS` global do CI força
recompilação sempre que muda.

**Fazer:** medir os dois no CI e escolher. Se adotar: `[build] warnings = "deny"` em
`.cargo/config.toml` e remover o `RUSTFLAGS` do `spike.yml` (linha 24).

⚠️ **Não é obrigatório neste plano.** Se a medição não mostrar ganho, **registre isso** no
`04_registro.md` como recusa medida e siga. ⛔ Uma medição que empata não escolheu nada.

## A9 — as APIs novas que substituem conta à mão (opcional, barato)

- **1.96 `assert_matches!`** — **236** ocorrências de `assert!(matches!(…))` na árvore.
- **1.97 `bit_width`** — 2 sítios: [`fx_stack_plan.rs:17`](../../crates/ph2d-render/src/fx_stack_plan.rs)
  e [`mipgen.rs:27`](../../crates/ph2d-render/src/mipgen.rs).

⛔ **Faça isto DEPOIS do bloco A fechar verde, e num commit próprio.** Misturar uma limpeza de 236
sítios com a subida da toolchain torna impossível dizer qual das duas partiu algo.

## A10 — ⛔ o que NÃO usar do Rust 1.98

O 1.98 estabilizou `f32::algebraic_add/sub/mul/div/rem`. Elas deixam o compilador reordenar contas de
ponto flutuante — mais rápido, **e não-determinístico**.

⛔ **Proibidas em qualquer caminho que o `physics_ecs_c9` atravesse**, e em qualquer coisa que o
replay-hash compare entre os três sistemas operacionais. O ADR-0131 e o HR-5 dependem de a resposta
ser *a mesma*, não *rápida*.

**Fazer:** nada. Esta tarefa existe para a proibição ficar escrita **antes** de alguém as descobrir
num perfil. Se um dia forem usadas em caminho puramente visual, o ADR tem de nomear o caminho.

## A11 — a suíte

**Fazer:** `CARGO_INCREMENTAL=0 cargo nextest run --workspace --no-fail-fast`

⚠️ **`CARGO_INCREMENTAL=0`**: o perfil `ci-test` roda em batch; a compilação incremental não colhe
nada e paga **11 GB** (medido 2026-08-16).

**Verificar:** compare com o `T3`. **O conjunto de vermelhos tem de ser o MESMO.**
⚠️ Se aparecer um vermelho novo, **re-rode-o sozinho antes de suspeitar do Rust** — a família de
flakes de recurso sob fan-out está catalogada no §5.0 do `CLAUDE.md`, e o sinal dela é o conjunto de
reprovadas **mudar entre corridas do mesmo binário**.

---

# Bloco B — as 31 compatíveis (4 tarefas) · 🟢

## B1 — subir

**Fazer:** `cargo update` (sem `-p`: nenhuma delas muda de major, então o `^` já as autoriza).
**Verificar:** `git diff --stat Cargo.lock` — só o lock muda; nenhum `Cargo.toml`.

## B2 — ler o diff do lock

**Por quê:** um `cargo update` também sobe **transitivas** que ninguém pediu. É onde uma surpresa entra.
**Fazer:** `git diff Cargo.lock | grep '^[-+]version' | sort | uniq -c | sort -rn | head -40`
⚠️ Procure especificamente por `wgpu`, `vello`, `kurbo`, `skrifa`, `glam`, `bevy_*`, `nalgebra`:
**nenhum deles deve mover neste bloco.** Se moveram, o `^` de alguém é mais largo do que o `01` diz.

## B3 — a prova

**Fazer:** `cargo check --workspace --all-targets` e depois
`CARGO_INCREMENTAL=0 cargo nextest run --workspace --no-fail-fast`.
**Verificar:** mesmo conjunto de vermelhos do `T3`.

## B4 — licenças e CVEs

**Fazer:** `cargo deny --all-features check` e `cargo audit`.
**Por quê aqui:** 31 crates novas trazem licenças e avisos novos. Descobrir isso no `ship.sh` do
bloco G é descobrir tarde.
**Recuo:** `git checkout Cargo.lock` desfaz o bloco inteiro.

---

# Bloco C — GPU e texto (20 tarefas) · 🔴 **muda pixel**

**As seis crates deste bloco sobem JUNTAS ou não sobem.** `vello`, `wgpu`, `naga`, `skrifa`,
`parley`, `fontique` estão amarradas (`01` §3). Subir uma sozinha dá duas cópias e o `vello` recusa
o nosso `Device`.

⛔ **Este bloco não fecha em teste verde.** Ver [`03_riscos.md`](03_riscos.md).

## C1 — a fotografia dos pixels, ANTES de tocar em qualquer coisa

**Por quê:** três mudanças deste bloco alteram a imagem e **compilam perfeitamente**. Sem o antes,
não há como dizer se um golden mudou porque melhorou ou porque quebramos.

**Fazer:**
```
cd /home/enio/Documentos/Projetos/PH2D
mkdir -p "docs/Atualizar Stack/_pixels_antes"
# 1. os goldens que já existem no repo
find crates shells -name '*.snap' -o -name '*golden*' | sort > "docs/Atualizar Stack/_pixels_antes/inventario.txt"
# 2. os despejos que as ferramentas de diagnóstico já sabem produzir
PH2D_PREVIEW_DUMP="docs/Atualizar Stack/_pixels_antes/preview" cargo run -p ph2d-host-desktop --release
PH2D_WALK_DUMP="docs/Atualizar Stack/_pixels_antes/walk"       cargo run -p ph2d-host-desktop --release
```
⚠️ **Rode também as cenas de smoke que usam vello com IMAGEM e GRADIENTE** — são as três que mudam:
o seletor de cor (gradiente), qualquer pré-visualização filtrada (`ph2d-host/src/filter.rs` mapeia
o nosso «Linear» para `ImageQuality::High`, que virou bicúbico), e qualquer coisa com `draw_image`
(45 sítios).

**Verificar:** a pasta `_pixels_antes/` tem conteúdo e o `inventario.txt` tem as **61** linhas de
ficheiros com golden.

⛔ **`_pixels_antes/` não entra em commit.** Acrescente ao `.gitignore` da pasta.

## C2 — vello 0.8 → 0.10.0
**Onde:** [`crates/ph2d-render/Cargo.toml:39`](../../crates/ph2d-render/Cargo.toml) ·
[`crates/ph2d-vector/Cargo.toml:24`](../../crates/ph2d-vector/Cargo.toml)
⚠️ O segundo declara `default-features = false, features = ["wgpu"]`. **Mantenha as features** — é
o que amarra a versão do wgpu.

## C3 — wgpu 28 → 29.0.4 nas 10 declarações
**Onde:** `ph2d-flip-render`, `ph2d-gpu`, `ph2d-gpu-cook`, `ph2d-inpaint`, `ph2d-mesh-render`,
`ph2d-paint-gpu`, `ph2d-render`, `ph2d-tool-color-equalization`, `ph2d-tool-painter`, `shells/desktop`.
⚠️ Duas delas pedem `features = ["wgsl"]` — preserve.
⛔ **Não escreva `29`. Escreva `29.0.4`** — o `vello` pede `^29.0.3`, e um `29.0.0` resolvido por
acaso não o satisfaz.

## C4 — naga 28 → 29.0.4
**Onde:** `ph2d-gpu-cook`, `ph2d-render` (só-dev). **Tem de casar com o wgpu.**

## C5 — skrifa 0.40 → 0.44.0 ⛔ **não 0.46**
**Onde:** [`crates/ph2d-vector-font/Cargo.toml:20`](../../crates/ph2d-vector-font/Cargo.toml)
⚠️ O comentário ali diz *«já na árvore via parley/vello (0.40)»*. **Atualize o comentário para 0.44
e diga que é TETO**, com o dono — senão a próxima pessoa sobe para 0.46 e passa uma semana a
descobrir por quê não pode.

## C6 — parley 0.6 → 0.11.1 · fontique 0.6 → 0.11.1
**Onde:** [`crates/ph2d-text/Cargo.toml:16`](../../crates/ph2d-text/Cargo.toml) ·
[`crates/ph2d-system-fonts/Cargo.toml:22`](../../crates/ph2d-system-fonts/Cargo.toml) · `shells/desktop`.

## C7 — accesskit: **fixar em 0.24.1 e escrever por quê**
**Onde:** [`crates/ph2d-a11y/Cargo.toml:22`](../../crates/ph2d-a11y/Cargo.toml)
**Fazer:** `"0.24"` → `"0.24.1"`, com comentário: *«TETO — o `parley 0.11.1` pede `^0.24.0`. Duas
cópias de accesskit dão dois `NodeId` incompatíveis na fronteira. Reconfira com
`bash scripts/stack-audit.sh --tetos` antes de subir.»*

## C8 — `get_current_texture()` devolve outro tipo
**O quê:** era `Result<SurfaceTexture, SurfaceError>`, virou o enum `CurrentSurfaceTexture`; o campo
`suboptimal` virou uma variante `Suboptimal`.
**Onde:** **5** ocorrências. `grep -rn 'get_current_texture' crates shells --include='*.rs'`
⚠️ Onde hoje se ignora `suboptimal`, a variante nova **força** uma decisão. Ler o que fazemos hoje
antes de escolher: reconfigurar a surface ou desenhar mesmo assim.

## C9 — `bind_group_layouts` virou `&[Option<&BindGroupLayout>]`
**Onde:** **28** ocorrências.
**Fazer:** envolver cada elemento em `Some(...)`. Mecânico.

## C10 — `VertexState::buffers` virou `&[Option<VertexBufferLayout>]`
**Onde:** **34** ocorrências de `buffers:`.
⚠️ **Nem todo `buffers:` é do `VertexState`.** Confira o contexto de cada um — um `sed` cego aqui
edita structs alheias e o erro só aparece 20 minutos depois.

## C11 — `depth_write_enabled` / `depth_compare` viraram `Option`
**Onde:** **10** ocorrências.

## C12 — `BufferView` / `QueueWriteBufferView` perderam o lifetime; leitura proibida em mapeado só-escrita
**Onde:** **0** ocorrências medidas hoje. **Tarefa de confirmação, não de edição** — reconfira com
`grep -rn 'BufferView\|map_async\|WriteOnly' crates shells --include='*.rs'` antes de riscar.

## C13 — WGSL: varyings inteiros precisam de `@interpolate(flat)`
**O quê:** o wgpu 29 deixou de assumir `flat` para entradas/saídas inteiras de shader.
**Onde:** **3** ocorrências em **18** ficheiros `.wgsl`.
```
grep -rnE '@location\([0-9]+\) *[a-z_]+ *: *(u32|i32|vec[234]<[ui]32>)' crates shells --include='*.wgsl'
```
⚠️ **Este é o único item do bloco que falha em RUNTIME, não na compilação do Rust** — o shader é
compilado pelo naga na hora de criar o pipeline. Se um passe morrer com erro de validação de shader,
é aqui.

## C14 — limites e features renomeados
`max_inter_stage_shader_components` → `max_inter_stage_shader_variables` · `SHADER_PRIMITIVE_INDEX`
→ `PRIMITIVE_INDEX`. **0 ocorrências hoje** — confirmar, não editar.

## C15 — parley: `Glyph::y` INVERTEU de Y-up para Y-down
⚠️⚠️ **Este é o item mais perigoso do bloco, porque ele compila.** Código que **subtraía** `glyph.y`
tem de **somar**. Um texto desenhado de cabeça para baixo ou deslocado é o sintoma.
**Onde:** `crates/ph2d-editor-core/src/paint.rs` faz `vello::Scene::draw_glyphs` por `GlyphRun` do
parley — comece por aí. `grep -rn 'glyph' crates/ph2d-editor-core/src/paint.rs crates/ph2d-text`

## C16 — parley: renomes de alinhamento
`Alignment::Justified` → `Justify` · `Alignment::Middle` → `Center`. Erro de compilação — seguro.

## C17 — parley: saiu do `peniko`/`kurbo`
`peniko::Font` → `parley::FontData` · `kurbo::Rect` → `parley::BoundingBox`.
⚠️ Onde a nossa cola converte entre parley e vello, agora há **dois** tipos de fonte no caminho.

## C18 — parley: `Layout::align`, `LineMetrics`, `InlineBox`
- `Layout::align` não recebe mais `width` (usa o `max_advance` do `break_all_lines()`);
- `LineMetrics`: `min_coord` → `block_min_coord`, `max_coord` → `inline_max_coord`;
- `InlineBox` exige `InlineBoxKind` (`InFlow` reproduz o comportamento antigo).

## C19 — a prova de que as cópias colapsaram
**Por quê:** o ganho estrutural do bloco é justamente este, e ele é fácil de perder por engano.
```
python3 - <<'PY'
import re
t=open('Cargo.lock').read()
for c in ('kurbo','skrifa','peniko','wgpu-types','wgpu','vello','naga','read-fonts','accesskit'):
    v=re.findall(r'\[\[package\]\]\nname = "%s"\nversion = "([^"]+)"'%c,t)
    print(f'{c:12} {v}')
PY
```
**Verificar:** `kurbo`, `skrifa`, `peniko`, `wgpu-types` com **UMA** versão cada.
⚠️ Hoje (antes) são **duas** de `kurbo` (0.11.3 + 0.13.0), **duas** de `skrifa` (0.37 + 0.40) e
**duas** de `wgpu-types` (27.0.1 + 28.0.0).

## C20 — compila e roda
`cargo check --workspace --all-targets` → `CARGO_INCREMENTAL=0 cargo nextest run --workspace --no-fail-fast`.
⛔ **Espere vermelho.** A triagem é o `03_riscos.md`, não este documento.

## C21 — os gates de GPU, que o CI nunca roda
⚠️ Os gates de GPU são `#[ignore]` e precisam de adaptador — *pular em silêncio não é verde*.
```
cargo nextest run --workspace --run-ignored all --no-fail-fast -E 'test(/gpu|device|adapter|parity/)'
```
⚠️ E os `--ignored` do Painter pedem `--test-threads=1` com a máquina calma.

## C22 — o smoke do Enio
**Comandos, inteiros e copiáveis** (§0.8 — cada um com o `cd`):
```
env PH2D_BUILD_SMOKE=1 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_GPU_COOK_DEMO=1 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
**O que ele tem de olhar** (e o `03_riscos.md` §2 diz o que esperar):
1. o **seletor de cores** — o gradiente mudou de espaço de mistura;
2. qualquer **pré-visualização filtrada** (Remoção de Fundo) — «alta qualidade» virou bicúbico;
3. **texto** em qualquer painel — o parley inverteu o eixo Y do glifo;
4. qualquer coisa com **imagem** no canvas — meio pixel de deslocamento.

---

# Bloco D — `bevy_ecs` 0.18 → 0.19 (14 tarefas) · 🟡 grande e mecânico

⚠️ **185 ficheiros.** Mas é **um** salto de versão, e quase tudo é renome com erro de compilação.
O trabalho é volume, não risco.

⚠️ **A mudança de fundo da 0.19: recursos passaram a ser componentes.** `#[derive(Resource)]` agora
também implementa `Component`, e recursos vivem em entidades dedicadas. Isso tem uma consequência
que **não** é renome — a **D4**.

## D1 — subir nas 8 declarações
`ph2d-ecs`, `ph2d-field-ecs`, `ph2d-physics-ecs`, `ph2d-render`, `ph2d-script`, `ph2d-timeline`,
`shells/desktop`, `tests/spike`. `"0.18"` → `"0.19"`.

## D2 — ⛔ nenhum tipo pode derivar `Component` **e** `Resource`
**O quê:** virou proibido. O padrão do upstream é partir em dois (`GlobalX` recurso + `X` componente).
**Fazer:** `grep -rn 'derive(.*Resource' crates shells --include='*.rs'` e cruzar com `Component`.

## D3 — `ReflectResource` virou vazio
`#[reflect(Resource)]` agora reflete o trait `Component`; use `ReflectComponent`.

## D4 — ⚠️ consultas largas passaram a CONFLITAR com acesso a recurso
**O quê:** `Query<()>`, `Query<Entity>`, `Query<Option<&T>>` agora colidem com quem lê recursos,
porque os recursos são entidades.
**Cura:** acrescentar `Without<IsResource>` (ou `Without<MyResource>`) ao filtro.
⚠️ **Este é o único item do bloco que pode mudar COMPORTAMENTO em vez de só não compilar** — uma
consulta larga que passa a ver entidades-recurso itera sobre coisas que não são do jogo.
**Fazer:** achar toda `Query<Entity` e `Query<()` e decidir uma a uma.

## D5 — recursos não-`Send` mudaram de nome
`init_non_send_resource` → `init_non_send` · `insert_non_send_resource` → `insert_non_send` ·
`non_send_resource_mut` → `non_send_mut` · `get_non_send_resource_mut` → `get_non_send_mut`.

## D6 — registro de componente
`register_resource` → `register_component` · `resource_id` → `component_id` ·
`ComponentDescriptor::new_resource` → `::new` · `register_resource_unchecked` → `register_non_send_unchecked`.
(Os antigos ficaram como *deprecated* — mas o nosso `-D warnings` transforma isso em erro.)

## D7 — a API de `Access` consolidou
`add_component_read`/`add_resource_read` → `add_read` · idem `write` · `has_component_read` →
`has_read` · `read_all_components` → `read_all` · `is_components_compatible` → `is_compatible` ·
`is_subset_components` → `is_subset`. **Removidos:** `read_all_resources`, `write_all_resources`,
`is_resources_compatible`, `is_subset_resources`.

## D8 — recursos agora podem ser imutáveis: código genérico precisa de limite
```
fn s<R: Resource>(mut r: ResMut<R>)                        // 0.18
fn s<R: Resource<Mutability = Mutable>>(mut r: ResMut<R>)  // 0.19
```

## D9 — `System::type_id()` → `System::system_type()`

## D10 — o executor mudou de forma
`ExecutorKind` **removido**; `Schedule::set_executor_kind(kind)` → `set_executor(instância)`
(`SingleThreadedExecutor::new()`, `MultiThreadedExecutor::new()`, `default_executor()`).
⚠️ **Confira se alguma cena de smoke ou gate de determinismo força o executor single-threaded** —
o `physics_ecs_c9` depende de ordem.

## D11 — `Ref<T>` virou `Clone + Copy`
⚠️⚠️ **Muda semântica sem erro de compilação:** `ref.clone()` agora devolve `Ref<T>`, **não** o `T`
clonado. Onde alguém escrevia `let v = r.clone();` esperando o valor, agora tem uma referência.
**Fazer:** `grep -rn '\.clone()' ` nos ficheiros que usam `Ref<` e conferir um a um.

## D12 — `Access::archetypal` devolve `&ComponentIdSet`, não um iterador
Acrescentar `.iter()` onde for iterado.

## D13 — miscelânea
`World::entities_allocator[_mut]` → `entity_allocator[_mut]` ·
`World::remove_resource_by_id` devolve `bool` (era `Option<()>`) ·
`World::clear_entities()` agora limpa recursos também ·
`DefaultErrorHandler` → `FallbackErrorHandler` · `#[derive(MapEntities)]` já não é preciso em recursos.

## D14 — a prova
`cargo check --workspace --all-targets` → suíte → **e o `physics_ecs_c9`**, que atravessa a ECS:
```
cargo run --release --locked --quiet --bin physics_ecs_c9 -p ph2d-physics-ecs
```
⚠️ **O valor vai mudar, e isso está certo.** O gate compara os **três sistemas operacionais entre
si**, não contra um número guardado — então o que importa é rodar **duas vezes** localmente e obter
o mesmo. A discordância entre OS só o CI mede.

---

# Bloco E — `rapier2d` 0.28 → 0.35 (14 tarefas) · 🔴 **muda o tato da física**

⚠️ **Sete versões.** 47 ficheiros, 145 menções, uma crate declarante (`ph2d-physics`).

⛔ **REFUTADO — não planeje para isto:** o `CHANGELOG` do `master` anuncia a migração de `nalgebra`
para `glam` na 0.32. **Nenhuma versão publicada faz isso** (`01` §7). Continuamos em `nalgebra`, que
sobe de 0.34 para 0.35.

## E1 — subir
**Onde:** [`crates/ph2d-physics/Cargo.toml:22`](../../crates/ph2d-physics/Cargo.toml)
⚠️ **Preserve as features:** `default-features = false, features = ["dim2", "f32", "enhanced-determinism"]`.
O `enhanced-determinism` é o que o HR-5 e o [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) exigem.

## E2 — as features de SIMD sumiram (0.35)
`simd-stable`, `simd-nightly`, `simd-is-enabled` **não existem mais** — SIMD é sempre ligado, via `wide`.
Se alguma delas estiver no nosso `Cargo.toml` ou num `cfg`, tirar.

## E3 — `IntegrationParameters::min_island_size` foi removido (0.35)

## E4 — o CCD foi reescrito (0.35)
`CCDSolver::predict_impacts_at_next_positions` / `clamp_motions`, `PredictedImpacts` e `TOIEntry`
foram substituídos por `CCDSolver::solve_continuous`.

## E5 — ⚠️ **os padrões de contato mudaram por um fator de 10** (0.35)
| parâmetro | era | virou |
|---|---|---|
| `normalized_prediction_distance` | 0,002 | **0,02** |
| `normalized_max_corrective_velocity` | 10,0 | **3,0** |

**Fazer:** decidir explicitamente — aceitar o novo padrão (é a recomendação do upstream) ou fixar o
antigo. ⛔ **Não deixe acontecer por omissão.** Registre a escolha no `04_registro.md`.

## E6 — ⚠️ **o adormecer virou por ilha, e os limiares mudaram** (0.35)
`normalized_linear_threshold` 0,4 → **0,05** · `time_until_sleep` 2,0 s → **0,5 s**.
Os formatos de serialização de `IslandManager` e `RigidBodyActivation` mudaram.
⚠️ **Corpo que hoje continua acordado pode dormir em meio segundo.** É visível: uma peça que parava
de tremer sozinha agora congela mais cedo — ou o contrário.

## E7 — ⚠️ **teto de velocidade novo** (0.35)
Corpos agora são limitados a `max_linear_velocity()` (padrão **400 unidades/s**) e a um teto angular.
⚠️ **Isto pode cortar comportamento que hoje existe.** Se algo do projeto voa rápido (um projétil,
uma explosão, o player numa rampa), **meça antes e depois**. O `pixels_per_meter` do projeto decide
se 400 é muito ou pouco.

## E8 — `SolverContact` mudou de forma (0.35)
Guarda âncoras por corpo em vez de um ponto no mundo; atrito e restituição passaram a ser por
manifold. **Toca quem lê contatos** — e nós lemos (`SignalOnHit`, o R0).

## E9 — `additional_solver_iterations` mudou de significado (0.35)
Agora acrescenta **sub-passos inteiros**, não iterações. Um número que era afinado passa a valer mais.

## E10 — `InteractionGroups` precisa de `InteractionTestMode` (0.31)
O construtor mudou. `InteractionTestMode::And` reproduz o comportamento anterior.
⚠️ **Toca as camadas de colisão**, que são feature do produto (§5, Física).

## E11 — a maciez do contato consolidou (0.31)
`contact_natural_frequency` + `contact_damping_ratio` → um `contact_softness`. Os métodos `erp`/`cfm`
foram removidos. Maciez por junta agora é `GenericJoint::softness`.
⚠️ **As juntas são ENTIDADES aqui** ([ADR-0149](../architecture/decisions/0149-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md)) —
7 tipos + polia/talha/tambor + pino de mundo. Cada uma tem de ser reconferida.

## E12 — `CoefficientCombineRule` (0.31) e o modelo de atrito (0.29)
`Min` passou a usar o resultado não-zero via `abs()`; entrou `ClampedSum`.
O modelo de atrito simplificado é o **padrão** desde a 0.29 (`IntegrationParameters::friction_model`).
`pgs_legacy` e `tgs_soft_without_warmstart` foram **removidos**.

## E13 — `*angular_inertia_sqrt` sumiu (0.29/0.30) e o `Voxels` do parry mudou (0.30)

## E14 — a prova, e ela é dupla
**Máquina:**
```
cargo run --release --locked --quiet --bin physics_ecs_c9 -p ph2d-physics-ecs   # 2×, tem de dar igual
CARGO_INCREMENTAL=0 cargo nextest run -p ph2d-physics -p ph2d-physics-ecs --no-fail-fast
```
**Enio** — o tato só ele mede:
```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_PHYSICS_SMOKE=6 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_PHYSICS_SMOKE=67 cargo run -p ph2d-host-desktop --release
```
⚠️ **`PH2D_PHYSICS_SMOKE=84` não existe, de propósito** (§5 do `CLAUDE.md`).

---

# Bloco F — a cauda (19 tarefas) · 🟡 uma de cada vez

**Regra do bloco:** cada tarefa é **um commit próprio**. São 19 crates independentes; misturá-las
faz um `git bisect` inútil no dia em que uma delas partir algo.

⚠️ **Para as que este plano não pesquisou em detalhe, o passo 1 é sempre o mesmo:** ler o
`CHANGELOG` da crate entre a nossa versão e a de destino. Está anotado onde é o caso — e é uma
instrução honesta, não uma lacuna: pesquisar 19 changelogs agora produziria notas envelhecidas antes
de alguém pegar a tarefa.

### F1 — `glam` 0.30 → 0.33.6 · 32 ficheiros · 8 crates
`ph2d-anim`, `-core`, `-mesh-render`, `-vec-edit`, `-vector`, `-vector-doc`, `-vector-font`, `-vector-traits`.
**Mudanças:** 0.31 — assinatura de `Quat::from_affine3` mudou (entrou o tipo `Affine3`); `angle_between`
de `Vec2`/`DVec2` **removido**; assinaturas passaram a usar `&self` para matrizes/afins e `self` para
vetores/quats. 0.32 — só o `rand` opcional. 0.33 — tipos além de `f32`/`bool` viraram opcionais
(ligados por padrão).
⚠️ **`glam` está no caminho do determinismo** (é o tipo de `Transform`). Rodar o `physics_ecs_c9` **2×**
depois.

### F2 — `rfd` 0.15 → 0.17.2 · 20 ficheiros · `shells/desktop`
Diálogos de ficheiro — **feature nova de 2026-08-23** (`Save As…`, `Open Project…`).
⚠️ **Ela segura o `pollster` em 0.4** (`01` §3). Ler o `CHANGELOG` do rfd.
**Verificar:** os três itens do menu de projeto abrem diálogo de verdade e o filtro mostra `.ph2dproj`.

### F3 — ⛔ `pollster`: **fica em 0.4** · 14 ficheiros
**Não subir.** O `rfd 0.17.2` pede `^0.4`. **Fazer:** trocar `"0.4"` por `"0.4.0"` nas 5 declarações
com o comentário do teto e o dono.

### F4 — `mlua` 0.10 → 0.12.0 · 11 ficheiros, 75 menções · `ph2d-script`, `tests/spike`
⚠️ **Contexto que muda a prioridade:** o `ScriptHost` do desktop corre um script *placeholder* e
**nunca** recebe `provide_read` (§5, Sprite Inspector). A superfície é real, a feature está parada.
**Ler o `CHANGELOG` do mlua** (0.11 e 0.12 mexeram em `Lua::new`/escopos).

### F5 — `linesweeper` 0.3 → 0.4.0 · 4 ficheiros · `ph2d-vec-boolean`
⚠️ **Esta crate é o motor da booleana viva** (§5, Vector — «um verbo por forma»).
**Verificar:** a suíte de `ph2d-vec-boolean` **inteira**, e a cena de smoke da booleana.
⚠️ O gate `a_round_live_offset_costs_like_the_other_joins` dessa crate é **membro conhecido da
família de flakes de carga** — se ele for o único ✗, re-rode sozinho antes de culpar o salto.

### F6 — `ctt` 0.4.0 → 0.5.0 · 4 ficheiros, 28 menções · `tools/asset-cooker`
⚠️ Segura o `miniz_oxide` em `^0.8`. Ler o `CHANGELOG`.

### F7 — ⛔ `miniz_oxide`: **fica em 0.8** · 3 crates
Preso por `ctt`, `exr` e `png` (`^0.8`), enquanto `flate2` pede `^0.9`. **Duas cópias, e tudo bem** —
é folha de compressão, não atravessa a nossa superfície. **Fazer:** comentário do teto nas 3 declarações.

### F8 — `cpal` 0.15 → 0.18.2 · 3 ficheiros, 17 menções · `shells/desktop`
**Três majors.** É a saída de áudio real.
⚠️ **Cerca conhecida:** «elos verdes = elo FORA» — um canal mudo pode estar mudo no PipeWire, fora do
processo ([memória](../../project-memory/feedback_a_silent_output_channel_can_be_muted_outside_the_process.md)).
**Verificar:** som saindo de verdade, não o log dizer que saiu. **Ler o `CHANGELOG`.**

### F9 — `taffy` 0.12 → 0.14.0 · 3 ficheiros · `ph2d-vec-layout`
Motor do auto layout ([ADR-0153](../architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md)).
⚠️ **A lei do ADR:** *o passe publica onde as coisas ficam; não escreve onde elas estão.* Nada no auto
layout toca `Transform`. **Se o salto tentar levar você a tocar, pare** — vira ADR.

### F10 — ⛔ `ndarray`: **fica em 0.15** · `ph2d-audio-ml`
Preso pelo `deep_filter` vendorizado (`^0.15`), que está **fora** do workspace de propósito
([ADR-0123](../architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md)).
Subir dá duas cópias **e** parte a fronteira por onde passamos arrays.
**Fazer:** comentário do teto, nomeando o `deep_filter`. ⚠️ O `stack-audit.sh` **não vê este teto**
(não varre `vendor/`) — por isso ele está escrito aqui.

### F11 — `criterion` 0.7 → 0.8.2 · 3 ficheiros · **só-dev**
Benches. Risco baixo; se resistir, **adiar é aceitável** e registrar.

### F12 — `zip` 2 → 8.6.0 · 3 ficheiros · `ph2d-imageio-ora`
**Seis majors, 3 menções.** Provavelmente meia hora. **Ler o `CHANGELOG`.**
**Verificar:** abrir e gravar um `.ora` de verdade.

### F13 — `symphonia` 0.5 → 0.6.1 · 2 ficheiros, 18 menções · `ph2d-audio-decode`
**Verificar:** decodificar um de cada formato que suportamos.

### F14 — `toml` 0.9 → 1.1.4 · 2 ficheiros · **só-dev**
`ph2d-tool-registry-init`, `tools/asset-cooker`.

### F15 — `wasmtime` 47 → 48.0.1 · 2 ficheiros · `tests/spike`
⚠️ **MSRV 1.95** — só possível depois do bloco A. Só o spike usa.
⛔ **Confirme que isto não reabre WASM como caminho de produto** — o [ADR-0075](../architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)
**rejeitou** plugin em runtime e WASM. Se a atualização começar a pedir mudanças fora do spike, **pare**.

### F16 — `usvg` 0.43 → 0.48.1 · 1 ficheiro · `ph2d-imageio-svg`
⚠️ **Ele também segura o `skrifa` em `^0.44`** — sobe **junto com o bloco C** ou depois dele, nunca antes.
**Verificar:** importar um `.svg` com texto e com gradiente.

### F17 — `jxl-oxide` 0.10 → 0.12.6 · 1 ficheiro · `ph2d-imageio-jxl`
**Verificar:** abrir um `.jxl`.

### F18 — `roxmltree` 0.20 → 0.21.1 · 1 ficheiro · `ph2d-imageio-ora`
Faz par com o **F12** (mesma crate). Podem ir no mesmo commit.

### F19 — ⛔ `core-graphics`: **fica em 0.23.2** · `shells/desktop`
Preso pelo `winit 0.30.13` (`^0.23.1`). Só macOS. **Fazer:** `"0.23"` → `"0.23.2"` + comentário do teto.
⚠️ **Este teto cai sozinho** quando o `winit 0.31` sair de beta. Anotar no `04_registro.md` §4.

---

# Bloco G — o fecho (6 tarefas)

### G1 — o ADR
**Onde:** `docs/architecture/decisions/0168-*.md` (o último em 2026-08-29 é o **0167**).
**Título sugerido:** *«O stack sobe até os TETOS, e quatro dependências ficam deliberadamente para trás»*.
**Tem de conter:** os 8 tetos com o dono de cada um · **por que o wgpu 30 foi recusado** · a lista de
alternativas rejeitadas (forçar duas cópias; abandonar o vello; congelar tudo) · e a data em que cada
teto deve ser reconferido.
⚠️ **A recusa do wgpu 30 é o item mais importante do ADR** — sem ele escrito, alguém tenta de novo em
dois meses e paga a mesma semana.

### G2 — o `stack-audit.sh` vira passo, não ponteiro
**Onde:** [`scripts/ship.sh`](../../scripts/ship.sh).
**Fazer:** acrescentar como `run_optional`, **informativo** (nunca reprovando) — o ship não pode ficar
vermelho porque o mundo publicou uma versão nova.
⚠️ **Isto é o que separa uma ferramenta viva de um ponteiro morto** (§2 do `CLAUDE.md`).

### G3 — o `ship.sh` inteiro
```
./scripts/ship.sh
```
⛔ **Corrija todo `✗` antes de qualquer push.** ⚠️ E lembre: um `✗` do ship pode ser **ambiente**, não
código ([memória](../../project-memory/feedback_a_ship_x_can_be_the_environment_not_the_code.md)).

### G4 — as 6 worktrees
As linhas continuam em `main` **anterior**. Depois de o `main` receber a atualização, cada worktree
que voltar à vida precisa de rebase — e o `target/` dela está **frio**.
**Fazer:** anotar isto no `04_registro.md` §5 e avisar o Enio. ⛔ **Não rebase worktree alheia sem
ordem** (§0.2).

### G5 — o `CLAUDE.md`
**Uma linha** no §5, e a linha do §1 que a **T5** já criou.
⛔ **Uma linha.** A narrativa desta jornada vive **aqui**, em `docs/Atualizar Stack/`, não lá.

### G6 — a memória
Candidatas a `project-memory/`, uma por arquivo:
- *«O changelog do `master` de uma crate descreve o futuro; o índice descreve o que se instala»* —
  a refutação do `glam` no rapier (`01` §7).
- *«Um alarme cuja condição perigosa tem duas metades não pode reprovar por uma só»* — o `btrfs-health`.
- *«"O mais recente possível" ≠ "o mais recente": conte os TETOS antes de planejar»* — com o ponteiro
  para `scripts/stack-audit.sh`.

⚠️ **Antes de escrever, confira se já existe arquivo que cubra** — o índice `MEMORY.md` tem teto de
~17 KB e a regra é atualizar, não duplicar.
