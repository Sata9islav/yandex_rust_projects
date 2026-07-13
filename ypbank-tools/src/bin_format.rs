use crate::{LibraryError, Transaction, TransactionKind};
use std::io::{Read, Write};

const MAGIC: &[u8; 4] = b"YPB1";


/// Читает транзакции из BIN-формата.
/// Возвращает ошибку при повреждённых данных или ошибке чтения.
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

/// Читает строку из бинарного потока.
/// Сначала читает длину строки как `u32` в little-endian,
/// затем читает указанное количество байтов и проверяет UTF-8.
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

/// Записывает строку в бинарный поток.
/// Сначала записывает длину строки как `u32` в little-endian,
/// затем записывает UTF-8 байты строки.
fn write_string<W: Write>(writer: &mut W, value: &str) -> Result<(), LibraryError> {
    let bytes = value.as_bytes();
    let len = bytes.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

/// Записывает транзакции в BIN-формат.
/// Возвращает ошибку при записи данных.
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
