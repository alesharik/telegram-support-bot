use std::ops::{Deref, DerefMut};

pub struct MessageBuilder<T>(T);

impl<T> MessageBuilder<T> {
    pub fn new(t: T) -> MessageBuilder<T> {
        MessageBuilder(t)
    }

    pub fn with<V>(self, opt: Option<V>, fun: fn(V, T) -> T) -> Self {
        if let Some(v) = opt {
            MessageBuilder(fun(v, self.0))
        } else {
            self
        }
    }

    pub fn build(self) -> T {
        self.0
    }
}

impl<T> Deref for MessageBuilder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MessageBuilder<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}