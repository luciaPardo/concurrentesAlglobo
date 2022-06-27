use helpers::alglobo_transaction::Pago;
use std::collections::HashSet;
use std::sync::Condvar;
use std::sync::Mutex;
extern crate csv;

use csv::Reader;
use std::collections::VecDeque;
use std::error::Error;

fn read_from_file(path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    for result in reader.deserialize() {
        let record: Pago = result?;
        println!("{:?}", record);
    }
    Ok(())
}
fn read_from_file_encolar(path: &str, cola: &Cola) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    for result in reader.deserialize() {
        let record: Pago = result?;
        println!("{:?}", record);
        cola.push(record.cliente);
    }
    Ok(())
}
fn main() {
    // If an error occurs print error
    if let Err(e) = read_from_file("pagos.csv") {
        eprintln!("{}", e);
    }
    let cola = Cola::new(6);
    if let Err(e) = read_from_file_encolar("pagos.csv", &cola) {
        eprintln!("{}", e);
    }
    for _ in 0..6 {
        println!("{:?}", cola.pop_front());
    }
}

struct ColaDePagos {
    cola: Cola,
    ids: u32,
}

impl ColaDePagos {
    pub fn new(&self, cola: &Cola) {
        let mut procesados = HashSet::<u32>::new();
        Self::get_ids("./fallidos", &mut procesados);
        Self::get_ids("./procesados", &mut procesados);
        let mut reader = Reader::from_path("./pagos").expect("error leyendo el archivo");
        for result in reader.deserialize() {
            let record: Pago = result.expect("error parseando el archivo");
            if procesados.contains(&record.id) {
                continue;
            } else {
                cola.push(record.id.to_string());
            }
        }
    }
    pub fn get_ids(path: &str, procesados: &mut HashSet<u32>) {
        let mut reader = Reader::from_path(path).expect("error leyendo el archivo");
        for result in reader.deserialize() {
            let record: Pago = result.expect("error parseando el archivo");
            procesados.insert(record.id);
        }
    }
}
struct Cola {
    data: Mutex<VecDeque<String>>,
    cv_lleno: Condvar,
    cv_vacio: Condvar,
    maximo: usize,
}

impl Cola {
    pub fn new(k: usize) -> Self {
        Self {
            data: Mutex::new(VecDeque::new()),
            cv_lleno: Condvar::new(),
            cv_vacio: Condvar::new(),
            maximo: k,
        }
    }
    pub fn push(&self, valor: String) {
        let mut data = self.data.lock().expect("poison");
        while data.len() == self.maximo {
            data = self.cv_lleno.wait(data).expect("poison");
        }
        data.push_back(valor);
        self.cv_vacio.notify_one()
    }
    pub fn pop_front(&self) -> Option<String> {
        let mut data = self.data.lock().expect("poison");
        while data.len() == 0 {
            data = self.cv_vacio.wait(data).expect("poison");
            break;
        }
        data.pop_front()
    }
}

// [P TID L U C H O] [C TID]*/
