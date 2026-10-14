# 35 — PROMPT: o Soft Pastel contra o Rebelle «Pastel Dry / Dry Thin», com o MÉTODO certo (14/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current` → `line/PainterWatercolor`; `git status --short` limpo; `bash scripts/btrfs-health.sh`.
Leia SÓ: este prompt · o [handoff W3/W4](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-14_W3_W4.md) ·
[`docs/_ComoInvestigarApps/00_o_metodo.md`](../../_ComoInvestigarApps/00_o_metodo.md) · a linha do Soft Pastel no
[02 §11/10](02_presets_para_wet_painter.md). Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no
fecho, edição provada). ⚠️ «faça como achar melhor e pare de me fazer perguntas» (dono, 11/10). Só o Wet Paint evolui.

## O smoke do dono (14/10), verbatim

Fotos: [Rebelle Dry Thin](35_smoke_dono_rebelle_dry_thin.png) · [PH2D Soft Pastel, None em cima e Noise em
baixo](35_smoke_dono_ph2d_soft_pastel_none_noise.png).

> *«bem diferente do nosso. A primeira coisa que percebo é que não temos essa textura do pincel, algo craqueado. O nosso
> tem um smear que o dele não tem. O nosso cria um ruído no mouse up. o dele não. Ou seja, não há nada a ver um com o de
> referência. […] Lembre-se que a textura noise em Shape não ajustei o size. Outra coisa: Use Texture Colors está apagada
> mas permite que o usuário check ela. Isso não deveria acontecer. Acho que está faltando um método mais correto de
> investigação e comparação da sua parte»*

## O que falhou no método (W3/W4) — não repita

1. Os números da W3 eram razões **entre variantes NOSSAS** (Noise ÷ None). Nenhum foi medido contra o traço do oráculo.
2. A calibração do Soft Pastel (02 §11/10) igualou a **tinta e a largura** a uma foto seca. Não mediu a TEXTURA (o
   craquelado), a BORDA, nem o TEMPO (o que muda depois do pen-up). Uma régua que só soma tinta aprova um traço borrado.
3. Dei ao dono um smoke sem ter visto, eu mesmo, os dois traços lado a lado.

## O método desta janela (CLAUDE.md §0.9, `_ComoInvestigarApps`)

**Fase 0 — a bancada de COMPARAÇÃO, antes de qualquer cura:**
- O MESMO gesto (recto, 220 px, mais um zig-zague fechado como o da foto) corrido por script nos dois:
  - **Rebelle:** `PASTEL`, `Pastel Dry/Dry Thin`, 29 px, a opacidade do próprio preset, na tela VIRTUAL
    (`gotas_inicia.sh` + `arranque.sh`; ⛔ nunca o ecrã do dono).
  - **PH2D:** o preset Soft Pastel pela bancada headless das sondas (`sonda_shape_casa.rs`, `tela()`/`traco()`).
- **Fotos no TEMPO, nos dois:** a meio do gesto, no pen-up, e 0,5, 1, 3 e 6 s depois. O «ruído no mouse up» e o
  «smear» são fenómenos de TEMPO: só os vê quem fotografa depois do pen-up.
- **Réguas por aspecto, cada uma com o seu CONTROLO** (a régua tem de distinguir o Rebelle de um borrão):
  - (a) textura fina: espectro ou contraste local a 2–8 px, e a fração de buracos no corpo;
  - (b) borda: o perfil transversal;
  - (c) cobertura e opacidade do miolo;
  - (d) deriva no tempo: píxeis que mudam depois do pen-up.
- **A prancha lado a lado, na MESMA escala**, olhada POR MIM antes de mandar ao dono. Fixtures com cabeçalho em
  `fixtures/rebelle/pastel_dry_thin_*`; cada corrida vira portão.

**Fase 1 — o mecanismo de cada diferença** (as hipóteses abaixo NÃO estão medidas):
- **O craquelado:** é a textura da PONTA do Dry Thin (os dados dele: `extrai_pinceis.py` → `Parameters`; a imagem
  embutida é asset do alvo, mede-se e não se copia) ou o papel? O nosso só tem o grão Noise 0,4 e o dente do papel.
- **O smear e o ruído do pen-up:** com Water 0 a simulação ainda corre ao soltar? Um pastel é SECO: no Rebelle nada se
  move depois do traço. Confira primeiro onde está a ÁGUA do Soft Pastel no pen-up (o `wetness`, o `Live Water`, o
  assentamento).
- **O Noise no Shape:** com o Size de fábrica o padrão é largo demais para a ponta de 29 px. Meça com o Size ajustado
  antes de concluir que «pinta fraco».

