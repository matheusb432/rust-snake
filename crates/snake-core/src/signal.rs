use std::{
    any::{Any, TypeId, type_name},
    cell::Cell,
    collections::{HashMap, HashSet, VecDeque},
    convert::Infallible,
    error::Error,
    fmt,
    marker::PhantomData,
    rc::Rc,
};

struct SignalEnvelope {
    signal_type_id: TypeId,
    signal: Box<dyn Any>,
}

type SignalQueue = Rc<Cell<VecDeque<SignalEnvelope>>>;
type SignalListener<Context, HandlerError> =
    Box<dyn FnMut(&dyn Any, &mut Context) -> Result<(), DispatchSignalError<HandlerError>>>;

pub struct SignalEmitter<Signal> {
    signals_pending: SignalQueue,
    signal_type: PhantomData<fn() -> Signal>,
}

impl<Signal> SignalEmitter<Signal>
where
    Signal: 'static,
{
    /// Queues a signal for the bus's next dispatch.
    pub fn emit(&self, signal: Signal) {
        let mut signals_pending = self.signals_pending.take();
        signals_pending.push_back(SignalEnvelope {
            signal_type_id: TypeId::of::<Signal>(),
            signal: Box::new(signal),
        });
        self.signals_pending.set(signals_pending);
    }
}

impl<Signal> Clone for SignalEmitter<Signal> {
    fn clone(&self) -> Self {
        Self {
            signals_pending: Rc::clone(&self.signals_pending),
            signal_type: PhantomData,
        }
    }
}

impl<Signal> fmt::Debug for SignalEmitter<Signal> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignalEmitter")
            .field("signal_type", &type_name::<Signal>())
            .finish_non_exhaustive()
    }
}

pub struct SignalBusBuilder<Context, HandlerError = Infallible> {
    signals_pending: SignalQueue,
    listeners: HashMap<TypeId, Vec<SignalListener<Context, HandlerError>>>,
    registered_signals: HashSet<TypeId>,
}

impl<Context, HandlerError> SignalBusBuilder<Context, HandlerError> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            signals_pending: Rc::new(Cell::new(VecDeque::new())),
            listeners: HashMap::new(),
            registered_signals: HashSet::new(),
        }
    }

    pub fn register<Signal>(&mut self) -> Result<SignalEmitter<Signal>, RegisterSignalError>
    where
        Signal: 'static,
    {
        let signal_type_id = TypeId::of::<Signal>();
        if !self.registered_signals.insert(signal_type_id) {
            return Err(RegisterSignalError {
                signal_type: type_name::<Signal>(),
            });
        }

        Ok(SignalEmitter {
            signals_pending: Rc::clone(&self.signals_pending),
            signal_type: PhantomData,
        })
    }

    /// best-effort attempt to listen on signal type
    pub fn on<Signal>(&mut self, mut listener: impl FnMut(&Signal, &mut Context) + 'static)
    where
        Signal: 'static,
    {
        let _ = self.try_on(move |signal, context| {
            listener(signal, context);
            Ok(())
        });
    }

    /// listens on signal type, fails if no emitter for it is already registered.
    pub fn try_on<Signal>(
        &mut self,
        mut listener: impl FnMut(&Signal, &mut Context) -> Result<(), HandlerError> + 'static,
    ) -> Result<(), ListenSignalError>
    where
        Signal: 'static,
    {
        let signal_type_id = TypeId::of::<Signal>();
        if !self.registered_signals.contains(&signal_type_id) {
            return Err(ListenSignalError {
                signal_type: type_name::<Signal>(),
            });
        }

        let subscriber = move |signal: &dyn Any, context: &mut Context| {
            let signal = signal.downcast_ref::<Signal>().ok_or(
                DispatchSignalError::UnexpectedSignalType {
                    expected: type_name::<Signal>(),
                },
            )?;
            listener(signal, context).map_err(DispatchSignalError::Handler)
        };
        self.listeners
            .entry(signal_type_id)
            .or_default()
            .push(Box::new(subscriber));
        Ok(())
    }

    #[must_use]
    pub fn build(self) -> SignalBus<Context, HandlerError> {
        SignalBus {
            signals_pending: self.signals_pending,
            signal_subscribers: self.listeners,
        }
    }
}

impl<Context, HandlerError> Default for SignalBusBuilder<Context, HandlerError> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SignalBus<Context, HandlerError = Infallible> {
    signals_pending: SignalQueue,
    signal_subscribers: HashMap<TypeId, Vec<SignalListener<Context, HandlerError>>>,
}

impl<Context, HandlerError> SignalBus<Context, HandlerError> {
    /// Dispatches the current queue in emission order.
    ///
    /// Signals emitted by subscribers remain queued for the next dispatch.
    pub fn dispatch_pending(
        &mut self,
        context: &mut Context,
    ) -> Result<(), DispatchSignalError<HandlerError>> {
        let signals_pending = self.signals_pending.take();

        for signal in signals_pending {
            let Some(subscribers) = self.signal_subscribers.get_mut(&signal.signal_type_id) else {
                continue;
            };
            for subscriber in subscribers {
                subscriber(signal.signal.as_ref(), context)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterSignalError {
    signal_type: &'static str,
}

impl fmt::Display for RegisterSignalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "signal {} is already registered",
            self.signal_type
        )
    }
}

impl Error for RegisterSignalError {}

#[derive(Debug, PartialEq, Eq)]
pub struct ListenSignalError {
    signal_type: &'static str,
}

impl fmt::Display for ListenSignalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "signal {} is not registered", self.signal_type)
    }
}

impl Error for ListenSignalError {}

#[derive(Debug, PartialEq, Eq)]
pub enum DispatchSignalError<HandlerError> {
    UnexpectedSignalType { expected: &'static str },
    Handler(HandlerError),
}

impl<HandlerError> fmt::Display for DispatchSignalError<HandlerError>
where
    HandlerError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedSignalType { expected } => {
                write!(formatter, "signal payload is not a {expected}")
            }
            Self::Handler(error) => error.fmt(formatter),
        }
    }
}

impl<HandlerError> Error for DispatchSignalError<HandlerError>
where
    HandlerError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedSignalType { .. } => None,
            Self::Handler(error) => Some(error),
        }
    }
}
