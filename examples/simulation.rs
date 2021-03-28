use simrs::{Component, ComponentId, Fifo, Key, QueueId, Scheduler, Simulation, State};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug)]
struct Product;

struct Producer {
    outgoing: QueueId<Fifo<Product>>,
    consumer: ComponentId<ConsumerEvent>,
    produced_count: Key<usize>,
    messages: Rc<RefCell<Vec<String>>>,
}

struct Consumer {
    incoming: QueueId<Fifo<Product>>,
    working_on: Key<Option<Product>>,
    messages: Rc<RefCell<Vec<String>>>,
}

#[derive(Debug)]
struct ProducerEvent;

#[derive(Debug)]
enum ConsumerEvent {
    Received,
    Finished,
}

impl Producer {
    fn produce(&self) -> Product {
        Product
    }
    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }
    fn log(&self) {
        self.messages.borrow_mut().push(String::from("Produced"));
    }
}

impl Consumer {
    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }
    fn log(&self, _: Product) {
        self.messages.borrow_mut().push(String::from("Consumed"));
    }
}

impl Component for Producer {
    type Event = ProducerEvent;

    fn process_event(
        &self,
        self_id: ComponentId<ProducerEvent>,
        _event: &ProducerEvent,
        scheduler: &mut Scheduler,
        state: &mut State,
    ) {
        let count = *state.get(self.produced_count).unwrap();
        if count < 10 {
            let _ = state.send(self.outgoing, self.produce());
            self.log();
            scheduler.schedule(self.interval(), self_id, ProducerEvent);
            scheduler.schedule(Duration::default(), self.consumer, ConsumerEvent::Received);
            *state.get_mut(self.produced_count).unwrap() = count + 1;
        }
    }
}

impl Component for Consumer {
    type Event = ConsumerEvent;

    fn process_event(
        &self,
        self_id: ComponentId<ConsumerEvent>,
        event: &ConsumerEvent,
        scheduler: &mut Scheduler,
        state: &mut State,
    ) {
        let busy = state.get(self.working_on).is_some();
        match event {
            ConsumerEvent::Received => {
                if busy {
                    if let Some(product) = state.recv(self.incoming) {
                        if let Some(w) = state.get_mut(self.working_on) {
                            *w = Some(product);
                        }
                        scheduler.schedule(self.interval(), self_id, ConsumerEvent::Finished);
                    }
                }
            }
            ConsumerEvent::Finished => {
                let product = state.get_mut(self.working_on).unwrap().take().unwrap();
                self.log(product);
                if state.len(self.incoming) > 0 {
                    scheduler.schedule(Duration::default(), self_id, ConsumerEvent::Received);
                }
            }
        }
    }
}

const EXPECTED: &str = "Produced
0ns
0ns
Produced
1s
Consumed
1s
1s
1s
Produced
2s
Consumed
2s
2s
2s
Produced
3s
Consumed
3s
3s
3s
Produced
4s
Consumed
4s
4s
4s
Produced
5s
Consumed
5s
5s
5s
Produced
6s
Consumed
6s
6s
6s
Produced
7s
Consumed
7s
7s
7s
Produced
8s
Consumed
8s
8s
8s
Produced
9s
Consumed
9s
9s
9s
10s
Consumed
10s";

fn main() {
    let messages = Rc::new(RefCell::new(Vec::<String>::new()));
    let mut simulation = Simulation::default();
    let queue = simulation.add_queue(Fifo::default());
    let working_on = simulation.state.insert::<Option<Product>>(None);
    let consumer = simulation.add_component(Consumer {
        incoming: queue,
        working_on,
        messages: messages.clone(),
    });
    let produced_count = simulation.state.insert(0_usize);
    let producer = simulation.add_component(Producer {
        outgoing: queue,
        consumer,
        produced_count,
        messages: messages.clone(),
    });
    simulation.schedule(Duration::new(0, 0), producer, ProducerEvent);
    // simulation.schedule(Duration::new(0, 0), consumer, ProducerEvent);
    // The above would fail with:                         ^^^^^^^^^^^^^ expected enum `ConsumerEvent`, found struct `ProducerEvent`
    {
        let messages = messages.clone();
        simulation.run(move |sim| {
            messages
                .borrow_mut()
                .push(format!("{:?}", sim.scheduler.time()));
        });
    }
    assert_eq!(*messages.borrow(), EXPECTED.split('\n').collect::<Vec<_>>());
}
