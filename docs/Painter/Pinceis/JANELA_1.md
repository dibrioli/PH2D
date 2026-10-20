# JANELA 1 de 5 — Fundação: oráculos, catálogo, a bancada do EXAME e a limpeza do P53

> Leia ANTES: `00_MISSAO.md` (o contrato), CLAUDE.md (auto), `project-memory/MEMORY.md`. Você é a 1.ª janela: começa já.

## Entregas (na ordem; paralelize o que é independente)

**A. O que o tempo mata primeiro — as fotos do Rebelle (expira ~06/11).** Um agente de fotos (UM, o Rebelle é uma
instância): defina a BATERIA de gestos do EXAME (§3.1 do contrato) como eventos (`gera_eventos.py`/`fotos_lote.sh`
generalizados) e fotografe, num lote, TODOS os presets do Rebelle que o catálogo (B) usar — comece pelas famílias que
o Rebelle tem (lápis, carvão, pastéis, óleo, acrílica, aguarela, guache, tinta, marcadores, aerógrafo, borradores).
Cabeçalho + README em `fixtures/` como os de hoje. Se o catálogo ainda não existir, fotografe as categorias inteiras
dos 281 (`rebelle_presets_281.json`) — sobra é barata, falta é irrecuperável.

**B. O CATÁLOGO** (`docs/Painter/Pinceis/01_CATALOGO.md`): pesquise (WebSearch/WebFetch, os manuais oficiais) os
pincéis MAIS usados e amados — Procreate (biblioteca padrão: Sketching, Inking, Drawing, Painting, Artistic,
Airbrushing, Calligraphy, Charcoals, Textures, Spraypaints, Elements…), Photoshop (Hard/Soft Round, Kyle T. Webster),
Krita (o bundle padrão), Clip Studio (G-pen, Mapping pen, Turnip pen, Real pencil, aguarelas, óleo), MyPaint
(deevad, ramon, tanda, classic), Rebelle (os 281). Junte por IDENTIDADE (um «6B pencil» é o mesmo pincel em 4 apps),
corte duplicados e escolha **80–120**. Por pincel: nome nosso · categoria · apps de origem · o oráculo que CORRE (qual
app/preset) ou «só documentado» · a identidade em uma frase · o motor provável · a razão de uso · prioridade.
Licença por artefacto (`pacman -Qi`, o `LICENSE` de cada ficheiro dos `mypaint-brushes`): marque o que pode ser
PORTADO (CC0/ISC/MIT) e o que só se corre.

**C. Os oráculos FORTES que correm por script** (`docs/Painter/Pinceis/02_ORACULOS.md`; ⛔ GIMP fora — dono:
«fraco»): Krita 6 (pintar a bateria por Python/linha de comando, headless ou `kwin --virtual`), MyPaint/libmypaint (um
harness ctypes/C sobre a lib ISC: o traço EXACTO de cada `.myb`), e **instale e ponha a correr sob Wine** as
avaliações de **Clip Studio Paint, ArtRage, Corel Painter** e o que mais der (Photoshop/Fresco: tente; registe se o Wine
não o leva) — cada um num prefixo próprio em `~/Apps/`, na tela virtual, com a sua porta de automação (script, plugin
ou eventos injectados, o caminho do Rebelle em `~/Apps/rebelle/oraculo/`), EULA da avaliação lida e registada. Para
cada um: o comando, a prova (uma prancha da bateria), a licença, quando expira. O que não correr (Procreate): a fonte
documental e as imagens de referência (só como alvo visual). Avaliações expiram: as fotos de cada app novo são tiradas
logo que ele corre, num lote, como as do Rebelle (A).

**D. A bancada do EXAME** (código de teste/ferramentas, não produto): (1) a bateria de gestos pintada pelos NOSSOS 4
motores para qualquer preset (generalize a `sonda_midias`/`pinta_cena`), (2) a régua generalizada aos gestos, (3) a
MATRIZ de distinção contra a biblioteca INTEIRA, (4) a prancha (oráculo · nosso · vizinho mais perto), (5) o protocolo
da REVISÃO CEGA por agente (o briefing pronto para colar), (6) a medição de ms/quadro e memória. Prove-a de ponta a
ponta em 2 presets existentes (Graphite 8B, Oil Bristle).

**E. Os 4 motores** (`03_MOTORES.md`): o que cada um exprime hoje (pelo CÓDIGO e por sondas), e a lista de LACUNAS
que o catálogo exige (ex.: a ponta que gira com o arrasto — C38; modos de mistura do pincel — Marker/Effect; dual
brush; dinâmica de cor; dispersão/contagem; borrador/smudge; presets dos meios Digital/Watercolor/Impasto no mesmo
navegador — confira se o formato `.ph2d-brush.png` e o navegador já os aceitam). Cada lacuna: o pincel do catálogo que
a pede, o custo, a ordem. Esta lista é a fila da JANELA 2.

**F. A limpeza do P53 pelo veredito do dono**: Filbert, Flat Oily e Knife Smooth saem da fábrica (ou ficam marcados
para refazer na janela 4 com a lâmina que gira) — contagem, portões (`MIDIAS`, `IRMAOS`, `NASCE_DE`), rótulos, o
teste da cor (31 fixo) e o doc 53 actualizados; os portões verdes por nome.

## Saída
`ESTADO.md` criado com a FILA da missão inteira (o que cada janela seguinte faz, por prioridade do catálogo), commits,
`HANDOFF_JANELA_1.md`, e o SINAL à JANELA 2 (§5 do contrato). Contexto a 300 mil ⇒ passe a vez antes.
