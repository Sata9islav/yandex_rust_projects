use std::io::Cursor;

use ypbank_tools:: {
    compare_transactions,
    convert,
    read_transactions,
    write_transactions,
    Format,
    Transaction,
    TransactionDifference,
    TransactionKind,
};


#[test]
fn parses_csv_successfully() {
    let input = "\
date,category,kind,amount
2026-04-01,salary,income,120000
2026-04-02,rent,expense,40000
";

    let reader = Cursor::new(input.as_bytes());
    let transactions = read_transactions(reader, Format::Csv)
        .expect("CSV should be parsed");

    assert_eq!(transactions.len(), 2);

    assert_eq!(
        transactions[0],
        Transaction::new(
            "2026-04-01",
            "salary",
            TransactionKind::Income,
            120000,
        )
    );
}

#[test]
fn parses_text_successfully() {
    let input = "\
2026-04-01;salary;income;120000
2026-04-02;rent;expense;40000
";

    let reader = Cursor::new(input.as_bytes());
    let transactions = read_transactions(reader, Format::Text)
        .expect("Text should be parsed");

    assert_eq!(transactions.len(), 2);
    assert_eq!(transactions[1].category, "rent");
    assert_eq!(transactions[1].kind, TransactionKind::Expense);
    assert_eq!(transactions[1].amount, 40000);
}

#[test]
fn parses_bin_successfully() {
    let original = vec![
        Transaction::new(
            "2026-04-01",
            "salary",
            TransactionKind::Income,
            120000,
        ),
        Transaction::new(
            "2026-04-02",
            "rent",
            TransactionKind::Expense,
            40000,
        ),
    ];

    let mut binary = Vec::new();

    write_transactions(&mut binary, Format::Bin, &original)
        .expect("Binary data should be written");

    let reader = Cursor::new(binary);
    let parsed = read_transactions(reader, Format::Bin)
        .expect("Binary data should be parsed");

    assert_eq!(parsed, original);
}

#[test]
fn csv_returns_error_for_invalid_row() {
    let input = "\
date,category,kind,amount
2026-04-01,salary,income
";

    let reader = Cursor::new(input.as_bytes());
    let result = read_transactions(reader, Format::Csv);

    assert!(result.is_err());
}

#[test]
fn text_returns_error_for_unknown_kind() {
    let input = "2026-04-01;salary;bonus;120000\n";

    let reader = Cursor::new(input.as_bytes());

    let result = read_transactions(reader, Format::Text);

    assert!(result.is_err());
}

#[test]
fn bin_returns_error_for_invalid_header() {
    let invalid_binary = b"BAD!";

    let reader = Cursor::new(invalid_binary);

    let result = read_transactions(reader, Format::Bin);

    assert!(result.is_err());
}

#[test]
fn converts_csv_to_text() {
    let input = "\
date,category,kind,amount
2026-04-01,salary,income,120000
";

    let reader = Cursor::new(input.as_bytes());
    let mut output = Vec::new();

    convert(
        reader,
        &mut output,
        Format::Csv,
        Format::Text,
    )
    .expect("Conversion should succeed");

    let output_text = String::from_utf8(output)
        .expect("Output should be valid UTF-8");

    assert_eq!(
        output_text,
        "2026-04-01;salary;income;120000\n"
    );
}

#[test]
fn comparer_returns_none_for_equal_transactions() {
    let left = vec![
        Transaction::new(
            "2026-04-01",
            "salary",
            TransactionKind::Income,
            120000,
        ),
    ];

    let right = left.clone();

    let difference = compare_transactions(&left, &right);

    assert_eq!(difference, None);
}

#[test]
fn comparer_reports_first_difference() {
    let left = vec![
        Transaction::new(
            "2026-04-01",
            "salary",
            TransactionKind::Income,
            120000,
        ),
    ];

    let right = vec![
        Transaction::new(
            "2026-04-01",
            "salary",
            TransactionKind::Income,
            125000,
        ),
    ];

    let difference = compare_transactions(&left, &right);

    assert_eq!(
        difference,
        Some(TransactionDifference::FieldMismatch {
            position: 1,
            field: "amount",
            left: "120000".to_string(),
            right: "125000".to_string(),
        })
    );
}