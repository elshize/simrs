use crate::Simulation;
use std::time::Duration;

/// Simulation execution trait.
pub trait Execute {
    /// Executes the simulation until some stopping condition is reached.
    /// The condition is implementation-specific.
    fn execute(self, sim: &mut Simulation);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EndCondition {
    Time(Duration),
    EmptyQueue,
    Steps(usize),
}

/// Executor is used for simple execution of an entire simulation.
///
/// See the crate level documentation for examples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Executor {
    end_condition: EndCondition,
}

impl Executor {
    /// Simulation will end only once there is no available events in the queue.
    #[must_use]
    pub fn unbound() -> Self {
        Self {
            end_condition: EndCondition::EmptyQueue,
        }
    }

    /// Simulation will be run no longer than the given time.
    /// It may terminate early if no events are available.
    #[must_use]
    pub fn timed(time: Duration) -> Self {
        Self {
            end_condition: EndCondition::Time(time),
        }
    }

    /// Simulation will execute exactly this many steps, unless we run out of events.
    #[must_use]
    pub fn steps(steps: usize) -> Self {
        Self {
            end_condition: EndCondition::Steps(steps),
        }
    }

    /// Registers a side effect that is called _after_ each simulation step.
    #[must_use]
    pub fn side_effect<F>(self, func: F) -> ExecutorWithSideEffect<F>
    where
        F: Fn(&Simulation),
    {
        ExecutorWithSideEffect {
            end_condition: self.end_condition,
            side_effect: func,
        }
    }
}

impl Execute for Executor {
    fn execute(self, sim: &mut Simulation) {
        run_with(sim, self.end_condition, |_| {});
    }
}

pub struct ExecutorWithSideEffect<F>
where
    F: Fn(&Simulation),
{
    end_condition: EndCondition,
    side_effect: F,
}

impl<F> Execute for ExecutorWithSideEffect<F>
where
    F: Fn(&Simulation),
{
    fn execute(self, sim: &mut Simulation) {
        run_with(sim, self.end_condition, self.side_effect);
    }
}

fn run_with<F>(sim: &mut Simulation, end_condition: EndCondition, side_effect: F)
where
    F: Fn(&Simulation),
{
    let step_fn = |sim: &mut Simulation| {
        let result = sim.step();
        if result {
            side_effect(sim);
        }
        result
    };
    match end_condition {
        EndCondition::Time(time) => execute_until(sim, time, step_fn),
        EndCondition::EmptyQueue => execute_until_empty(sim, step_fn),
        EndCondition::Steps(steps) => execute_steps(sim, steps, step_fn),
    }
}

fn execute_until_empty<F>(sim: &mut Simulation, step: F)
where
    F: Fn(&mut Simulation) -> bool,
{
    while step(sim) {}
}

fn execute_until<F>(sim: &mut Simulation, time: Duration, step: F)
where
    F: Fn(&mut Simulation) -> bool,
{
    while sim.scheduler.peek().map_or(false, |e| e.time() <= time) {
        step(sim);
    }
}

fn execute_steps<F>(sim: &mut Simulation, steps: usize, step: F)
where
    F: Fn(&mut Simulation) -> bool,
{
    for _ in 0..steps {
        if !step(sim) {
            break;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Component;

    struct TestComponent {
        counter: crate::Key<usize>,
    }

    #[derive(Debug)]
    struct TestEvent;

    impl Component for TestComponent {
        type Event = TestEvent;

        fn process_event(
            &self,
            self_id: crate::ComponentId<Self::Event>,
            _event: &Self::Event,
            scheduler: &mut crate::Scheduler,
            state: &mut crate::State,
        ) {
            let counter = state.get_mut(self.counter).unwrap();
            *counter += 1;
            if *counter < 10 {
                scheduler.schedule(Duration::from_secs(2), self_id, TestEvent);
            }
        }
    }

    #[test]
    fn test_create_executor() {
        assert_eq!(
            Executor::unbound(),
            Executor {
                end_condition: EndCondition::EmptyQueue
            }
        );
        assert_eq!(
            Executor::timed(Duration::default()),
            Executor {
                end_condition: EndCondition::Time(Duration::default())
            }
        );
        assert_eq!(
            Executor::steps(7),
            Executor {
                end_condition: EndCondition::Steps(7)
            }
        );
        // Bonus: satisfy codecov on derive
        assert_eq!(&format!("{:?}", TestEvent), "TestEvent");
    }

    #[test]
    fn test_steps() {
        let mut sim = Simulation::default();
        let counter_key = sim.state.insert(0_usize);
        let component = sim.add_component(TestComponent {
            counter: counter_key,
        });
        sim.schedule(Duration::default(), component, TestEvent);
        Executor::steps(10).execute(&mut sim);
        assert_eq!(sim.state.get(counter_key), Some(&10));
    }

    #[test]
    fn test_steps_stops_before() {
        let mut sim = Simulation::default();
        let counter_key = sim.state.insert(0_usize);
        let component = sim.add_component(TestComponent {
            counter: counter_key,
        });
        sim.schedule(Duration::default(), component, TestEvent);
        // After 10 steps there are no events, so it will not execute all 100
        Executor::steps(100).execute(&mut sim);
        assert_eq!(sim.state.get(counter_key), Some(&10));
    }

    #[test]
    fn test_timed() {
        let mut sim = Simulation::default();
        let counter_key = sim.state.insert(0_usize);
        let component = sim.add_component(TestComponent {
            counter: counter_key,
        });
        sim.schedule(Duration::default(), component, TestEvent);
        Executor::timed(Duration::from_secs(6)).execute(&mut sim);
        assert_eq!(sim.state.get(counter_key), Some(&4));
        assert_eq!(sim.scheduler.clock().time(), Duration::from_secs(6));
    }

    #[test]
    fn test_timed_clock_stops_early() {
        let mut sim = Simulation::default();
        let counter_key = sim.state.insert(0_usize);
        let component = sim.add_component(TestComponent {
            counter: counter_key,
        });
        sim.schedule(Duration::default(), component, TestEvent);
        Executor::timed(Duration::from_secs(5)).execute(&mut sim);
        assert_eq!(sim.state.get(counter_key), Some(&3));
        assert_eq!(sim.scheduler.clock().time(), Duration::from_secs(4));
    }
}
