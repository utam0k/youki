use std::{io::Read, time::Duration};

use anyhow::{bail, Result};
use mio::unix::pipe;
use mio::unix::pipe::{Receiver, Sender};
use mio::{Events, Interest, Poll, Token};

const PARENT: Token = Token(0);
pub struct ParentProcess {
    receiver: Receiver,
    poll: Poll,
}

impl ParentProcess {
    pub fn new() -> Result<(Self, Sender)> {
        let (sender, mut receiver) = pipe::new()?;
        // sender.set_nonblocking(false)?;
        // receiver.set_nonblocking(false)?;
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut receiver, PARENT, Interest::READABLE)?;
        Ok((Self { receiver, poll }, sender))
    }

    pub fn wait_for_child_ready(&mut self) -> Result<i32> {
        let mut events = Events::with_capacity(128);
        // loop {
        self.poll.poll(&mut events, Some(Duration::from_secs(2)))?;
        for event in events.iter() {
            if let PARENT = event.token() {
                // let mut buf = [0; 20];
                eprintln!("buffer start");
                let mut buf = vec![];
                eprintln!("buffer ok");
                // self.receiver.read_to_end(&mut buf)?;
                if let Err(e) = self.receiver.read_to_end(&mut buf) {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // let message = String::from_utf8_lossy(&buf).to_string();
                        // eprintln!("retry because of WouldBlock: {}", message);
                        // bail!("aa")
                    } else {
                        bail!("{:?}", e);
                    }
                }

                log::debug!("receive a message from child: {:?}", buf);
                let message = String::from_utf8_lossy(&buf).to_string();
                // let err_msg = format!("receive unexpected message {:?} in parent process", &message);
                let err_msg = "receive unexpected message in parent process".to_string();
                let mut a = message.split("=");
                let sign = a.next().expect(&err_msg);

                assert!(sign == "ChildReady");
                let pid = a.next().expect(&err_msg).parse::<i32>()?;
                return Ok(pid);

                // match Message::from(u8::from_be_bytes(buf)) {
                //     Message::ChildReady => {
                //         let mut buf = [0; 4];
                //         self.receiver.read_exact(&mut buf)?;
                //         log::debug!("receive a message from child: {:?}", buf);
                //         return Ok(i32::from_be_bytes(buf));
                //     }
                //     msg => bail!("receive unexpected message {:?} in parent process", msg),
                // }
            } else {
                unreachable!()
            }
        }
        // }
        bail!("unexpected message.")
    }
}
