use simrs::{Component, ComponentId, Key, QueueId, Scheduler, Simulation, State};
use std::time::Duration;

#[derive(Debug)]
struct Product;

struct Producer {
    outgoing: QueueId<Product>,
    consumer: ComponentId<ConsumerEvent>,
    produced_count: Key<usize>,
}

struct Consumer {
    incoming: QueueId<Product>,
    working_on: Key<Option<Product>>,
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
}

impl Consumer {
    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }
    fn log(&self, product: Product) {
        println!("{:?}", product)
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

fn main() {
    let mut simulation = Simulation::default();
    let queue = simulation.add_queue::<Product>();
    let working_on = simulation.state.insert::<Option<Product>>(None);
    let consumer = simulation.add_component(Consumer {
        incoming: queue,
        working_on,
    });
    let produced_count = simulation.state.insert(0_usize);
    let producer = simulation.add_component(Producer {
        outgoing: queue,
        consumer,
        produced_count,
    });
    simulation.schedule(Duration::new(0, 0), producer, ProducerEvent);
    // simulation.schedule(Duration::new(0, 0), consumer, ProducerEvent);
    // The above would fail with:                         ^^^^^^^^^^^^^ expected enum `ConsumerEvent`, found struct `ProducerEvent`
    simulation.run(|sim| {
        println!("{:?}", sim.scheduler.time());
    });
}
