use lunatic::{test, AbstractMessage, Mailbox, Process, Request};
use serde::{Deserialize, Serialize};

#[test]
fn abstract_message() {
    #[derive(AbstractMessage, Serialize, Deserialize)]
    pub enum CounterMessage {
        Increment { amount: u32 },
        Decrement { amount: u32 },
        Count(Request<i64>),
    }

    fn counter_process((): (), mailbox: Mailbox<CounterMessage>) {
        // State
        let mut count = 0;

        loop {
            match mailbox.receive() {
                CounterMessage::Increment { amount } => {
                    count += amount as i64;
                }
                CounterMessage::Decrement { amount } => {
                    count -= amount as i64;
                }
                CounterMessage::Count(request) => {
                    request.reply(count);
                }
            }
        }
    }

    let counter = Process::spawn((), counter_process);
    assert_eq!(CounterMessage::count(counter), 0);
    CounterMessage::increment(counter, 5);
    assert_eq!(CounterMessage::count(counter), 5);
    CounterMessage::increment(counter, 20);
    assert_eq!(CounterMessage::count(counter), 25);
    CounterMessage::decrement(counter, 50);
    assert_eq!(CounterMessage::count(counter), -25);
}
