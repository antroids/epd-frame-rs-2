import sys
import struct
from pathlib import Path
from PIL import Image

COLOR_MAP = {
    (0, 0, 0): 0,       # Black
    (255, 255, 255): 1, # White
    (255, 255, 0): 2,   # Yellow
    (255, 0, 0): 3,     # Red
    (0, 0, 255): 5,     # Blue
    (0, 255, 0): 6,     # Green
}

VERSION = 1


def parse_color(color_str):
    parts = list(map(int, color_str.split(",")))
    if len(parts) != 3:
        raise ValueError("Background color must be R,G,B")
    return tuple(parts)


def encode_image(img, bg_color):
    width, height = img.size
    pixels = img.convert("RGB").load()

    encoded_nibbles = []
    transparent_run = 0

    def flush_transparent_run():
        nonlocal transparent_run
        while transparent_run > 0:
            chunk = min(transparent_run, 0x7F)  # 7-bit max (127)

            high = (chunk >> 4) & 0x7
            low = chunk & 0xF

            encoded_nibbles.append(0b1000 | high)
            encoded_nibbles.append(low)

            transparent_run -= chunk

    for y in range(height):
        for x in range(width):
            color = pixels[x, y]
            value = COLOR_MAP.get(color)

            if color == bg_color or value is None:
                transparent_run += 1
                continue

            if transparent_run > 0:
                flush_transparent_run()

            encoded_nibbles.append(value)

    if transparent_run > 0:
        flush_transparent_run()

    return width, height, encoded_nibbles


def pack_nibbles(nibbles):
    output = bytearray()

    for i in range(0, len(nibbles), 2):
        high = nibbles[i]
        low = nibbles[i + 1] if i + 1 < len(nibbles) else 0
        output.append((high << 4) | low)

    return output


def process_file(input_path, output_path, bg_color):
    img = Image.open(input_path).convert("RGB")

    width, height, nibbles = encode_image(img, bg_color)
    pixel_bytes = pack_nibbles(nibbles)

    with open(output_path, "wb") as f:
        f.write(struct.pack("<BHHB", VERSION, width, height, 0))
        f.write(pixel_bytes)

    print(f"[OK] {input_path.name} -> {output_path.name} ({width}x{height})")


def main():
    if len(sys.argv) != 3:
        print("Usage: script.py <directory> R,G,B")
        return

    directory = Path(sys.argv[1])
    bg_color = parse_color(sys.argv[2])

    if not directory.is_dir():
        print("Error: provided path is not a directory")
        return

    bmp_files = list(directory.glob("*.bmp"))

    if not bmp_files:
        print("No BMP files found")
        return

    for bmp_path in bmp_files:
        output_path = bmp_path.with_suffix(".e6spectra")
        process_file(bmp_path, output_path, bg_color)

    print(f"\nDone. Converted {len(bmp_files)} files.")


if __name__ == "__main__":
    main()
