---
name: feedback_the_shader_cache_is_trimmed_by_any_process_without_the_cap
description: "O GLCache da NVIDIA é UM para todos; qualquer processo sem __GL_SHADER_DISK_CACHE_SIZE apara-o a 1 GiB e o app do dono volta a abrir a frio; e nunca `systemctl --user import-environment` sem nomes"
metadata:
  node_type: memory
  type: feedback
  originSessionId: bf30ab65-f35b-4c33-ab77-5f7b6882fe36
  modified: 2026-10-13T23:31:43.718Z
---

O `~/.cache/nvidia/GLCache` é partilhado por todos os programas. O `[env]` do `.cargo/config.toml` (16 GiB) só vale
para o que corre pelo cargo: um binário corrido direto (`./target/smoke/ph2d-host-desktop`, o `fotografa_cena.sh`
até 13/10) ou os testes de uma linha cuja base não tem o `[env]` correm com o tecto de fábrica (1 GiB) e APARAM o
cache — a luz do dono, aquecida, abriu a frio minutos depois (2,18 → 0,05 → 1,56 → 0,05 → 2,33 s, W5p 13/10; o
dono: «não vi melhoria»). Cura feita com o sim dele: `~/.config/environment.d/60-nvidia-shader-cache.conf`.

⛔ `systemctl --user import-environment` SEM argumentos copia o ambiente inteiro do agente para o gestor da sessão
do dono (`ELECTRON_RUN_AS_NODE=1`, `CLAUDE_*`, `VSCODE_*`) — desfazer um nome de cada vez (o zsh não parte
`$V` em palavras: use `bash -c '… | while read v'`).

**Why:** um «aquecer o cache do dono» no fecho de uma linha vale ZERO se outro processo o apara depois; e o
ambiente da sessão do dono é dele.

**How to apply:** antes de dizer «o cache está quente», abra o app 2× com `PH2D_LIGHT2D_TEMPO=1` e veja o
`du -sm ~/.cache/nvidia/GLCache` (colado em ~1 024 = alguém apara). Variável de sessão: `set-environment NOME=valor`,
nunca `import-environment` cru. Relacionado: [[project_speed_study_2026_10_12_gate_critical_path]].
