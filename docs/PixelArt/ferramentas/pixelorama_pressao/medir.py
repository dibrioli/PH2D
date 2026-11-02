import sys, zlib, struct
def png(path):
    d=open(path,'rb').read(); i=8; idat=b''; 
    while i<len(d):
        n,=struct.unpack('>I',d[i:i+4]); t=d[i+4:i+8]; c=d[i+8:i+8+n]; i+=12+n
        if t==b'IHDR': w,h,bd,ct=struct.unpack('>IIBB',c[:10])
        if t==b'IDAT': idat+=c
    raw=zlib.decompress(idat); bpp={6:4,2:3,4:2,0:1}[ct]; st=w*bpp; rows=[]; prev=bytearray(st); p=0
    for y in range(h):
        f=raw[p]; line=bytearray(raw[p+1:p+1+st]); p+=1+st
        for x in range(st):
            a=line[x-bpp] if x>=bpp else 0; b=prev[x]; cc=prev[x-bpp] if x>=bpp else 0
            if f==1: line[x]=(line[x]+a)&255
            elif f==2: line[x]=(line[x]+b)&255
            elif f==3: line[x]=(line[x]+(a+b)//2)&255
            elif f==4:
                pa=abs(b-cc); pb=abs(a-cc); pc=abs(a+b-2*cc)
                pr=a if pa<=pb and pa<=pc else (b if pb<=pc else cc); line[x]=(line[x]+pr)&255
        rows.append(line); prev=line
    alpha=lambda x,y: rows[y][x*bpp+bpp-1] if ct in (6,4) else 255
    return w,h,alpha
for name in sys.argv[2:]:
    w,h,A=png(f"{sys.argv[1]}/{name}.png")
    print("==",name,w,h)
    for i in range(21):
        cx=10+i*20
        xs=[x for x in range(cx-10,cx+10) for y in range(0,40) if A(x,y)>0]
        ys=[y for x in range(cx-10,cx+10) for y in range(0,40) if A(x,y)>0]
        wd=(max(xs)-min(xs)+1) if xs else 0; hd=(max(ys)-min(ys)+1) if ys else 0
        print(f"raw={i/20:.2f} dot w={wd} h={hd} alpha_center={A(cx,20)}")
    col=[]
    for x in range(10,w-10,20):
        hh=sum(1 for y in range(40,80) if A(x,y)>0); col.append((x,hh,A(x,60)))
    print("stroke (x, height, alpha@y60):",col)
