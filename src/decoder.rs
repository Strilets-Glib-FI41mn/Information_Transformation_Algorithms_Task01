use std::io;
use crate::errors::DecodeError;
use crate::alphabet::Alphabet;


pub fn decode_using_alphabet<T: Alphabet>(alphabet: T, data: &String) -> Result<(Vec<u8>, Vec<String>, Vec<DecodeError>), io::Error> {
    // if data is not multiple of four bytes, data is invalid
    /*if data.chars().count() % 4 != 0 {
        return Err(io::Error::from(io::ErrorKind::InvalidInput))
    }*/
    let mut comments = vec![];

    let (text_u8, errors) = {

        let mut commentless: Vec<_> = 
        data
            .lines()
            .enumerate()
            .map(|(lines, text)|{
                Ok((lines, text))
            })
            .filter(|val|{
                if let Ok((_, text)) = val{
                    if let Some(first) = text.chars().nth(0) && first == alphabet.comment_char(){
                        comments.push(text.to_string());
                        return false;
                    }
                }
                true
            }).collect();
            let max_uncomment_line = commentless.len();

            for line in 0..max_uncomment_line - 1{
                if let Ok((_, text)) = commentless[line] && let lenth = text.len() && lenth != 76{
                    commentless[line] = Err(DecodeError::IncorrectLength{
                        line,
                        lenth
                    });
                } else if let Ok((_, text)) = commentless[line] && let Some(position) = text.find(alphabet.padding_char()){
                    commentless[line] = Err(DecodeError::IncorrectPadding{
                        line: line,
                        position
                    });
                }
            }
            let filtered = commentless.into_iter()
            .map(|val|{
                if let Ok((line, text)) = val && let Some(position) = text[1..].find(alphabet.comment_char()){
                    return Err(DecodeError::IncorrectInputSymbol{
                        line: line,
                        position,
                        symbol: alphabet.comment_char()
                    })
                }
                val
            })
            .map(|val|{
                if let Ok((line, text)) = val && let v = alphabet.search_incorrect_input_symbols(text) && v.len() > 0{
                    return Err(DecodeError::IncorrectInputSymbol{
                        line,
                        position: v[0].0,
                        symbol: v[0].1
                    })
                }
                val
            })
            ;
            let (bytes, errors): (Vec<Result<_, _>>, Vec<Result<_, _>>) = 
            filtered
            .into_iter()
            
            //.collect::<Vec<_>>()
            .partition(|thing|{
                thing.is_ok()
            });

            let errors = errors.into_iter().map(|a|{a.unwrap_err()}).collect();
            let bytes = 
            bytes.into_iter()
            .map(|s|{
                s.unwrap()
            })
            .flat_map(|(_, text)|{ text.chars()})
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|chunk| original(&alphabet, chunk) )
            .flat_map(stitch)
            .collect();
            (bytes, errors)
        };
    Ok((text_u8, comments, errors))
}


fn original<T: Alphabet>(alphabet: &T, chunk: &[char]) -> Vec<u8> {
    chunk
        .iter()
        .filter(|character| *character != &alphabet.padding_char())
        .map(|character| { 
            alphabet
                .index_for_char(*character)
                .expect("unable to find character in alphabet")
        })
        .collect()
}



fn stitch(bytes: Vec<u8>) -> Vec<u8> {
    let out = match bytes.len() {
        2 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4,
        ],

        3 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4 | bytes[2] >> 2,
            (bytes[2] & 0b00000011) << 6,
        ],

        4 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4 | bytes[2] >> 2,
            (bytes[2] & 0b00000011) << 6 | bytes[3] & 0b00111111,
        ],

        _ => unreachable!()
    };

    out.into_iter().filter(|&x| x > 0).collect()
}