use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::GameObjectId;

#[derive(Debug, PartialEq, Eq)]
pub enum Signal {
    AppleEaten { apple_id: GameObjectId },
    GamePauseChanged { is_paused: bool },
    GameOver,
}

type SignalSubscriber = Box<dyn FnMut(&Signal)>;

#[derive(Clone)]
pub struct SignalEmitter {
    signals_pending: Rc<RefCell<VecDeque<Signal>>>,
}

impl SignalEmitter {
    /// Queues a signal for the bus's next dispatch.
    pub fn emit(&self, signal: Signal) {
        self.signals_pending.borrow_mut().push_back(signal);
    }
}

pub struct SignalBus {
    signals_pending: Rc<RefCell<VecDeque<Signal>>>,
    signal_subscribers: Vec<SignalSubscriber>,
}

impl SignalBus {
    pub fn new() -> Self {
        Self {
            signals_pending: Rc::new(RefCell::new(VecDeque::new())),
            signal_subscribers: Vec::new(),
        }
    }

    pub fn emitter(&self) -> SignalEmitter {
        SignalEmitter {
            signals_pending: Rc::clone(&self.signals_pending),
        }
    }

    pub fn subscribe(&mut self, subscriber: impl FnMut(&Signal) + 'static) {
        self.signal_subscribers.push(Box::new(subscriber));
    }

    /// Dispatches the current queue in emission order.
    ///
    /// Signals emitted by subscribers remain queued for the next dispatch.
    pub fn dispatch_pending(&mut self) {
        let signals_pending = {
            let mut signals_pending = self.signals_pending.borrow_mut();
            std::mem::take(&mut *signals_pending)
        };

        for signal in signals_pending {
            for subscriber in &mut self.signal_subscribers {
                subscriber(&signal);
            }
        }
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}
