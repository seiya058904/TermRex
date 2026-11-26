"""Build the embedded 1-bit ink and opaque masks from the vendored 1x atlas (Pillow)."""
from pathlib import Path
from PIL import Image

assets = Path(__file__).resolve().parents[1] / "assets"
image = Image.open(assets / "chromium.png").convert("RGBA")
stride = (image.width + 7) // 8
planes = [bytearray(stride * image.height) for _ in range(2)]
for y in range(image.height):
    for x in range(image.width):
        red, _, _, alpha = image.getpixel((x, y))
        if alpha:
            index = y * stride + x // 8
            planes[0][index] |= 1 << (x % 8)
            if red < 240:
                planes[1][index] |= 1 << (x % 8)
(assets / "chromium.bits").write_bytes(b"".join(planes))

