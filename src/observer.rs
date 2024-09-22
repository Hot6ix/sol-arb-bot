use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum Event {
    UpdateAccounts
}

pub type Subscriber = fn();

#[derive(Default, Clone)]
pub struct Publisher {
    events: HashMap<Event, Vec<Subscriber>>
}

impl Publisher {
    pub fn subscribe(&mut self, event: Event, listener: Subscriber) {
        self.events.entry(event.clone()).or_default();
        if let Some(events) = self.events.get_mut(&event) {
            events.push(listener);
        }
    }

    pub fn unsubscribe(&mut self, event: Event, listener: Subscriber) {
        self.events.get_mut(&event).unwrap().retain(|&subscriber| {
            subscriber != listener
        })
    }

    pub fn notify(&self, event: Event) {
        if let Some(listeners) = &self.events.get(&event) {
            listeners.iter().for_each(|subscriber| {
                subscriber()
            })
        }
        else {

        }
    }
}