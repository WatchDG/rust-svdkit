# svdkit

Генератор Rust-кода из CMSIS-SVD файлов. Создаёт PAC (Peripheral Access Crate) и HAL (Hardware Abstraction Layer) для микроконтроллеров на базе Cortex-M.

## Установка

```bash
cargo build --release
```

## Использование

### Генератор кода `svd2rs`

```bash
svd2rs --svd_file <SVD-файл> --out_dir <выходная_директория> [опции]
```

| Флаг | Описание |
|---|---|
| `--svd_file FILE` | Путь к входному SVD-файлу |
| `--out_dir DIR` | Выходная директория |
| `--pac` | Сгенерировать PAC одним файлом |
| `--hal` | Сгенерировать HAL одним файлом |
| `--rt` | Сгенерировать runtime-файлы (startup, linker script) |
| `--async-rt` | Сгенерировать async runtime |

Пример:

```bash
svd2rs --svd_file tests/svds/nrf52840.svd --out_dir generated/nrf52840
```

### Конвертер `bin2uf2`

Утилита [tools/bin2uf2.py](tools/bin2uf2.py) преобразует `.bin`-файл прошивки в формат [UF2](https://github.com/microsoft/uf2), используемый для прошивки через USB Mass Storage (drag-and-drop).

```bash
python tools/bin2uf2.py --input <входной.bin> --output <выходной.uf2> --base <адрес> [опции]
```

| Параметр | Обязательный | По умолчанию | Описание |
|---|---|---|---|
| `--input` | Да | — | Путь к входному `.bin`-файлу |
| `--output` | Да | — | Путь для выходного `.uf2`-файла |
| `--base` | Да | — | Базовый адрес flash (напр. `0x0000`) |
| `--family` | Нет | `0xADA52840` | Family ID чипа (nRF52840) |
| `--payload` | Нет | `256` | Размер полезной нагрузки блока (1..476) |

Пример для nRF52840 (ARM Cortex-M4, flash по адресу `0x00000000`):

```bash
python tools/bin2uf2.py \
  --input firmware.bin \
  --output firmware.uf2 \
  --base 0x0000 \
  --family 0xADA52840
```

Результат — файл `firmware.uf2`, готовый для копирования на устройство в режиме bootloader.

## Лицензия

MIT
