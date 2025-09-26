# anLocales

`anLocales` &mdash; a cross-platform Rust library for working with locales, similar to `glibc-locales`, but simpler and with C API support.

* Supports **LC\_TIME, LC\_NUMERIC, LC\_MONETARY, LC\_COLLATE, plural rules**
* Date/number formats in `data_format.json`
* Interface strings in `locale.toml`
* Caching in `temp/`
* C API compatible (`.so`/`.dll`)

---

## 📁 Locale Directory Structure

**Linux/macOS:**

```
/usr/share/anlocales/
├─ locales/
│   ├─ ru_RU/
│   │   ├─ data_format.json
│   │   └─ locale.toml
│   └─ en_US/
├─ temp/               # cache
└─ settings.json       # default_locale, fallback_locale
```

**Windows:**

```
C:\ProgramData\anlocales\
├─ locales\
│   ├─ ru_RU\
│   │   ├─ data_format.json
│   │   └─ locale.toml
│   └─ en_US\
├─ temp\               # cache
└─ settings.json       # default_locale, fallback_locale
```

**Example `data_format.json`:**

```json
{
  "LC_TIME": {
    "days": ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
    "months": ["January", "February", "..."],
    "date_fmt": "%Y-%m-%d"
  },
  "LC_NUMERIC": {
    "decimal_point": ".",
    "thousands_sep": ",",
    "grouping": [3]
  },
  "LC_MONETARY": {
    "currency_symbol": "$",
    "int_curr_symbol": "USD",
    "mon_decimal_point": ".",
    "mon_thousands_sep": ",",
    "positive_sign": "",
    "negative_sign": "-",
    "frac_digits": 2,
    "p_cs_precedes": true,
    "n_cs_precedes": true,
    "p_sep_by_space": false,
    "n_sep_by_space": false,
    "p_sign_posn": 1,
    "n_sign_posn": 1
  },
  "LC_COLLATE": {
    "sort_order": "unicode"
  },
  "PLURAL_RULES": "n != 1"
}
```

**Example `locale.toml`:**

```toml
settings = ["Settings"]
exit = ["Exit"]
hello = ["Hello"]
```

---

## ⚡ Quick Start (Rust)

```rust
use anlocales::AnLocales;
use chrono::NaiveDate;

fn main() {
    let mut anlocales = AnLocales::new();
    let ru = anlocales.load_locale("ru_RU");

    println!("{}", ru.t("settings"));  // Settings in Russian

    let today = NaiveDate::from_ymd(2025, 9, 21);
    println!("{}", ru.format_date(today)); // 21.09.2025

    println!("{}", ru.format_money(1234.56)); // ₽1234.56
}
```

---

## 🔗 Using from C

```c
#include "anlocales.h"
#include <stdio.h>

int main() {
    AnLocales* al = anlocales_new();
    Locale* ru = locale_load(al, "ru_RU");

    const char* s = locale_t(ru, "settings");
    printf("%s\n", s);
    locale_free_str((char*)s);

    const char* date = locale_format_date(ru, 2025, 9, 21);
    printf("%s\n", date);
    locale_free_str((char*)date);

    const char* money = locale_format_money(ru, 1234.56);
    printf("%s\n", money);
    locale_free_str((char*)money);

    locale_free(ru);
    anlocales_free(al);
}
```

---

## 🛠 Building

### *nix

```bash
make
# Outputs:
# dist/libanLocales.so  # Linux
# dist/libanLocales.dylib # macOS
```

### Windows

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# Outputs:
# target/x86_64-pc-windows-gnu/release/anLocales.dll
# you can run "make" but if its not working use this method
```

## Installing anLocales Library 

To enable localization in some programs using this library, you need the anLocales library. Follow the instructions for your platform:

### 1. Download or build the library

I think you already build the library, but if not, you can download from artifacts, or follow the instructions from [building guide](#-building)

* After building, you will have one of the compiled libraries (depending on the platform):
  * Linux: `libanLocales.so`
  * macOS: `libanLocales.dylib`
  * Windows: `anLocales.dll`

### 2. Place the library

* Move or copy the library to a folder in your system’s library path.
  Examples:

    * Linux/macOS:

      ```bash
      cp target/release/libanLocales.so /usr/local/lib/
      ```
    * Windows:

        * Copy `anLocales.dll` to a folder in `%PATH%`.

> 💡 Tip: On Linux/macOS, if the library is not found, you may need to update the library path:
>
> ```bash
> export LD_LIBRARY_PATH=/path/to/library:$LD_LIBRARY_PATH  # Linux
> export DYLD_LIBRARY_PATH=/path/to/library:$DYLD_LIBRARY_PATH  # macOS
> ```

### Linking in C

```bash
# Linux
gcc main.c -L. -lanLocales -o main
# Windows
cl main.c anLocales.dll
```

---

## 💡 Features

* Caching loaded locales in `temp/` for fast startup
* Fallback locale if a key is missing
* Extendable LC\_\* structure (add LC\_MESSAGES, LC\_MONETARY, etc.)
* Cross-platform (`.so`/`.dll`)
* Full C API compatible (`anlocales.h`)
