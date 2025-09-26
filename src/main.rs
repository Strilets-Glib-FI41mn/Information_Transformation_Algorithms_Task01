
mod alphabet;
mod base64_classic;
mod decoder;
mod errors;

use alphabet::Alphabet;
//use clap::Error;
use clap::Parser;
use serde::Serialize;
use std::{fs::File, io::Write, path::PathBuf};
use dialoguer::Confirm;
use dialoguer::Editor;



/*
#[derive(
    clap::ValueEnum, Clone, Default, Serialize
)]
#[serde(rename_all = "kebab-case")]
enum Encoding{
    #[default]
    BASE64,
    RLE
}*/


#[derive(
    clap::ValueEnum, Clone, Default, Serialize
)]
#[serde(rename_all = "kebab-case")]
enum Mode{
    #[default]
    Encode,
    Decode
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    input_file: PathBuf,
    output_file: Option<String>,
    #[arg(long, short, default_value_t = Mode::Encode, value_enum)]
    mode: Mode,
    //#[arg(long, short, default_value_t = Encoding::BASE64, value_enum)]
    //encoding: Encoding,
    #[arg(long, action)]
    extension_comment: bool,
    #[arg(long, action)]
    name_comment: bool
}


fn split(chunk: &[u8]) -> Vec<u8> {
    match chunk.len() {
        1 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4
        ],

        2 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4,
            (&chunk[1] & 0b00001111) << 2,
        ],

        3 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4,
            (&chunk[1] & 0b00001111) << 2 | &chunk[2] >> 6,
            &chunk[2] & 0b00111111
        ],

        _ => unreachable!()
    }
}

fn encode_base64<T: Alphabet>(data: &[u8], alphabet: &T) -> String{
    let mut index = 4;
    let encoded = data.chunks(3).map(split).flat_map(
        |chunk| encode_chunk(alphabet, &chunk, &mut index)
    );
    String::from_iter(encoded)
}

fn encode_chunk<T: Alphabet>(alphabet: &T, chunk: &Vec<u8>, index: &mut u64) -> Vec<char> {
    let mut out = vec![alphabet.padding_char(); 4];
    for i in 0..chunk.len() {
        if let Some(chr) = alphabet.char_for_index(chunk[i]) {
            out[i] = chr;
        }
        if (*index % 76 == 0) && *index > 0{
            //out.insert(i, '\n');
            out.push('\n');
        }
        *index += 1;
    }
    out
}



fn main() {
    let cli = Cli::parse();
    let path = PathBuf::from(&cli.input_file);
    let mut output_path = path.clone();
    let classic_alphabet = base64_classic::Base64Classic;
    let comment_char = classic_alphabet.comment_char();
    let extension_comment_preffix = format!("{} extension: ", comment_char);
    let name_comment_preffix = format!("{} name: ", comment_char);
    match cli.mode{
        Mode::Encode => {
            match &cli.output_file{
                Some(output_file_path) => output_path = output_file_path.into(), //println!("{}", output),
                None => {
                    output_path = format!("{}.base64", path.to_str().unwrap()).into()
                }
                
            }


            if output_path.exists(){
                let confirmation = Confirm::new()
                .with_prompt("File already exists. Do you want to replace it?")
                .interact().unwrap();
                if !confirmation{
                    println!("Canceled encoding into existing file");
                    return;
                }
            }

            let bytes = std::fs::read(&cli.input_file).unwrap();

            let resulting = encode_base64(&bytes, &base64_classic::Base64Classic);
            let mut file = File::create(output_path).unwrap();
            file.write(resulting.as_bytes()).unwrap();
            if cli.extension_comment && let Some(ext) = cli.input_file.extension() 
            && let Ok(ext_v) = ext.to_os_string().into_string() {
                let comment =format!("\n{}{}", &extension_comment_preffix, ext_v);
                if comment.len() <= 76{
                    file.write(comment.as_bytes()).unwrap();
                }
            }
            if cli.name_comment && let Some(name) = &cli.input_file.file_stem()
            && let Ok(name_v) = name.to_os_string().into_string() 
            {
                let comment =format!("\n{}{}", &name_comment_preffix,  &name_v);
                if comment.len() <= 76{
                    file.write(comment.as_bytes()).unwrap();
                }
            }
        },
        Mode::Decode => {
            match std::fs::exists(&cli.input_file){
                Ok(exists) => {
                    if !exists{
                        println!("No such file or directory");
                        return;
                    }
                },
                Err(err) => {
                    println!("{err}");
                    return;
                },
            }
            
            let file_string = std::fs::read_to_string(&cli.input_file).unwrap();
            let (decoded, comments, errors) = crate::decoder::decode_using_alphabet(base64_classic::Base64Classic, &file_string)
                .expect("Failed");
            for error in &errors{
                println!("{}", error);
            }
            match &cli.output_file{
                Some(output_file_path) =>{
                    output_path = output_file_path.into();
                },
                None => {
                    let mut ext = None;
                    for comment in &comments {
                        if comment.contains(&name_comment_preffix) && let Some(file_name) = comment.split(&name_comment_preffix).nth(1){
                            output_path = file_name.into();
                        }
                        if comment.contains(&extension_comment_preffix) && let Some(extension) 
                        = comment.split(&extension_comment_preffix).nth(1){
                                ext = Some(extension);
                        }
                    }
                    if let Some(ext) = ext{
                        output_path.set_extension(ext);
                    }
                    let output_path_str = output_path.as_os_str().to_os_string().into_string().unwrap();
                    
                    let confirmation = Confirm::new()
                        .with_prompt(format!("Should the name of new file be {}", &output_path_str))
                        .interact()
                        .unwrap();

                    if confirmation {
                        output_path = output_path_str.into();
                        println!("Looks like you want to continue");
                    } else {
                        println!("Change it then");

                        if let Some(rv) = Editor::new().edit(&format!("{}", &output_path_str) ).unwrap() {
                            println!("The file will become:");
                            println!("{}", rv);
                            output_path = rv.into();
                        } else {
                            println!("No name for the output file found! Exiting");
                            return;
                        }
                    }

                }
            }

            if output_path.exists(){
                let confirmation = Confirm::new()
                .with_prompt("File already exists. Do you want to replace it?")
                .interact().unwrap();
                if !confirmation{
                    println!("Canceled decoding into existing file");
                    return;
                }
            }
            
            let mut file = File::create(output_path).unwrap();
            file.write(&decoded).unwrap();
        },
    }

    

}
