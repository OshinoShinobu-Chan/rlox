use crate::Error;
use crate::Scopes;
use std::rc::Rc;

pub trait Resolver {
    fn resolve(self: Rc<Self>, scopes: &mut Scopes) -> Result<(), Error>;
}
