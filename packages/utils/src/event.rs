use cosmwasm_std::{Attribute, Event};
use std::string::ToString;

pub struct EventHelper {
    event: Event,
}

impl EventHelper {
    pub fn new(event_name: impl Into<String>) -> EventHelper {
        Self {
            event: Event::new(event_name),
        }
    }

    pub fn check_add_attribute<T: ToString>(
        &mut self,
        check: &Option<T>,
        key: &str,
        value: impl Into<String>,
    ) -> EventHelper {
        if check.is_none() {
            return Self {
                event: self.event.clone(),
            };
        };
        Self {
            event: self.get().add_attribute(key, value),
        }
    }

    pub fn add_attribute(self, key: impl Into<String>, value: impl Into<String>) -> EventHelper {
        Self {
            event: self.get().add_attribute(key, value),
        }
    }

    pub fn add_attributes(self, attributes: Vec<Attribute>) -> EventHelper {
        Self {
            event: self.get().add_attributes(attributes),
        }
    }

    pub fn get(&self) -> Event {
        self.event.clone()
    }
}
