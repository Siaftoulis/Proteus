import os
from PIL import Image, ImageFilter
from collections import deque

src_path = r"C:\Users\PDS_Dev\.gemini\antigravity-ide\brain\df120715-1f16-4a6e-9ffc-537b80ebf279\.user_uploaded\media_1790619253197.png"
assets_dir = r"C:\Users\PDS_Dev\1_Production\Projects\CRM-Builder\project\assets"
affinity_logos_dir = r"C:\Users\PDS_Dev\1_Production\Projects\CRM-Builder\affinity-assets\logos"
web_assets_dir = r"C:\Users\PDS_Dev\1_Production\Projects\CRM-Builder\project\proteus-web\assets"

os.makedirs(assets_dir, exist_ok=True)
os.makedirs(affinity_logos_dir, exist_ok=True)
os.makedirs(web_assets_dir, exist_ok=True)

src = Image.open(src_path).convert("RGBA")
w, h = src.size

# 1. Save original master
src.save(os.path.join(assets_dir, "proteus_emblem_original.png"), "PNG")
src.save(os.path.join(affinity_logos_dir, "proteus_emblem_original.png"), "PNG")

# 2. Flood fill outer cream to create clean transparent isolated emblem
pixels = src.load()

def is_cream(r, g, b):
    # Cream background has high R, G, B with slight warm tone
    return r > 185 and g > 185 and b > 155

visited = set()
q = deque()

for x in range(w):
    q.append((x, 0))
    q.append((x, h - 1))
    visited.add((x, 0))
    visited.add((x, h - 1))

for y in range(h):
    q.append((0, y))
    q.append((w - 1, y))
    visited.add((0, y))
    visited.add((w - 1, y))

outer_cream = set()

while q:
    x, y = q.popleft()
    r, g, b, a = pixels[x, y]
    if is_cream(r, g, b):
        outer_cream.add((x, y))
        for dx, dy in [(-1,0), (1,0), (0,-1), (0,1)]:
            nx, ny = x + dx, y + dy
            if 0 <= nx < w and 0 <= ny < h and (nx, ny) not in visited:
                visited.add((nx, ny))
                nr, ng, nb, _ = pixels[nx, ny]
                if is_cream(nr, ng, nb):
                    q.append((nx, ny))

# Create isolated transparent image
isolated = Image.new("RGBA", (w, h), (0, 0, 0, 0))
iso_pixels = isolated.load()

for y in range(h):
    for x in range(w):
        if (x, y) not in outer_cream:
            iso_pixels[x, y] = pixels[x, y]

# Anti-alias outer fringe: soften boundary pixels adjacent to outer cream
fringe = set()
for x, y in outer_cream:
    for dx, dy in [(-1,0), (1,0), (0,-1), (0,1)]:
        nx, ny = x + dx, y + dy
        if (nx, ny) not in outer_cream and 0 <= nx < w and 0 <= ny < h:
            fringe.add((nx, ny))

for x, y in fringe:
    r, g, b, a = iso_pixels[x, y]
    # If it's near cream color, reduce alpha
    if is_cream(r, g, b):
        iso_pixels[x, y] = (0, 0, 0, 0)
    else:
        # It's teal edge, ensure full opacity
        iso_pixels[x, y] = (r, g, b, 255)

# Save master isolated
isolated.save(os.path.join(assets_dir, "proteus_emblem_isolated.png"), "PNG")
isolated.save(os.path.join(affinity_logos_dir, "proteus_emblem_isolated.png"), "PNG")
isolated.save(os.path.join(web_assets_dir, "proteus_emblem.png"), "PNG")

# 3. Generate multi-resolution icons (Lanczos resampling for extreme crispness)
sizes = [512, 256, 128, 64, 48, 32, 24, 16]
for s in sizes:
    resized = isolated.resize((s, s), Image.Resampling.LANCZOS)
    resized.save(os.path.join(assets_dir, f"proteus_emblem_{s}.png"), "PNG")
    resized.save(os.path.join(affinity_logos_dir, f"proteus_emblem_{s}.png"), "PNG")
    if s in [256, 128, 64, 32]:
        resized.save(os.path.join(web_assets_dir, f"proteus_emblem_{s}.png"), "PNG")

# Save primary pds_logo.png replacement
isolated.resize((256, 256), Image.Resampling.LANCZOS).save(os.path.join(assets_dir, "pds_logo.png"), "PNG")

# Generate favicon.ico
ico_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
isolated.save(os.path.join(web_assets_dir, "favicon.ico"), format="ICO", sizes=ico_sizes)
isolated.save(os.path.join(assets_dir, "favicon.ico"), format="ICO", sizes=ico_sizes)

# Export raw uncompressed RGBA bytes for instant zero-dependency inclusion in Rust via include_bytes!
for s in [64, 128, 256]:
    r_img = isolated.resize((s, s), Image.Resampling.LANCZOS)
    raw_rgba = r_img.tobytes("raw", "RGBA")
    with open(os.path.join(assets_dir, f"proteus_emblem_{s}.rgba"), "wb") as f:
        f.write(raw_rgba)

# Generate SVG vector wrappers
import base64

with open(os.path.join(affinity_logos_dir, "proteus_emblem_isolated.png"), "rb") as f:
    b64_iso = base64.b64encode(f.read()).decode("utf-8")

svg_content = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">
  <title>Proteus Business OS — Official King Proteus Emblem</title>
  <defs>
    <style>
      .proteus-emblem {{ width: 100%; height: 100%; }}
    </style>
  </defs>
  <image href="data:image/png;base64,{b64_iso}" width="1024" height="1024" class="proteus-emblem" />
</svg>'''

with open(os.path.join(affinity_logos_dir, "proteus_king_emblem.svg"), "w", encoding="utf-8") as f:
    f.write(svg_content)

with open(os.path.join(assets_dir, "proteus_king_emblem.svg"), "w", encoding="utf-8") as f:
    f.write(svg_content)

with open(os.path.join(web_assets_dir, "proteus_king_emblem.svg"), "w", encoding="utf-8") as f:
    f.write(svg_content)

print("All raster & SVG logo assets successfully created across all resolutions!")
