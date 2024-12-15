use std::sync::Arc;

pub trait Client {}

impl<C: ?Sized + Client> Client for &C {}

impl<C: ?Sized + Client> Client for &mut C {}

impl<C: ?Sized + Client> Client for Box<C> {}

impl<C: ?Sized + Client> Client for Arc<C> {}
