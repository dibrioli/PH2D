import zlib,struct
def png(path,w,h,f):
    raw=b''.join(b'\0'+b''.join(bytes(f(x,y)) for x in range(w)) for y in range(h))
    c=lambda t,d: struct.pack('>I',len(d))+t+d+struct.pack('>I',zlib.crc32(t+d)&0xffffffff)
    open(path,'wb').write(b'\x89PNG\r\n\x1a\n'+c(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+c(b'IDAT',zlib.compress(raw))+c(b'IEND',b''))
png('idle.png',128,32,lambda x,y:(40*(x//32),200,90,255))
png('run.png',288,96,lambda x,y:(220,40*(x//48),40,255))
