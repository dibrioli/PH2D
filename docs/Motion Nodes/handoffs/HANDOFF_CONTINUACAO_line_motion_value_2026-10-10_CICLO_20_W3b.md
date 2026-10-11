═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–20 em `line/motion-value`, REBASEADA no `main` `782f90b69` (10/10), POR INTEGRAR.
O ciclo 20 (O ACABAMENTO, [doc 129](../129_ciclo_20_o_acabamento.md)) está ABERTO: W0–W3 fecharam;
o smoke do dono do `fx.filter` REPROVOU (a sombra interna não age em cada forma; o bevel treme) e a
auditoria está no doc 129 §5.5. O dono decidiu: **as duas opções no nó — cada forma e a camada
inteira**. Você abre a **W3b** (doc 129 §3.4). Depois dela vem a W4
([prompt](HANDOFF_CONTINUACAO_line_motion_value_2026-10-10_CICLO_20_W4.md)).

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 &&
        git status --short — ANTES de abrir qualquer ficheiro.

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8). Integrar/pushar só por ordem dele.
⭐ LEIA o doc 129 §3.3, §3.4, §5.4 e §5.5 (é o caro: medições e mecanismos — não refaça).
⭐ CUSTO: leitura larga a `explorador`; gate batched a um `verificador`. `/pd-feature`: plano primeiro.

O QUE JÁ EXISTE (não reconstrua): o nó `crates/ph2d-node-fx-filter` (a pilha = os `fx.filter` do fio;
ficha derivada da `SPECS` com gates compactos por genéricos `const`); `FxOpGpu::resolve`
(porta única mundo → pixel); `FxStackPass::run_premultiplied`; `ph2d-app-motion/src/motion_filter.rs`
(a camada inteira: sprites num alvo linear + a placa das formas → camada no espaço do desenhista →
pilha → `WorldRt`, antes da lente); a SONDA `ph2d-render/tests/it/fx_stack_camada_sonda.rs` (a sombra
interna cercada × sozinha; o bevel raster × exacto sob ¼ px) — vire-a em GATES com o alvo da §3.4.

FACTOS MEDIDOS QUE DECIDEM O DESENHO (explorador, 10/10):
- `RenderInstance.world_pos` é translação pura no shader (`sprite.wgsl:244`, sem matriz de modelo);
  o buffer do dispositivo (`ph2d-gpu-cook/src/instances.rs`, `INSTANCE_WORDS = 47`, `lower.rs:29`)
  tem o MESMO layout e uso `STORAGE | COPY_SRC` (⚠️ o comentário de `instance.rs:73` diz stride
  144 B e o `lower.rs:27` 188 B — confira); `ensure_instance_capacity` recria o buffer ao crescer.
- As formas: `ShapeInstance { pos, size, basis, anchor, geometry, tint }` (`ph2d-shape-gpu/src/pass.rs:25`),
  cópias da CPU ou do dispositivo (`motion_shape_placa.rs:241`); `redesenha_em` redesenha num alvo do
  MESMO tamanho, sem override da câmera.
- `sort_render_order` ordena por (clip, `z_order`, `sub_order`, texture, sampling); a ordem dentro de
  um stream é determinística. Não há id buffer nem passe de picking no `ph2d-render`.
- A `=135` são 16×16 = 256 discos (`ESTRELA = 0,15`, confira se é diâmetro).

A W3b (doc 129 §3.4), num bloco só, com um gate por achado da §5.5 e o controlo de cada um:
`scope` + a partição da pilha · o átlas por forma (rota da CPU e do dispositivo) · a semente
sub-pixel do campo (Gustavson & Strand 2011) · A4 cor linear→sRGB · A2/A5 defaults do tipo ·
A6 origem do ruído · A7 luz sub-pixel · A8 régua na janela do produto · U1 nomes únicos. A cena `=135`
mostra as duas opções (o roteiro diz o que cada uma faz) e as FOTOS antes do dono
(`fotografa_cena.sh`). Feche com o doc 129 §5.6 e o prompt da W4 atualizado.
═══════════════════════════════════════════════════════════════════
