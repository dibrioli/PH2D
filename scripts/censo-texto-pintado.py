"""CENSO DO TEXTO PINTADO — quantos rótulos chegam ao ecrã como LITERAL (HR-15).

    python3 scripts/censo-texto-pintado.py [-v]

⚠️⚠️ **Ele existe porque a primeira medição desta grandeza estava 4× errada.** Em 2026-09-09 o
`docs/UI_New_and_Simple/medicoes/07` publicou **108** literais pintados em 13 crates, contando as
chamadas a `paint_text*`. Em 2026-09-10, a tentar curá-los, apareceu `paint_panel_title(rect,
"Widget Gallery", …)` — uma porta que aquele censo não conhecia. Derivadas do fonte, as portas são
**127**, e os literais **418**.

⛔⛔ **E este ficheiro ERROU a mesma classe de erro na primeira hora de vida:** ele não conhecia o
literal de CARÁCTER (`find('"')` é Rust legítimo), lia aquela aspa como uma string a abrir e passava
a ler o resto do ficheiro ao contrário — comentário como texto, texto como código. Leu **438** onde
o número é **418**. *Um censo textual tem de saber TODAS as formas do que lê, e «todas» inclui as do
próprio leitor.*

⭐⭐ **A régua certa é um PONTO FIXO, não uma lista de nomes.** Um literal chega ao ecrã se for
passado no argumento de texto de um pintor — **ou** a uma função que repassa esse parâmetro a um
pintor, e isso é recursivo:

    paint_text(ts, scene, texto, …)                      ← a semente
    fn label(ctx, texto: &str, …) { paint_text(.., texto, ..) }   ← porta de 1.º grau
    fn paint_slider_chip_row(.., rotulo: &str, ..) { label(.., rotulo, ..) }  ← 2.º grau

⚠️ **É por isso que duas medições da mesma grandeza deram `108` e `79`:** nenhuma das duas
declarava quantos SALTOS seguia. *Um censo que não declara o seu alcance não é comparável consigo
mesmo* — este segue até ao ponto fixo e imprime quantas portas achou.

⛔ **Ele IMPRIME e não escreve nada.** Não há `--write`, e não há catraca global: um censo com a
dívida de 13 crates dentro faria a próxima linha que pinte um rótulo ficar vermelha por causa de um
gate desta (`CLAUDE.md` §0.2 — foundational novo projecta-se para ISOLAMENTO). A catraca, quando
existir, é **por crate, na crate dela**.

⚠️ **O que ele NÃO vê**, declarado: literais que chegam por variável local ou por `format!` — os
dois continuam a ser texto pintado. ⇒ *o número que ele imprime é um PISO.*

⭐⭐⭐ **A TABELA de `&str` SAIU dessa lista em 2026-09-19, e a dívida real estava lá.** Este
parágrafo dizia *«por `const`, por tabela de `&str`»* havia semanas, e as cenas da conferência do
Motion pintam `for (k, word) in ROW_LABELS.iter() { label(g, word, …) }` — **treze** palavras que
nunca entraram em contagem nenhuma, e mais **dezoito** num painel de produto. *Uma cegueira escrita
num doc-comment não é uma medição: é uma nota que envelhece.* Hoje ele segue a tabela (alcance
declarado: `const`/`static` de literais, lida por um `for` no MESMO ficheiro).

⛔⛔ **E ele MENTIA para o outro lado também, em duas formas — as duas curadas no mesmo dia:**

- um `#[cfg(test)] mod … { … }` **em linha** era contado como produto (`5` achados, entre eles
  `"Um nome absurdamente comprido para uma caixa"`, que é uma FIXTURA). O `is_test` só conhecia o
  ficheiro-irmão `*_tests.rs`; a régua da crate (`ph2d_label_census::cfg_test_modules`) já sabia.
- *«tem uma letra»* não é *«é uma palavra»*: ele contava os códigos de canal de cor
  (`R Y G C B M W N K`) e as SETAS (`u{25c0}`/`u{25b6}`) — estas porque lia o **escape do fonte**
  (que tem um `u` e um `b`) em vez do carácter pintado. A régua registada pede **duas letras ASCII
  seguidas**, e agora as duas concordam sobre o que é uma palavra.

⭐ **`--autoteste`** corre o controlo positivo das três curas, sem a árvore. *Uma cura de instrumento
sem controlo positivo é uma afirmação sobre o instrumento antigo.*

⛔⛔ **E O PISO ERA MUITO MAIS BAIXO DO QUE O TECTO (medido 2026-09-13).** Os construtores de widget
(`Button::new(id, "Apply")`), as tabelas (`["Paint", "Erase"]`) e o `format!` são a MAIORIA do
texto de um painel: sobre a mesma árvore este script conta `166` na `ph2d-panel-painter-layers` e a
régua lexical conta `376`; `32` contra `593` na `ph2d-editor-core`; `445` contra `5 533` nas crates
de UI inteiras. ⇒ **a régua de registo é a da crate `ph2d-label-census`**, a mesma que os gates por
crate correm:

    cargo run -q -p ph2d-label-census --example censo -- --resumo

Este script fica como o IRMÃO que mede a outra grandeza (o que chega a um pintor pelo nome dele).

⚠️ **E ele varria por PREFIXO** (`ph2d-panel-*` + `ph2d-editor-core`) até 2026-09-13: o código de
família que a W2 levou da shell para `ph2d-app-*` ficou fora dele em silêncio (`10` literais na
`ph2d-app-motion`, `1` na `ph2d-param-editors`). A raiz passou a ser a árvore inteira.
"""
import os,re,sys,collections,json

