use std::io::{self, stdout, Write};

use crossterm::cursor::{
    MoveUp,
    MoveToColumn,
};

pub fn choose_options<T> (prompt: &str, options: &[(&str, T)]) -> Vec<T> 
    where T: Copy 
{

    if options.is_empty() {
        return vec![];
    }

    let mut options: Vec<(bool, &str, T)> = options.iter().map(|option| (false, option.0, option.1)).collect();
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        println!("{}", prompt);
        let longest_line = options.iter().enumerate().map( |(option_number, option)| { 
            let indicator = if option.0 {"*"} else {" "};
            let line = format!("[{}] {}. {}", indicator, option_number, option.1);
            println!("{}",line);
            line.len()
        })
            .chain(Some(prompt.len()).into_iter())
            .max().unwrap();
        print!("\n{}(0-{}) > ",MoveUp(1), options.len());
        let _ = stdout().flush();
        input.clear();
        let _ = stdin.read_line(&mut input);
        let input = input.strip_suffix("\n").unwrap();
        if input.is_empty() {
            break;
        }
        match input.parse::<usize>() {
            Ok(option) => {
                if let Some(option) = options.get_mut(option) {
                    option.0 ^= true;
                }
            }
            Err(_err) => {}
        }
        let eraser = String::from(" ").repeat(longest_line);
        (0..(options.len()+2)).into_iter().for_each(|_| {
            print!("{}{}{}", MoveUp(1), MoveToColumn(0), eraser);
        });
        print!("{}", MoveToColumn(0));
    }

    options.into_iter().filter_map(|option| { 
        if option.0 {
            Some(option.2)
        } else {
            None
        }
    }).collect()
}
