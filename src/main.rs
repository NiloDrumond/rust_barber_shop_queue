use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::sleep,
    time::Duration,
};

use queues::*;
use rand::Rng;

struct State {
    queue: Queue<u32>,
}

fn barber(rx: Receiver<u8>, arc_state: Arc<Mutex<State>>) {
    loop {
        rx.recv().unwrap(); // barbeiro dormindo aguardando ser acordado por uma
                            // execucao aleatoria de CortarCabelo na thread principal
        let mut state = arc_state.lock().unwrap();
        // Equivalente a funcao CortarCabelo()
        if let Ok(client) = state.queue.remove() {
            println!(
                "Corte fino pai {:?}; clientes esperando: {:?}",
                client,
                state.queue.size()
            );
        }
        // unlock implicito
    }
}

fn client(index: u32, arc_state: Arc<Mutex<State>>) {
    let mut space_available = false;
    // Equivalente a funcao DesejoCortarCabelo()
    {
        let mut state = arc_state.lock().unwrap();
        let size = state.queue.size();
        if size < 5 {
            state.queue.add(index).unwrap();
            space_available = true;
        }
        // unlock implicito
    }
    if space_available {
        loop {
            sleep(Duration::from_millis(100));
            let state = arc_state.lock().unwrap();
            // Confere se seu cabelo ja foi cortado para encerrar a thread
            match state.queue.peek() {
                Ok(value) => {
                    if value > index {
                        break;
                    }
                }
                Err(_) => {
                    // Erro significa que a fila esta vazia
                    break;
                }
            }
        }
    }
}

fn main() {
    let arc_state = Arc::new(Mutex::new(State { queue: queue![] }));
    let (tx, rx) = mpsc::channel::<u8>();
    let pool = threadpool::ThreadPool::new(20);
    let mut client_index = 1;

    let barber_arc_state = arc_state.clone();
    pool.execute(move || {
        barber(rx, barber_arc_state);
    });

    loop {
        sleep(Duration::from_millis(10));
        let is_barber = rand::thread_rng().gen_range(0..100);
        if is_barber < 20 {
            // Desbloqueia a thread do barbeiro para ela executar a funcao CortarCabelo()
            tx.send(1).unwrap();
        } else {
            // Inicializa a thread do cliente que chamara a funcao DesejoCortarCabelo()
            let arc_state = arc_state.clone();
            pool.execute(move || {
                client(client_index, arc_state);
            });
            client_index += 1;
        }
    }
}
