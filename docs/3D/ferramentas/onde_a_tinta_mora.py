#!/usr/bin/env python3
"""ONDE A TINTA MORA — o instrumento da pesquisa do doc 27.

    python3 docs/3D/ferramentas/onde_a_tinta_mora.py <peca.obj|.obj.gz>

Ele responde a UMA pergunta, e e' a que o dono julga: **quantas amostras de tinta
por unidade de MUNDO ha' no PIOR sitio da peca**, para cada esquema de
armazenamento, ao mesmo orcamento de memoria.

⚠️ DUAS constantes sao MEDIDAS e duas sao ESTIMADAS, e a diferenca esta' marcada:
  * TINTA_NOSSA / P05_NOSSO  — medidos no nosso atlas (doc 26 §12.5 e §13.3);
  * TINTA_SOTA  / P05_SOTA   — ⚠️ ESTIMADOS da literatura, NAO medidos nesta casa.
    Eles existem para a conclusao nao depender de o nosso atlas ser fraco.

⭐ A linha `mesh colors` conta a fronteira UMA vez (ela e' PARTILHADA); a linha
`per-face` conta-a por face (ela e' DUPLICADA) — e essa e' a diferenca de especie
entre as duas familias, nao um detalhe de contagem.

⛔ O que ele NAO mede: relogio, qualidade de filtragem, e o custo do shader.
"""
import sys, math, gzip
def load(p):
    op=gzip.open if p.endswith('.gz') else open
    V=[];F=[]
    with op(p,'rt') as f:
        for ln in f:
            if ln.startswith('v '):
                q=ln.split(); V.append((float(q[1]),float(q[2]),float(q[3])))
            elif ln.startswith('f '):
                i=[int(t.split('/')[0]) for t in ln.split()[1:]]
                F.append([k-1 if k>0 else len(V)+k for k in i])
    return V,F
def area(V,f):
    a=0.0
    for k in range(1,len(f)-1):
        p0,p1,p2=V[f[0]],V[f[k]],V[f[k+1]]
        u=[p1[i]-p0[i] for i in range(3)]; v=[p2[i]-p0[i] for i in range(3)]
        c=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        a+=0.5*math.sqrt(sum(x*x for x in c))
    return a
TINTA_NOSSA=0.373; P05_NOSSO=0.37      # MEDIDOS (doc 26 §12.5, §13.3)
TINTA_SOTA=0.75;   P05_SOTA=0.70       # ESTIMADOS da literatura — NAO medidos aqui
T=2048; BYTES=16                        # cor+altura+cobertura+material (doc 25 §7)
path=sys.argv[1]
V,F=load(path); A=sum(area(V,f) for f in F)
E=set()
for f in F:
    n=len(f)
    for k in range(n): a,b=f[k],f[(k+1)%n]; E.add((min(a,b),max(a,b)))
print(f"peca {path.split('/')[-1]}: V={len(V)} E={len(E)} F={len(F)} area={A:.2f}\n")
print(f"{'esquema':<34} {'amostras':>10} {'MB':>7} {'dens p50':>9} {'dens PIOR':>10} {'costuras'}")
def li(n,nome,p50,pior,cost):
    print(f"{nome:<34} {n/1e6:>9.2f}M {n*BYTES/1e6:>7.1f} {p50:>9.0f} {pior:>10.0f}   {cost}")
# por-vertice, nas densidades que a malha de escultura tem
for nv,rot in ((len(V),"na malha DESTA peca"),(98306,"no default do modulo"),(1572866,"depois de K,K")):
    d=math.sqrt(nv/A); li(nv,f"por-vertice ({rot})",d,d,"nenhuma")
# atlas
for tinta,p05,rot in ((TINTA_NOSSA,P05_NOSSO,"o NOSSO, medido"),(TINTA_SOTA,P05_SOTA,"SOTA, ESTIMADO")):
    dm=math.sqrt(tinta*T*T/A); li(T*T,f"atlas 2048² ({rot})",dm,p05*dm,"ilhas")
# per-face e mesh colors, com o MESMO orcamento do atlas
k=math.sqrt(T*T/A)
n3=0; interior=0; Rf={}
for idx,f in enumerate(F):
    a=area(V,f); lado=max(1.0,k*math.sqrt(a))
    r=2**max(0,round(math.log2(lado))); n3+=r*r
    R=max(1,r-1); Rf[idx]=R
    interior += (R-1)**2 if len(f)==4 else max(0,(R-1)*(R-2)//2)
li(n3,"per-face (Ptex/Htex)",math.sqrt(n3/A),math.sqrt(n3/A)/math.sqrt(2),"fronteira por face")
Re={}
for idx,f in enumerate(F):
    n=len(f)
    for j in range(n):
        a,b=f[j],f[(j+1)%n]; e=(min(a,b),max(a,b)); Re[e]=max(Re.get(e,0),Rf[idx])
n4=len(V)+sum(R-1 for R in Re.values())+interior
li(n4,"mesh colors (R por face)",math.sqrt(n4/A),math.sqrt(n4/A)/math.sqrt(2),"NENHUMA")
# e a memoria de mesh colors para IGUALAR o pior sitio do nosso atlas
alvo=P05_NOSSO*math.sqrt(TINTA_NOSSA*T*T/A)
N=(alvo*math.sqrt(2))**2*A
print(f"\nmesh colors para IGUALAR o pior sitio do NOSSO atlas: {N/1e6:.2f}M amostras ({T*T/N:.1f}x menos)")
alvo2=P05_SOTA*math.sqrt(TINTA_SOTA*T*T/A)
N2=(alvo2*math.sqrt(2))**2*A
print(f"mesh colors para IGUALAR o pior sitio de um atlas SOTA: {N2/1e6:.2f}M amostras ({T*T/N2:.1f}x menos)")
