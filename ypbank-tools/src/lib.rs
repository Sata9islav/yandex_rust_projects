use ::std::io::{Read, Write};

pub mod error;
pub mod format;
pub mod model;

pub use error::LibraryError;
pub use format::Format;
pub use model::{Transaction, TransactionDiffernce, TransactionKind};

use crate::format::{read_bin, read_csv, read_text, write_bin, write_csv, write_text};

pub fn read_transactions<R: Read>(
    reader: R,
    format: Format,
) -> Result<Vec<Transaction>, LibraryError> {
    match format {
        Format::Csv => read_csv(reader),
        Format::Text => read_text(reader),
        Format::Bin => read_bin(reader),
    }
}

pub fn write_transactions<W: Write>(
    writer: W,
    format: Format,
    transactions: &[Transaction],
) -> Result<(), LibraryError> {
    match format {
        Format::Csv => write_csv(writer, transactions),
        Format::Text => write_text(writer, transactions),
        Format::Bin => write_bin(writer, transactions),
    }
}

pub fn convert<R: Read, W: Write>(
    reader: R,
    writer: W,
    input_format: Format,
    output_format: Format,
) -> Result<(), LibraryError> {
    let transactions = read_transactions(reader, input_format)?;
    write_transactions(writer, output_format, &transactions)?;
    Ok(())
}

pub fn compare_transactions(
    left: &[Transaction],
    right: &[Transaction],
) -> Option<TransactionDiffernce> {
    let min_len = left.len().min(right.len());
    for index in 0..min_len {
        let position = index + 1;
        let left_tx = &left[index];
        let right_tx = &right[index];

        if left_tx.date != right_tx.date {
            return Some(TransactionDiffernce::FieldsMismatch {
                position: position,
                field: "date",
                left: left_tx.date.clone(),
                right: right_tx.date.clone(),
            });
        }

        if left_tx.category != right_tx.category {
            return Some(TransactionDiffernce::FieldsMismatch {
                position: position,
                field: "category",
                left: left_tx.category.clone(),
                right: right_tx.category.clone(),
            });
        }

        if left_tx.kind != right_tx.kind {
            return Some(TransactionDiffernce::FieldsMismatch {
                position: position,
                field: "kind",
                left: left_tx.kind.as_str().to_string(),
                right: right_tx.kind.as_str().to_string(),
            });
        }

        if left_tx.amount != right_tx.amount {
            return Some(TransactionDiffernce::FieldsMismatch {
                position: position,
                field: "amount",
                left: left_tx.amount.to_string(),
                right: right_tx.amount.to_string(),
            });
        }
    }

    if left.len() != right.len() {
        return Some(TransactionDiffernce::LegthMismatch {
            left_len: (left.len()),
            right_len: (right.len()),
        });
    }
    None
}
