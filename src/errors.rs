use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum DecodeError{
    IncorrectInputSymbol{
        line: usize,
        position: usize,
        symbol: char,
    },
    IncorrectLength{
        line: usize,
        lenth: usize
    },
    IncorrectPadding{
        line: usize,
        position: usize
    },
    DataAfterLast
}
impl Display for DecodeError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            DecodeError::IncorrectInputSymbol { line, position, symbol } => {
                write!(f, "Рядок {}, символ {}: Некоректний вхідний символ ('{}')", line, position, symbol)
            },
            DecodeError::IncorrectLength { line, lenth } => {
                write!(f, "Рядок {}: Некоректна довжина рядку {}", line, lenth)
            },
            DecodeError::IncorrectPadding { line, position } => {
                write!(f, "Рядок {}: Некоректна використання паддінгу {}", line, position)
            },
            DecodeError::DataAfterLast  => {
                write!(f, "Наявні дані після кінця повідомлення")
            },
        }
    }
}