ROOTS=[f"crates/{d}/src" for d in sorted(os.listdir("crates")) if os.path.isdir(f"crates/{d}/src")] \
     +[f"shells/{d}/src" for d in sorted(os.listdir("shells")) if os.path.isdir(f"shells/{d}/src")]

def strip_comments(src):
    out=[];i=0;n=len(src);instr=False;esc=False
    while i<n:
        c=src[i]
        if instr:
            out.append(c)
            if esc: esc=False
            elif c=='\\': esc=True
            elif c=='"': instr=False
            i+=1; continue
        if c=='"': instr=True; out.append(c); i+=1; continue
        # ⚠️⚠️ UM LITERAL DE CARÁCTER TEM ASPAS DENTRO -- `find('"')` é Rust legítimo, e um leitor
        #    que não o conheça vê ali uma aspa a ABRIR uma string e lê o resto do ficheiro ao
        #    contrário. ⚠️ O `'` também abre um TEMPO DE VIDA (`&'a str`), que não fecha: o
        #    discriminador é haver um `'` a fechar dentro do alcance de um escape.
        if c=="'":
            close=None
            if i+1<n and src[i+1]=="\\":
                for j in range(i+2, min(i+8,n)):
                    if src[j]=="'": close=j; break
            elif i+2<n and src[i+2]=="'":
                close=i+2
            if close is not None:
                out.append(" "*(close-i+1)); i=close+1; continue
        if src.startswith("//",i):
            j=src.find("\n",i); j=n if j<0 else j; out.append(" "*(j-i)); i=j; continue
        if src.startswith("/*",i):
            j=src.find("*/",i); j=n if j<0 else j+2; out.append(" "*(j-i)); i=j; continue
        out.append(c); i+=1
    return "".join(out)

def is_test(f): return f=="tests.rs" or f.endswith("_tests.rs")

# ⛔⛔ **CURA (2026-09-19): um `#[cfg(test)] mod … { … }` EM LINHA não é produto.**
#
# O `is_test` acima só conhece o ficheiro-irmão (`*_tests.rs`), e o módulo em linha escapava-lhe:
# medido, este censo acusava **4** literais do `ph2d-panel-motion-params` — entre eles
# `"Um nome absurdamente comprido para uma caixa"` — que são FIXTURAS de um `#[cfg(test)] mod
# tests`. ⚠️ *Um censo que conta código de teste afirma sobre um programa que o artista não corre*,
# e a régua da crate (`ph2d_label_census::cfg_test_modules`) já sabia disto — esta não.
CFG_TEST = re.compile(r'#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{')

