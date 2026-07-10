use std::io::{BufRead, BufReader, Read, Write};

use crate::{LibraryError, Transaction, TransactionKind};

const MAGIC: &[u8; 4] = b"YPB1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Csv,
    Text,
    Bin,
}

impl Format {
    pub fn parse(value: &str) -> Result<Self, LibraryError> {
        match value.to_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "text" => Ok(Self::Text),
            "bin" => Ok(Self::Bin),
            unknown => Err(LibraryError::FormatError(format!(
                "Unknown format: {unknown}"
            ))),
        }
    }
}

pub fn read_csv<R: Read>(reader: R) -> Result<Vec<Transaction>, LibraryError> {
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| LibraryError::ParseError("CSV header is missing".to_string()))??;

    if header.trim().to_lowercase() != "date,category,kind,amount" {
        return Err(LibraryError::ParseError(format!(
            "Invalid CSV header: {header}"
        )));
    }

    let mut transactions = Vec::new();

    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let line = line?.to_lowercase();

        let parts: Vec<&str> = line.trim().split(',').map(str::trim).collect();

        if parts.len() != 4 {
            return Err(LibraryError::ParseError(format!(
                "Line {line_number}: expected 4 fields, got {}",
                parts.len()
            )));
        }

        let date = parts[0];
        let category = parts[1];

        let kind: TransactionKind = match parts[2] {
            "income" => TransactionKind::Income,
            "expense" => TransactionKind::Expense,
            unknown => {
                return Err(LibraryError::ParseError(format!(
                    "Line {line_number}: unknown operation {unknown}"
                )));
            }
        };

        let amount: i64 = parts[3].parse::<i64>().map_err(|err| {
            LibraryError::ParseError(format!(
                "Line {line_number}: invalid amount '{}': {err}",
                parts[3]
            ))
        })?;

        let transaction = Transaction::new(date, category, kind, amount);
        transactions.push(transaction);
    }
    Ok(transactions)
}

pub fn write_csv<W: Write>(
    mut writer: W,
    transactions: &[Transaction],
) -> Result<(), LibraryError> {
    writeln!(writer, "date,category,kind,amount")?;

    for t in transactions {
        writeln!(
            writer,
            "{},{},{},{}",
            t.date,
            t.category,
            t.kind.as_str(),
            t.amount
        )?;
    }
    Ok(())
}

pub fn read_text<R: Read>(reader: R) -> Result<Vec<Transaction>, LibraryError> {
    let reader = BufReader::new(reader);
    let lines = reader.lines();

    let mut transactions = Vec::new();

    for (index, line) in lines.enumerate() {
        let line_number = index + 1;
        let line = line?.to_lowercase();

        let parts: Vec<&str> = line.trim().split(';').map(str::trim).collect();

        if parts.len() != 4 {
            return Err(LibraryError::ParseError(format!(
                "Line {line_number}: expected 4 fields, got {}",
                parts.len()
            )));
        }

        let date = parts[0];
        let category = parts[1];

        let kind: TransactionKind = match parts[2] {
            "income" => TransactionKind::Income,
            "expense" => TransactionKind::Expense,
            unknown => {
                return Err(LibraryError::ParseError(format!(
                    "Line {line_number}: unknown operation {unknown}"
                )));
            }
        };

        let amount: i64 = parts[3].parse::<i64>().map_err(|err| {
            LibraryError::ParseError(format!(
                "Line {line_number}: invalid amount '{}': {err}",
                parts[3]
            ))
        })?;

        let transaction: Transaction = Transaction::new(date, category, kind, amount);
        transactions.push(transaction);
    }

    Ok(transactions)
}

pub fn write_text<W: Write>(
    mut writer: W,
    transactions: &[Transaction],
) -> Result<(), LibraryError> {
    for t in transactions {
        writeln!(
            writer,
            "{};{};{};{}",
            t.date,
            t.category,
            t.kind.as_str(),
            t.amount
        )?;
    }
    Ok(())
}

pub fn read_bin<R: Read>(mut reader: R) -> Result<Vec<Transaction>, LibraryError> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;

    if &magic != MAGIC {
        return Err(LibraryError::FormatError(
            "Invalid binary header".to_string(),
        ));
    }

    let mut count_bytes = [0u8; 4];
    reader.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes);
    let mut transactions = Vec::new();

    for index in 0..count {
        let date = read_string(&mut reader)?;
        let category = read_string(&mut reader)?;
        let mut kind_bytes = [0u8; 1];
        reader.read_exact(&mut kind_bytes)?;
        let kind = match kind_bytes[0] {
            1 => TransactionKind::Income,
            2 => TransactionKind::Expense,
            unknown => {
                return Err(LibraryError::ParseError(format!(
                    "Transaction {}: unknown kind byte: {}",
                    index + 1,
                    unknown
                )));
            }
        };
        let mut amount_bytes = [0u8; 8];
        reader.read_exact(&mut amount_bytes)?;
        let amount = i64::from_le_bytes(amount_bytes);
        let transaction = Transaction::new(&date, &category, kind, amount);
        transactions.push(transaction);
    }
    Ok(transactions)
}

fn read_string<R: Read>(reader: &mut R) -> Result<String, LibraryError> {
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    let mut bytes = vec![0u8; len];
    reader.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|err| {
        LibraryError::ParseError(format!("Invalid UTF-8 string in binary data: {err}"))
    })
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> Result<(), LibraryError> {
    let bytes = value.as_bytes();
    let len = bytes.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

pub fn write_bin<W: Write>(
    mut writer: W,
    transactions: &[Transaction],
) -> Result<(), LibraryError> {
    writer.write_all(MAGIC)?;
    let count = transactions.len() as u32;
    writer.write_all(&count.to_le_bytes())?;
    for t in transactions {
        write_string(&mut writer, &t.date)?;
        write_string(&mut writer, &t.category)?;

        let kind_byte = match t.kind {
            TransactionKind::Income => 1u8,
            TransactionKind::Expense => 2u8,
        };
        writer.write_all(&[kind_byte])?;
        writer.write_all(&t.amount.to_le_bytes())?;
    }

    Ok(())
}
