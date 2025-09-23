
pub trait Alphabet {
    fn char_for_index(&self, index: u8) -> Option<char>;
    fn index_for_char(&self, character: char) -> Option<u8>;
    fn padding_char(&self) -> char;
    fn comment_char(&self) -> char;
}
