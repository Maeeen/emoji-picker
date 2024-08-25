pub(crate) struct Volatile<T>(T);

impl<T> Volatile<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub fn get(&self) -> T {
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    pub fn set(&mut self, value: T) {
        unsafe { core::ptr::write_volatile(&mut self.0, value); }
    }
}
