use std::io;

use crate::alphabet::Alphabet;

pub fn decode_using_alphabet<T: Alphabet>(alphabet: T, data: &String) -> Result<(Vec<u8>, Vec<String>), io::Error> {
    // if data is not multiple of four bytes, data is invalid
    /*if data.chars().count() % 4 != 0 {
        return Err(io::Error::from(io::ErrorKind::InvalidInput))
    }*/
    let mut comments:Vec<String> = vec![];

    let result = {
        let commentless = 
        data
            .lines()
            .filter(|line|{
                if let Some(first) = line.chars().nth(0) && first == alphabet.comment_char(){
                    comments.push(line.to_string());
                    return false;
                }
                true
            })
            .map(|line|{
                if line[1..].contains('-'){
                    panic!("Некоректний вхідний символ");
                }
                line
            })
            .collect::<Vec<&str>>();
            for i in 0..commentless.len() - 1{
                if commentless[i].chars().count() != 76{
                    panic!("Некоректна довжина рядку");
                }
            }

            commentless.iter().flat_map(|line|{ line.chars()})
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|chunk| original(&alphabet, chunk) )
            .flat_map(stitch)
            .collect()
        };
    Ok((result, comments))
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