**Fase 2 — curar pela régua**, um bloco só (§0.10), e a prancha antes/depois.

## O item do painel (fazer nesta janela)

**«Use Texture Colors» esmaecido mas CLICÁVEL.** O esmaecido de 04/10 (`esmaecer.rs`, o `active = False` do Blender)
existe para um controlo que vai agir quando o artista mudar OUTRA coisa no mesmo meio: ele prepara o valor antes.
O «Use Texture Colors» no Wet Paint e na Watercolor nunca age, mude-se o que se mudar ⇒ é da família do **Color inativo
do Bug #51** (`paint_brush.rs:458`, `esmaecer::esmaecido`: não age e não aceita clique). A cura:
- a linha não aceita o clique nesses meios;
- o portão conduz o clique pelo ponteiro e confirma que o estado não muda;
- o censo continua verde;
- em `inercia.rs`, distinguir «depende de outro controlo» de «este meio não o lê», para a próxima linha deste tipo
  nascer certa.

## Fecho

O handoff ≤ 8 KB e a lista 26 atualizada. O smoke vai ao dono com a PRANCHA lado a lado e os números de cada régua,
nunca só com passos.

## Resultado (14/10, a janela do prompt)

**Bancada (Fase 0):** `ferramentas/pastel_eventos.py` + `pastel_tela.sh` (Rebelle, tela virtual, zoom 100 %, fotos a
meio · pen-up · 0,5 · 1 · 3 · 6 s) · `sonda_pastel_no_tempo` (os MESMOS gestos, PPM por instante; envs para cada
variante) · `pastel_regua.py` (controlo: o Rebelle borrado a σ 1,5). Fixtures `fixtures/rebelle/pastel_dry_thin_{29,70}`.
⚠️ No Rebelle `size_px` não é a largura vista: o Dry Thin a 29 mostra **13 px** (60 → 25). Compara-se à MESMA largura
vista (Rebelle 70 ≈ 30 px). O Rebelle **não muda nada** de 0,5 a 6 s depois do traço.

**Mecanismos (Fase 1), medidos:**
| o dono viu | mecanismo | prova |
|---|---|---|
| ruído no mouse up | a ponta de Water 0 pousava o pigmento SUSPENSO; o 4.º/5.º passo da água depois do pen-up advectava-o (1 854 px no recto, só no 1.º traço da sessão) | `Σsusp` igual, hash do `susp` muda nos quadros 6–7 |
| smear (riscas) | o arrasto da película (`Drag` 0,16) puxa o pigmento da âncora anterior ao longo do traço | `drag=0` tira as riscas |
| sem craquelado | o grão Noise 0,4 em `Tiled` (base 256 px) é um ladrilho de ~640 px — uma MANCHA, não um grão: o mesmo traço sai com 15 px a y=40 e 34 a y=450 | varredura por altura |
| Noise no Shape pálido | o Size de fábrica (1) põe UM ladrilho de ruído na ponta inteira: 0,43 da tinta; Size ≥ 4 → 0,74–0,78 | por desenho — não se mexeu |

**Cura (Fase 2):** a lei da PONTA SECA no motor (`trail/transfer.rs`: janela sem água sobre célula seca ⇒ pousa em
`sett`, sem arrasto) + o Soft Pastel Noise 100 · papel 0,4 · carga 100 · dureza 0,2, escolhido por nota (soma de
|log(nosso/Rebelle)| em textura, buracos, miolo, ondulação e subida da borda, mancha grossa):

| régua (recto · zigue) | Rebelle | hoje | depois |
|---|---|---|---|
| textura 2–8 px | 0,114 · 0,104 | 0,282 · 0,154 | 0,115 · 0,124 |
| buracos | 0,024 · 0,037 | 0,115 · 0,051 | 0,023 · 0,041 |
| miolo (zigue) | 0,914 | 0,983 | 0,974 |
| ondulação da borda (px) | 0,46 | 1,45 | 0,72 |
| subida 10→90 % (px) | 9,0 | 11,5 | 5,5 |
| deriva depois do pen-up (px) | 0 | 1 854 | **0** |
| **nota** | — | 4,76 | **1,40** |

[Prancha](35_prancha_rebelle_antes_depois.png): Rebelle · hoje · depois, à mesma escala. **Fica diferente:** a poeira
fora do traço (Rebelle 0,011, nós 0), o miolo um pouco mais cheio, a borda mais firme (5,5 contra 9) e o cruzamento de
dois traços (no Rebelle o segundo passa mais transparente). O painel: «Use Texture Colors» e as linhas «Layer N Color»
recusam o clique no Wet Paint (`inercia::Natureza::MeioNaoLe`).
