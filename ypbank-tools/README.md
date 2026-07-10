# YPBank Tools

A Rust project for reading, converting, and comparing financial transactions.

## Components

- `src/lib.rs` — lib with basic logic.
- `src/main.rs` — CLI-app converter.
- `src/bin/comparer.rs` — CLI-app comparer.

## Supported formats

### CSV
```text
    date,category,kind,amount
    2026-04-01,salary,income,120000
    2026-04-02,rent,expense,40000
```

### Text

```text
    2026-04-01;salary;income;120000
    2026-04-02;rent;expense;40000
```
### Binary format

The project uses its own binary format to store a list of transactions.

The format is deterministic: all fields are written in a strictly defined order, numbers are stored in little-endian, and strings are stored as UTF-8 bytes with a pre-written length.

#### General file structure

The file begins with a header:

| Field | Size | Description |
|---|---:|---|
| `MAGIC` | 4 bytes | ASCII-string `YPB1`, format identifier |
| `COUNT` | 4 bytes | Number of transactions, `u32` little-endian |

After the header, there are `COUNT` transactions.

#### Structure of a single transaction

Each transaction is recorded in the following order:

| Field | Size | Description |
|---|---:|---|
| `date_len` | 4 bytes | Length of the `date` string in bytes, `u32` little-endian |
| `date_bytes` | `date_len` bytes | Date of the operation in UTF-8 |
| `category_len` | 4 bytes | Length of the `category` string in bytes, `u32` little-endian |
| `category_bytes` | `category_len` bytes | Category of the operation in UTF-8 |
| `kind` | 1 byte | Type


#### Example of a logical record

Transaction:

```text
2026-04-01;salary;income;120000
```


## Start

### Comparer

```bash
    cargo run --bin comparer -- first.csv csv second.txt text
```

### Converter

Start from file:

```bash
    cargo run --bin ypbank-tools -- transactions.csv csv text
```

Start from stdin:

```bash
    cat transactions.csv | cargo run --bin ypbank-tools -- - csv text
```
