use std::ops::{Deref, DerefMut};
// Check if the value has been changed since the last use
#[derive(Default)]
pub struct CValue<T> {
    value: T,
    changed: bool,
}

impl<T> CValue<T> {
    pub fn is_changed(&mut self) -> bool {
        let b = self.changed;
        self.changed = false;
        b
    }
}

impl<T> Deref for CValue<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl <T> DerefMut for CValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.value
    }
}