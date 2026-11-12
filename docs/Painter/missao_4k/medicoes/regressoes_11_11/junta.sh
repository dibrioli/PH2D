# uso: junta.sh <dir com b1..bN> → <dir>all/matriz + agregação por tela
D=$1; A=${D}all; rm -rf $A; mkdir -p $A/matriz
for b in $D/b*; do i=${b##*/b}; for f in $b/matriz/*.*; do n=$(basename $f); cp $f $A/matriz/${n/_1_/_${i}_}; done; cat $b/carga.txt >> $A/carga.txt; done
python3 /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/docs/Painter/ferramentas/mede_pen_up_4k.py --matriz $A/matriz $A/agr > /dev/null
python3 - $A/agr/matriz.jsonl <<'PY'
import json,sys,statistics as st,collections as c
d=c.defaultdict(list)
for l in open(sys.argv[1]):
    r=json.loads(l); b=r["rot"].rsplit("_",1)[0]; d[(r["pincel"],r["painel"],r["tela"],b)].append(r)
for k in sorted(d):
    rs=d[k]; out=[]
    for f in ["toque","traco_p50","soltar","ctrlz"]:
        v=[r[f] for r in rs if r.get(f) is not None]
        out.append(f"{f} {min(v):.1f}/{st.median(v):.1f}")
    print(*k, f"n={len(rs)}", " · ".join(out))
PY
