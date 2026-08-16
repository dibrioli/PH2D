---
name: feedback_in_use_is_answered_by_the_global_config_and_a_probe_can_start_what_it_measures
description: "\"Esta ferramenta está em uso?\" se responde na config GLOBAL, não na do repo — e uma sonda que INICIA o serviço mede a sessão que ela mesma criou"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4d854466-7014-4e58-98f6-b338a1e78fc3
  modified: 2026-08-15T11:43:23.810Z
---

Em 2026-08-15 apaguei **101 GB de `~/.cache/sccache`** concluindo que ninguém o
usava. Ele estava **ativo**, e a `DIRETIVA_FIM_DE_DIA §3` proíbe apagá-lo em
letras maiúsculas (*"o instinto 'limpar caches pra liberar memória' está ERRADO
aqui"*). Dois erros de método, e os dois se repetem fora do sccache.

**1. Perguntei à config ERRADA.** Conferi `RUSTC_WRAPPER` no ambiente e o
`.cargo/config.toml` **do repo** — os dois vazios — e li isso como *"não está
em uso"*. Mas o cargo lê `build.rustc-wrapper` do **`~/.cargo/config.toml`
GLOBAL**, que tinha `rustc-wrapper = "sccache"` + `SCCACHE_CACHE_SIZE = "100G"`.

⚠️ **E o repo DIZ que é assim, em dois lugares que eu tinha lido:** o CLAUDE.md
§2 (*"Linker = mold … **nunca no `.cargo/config.toml` do repo** — global"*) e o
`scripts/target-on-tmpfs.sh` (*"survives reboot via sccache (see
**~/.cargo/config.toml**)"*). Nesta máquina a config de build é global **por
decisão** — pinar linker/target-dir no repo já quebrou o CI de Mac/Windows.

**2. A sonda INICIOU o que ela media.** `sccache -s` **sobe o servidor** se ele
não estiver de pé, e as estatísticas são da **sessão daquele servidor**, não
históricas — então ele reportou `Compile requests 16 / hits 0`, que eu li como
*"ninguém usa"* quando significava *"este servidor nasceu agora"*. Pior: aquele
start **tocou o diretório**, e foi o que produziu o "último uso: hoje" que
contradizia o meu próprio `find -newermt "7 days ago"` — eu vi a contradição,
atribuí à minha sonda, e **não voltei para refazer a conclusão que ela apoiava**.

⚠️ **E `mtime` não mede uso de um cache LRU:** um hit é uma LEITURA, e leitura
não move mtime. *"0 arquivos tocados em 7 dias"* é compatível com um cache
servindo perfeitamente.

**Why:** o dano não é o espaço, é o tempo — o sccache serve os deps **entre as
worktrees** (hit ~78% num target frio), e sem ele cada linha recompila tudo (o
comentário do config mede: *6× o build, 836 GB de target somados*). Combinado
com eu ter apagado o `incremental/` das cinco, o próximo build de cada linha
ficou duplamente frio.

**How to apply:** antes de apagar QUALQUER cache, (a) `cat ~/.cargo/config.toml`
e `git grep -rn <ferramenta> scripts/ docs/` — a proibição costuma estar escrita
—, e (b) desconfie de toda sonda cujo comando possa **criar** o estado que ela
observa (`<serviço> -s`, `--status`, um client que auto-inicia o daemon). Se a
sonda pode ligar o serviço, ela não pode responder *"o serviço estava em uso?"*.
Reparação parcial: o cache se repovoa sozinho, e **um** build completo serve as
outras worktrees — não são cinco builds frios, é um frio e quatro mornos.
Ver [[project_modo_l_speed_hole_worktree_targets_slow_path]] e
[[feedback_a_ship_x_can_be_the_environment_not_the_code]].
