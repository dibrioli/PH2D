# Prompt da janela nova — line/PainterWatercolor, 26/10: o «Óleo Real» no Wet Paint (W0)

```
Linha line/PainterWatercolor (Modo L) — onda nova: o «Óleo Real» no Wet Paint (RealShader + SoftShadows do Rebelle 8).
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md e docs/Painter/49_realshader/PROMPT_NOVA_JANELA_W0.md inteiro.
Delegue leitura e verificação.
```

## Estado ao abrir

- Árvore limpa em `e39834b84a`, com o backup `backup/line-PainterWatercolor-janela15-2026-10-26` e a etiqueta
  `backup-PainterWatercolor-janela15-2026-10-26`.
- A onda anterior (janela 15, as ferramentas da água N1–N4) está fechada e comitada. **O smoke dela ficou adiado
  pelo dono** («por enquanto deixe como está»): handoff por escrever. Os passos do smoke e os abertos estão nos
  commits `35ba550318`..`e39834b84a`. Abertos: `a_largura_e_a_prometida` vermelho no Smear Shattered (jitter), no
  Smear Wet (a água espalha) e nos Blender a raio 120 / Textured a 300 (a régua do misturador salta). Não mexa
  nisto nesta onda.

## A onda: o Óleo Real

1. Leia, nesta ordem:
   - `docs/Painter/49_realshader/README.md`: o que o Rebelle 8 faz, medido no ecrã real em 26/10 (M1–M11), o que
     temos hoje, a especificação (§4) e as recusas (§6).
   - `docs/Painter/49_realshader/plano.md`: as ondas W0–W6, as condições para abrir e cancelar cada uma e os
     portões de cada uma.
   - `docs/Painter/27_plano_luz_sombra_propria.md` §3: o desenho antigo da sombra própria e da AO, nunca
     construído. Reaproveite a forma da função «por direcção».
   - `crates/ph2d-wet-paint/src/render.rs:385-450`: o relevo de hoje é um emboss cinza somado (±40), com a luz fixa
     topo-esquerda.
   - `crates/ph2d-render/src/shaders/impasto_light.wgsl`: o passe de luz do Impasto, modelo do irmão novo.

2. Factos que mandam e já não se discutem:
   - Só o Wet Paint evolui (dono 07/10). O Impasto não é objectivo.
   - A fenda do Rebelle não é cinza: em linear, fenda ≈ mediana · 0,25 · albedo^2,1 (M2). É o termo que mais pesa
     no «parece real».
   - O escuro das fendas vem do sombreador e não da sombra (M7). A sombra é macia, cai também no papel, é linear
     no slider (M6) e fica mais longa com a luz mais baixa (M10).
   - Nada do Rebelle se lê nem se copia: nem binário, nem shaders, nem «Reflection Maps». Ambientes só CC0 (Poly
     Haven) ou fotos do dono, com triagem de licença por ficheiro.
   - Neutro byte-idêntico: com o Óleo Real desligado, o render de hoje sai ao byte (gate).

3. Comece por W0 (medir antes de limitar, §0.0), num lote só (§0.10):
   - (a) o custo do passe a 1080p e a 4K, com k = 8/16/32 passos × N = 4/8 direcções, no NVIDIA e na Radeon
     onboard;
   - (b) a sombra a 15° no Rebelle, que falta (M10);
   - (c) a escala de altura do Wet Paint (sett+susp e a altura dos fios) contra a espessura do Rebelle na mesma
     pincelada.

   A tabela vai para o README. Nenhuma constante entra sem ela.

4. O oráculo é o Rebelle 8.3.4 Pro trial. O trial acaba por volta de 06/11/2026: capture o que faltar antes disso.
   - Precisa do ecrã REAL com a placa. Na tela virtual o render é por software e o RealShader não aparece; o dono
     já rejeitou essas fotos.
   - As ferramentas estão em `docs/Painter/49_realshader/ferramentas/`:
     - `sessao.sh <tag> [chave=valor do settings.ini]`
     - `tracos.py` (pinceladas-sonda por WebSocket)
     - `foto0.sh` (fotografa só a janela do Rebelle)
   - Antes de cada clique, chame `front <título>`.
   - Antes de cada foto, rode `kscreen-doctor --dpms on`: com os monitores apagados a janela fica HIDDEN e o
     import falha com «missing an image filename».
   - Mexer no ecrã real exige o dono ausente ou a autorização dele.
   - O modo lote (`-batch-json`) e o SAVE/LOAD pelo WebSocket não existem no trial.
   - As réguas usam o crop da Knife Textured (sai igual entre aberturas) e o perfil à direita da espátula vertical.

5. Regras da linha:
   - Inner loop: `bash scripts/cargo-check-narrow.sh`.
   - Testes só por nome; a suíte completa fica para o integrador.
   - Clippy só no fecho (`PH2D_SUITE=fecho`).
   - Comandos pesados vão por `scripts/ph2d-run.sh`, em `--profile smoke`.
   - Código novo da óptica em `ph2d-render` (o passe) e `ph2d-wet-paint` (a referência em CPU). A shell só compõe.
   - UI só com tokens e i18n.
   - Sem push.

6. Fim de cada onda: smoke ao dono em passos numerados (o comando com o `cd`, onde clicar, o que tem de
   acontecer, como saber que deu errado).
   - Use espátula e pincel grosso em tinta clara, média e escura, lado a lado com
     `docs/Painter/49_realshader/capturas/A_base.jpg`.
   - O critério é o olho do dono contra o Rebelle.
   - Handoff e prompt seguinte só DEPOIS do veredito do smoke.
