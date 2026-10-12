# 09 — O texto dos SVG (W18, A3)

> Um SVG largado no quadro (e o PDF, que o rasteriza) desenhava tudo menos os `<text>`: o `usvg` corre
> sem a feature `text` (recusa medida — seria uma 2.ª stack de fontes ao lado do `parley`,
> `crates/ph2d-vec-svg/Cargo.toml`) e apaga-os. Agora eles DESENHAM-SE, pela stack da casa.
> Código: `crates/ph2d-imageio-svg/src/text/`. Gate: `text::oracle_tests::the_texts_match_chromium`.

## 1. O desenho: uma passagem ANTES do usvg

`raster::parse` chama `text::outline_texts`: cada `<text>` do XML é trocado por um `<g>` com um
`<path d>` por elemento (`<text>`, `<tspan>`, `<a>`), com os contornos dos glifos no MESMO espaço do
utilizador. Moldado pelo `parley` (fontes do sistema pelo `fontique`/fontconfig), contorno pelo `skrifa`
sem hinting. Porquê aqui e não depois:

- a árvore do usvg não aceita nós de fora; reescrever o XML deixa ao usvg tudo o resto — `transform`,
  pintura (`fill`, `stroke`, `opacity`, gradientes), herança, `clip-path`, `mask`, `filter`, `<use>`;
- o `<g>` leva os atributos do elemento (menos `x`/`y`/`dx`/`dy`/`rotate`/`textLength`/`lengthAdjust`;
  o `transform` só no `<text>` — no `<tspan>` o Chromium ignora-o) e, no `style`, as declarações das
  regras do `<style>` que casavam com ELE — uma regra por tipo (`text { fill: … }`) continua a valer;
- o `<path>` leva `fill-rule:nonzero` (o texto pinta-se sempre não-zero);
- dentro de um `<clipPath>` (que não aceita `<g>`) sai um só `<path>`;
- o prefixo do ficheiro mantém-se (`<svg:text>` → `<svg:g>`); um `<text>` vindo de uma entidade não tem
  lugar nos bytes e não se troca (é reportado);
- um SVG aninhado num `data:` passa pela mesma passagem (`raster::capped_data_resolver`);
- o exportador de SVG grava os bytes ORIGINAIS: não passa por aqui.

Sem `<text>` nos bytes (`has_text`, uma varredura), não há passagem nem cópia.

## 2. A disposição (a do Chromium)

- Espaço em branco: quebra e tab viram espaço; sem `xml:space="preserve"` (ou `white-space: pre*`),
  o espaço do início, o repetido (através dos `<tspan>`) e o do fim saem. O Chromium troca a quebra por
  espaço (não a apaga como o SVG 1.1) — t11.
- `x`/`y`/`dx`/`dy` por caracter: o i-ésimo valor de um elemento vai para o i-ésimo caracter da sua
  sub-árvore; o descendente manda (pré-ordem). `dx`/`dy` acumulam até ao fim do bloco (t07).
- Um BLOCO novo em cada `x` ou `y` absoluto; cada bloco moldado à parte e ancorado pelo `text-anchor`
  do seu 1.º caracter (largura = avanço + `dx` acumulado) — t06, t07, t16.
- `font-size` em px/pt/pc/in/cm/mm, `em`/`%` (do pai), `ex` = ½ em (a regra do CSS sem altura-x), as
  palavras-chave do CSS Fonts 4 §2.5; `font-weight` com `bolder`/`lighter` (§2.2); `font-style`;
  `font-stretch`; `letter-spacing`/`word-spacing`; o `font` abreviado.

## 3. As famílias (a do Chromium no Linux — MEDIDO)

| Pedido | O Chromium desenhou (oráculo) | Prova na fixture |
|---|---|---|
| ausente (`Nao Existe Font`) e nenhum | Liberation Serif (= «Times New Roman») | t02/t03: «H» avança 28,875 a 40 px = 0,722 em (Times) |
| `serif` | Liberation Serif | t04 «e» = 15,98 a 36 px = 0,444 em |
| `sans-serif` | **Liberation Sans** (= «Arial»), NÃO o `fc-match sans-serif` (Noto Sans) | t04 «S» = 24,0 a 36 px = 0,667 em |
| `monospace` | Noto Sans Mono (`fc-match monospace`) | t04 avanço 21,594 = 0,6 em em 1/64 |
| lista `'Nao Existe', 'DejaVu Serif', serif` | DejaVu Serif | t05 |

Daí a lei de `text/fonts.rs`: as genéricas são os nomes das preferências do Chromium (serif = Times New
Roman, sans-serif = Arial; `monospace` aceita o que o fontconfig dá); um nome ausente vale pelo gémeo de
MÉTRICAS instalado (Arial ↔ Arimo ↔ Liberation Sans, Times New Roman ↔ Tinos ↔ Liberation Serif,
Courier New ↔ Cousine ↔ Liberation Mono, Cambria ↔ Caladea, Calibri ↔ Carlito, Symbol ↔ Symbol Neu) ou
pelo nome alternativo do Blink (Arial ↔ Helvetica, Times ↔ Times New Roman, Courier ↔ Courier New);
sem nenhuma, a de `serif`. ⚠️ Só a linha Times e a Arial estão provadas pelo oráculo (t02–t04); as
outras são a mesma regra, sem caso.

