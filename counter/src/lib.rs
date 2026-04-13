use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct Counter {
    value: i32,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }

    pub fn decrement(&mut self) -> i32 {
        self.value -= 1;
        self.value
    }

    pub fn reset(&mut self) -> i32 {
        self.value = 0;
        self.value
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_increments() {
        let mut counter = Counter::new();
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
    }

    #[test]
    fn counter_decrements() {
        let mut counter = Counter::new();
        counter.increment();
        assert_eq!(counter.decrement(), 0);
        assert_eq!(counter.decrement(), -1);
    }

    #[test]
    fn counter_resets() {
        let mut counter = Counter::new();
        counter.increment();
        counter.increment();
        assert_eq!(counter.reset(), 0);
        assert_eq!(counter.value(), 0);
    }
}
