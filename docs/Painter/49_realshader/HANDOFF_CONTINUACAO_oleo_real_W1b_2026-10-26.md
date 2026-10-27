# Handoff de continuação — Óleo Real, W1b (line/PainterWatercolor, 26/10)

> Continuação (NÃO é handoff de integração: a linha segue). Anterior: [W0+W1 CPU](HANDOFF_CONTINUACAO_oleo_real_W0_W1cpu_2026-10-26.md) ·
> plano: [`plano.md`](plano.md) · próximo prompt: [`PROMPT_NOVA_JANELA_W1c.md`](PROMPT_NOVA_JANELA_W1c.md).

## 1. Veredito do smoke do dono (26/10, noite)

**Aprovado no rumo:** «bem melhor. sombras interessantes entre a tinta! Bom!» — captura
[`capturas/smoke_W1b_dono_2026-10-26.png`](capturas/smoke_W1b_dono_2026-10-26.png) (vermelho, Dabber + Oil Bristle empilhados).
Pergunta dele, que abre a W1c: **«Porque o render parece mais um Eevee antigo que um Cycles moderno?»**

## 2. Commits

| Commit | O quê |
|---|---|
| `a1eb3643eb` | Passo 1: `Grid::espessura` (factor preguiçoso ≥ 1 sobre `sett+susp`, `grid/espessura.rs`); o depósito fixa-a com o `v` inteiro antes do tecto de 3000; o arrasto leva-a; quem tira massa leva-a na proporção; undo (recorte), instantâneo, limpar. `CampoAltura` lê-a. |
| `a3095f714f` | Passos 2–3: altura da SOMBRA própria (`raio_sombra` 1) com marcha bilinear; cavidade na FORMA; altura × cobertura; fios ∝ espessura; detalhe pré-filtrado (`raio_detalhe` 1); brilho com antisserrilhado Kaplanyan 2016 e × cobertura; composite: sombra no papel também sob a orla fina. Folhas em `out/W1b_*.jpg`. |

**Gravação:** a grade do Wet Paint NÃO é gravada no projecto (`PaintedDocument` guarda `heights/covers/…`, sem grade) ⇒ a espessura não pede degrau de esquema. Se um dia a grade for gravada, a espessura vai junto.

## 3. Números (`medida_oleo_real_acumulo`, faixa central, 1/2/4/8 passagens)

- Massa p50 (a cor) **igual ao byte**: Dabber 3845/6226/6371/6419.
- Altura p50: Dabber 15,5 → 26,5 → 43,3 → 69,1 px · Oil Bristle 12,1 → 24,3 → 50,1 → 110,3 (antes: parava na 2.ª).
- Sombra própria (luz < 0,9): Dabber 14,6 → 17,8 → 20,6 → 27,1 % (antes 6,0 → 16,1).
- Off ao byte: `fingerprint`, `to_the_byte`, `byte_identical`, `seam_wetpaint_oleo_real`, `composite` (103) verdes.

## 4. ⛔ Recusas MEDIDAS (não refazer)

| Tentativa | Porque saiu |
|---|---|
| Cavidade sobre a altura fina (raio 1) | cada poro do Dabber virou fenda saturada: manchas navy, cinza sujo na tinta branca (folha W1b) |
| `raio_sombra` 0 | Oil Bristle h max 875 e horiz p99 274: a sombra vira ruído de célula |
| `raio_sombra` 3 | achata: sombra própria 6 % a 1 passagem (`out/W1b_sombra_raio1_vs_raio3.jpg`) |
| Régua «cristas brancas» (pixel > vizinhos a 2 células + 8) | contava a textura: 6 685 vs 6 659 entre a regra velha e a nova — não vê o defeito |

## 5. Ficou aberto

- **Pontos pretos redondos** em fila nas cristas (captura do dono): por medir — suspeita M2 (`fenda = s·albedoᵏ` leva o vermelho saturado a quase preto) × sombra, sem luz de rebote. Onde aterra ≠ o que provoca: medir antes.
- **Linhas brancas finas** ao longo das estrias do Oil Bristle (captura): brilho GGX de uma só lâmpada; por medir.
- Pontilhado claro/escuro ao longo das estrias **já está na cor do depósito** sem Óleo Real (folha «hoje») — não é desta onda.
- Do plano W1b: (4) GTAO com rebote, (5) pirâmide de máximos + passe de placa; facas/rebordo; IBL. Vermelhos pré-existentes do handoff anterior §8 intocados.