O caracter que a família não tem cai na reserva do `fontique` (por escrita), não na do fontconfig por
caracter do Blink — os acentos (t14) a DejaVu tem; uma reserva real não está no oráculo.

## 4. O gate e as tolerâncias

Fixture: `ferramentas/excalidraw_oracle/saidas/w18/w18_sondas_offline.json` (Chromium, `sonda_w18.mjs`,
entradas `entradas/w18_texto/t01..t16`). Por caso: a origem de cada letra contra
`getStartPositionOfChar`, e a caixa da tinta de cada cor-chave do nosso desenho a 1× e 2× (o mesmo
classificador da sonda, sobre branco).

- **Letra ≤ 0,05 unidade.** Medido: pior caso 0,020. O próprio Chromium discorda de si em 1/64 (o fim
  de uma letra contra o início da seguinte). Sem o kerning, o «rg» de «Hamburg» desvia 0,71 — uma
  tolerância de 1 unidade deixá-lo-ia passar.
- **Caixa ≤ 2 px.** Medido: pior caso 1 px (dois rasterizadores, anti-aliasing sob o limiar 0,35).
- Família ausente na máquina: o caso SALTA com aviso (`eprintln!`); nenhum caso a correr reprova.

Vermelho antes da cura (a passagem desligada): 56 desvios, todos «nosso None, Chromium Some(caixa)».
Verde depois:

| caso | letra (máx) | caixa 1× | caixa 2× |
|---|---|---|---|
| t01 família presente | 0,012 | 1 | 0 |
| t02 família ausente | 0,012 | 0 | 0 |
| t03 sem família | 0,012 | 0 | 0 |
| t04 genéricas | 0,012 | 1 | 1 |
| t05 lista de famílias | 0,011 | 1 | 0 |
| t06 anchor | 0,001 | 1 | 1 |
| t07 tspan | 0,013 | 1 | 1 |
| t08 transform | 0,020 | 0 | 1 |
| t09 peso/itálico | 0,014 | 0 | 1 |
| t10 unidades/estilo | 0,014 | 1 | 1 |
| t11 espaços | 0,013 | 1 | 0 |
| t12 contorno | 0,012 | 0 | 1 |
| t13 viewBox | 0,002 | 0 | 1 |
| t14 acentos | 0,014 | 1 | 1 |
| t15 letter-spacing | 0,001 | 0 | 1 |
| t16 lista x | 0,008 | 0 | 1 |

## 5. O custo (medido, release, o MÍNIMO de 20 corridas, `load` 10–18)

| | passagem | parse com texto | parse sem texto | desenho ao tamanho natural |
|---|---|---|---|---|
| t04 (3 textos, 25 glifos) | 117 µs | 43–53 µs (guardado) | 2,4 µs | 140–150 µs |
| 200 textos, ~5 800 glifos | 19,4 ms | 6,2–6,5 ms (guardado) | 56 µs | 11 ms |

A 1.ª passagem do processo abre a colecção de fontes: 15–27 ms. O XML cresce ~130× (19 KB → 2,4 MB
no pesado). Como o quadro desenha o mesmo SVG a cada nível de zoom, a passagem GUARDA-SE por ficheiro
(`text/cache.rs`): a chave são os próprios bytes (comparados inteiros — sem hash, HR-5), o tecto é de
memória (`MAX_ARCHIVE_TEXT_BYTES`, original + reescrito, sai o usado há mais tempo). O que fica por
nível é o parse do usvg dos contornos (6 ms no pesado).

### ⛔ Recusas MEDIDAS

| Ideia | Medido | Porquê não |
|---|---|---|
| Casas decimais até 1/10 de unidade da fonte nos `d` | −4 % de bytes (2,49 → 2,39 MB) | ganho desprezável; o código fica |
| Feature `text` do usvg | 2.ª stack (`fontdb` + tabelas `unicode-*`) | a recusa de 2026-09-05 continua |

## 6. O que NÃO se desenha como escrito — `raster::text_gaps`

`pub fn text_gaps(src: &[u8]) -> TextGaps` (`missing_families: Vec<String>`, `unsupported: Vec<String>`,
`undrawn: usize`); o app (`ph2d-app-board::images::Placed`) nomeia `unsupported` e `missing_families` no aviso; a ingestão lê o tamanho por `raster::parse_size` (sem a passagem, fora das fontes).

- `missing_families`: as famílias pedidas que o sistema não tem e que vinham ANTES da usada (a 1.ª
  escolha do autor), sem repetição. Genéricas nunca faltam.
- `unsupported` (desenha-se sem a construção, ou não se desenha): `textPath`, `tref`, `rotate`,
  `textLength`, `writing-mode` vertical, `direction` rtl, `font-variant`, `dominant-baseline`,
  `alignment-baseline`, `baseline-shift`, `text-decoration`, `synthetic bold` (a família não tem
  negrito: sai regular), `text` (um `<text>` vindo de uma entidade), `MAX_ARCHIVE_TEXT_BYTES` (o SVG
  reescrito passaria do tecto: esse `<text>` fica de fora).

Limites conhecidos, sem aviso: o gradiente `objectBoundingBox` de um `<tspan>` usa a caixa dos glifos
dele; uma regra CSS por tipo `g`/`path` passa a apanhar o substituto; o `text-decoration` herdado de um
`<g>` acima não se detecta; as lacunas de um SVG aninhado num `data:` não se contam.
