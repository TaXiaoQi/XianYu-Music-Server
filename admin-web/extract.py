import zipfile, os
zf = zipfile.ZipFile('/tmp/dist.zip')
dist = '/opt/xianyu-music/admin-web/dist'
count = 0
for m in zf.infolist():
    name = m.filename.replace(chr(92), '/')
    if name.startswith('/'):
        name = name[1:]
    if name.endswith('/'):
        continue
    path = os.path.join(dist, name)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'wb') as f:
        f.write(zf.read(m))
    count += 1
zf.close()
print(f'Extracted {count} files')
