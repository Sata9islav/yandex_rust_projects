use crate::{LibraryError, Transaction, TransactionKind};
use std::io::{BufRead, BufReader, Read, Write};

/// Читает транзакции из CSV-формата.
/// Возвращает ошибку при повреждённых данных или ошибке чтения.
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
        let line = line?;

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

/// Записывает транзакции в CSV-формат.
/// Возвращает ошибку при записи данных.
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