def strip_cfg_test(src):
    """Apaga o CORPO de cada `#[cfg(test)] mod … { … }`, preservando as posições de linha."""
    out = src
    while True:
        m = CFG_TEST.search(out)
        if not m:
            return out
        i = m.end() - 1  # a `{`
        depth = 0
        instr = False
        esc = False
        j = i
        while j < len(out):
            c = out[j]
            if instr:
                if esc: esc = False
                elif c == '\\': esc = True
                elif c == '"': instr = False
            elif c == '"': instr = True
            elif c == '{': depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0:
                    break
            j += 1
        bloco = out[m.start():j + 1]
        # ⚠️ As QUEBRAS DE LINHA ficam: o `-v` imprime números de linha, e apagá-las movia todos os
        #    achados do resto do ficheiro.
        out = out[:m.start()] + re.sub(r'[^\n]', ' ', bloco) + out[j + 1:]

FILES=[]
for r in ROOTS:
    crate=r.split("/")[1]
    for dp,dn,fn in os.walk(r):
        if os.path.basename(dp)=="tests": dn[:]=[]; continue
        for f in sorted(fn):
            if f.endswith(".rs") and not is_test(f):
                p=os.path.join(dp,f)
                FILES.append((crate,p,strip_cfg_test(strip_comments(open(p,encoding='utf8').read()))))

def args_of(src,op):
    depth=0;i=op;instr=False;esc=False;start=None;parts=[]
    while i<len(src):
        c=src[i]
        if instr:
            if esc: esc=False
            elif c=='\\': esc=True
            elif c=='"': instr=False
            i+=1; continue
        if c=='"': instr=True; i+=1; continue
        if c in "([{":
            depth+=1
            if depth==1: start=i+1
        elif c in ")]}":
            depth-=1
            if depth==0: parts.append(src[start:i]); return parts
        elif c==',' and depth==1: parts.append(src[start:i]); start=i+1
        i+=1
    return parts

# --- todas as assinaturas: nome -> [(indice, nome_do_param) dos params &str] ---
SIG=re.compile(r'\bfn\s+(\w+)\s*(?:<[^>]*>)?\s*\(', re.S)
sigs={}
for crate,p,src in FILES:
    for m in SIG.finditer(src):
        name=m.group(1)
        params=args_of(src,m.end()-1)
        slots=[]
        for i,a in enumerate(params):
            mm=re.match(r'\s*(?:mut\s+)?(\w+)\s*:\s*&\s*(?:\'\w+\s+)?str\s*$', a)
            if mm: slots.append((i,mm.group(1)))
        if slots: sigs.setdefault(name,set()).update(slots)

# --- semente: os pintores cujo nome diz que pintam ---
seed={n for n in sigs if n.startswith(("paint_","draw_"))}
carriers={n:{s for s in sigs[n]} for n in seed}

CALL=lambda names: re.compile(r'\b('+"|".join(sorted(names,key=len,reverse=True))+r')\s*\(')
for _ in range(12):
    grew=False
    rx=CALL(carriers)
    for crate,p,src in FILES:
        # onde comeca cada fn deste ficheiro, para saber em que fn cai uma chamada
        bounds=[(m.start(),m.group(1)) for m in SIG.finditer(src)]
        for m in rx.finditer(src):
            callee=m.group(1)
            parts=args_of(src,m.end()-1)
            # em que funcao estamos?
            host=None
            for s,n in bounds:
                if s<m.start(): host=n
                else: break
            if host is None or host not in sigs: continue
            for idx,_pname in carriers[callee]:
                if idx>=len(parts): continue
                a=parts[idx].strip().lstrip('&')
                for hidx,hname in sigs[host]:
                    if a==hname and (hidx,hname) not in carriers.get(host,set()):
                        carriers.setdefault(host,set()).add((hidx,hname)); grew=True
    if not grew: break

