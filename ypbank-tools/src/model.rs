use std::fmt;

/// Тип финансовой операции.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionKind {
    /// Доход.
    Income,
    /// Расход
    Expense,
}

impl TransactionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }
}

/// Финансовая транзакция в общем внутреннем формате.
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    /// Дата операции.
    pub date: String,
    /// Категория операции.
    pub category: String,
    /// Тип операции.
    pub kind: TransactionKind,
    /// Сумма операции.
    pub amount: i64,
}


impl Transaction {
    /// Создаёт новую транзакцию.
    pub fn new(date: &str, category: &str, kind: TransactionKind, amount: i64) -> Self {
        Transaction {
            date: date.to_string(),
            category: category.to_string(),
            kind: kind,
            amount: amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionDiffernce {
    LegthMismatch {
        left_len: usize,
        right_len: usize,
    },
    FieldsMismatch {
        position: usize,
        field: &'static str,
        left: String,
        right: String,
    },
}

impl fmt::Display for TransactionDiffernce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegthMismatch {
                left_len,
                right_len,
            } => {
                write!(
                    f,
                    "Different number of transactions: {left_len} != {right_len}"
                )
            }
            Self::FieldsMismatch {
                position,
                field,
                left,
                right,
            } => {
                write!(
                    f,
                    "Transactions differ at position {position}: {field}: {left} != {right}"
                )
            }
        }
    }
}
