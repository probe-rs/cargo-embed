use interface::{Command, Register, Registers, Update};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use probe_rs::{Error, MemoryInterface, Session};

use super::updater::{Updater, UpdaterChannel, WebsocketUpdater};

pub fn run(session: Arc<Mutex<Session>>) -> Result<(), Error> {
    let mut watch = Vec::new();
    let mut update_interval: u64 = 5000;

    let mut websockets: UpdaterChannel<Command, Update> =
        WebsocketUpdater::new("localhost:3031").start();

    let mut timestamp = std::time::Instant::now();

    loop {
        let elapsed = timestamp.elapsed();
        if elapsed.as_millis() > update_interval as u128 {
            timestamp = std::time::Instant::now();
            match websockets.rx().try_recv() {
                Ok(command) => {
                    log::info!("Got backend command message: {:?}", command);

                    match command {
                        Command::UpdateInterval(interval) => update_interval = interval as u64,
                        Command::Watch(watch_registers) => {
                            watch = watch_registers
                                .into_iter()
                                .map(|address| Register { address, value: 0 })
                                .collect()
                        }
                    };
                }
                _ => (),
            }

            let mut session = session.lock().unwrap();
            for register in &mut watch {
                let mut core = session.core(0)?;
                register.value = core.read_word_32(register.address)?;
            }

            let _ = websockets.tx().send(Update::Registers(Registers {
                registers: watch.clone(),
            }));

            let remaining = Duration::from_millis(update_interval) - timestamp.elapsed();
            std::thread::sleep(remaining);
        }
    }
}
