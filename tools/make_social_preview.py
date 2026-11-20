"""Generates the card shown when someone shares a link to dindim.

Writes docs/social-preview.png at 1280x640, GitHub's size, to be uploaded by
hand in the repository's Settings under "Social preview".

The card is read as a small thumbnail, so it holds the logo, one line set
large and the app itself, cropped, with everything that matters at least 80px
from the edges. The screenshot is docs/screenshot.png, taken by the
Screenshots workflow. Needs Pillow.

    python3 tools/make_social_preview.py
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
W, H = 1280, 640
MARGIN = 80

PANEL = "#1C1B19"
INK = "#F1EEE7"
GOLD = "#F2B300"

LINES = ["Know where", "your money goes."]

# Size of the macOS window shadow around the window in a 2x capture.
SHADOW_LEFT, SHADOW_TOP = 112, 76


def font(name, size):
    return ImageFont.truetype(str(ROOT / "assets" / "fonts" / name), size)


def card():
    canvas = Image.new("RGBA", (W, H), PANEL)

    shot = Image.open(ROOT / "docs" / "screenshot.png").convert("RGBA")
    scale = 1150 / shot.width
    shot = shot.resize((round(shot.width * scale), round(shot.height * scale)), Image.LANCZOS)
    left, top = 610, 150
    canvas.alpha_composite(shot, (left - round(SHADOW_LEFT * scale), top - round(SHADOW_TOP * scale)))

    coin = Image.open(ROOT / "assets" / "icons" / "dindim-256.png").convert("RGBA")
    coin = coin.resize((96, 96), Image.LANCZOS)
    canvas.alpha_composite(coin, (MARGIN, 150))

    draw = ImageDraw.Draw(canvas)
    draw.text((MARGIN + 116, 146), "dindim", font=font("BricolageGrotesque-ExtraBold.ttf", 84), fill=INK)
    y = 330
    for line in LINES:
        draw.text((MARGIN - 3, y), line, font=font("Figtree-Regular.ttf", 58), fill=INK)
        y += 70
    draw.text((MARGIN, 500), "Linux · macOS · Windows", font=font("Figtree-SemiBold.ttf", 26), fill=GOLD)
    return canvas.convert("RGB")


if __name__ == "__main__":
    out = ROOT / "docs" / "social-preview.png"
    card().save(out, optimize=True)
    print(f"wrote {out.relative_to(ROOT)}")
