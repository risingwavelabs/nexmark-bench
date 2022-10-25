use crate::generator::{config::FIRST_PERSON_ID, NexmarkGenerator};

use super::Person;
use arcstr::ArcStr;
use rand::{seq::SliceRandom, Rng};
use std::{
    cmp::min,
    mem::{size_of, size_of_val},
};

// Number of yet-to-be-created people and auction ids allowed.
pub const PERSON_ID_LEAD: usize = 10;

static US_STATES: [ArcStr; 6] = [
    arcstr::literal!("AZ"),
    arcstr::literal!("CA"),
    arcstr::literal!("ID"),
    arcstr::literal!("OR"),
    arcstr::literal!("WA"),
    arcstr::literal!("WY"),
];

static US_CITIES: [ArcStr; 10] = [
    arcstr::literal!("Phoenix"),
    arcstr::literal!("Los Angeles"),
    arcstr::literal!("San Francisco"),
    arcstr::literal!("Boise"),
    arcstr::literal!("Portland"),
    arcstr::literal!("Bend"),
    arcstr::literal!("Redmond"),
    arcstr::literal!("Seattle"),
    arcstr::literal!("Kent"),
    arcstr::literal!("Cheyenne"),
];

const FIRST_NAMES: &[&str] = &[
    "Peter", "Paul", "Luke", "John", "Saul", "Vicky", "Kate", "Julie", "Sarah", "Deiter", "Walter",
];

const LAST_NAMES: &[&str] = &[
    "Shultz", "Abrams", "Spencer", "White", "Bartels", "Walton", "Smith", "Jones", "Noris",
];

impl<R: Rng> NexmarkGenerator<R> {
    pub fn next_person(&mut self, next_event_id: u64, timestamp: u64) -> Person {
        let id = self.last_base0_person_id(next_event_id) + FIRST_PERSON_ID as u64;
        let name = self.next_person_name();
        let email_address = self.next_email();
        let credit_card = self.next_credit_card();
        let city = self.next_us_city();
        let state = self.next_us_state();
        let current_size = size_of::<u64>()
            + size_of_val(name.as_str())
            + size_of_val(email_address.as_str())
            + size_of_val(credit_card.as_str())
            + size_of_val(city.as_str())
            + size_of_val(state.as_str());
        Person {
            id,
            name,
            email_address,
            credit_card,
            city,
            state,
            date_time: timestamp,
            extra: self.next_extra(
                current_size,
                self.config.nexmark_config.avg_person_byte_size,
            ),
        }
    }

    pub fn next_base0_person_id(&mut self, event_id: u64) -> u64 {
        let num_people = self.last_base0_person_id(event_id) + 1;
        let active_people = min(
            num_people,
            self.config.nexmark_config.num_active_people as u64,
        );
        let n = self
            .rng
            .gen_range(0..(active_people + PERSON_ID_LEAD as u64));
        num_people - active_people + n
    }

    pub fn last_base0_person_id(&self, event_id: u64) -> u64 {
        let epoch = event_id / self.config.nexmark_config.total_proportion() as u64;
        let mut offset = event_id % self.config.nexmark_config.total_proportion() as u64;

        if offset >= self.config.nexmark_config.person_proportion as u64 {
            offset = self.config.nexmark_config.person_proportion as u64 - 1;
        }
        epoch * self.config.nexmark_config.person_proportion as u64 + offset
    }

    fn next_us_state(&mut self) -> ArcStr {
        US_STATES.choose(&mut self.rng).unwrap().clone()
    }

    fn next_us_city(&mut self) -> ArcStr {
        US_CITIES.choose(&mut self.rng).unwrap().clone()
    }

    fn next_person_name(&mut self) -> ArcStr {
        format!(
            "{} {}",
            FIRST_NAMES.choose(&mut self.rng).unwrap(),
            LAST_NAMES.choose(&mut self.rng).unwrap()
        )
        .into()
    }

    fn next_email(&mut self) -> ArcStr {
        format!("{}@{}.com", self.next_string(7), self.next_string(5)).into()
    }

    fn next_credit_card(&mut self) -> ArcStr {
        format!(
            "{:04} {:04} {:04} {:04}",
            &mut self.rng.gen_range(0..10_000),
            &mut self.rng.gen_range(0..10_000),
            &mut self.rng.gen_range(0..10_000),
            &mut self.rng.gen_range(0..10_000)
        )
        .into()
    }
}
