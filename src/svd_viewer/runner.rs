use interface::{Command, Register, Registers, Update};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use probe_rs::{Error, MemoryInterface, Session};

use super::updater::{Updater, UpdaterChannel, WebsocketUpdater};
use crate::config::Config;

pub fn run(session: Arc<Mutex<Session>>, config: &Config) -> Result<(), Error> {
    let mut watch = Vec::new();

    let mut websockets: UpdaterChannel<Command, Update> =
        WebsocketUpdater::new(&config.svd.websocket_server).start();

    let mut timestamp = std::time::Instant::now();

    loop {
        match websockets.rx().try_recv() {
            Ok(command) => {
                log::info!("Got backend command message: {:?}", command);

                match command {
                    Command::Watch(watch_registers) => {
                        watch = watch_registers
                            .into_iter()
                            .map(|address| Register { address, value: 0 })
                            .collect()
                    }
                    Command::SetRegister(register) => {
                        let mut session = session.lock().unwrap();

                        let mut core = session.core(0)?;
                        core.write_word_32(register.address, register.value)?;
                    }
                    Command::Halt => {
                        let mut session = session.lock().unwrap();

                        let mut core = session.core(0)?;
                        core.halt(Duration::from_millis(1000))?;
                        for register in &mut watch {
                            register.value = core.read_word_32(register.address)?;
                        }

                        let _ = websockets.tx().send(Update::Halted);
                        let _ = websockets.tx().send(Update::Registers(Registers {
                            registers: watch.clone(),
                        }));
                    }
                    Command::Run => {
                        let mut session = session.lock().unwrap();

                        let mut core = session.core(0)?;
                        core.run()?;
                        let _ = websockets.tx().send(Update::Running);
                    }
                };
            }
            _ => (),
        }
    }
}
