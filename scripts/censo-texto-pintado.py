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

⚠️ **O que ele NÃO vê**, declarado: literais que chegam por variável, por `const`, por tabela de
`&str` ou por `format!` — todos eles continuam a ser texto pintado. ⇒ *o número que ele imprime é
um PISO.*

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

FILES=[]
for r in ROOTS:
    crate=r.split("/")[1]
    for dp,dn,fn in os.walk(r):
        if os.path.basename(dp)=="tests": dn[:]=[]; continue
        for f in sorted(fn):
            if f.endswith(".rs") and not is_test(f):
                p=os.path.join(dp,f)
                FILES.append((crate,p,strip_comments(open(p,encoding='utf8').read())))

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

LIT=re.compile(r'^\s*"((?:[^"\\]|\\.)*)"\s*$')
rx=CALL(carriers)
per=collections.Counter(); rows=[]
for crate,p,src in FILES:
    for m in rx.finditer(src):
        parts=args_of(src,m.end()-1)
        for idx,_ in carriers[m.group(1)]:
            if idx>=len(parts): continue
            g=LIT.match(parts[idx])
            if g and any(c.isalpha() for c in g.group(1)):
                rows.append((crate,p,src[:m.start()].count("\n")+1,m.group(1),g.group(1)))
                per[crate]+=1
                break
print("PORTAS DE TEXTO (ponto fixo):",len(carriers))
print(f"\n{'crate':34} {'literais':>8}")
for c,n in per.most_common(): print(f"{c:34} {n:8d}")
print(f"{'TOTAL':34} {sum(per.values()):8d}")
if "-v" in sys.argv:
    for r in rows: print(f"  {r[1]}:{r[2]}\t{r[3]}\t{r[4]}")
