# Prompt da janela nova — line/PainterWatercolor, 26/10: Óleo Real W1b (espessura, relevo e sombra de verdade)

```
Linha line/PainterWatercolor (Modo L) — onda W1b do Óleo Real no Wet Paint: relevo que SOBE com o acúmulo, sombra DENTRO da tinta, sem pixels brancos na borda.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, docs/Painter/49_realshader/PROMPT_NOVA_JANELA_W1b.md e
docs/Painter/49_realshader/HANDOFF_CONTINUACAO_oleo_real_W0_W1cpu_2026-10-26.md inteiros.
Delegue leitura e verificação.
```

## Estado ao abrir

- Árvore limpa em `HEAD_A_PREENCHER` (o commit destes docs; confirme com `git log -1`).
- W0 feita; W1 em CPU feita e ligada ao Wet Paint (cartão «Real Oil»), mas **rejeitada no olho** pelo dono.

## Factos que mandam

- **Veredito do dono (smoke 26/10):** «Resultado interessante»; problemas: (a) pixels brancos nas bordas; (b) sombra de «drop shadow», nenhuma DENTRO da tinta; (c) o relevo não sobe com o acúmulo. Prioridade: «relevo realista, shader realista, luz e sombras realistas».
- **Causa de (c), medida:** tecto duro de 3000 por célula em `crates/ph2d-wet-paint/src/trail/transfer.rs` (`:268–275` molhado, `:285–288` assentado, `:178–181` seco). A massa pára na 2.ª passagem (Dabber p50 3845→6226→6371→6419 a 1/2/4/8). Auto-sombra interior 0 % / ~2 % a 42°.
- **Pesquisa** (`pesquisa_tecnologia.md`): o Rebelle acumula altura sem tecto, marcha de raio sobre o campo, oclusão de fenda própria, IBL de esfera cinza.
- Lei actual em `crates/ph2d-wet-paint/src/oleo_real.rs` (forma suavizada raio 3, M2 `s=0,25 k=2,1`, penumbra angular 0,6, `px_por_massa` 0,0015). Desligado = ao byte.
- Vermelhos pré-existentes (handoff §8) NÃO são seus; não mexa neles.

## O trabalho (ordem; série justificada: cada passo muda a medida do seguinte)

1. `Grid::espessura` (escrita antes do tecto, nunca aparada; arrasto, empurrão do molhado e ferramentas de levantar a levam; secagem deixa; modelo `grid/altura.rs` com undo e gravação; `CampoAltura` lê-a). Recalibre `px_por_massa` com `ferramentas/regua_sombra.py`. Meça de novo `medida_oleo_real_acumulo`: tem de CRESCER com as passagens.
2. Altura de sombra/oclusão sem suavizar; textura ∝ espessura; rebordo.
3. Altura e normais pesadas pela cobertura na borda; especular × cobertura + AA ⇒ sem pixels brancos.
4. Cavidade tipo GTAO com rebote pelo albedo.
5. Pirâmide de máximos (Snyder 2008 / Timonen 2010); depois o passe de placa em `ph2d-render` (paridade com a CPU; K/N por placa, W0(a)).
6. Facas H3 → presets Knife Smooth/Textured; depois a lei do rebordo.
7. IBL com foto nossa / CC0 (triagem de licença por ficheiro).

## Regras

- Inner loop `bash scripts/cargo-check-narrow.sh <crate>`; testes só POR NOME (`cargo-test-narrow.sh`); clippy só no fecho (`PH2D_SUITE=fecho`); suíte completa é do integrador.
- Comando pesado por `bash scripts/ph2d-run.sh …`; sempre `--profile smoke`.
- UI só com tokens e i18n (zero hex, zero string solta). Formato gravado novo ⇒ versão/degrau do esquema contado, não escolhido.
- Nada do Rebelle se lê nem se copia. Sem push. Delegue leitura e verificação.
- Antes de limitar um número, MEÇA (§0.0); agrupe medições num lote (§0.10).

## Smoke ao fim (para o dono; passos numerados)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`
2. Meio Wet Paint; Oil → Dabber e Oil Bristle; três pinceladas grossas (clara, média, escura).
3. Cartão do Wet Paint: marque «Real Oil»; mexa Light angle / Light height / Shadow / Gloss / Depth.
4. Passe 4 vezes sobre a mesma pincelada: tem de subir à vista e projectar sombra dentro da tinta.
5. Compare com `capturas/A_base.jpg`. Errado se: pixels brancos na borda, sombra só fora da tinta, 4 passagens iguais a 1. Fotografe a cena antes de mandar.

**Handoff e prompt seguinte só depois do veredito do smoke.**
