extern crate csv;

use csv::{Position, Reader};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

fn string_record_to_vec(record: &csv::StringRecord) -> Vec<String> {
    let mut string_vec= Vec::new();
    for field in record.iter() {
        string_vec.push(String::from(field));
    }
    string_vec
}

pub struct CsvLensReader {
    reader: Reader<File>,
    pub headers: Vec<String>,
    internal: Arc<Mutex<ReaderInternalState>>,
    bg_handle: thread::JoinHandle<()>,
}

impl CsvLensReader {

    pub fn new(filename: &str) -> Self {

        let mut reader = Reader::from_path(filename).unwrap();
        let headers_record = reader.headers().unwrap();
        let headers = string_record_to_vec(headers_record);

        let (m_internal, handle) = ReaderInternalState::init_internal(filename);

        Self {
            reader: reader,
            headers: headers,
            internal: m_internal,
            bg_handle: handle,
        }
    }

    pub fn get_rows(&mut self, rows_from: u64, num_rows: u64) -> csv::Result<Vec<Vec<String>>> {

        let pos = Position::new();
        self.reader.seek(pos)?;

        let records = self.reader.records();
        let mut res = Vec::new();

        let rows_to = rows_from + num_rows;

        for (i, r) in records.enumerate() {
            // TODO: always assume has header
            if i == 0 {
                continue;
            }
            // rows_from is 0-based
            let i = i - 1;
            if i >= rows_from as usize && i < rows_to as usize {
                let string_record = r.unwrap();
                let mut row = Vec::new();
                for field in string_record.iter() {
                    row.push(String::from(field));
                }
                res.push(row);
            }

            if i >= rows_to as usize {
                break;
            }

        }
        Ok(res)
    }

    pub fn get_total_line_numbers(&self) -> Option<u64> {
        let res = (*self.internal.lock().unwrap()).total_line_number;
        res
    }
}

struct ReaderInternalState {
    total_line_number: Option<u64>,
}

impl ReaderInternalState {

    pub fn init_internal(filename: &str) -> (Arc<Mutex<ReaderInternalState>>, JoinHandle<()>) {

        let internal = ReaderInternalState {
            total_line_number: None,
        };

        let m_state = Arc::new(Mutex::new(internal));

        let _m = m_state.clone();
        let _filename = filename.to_string();
        let handle = thread::spawn(move || {
            let mut bg_reader = Reader::from_path(_filename.as_str()).unwrap();

            // TODO: would be faster to count lines as wc without csv parsing
            // and update the actual count later
            let mut n = 0;
            for _ in bg_reader.records() {
                n += 1;
            }
            let mut num = _m.lock().unwrap();
            (*num).total_line_number = Some(n);
        });

        (m_state, handle)
    }

}