/*! Converts org file
org2mds is a command line tool which parses org-file's top level headings and
their content to separate notes and writes them to separate files in current directory.
org-file's top level headings should be formatted like this:
'* some heading title <yyyy-MM-dd EEE kk:mm>'

Usage:
% org2mds <org-file>
*/
use chrono::prelude::*;
use std::env;
use std::error;
use std::fmt;
use std::fs;
use std::fs::read_to_string;
use std::str::FromStr;

const DATE_FMT: &str = "%Y-%m-%d %a %H:%M";

struct Note {
    title: String,
    date: DateTime<Utc>,
    content: String,
}

impl FromStr for Note {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let all_lines: Vec<&str> = s.split('\n').collect();
        let title_date: Vec<&str> = all_lines[0]
            .trim_end()
            .trim_end_matches('>')
            .split('<')
            .collect();
        if title_date.len() != 2 {
            return Err(format!("Unable to parse header line: {}", all_lines[0]));
        }
        let result = Note {
            title: title_date[0].trim().to_string(),
            date: Utc
                .datetime_from_str(title_date[1], DATE_FMT)
                .map_err(|_| {
                    format!(
                        "unable to parse date from '{}'. Date '{}' not in require format '{}' or date is wrong",
                        all_lines[0], title_date[1], DATE_FMT
                    )
                })?,
            content: all_lines[1..].iter().map(|s| {let mut ss = s.to_string();ss.push_str("\n"); ss}).collect(),
        };
        Ok(result)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# {}\n{}\n\n{}",
            self.title,
            self.date.format(DATE_FMT).to_string(),
            self.content
        )
    }
}

fn parse_notes(fname: &str) -> Result<Vec<Note>, Box<error::Error>> {
    let mut result: Vec<Note> = vec![];
    let mut notes = "\n".to_owned();
    notes.push_str(&read_to_string(fname)?.trim());
    let raw_notes: Vec<&str> = notes.split("\n* ").collect();
    for rnote in raw_notes.iter().skip(1) {
        result.push(rnote.parse::<Note>()?);
    }
    Ok(result)
}

fn create_file(note: &Note) -> Result<(), Box<error::Error>> {
    let mut fname = note.title.replace("/", " ");
    fname.push_str(".md");
    if fs::File::open(&fname).is_err() {
        fs::write(&fname, note.to_string())?;
        Ok(())
    } else {
        Err(format!("cannot save note {}, maybe file already exists", note.title).into())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Usage: {} <org-file>", args[0]);
    } else {
        match parse_notes(&args[1]) {
            Ok(notes) => {
                println!("Parsed {:?} notes:", notes.len());
                for note in notes {
                    println!("'{}' {}", note.title, note.date);
                    match create_file(&note) {
                        Err(e) => {
                            eprintln!("could not create file for note {}: {}", &note.title, &e)
                        }
                        _ => (),
                    }
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
