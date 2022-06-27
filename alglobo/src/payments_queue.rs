use helpers::alglobo_transaction::Pago;
use std::collections::HashSet;
use std::sync::Condvar;
use std::sync::Mutex;
extern crate csv;

use csv::Reader;
use std::collections::VecDeque;

struct ColaDePagos {
    cola: Cola,
}

impl ColaDePagos {
    pub fn new(size: usize) -> Self {
        let cola = Cola::new(size);
        let mut procesados = HashSet::<u32>::new();
        Self::get_ids("./fallidos.csv", &mut procesados);
        Self::get_ids("./procesados.csv", &mut procesados);
        let mut reader = Reader::from_path("./pagos.csv").expect("error leyendo el archivo");
        for result in reader.deserialize() {
            let record: Pago = result.expect("error parseando el archivo");
            if procesados.contains(&record.id) {
                continue;
            } else {
                cola.push(record);
            }
        }
        Self { cola }
    }
    pub fn get_ids(path: &str, procesados: &mut HashSet<u32>) {
        let mut reader = Reader::from_path(path).expect("error leyendo el archivo");
        for result in reader.deserialize() {
            let record: Pago = result.expect("error parseando el archivo");
            procesados.insert(record.id);
        }
    }
    pub fn pop_pago(&self) -> Option<Pago> {
        let pago = self.cola.pop_front();
        pago
    }
}
struct Cola {
    data: Mutex<VecDeque<Pago>>,
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
    pub fn push(&self, valor: Pago) {
        let mut data = self.data.lock().expect("poison");
        while data.len() == self.maximo {
            data = self.cv_lleno.wait(data).expect("poison");
        }
        data.push_back(valor);
        self.cv_vacio.notify_one()
    }
    pub fn pop_front(&self) -> Option<Pago> {
        let mut data = self.data.lock().expect("poison");
        while data.len() == 0 {
            data = self.cv_vacio.wait(data).expect("poison");
            break;
        }
        data.pop_front()
    }
}

// [P TID L U C H O] [C TID]*/
