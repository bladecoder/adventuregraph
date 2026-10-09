#!/usr/bin/env python3
"""Rebuild the static font from cached pinned crate sources, without font allocations.
Run from the repository root after cargo fetch. See core/assets/README.md.
"""
import glob
import re
from pathlib import Path

registry = Path.home() / ".cargo/registry/src"

base=glob.glob(str(registry / '*/embedded-graphics-unicodefonts-0.2.0'))[0]
s=open(base+'/src/mono_6x10_atlas.rs').read(); encoded=re.search(r'from_mapping_str\(8, "(.*?)"\)',s).group(1)
mapping=re.sub(r'\\u\{(.*?)\}',lambda m:chr(int(m[1],16)),encoded).replace('\\0','\0')
chars=[];i=0
while i<len(mapping):
 if mapping[i]=='\0':
  chars.extend(chr(c) for c in range(ord(mapping[i+1]),ord(mapping[i+2])+1));i+=3
 else: chars.append(mapping[i]);i+=1
src=open(glob.glob(str(registry / '*/tui-big-text-0.8.10/src/pixel_size.rs'))[0]).read()
shapes={}
for name,rows in [('QUADRANT_SYMBOLS',2),('SEXANT_SYMBOLS',3),('OCTANT_SYMBOLS',4)]:
 table=re.search(r'const '+name+r'.*?= \[(.*?)\];',src,re.S)[1]
 for mask,c in enumerate(re.findall("'(.)'",table)):
  shapes[c]=(mask,rows)
raw=bytearray(open(base+'/src/raw/mono_6x10.data','rb').read())
for c in shapes:
 if c not in chars: chars.append(c)
needed=((len(chars)+15)//16)*10*12
raw.extend(bytes(max(0,needed-len(raw))))
for c,(mask,rows) in shapes.items():
 idx=chars.index(c)
 for y in range(10):
  for x in range(6):
   bit=(idx//16*10+y)*96+(idx%16*6+x)
   filled=mask & (1<<((y*rows//10)*2+x//3))
   if filled: raw[bit//8]|=1<<(7-bit%8)
   else: raw[bit//8]&=~(1<<(7-bit%8))
open('core/assets/banner-6x10.data','wb').write(raw)
# Keep the original Unicode glyph order, adding only missing banner symbols.
text=''.join(chars)
rust=''.join('\\u{%x}'%ord(c) for c in text)
open('core/src/embedded_font.rs','w').write('''//! Static 6×10 Unicode font, with exact block subdivisions for BigText output.
use embedded_graphics::{geometry::Size, image::ImageRaw, mono_font::{MonoFont, DecorationDimensions, mapping::StrGlyphMapping}};
static MAPPING: StrGlyphMapping = StrGlyphMapping::new("'''+rust+'''", 0);
pub static FONT: MonoFont<'static> = MonoFont {
 image: ImageRaw::new(include_bytes!("../assets/banner-6x10.data"), 96),
 glyph_mapping: &MAPPING,
 character_size: Size::new(6, 10), character_spacing: 0, baseline: 7,
 underline: DecorationDimensions::new(9, 1),
 strikethrough: DecorationDimensions::new(5, 1),
};
''')

open('core/assets/banner-symbols.txt', 'w').write(''.join(shapes))

with open('core/src/embedded_font.rs', 'a') as output:
    output.write('\n#[cfg(test)]\n#[path = "embedded_font_tests.rs"]\nmod tests;\n')