# ⛔⛔⛔ **CURA (2026-09-19): uma TABELA de `&str` que desagua num pintor É texto pintado.**
#
# O cabeçalho deste ficheiro declarava a cegueira por escrito (*«literais que chegam por variável,
# por `const`, por tabela de `&str`… continuam a ser texto pintado ⇒ o número é um PISO»*) — e a
# dívida real estava exactamente ali: as cenas da conferência do Motion pintam
# `for (k, word) in ROW_LABELS.iter() { label(g, word, …) }`, e as **treze** palavras daquelas
# tabelas nunca entraram em contagem nenhuma. *Uma cegueira escrita num doc-comment não é uma
# medição — é uma nota que envelhece.*
#
# ⚠️ **O alcance é declarado e curto de propósito:** uma tabela `const`/`static` de literais, lida
# por um `for` no MESMO ficheiro, cujo ligado vai a uma ranhura de texto de um pintor. Não segue
# variável local, `format!` nem tabela que atravesse crate — *um censo que não declara o seu
# alcance não é comparável consigo mesmo*.
TABELA = re.compile(
    r'\b(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+(\w+)\s*:\s*(?:&\s*)?\[\s*&\s*(?:\'\w+\s+)?str\s*(?:;[^\]]*)?\]\s*=\s*&?\s*\[',
    re.S,
)
# `for (i, v) in TABELA.iter()` · `for v in TABELA` · `for v in TABELA.iter()`
LIGA = re.compile(r'\bfor\s+(?:\(\s*\w+\s*,\s*(\w+)\s*\)|(\w+))\s+in\s+(?:\w+::)*(\w+)\s*(?:\.iter\(\))?')

def tabelas_de(src):
    """nome da tabela -> os literais dela, no ficheiro dado."""
    out={}
    for m in TABELA.finditer(src):
        corpo=args_of(src, m.end()-1)
        literais=[]
        for a in corpo:
            g=LIT.match(a)
            if g: literais.append(g.group(1))
        if literais: out[m.group(1)]=literais
    return out

LIT=re.compile(r'^\s*"((?:[^"\\]|\\.)*)"\s*$')
# ⚠️ **Uma CHAVE não é texto** (2026-09-16): desde que as portas do painel Painter passaram a receber
#    a chave e a traduzir lá dentro (`paint_checkbox_row(…, "panel.painter_layers.brush.accumulate", …)`),
#    este censo contava **44** chaves como literais pintados. A regra é a mesma da régua da crate
#    (`ph2d_label_census::lexical::is_language`): tem um ponto e só minúsculas, dígitos, `_` e `.`.
KEY=re.compile(r'^[a-z0-9_]+(\.[a-z0-9_]+)+$')
rx=CALL(carriers)
per=collections.Counter(); rows=[]
# ⛔⛔⛔ **CURA (2026-09-19): «tem uma letra» NÃO é «é uma palavra», e a régua REGISTADA já o
# sabia.** O `ph2d_label_census::lexical::cegueira` pede **duas letras ASCII seguidas** — e sem essa
# lei este censo contava `18` literais do `ph2d-panel-painter-layers` que são os códigos de canal de
# COR (`R Y G C B M W N K`, a tabela `TABS` pintada em dois sítios) e o `X`/`Y`/`x` da paleta de
# comandos e da vitrina. ⚠️ E contava as SETAS `\u{25c0}`/`\u{25b6}` do Audio Editor, porque ele lia
# o ESCAPE do fonte (que tem um `u` e um `b`) em vez do carácter pintado.
#
# ⚠️ *Duas réguas da mesma grandeza que discordam sobre o que é uma PALAVRA não são comparáveis* —
# e era esta que estava a mais.
ESCAPE_U = re.compile(r'\\u\{[0-9a-fA-F]*\}')

def e_palavra(texto):
    u = ESCAPE_U.sub("\u00b7", texto).replace('\\"', '"')
    b = u.encode("utf8", "ignore")
    return any(
        chr(b[i]).isascii() and chr(b[i]).isalpha()
        and chr(b[i + 1]).isascii() and chr(b[i + 1]).isalpha()
        for i in range(len(b) - 1)
    )

def conta(crate,p,linha,via,texto):
    if e_palavra(texto) and not KEY.match(texto):
        rows.append((crate,p,linha,via,texto)); per[crate]+=1; return True
    return False

