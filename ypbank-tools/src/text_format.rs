use crate::{LibraryError, Transaction, TransactionKind};
use std::io::{BufRead, BufReader, Read, Write};


/// Читает транзакции из TEXT-формата.
/// Возвращает ошибку при повреждённых данных или ошибке чтения.
pub fn read_text<R: Read>(reader: R) -> Result<Vec<Transaction>, LibraryError> {
    let reader = BufReader::new(reader);
    let lines = reader.lines();

    let mut transactions = Vec::new();

    for (index, line) in lines.enumerate() {
        let line_number = index + 1;
        let line = line?;

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

/// Записывает транзакции в TEXT-формат.
/// Возвращает ошибку при записи данных.
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
