import argparse
import math
import struct
from pathlib import Path


UF2_MAGIC_START0 = 0x0A324655
UF2_MAGIC_START1 = 0x9E5D5157
UF2_MAGIC_END = 0x0AB16F30
UF2_FLAG_FAMILY_ID_PRESENT = 0x00002000


def to_int(s: str) -> int:
    s = s.strip().lower()
    return int(s, 0)


def bin_to_uf2(data: bytes, base_addr: int, family_id: int, payload_size: int = 256) -> bytes:
    if payload_size <= 0 or payload_size > 476:
        raise ValueError("payload_size must be in 1..476")

    num_blocks = math.ceil(len(data) / payload_size) if data else 1
    out = bytearray()

    for block_no in range(num_blocks):
        chunk = data[block_no * payload_size : (block_no + 1) * payload_size]
        chunk = chunk.ljust(payload_size, b"\x00")

        header = struct.pack(
            "<IIIIIIII",
            UF2_MAGIC_START0,
            UF2_MAGIC_START1,
            UF2_FLAG_FAMILY_ID_PRESENT,
            base_addr + block_no * payload_size,
            payload_size,
            block_no,
            num_blocks,
            family_id,
        )

        body = chunk.ljust(476, b"\x00")
        block = header + body + struct.pack("<I", UF2_MAGIC_END)

        if len(block) != 512:
            raise RuntimeError("internal error: UF2 block must be 512 bytes")

        out += block

    return bytes(out)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", required=True)
    ap.add_argument("--output", required=True)
    ap.add_argument("--base", required=True, type=to_int)
    ap.add_argument("--family", default="0xADA52840", type=to_int)
    ap.add_argument("--payload", default=256, type=int)
    args = ap.parse_args()

    inp = Path(args.input)
    outp = Path(args.output)

    data = inp.read_bytes()
    uf2 = bin_to_uf2(data, base_addr=args.base, family_id=args.family, payload_size=args.payload)
    outp.parent.mkdir(parents=True, exist_ok=True)
    outp.write_bytes(uf2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