for crate,p,src in FILES:
    tabs=tabelas_de(src)
    # que identificador foi ligado a que tabela, e onde
    ligado={}
    for m in LIGA.finditer(src):
        nome=m.group(1) or m.group(2); tab=m.group(3)
        if tab in tabs: ligado[nome]=tab
    for m in rx.finditer(src):
        parts=args_of(src,m.end()-1)
        linha=src[:m.start()].count("\n")+1
        for idx,_ in carriers[m.group(1)]:
            if idx>=len(parts): continue
            bruto=parts[idx].strip().lstrip('&')
            g=LIT.match(parts[idx])
            if g:
                if conta(crate,p,linha,m.group(1),g.group(1)): break
            elif bruto in ligado:
                # ⭐ A tabela inteira chega ao ecrã — uma linha por palavra dela.
                achou=False
                for t in tabs[ligado[bruto]]:
                    achou=conta(crate,p,linha,f"{m.group(1)}[{ligado[bruto]}]",t) or achou
                if achou: break
# ⭐⭐⭐ **O CONTROLO POSITIVO das três curas de 2026-09-19** — `--autoteste`.
#
# ⛔⛔ *Uma cura de instrumento sem controlo positivo é uma afirmação sobre o instrumento antigo.*
# As três leis desta jornada são testáveis sem a árvore, e é isso que esta função faz: ela planta um
# fonte sintético com as três formas (e as três NEGATIVAS) e confere o veredito de cada uma.
def autoteste():
    falhas=[]
    def diz(nome,cond):
        if not cond: falhas.append(nome)

    # (1) `#[cfg(test)] mod` EM LINHA some — e o resto do ficheiro FICA.
    fonte='fn a(){ paint_text(x,y,"Vivo",1); }\n#[cfg(test)]\nmod tests {\n  fn b(){ paint_text(x,y,"Morto",1); }\n}\nfn c(){ paint_text(x,y,"Tambem vivo",1); }\n'
    limpo=strip_cfg_test(fonte)
    diz("cfg-test: o de dentro sai", '"Morto"' not in limpo)
    diz("cfg-test: o de fora fica", '"Vivo"' in limpo and '"Tambem vivo"' in limpo)
    diz("cfg-test: as linhas nao se movem", limpo.count("\n")==fonte.count("\n"))

    # (2) uma TABELA de `&str` é lida, e uma de números NÃO.
    tab='pub const ROTULOS: [&str; 2] = ["Alfa", "Beta"];\nconst NUMS: [f32; 2] = [1.0, 2.0];\n'
    t=tabelas_de(tab)
    diz("tabela: le a de &str", t.get("ROTULOS")==["Alfa","Beta"])
    diz("tabela: ignora a de numeros", "NUMS" not in t)
    diz("tabela: liga o `for`", bool(LIGA.search("for (k, word) in ROTULOS.iter() {")))

    # (3) o que é PALAVRA — a mesma lei da régua registada (duas letras ASCII seguidas).
    diz("palavra: prosa", e_palavra("Snap to grid"))
    diz("palavra: com escape", e_palavra("Canonical \\u{00b7} reference"))
    diz("palavra: uma letra NAO", not e_palavra("R"))
    # ⛔⛔ **A metade que DISCRIMINA, e ela nasceu de uma mutação SOBREVIVENTE:** com `"R"` sozinho a
    #    lei da ADJACÊNCIA é inobservável (uma cadeia de um carácter não tem par nenhum para
    #    testar), logo apagar a exigência da segunda letra passava. *Um controlo tem de conter o
    #    fenómeno que ele afirma* — e o fenómeno aqui são duas letras SEPARADAS.
    diz("palavra: duas letras separadas NAO", not e_palavra("a b"))
    diz("palavra: a seta NAO", not e_palavra("\\u{25b6}"))
    diz("palavra: numero NAO", not e_palavra("2026"))

    for f in falhas: print("✗",f)
    print("AUTOTESTE:", "OK" if not falhas else f"{len(falhas)} FALHA(S)")
    sys.exit(1 if falhas else 0)

if "--autoteste" in sys.argv: autoteste()

print("PORTAS DE TEXTO (ponto fixo):",len(carriers))
print(f"\n{'crate':34} {'literais':>8}")
for c,n in per.most_common(): print(f"{c:34} {n:8d}")
print(f"{'TOTAL':34} {sum(per.values()):8d}")
if "-v" in sys.argv:
    for r in rows: print(f"  {r[1]}:{r[2]}\t{r[3]}\t{r[4]}")
